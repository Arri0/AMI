use crate::render::node;
use crate::{
    control,
    json::JsonFieldUpdate,
    midi,
    path::VirtualPaths,
    webserver::{Cache, Clients, ServerMessageKind},
};
use node::RenderPtr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tracing::error;

pub type Requester = mpsc::Sender<(RequestKind, Responder)>;
pub type RequestListener = mpsc::Receiver<(RequestKind, Responder)>;
pub type Responder = oneshot::Sender<ResponseKind>;
pub type ResponseListener = oneshot::Receiver<ResponseKind>;

pub fn create_request_channel(buffer: usize) -> (Requester, RequestListener) {
    mpsc::channel(buffer)
}

pub fn create_response_channel() -> (Responder, ResponseListener) {
    oneshot::channel()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RequestKind {
    NodeRequest { id: usize, kind: node::RequestKind },
    AddNode { kind: String },
    RemoveNode { id: usize },
    CloneNode { id: usize },
    MoveNode { id: usize, new_id: usize },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResponseKind {
    InvalidNodeKind,
    InvalidId,
    Denied,
    Failed,
    Ok,
    NodeResponse { id: usize, kind: node::ResponseKind },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UpdateKind {
    AddNode {
        id: usize,
        kind: String,
        instance: serde_json::Value,
    },
    RemoveNode {
        id: usize,
    },
    CloneNode {
        id: usize,
    },
    MoveNode {
        id: usize,
        new_id: usize,
    },
    NodeUpdates {
        id: usize,
        updates: Vec<JsonFieldUpdate>,
    },
}

pub type NodeKindConstructor = Box<dyn Fn() -> RenderPtr + 'static + Sync + Send>;

pub struct Renderer {
    registered_node_kinds: HashMap<String, NodeKindConstructor>,
    nodes: Vec<(String, RenderPtr)>,
    midi_rx: midi::Receiver,
    req_rx: RequestListener,
    dm_ctr_rx: control::CtrReceiver,
    sample_rate: Option<u32>,
    global_transposition: i8,
    virtual_paths: VirtualPaths,
    clients: Clients,
    cache: Cache,
}

impl Renderer {
    pub fn new(
        midi_rx: midi::Receiver,
        req_rx: RequestListener,
        dm_ctr_rx: control::CtrReceiver,
        virtual_paths: VirtualPaths,
        clients: Clients,
        cache: Cache,
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
            clients,
            cache,
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

    pub async fn update(&mut self) {
        self.receive_requests().await;
        self.receive_midi_messages();
        self.receive_drum_machine_messages();
        self.process_json_updates().await;
    }

    pub fn render(&mut self, lbuf: &mut [f32], rbuf: &mut [f32]) {
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

    pub async fn receive_requests(&mut self) {
        while let Ok((kind, responder)) = self.req_rx.try_recv() {
            self.process_request(kind, responder).await;
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

    async fn process_json_updates(&mut self) {
        for (id, node) in self.nodes.iter_mut().enumerate() {
            if let Some(updates) = node.1.json_updates() {
                self.cache.render_node_updates(id, &updates).await;
                self.clients.broadcast(ServerMessageKind::RendererUpdate(
                    UpdateKind::NodeUpdates { id, updates },
                ));
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

    async fn process_request(&mut self, kind: RequestKind, responder: Responder) {
        match kind {
            RequestKind::NodeRequest { id, kind } => self.process_node_request(responder, id, kind),
            RequestKind::AddNode { kind } => self.process_add_node(responder, kind).await,
            RequestKind::RemoveNode { id } => self.process_remove_node(responder, id).await,
            RequestKind::CloneNode { id } => self.process_clone_node(responder, id).await,
            RequestKind::MoveNode { id, new_id } => {
                self.process_move_node(responder, id, new_id).await
            }
        }
    }

    fn process_node_request(&mut self, responder: Responder, id: usize, kind: node::RequestKind) {
        if id >= self.nodes.len() {
            respond(responder, ResponseKind::InvalidId);
        } else {
            let cb = move |kind| respond(responder, ResponseKind::NodeResponse { id, kind });
            self.nodes[id].1.process_request(kind, Box::new(cb));
        }
    }

    async fn process_add_node(&mut self, responder: Responder, kind: String) {
        if !self.registered_node_kinds.contains_key(&kind) {
            respond(responder, ResponseKind::InvalidNodeKind);
            return;
        }

        let node: RenderPtr = self.registered_node_kinds[&kind]();
        if let Ok(value) = node.serialize() {
            self.add_node(kind.clone(), node);
            self.cache.add_render_node(&kind, &value).await;
            respond(responder, ResponseKind::Ok);
            self.broadcast_update(UpdateKind::AddNode {
                id: self.nodes.len() - 1,
                kind,
                instance: value,
            });
        } else {
            respond(responder, ResponseKind::Failed);
        }
    }

    async fn process_remove_node(&mut self, responder: Responder, id: usize) {
        if id >= self.nodes.len() {
            respond(responder, ResponseKind::InvalidId);
        } else {
            self.nodes.remove(id);
            self.cache.remove_render_node(id).await;
            respond(responder, ResponseKind::Ok);
            self.broadcast_update(UpdateKind::RemoveNode { id });
        }
    }

    async fn process_clone_node(&mut self, responder: Responder, id: usize) {
        if id >= self.nodes.len() {
            respond(responder, ResponseKind::InvalidId);
        } else {
            let node = &self.nodes[id];
            self.add_node(node.0.clone(), node.1.clone_node());
            self.cache.clone_render_node(id).await;
            respond(responder, ResponseKind::Ok);
            self.broadcast_update(UpdateKind::CloneNode { id });
        }
    }

    async fn process_move_node(&mut self, responder: Responder, id: usize, new_id: usize) {
        if id >= self.nodes.len() || new_id >= self.nodes.len() {
            respond(responder, ResponseKind::InvalidId);
        } else {
            let node = self.nodes.remove(id);
            self.nodes.insert(new_id, node);
            self.cache.move_render_node(id, new_id).await;
            respond(responder, ResponseKind::Ok);
            self.broadcast_update(UpdateKind::MoveNode { id, new_id });
        }
    }

    fn broadcast_update(&mut self, kind: UpdateKind) {
        self.clients
            .broadcast(ServerMessageKind::RendererUpdate(kind));
    }
}

fn respond(responder: Responder, response_kind: ResponseKind) {
    if let Err(e) = responder.send(response_kind) {
        error!("Failed to send a response: {e:?}");
    }
}
