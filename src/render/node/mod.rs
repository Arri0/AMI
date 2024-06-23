use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{deser::{DeserializationResult, SerializationError, SerializationResult}, midi, path::VirtualPaths};

use super::{command::{midi_filter::UpdateMidiFilterKind, ResponseCallback}, velocity_map};

pub mod sound_font_synth;

pub const NUM_USER_PRESETS: usize = 16;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RequestKind {
    SetName(String),
    SetEnabled(bool),
    LoadFile(PathBuf),
    SetGain(f32),
    SetTransposition(i8),
    SetVelocityMapping(velocity_map::Kind),
    SetIgnoreGlobalTransposition(bool),
    SetBankAndPreset(u8, u8),
    AddDrumMachineVoice,
    RemoveDrumMachineVoice(usize),
    ClearDrumMachineVoices,
    SetDrumMachineVoiceInstrument(usize, Option<usize>),
    SetDrumMachineVoiceNote(usize, u8),
    SetDrumMachineSlot(usize, usize, u8),
    UpdateMidiFilter(UpdateMidiFilterKind),
    SetUserPreset(usize),
    SetUserPresetEnabled(usize, bool),
}

pub type JsonFieldUpdate = (String, serde_json::Value);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JsonUpdateKind {
    InvalidId,
    Denied,
    Failed,
    UpdateFields(Vec<JsonFieldUpdate>),
}

pub fn update_fields_or_fail(
    callback: impl FnOnce(&mut Vec<JsonFieldUpdate>) -> Result<(), SerializationError>,
) -> JsonUpdateKind {
    let mut updates = Vec::with_capacity(1);
    if let Ok(()) = callback(&mut updates) {
        JsonUpdateKind::UpdateFields(updates)
    } else {
        JsonUpdateKind::Failed
    }
}

pub trait Render: Sync + Send {
    fn render_additive(&mut self, lbuf: &mut [f32], rbuf: &mut [f32]);
    fn reset_rendering(&mut self);
    fn set_virtual_paths(&mut self, vp: VirtualPaths);
    fn set_sample_rate(&mut self, sample_rate: u32);
    fn receive_midi_message(&mut self, message: &midi::Message);
    fn set_global_transposition(&mut self, transposition: i8);
    fn process_request(&mut self, kind: RequestKind, cb: ResponseCallback);
    fn serialize(&self) -> SerializationResult;
    fn deserialize(&mut self, source: &serde_json::Value) -> DeserializationResult;
    fn clone_node(&self) -> RenderPtr;
}

pub type RenderPtr = Box<dyn Render>;
