use super::Render;
use crate::{
    deser::{deser_field_opt, serialize, DeserializationResult, SerializationResult}, json::{update_fields_or_fail, JsonUpdateKind, JsonUpdater}, midi::{self, ControlChangeKind}, path::VirtualPaths, render::{
        self,
        command::{midi_filter::UpdateMidiFilterKind, ResponseCallback},
        midi_filter::{self, MidiFilterUser},
        node::RequestKind,
        preset_map::{Preset, PresetMap},
        velocity_map,
    }
};
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};
use serde_json::json;
use std::{
    fmt::Display,
    fs::File,
    mem,
    path::{Path, PathBuf},
    sync::Arc,
    thread::{self, JoinHandle},
};

const DEFAULT_NAME: &str = "Rusty Synth";

type SynthInitRes = (Synthesizer, PresetMap, Option<u16>, Option<u8>);
type SynthInitResHandle = JoinHandle<Result<SynthInitRes, String>>;

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
    synth: Option<Synthesizer>,
    last_file: Option<PathBuf>,
    last_virtual_paths: Option<VirtualPaths>,
    last_sample_rate: Option<u32>,
    last_bank: Option<u16>,
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
    synth_init_handle: Option<SynthInitResHandle>,
    synth_init_res_cb: Option<ResponseCallback>,
    last_timestamp: u128,
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
        if let Ok(()) = self.init_synth_non_blocking() {
            self.synth_init_res_cb = Some(cb);
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

    fn set_preset(&mut self, bank: u16, preset: u8) -> JsonUpdateKind {
        self.last_bank = Some(bank);
        self.last_preset = Some(preset);
        if let Some(synth) = &mut self.synth {
            synth.process_midi_message(0, 0xB0, 0x00, bank as i32);
            synth.process_midi_message(0, 0xC0, preset as i32, 0x00);
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
            Kind::PolyphonicAftertouch { .. } => {}
            Kind::ControlChange { kind, value } => self.control_change(kind, value),
            Kind::ProgramChange { .. } => {}
            Kind::ChannelAftertouch { .. } => {}
            Kind::PitchWheel { value } => self.pitch_wheel(value),
        }
    }

    fn note_on(&mut self, note: u8, velocity: u8) {
        let note = self.transpose_note(note);
        if let Some(s) = self.synth.as_mut() {
            s.note_on(0, note as i32, velocity as i32)
        }
    }

    fn note_off(&mut self, note: u8) {
        let note = self.transpose_note(note);
        if let Some(s) = self.synth.as_mut() {
            s.note_off(0, note as i32)
        }
    }

    fn control_change(&mut self, kind: ControlChangeKind, value: u8) {
        if let Some(s) = self.synth.as_mut() {
            s.process_midi_message(0, 0xB0, kind.as_number() as i32, value as i32)
        }
    }

    fn pitch_wheel(&mut self, value: u16) {
        let data1 = (value & 0x7F) | 0x80;
        let data2 = (value >> 7) & 0x7F;
        if let Some(s) = self.synth.as_mut() {
            s.process_midi_message(0, 0xE0, data1 as i32, data2 as i32)
        }
    }

    fn init_synth_non_blocking(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let (Some(file), Some(sample_rate), Some(vp)) = (
            &self.last_file,
            self.last_sample_rate,
            &self.last_virtual_paths,
        ) {
            if let Some(file) = vp.translate(file) {
                let mut last_bank = self.last_bank;
                let mut last_preset = self.last_preset;
                let block_size = self.tmp_lbuf.len();
                self.synth_init_handle =
                    Some(thread::spawn(move || -> Result<SynthInitRes, String> {
                        let mut sf2 = File::open(file).map_err(|e| e.to_string())?;
                        let sound_font =
                            Arc::new(SoundFont::new(&mut sf2).map_err(|e| e.to_string())?);
                        let preset_map = get_preset_map(&sound_font);
                        let mut settings = SynthesizerSettings::new(sample_rate as i32);
                        settings.block_size = block_size;
                        settings.maximum_polyphony = 32;
                        settings.enable_reverb_and_chorus = false;
                        let mut synth =
                            Synthesizer::new(&sound_font, &settings).map_err(|e| e.to_string())?;
                        if let (Some(bank), Some(preset)) = (last_bank, last_preset) {
                            if preset_map.has_preset(bank, preset) {
                                synth.process_midi_message(0, 0xB0, 0x00, bank as i32);
                                synth.process_midi_message(0, 0xC0, preset as i32, 0x00);
                            } else if let Some((bank, preset)) = preset_map.first_available_preset()
                            {
                                last_bank = Some(bank);
                                last_preset = Some(preset);
                                synth.process_midi_message(0, 0xB0, 0x00, bank as i32);
                                synth.process_midi_message(0, 0xC0, preset as i32, 0x00);
                            } else {
                                last_bank = None;
                                last_preset = None;
                            }
                        } else if let Some((bank, preset)) = preset_map.first_available_preset() {
                            last_bank = Some(bank);
                            last_preset = Some(preset);
                            synth.process_midi_message(0, 0xB0, 0x00, bank as i32);
                            synth.process_midi_message(0, 0xC0, preset as i32, 0x00);
                        }
                        Ok((synth, preset_map, last_bank, last_preset))
                    }));
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
        self.handle_synth_init();
    }

    fn synth_init_finished(&mut self) -> Option<SynthInitResHandle> {
        let finished = self
            .synth_init_handle
            .as_ref()
            .map(|h| h.is_finished())
            .unwrap_or(false);

        if finished {
            let mut handle2: Option<SynthInitResHandle> = None;
            mem::swap(&mut self.synth_init_handle, &mut handle2);
            handle2
        } else {
            None
        }
    }

    fn handle_synth_init(&mut self) {
        if let Some(handle) = self.synth_init_finished() {
            let res = handle.join();
            if let Ok(Ok(res)) = res {
                self.handle_synth_init_success(res);
            } else {
                self.call_synth_init_cb(JsonUpdateKind::Failed);
            }
        }
    }

    fn handle_synth_init_success(&mut self, res: SynthInitRes) {
        self.synth = Some(res.0);
        self.preset_map = Some(res.1);
        self.last_bank = res.2;
        self.last_preset = res.3;
        self.call_synth_init_cb(update_fields_or_fail(|updates| {
            updates.push(("loaded_file".to_owned(), serialize(self.last_file.clone())?));
            updates.push(("preset_map".to_owned(), serialize(self.preset_map.clone())?));
            updates.push(("bank".to_owned(), serialize(self.last_bank)?));
            updates.push(("preset".to_owned(), serialize(self.last_preset)?));
            Ok(())
        }));
    }

    fn call_synth_init_cb(&mut self, res: JsonUpdateKind) {
        let mut cb: Option<ResponseCallback> = None;
        mem::swap(&mut self.synth_init_res_cb, &mut cb);
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
            synth_init_handle: None,
            synth_init_res_cb: None,
            last_timestamp: 0,
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
            synth_init_handle: None,
            synth_init_res_cb: None,
            last_timestamp: 0,
        };
        _ = res.init_synth_non_blocking();
        res
    }
}

impl Render for Node {
    fn render_additive(&mut self, lbuf: &mut [f32], rbuf: &mut [f32]) {
        self.update();
        self.resize_buffers(usize::min(lbuf.len(), rbuf.len()));
        if let Some(synth) = &mut self.synth {
            self.last_timestamp += 1;
            let tmp_lbuf = &mut self.tmp_lbuf[..lbuf.len()];
            let tmp_rbuf = &mut self.tmp_rbuf[..rbuf.len()];
            let start = std::time::Instant::now();
            synth.render(tmp_lbuf, tmp_rbuf);
            let duration = start.elapsed();
            if duration.as_micros() > 2500 {
                //FIXME: use fluidsynth instead (it's faster, maybe?)
                synth.note_off_all(true);
            }
            // if self.last_timestamp % 100 == 0 {
            //     tracing::trace!("{:?}", duration);
            // }
            render::amplify_buffer(tmp_lbuf, self.gain);
            render::amplify_buffer(tmp_rbuf, self.gain);
            render::add_buf_to_buf(lbuf, tmp_lbuf);
            render::add_buf_to_buf(rbuf, tmp_rbuf);
        }
    }

    fn reset_rendering(&mut self) {
        if let Some(s) = self.synth.as_mut() {
            s.reset()
        }
    }

    fn set_virtual_paths(&mut self, vp: VirtualPaths) {
        self.last_virtual_paths = Some(vp);
    }

    fn set_sample_rate(&mut self, sample_rate: u32) {
        self.last_sample_rate = Some(sample_rate);
        _ = self.init_synth_non_blocking();
    }

    fn receive_midi_message(&mut self, message: &midi::Message) {
        if self.midi_filter.does_pass(message) && self.does_midi_msg_pass(message) {
            self.process_midi_message(message);
        }
    }

    fn set_global_transposition(&mut self, transposition: i8) {
        self.global_transposition = transposition;
    }

    fn set_json_updater(&mut self, updater: JsonUpdater) {
        // TODO: implement this fn
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

fn get_preset_map(sf: &SoundFont) -> PresetMap {
    let mut map = PresetMap::new();

    sf.get_presets().iter().for_each(|p| {
        let mut preset = Preset::new(p.get_name());
        for r in p.get_regions() {
            preset.add_note_range(r.get_key_range_start() as u8, r.get_key_range_end() as u8);
        }
        map.add_preset(
            p.get_bank_number() as u16,
            p.get_patch_number() as u8,
            preset,
        );
    });

    map
}
