use super::{Render, ResponseCallback};
use crate::{
    json::{
        deser_field_opt, serialize, DeserializationResult, JsonFieldUpdate, SerializationResult,
    },
    json_try,
    midi::{self, ControlChangeKind},
    path::VirtualPaths,
    render::{
        self,
        midi_filter::{self, MidiFilterUser},
        node::{RequestKind, ResponseKind},
        preset_map::{Preset, PresetMap},
        velocity_map,
    },
};
use oxisynth::{SoundFont, Synth};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::HashMap,
    fmt::Display,
    fs::File,
    mem,
    path::{Path, PathBuf},
    thread::{self, JoinHandle},
};

const DEFAULT_NAME: &str = "Oxi Synth";
const POLYPHONY: u16 = 64;

type SoundFontLoadRes = (Synth, PresetMap, Option<u16>, Option<u8>);
type SoundFontLoadHandle = JoinHandle<Result<SoundFontLoadRes, String>>;

#[derive(Debug)]
pub struct CouldNotInitSynth;

impl Display for CouldNotInitSynth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "Could not init synth - uninitialized parameters.".fmt(f)
    }
}

impl std::error::Error for CouldNotInitSynth {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct ReverbParams {
    active: bool,
    room_size: f32,
    damping: f32,
    width: f32,
    level: f32,
}

impl Default for ReverbParams {
    fn default() -> Self {
        Self {
            active: false,
            room_size: 0.2,
            damping: 0.0,
            width: 0.5,
            level: 0.9,
        }
    }
}

pub struct Node {
    name: String,
    enabled: bool,
    midi_filter: midi_filter::MidiFilter,
    synth: Option<Synth>,
    last_file: Option<PathBuf>,
    last_virtual_paths: Option<VirtualPaths>,
    last_sample_rate: Option<u32>,
    last_bank: Option<u16>,
    last_preset: Option<u8>,
    last_cc: HashMap<u8, u8>,
    last_pitch_wheel: u16,
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
    reverb: ReverbParams,
    json_updates: Vec<JsonFieldUpdate>,
}

impl Node {
    fn set_name(&mut self, name: &str) -> ResponseKind {
        self.name = name.into();
        json_try! {
            self.json_updates.push(("name".to_owned(), serialize(name)?))
        }
        ResponseKind::Ok
    }

    fn set_enabled(&mut self, flag: bool) -> ResponseKind {
        self.enabled = flag;
        json_try! {
            self.json_updates.push(("enabled".to_owned(), serialize(flag)?))
        }
        ResponseKind::Ok
    }

    fn load_file(&mut self, path: &Path, cb: ResponseCallback) {
        self.last_file = Some(path.to_owned());
        if let Ok(()) = self.load_file_non_blocking() {
            self.sf_load_res_cb = Some(cb);
        } else {
            cb(ResponseKind::Failed);
        }
    }

    fn set_gain(&mut self, gain: f32) -> ResponseKind {
        self.gain = gain;
        json_try! {
            self.json_updates.push(("gain".into(), serialize(gain)?))
        }
        ResponseKind::Ok
    }

    fn set_transposition(&mut self, transposition: i8) -> ResponseKind {
        self.transposition = transposition;
        json_try! {
            self.json_updates.push(("transposition".into(), serialize(transposition)?))
        }
        ResponseKind::Ok
    }

    fn set_velocity_mapping(&mut self, mapping: velocity_map::Kind) -> ResponseKind {
        self.velocity_mapping = mapping;
        json_try! {
            self.json_updates.push(("velocity_mapping".into(), serialize(mapping)?))
        }
        ResponseKind::Ok
    }

    fn set_ignore_global_transposition(&mut self, flag: bool) -> ResponseKind {
        self.ignore_global_transposition = flag;
        json_try! {
            self.json_updates.push(("ignore_global_transposition".into(), serialize(flag)?))
        }
        ResponseKind::Ok
    }

    fn set_preset(&mut self, bank: u16, preset: u8) -> ResponseKind {
        self.last_bank = Some(bank);
        self.last_preset = Some(preset);
        if let Some(synth) = &mut self.synth {
            if synth.font_bank().count() != 0 {
                _ = synth.bank_select(0, bank as u32);
                _ = synth.send_event(oxisynth::MidiEvent::ProgramChange {
                    channel: 0,
                    program_id: preset,
                });
                json_try! {
                    self.json_updates.push(("bank".into(), serialize(bank)?))
                    self.json_updates.push(("preset".into(), serialize(preset)?))
                }
                ResponseKind::Ok
            } else {
                ResponseKind::Failed
            }
        } else {
            ResponseKind::Failed
        }
    }

    fn set_reverb_active(&mut self, active: bool) -> ResponseKind {
        self.reverb.active = active;

        if let Some(synth) = &mut self.synth {
            synth.get_reverb_mut().set_active(active);
        }

        json_try! {
            self.json_updates.push(("reverb".into(), serialize(self.reverb)?))
        }

        ResponseKind::Ok
    }

    fn set_reverb_params(
        &mut self,
        room_size: f32,
        damping: f32,
        width: f32,
        level: f32,
    ) -> ResponseKind {
        self.reverb.room_size = room_size;
        self.reverb.damping = damping;
        self.reverb.width = width;
        self.reverb.level = level;

        if let Some(synth) = &mut self.synth {
            synth
                .get_reverb_mut()
                .set_reverb_params(room_size, damping, width, level);
        }

        json_try! {
            self.json_updates.push(("reverb".into(), serialize(self.reverb)?))
        }

        ResponseKind::Ok
    }

    fn update_midi_filter(&mut self, kind: midi_filter::UpdateKind) -> ResponseKind {
        if MidiFilterUser::process_update_request(self, kind).is_ok() {
            json_try! {
                self.json_updates.push(("midi_filter".into(), serialize(&self.midi_filter)?))
            }
            ResponseKind::Ok
        } else {
            ResponseKind::Failed
        }
    }

    fn set_user_preset(&mut self, preset: usize) -> ResponseKind {
        if preset >= self.user_presets.len() {
            ResponseKind::Failed
        } else {
            self.enabled = self.user_presets[preset];
            json_try! {
                self.json_updates.push(("enabled".into(), serialize(self.enabled)?))
            }
            ResponseKind::Ok
        }
    }

    fn set_user_preset_enabled(&mut self, preset: usize, flag: bool) -> ResponseKind {
        if preset >= self.user_presets.len() {
            ResponseKind::Failed
        } else {
            self.user_presets[preset] = flag;
            json_try! {
                self.json_updates.push(("user_presets".into(), serialize(&self.user_presets)?))
            }
            ResponseKind::Ok
        }
    }

    fn process_midi_message(&mut self, message: &midi::Message) {
        self.process_midi_message_kind(&message.kind);
    }

    fn process_midi_message_kind(&mut self, kind: &midi::MessageKind) {
        use midi::MessageKind as Kind;
        match *kind {
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
            _ = synth.send_event(oxisynth::MidiEvent::NoteOn {
                channel: 0,
                key: note,
                vel: velocity,
            });
        }
    }

    fn note_off(&mut self, note: u8) {
        let note = self.transpose_note(note);
        if let Some(synth) = &mut self.synth {
            _ = synth.send_event(oxisynth::MidiEvent::NoteOff {
                channel: 0,
                key: note,
            });
        }
    }

    fn polyphonic_aftertouch(&mut self, note: u8, pressure: u8) {
        if let Some(synth) = &mut self.synth {
            _ = synth.send_event(oxisynth::MidiEvent::PolyphonicKeyPressure {
                channel: 0,
                key: note,
                value: pressure,
            });
        }
    }

    fn control_change(&mut self, kind: ControlChangeKind, value: u8) {
        self.last_cc.insert(kind.as_number(), value);
        if let Some(synth) = &mut self.synth {
            _ = synth.send_event(oxisynth::MidiEvent::ControlChange {
                channel: 0,
                ctrl: kind.as_number(),
                value,
            });
        }
    }

    fn program_change(&mut self, program: u8) {
        if let Some(synth) = &mut self.synth {
            _ = synth.send_event(oxisynth::MidiEvent::ProgramChange {
                channel: 0,
                program_id: program,
            });
        }
    }

    fn channel_aftertouch(&mut self, pressure: u8) {
        if let Some(synth) = &mut self.synth {
            _ = synth.send_event(oxisynth::MidiEvent::ChannelPressure {
                channel: 0,
                value: pressure,
            });
        }
    }

    fn pitch_wheel(&mut self, value: u16) {
        self.last_pitch_wheel = value;
        if let Some(synth) = &mut self.synth {
            _ = synth.send_event(oxisynth::MidiEvent::PitchBend { channel: 0, value });
        }
    }

    fn load_file_non_blocking(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let (Some(file), Some(vp)) = (&self.last_file, &self.last_virtual_paths) {
            if let Some(file) = vp.translate(file) {
                let mut last_bank = self.last_bank;
                let mut last_preset = self.last_preset;
                let sample_rate = self.last_sample_rate;
                let reverb = self.reverb;
                let last_cc = self.last_cc.clone();
                let last_pitch_wheel = self.last_pitch_wheel;
                self.sf_load_handle = Some(thread::spawn(
                    move || -> Result<SoundFontLoadRes, String> {
                        let font = SoundFont::load(
                            &mut File::open(file.clone()).map_err(|e| e.to_string())?,
                        )
                        .map_err(|_| String::from("Failed to parse SoundFont file"))?;
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
                        let mut synth = Synth::default();
                        synth.add_font(font, true);
                        _ = synth.set_polyphony(POLYPHONY);
                        synth.get_reverb_mut().set_active(reverb.active);
                        synth.get_reverb_mut().set_reverb_params(
                            reverb.room_size,
                            reverb.damping,
                            reverb.width,
                            reverb.level,
                        );

                        if let Some(sample_rate) = sample_rate {
                            synth.set_sample_rate(sample_rate as f32);
                        }
                        _ = synth.send_event(oxisynth::MidiEvent::PitchBend {
                            channel: 0,
                            value: last_pitch_wheel,
                        });
                        for (ctrl, value) in last_cc {
                            _ = synth.send_event(oxisynth::MidiEvent::ControlChange {
                                channel: 0,
                                ctrl,
                                value,
                            })
                        }
                        Ok((synth, preset_map, last_bank, last_preset))
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
                self.call_sf_load_cb(ResponseKind::Failed);
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
            _ = synth.bank_select(0, bank as u32);
            _ = synth.send_event(oxisynth::MidiEvent::ProgramChange {
                channel: 0,
                program_id: preset,
            });
        }
        json_try! {
            self.json_updates.push(("loaded_file".to_owned(), serialize(self.last_file.clone())?))
            self.json_updates.push(("preset_map".to_owned(), serialize(self.preset_map.clone())?))
            self.json_updates.push(("bank".to_owned(), serialize(self.last_bank)?))
            self.json_updates.push(("preset".to_owned(), serialize(self.last_preset)?))
        }
        self.call_sf_load_cb(ResponseKind::Ok);
    }

    fn call_sf_load_cb(&mut self, res: ResponseKind) {
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
            last_cc: HashMap::new(),
            last_pitch_wheel: 8192, // TODO: make sure this is the correct default value
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
            reverb: Default::default(),
            json_updates: Default::default(),
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
            last_cc: self.last_cc.clone(),
            last_pitch_wheel: self.last_pitch_wheel,
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
            reverb: self.reverb,
            json_updates: Default::default(),
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
            synth.write_f32(len, tmp_lbuf, 0, 1, tmp_rbuf, 0, 1);
        }
        render::amplify_buffer(tmp_lbuf, self.gain);
        render::amplify_buffer(tmp_rbuf, self.gain);
        render::add_buf_to_buf(lbuf, tmp_lbuf);
        render::add_buf_to_buf(rbuf, tmp_rbuf);
    }

    fn reset_rendering(&mut self) {
        if let Some(synth) = &mut self.synth {
            _ = synth.send_event(oxisynth::MidiEvent::AllSoundOff { channel: 0 });
        }
    }

    fn set_virtual_paths(&mut self, vp: VirtualPaths) {
        self.last_virtual_paths = Some(vp);
    }

    fn set_sample_rate(&mut self, sample_rate: u32) {
        self.last_sample_rate = Some(sample_rate);
        if let Some(synth) = &mut self.synth {
            synth.set_sample_rate(sample_rate as f32);
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
            RK::MidiMessage(kind) => {
                self.process_midi_message_kind(&kind);
                json_try! {
                    //TODO: support indices and fields for optimization
                    self.json_updates.push(("cc".into(), serialize(self.last_cc.clone())?))
                    self.json_updates.push(("pitch_wheel".into(), serialize(self.last_pitch_wheel)?))
                }
                cb(ResponseKind::Ok)
            }
            RK::SetSfReverbActive(active) => cb(self.set_reverb_active(active)),
            RK::SetSfReverbParams {
                room_size,
                damping,
                width,
                level,
            } => cb(self.set_reverb_params(room_size, damping, width, level)),
            RK::UpdateMidiFilter(kind) => cb(self.update_midi_filter(kind)),
            RK::SetUserPreset(preset) => cb(self.set_user_preset(preset)),
            RK::SetUserPresetEnabled(p, f) => cb(self.set_user_preset_enabled(p, f)),
            _ => cb(ResponseKind::Denied),
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
            "cc": serialize(self.last_cc.clone())?,
            "pitch_wheel": serialize(self.last_pitch_wheel)?,
            "user_presets": serialize(&self.user_presets)?,
            "reverb": serialize(self.reverb)?,
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
        deser_field_opt(source, "cc", |v| self.last_cc = v)?;
        deser_field_opt(source, "pitch_wheel", |v| self.last_pitch_wheel = v)?;
        deser_field_opt(source, "user_presets", |v| self.user_presets = v)?;
        deser_field_opt(source, "reverb", |v| self.reverb = v)?;
        Ok(())
    }

    fn json_updates(&mut self) -> Option<Vec<JsonFieldUpdate>> {
        if !self.json_updates.is_empty() {
            let mut new_updates = Default::default();
            mem::swap(&mut new_updates, &mut self.json_updates);
            Some(new_updates)
        } else {
            None
        }
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
            p.get_bank_number() as u16,
            p.get_patch_number() as u8,
            preset,
        );
    });

    map
}
