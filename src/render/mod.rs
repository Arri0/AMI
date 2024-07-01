use crate::{control, midi, path::VirtualPaths};
use command::{RequestKind, Responder, ResponseKind};
use node::RenderPtr;
use std::collections::HashMap;
use tracing::error;

pub mod command;
pub mod midi_filter;
pub mod node;
pub mod preset_map;
pub mod velocity_map;

pub const MAX_BUFFER_SIZE: usize = 192000;

pub type NodeKindConstructor = Box<dyn Fn() -> RenderPtr + 'static + Sync + Send>;

pub struct Renderer {
    registered_node_kinds: HashMap<String, NodeKindConstructor>,
    nodes: Vec<(String, RenderPtr)>,
    midi_rx: midi::Receiver,
    req_rx: command::RequestListener,
    dm_ctr_rx: control::CtrReceiver,
    sample_rate: Option<u32>,
    global_transposition: i8,
    virtual_paths: VirtualPaths,
}

impl Renderer {
    pub fn new(
        midi_rx: midi::Receiver,
        req_rx: command::RequestListener,
        dm_ctr_rx: control::CtrReceiver,
        virtual_paths: VirtualPaths,
    ) -> Self {
        Self {
            registered_node_kinds: Default::default(),
            nodes: Default::default(),
            midi_rx,
            req_rx,
            dm_ctr_rx,
            sample_rate: None,
            global_transposition: 0,
            virtual_paths,
        }
    }

    pub fn register_node_kind<F>(&mut self, name: &str, constructor: F)
    where
        F: Fn() -> RenderPtr + 'static + Sync + Send,
    {
        self.registered_node_kinds
            .insert(name.to_owned(), Box::new(constructor));
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = Some(sample_rate);
        for (_, node) in &mut self.nodes {
            node.set_sample_rate(sample_rate);
        }
    }

    pub fn set_global_transposition(&mut self, transposition: i8) {
        self.global_transposition = transposition;
        for (_, node) in &mut self.nodes {
            node.set_global_transposition(transposition);
        }
    }

    pub fn render(&mut self, lbuf: &mut [f32], rbuf: &mut [f32]) {
        self.receive_requests();
        self.receive_midi_messages();
        self.receive_drum_machine_messages();
        self.render_audio(lbuf, rbuf);
    }

    pub fn add_node(&mut self, kind: String, mut node: RenderPtr) {
        if let Some(sample_rate) = self.sample_rate {
            node.set_sample_rate(sample_rate);
        }
        node.set_virtual_paths(self.virtual_paths.clone());
        node.set_global_transposition(self.global_transposition);
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

    fn receive_drum_machine_messages(&mut self) {
        while let Ok(msg) = self.dm_ctr_rx.try_recv() {
            let node_id = msg.instrument_id;
            if node_id < self.nodes.len() {
                let node = &mut self.nodes[node_id].1;
                if msg.velocity > 0 {
                    let msg = midi::Message {
                        kind: midi::MessageKind::NoteOn {
                            note: msg.note,
                            velocity: msg.velocity,
                        },
                        channel: msg.channel,
                    };
                    node.receive_midi_message(&msg);
                } else {
                    let msg = midi::Message {
                        kind: midi::MessageKind::NoteOff {
                            note: msg.note,
                            velocity: 0,
                        },
                        channel: msg.channel,
                    };
                    node.receive_midi_message(&msg);
                }
            }
        }
    }

    fn render_audio(&mut self, lbuf: &mut [f32], rbuf: &mut [f32]) {
        lbuf.fill(0.0);
        rbuf.fill(0.0);
        for (_, node) in &mut self.nodes {
            node.render_additive(lbuf, rbuf)
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

                let node: RenderPtr = self.registered_node_kinds[&kind]();
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

pub fn amplify_buffer(buffer: &mut [f32], gain: f32) {
    if gain != 1.0 {
        buffer.iter_mut().for_each(|x| *x *= gain);
    }
}

pub fn clear_buffer(buffer: &mut [f32]) {
    buffer.fill(0.0);
}

pub fn render_nodes_to_bufs(nodes: &mut [RenderPtr], lbuf: &mut [f32], rbuf: &mut [f32]) {
    nodes
        .iter_mut()
        .for_each(|child| child.render_additive(lbuf, rbuf));
}

pub fn add_buf_to_buf(buffer: &mut [f32], tmp_buffer: &[f32]) {
    let len = usize::min(buffer.len(), tmp_buffer.len());
    for i in 0..len {
        buffer[i] += tmp_buffer[i];
    }
}

fn respond(responder: Responder, response_kind: ResponseKind) {
    if let Err(e) = responder.send(response_kind) {
        error!("Failed to send a response: {e:?}");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn amplify_buffer() {
        let gain = 3.2;
        let mut buffer = [1.0, 0.0, 3.2];
        super::amplify_buffer(&mut buffer, gain);
        assert_eq!(buffer, [1.0 * gain, 0.0 * gain, 3.2 * gain])
    }
}
