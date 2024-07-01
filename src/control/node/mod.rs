use super::{command::ResponseCallback, drum_machine, CtrSender};
use crate::{
    deser::{DeserializationResult, SerializationResult},
    json::JsonUpdater,
    midi,
    path::VirtualPaths,
    rhythm::Rhythm,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RequestKind {
    SetName(String),
    SetEnabled(bool),
    LoadPreset(PathBuf),
    SavePreset(PathBuf),
    SetUserPreset(usize),
    SetUserPresetEnabled(usize, bool),
    DrumMachine(drum_machine::RequestKind),
}

#[async_trait]
pub trait Control: Sync + Send {
    async fn reset(&mut self);
    async fn beat_tick(&mut self, beat_num: u8, div_num: u8);
    fn set_virtual_paths(&mut self, vp: VirtualPaths);
    fn set_rhythm(&mut self, rhythm: Rhythm);
    fn set_tempo_bpm(&mut self, tempo_bpm: f32);
    fn receive_midi_message(&mut self, message: &midi::Message);
    fn set_control_sender(&mut self, sender: CtrSender);
    fn set_json_updater(&mut self, updater: JsonUpdater);
    fn process_request(&mut self, kind: RequestKind, cb: ResponseCallback);
    fn serialize(&self) -> SerializationResult;
    fn deserialize(&mut self, source: &serde_json::Value) -> DeserializationResult;
    fn clone_node(&self) -> ControlPtr;
}

pub type ControlPtr = Box<dyn Control>;
