use crate::deser::SerializationError;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

pub type JsonFieldUpdate = (String, serde_json::Value);
pub type JsonUpdateSender = mpsc::Sender<(usize, JsonUpdateKind)>;
pub type JsonUpdateListener = mpsc::Receiver<(usize, JsonUpdateKind)>;

pub fn create_json_update_channel(buffer: usize) -> (JsonUpdateSender, JsonUpdateListener) {
    mpsc::channel(buffer)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JsonUpdateKind {
    InvalidId,
    Denied,
    Failed,
    Ok,
    UpdateFields(Vec<JsonFieldUpdate>),
}

pub fn update_fields_or_fail(
    callback: impl FnOnce(&mut Vec<JsonFieldUpdate>) -> Result<(), SerializationError>,
) -> JsonUpdateKind {
    let mut updates = Vec::with_capacity(1);
    if let Ok(()) = callback(&mut updates) {
        JsonUpdateKind::UpdateFields(updates)
    } else {
        JsonUpdateKind::Failed
    }
}

pub struct JsonUpdater {
    id: usize,
    tx: JsonUpdateSender,
}

impl JsonUpdater {
    pub fn new(id: usize, tx: JsonUpdateSender) -> Self {
        Self { id, tx }
    }

    pub async fn broadcast(&self, kind: JsonUpdateKind) {
        self.tx
            .send((self.id, kind))
            .await
            .unwrap_or_else(|e| tracing::error!("Error: {e}"));
    }
}