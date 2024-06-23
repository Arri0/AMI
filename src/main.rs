use clap::Parser;
use midi::MidiReader;
use render::{
    command,
    node::{self, fluidlite_synth, oxi_synth, rusty_synth},
    Renderer,
};
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use webserver::{Clients, ServerMessageKind};

pub mod audio;
pub mod deser;
pub mod midi;
pub mod path;
pub mod render;
mod webserver;

const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(about = "Simple software for adding two integers numbers.")]
pub struct Args {
    // #[clap(index=1)]
    // a: i32,

    // #[clap(index=2)]
    // b: Option<i32>,
    #[arg(short, long, help = "Path to samples directory")]
    samples: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!(
        "Starting AMI (Arri's Midi Instrument) > version: {}",
        VERSION.unwrap_or("?")
    );

    info!("| Samples directory: {:?}", args.samples);

    let (midi_tx, midi_rx) = midi::create_channel(32);
    let (req_tx, req_rx) = command::create_request_channel(32);

    let mut virtual_paths = crate::path::VirtualPaths::default();
    virtual_paths.insert("samples:".into(), args.samples);

    info!("| Available MIDI ports:");
    for port in midi::MidiReader::get_available_ports() {
        info!("| - {port}");
    }

    let clients = Clients::new(256);
    let mut midi_reader = midi::MidiReader::with_slots(
        midi_tx.clone(),
        16,
    );

    if midi_reader.connect_input(0, "VMPK Output:out 130:0").is_ok() {
        tracing::debug!("MIDI port connected: VMPK Output:out 130:0");
    }
    if midi_reader.connect_input(0, "Hammer 88 Pro:Hammer 88 Pro USB MIDI 20:0").is_ok() {
        tracing::debug!("MIDI port connected: Hammer 88 Pro:Hammer 88 Pro USB MIDI 20:0");
    }

    let midi_reader = Arc::new(Mutex::new(midi_reader));

    tokio::spawn(run_midi_logger(midi_rx, clients.clone()));
    tokio::spawn(run_midi_port_watchdog(clients.clone()));

    let mut renderer = Renderer::new(midi_tx.subscribe(), req_rx, virtual_paths.clone());
    renderer.register_node_kind("RustySynth", || Box::<rusty_synth::Node>::default());
    renderer.register_node_kind("OxiSynth", || Box::<oxi_synth::Node>::default());
    renderer.register_node_kind("FluidliteSynth", || Box::<fluidlite_synth::Node>::default());

    let renderer = Arc::new(Mutex::new(renderer));
    let mut audio_ctr = audio::output::Controller::new(renderer);
    audio_ctr.sample_rate = 44100;
    audio_ctr.buffer_size = 2048;
    audio_ctr
        .connect_to_default_output_device()
        .expect("Failed to connect to output device");

    let cache = Arc::new(Mutex::new(webserver::Cache::new()));

    let req_tx2 = req_tx.clone();
    let cache2 = Arc::clone(&cache);
    tokio::spawn(async move {
        let req = command::RequestKind::AddNode {
            kind: "OxiSynth".into(),
        };
        if let Some(res) = send_renderer_request(&req_tx2, req).await {
            cache2.lock().await.cache_renderer_response(&res);
        }

        let file_path = PathBuf::from("samples:/MS_Basic.sf2");
        let req = command::RequestKind::NodeRequest {
            id: 0,
            kind: node::RequestKind::LoadFile(file_path),
        };

        if let Some(res) = send_renderer_request(&req_tx2, req).await {
            cache2.lock().await.cache_renderer_response(&res);
        }
    });

    let shared_state = webserver::SharedState {
        clients: Clients::clone(&clients),
        midi_reader: Arc::clone(&midi_reader),
        cache: Arc::clone(&cache),
    };

    webserver::run(3000, shared_state, move |addr, req| {
        let midi_reader = Arc::clone(&midi_reader);
        let mut clients = Clients::clone(&clients);
        let cache = Arc::clone(&cache);
        let req_tx = req_tx.clone();
        let vp = virtual_paths.clone();
        async move {
            use webserver::ClientMessageKind;
            use webserver::ServerMessageKind;
            match req {
                ClientMessageKind::Ping => ServerMessageKind::Pong,
                ClientMessageKind::Report(report) => {
                    info!("Report from [{addr}]: {report}");
                    ServerMessageKind::Ack
                }
                ClientMessageKind::ConnectMidiInput(slot, name) => {
                    let mut midi_reader = midi_reader.lock().await;
                    if let Ok(()) = midi_reader.connect_input(slot, &name) {
                        clients.broadcast(ServerMessageKind::ConnectedMidiInputs(
                            midi_reader.connected_input_names(),
                        ));
                        ServerMessageKind::Ack
                    } else {
                        ServerMessageKind::Nak
                    }
                }
                ClientMessageKind::DisconnectMidiInput(slot) => {
                    let mut midi_reader = midi_reader.lock().await;
                    if let Ok(()) = midi_reader.disconnect_input(slot) {
                        clients.broadcast(ServerMessageKind::ConnectedMidiInputs(
                            midi_reader.connected_input_names(),
                        ));
                        ServerMessageKind::Ack
                    } else {
                        ServerMessageKind::Nak
                    }
                }
                ClientMessageKind::RendererRequest(req) => {
                    let mut cache = cache.lock().await;
                    let res = send_renderer_request(&req_tx, req).await;
                    if let Some(res) = res {
                        cache.cache_renderer_response(&res);
                        clients.broadcast(ServerMessageKind::RendererResponse(res));
                        ServerMessageKind::Ack
                    } else {
                        ServerMessageKind::Nak
                    }
                }
                ClientMessageKind::ReadDir(path) => {
                    if let Some(path) = vp.translate(&path) {
                        if let Ok(dir) = std::fs::read_dir(&path) {
                            let entries = dir
                                .into_iter()
                                .flatten()
                                .map(|x| {
                                    (
                                        x.path().is_dir(),
                                        crate::path::remove_prefix(x.path().as_path(), &path),
                                    )
                                })
                                .collect();
                            return ServerMessageKind::DirInfo(Some(entries));
                        }
                    }
                    ServerMessageKind::DirInfo(None)
                }
            }
        }
    })
    .await;

    Ok(())
}

async fn run_midi_logger(mut midi_rx: midi::Receiver, mut clients: Clients) {
    while let Ok(message) = midi_rx.recv().await {
        clients.broadcast(ServerMessageKind::MidiEvent(message));
    }
}

async fn run_midi_port_watchdog(mut clients: Clients) {
    loop {
        clients.broadcast(ServerMessageKind::AvailableMidiInputs(
            MidiReader::get_available_ports(),
        ));
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}

async fn send_renderer_request(
    req_tx: &command::Requester,
    req: command::RequestKind,
) -> Option<command::ResponseKind> {
    let (res_tx, res_rx) = command::create_response_channel();

    if let Ok(()) = req_tx.send((req, res_tx)).await {
        if let Ok(response_kind) = res_rx.await {
            Some(response_kind)
        } else {
            None
        }
    } else {
        None
    }
}
