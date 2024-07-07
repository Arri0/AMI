use serde::{Deserialize, Serialize};

use crate::midi;

const NUM_CHANNELS: usize = 16;
const NUM_NOTES: usize = 128;
const NUM_CONTROL_COMMANDS: usize = 128;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum UpdateKind {
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct MidiFilter {
    pub enabled: bool,
    pub channels: Vec<bool>,
    pub notes: Vec<bool>,
    pub control_commands: Vec<bool>,
    pub program_change: bool,
    pub channel_aftertouch: bool,
    pub pitch_wheel: bool,
}

impl MidiFilter {
    pub fn does_pass(&self, message: &midi::Message) -> bool {
        if !self.enabled {
            true
        } else {
            self.does_pass_when_enabled(message)
        }
    }

    fn does_pass_when_enabled(&self, message: &midi::Message) -> bool {
        if !self.channels[message.channel as usize] {
            return false;
        }
        match message.kind {
            midi::MessageKind::NoteOn { note, .. } => self.notes[note as usize],
            midi::MessageKind::NoteOff { .. } => true,
            midi::MessageKind::PolyphonicAftertouch { note, .. } => self.notes[note as usize],
            midi::MessageKind::ControlChange { kind, .. } => {
                self.control_commands[kind.as_number() as usize]
            }
            midi::MessageKind::ProgramChange { .. } => self.program_change,
            midi::MessageKind::ChannelAftertouch { .. } => self.channel_aftertouch,
            midi::MessageKind::PitchWheel { .. } => self.pitch_wheel,
        }
    }
}

impl Default for MidiFilter {
    fn default() -> Self {
        Self {
            enabled: true,
            channels: vec![true; NUM_CHANNELS],
            notes: vec![true; NUM_NOTES],
            control_commands: vec![true; NUM_CONTROL_COMMANDS],
            program_change: true,
            channel_aftertouch: true,
            pitch_wheel: true,
        }
    }
}

pub struct InvalidUpdateRequest;
pub type UpdateResult = Result<(), InvalidUpdateRequest>;

pub trait MidiFilterUser {
    fn midi_filter_mut(&mut self) -> &mut MidiFilter;

    fn process_update_request(&mut self, kind: UpdateKind) -> UpdateResult {
        let f = self.midi_filter_mut();
        match kind {
            UpdateKind::Enabled(flag) => f.enabled = flag,
            UpdateKind::Channel(c, fl) => ur_set_channel(f, c, fl)?,
            UpdateKind::Channels(channels) => ur_set_channels(f, channels)?,
            UpdateKind::Note(n, fl) => ur_set_note(f, n, fl)?,
            UpdateKind::Notes(notes) => ur_set_notes(f, notes)?,
            UpdateKind::ControlChange(cc, fl) => ur_set_cc(f, cc, fl)?,
            UpdateKind::ControlChanges(ccs) => ur_set_ccs(f, ccs)?,
            UpdateKind::ProgramChange(fl) => f.program_change = fl,
            UpdateKind::ChannelAftertouch(fl) => f.channel_aftertouch = fl,
            UpdateKind::PitchWheel(fl) => f.pitch_wheel = fl,
        }
        Ok(())
    }
}

fn ur_set_channel(filter: &mut MidiFilter, channel: usize, flag: bool) -> UpdateResult {
    if channel < filter.channels.len() {
        filter.channels[channel] = flag;
        Ok(())
    } else {
        Err(InvalidUpdateRequest)
    }
}

fn ur_set_channels(filter: &mut MidiFilter, channels: Vec<bool>) -> UpdateResult {
    if channels.len() == filter.channels.len() {
        filter.channels = channels;
        Ok(())
    } else {
        Err(InvalidUpdateRequest)
    }
}

fn ur_set_note(filter: &mut MidiFilter, note: usize, flag: bool) -> UpdateResult {
    if note < filter.notes.len() {
        filter.notes[note] = flag;
        Ok(())
    } else {
        Err(InvalidUpdateRequest)
    }
}

fn ur_set_notes(filter: &mut MidiFilter, notes: Vec<bool>) -> UpdateResult {
    if notes.len() == filter.notes.len() {
        filter.notes = notes;
        Ok(())
    } else {
        Err(InvalidUpdateRequest)
    }
}

fn ur_set_cc(filter: &mut MidiFilter, cc: usize, flag: bool) -> UpdateResult {
    if cc < filter.control_commands.len() {
        filter.control_commands[cc] = flag;
        Ok(())
    } else {
        Err(InvalidUpdateRequest)
    }
}

fn ur_set_ccs(filter: &mut MidiFilter, ccs: Vec<bool>) -> UpdateResult {
    if ccs.len() == filter.control_commands.len() {
        filter.control_commands = ccs;
        Ok(())
    } else {
        Err(InvalidUpdateRequest)
    }
}
