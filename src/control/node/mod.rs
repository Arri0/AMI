use super::CtrSender;
use crate::{
    json::{DeserializationResult, JsonFieldUpdate, SerializationResult},
    midi,
    path::VirtualPaths,
    rhythm::Rhythm,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod drum_machine;

pub type ResponseCallback = Box<dyn FnOnce(ResponseKind) + 'static + Send + Sync>;

pub const NUM_USER_PRESETS: usize = 16;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RequestKind {
    SetName(String),
    SetEnabled(bool),
    LoadPreset(PathBuf),
    SavePreset(PathBuf),
    SetUserPresetEnabled(usize, bool),
    AddVoice,
    RemoveVoice(usize),
    ClearVoices,
    SetVoiceName(usize, String),
    SetVoiceInstrument(usize, Option<usize>),
    SetVoiceNote(usize, u8),
    SetVoiceVelocity(usize, u8),
    SetVoiceChannel(usize, u8),
    SetSlot(usize, usize, bool),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResponseKind {
    InvalidId,
    Denied,
    Failed,
    Ok,
}

#[async_trait]
pub trait Control: Sync + Send {
    fn reset(&mut self);
    async fn beat_tick(&mut self, beat_num: u8, div_num: u8);
    fn set_virtual_paths(&mut self, vp: VirtualPaths);
    fn set_rhythm(&mut self, rhythm: Rhythm);
    fn set_tempo_bpm(&mut self, tempo_bpm: f32);
    fn set_control_sender(&mut self, sender: CtrSender);
    fn set_user_preset(&mut self, preset: usize);
    fn receive_midi_message(&mut self, message: &midi::Message);
    fn process_request(&mut self, kind: RequestKind, cb: ResponseCallback);
    fn render_node_moved(&mut self, id: usize, new_id: usize);
    fn serialize(&self) -> SerializationResult; //TODO: return serde_json::Value instead
    fn deserialize(&mut self, source: &serde_json::Value) -> DeserializationResult;
    fn json_updates(&mut self) -> Option<Vec<JsonFieldUpdate>>;
    fn clone_node(&self) -> ControlPtr;
}

pub type ControlPtr = Box<dyn Control>;
