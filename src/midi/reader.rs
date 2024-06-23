use std::{error::Error, fmt};

use midir::MidiInput;

use super::{Message, Sender};

pub type Result<T> = std::result::Result<T, ReaderError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReaderError {
    ConnectError,
    InvalidSlot(usize),
}

impl Error for ReaderError {}

impl fmt::Display for ReaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReaderError::ConnectError => "Failed to connect MIDI port.".fmt(f),
            ReaderError::InvalidSlot(slot) => write!(f, "Invalid slot: {slot}"),
        }
    }
}

pub struct MidiReader {
    connections: Vec<Option<(String, midir::MidiInputConnection<()>)>>,
    tx: Sender,
}

impl MidiReader {
    pub fn with_slots(tx: Sender, num_of_slots: usize) -> Self {
        let mut connections = vec![];
        connections.resize_with(num_of_slots, || None);
        Self { connections, tx }
    }

    pub fn get_available_ports() -> Vec<String> {
        midir::MidiInput::new("")
            .map(get_available_ports_of)
            .unwrap_or_else(|_| vec![])
    }

    pub fn connect_input(&mut self, slot: usize, port_name: &str) -> Result<()> {
        if let Some(con) = self.connections.get_mut(slot) {
            let midi_in = midir::MidiInput::new("").map_err(|_| ReaderError::ConnectError)?;
            let index = get_port_index(&midi_in, port_name).ok_or(ReaderError::ConnectError)?;
            let conn = connect_midi_in_to_port(midi_in, index, self.tx.clone())?;
            *con = Some((port_name.into(), conn));
            Ok(())
        } else {
            Err(ReaderError::InvalidSlot(slot))
        }
    }

    pub fn disconnect_input(&mut self, slot: usize) -> Result<()> {
        if let Some(con) = self.connections.get_mut(slot) {
            *con = None;
            Ok(())
        } else {
            Err(ReaderError::InvalidSlot(slot))
        }
    }

    pub fn connected_input_names(&self) -> Vec<Option<String>> {
        self.connections
            .iter()
            .map(|opt| opt.as_ref().map(|(s, _)| s.clone()))
            .collect()
    }
}

fn get_available_ports_of(midi_in: MidiInput) -> Vec<String> {
    midi_in
        .ports()
        .iter()
        .filter_map(|port| midi_in.port_name(port).ok().clone())
        .collect()
}

fn get_port_index(midi_in: &MidiInput, port_name: &str) -> Option<usize> {
    midi_in.ports().iter().position(|port| {
        if let Ok(name) = midi_in.port_name(port) {
            name == port_name
        } else {
            false
        }
    })
}

fn connect_midi_in_to_port(
    midi_in: MidiInput,
    port_index: usize,
    tx: Sender,
) -> Result<midir::MidiInputConnection<()>> {
    let ports = midi_in.ports();
    midi_in
        .connect(
            &ports[port_index],
            "",
            move |_, message, _| {
                if let Some(msg) = Message::decode(message) {
                    _ = tx.send(msg);
                }
            },
            (),
        )
        .map_err(|_| ReaderError::ConnectError)
}
