use crate::midi;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

pub mod drum_machine;
pub mod node;
pub mod controller;
pub mod voices;

pub const MAX_BUFFER_SIZE: usize = 192000;

pub type CtrSender = mpsc::Sender<ControlMessage>;
pub type CtrReceiver = mpsc::Receiver<ControlMessage>;

pub fn create_control_channel(buffer: usize) -> (CtrSender, CtrReceiver) {
    mpsc::channel(buffer)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlMessage {
    pub instrument_id: usize,
    pub midi_msg: midi::Message,
}