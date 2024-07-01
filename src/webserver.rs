use crate::{
    control::drum_machine, json::JsonUpdateKind, midi::{self, MidiReader}, render::command
};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    http::Method,
    response::IntoResponse,
    routing::get,
};
use axum_embed::ServeEmbed;
use axum_extra::{headers, TypedHeader};
use futures::{stream::SplitSink, Future, SinkExt, StreamExt};
use rust_embed::Embed;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::sync::{broadcast, Mutex};
use tower_http::cors::CorsLayer;
use tracing::{error, info, warn};

#[derive(Embed, Clone)]
#[folder = "client/build/"]
struct WebClientAssets;

#[derive(Clone)]
pub struct SharedState {
    pub clients: Clients,
    pub midi_reader: Arc<Mutex<MidiReader>>,
    pub cache: Arc<Mutex<Cache>>,
}

pub async fn run<F, Fut>(http_port: u16, state: SharedState, req_handler: F)
where
    F: FnMut(SocketAddr, ClientMessageKind) -> Fut + Send + Sync + Clone + 'static,
    Fut: Future<Output = ServerMessageKind> + Send + 'static,
{
    let cors = CorsLayer::new()
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::HEAD,
            Method::OPTIONS,
            Method::CONNECT,
            Method::PATCH,
            Method::TRACE,
        ])
        .allow_origin(tower_http::cors::Any);

    let wc_assets = ServeEmbed::<WebClientAssets>::new();

    let app = axum::Router::new()
        .fallback_service(wc_assets)
        .route("/ws", get(ws_handler))
        .layer(cors)
        // .layer(
        //     TraceLayer::new_for_http()
        //         .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        // )
        .with_state((state, req_handler));
    // .route("/", get(|| async { "Hello, World!" }))

    info!("Starting server on http://localhost:{http_port}/");

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{http_port}"))
        .await
        .unwrap();

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn ws_handler<F, Fut>(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State((state, req_handler)): State<(SharedState, F)>,
) -> impl IntoResponse
where
    F: FnMut(SocketAddr, ClientMessageKind) -> Fut + Send + Sync + Clone + 'static,
    Fut: Future<Output = ServerMessageKind> + Send + 'static,
{
    let _user_agent = user_agent
        .map(|TypedHeader(user_agent)| user_agent.to_string())
        .unwrap_or_else(|| String::from("Unknown browser"));
    info!(
        "New connection from {addr}. (clients connected: {})",
        state.clients.len().await + 1
    );
    ws.on_upgrade(move |socket| handle_socket(socket, addr, state, req_handler))
}

async fn handle_socket<F, Fut>(
    socket: WebSocket,
    addr: SocketAddr,
    state: SharedState,
    mut req_handler: F,
) where
    F: FnMut(SocketAddr, ClientMessageKind) -> Fut + Send + Sync + Clone + 'static,
    Fut: Future<Output = ServerMessageKind> + Send + 'static,
{
    let (tx, mut rx) = socket.split();
    let mut brd_rx = state.clients.tx.subscribe();
    let mut clients = state.clients;
    let midi_reader = state.midi_reader;
    clients.push(Client { addr }).await;
    let tx = Arc::new(Mutex::new(tx));
    let tx2 = Arc::clone(&tx);

    send_broadcast(
        &mut *tx.lock().await,
        ServerMessageKind::ConnectedMidiInputs(midi_reader.lock().await.connected_input_names()),
    )
    .await;

    send_broadcast(
        &mut *tx.lock().await,
        ServerMessageKind::Cache(state.cache.lock().await.get().clone()),
    )
    .await;

    tokio::select! {
        _ = async move {
            while let Ok(msg) = brd_rx.recv().await {
                // tracing::trace!("Sending broadcast message to a client at {addr}: {msg:?}");
                send_raw_msg(&mut *tx.lock().await, msg).await;
            }
        } => {},
        _ = async move {
            while let Some(Ok(msg)) = rx.next().await {
                match msg {
                    Message::Text(msg) => {
                        if let Ok(msg) = serde_json::from_str::<ClientMessage>(&msg) {
                            send_msg(&mut *tx2.lock().await, ServerMessage {
                                id: msg.id,
                                response: true,
                                payload: req_handler(addr, msg.payload).await,
                            }).await;
                        } else {
                            warn!("Invalid message from {addr}: {msg}");
                        }
                    }
                    Message::Close(_) => {
                        break;
                    }
                    _ => {}
                }
            }
        } => {},
    };

    clients.remove(addr).await;
    info!(
        "Client at {addr} disconnected. (clients connected: {})",
        clients.len().await
    );
}

#[derive(Debug)]
pub struct Client {
    pub addr: SocketAddr,
}

pub async fn send_raw_msg(tx: &mut SplitSink<WebSocket, Message>, msg: Message) {
    tx.send(msg)
        .await
        .unwrap_or_else(|e| error!("Send error: {e}"));
}

pub async fn send_msg(tx: &mut SplitSink<WebSocket, Message>, msg: ServerMessage) {
    let msg = serde_json::to_string(&msg).expect("Failed to serialize server message");
    let msg = Message::Text(msg);
    send_raw_msg(tx, msg).await;
}

pub async fn send_broadcast(tx: &mut SplitSink<WebSocket, Message>, msg: ServerMessageKind) {
    let msg = ServerMessage {
        id: 0,
        response: false,
        payload: msg,
    };
    send_msg(tx, msg).await;
}

#[derive(Debug, Clone)]
pub struct Clients {
    // thread safe struct of Clients, can be cloned
    clients: Arc<Mutex<Vec<Client>>>,
    tx: broadcast::Sender<Message>,
}

impl Clients {
    pub fn new(broadcast_channel_capacity: usize) -> Self {
        let (tx, _) = broadcast::channel::<Message>(broadcast_channel_capacity);
        Self {
            clients: Default::default(),
            tx,
        }
    }

    pub async fn len(&self) -> usize {
        let clients = self.clients.lock().await;
        clients.len()
    }

    pub async fn push(&mut self, client: Client) {
        let mut clients = self.clients.lock().await;
        clients.push(client);
    }

    pub async fn remove(&mut self, addr: SocketAddr) {
        let mut clients = self.clients.lock().await;
        clients.retain(|c| c.addr != addr);
    }

    pub fn broadcast(&mut self, payload: ServerMessageKind) {
        if self.tx.receiver_count() == 0 {
            return;
        }

        let msg = ServerMessage {
            id: 0,
            response: false,
            payload,
        };
        let msg = serde_json::to_string(&msg).expect("Failed to serialize server message");
        let msg = Message::Text(msg);

        self.tx.send(msg).unwrap_or_else(|e| {
            error!("Broadcast error: {e}");
            0
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessageKind {
    Pong,
    Ack,
    Nak,
    Log(String),
    MidiEvent(midi::Message),
    AvailableMidiInputs(Vec<String>),
    ConnectedMidiInputs(Vec<Option<String>>),
    Cache(serde_json::Value),
    RendererResponse(command::ResponseKind),
    DirInfo(Option<Vec<(bool, PathBuf)>>), // (is_dir, path)
    DrumMachineUpdate(JsonUpdateKind),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerMessage {
    id: usize,
    response: bool,
    payload: ServerMessageKind,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessageKind {
    Ping,
    Report(String),
    ConnectMidiInput(usize, String),
    DisconnectMidiInput(usize),
    RendererRequest(command::RequestKind),
    ReadDir(PathBuf),
    DrumMachineRequest(drum_machine::RequestKind),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientMessage {
    id: usize,
    request: bool,
    payload: ClientMessageKind,
}

pub struct Cache {
    cache: serde_json::Value,
}

impl Cache {
    pub fn new(drum_machine_json: serde_json::Value) -> Self {
        Self {
            cache: json!({
                "nodes": [],
                "drum_machine": drum_machine_json,
            }),
        }
    }

    pub fn get(&self) -> &serde_json::Value {
        &self.cache
    }

    pub fn cache_renderer_response(&mut self, res: &command::ResponseKind) {
        match res {
            command::ResponseKind::InvalidNodeKind => todo!(),
            command::ResponseKind::InvalidId => {}
            command::ResponseKind::Denied => {}
            command::ResponseKind::Failed => {}
            command::ResponseKind::NodeResponse { id, kind } => self.node_update(*id, kind),
            command::ResponseKind::AddNode { kind, instance, .. } => self.add_node(kind, instance),
            command::ResponseKind::RemoveNode { id } => self.remove_node(*id),
            command::ResponseKind::CloneNode { id } => self.clone_node(*id),
            command::ResponseKind::MoveNode { id, new_id } => todo!(),
        }
    }

    pub fn chache_drum_machine_update(&mut self, kind: &JsonUpdateKind) {
        match kind {
            JsonUpdateKind::InvalidId => {}
            JsonUpdateKind::Denied => {}
            JsonUpdateKind::Failed => {}
            JsonUpdateKind::Ok => {}
            JsonUpdateKind::UpdateFields(updates) => {
                for update in updates {
                    self.cache["drum_machine"][&update.0] = update.1.clone();
                }
            }
        }
    }

    fn add_node(&mut self, kind: &str, value: &serde_json::Value) {
        if let Some(nodes) = self.cache["nodes"].as_array_mut() {
            nodes.push(json!({
                "kind": kind,
                "instance": value,
            }));
        }
    }

    fn remove_node(&mut self, id: usize) {
        if let Some(nodes) = self.cache["nodes"].as_array_mut() {
            nodes.remove(id);
        }
    }

    fn clone_node(&mut self, id: usize) {
        if let Some(nodes) = self.cache["nodes"].as_array_mut() {
            if id <= nodes.len() {
                nodes.push(nodes[id].clone());
            }
        }
    }

    fn node_update(&mut self, node_id: usize, kind: &JsonUpdateKind) {
        match kind {
            JsonUpdateKind::InvalidId => {}
            JsonUpdateKind::Denied => {}
            JsonUpdateKind::Failed => {}
            JsonUpdateKind::Ok => {}
            JsonUpdateKind::UpdateFields(updates) => {
                for update in updates {
                    self.cache["nodes"][node_id]["instance"][&update.0] = update.1.clone();
                }
            }
        }
    }
}
