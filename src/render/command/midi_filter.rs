use serde::{Serialize, Deserialize};


#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum UpdateMidiFilterKind {
    Enabled(bool),
    Channel(usize, bool),
    Note(usize, bool),
    ControlChange(usize, bool),
    ProgramChange(bool),
    ChannelAftertouch(bool),
    PitchWheel(bool),
}