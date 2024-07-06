use super::Render;
use crate::{
    deser::{deser_field_opt, serialize, DeserializationResult, SerializationResult}, json::{update_fields_or_fail, JsonUpdateKind, JsonUpdater}, midi::{self, ControlChangeKind}, path::VirtualPaths, render::{
        self,
        command::{midi_filter::UpdateMidiFilterKind, ResponseCallback},
        midi_filter::{self, MidiFilterUser},
        node::RequestKind,
        velocity_map,
    }, synth::sfizz
};
use serde_json::json;
use std::{
    mem,
    path::{Path, PathBuf},
    sync::Mutex,
    thread::{self, JoinHandle},
};

const DEFAULT_NAME: &str = "Sfizz Synth";

type SoundFontLoadHandle = JoinHandle<Result<std::sync::Mutex<sfizz::Synth>, String>>;

pub struct Node {
    name: String,
    enabled: bool,
    midi_filter: midi_filter::MidiFilter,
    synth: Option<Mutex<sfizz::Synth>>,
    last_file: Option<PathBuf>,
    last_virtual_paths: Option<VirtualPaths>,
    last_sample_rate: Option<u32>,
    last_buffer_size: Option<usize>,
    gain: f32,
    transposition: i8,
    global_transposition: i8,
    velocity_mapping: velocity_map::Kind,
    ignore_global_transposition: bool,
    tmp_lbuf: Vec<f32>,
    tmp_rbuf: Vec<f32>,
    user_presets: Vec<bool>,
    json_updater: Option<JsonUpdater>,
    file_load_handle: Option<SoundFontLoadHandle>,
    file_load_res_cb: Option<ResponseCallback>,
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
            self.file_load_res_cb = Some(cb);
        } else {
            cb(JsonUpdateKind::Failed);
        }
    }

    fn load_file_non_blocking(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let (Some(file), Some(vp)) = (&self.last_file, &self.last_virtual_paths) {
            if let Some(file) = vp.translate(file) {
                let sample_rate = self.last_sample_rate;
                let buffer_size = self.last_buffer_size;
                self.file_load_handle = Some(thread::spawn(
                    move || -> Result<Mutex<sfizz::Synth>, String> {
                        let mut synth = sfizz::Synth::default();
                        if let Some(sample_rate) = sample_rate {
                            synth.set_sample_rate(sample_rate);
                        }
                        if let Some(buffer_size) = buffer_size {
                            synth.set_num_frames(buffer_size);
                        }
                        match synth.load_file(&file) {
                            Ok(()) => Ok(std::sync::Mutex::new(synth)),
                            Err(e) => Err(e.to_string()),
                        }
                    },
                ));
                Ok(())
            } else {
                Err(String::from("Could not load file.").into())
            }
        } else {
            Err(String::from("Could not load file.").into())
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

    fn set_velocity_mapping(&mut self, mapping: &velocity_map::Kind) -> JsonUpdateKind {
        self.velocity_mapping = *mapping;
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
            Kind::NoteOff { note, velocity } => self.note_off(note, velocity),
            Kind::PolyphonicAftertouch { note, pressure } => self.poly_aftt(note, pressure),
            Kind::ControlChange { kind, value } => self.cc(kind, value),
            Kind::ProgramChange { .. } => {}
            Kind::ChannelAftertouch { pressure } => self.channel_aftt(pressure),
            Kind::PitchWheel { value } => self.pitch_wheel(value),
        }
    }

    fn note_on(&mut self, note: u8, velocity: u8) {
        let note = self.transpose_note(note);
        if let Some(synth) = &self.synth {
            if let Ok(mut synth) = synth.lock() {
                synth.send_note_on(note, velocity);
            }
        }
    }

    fn note_off(&mut self, note: u8, velocity: u8) {
        let note = self.transpose_note(note);
        if let Some(synth) = &self.synth {
            if let Ok(mut synth) = synth.lock() {
                synth.send_note_off(note, velocity);
            }
        }
    }

    fn poly_aftt(&mut self, note: u8, pressure: u8) {
        let note = self.transpose_note(note);
        if let Some(synth) = &self.synth {
            if let Ok(mut synth) = synth.lock() {
                synth.send_polyphonic_aftertouch(note, pressure);
            }
        }
    }

    fn cc(&mut self, kind: ControlChangeKind, value: u8) {
        if let Some(synth) = &self.synth {
            if let Ok(mut synth) = synth.lock() {
                synth.send_cc(kind.as_number(), value);
            }
        }
    }

    fn channel_aftt(&mut self, pressure: u8) {
        if let Some(synth) = &self.synth {
            if let Ok(mut synth) = synth.lock() {
                synth.send_channel_aftertouch(pressure);
            }
        }
    }

    fn pitch_wheel(&mut self, value: u16) {
        if let Some(synth) = &self.synth {
            if let Ok(mut synth) = synth.lock() {
                synth.send_pitch_wheel(midi::Message::get_pitch_wheel_signed(value));
            }
        }
    }

    fn resize_buffers(&mut self, min_size: usize) {
        if self.tmp_lbuf.len() < min_size {
            self.last_buffer_size = Some(min_size);
            if let Some(synth) = &self.synth {
                if let Ok(mut synth) = synth.lock() {
                    synth.set_num_frames(min_size);
                }
            }
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
        self.handle_file_load();
    }

    fn file_load_finished(&mut self) -> Option<SoundFontLoadHandle> {
        let finished = self
            .file_load_handle
            .as_ref()
            .map(|h| h.is_finished())
            .unwrap_or(false);

        if finished {
            let mut handle2: Option<SoundFontLoadHandle> = None;
            mem::swap(&mut self.file_load_handle, &mut handle2);
            handle2
        } else {
            None
        }
    }

    fn handle_file_load(&mut self) {
        if let Some(handle) = self.file_load_finished() {
            let res = handle.join();
            if let Ok(Ok(res)) = res {
                self.handle_sf_load_success(res);
            } else {
                self.call_sf_load_cb(JsonUpdateKind::Failed);
            }
        }
    }

    fn handle_sf_load_success(&mut self, synth: Mutex<sfizz::Synth>) {
        self.synth = Some(synth);
        self.call_sf_load_cb(update_fields_or_fail(|updates| {
            updates.push(("loaded_file".to_owned(), serialize(self.last_file.clone())?));
            Ok(())
        }));
    }

    fn call_sf_load_cb(&mut self, res: JsonUpdateKind) {
        let mut cb: Option<ResponseCallback> = None;
        mem::swap(&mut self.file_load_res_cb, &mut cb);
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
            synth: Some(Mutex::new(sfizz::Synth::default())),
            last_file: None,
            last_virtual_paths: None,
            last_sample_rate: None,
            last_buffer_size: None,
            gain: 1.0,
            transposition: 0,
            global_transposition: 0,
            velocity_mapping: velocity_map::Kind::Identity,
            ignore_global_transposition: false,
            tmp_lbuf: Vec::new(),
            tmp_rbuf: Vec::new(),
            user_presets: vec![true; super::NUM_USER_PRESETS],
            json_updater: None,
            file_load_handle: None,
            file_load_res_cb: None,
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
            last_buffer_size: self.last_buffer_size,
            gain: self.gain,
            transposition: self.transposition,
            global_transposition: self.global_transposition,
            velocity_mapping: self.velocity_mapping,
            ignore_global_transposition: self.ignore_global_transposition,
            tmp_lbuf: vec![0.0; self.tmp_lbuf.len()],
            tmp_rbuf: vec![0.0; self.tmp_rbuf.len()],
            user_presets: self.user_presets.clone(),
            json_updater: None,
            file_load_handle: None,
            file_load_res_cb: None,
        };
        _ = res.load_file_non_blocking();
        res
    }
}

impl Render for Node {
    fn render_additive(&mut self, lbuf: &mut [f32], rbuf: &mut [f32]) {
        self.update();
        self.resize_buffers(lbuf.len());
        let tmp_lbuf = &mut self.tmp_lbuf[..lbuf.len()];
        let tmp_rbuf = &mut self.tmp_rbuf[..rbuf.len()];
        if let Some(synth) = &self.synth {
            if let Ok(mut synth) = synth.lock() {
                synth.render_block(tmp_lbuf, tmp_rbuf);
            }
        }
        render::amplify_buffer(tmp_lbuf, self.gain);
        render::amplify_buffer(tmp_rbuf, self.gain);
        render::add_buf_to_buf(lbuf, tmp_lbuf);
        render::add_buf_to_buf(rbuf, tmp_rbuf);
    }

    fn reset_rendering(&mut self) {
        if let Some(synth) = &self.synth {
            if let Ok(mut synth) = synth.lock() {
                synth.silence();
            }
        }
    }

    fn set_virtual_paths(&mut self, vp: VirtualPaths) {
        self.last_virtual_paths = Some(vp);
    }

    fn set_sample_rate(&mut self, sample_rate: u32) {
        self.last_sample_rate = Some(sample_rate);
        if let Some(synth) = &self.synth {
            if let Ok(mut synth) = synth.lock() {
                synth.set_sample_rate(sample_rate);
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

    fn set_json_updater(&mut self, updater: JsonUpdater) {
        self.json_updater = Some(updater);
    }

    fn process_request(&mut self, kind: RequestKind, cb: ResponseCallback) {
        type RK = RequestKind;
        match kind {
            RK::SetName(name) => cb(self.set_name(&name)),
            RK::SetEnabled(flag) => cb(self.set_enabled(flag)),
            RK::LoadFile(path) => self.load_file(&path, cb),
            RK::SetGain(gain) => cb(self.set_gain(gain)),
            RK::SetTransposition(tr) => cb(self.set_transposition(tr)),
            RK::SetVelocityMapping(kind) => cb(self.set_velocity_mapping(&kind)),
            RK::SetIgnoreGlobalTransposition(flag) => {
                cb(self.set_ignore_global_transposition(flag))
            }
            RK::UpdateMidiFilter(kind) => cb(self.update_midi_filter(kind)),
            RK::SetUserPreset(preset) => cb(self.set_user_preset(preset)),
            RK::SetUserPresetEnabled(p, f) => cb(self.set_user_preset_enabled(p, f)),
            _ => cb(JsonUpdateKind::Denied),
        }
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
            "user_presets": serialize(&self.user_presets)?,
        });
        Ok(result)
    }

    fn deserialize(&mut self, source: &serde_json::Value) -> DeserializationResult {
        deser_field_opt(source, "name", |v| self.name = v)?;
        deser_field_opt(source, "enabled", |v| self.enabled = v)?;
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
