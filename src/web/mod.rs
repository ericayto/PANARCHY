mod assets;

use std::{
    convert::Infallible,
    net::SocketAddr,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use anyhow::Result;
use axum::body::Body;
use axum::http::StatusCode;
use axum::{
    extract::{Path, State},
    http::header,
    response::{
        sse::{Event, KeepAlive, Sse},
        Html, IntoResponse, Response,
    },
    routing::get,
    Json, Router,
};
use bytes::Bytes;
use serde::Serialize;
use tokio::{net::TcpListener, sync::broadcast};
use tokio_stream::{wrappers::BroadcastStream, Stream, StreamExt};

use crate::{
    engine::{EngineBuilder, EngineSettings},
    scenario::Scenario,
    systems::{
        BookkeepingSystem, EconomySystem, EnvironmentSystem, FinanceSystem, InfrastructureSystem,
        PolicySystem, PopulationSystem, TechnologySystem,
    },
    world::WorldSnapshot,
};

#[derive(Clone, Serialize)]
pub struct UiFrame {
    pub snapshot: WorldSnapshot,
    pub completed: bool,
}

#[derive(Clone, Serialize)]
pub struct StateEnvelope {
    pub scenario: String,
    pub total_ticks: u64,
    pub frame: Option<UiFrame>,
    pub completed: bool,
}

#[derive(Clone)]
struct AppState {
    broadcaster: broadcast::Sender<String>,
    latest_frame: Arc<Mutex<Option<UiFrame>>>,
    frames: Arc<Mutex<Vec<UiFrame>>>,
    total_ticks: u64,
    scenario_name: String,
    simulation_done: Arc<AtomicBool>,
}

pub struct WebServerConfig {
    pub scenario: Scenario,
    pub ticks: u64,
    pub snapshot_interval: u64,
    pub snapshot_dir: PathBuf,
    pub host: String,
    pub port: u16,
}

pub async fn run(config: WebServerConfig) -> Result<()> {
    let WebServerConfig {
        scenario,
        ticks,
        snapshot_interval,
        snapshot_dir,
        host,
        port,
    } = config;

    let scenario_name = scenario.name.clone();
    let mut world = scenario.build_world();
    let settings = EngineSettings {
        scenario_name: scenario_name.clone(),
        seed: scenario.seed,
        snapshot_interval_ticks: snapshot_interval,
        snapshot_dir: snapshot_dir.clone(),
    };

    let mut engine = EngineBuilder::new(settings)
        .with_system(EnvironmentSystem::new())
        .with_system(InfrastructureSystem::new())
        .with_system(PopulationSystem::new())
        .with_system(EconomySystem::new())
        .with_system(FinanceSystem::new())
        .with_system(PolicySystem::new())
        .with_system(TechnologySystem::new())
        .with_system(BookkeepingSystem::new())
        .build();

    let (tx, _) = broadcast::channel::<String>(512);
    let latest_frame: Arc<Mutex<Option<UiFrame>>> = Arc::new(Mutex::new(None));
    let frames: Arc<Mutex<Vec<UiFrame>>> = Arc::new(Mutex::new(Vec::new()));
    let simulation_done = Arc::new(AtomicBool::new(false));

    let latest_for_sim = latest_frame.clone();
    let frames_for_sim = frames.clone();
    let done_for_sim = simulation_done.clone();
    let tx_for_sim = tx.clone();
    let ticks = ticks;
    let scenario_label = scenario_name.clone();

    let sim_handle = tokio::task::spawn_blocking(move || -> Result<()> {
        engine.run_with_hook(&mut world, ticks, |snapshot| {
            let frame = UiFrame {
                snapshot,
                completed: false,
            };
            {
                let mut guard = latest_for_sim.lock().expect("latest frame lock poisoned");
                *guard = Some(frame.clone());
            }
            {
                let mut guard = frames_for_sim.lock().expect("frames lock poisoned");
                guard.push(frame.clone());
            }
            if let Ok(payload) = serde_json::to_string(&frame) {
                let _ = tx_for_sim.send(payload);
            }
        })?;

        done_for_sim.store(true, Ordering::SeqCst);

        let final_frame = {
            let guard = latest_for_sim.lock().expect("latest frame lock poisoned");
            guard.clone()
        };

        if let Some(mut frame) = final_frame {
            frame.completed = true;
            {
                let mut guard = latest_for_sim.lock().expect("latest frame lock poisoned");
                *guard = Some(frame.clone());
            }
            {
                let mut guard = frames_for_sim.lock().expect("frames lock poisoned");
                if let Some(last) = guard.last_mut() {
                    *last = frame.clone();
                } else {
                    guard.push(frame.clone());
                }
            }
            if let Ok(payload) = serde_json::to_string(&frame) {
                let _ = tx_for_sim.send(payload);
            }
        }

        Ok(())
    });

    let state = Arc::new(AppState {
        broadcaster: tx.clone(),
        latest_frame: latest_frame.clone(),
        frames: frames.clone(),
        total_ticks: ticks,
        scenario_name: scenario_label.clone(),
        simulation_done: simulation_done.clone(),
    });

    tokio::spawn(async move {
        match sim_handle.await {
            Ok(Ok(())) => {
                println!("[web] Simulation completed for '{}'.", scenario_label);
            }
            Ok(Err(err)) => {
                eprintln!("[web] Simulation error: {err:?}");
            }
            Err(err) => {
                eprintln!("[web] Simulation task failed: {err:?}");
            }
        }
    });

    let router = Router::new()
        .route("/", get(index))
        .route("/styles.css", get(styles))
        .route("/app.js", get(script))
        .route("/api/state", get(latest_state))
        .route("/sprites/:name", get(sprite))
        .route("/api/frames", get(all_frames))
        .route("/api/events", get(stream_events))
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .expect("invalid address");

    println!(
        "🚀 PANARCHY UI live at http://{}:{} (Ctrl+C to stop)",
        host, port
    );

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    println!("Shutting down web UI...");
}

async fn index() -> Html<&'static str> {
    Html(assets::INDEX_HTML)
}

async fn styles() -> impl IntoResponse {
    Response::builder()
        .header(header::CONTENT_TYPE, "text/css; charset=utf-8")
        .body(assets::STYLES_CSS.to_string())
        .unwrap()
}

async fn script() -> impl IntoResponse {
    Response::builder()
        .header(
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )
        .body(assets::APP_JS.to_string())
        .unwrap()
}

async fn sprite(Path(name): Path<String>) -> Response {
    match assets::sprite(&name) {
        Some(bytes) => Response::builder()
            .header(header::CONTENT_TYPE, "image/png")
            .body(Body::from(Bytes::from_static(bytes)))
            .unwrap(),
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from(Bytes::from_static(b"")))
            .unwrap(),
    }
}

async fn latest_state(State(state): State<Arc<AppState>>) -> Json<StateEnvelope> {
    let frame = state
        .latest_frame
        .lock()
        .expect("latest frame lock poisoned")
        .clone();
    Json(StateEnvelope {
        scenario: state.scenario_name.clone(),
        total_ticks: state.total_ticks,
        frame,
        completed: state.simulation_done.load(Ordering::SeqCst),
    })
}

#[derive(Serialize)]
struct FramesResponse {
    scenario: String,
    total_ticks: u64,
    completed: bool,
    frames: Vec<UiFrame>,
}

async fn all_frames(State(state): State<Arc<AppState>>) -> Json<FramesResponse> {
    let frames = state.frames.lock().expect("frames lock poisoned").clone();
    Json(FramesResponse {
        scenario: state.scenario_name.clone(),
        total_ticks: state.total_ticks,
        completed: state.simulation_done.load(Ordering::SeqCst),
        frames,
    })
}

async fn stream_events(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.broadcaster.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|msg| match msg {
        Ok(payload) => Some(Ok(Event::default().data(payload))),
        Err(_) => None,
    });
    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(2))
            .text("keep-alive"),
    )
}
