use serde::{Serialize, Deserialize};


#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum UpdateMidiFilterKind {
    Enabled(bool),
    Channel(usize, bool),
    Channels(Vec<bool>),
    Note(usize, bool),
    Notes(Vec<bool>),
    ControlChange(usize, bool),
    ControlChanges(Vec<bool>),
    ProgramChange(bool),
    ChannelAftertouch(bool),
    PitchWheel(bool),
}