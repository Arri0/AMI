use super::{midi_filter, velocity_map};
use crate::{
    json::{DeserializationResult, JsonFieldUpdate, SerializationResult},
    midi,
    path::VirtualPaths,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod fluidlite_synth;
pub mod oxi_synth;
pub mod rusty_synth;
pub mod sfizz_synth;

pub const NUM_USER_PRESETS: usize = 16;

pub type ResponseCallback = Box<dyn FnOnce(ResponseKind) + 'static + Send + Sync>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RequestKind {
    SetName(String),
    SetEnabled(bool),
    LoadFile(PathBuf),
    SetGain(f32),
    SetTransposition(i8),
    SetVelocityMapping(velocity_map::Kind),
    SetIgnoreGlobalTransposition(bool),
    SetBankAndPreset(u16, u8),
    MidiMessage(midi::MessageKind),
    SetSfReverbActive(bool),
    SetSfReverbParams {
        room_size: f32,
        damping: f32,
        width: f32,
        level: f32,
    },
    AddDrumMachineVoice,
    RemoveDrumMachineVoice(usize),
    ClearDrumMachineVoices,
    SetDrumMachineVoiceInstrument(usize, Option<usize>),
    SetDrumMachineVoiceNote(usize, u8),
    SetDrumMachineSlot(usize, usize, u8),
    UpdateMidiFilter(midi_filter::UpdateKind),
    SetUserPresetEnabled(usize, bool),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResponseKind {
    InvalidId,
    Denied,
    Failed,
    Ok,
}

pub trait Render: Sync + Send {
    fn render_additive(&mut self, lbuf: &mut [f32], rbuf: &mut [f32]);
    fn reset_rendering(&mut self);
    fn set_virtual_paths(&mut self, vp: VirtualPaths);
    fn set_sample_rate(&mut self, sample_rate: u32);
    fn receive_midi_message(&mut self, message: &midi::Message);
    fn set_global_transposition(&mut self, transposition: i8);
    fn set_user_preset(&mut self, preset: usize);
    fn process_request(&mut self, kind: RequestKind, cb: ResponseCallback);
    fn serialize(&self) -> SerializationResult; //TODO: return serde_json::Value instead
    fn deserialize(&mut self, source: &serde_json::Value) -> DeserializationResult;
    fn json_updates(&mut self) -> Option<Vec<JsonFieldUpdate>>;
    fn clone_node(&self) -> RenderPtr;
}

pub type RenderPtr = Box<dyn Render>;
