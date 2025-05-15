mod common;
mod endpoints;

use axum::{
    Router,
    routing::{get, post},
};
use common::AppState;
use endpoints::{execute_program, get_server_info, prove_program, register_program, verify_proof};
use std::{
    collections::HashMap,
    fs,
    net::SocketAddr,
    path::PathBuf,
    sync::Arc,
};
use tokio::sync::RwLock;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::trace::TraceLayer;

fn app(state: AppState) -> Router {
    Router::new()
        .route("/register_program", post(register_program))
        .route("/execute/:program_id", post(execute_program))
        .route("/prove/:program_id", post(prove_program))
        .route("/verify/:program_id", post(verify_proof))
        .route("/info", get(get_server_info))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .with_thread_names(true)
        .with_ansi(true)
        .init();

    // Create programs directory
    let programs_dir = PathBuf::from("programs");
    fs::create_dir_all(&programs_dir)?;

    let state = AppState {
        programs_dir,
        programs: Arc::new(RwLock::new(HashMap::new())),
    };

    let app = app(state);

    let addr: SocketAddr = "0.0.0.0:3000".parse()?;
    println!("ZKVM Program Service listening on {addr}");

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C handler");
    println!("graceful shutdown");
}
