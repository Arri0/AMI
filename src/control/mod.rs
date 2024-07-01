use crate::{midi, path::VirtualPaths};
use command::{RequestKind, Responder, ResponseKind};
use node::ControlPtr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::error;

pub mod command;
pub mod drum_machine;
pub mod node;

pub const MAX_BUFFER_SIZE: usize = 192000;

pub type CtrSender = mpsc::Sender<ControlMessage>;
pub type CtrReceiver = mpsc::Receiver<ControlMessage>;

pub fn create_control_channel(buffer: usize) -> (CtrSender, CtrReceiver) {
    mpsc::channel(buffer)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlMessage {
    pub instrument_id: usize,
    pub channel: u8,
    pub note: u8,
    pub velocity: u8,
}

pub type NodeKindConstructor = Box<dyn Fn() -> ControlPtr + 'static + Sync + Send>;

pub struct Controller {
    registered_node_kinds: HashMap<String, NodeKindConstructor>,
    nodes: Vec<(String, ControlPtr)>,
    midi_rx: midi::Receiver,
    req_rx: command::RequestListener,
    virtual_paths: VirtualPaths,
}

impl Controller {
    pub fn new(
        midi_rx: midi::Receiver,
        req_rx: command::RequestListener,
        virtual_paths: VirtualPaths,
    ) -> Self {
        Self {
            registered_node_kinds: Default::default(),
            nodes: Default::default(),
            midi_rx,
            req_rx,
            virtual_paths,
        }
    }

    pub fn register_node_kind<F>(&mut self, name: &str, constructor: F)
    where
        F: Fn() -> ControlPtr + 'static + Sync + Send,
    {
        self.registered_node_kinds
            .insert(name.to_owned(), Box::new(constructor));
    }

    pub fn tick(&mut self) {
        self.receive_requests();
        self.receive_midi_messages();
    }

    pub fn add_node(&mut self, kind: String, mut node: ControlPtr) {
        node.set_virtual_paths(self.virtual_paths.clone());
        self.nodes.push((kind, node));
    }

    pub fn receive_requests(&mut self) {
        while let Ok((kind, responder)) = self.req_rx.try_recv() {
            self.process_request(kind, responder);
        }
    }

    fn receive_midi_messages(&mut self) {
        while let Ok(msg) = self.midi_rx.try_recv() {
            for (_, node) in &mut self.nodes {
                node.receive_midi_message(&msg)
            }
        }
    }

    fn process_request(&mut self, kind: RequestKind, responder: Responder) {
        match kind {
            RequestKind::NodeRequest { id, kind } => {
                if id >= self.nodes.len() {
                    respond(responder, ResponseKind::InvalidId);
                } else {
                    let cb =
                        move |kind| respond(responder, ResponseKind::NodeResponse { id, kind });
                    self.nodes[id].1.process_request(kind, Box::new(cb));
                }
            }
            RequestKind::AddNode { kind } => {
                if !self.registered_node_kinds.contains_key(&kind) {
                    respond(responder, ResponseKind::InvalidNodeKind);
                    return;
                }

                let node: ControlPtr = self.registered_node_kinds[&kind]();
                if let Ok(value) = node.serialize() {
                    self.add_node(kind.clone(), node);
                    respond(
                        responder,
                        ResponseKind::AddNode {
                            id: self.nodes.len() - 1,
                            kind,
                            instance: value,
                        },
                    );
                } else {
                    respond(responder, ResponseKind::Failed);
                }
            }
            RequestKind::RemoveNode { id } => {
                if id >= self.nodes.len() {
                    respond(responder, ResponseKind::InvalidId);
                } else {
                    self.nodes.remove(id);
                    respond(responder, ResponseKind::RemoveNode { id })
                }
            }
            RequestKind::CloneNode { id } => {
                if id >= self.nodes.len() {
                    respond(responder, ResponseKind::InvalidId)
                } else {
                    let node = &self.nodes[id];
                    self.add_node(node.0.clone(), node.1.clone_node());
                    respond(responder, ResponseKind::CloneNode { id })
                }
            }
            RequestKind::MoveNode { id, new_id } => todo!(),
        }
    }
}

fn respond(responder: Responder, response_kind: ResponseKind) {
    if let Err(e) = responder.send(response_kind) {
        error!("Failed to send a response: {e:?}");
    }
}
