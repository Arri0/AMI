use crate::{
    json::{
        deser_field, deser_field_opt, serialize, DeserializationResult, JsonFieldUpdate,
        SerializationResult,
    }, json_try, midi, path::VirtualPaths, rhythm::Rhythm
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    fs, mem,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};
use tokio::sync::{mpsc, oneshot};

use super::{voices::Voices, ControlMessage, CtrSender};

pub type Requester = mpsc::Sender<(RequestKind, Responder)>;
pub type RequestListener = mpsc::Receiver<(RequestKind, Responder)>;
pub type Responder = oneshot::Sender<ResponseKind>;
pub type ResponseListener = oneshot::Receiver<ResponseKind>;

pub fn create_request_channel(buffer: usize) -> (Requester, RequestListener) {
    mpsc::channel(buffer)
}

pub fn create_response_channel() -> (Responder, ResponseListener) {
    oneshot::channel()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RequestKind {
    SetEnabled(bool),
    AddVoice,
    RemoveVoice(usize),
    ClearVoices,
    SetVoiceName(usize, String),
    SetVoiceInstrument(usize, Option<usize>),
    SetVoiceNote(usize, u8),
    SetVoiceVelocity(usize, u8),
    SetSlot(usize, usize, bool),
    SetRhythm(Rhythm),
    SetTempoBpm(f32),
    Reset,
    LoadPreset(PathBuf),
    SavePreset(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResponseKind {
    Denied,
    Failed,
    Ok,
}

pub struct DrumMachine {
    enabled: bool,
    voices: Voices,
    rhythm: Rhythm,
    tempo_bpm: f32,
    sender: CtrSender,
    req_rx: RequestListener,
    last_time: f32,
    start: SystemTime,
    current_beat: u8,
    current_div: u8,
    virtual_paths: VirtualPaths,
    json_updates: Vec<JsonFieldUpdate>,
}

impl DrumMachine {
    pub fn new(sender: CtrSender, req_rx: RequestListener, virtual_paths: VirtualPaths) -> Self {
        let mut res = Self {
            enabled: true,
            voices: Default::default(),
            rhythm: Default::default(),
            tempo_bpm: 90.0,
            sender,
            req_rx,
            last_time: 0.0,
            start: SystemTime::now(),
            current_beat: 0,
            current_div: 0,
            virtual_paths,
            json_updates: Default::default(),
        };
        res.voices.set_num_slots(res.rhythm.num_slots());
        res
    }

    fn set_enabled(&mut self, flag: bool) -> ResponseKind {
        self.enabled = flag;
        if flag {
            self.reset();
        }
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
        self.rhythm = rhythm;
        self.voices.set_num_slots(self.rhythm.num_slots());
        json_try! {
            self.json_updates.push(("rhythm".to_owned(), serialize(rhythm)?))
            self.json_updates.push(("voices".into(), serialize(&self.voices)?))
        }
        ResponseKind::Ok
    }

    fn set_tempo_bpm(&mut self, tempo_bpm: f32) -> ResponseKind {
        self.tempo_bpm = tempo_bpm;
        json_try! {
            self.json_updates.push(("tempo_bpm".to_owned(), serialize(tempo_bpm)?))
        }
        ResponseKind::Ok
    }

    fn reset(&mut self) -> ResponseKind {
        self.last_time = self.timestamp() - self.period();
        self.current_beat = self.rhythm.num_beats - 1;
        self.current_div = self.rhythm.num_divs - 1;
        json_try! {
            self.json_updates.push(("current_beat".to_owned(), serialize(self.current_beat)?))
            self.json_updates.push(("current_div".to_owned(), serialize(self.current_div)?))
        }
        ResponseKind::Ok
    }

    fn slot_index(&self, beat_num: u8, div_num: u8) -> usize {
        beat_num as usize * self.rhythm.num_divs as usize + div_num as usize
    }

    async fn beat_tick(&mut self, beat_num: u8, div_num: u8) {
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

    async fn produce_noise(&self, instrument_id: usize, channel: u8, note: u8, velocity: u8) {
        _ = self
            .sender
            .send(ControlMessage {
                instrument_id,
                midi_msg: midi::Message {
                    kind: midi::MessageKind::NoteOn { note, velocity },
                    channel,
                },
            })
            .await;
        _ = self
            .sender
            .send(ControlMessage {
                instrument_id,
                midi_msg: midi::Message {
                    kind: midi::MessageKind::NoteOn { note, velocity: 0 },
                    channel,
                },
            })
            .await;
    }

    pub async fn tick(&mut self) {
        self.receive_requests();
        if self.enabled {
            let time = self.timestamp();
            let period = self.period();
            if time - self.last_time >= period {
                self.beat_tick(self.current_beat, self.current_div).await;
                self.advance_div();
                self.last_time += period;
            }
        }
    }

    pub fn period(&self) -> f32 {
        60.0 / (self.tempo_bpm * self.rhythm.num_divs as f32)
    }

    fn advance_div(&mut self) {
        self.current_div = (self.current_div + 1) % self.rhythm.num_divs;
        if self.current_div == 0 {
            self.advance_beat();
        }
    }

    fn advance_beat(&mut self) {
        self.current_beat = (self.current_beat + 1) % self.rhythm.num_beats;
    }

    fn timestamp(&self) -> f32 {
        self.start.elapsed().unwrap_or(Duration::ZERO).as_secs_f32()
    }

    fn receive_requests(&mut self) {
        while let Ok((kind, responder)) = self.req_rx.try_recv() {
            let update = self.process_request(kind);
            if let Err(e) = responder.send(update) {
                tracing::error!("Failed to send a response: {e:?}");
            }
        }
    }

    fn load_preset_from_file(&mut self, path: &Path) -> ResponseKind {
        if let Some(path) = self.virtual_paths.translate(path) {
            if let Ok(file) = fs::read_to_string(path) {
                if let Ok(source) = serde_json::from_str(&file) {
                    if self.deserialize_preset(&source).is_ok() {
                        self.reset();
                        json_try! {
                            self.json_updates.push(("rhythm".to_owned(), serialize(self.rhythm)?))
                            self.json_updates.push(("voices".into(), serialize(&self.voices)?))
                            self.json_updates.push(("tempo_bpm".into(), serialize(self.tempo_bpm)?))
                        }
                        return ResponseKind::Ok;
                    }
                }
            }
        }
        ResponseKind::Failed
    }

    fn save_preset_to_file(&self, path: &Path) -> ResponseKind {
        if let Some(path) = self.virtual_paths.translate(path) {
            if let Ok(source) = self.serialize_preset() {
                if let Ok(source) = serde_json::to_string_pretty(&source) {
                    if fs::write(path, source).is_ok() {
                        return ResponseKind::Ok;
                    }
                }
            }
        }
        ResponseKind::Failed
    }

    fn deserialize_preset(&mut self, source: &serde_json::Value) -> DeserializationResult {
        deser_field(source, "voices", |v| self.voices = v)?;
        deser_field(source, "rhythm", |v| self.rhythm = v)?;
        deser_field(source, "tempo_bpm", |v| self.tempo_bpm = v)?;
        Ok(())
    }

    fn serialize_preset(&self) -> SerializationResult {
        let result: serde_json::Value = json!({
            "voices": serialize(&self.voices)?,
            "rhythm": serialize(self.rhythm)?,
            "tempo_bpm": serialize(self.tempo_bpm)?,
        });
        Ok(result)
    }

    fn process_request(&mut self, kind: RequestKind) -> ResponseKind {
        match kind {
            RequestKind::SetEnabled(flag) => self.set_enabled(flag),
            RequestKind::AddVoice => self.add_voice(),
            RequestKind::RemoveVoice(index) => self.remove_voice(index),
            RequestKind::ClearVoices => self.clear_voices(),
            RequestKind::SetVoiceName(index, name) => self.set_voice_name(index, name),
            RequestKind::SetVoiceInstrument(index, ins) => self.set_voice_instrument(index, ins),
            RequestKind::SetVoiceNote(index, note) => self.set_voice_note(index, note),
            RequestKind::SetVoiceVelocity(index, veloc) => self.set_voice_velocity(index, veloc),
            RequestKind::SetSlot(vi, si, slot) => self.set_slot(vi, si, slot),
            RequestKind::SetRhythm(rhythm) => self.set_rhythm(rhythm),
            RequestKind::SetTempoBpm(tempo_bpm) => self.set_tempo_bpm(tempo_bpm),
            RequestKind::Reset => self.reset(),
            RequestKind::LoadPreset(path) => self.load_preset_from_file(&path),
            RequestKind::SavePreset(path) => self.save_preset_to_file(&path),
        }
    }

    pub fn serialize(&self) -> SerializationResult {
        let result: serde_json::Value = json!({
            "enabled": serialize(self.enabled)?,
            "voices": serialize(&self.voices)?,
            "rhythm": serialize(self.rhythm)?,
            "tempo_bpm": serialize(self.tempo_bpm)?,
            "current_beat": serialize(self.current_beat)?,
            "current_div": serialize(self.current_div)?,
        });
        Ok(result)
    }

    pub fn deserialize(&mut self, source: &serde_json::Value) -> DeserializationResult {
        deser_field_opt(source, "enabled", |v| self.enabled = v)?;
        deser_field_opt(source, "voices", |v| self.voices = v)?;
        deser_field_opt(source, "rhythm", |v| self.rhythm = v)?;
        deser_field_opt(source, "tempo_bpm", |v| self.tempo_bpm = v)?;
        // do not load current_beat and current_div
        self.voices.set_num_slots(self.rhythm.num_slots());
        Ok(())
    }

    pub fn json_updates(&mut self) -> Option<Vec<JsonFieldUpdate>> {
        if !self.json_updates.is_empty() {
            let mut new_updates = Default::default();
            mem::swap(&mut new_updates, &mut self.json_updates);
            Some(new_updates)
        } else {
            None
        }
    }
}