use crate::{
    deser::{deser_field_opt, serialize, DeserializationResult, SerializationResult},
    midi::{self, ControlChangeKind},
    path::VirtualPaths,
    render::{
        self,
        command::{midi_filter::UpdateMidiFilterKind, ResponseCallback},
        midi_filter::{self, MidiFilterUser},
        node::{JsonUpdateKind, RequestKind},
        preset_map::{Preset, PresetMap},
        velocity_map,
    },
};
use fluidlite::Synth;
use serde_json::json;
use std::{
    fmt::Display,
    fs::File,
    mem,
    path::{Path, PathBuf},
    thread::{self, JoinHandle},
};

use super::{update_fields_or_fail, Render};

const DEFAULT_NAME: &str = "FluidliteSynth";
const POLYPHONY: u16 = 64;

type SoundFontLoadRes = (std::sync::Mutex<Synth>, PresetMap, Option<u8>, Option<u8>);
type SoundFontLoadHandle = JoinHandle<Result<SoundFontLoadRes, String>>;

#[derive(Debug)]
pub struct CouldNotInitSynth;

impl Display for CouldNotInitSynth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "Could not init synth - uninitialized parameters.".fmt(f)
    }
}

impl std::error::Error for CouldNotInitSynth {}

pub struct Node {
    name: String,
    enabled: bool,
    midi_filter: midi_filter::MidiFilter,
    synth: Option<std::sync::Mutex<Synth>>,
    last_file: Option<PathBuf>,
    last_virtual_paths: Option<VirtualPaths>,
    last_sample_rate: Option<u32>,
    last_bank: Option<u8>,
    last_preset: Option<u8>,
    preset_map: Option<PresetMap>,
    gain: f32,
    transposition: i8,
    global_transposition: i8,
    velocity_mapping: velocity_map::Kind,
    ignore_global_transposition: bool,
    tmp_lbuf: Vec<f32>,
    tmp_rbuf: Vec<f32>,
    user_presets: Vec<bool>,
    sf_load_handle: Option<SoundFontLoadHandle>,
    sf_load_res_cb: Option<ResponseCallback>,
}

impl Node {
    fn set_name(&mut self, name: &str) -> JsonUpdateKind {
        self.name = name.into();
        update_fields_or_fail(|updates| {
            updates.push(("name".to_owned(), serialize(name)?));
            Ok(())
        })
    }

    fn set_enabled(&mut self, flag: bool) -> JsonUpdateKind {
        self.enabled = flag;
        update_fields_or_fail(|updates| {
            updates.push(("enabled".to_owned(), serialize(flag)?));
            Ok(())
        })
    }

    fn load_file(&mut self, path: &Path, cb: ResponseCallback) {
        self.last_file = Some(path.to_owned());
        if let Ok(()) = self.load_file_non_blocking() {
            self.sf_load_res_cb = Some(cb);
        } else {
            cb(JsonUpdateKind::Failed);
        }
    }

    fn set_gain(&mut self, gain: f32) -> JsonUpdateKind {
        self.gain = gain;
        update_fields_or_fail(|updates| {
            updates.push(("gain".into(), serialize(gain)?));
            Ok(())
        })
    }

    fn set_transposition(&mut self, transposition: i8) -> JsonUpdateKind {
        self.transposition = transposition;
        update_fields_or_fail(|updates| {
            updates.push(("transposition".into(), serialize(transposition)?));
            Ok(())
        })
    }

    fn set_velocity_mapping(&mut self, mapping: velocity_map::Kind) -> JsonUpdateKind {
        self.velocity_mapping = mapping;
        update_fields_or_fail(|updates| {
            updates.push(("velocity_mapping".into(), serialize(mapping)?));
            Ok(())
        })
    }

    fn set_ignore_global_transposition(&mut self, flag: bool) -> JsonUpdateKind {
        self.ignore_global_transposition = flag;
        update_fields_or_fail(|updates| {
            updates.push(("ignore_global_transposition".into(), serialize(flag)?));
            Ok(())
        })
    }

    fn set_preset(&mut self, bank: u8, preset: u8) -> JsonUpdateKind {
        self.last_bank = Some(bank);
        self.last_preset = Some(preset);
        if let Some(synth) = &mut self.synth {
            if let Ok(synth) = synth.get_mut() {
                _ = synth.bank_select(0, bank as u32);
                _ = synth.program_change(0, preset as u32);
            }
            update_fields_or_fail(|updates| {
                updates.push(("bank".into(), serialize(bank)?));
                updates.push(("preset".into(), serialize(preset)?));
                Ok(())
            })
        } else {
            JsonUpdateKind::Failed
        }
    }

    fn update_midi_filter(&mut self, kind: UpdateMidiFilterKind) -> JsonUpdateKind {
        if MidiFilterUser::process_update_request(self, kind).is_ok() {
            update_fields_or_fail(|updates| {
                updates.push(("midi_filter".into(), serialize(&self.midi_filter)?));
                Ok(())
            })
        } else {
            JsonUpdateKind::Failed
        }
    }

    fn set_user_preset(&mut self, preset: usize) -> JsonUpdateKind {
        if preset >= self.user_presets.len() {
            JsonUpdateKind::Failed
        } else {
            self.enabled = self.user_presets[preset];
            update_fields_or_fail(|updates| {
                updates.push(("enabled".into(), serialize(self.enabled)?));
                Ok(())
            })
        }
    }

    fn set_user_preset_enabled(&mut self, preset: usize, flag: bool) -> JsonUpdateKind {
        if preset >= self.user_presets.len() {
            JsonUpdateKind::Failed
        } else {
            self.user_presets[preset] = flag;
            update_fields_or_fail(|updates| {
                updates.push(("user_presets".into(), serialize(&self.user_presets)?));
                Ok(())
            })
        }
    }

    fn process_midi_message(&mut self, message: &midi::Message) {
        use midi::MessageKind as Kind;
        match message.kind {
            Kind::NoteOn { note, velocity } => self.note_on(note, velocity),
            Kind::NoteOff { note, .. } => self.note_off(note),
            Kind::PolyphonicAftertouch { note, pressure } => {
                self.polyphonic_aftertouch(note, pressure);
            }
            Kind::ControlChange { kind, value } => self.control_change(kind, value),
            Kind::ProgramChange { program } => self.program_change(program),
            Kind::ChannelAftertouch { pressure } => self.channel_aftertouch(pressure),
            Kind::PitchWheel { value } => self.pitch_wheel(value),
        }
    }

    fn note_on(&mut self, note: u8, velocity: u8) {
        let note = self.transpose_note(note);
        if let Some(synth) = &mut self.synth {
            if let Ok(synth) = synth.get_mut() {
                _ = synth.note_on(0, note as u32, velocity as u32);
            }
        }
    }

    fn note_off(&mut self, note: u8) {
        let note = self.transpose_note(note);
        if let Some(synth) = &mut self.synth {
            if let Ok(synth) = synth.get_mut() {
                _ = synth.note_off(0, note as u32);
            }
        }
    }

    fn polyphonic_aftertouch(&mut self, note: u8, pressure: u8) {
        if let Some(synth) = &mut self.synth {
            if let Ok(synth) = synth.get_mut() {
                _ = synth.key_pressure(0, note as u32, pressure as u32);
            }
        }
    }

    fn control_change(&mut self, kind: ControlChangeKind, value: u8) {
        if let Some(synth) = &mut self.synth {
            if let Ok(synth) = synth.get_mut() {
                _ = synth.cc(0, kind.as_number() as u32, value as u32);
            }
        }
    }

    fn program_change(&mut self, program: u8) {
        if let Some(synth) = &mut self.synth {
            if let Ok(synth) = synth.get_mut() {
                _ = synth.program_change(0, program as u32);
            }
        }
    }

    fn channel_aftertouch(&mut self, pressure: u8) {
        if let Some(synth) = &mut self.synth {
            if let Ok(synth) = synth.get_mut() {
                _ = synth.channel_pressure(0, pressure as u32);
            }
        }
    }

    fn pitch_wheel(&mut self, value: u16) {
        if let Some(synth) = &mut self.synth {
            if let Ok(synth) = synth.get_mut() {
                _ = synth.pitch_bend(0, value as u32);
            }
        }
    }

    fn load_file_non_blocking(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let (Some(file), Some(vp)) = (&self.last_file, &self.last_virtual_paths) {
            if let Some(file) = vp.translate(file) {
                let mut last_bank = self.last_bank;
                let mut last_preset = self.last_preset;
                let sample_rate = self.last_sample_rate;
                self.sf_load_handle = Some(thread::spawn(
                    move || -> Result<SoundFontLoadRes, String> {
                        let settings = fluidlite::Settings::new().map_err(|e| e.to_string())?;
                        let synth = Synth::new(settings).map_err(|e| e.to_string())?;
                        synth
                            .sfload(file.clone(), true)
                            .map_err(|e| e.to_string())?;
                        let _ = synth.set_polyphony(POLYPHONY as u32);

                        let preset_map = get_preset_map(
                            &rustysynth::SoundFont::new(
                                &mut File::open(file).map_err(|e| e.to_string())?,
                            )
                            .map_err(|e| e.to_string())?,
                        );

                        if let (Some(bank), Some(preset)) = (last_bank, last_preset) {
                            if preset_map.has_preset(bank, preset) {
                            } else if let Some((bank, preset)) = preset_map.first_available_preset()
                            {
                                last_bank = Some(bank);
                                last_preset = Some(preset);
                            } else {
                                last_bank = None;
                                last_preset = None;
                            }
                        } else if let Some((bank, preset)) = preset_map.first_available_preset() {
                            last_bank = Some(bank);
                            last_preset = Some(preset);
                        }
                        if let Some(sample_rate) = sample_rate {
                            synth.set_sample_rate(sample_rate as f32);
                        }
                        Ok((std::sync::Mutex::new(synth), preset_map, last_bank, last_preset))
                    },
                ));
                Ok(())
            } else {
                Err(Box::new(CouldNotInitSynth))
            }
        } else {
            Err(Box::new(CouldNotInitSynth))
        }
    }

    // fn init_synth(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    //     if let (Some(file), Some(sample_rate), Some(vp)) = (
    //         &self.last_file,
    //         self.last_sample_rate,
    //         &self.last_virtual_paths,
    //     ) {
    //         if let Some(file) = vp.translate(file) {
    //             let mut sf2 = File::open(file)?;
    //             let sound_font = Arc::new(SoundFont::new(&mut sf2)?);
    //             let preset_map = get_preset_map(&sound_font);
    //             let settings = SynthesizerSettings::new(sample_rate as i32);
    //             let mut synth = Synthesizer::new(&sound_font, &settings)?;
    //             if let (Some(bank), Some(preset)) = (self.last_bank, self.last_preset) {
    //                 if preset_map.has_preset(bank, preset) {
    //                     synth.process_midi_message(0, 0xB0, 0x00, bank as i32);
    //                     synth.process_midi_message(0, 0xC0, preset as i32, 0x00);
    //                 } else if let Some((bank, preset)) = preset_map.first_available_preset() {
    //                     self.last_bank = Some(bank);
    //                     self.last_preset = Some(preset);
    //                     synth.process_midi_message(0, 0xB0, 0x00, bank as i32);
    //                     synth.process_midi_message(0, 0xC0, preset as i32, 0x00);
    //                 } else {
    //                     self.last_bank = None;
    //                     self.last_preset = None;
    //                 }
    //             } else if let Some((bank, preset)) = preset_map.first_available_preset() {
    //                 self.last_bank = Some(bank);
    //                 self.last_preset = Some(preset);
    //                 synth.process_midi_message(0, 0xB0, 0x00, bank as i32);
    //                 synth.process_midi_message(0, 0xC0, preset as i32, 0x00);
    //             }
    //             self.synth = Some(synth);
    //             self.preset_map = Some(preset_map);
    //             Ok(())
    //         } else {
    //             Err(Box::new(CouldNotInitSynth))
    //         }
    //     } else {
    //         Err(Box::new(CouldNotInitSynth))
    //     }
    // }

    fn resize_buffers(&mut self, min_size: usize) {
        if self.tmp_lbuf.len() < min_size {
            self.tmp_lbuf.resize(min_size, 0.0);
            self.tmp_rbuf.resize(min_size, 0.0);
        }
    }

    fn does_midi_msg_pass(&self, msg: &midi::Message) -> bool {
        if let midi::MessageKind::NoteOn { .. } = msg.kind {
            self.enabled
        } else {
            true
        }
    }

    fn get_total_transposition(&self) -> i8 {
        if self.ignore_global_transposition {
            self.transposition
        } else {
            self.transposition.saturating_add(self.global_transposition)
        }
    }

    fn transpose_note(&self, note: u8) -> u8 {
        (note as i16 + self.get_total_transposition() as i16) as u8
    }

    fn update(&mut self) {
        self.handle_sf_load();
    }

    fn sf_load_finished(&mut self) -> Option<SoundFontLoadHandle> {
        let finished = self
            .sf_load_handle
            .as_ref()
            .map(|h| h.is_finished())
            .unwrap_or(false);

        if finished {
            let mut handle2: Option<SoundFontLoadHandle> = None;
            mem::swap(&mut self.sf_load_handle, &mut handle2);
            handle2
        } else {
            None
        }
    }

    fn handle_sf_load(&mut self) {
        if let Some(handle) = self.sf_load_finished() {
            let res = handle.join();
            if let Ok(Ok(res)) = res {
                self.handle_sf_load_success(res);
            } else {
                self.call_sf_load_cb(JsonUpdateKind::Failed);
            }
        }
    }

    fn handle_sf_load_success(&mut self, res: SoundFontLoadRes) {
        self.synth = Some(res.0);
        self.preset_map = Some(res.1);
        self.last_bank = res.2;
        self.last_preset = res.3;
        if let (Some(synth), Some(bank), Some(preset)) =
            (&mut self.synth, self.last_bank, self.last_preset)
        {
            if let Ok(synth) = synth.get_mut() {
                _ = synth.bank_select(0, bank as u32);
                _ = synth.program_change(0, preset as u32);
            }
        }
        self.call_sf_load_cb(update_fields_or_fail(|updates| {
            updates.push(("loaded_file".to_owned(), serialize(self.last_file.clone())?));
            updates.push(("preset_map".to_owned(), serialize(self.preset_map.clone())?));
            updates.push(("bank".to_owned(), serialize(self.last_bank)?));
            updates.push(("preset".to_owned(), serialize(self.last_preset)?));
            Ok(())
        }));
    }

    fn call_sf_load_cb(&mut self, res: JsonUpdateKind) {
        let mut cb: Option<ResponseCallback> = None;
        mem::swap(&mut self.sf_load_res_cb, &mut cb);
        if let Some(cb) = cb {
            cb(res);
        }
    }
}

impl Default for Node {
    fn default() -> Self {
        Self {
            name: DEFAULT_NAME.into(),
            enabled: true,
            midi_filter: Default::default(),
            synth: None,
            last_file: None,
            last_virtual_paths: None,
            last_sample_rate: None,
            last_bank: None,
            last_preset: None,
            preset_map: None,
            gain: 1.0,
            transposition: 0,
            global_transposition: 0,
            velocity_mapping: velocity_map::Kind::Identity,
            ignore_global_transposition: false,
            tmp_lbuf: vec![],
            tmp_rbuf: vec![],
            user_presets: vec![true; super::NUM_USER_PRESETS],
            sf_load_handle: None,
            sf_load_res_cb: None,
        }
    }
}

impl Clone for Node {
    fn clone(&self) -> Self {
        let mut res = Self {
            name: self.name.clone(),
            enabled: self.enabled,
            midi_filter: self.midi_filter.clone(),
            synth: None,
            last_file: self.last_file.clone(),
            last_virtual_paths: self.last_virtual_paths.clone(),
            last_sample_rate: self.last_sample_rate,
            last_bank: self.last_bank,
            last_preset: self.last_preset,
            preset_map: None,
            gain: self.gain,
            transposition: self.transposition,
            global_transposition: self.global_transposition,
            velocity_mapping: self.velocity_mapping,
            ignore_global_transposition: self.ignore_global_transposition,
            tmp_lbuf: vec![0.0; self.tmp_lbuf.len()],
            tmp_rbuf: vec![0.0; self.tmp_rbuf.len()],
            user_presets: self.user_presets.clone(),
            sf_load_handle: None,
            sf_load_res_cb: None,
        };
        _ = res.load_file_non_blocking();
        res
    }
}

impl Render for Node {
    fn render_additive(&mut self, lbuf: &mut [f32], rbuf: &mut [f32]) {
        self.update();
        let len = usize::min(lbuf.len(), rbuf.len());
        self.resize_buffers(len);
        let tmp_lbuf = &mut self.tmp_lbuf[..len];
        let tmp_rbuf = &mut self.tmp_rbuf[..len];
        if let Some(synth) = &mut self.synth {
            if let Ok(synth) = synth.get_mut() {
                let _ = synth.write((tmp_lbuf, tmp_rbuf));
            }
        }
        let tmp_lbuf = &mut self.tmp_lbuf[..len];
        let tmp_rbuf = &mut self.tmp_rbuf[..len];
        render::amplify_buffer(tmp_lbuf, self.gain);
        render::amplify_buffer(tmp_rbuf, self.gain);
        render::add_buf_to_buf(lbuf, tmp_lbuf);
        render::add_buf_to_buf(rbuf, tmp_rbuf);
    }

    fn reset_rendering(&mut self) {
        if let Some(synth) = &mut self.synth {
            if let Ok(synth) = synth.get_mut() {
                _ = synth.cc(
                    0,
                    midi::ControlChangeKind::AllSoundsOff.as_number() as u32,
                    0,
                );
            }
        }
    }

    fn set_virtual_paths(&mut self, vp: VirtualPaths) {
        self.last_virtual_paths = Some(vp);
    }

    fn set_sample_rate(&mut self, sample_rate: u32) {
        self.last_sample_rate = Some(sample_rate);
        if let Some(synth) = &mut self.synth {
            if let Ok(synth) = synth.get_mut() {
                synth.set_sample_rate(sample_rate as f32);
            }
        }
    }

    fn receive_midi_message(&mut self, message: &midi::Message) {
        if self.midi_filter.does_pass(message) && self.does_midi_msg_pass(message) {
            self.process_midi_message(message);
        }
    }

    fn set_global_transposition(&mut self, transposition: i8) {
        self.global_transposition = transposition;
    }

    fn process_request(&mut self, kind: RequestKind, cb: ResponseCallback) {
        type RK = RequestKind;
        match kind {
            RK::SetName(name) => cb(self.set_name(&name)),
            RK::SetEnabled(flag) => cb(self.set_enabled(flag)),
            RK::LoadFile(path) => self.load_file(&path, cb),
            RK::SetGain(gain) => cb(self.set_gain(gain)),
            RK::SetTransposition(tr) => cb(self.set_transposition(tr)),
            RK::SetVelocityMapping(kind) => cb(self.set_velocity_mapping(kind)),
            RK::SetIgnoreGlobalTransposition(flag) => {
                cb(self.set_ignore_global_transposition(flag))
            }
            RK::SetBankAndPreset(bank, preset) => cb(self.set_preset(bank, preset)),
            RK::UpdateMidiFilter(kind) => cb(self.update_midi_filter(kind)),
            RK::SetUserPreset(preset) => cb(self.set_user_preset(preset)),
            RK::SetUserPresetEnabled(p, f) => cb(self.set_user_preset_enabled(p, f)),
            _ => cb(JsonUpdateKind::Denied),
        };
    }

    fn serialize(&self) -> SerializationResult {
        let result: serde_json::Value = json!({
            "name": serialize(&self.name)?,
            "enabled": serialize(self.enabled)?,
            "midi_filter": serialize(&self.midi_filter)?,
            "gain": serialize(self.gain)?,
            "transposition": serialize(self.transposition)?,
            "global_transposition": serialize(self.global_transposition)?,
            "velocity_mapping": serialize(self.velocity_mapping)?,
            "ignore_global_transposition": serialize(self.ignore_global_transposition)?,
            "loaded_file": serialize(&self.last_file)?,
            "preset_map": serialize(&self.preset_map)?,
            "bank": serialize(self.last_bank)?,
            "preset": serialize(self.last_preset)?,
            "user_presets": serialize(&self.user_presets)?,
        });
        Ok(result)
    }

    fn deserialize(&mut self, source: &serde_json::Value) -> DeserializationResult {
        deser_field_opt(source, "enabled", |v| self.enabled = v)?;
        deser_field_opt(source, "name", |v| self.name = v)?;
        deser_field_opt(source, "midi_filter", |v| self.midi_filter = v)?;
        deser_field_opt(source, "gain", |v| self.gain = v)?;
        deser_field_opt(source, "transposition", |v| self.transposition = v)?;
        deser_field_opt(source, "global_transposition", |v| {
            self.global_transposition = v
        })?;
        deser_field_opt(source, "ignore_global_transposition", |v| {
            self.ignore_global_transposition = v
        })?;
        deser_field_opt(source, "loaded_file", |v| self.last_file = v)?;
        deser_field_opt(source, "bank", |v| self.last_bank = v)?;
        deser_field_opt(source, "preset", |v| self.last_preset = v)?;
        deser_field_opt(source, "user_presets", |v| self.user_presets = v)?;
        Ok(())
    }

    fn clone_node(&self) -> super::RenderPtr {
        Box::new(self.clone())
    }
}

impl MidiFilterUser for Node {
    fn midi_filter_mut(&mut self) -> &mut midi_filter::MidiFilter {
        &mut self.midi_filter
    }
}

fn get_preset_map(sf: &rustysynth::SoundFont) -> PresetMap {
    let mut map = PresetMap::new();

    sf.get_presets().iter().for_each(|p| {
        let mut preset = Preset::new(p.get_name());
        for r in p.get_regions() {
            preset.add_note_range(r.get_key_range_start() as u8, r.get_key_range_end() as u8);
        }
        map.add_preset(
            p.get_bank_number() as u8,
            p.get_patch_number() as u8,
            preset,
        );
    });

    map
}
