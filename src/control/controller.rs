use super::node::{self, ControlPtr};
use crate::{
    control,
    json::{deser_field_opt, expect_serialize, DeserializationResult, JsonFieldUpdate},
    midi,
    path::VirtualPaths,
    rhythm::Rhythm,
    webserver::{Cache, Clients, ServerMessageKind},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, time::SystemTime};
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
    Reset,
    SetEnabled(bool),
    SetTempoBpm(f32),
    SetRhythm(Rhythm),
    SetUserPreset(usize),
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
    Enabled(bool),
    TempoBpm(f32),
    Rhythm(Rhythm),
    BeatState {
        beat: u8,
        div: u8,
    },
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

pub type NodeKindConstructor = Box<dyn Fn() -> ControlPtr + 'static + Sync + Send>;

pub struct Controller {
    enabled: bool,
    registered_node_kinds: HashMap<String, NodeKindConstructor>,
    nodes: Vec<(String, ControlPtr)>,
    midi_rx: midi::Receiver,
    req_rx: RequestListener,
    ctr_tx: control::CtrSender,
    tempo_bpm: f32,
    rhythm: Rhythm,
    virtual_paths: VirtualPaths,
    clients: Clients,
    cache: Cache,
    last_start: SystemTime,
    last_time: f32,
    current_beat: u8,
    current_div: u8,
}

impl Controller {
    pub fn new(
        midi_rx: midi::Receiver,
        req_rx: RequestListener,
        ctr_tx: control::CtrSender,
        virtual_paths: VirtualPaths,
        clients: Clients,
        cache: Cache,
    ) -> Self {
        let rhythm = Default::default();
        Self {
            enabled: false,
            registered_node_kinds: Default::default(),
            nodes: Default::default(),
            midi_rx,
            req_rx,
            ctr_tx,
            tempo_bpm: 90.0,
            rhythm,
            virtual_paths,
            clients,
            cache,
            last_start: SystemTime::now(),
            last_time: 0.0,
            current_beat: rhythm.num_beats - 1,
            current_div: rhythm.num_divs - 1,
        }
    }

    pub fn register_node_kind<F>(&mut self, name: &str, constructor: F)
    where
        F: Fn() -> ControlPtr + 'static + Sync + Send,
    {
        self.registered_node_kinds
            .insert(name.to_owned(), Box::new(constructor));
    }

    pub async fn update(&mut self) {
        self.receive_requests().await;
        self.receive_midi_messages();
        self.process_json_updates().await;

        if self.enabled {
            let time = self.timestamp();
            let period = self.period();
            if time - self.last_time >= period {
                self.advance_div();
                self.beat_tick(self.current_beat, self.current_div).await;
                self.last_time += period;
            }
        }
    }

    pub fn add_node(&mut self, kind: String, mut node: ControlPtr) {
        node.set_virtual_paths(self.virtual_paths.clone());
        node.set_rhythm(self.rhythm);
        node.set_tempo_bpm(self.tempo_bpm);
        node.set_control_sender(self.ctr_tx.clone());
        self.nodes.push((kind, node));
    }

    pub async fn receive_requests(&mut self) {
        while let Ok((kind, responder)) = self.req_rx.try_recv() {
            self.process_request(kind, responder).await;
        }
    }

    pub async fn deserialize(&mut self, source: &serde_json::Value) -> DeserializationResult {
        deser_field_opt(source, "enabled", |v| self.enabled = v)?;
        deser_field_opt(source, "tempo_bpm", |v| self.tempo_bpm = v)?;
        deser_field_opt(source, "rhythm", |v| self.rhythm = v)?;
        self.cache.set_controller_enabled(self.enabled).await;
        self.cache.set_controller_tempo_bpm(self.tempo_bpm).await;
        self.cache.set_controller_rhythm(self.rhythm).await;
        self.broadcast_update(UpdateKind::Enabled(self.enabled));
        self.broadcast_update(UpdateKind::TempoBpm(self.tempo_bpm));
        self.broadcast_update(UpdateKind::Rhythm(self.rhythm));
        Ok(())
    }

    pub async fn serialize(&self) -> serde_json::Value {
        json!({
            "enabled": expect_serialize(self.enabled),
            "tempo_bpm": expect_serialize(self.tempo_bpm),
            "rhythm": expect_serialize(self.rhythm),
        })
    }

    fn receive_midi_messages(&mut self) {
        while let Ok(msg) = self.midi_rx.try_recv() {
            for (_, node) in &mut self.nodes {
                node.receive_midi_message(&msg)
            }
        }
    }

    async fn process_json_updates(&mut self) {
        for (id, node) in self.nodes.iter_mut().enumerate() {
            if let Some(updates) = node.1.json_updates() {
                self.cache.control_node_updates(id, &updates).await;
                self.clients.broadcast(ServerMessageKind::ControllerUpdate(
                    UpdateKind::NodeUpdates { id, updates },
                ));
            }
        }
    }

    async fn process_request(&mut self, kind: RequestKind, responder: Responder) {
        match kind {
            RequestKind::Reset => {
                respond(responder, ResponseKind::Ok);
                self.reset();
            }
            RequestKind::SetEnabled(enabled) => {
                respond(responder, ResponseKind::Ok);
                self.set_enabled(enabled).await;
            }
            RequestKind::SetTempoBpm(tempo_bpm) => {
                respond(responder, ResponseKind::Ok);
                self.set_tempo_bpm(tempo_bpm).await;
            }
            RequestKind::SetRhythm(rhythm) => {
                respond(responder, ResponseKind::Ok);
                self.set_rhythm(rhythm).await;
            }
            RequestKind::SetUserPreset(preset) => {
                if preset < node::NUM_USER_PRESETS {
                    self.set_user_preset(preset);
                    respond(responder, ResponseKind::Ok);
                } else {
                    respond(responder, ResponseKind::Failed);
                }
            }
            RequestKind::NodeRequest { id, kind } => self.process_node_request(responder, id, kind),
            RequestKind::AddNode { kind } => self.process_add_node(responder, kind).await,
            RequestKind::RemoveNode { id } => self.process_remove_node(responder, id).await,
            RequestKind::CloneNode { id } => self.process_clone_node(responder, id).await,
            RequestKind::MoveNode { id, new_id } => {
                self.process_move_node(responder, id, new_id).await
            }
        }
    }

    fn set_user_preset(&mut self, preset: usize) {
        for (_, node) in &mut self.nodes {
            node.set_user_preset(preset);
        }
    }

    async fn set_enabled(&mut self, flag: bool) {
        self.enabled = flag;
        if flag {
            self.reset();
        }
        self.cache.set_controller_enabled(flag).await;
        self.broadcast_update(UpdateKind::Enabled(flag));
    }

    async fn set_tempo_bpm(&mut self, tempo_bpm: f32) {
        self.tempo_bpm = tempo_bpm;

        for node in &mut self.nodes {
            node.1.set_tempo_bpm(tempo_bpm);
        }

        self.cache.set_controller_tempo_bpm(tempo_bpm).await;
        self.broadcast_update(UpdateKind::TempoBpm(tempo_bpm));
    }

    async fn set_rhythm(&mut self, rhythm: Rhythm) {
        self.rhythm = rhythm;
        self.reset();

        for node in &mut self.nodes {
            node.1.set_rhythm(rhythm);
        }
        self.cache.set_controller_rhythm(rhythm).await;
        self.broadcast_update(UpdateKind::Rhythm(rhythm));
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

        let node: ControlPtr = self.registered_node_kinds[&kind]();
        if let Ok(value) = node.serialize() {
            self.add_node(kind.clone(), node);
            self.cache.add_control_node(&kind, &value).await;
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
            self.cache.remove_control_node(id).await;
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
            self.cache.clone_control_node(id).await;
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
            self.cache.move_control_node(id, new_id).await;
            respond(responder, ResponseKind::Ok);
            self.broadcast_update(UpdateKind::MoveNode { id, new_id });
        }
    }

    fn reset(&mut self) {
        self.last_start = SystemTime::now();
        self.last_time = self.timestamp() - self.period();
        self.current_beat = self.rhythm.num_beats - 1;
        self.current_div = self.rhythm.num_divs - 1;

        for node in &mut self.nodes {
            node.1.reset();
        }

        self.broadcast_update(UpdateKind::BeatState {
            beat: self.current_beat,
            div: self.current_div,
        });
    }

    async fn beat_tick(&mut self, beat_num: u8, div_num: u8) {
        for node in &mut self.nodes {
            node.1.beat_tick(beat_num, div_num).await;
        }
        self.broadcast_update(UpdateKind::BeatState {
            beat: beat_num,
            div: div_num,
        });
    }

    fn broadcast_update(&mut self, kind: UpdateKind) {
        self.clients
            .broadcast(ServerMessageKind::ControllerUpdate(kind));
    }

    pub fn period(&self) -> f32 {
        60.0 / (self.tempo_bpm * self.rhythm.num_divs as f32)
    }

    fn advance_div(&mut self) {
        self.current_div = (self.current_div + 1) % self.rhythm.num_divs;
        if self.current_div == 0 {
            self.advance_beat();
        }
    }

    fn advance_beat(&mut self) {
        self.current_beat = (self.current_beat + 1) % self.rhythm.num_beats;
    }

    fn timestamp(&self) -> f32 {
        let duration = self
            .last_start
            .elapsed()
            .expect("Unexpected error while getting timestamp");
        duration.as_secs_f32()
    }
}

fn respond(responder: Responder, response_kind: ResponseKind) {
    if let Err(e) = responder.send(response_kind) {
        error!("Failed to send a response: {e:?}");
    }
}
