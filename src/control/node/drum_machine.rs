use super::{ControlPtr, RequestKind, ResponseCallback, ResponseKind};
use crate::{
    control::{voices::Voices, ControlMessage, CtrSender},
    json::{
        deser_field, deser_field_opt, serialize, DeserializationResult, JsonFieldUpdate,
        SerializationResult,
    },
    json_try, midi,
    path::VirtualPaths,
    rhythm::Rhythm,
};
use axum::async_trait;
use serde_json::json;
use std::{fs, mem, path::Path};

const DEFAULT_NAME: &str = "Drum Machine";

pub struct Node {
    name: String,
    enabled: bool,
    voices: Voices,
    sender: Option<CtrSender>,
    virtual_paths: Option<VirtualPaths>,
    rhythm: Option<Rhythm>,
    user_presets: Vec<bool>,
    json_updates: Vec<JsonFieldUpdate>,
}

impl Node {
    fn set_name(&mut self, name: String) -> ResponseKind {
        self.name = name;
        json_try! {
            self.json_updates.push(("name".to_owned(), serialize(&self.name)?))
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

    fn add_voice(&mut self) -> ResponseKind {
        self.voices.add_voice();
        json_try! {
            self.json_updates.push(("voices".into(), serialize(&self.voices)?))
        }
        ResponseKind::Ok
    }

    fn remove_voice(&mut self, index: usize) -> ResponseKind {
        if self.voices.remove_voice(index).is_ok() {
            json_try! {
                self.json_updates.push(("voices".into(), serialize(&self.voices)?))
            }
            ResponseKind::Ok
        } else {
            ResponseKind::Failed
        }
    }

    fn clear_voices(&mut self) -> ResponseKind {
        self.voices.clear();
        json_try! {
            self.json_updates.push(("voices".into(), serialize(&self.voices)?))
        }
        ResponseKind::Ok
    }

    fn set_voice_name(&mut self, voice_index: usize, name: String) -> ResponseKind {
        let res = self.voices.set_voice_name(voice_index, name).is_ok();
        if res {
            json_try! {
                self.json_updates.push(("voices".into(), serialize(&self.voices)?))
            }
            ResponseKind::Ok
        } else {
            ResponseKind::Failed
        }
    }

    fn set_voice_instrument(
        &mut self,
        voice_index: usize,
        instrument_index: Option<usize>,
    ) -> ResponseKind {
        let res = self
            .voices
            .set_voice_instrument(voice_index, instrument_index)
            .is_ok();
        if res {
            json_try! {
                self.json_updates.push(("voices".into(), serialize(&self.voices)?))
            }
            ResponseKind::Ok
        } else {
            ResponseKind::Failed
        }
    }

    fn set_voice_note(&mut self, voice_index: usize, note: u8) -> ResponseKind {
        if self.voices.set_voice_note(voice_index, note).is_ok() {
            json_try! {
                self.json_updates.push(("voices".into(), serialize(&self.voices)?))
            }
            ResponseKind::Ok
        } else {
            ResponseKind::Failed
        }
    }

    fn set_voice_velocity(&mut self, voice_index: usize, velocity: u8) -> ResponseKind {
        if self
            .voices
            .set_voice_velocity(voice_index, velocity)
            .is_ok()
        {
            json_try! {
                self.json_updates.push(("voices".into(), serialize(&self.voices)?))
            }
            ResponseKind::Ok
        } else {
            ResponseKind::Failed
        }
    }

    fn set_voice_channel(&mut self, voice_index: usize, channel: u8) -> ResponseKind {
        if self.voices.set_voice_channel(voice_index, channel).is_ok() {
            json_try! {
                self.json_updates.push(("voices".into(), serialize(&self.voices)?))
            }
            ResponseKind::Ok
        } else {
            ResponseKind::Failed
        }
    }

    fn set_slot(&mut self, voice_index: usize, slot_index: usize, enabled: bool) -> ResponseKind {
        let res = self
            .voices
            .set_slot(voice_index, slot_index, enabled)
            .is_ok();
        if res {
            json_try! {
                self.json_updates.push(("voices".into(), serialize(&self.voices)?))
            }
            ResponseKind::Ok
        } else {
            ResponseKind::Failed
        }
    }

    fn set_rhythm(&mut self, rhythm: Rhythm) -> ResponseKind {
        self.rhythm = Some(rhythm);
        self.voices.set_num_slots(rhythm.num_slots());
        json_try! {
            self.json_updates.push(("voices".into(), serialize(&self.voices)?))
        }
        ResponseKind::Ok
    }

    fn slot_index(&self, beat_num: u8, div_num: u8) -> usize {
        let rhythm = self.rhythm.unwrap_or_default();
        beat_num as usize * rhythm.num_divs as usize + div_num as usize
    }

    async fn produce_noise(&self, instrument_id: usize, channel: u8, note: u8, velocity: u8) {
        if let Some(sender) = &self.sender {
            _ = sender
                .send(ControlMessage {
                    instrument_id,
                    midi_msg: midi::Message {
                        kind: midi::MessageKind::NoteOn { note, velocity },
                        channel,
                    },
                })
                .await;
            _ = sender
                .send(ControlMessage {
                    instrument_id,
                    midi_msg: midi::Message {
                        kind: midi::MessageKind::NoteOn { note, velocity: 0 },
                        channel,
                    },
                })
                .await;
        };
    }

    fn load_preset_from_file(&mut self, path: &Path) -> ResponseKind {
        if let Some(vp) = &self.virtual_paths {
            if let Some(path) = vp.translate(path) {
                if let Ok(file) = fs::read_to_string(path) {
                    if let Ok(source) = serde_json::from_str(&file) {
                        if self.deserialize_preset(&source).is_ok() {
                            json_try! {
                                self.json_updates.push(("voices".into(), serialize(&self.voices)?))
                            }
                            return ResponseKind::Ok;
                        }
                    }
                }
            }
        }
        ResponseKind::Failed
    }

    fn save_preset_to_file(&self, path: &Path) -> ResponseKind {
        if let Some(vp) = &self.virtual_paths {
            if let Some(path) = vp.translate(path) {
                if let Ok(source) = self.serialize_preset() {
                    if let Ok(source) = serde_json::to_string_pretty(&source) {
                        if fs::write(path, source).is_ok() {
                            return ResponseKind::Ok;
                        }
                    }
                }
            }
        }
        ResponseKind::Failed
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

    fn deserialize_preset(&mut self, source: &serde_json::Value) -> DeserializationResult {
        deser_field(source, "voices", |v| self.voices = v)?;
        if let Some(rhythm) = self.rhythm {
            self.voices.set_num_slots(rhythm.num_slots());
        }
        json_try! {
            self.json_updates.push(("voices".into(), serialize(&self.voices)?))
        }
        Ok(())
    }

    fn serialize_preset(&self) -> SerializationResult {
        let result: serde_json::Value = json!({
            "voices": serialize(&self.voices)?,
            "rhythm": serialize(self.rhythm)?,
        });
        Ok(result)
    }
}

impl Default for Node {
    fn default() -> Self {
        Self {
            name: DEFAULT_NAME.into(),
            enabled: true,
            voices: Default::default(),
            sender: None,
            virtual_paths: None,
            rhythm: Default::default(),
            user_presets: vec![true; super::NUM_USER_PRESETS],
            json_updates: Default::default(),
        }
    }
}

impl Clone for Node {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            enabled: self.enabled,
            voices: self.voices.clone(),
            sender: None,
            virtual_paths: None,
            rhythm: Default::default(),
            user_presets: self.user_presets.clone(),
            json_updates: Default::default(),
        }
    }
}

#[async_trait]
impl super::Control for Node {
    fn reset(&mut self) {}

    async fn beat_tick(&mut self, beat_num: u8, div_num: u8) {
        if !self.enabled {
            return;
        }

        let slot_index = self.slot_index(beat_num, div_num);
        for voice in self.voices.voices() {
            if let Some(instrument_index) = &voice.instrument_index {
                let channel = voice.channel;
                if slot_index < voice.slots().len() {
                    let enabled = voice.slots()[slot_index];
                    if enabled {
                        self.produce_noise(*instrument_index, channel, voice.note, voice.velocity)
                            .await;
                    }
                }
            }
        }
    }

    fn set_virtual_paths(&mut self, vp: VirtualPaths) {
        self.virtual_paths = Some(vp);
    }

    fn set_rhythm(&mut self, rhythm: Rhythm) {
        self.set_rhythm(rhythm);
    }

    fn set_tempo_bpm(&mut self, _tempo_bpm: f32) {}

    fn set_control_sender(&mut self, sender: CtrSender) {
        self.sender = Some(sender);
    }

    fn set_user_preset(&mut self, preset: usize) {
        if preset < self.user_presets.len() {
            self.enabled = self.user_presets[preset];
            json_try! {
                self.json_updates.push(("enabled".into(), serialize(self.enabled)?))
            }
        }
    }

    fn receive_midi_message(&mut self, _message: &midi::Message) {}

    fn process_request(&mut self, kind: RequestKind, cb: ResponseCallback) {
        match kind {
            RequestKind::SetName(name) => cb(self.set_name(name)),
            RequestKind::SetEnabled(flag) => cb(self.set_enabled(flag)),
            RequestKind::LoadPreset(path) => cb(self.load_preset_from_file(&path)),
            RequestKind::SavePreset(path) => cb(self.save_preset_to_file(&path)),
            RequestKind::SetUserPresetEnabled(p, f) => cb(self.set_user_preset_enabled(p, f)),
            RequestKind::AddVoice => cb(self.add_voice()),
            RequestKind::RemoveVoice(index) => cb(self.remove_voice(index)),
            RequestKind::ClearVoices => cb(self.clear_voices()),
            RequestKind::SetVoiceName(index, name) => cb(self.set_voice_name(index, name)),
            RequestKind::SetVoiceInstrument(i, ins) => cb(self.set_voice_instrument(i, ins)),
            RequestKind::SetVoiceNote(index, note) => cb(self.set_voice_note(index, note)),
            RequestKind::SetVoiceVelocity(i, v) => cb(self.set_voice_velocity(i, v)),
            RequestKind::SetVoiceChannel(i, c) => cb(self.set_voice_channel(i, c)),
            RequestKind::SetSlot(vi, si, slot) => cb(self.set_slot(vi, si, slot)),
        }
    }

    fn render_node_moved(&mut self, _id: usize, _new_id: usize) {
        todo!();
    }

    fn serialize(&self) -> SerializationResult {
        let result: serde_json::Value = json!({
            "name": serialize(&self.name)?,
            "enabled": serialize(self.enabled)?,
            "voices": serialize(&self.voices)?,
            "user_presets": serialize(&self.user_presets)?,
        });
        Ok(result)
    }

    fn deserialize(&mut self, source: &serde_json::Value) -> DeserializationResult {
        deser_field_opt(source, "name", |v| self.name = v)?;
        deser_field_opt(source, "enabled", |v| self.enabled = v)?;
        deser_field_opt(source, "voices", |v| self.voices = v)?;
        deser_field_opt(source, "user_presets", |v| self.user_presets = v)?;
        if let Some(rhythm) = self.rhythm {
            self.voices.set_num_slots(rhythm.num_slots());
        }
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

    fn clone_node(&self) -> ControlPtr {
        Box::new(self.clone())
    }
}
