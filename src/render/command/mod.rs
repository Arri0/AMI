use crate::json::JsonUpdateKind;
use crate::render::node;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::sync::oneshot;

pub mod midi_filter;

pub type Requester = mpsc::Sender<(RequestKind, Responder)>;
pub type RequestListener = mpsc::Receiver<(RequestKind, Responder)>;
pub type Responder = oneshot::Sender<ResponseKind>;
pub type ResponseListener = oneshot::Receiver<ResponseKind>;
pub type ResponseCallback = Box<dyn FnOnce(JsonUpdateKind) + 'static + Send + Sync>;

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
    NodeResponse {
        id: usize,
        kind: JsonUpdateKind,
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
}
