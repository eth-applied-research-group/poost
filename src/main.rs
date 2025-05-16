mod common;
mod endpoints;
mod program_input;

use axum::{
    Router,
    routing::{get, post},
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use common::{AppState, Program};
use endpoints::{execute_program, get_server_info, prove_program, verify_proof};
use ere_sp1::RV32_IM_SUCCINCT_ZKVM_ELF;
use std::{fs, net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::trace::TraceLayer;
use zkvm_interface::Compiler;

fn app(state: AppState) -> Router {
    Router::new()
        .route("/execute", post(execute_program))
        .route("/prove", post(prove_program))
        .route("/verify", post(verify_proof))
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

    // Create programs directory if it doesn't exist
    let programs_dir = PathBuf::from("programs");
    fs::create_dir_all(&programs_dir).expect("Failed to create programs directory");

    let state = AppState {
        programs: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        programs_dir: programs_dir.clone(),
    };

    // Compile the SP1 program at startup
    let sp1_program_dir = PathBuf::from("programs/sp1");
    println!("Compiling SP1 program...");
    let program = RV32_IM_SUCCINCT_ZKVM_ELF::compile(&sp1_program_dir)
        .expect("Failed to compile SP1 program");
    println!("SP1 program compiled successfully");

    // Save the compiled program in the app state with a fixed program ID
    let program_id = "sp1".to_string();
    {
        let mut programs = state.programs.write().await;
        programs.insert(program_id.clone(), Program::SP1(BASE64.encode(&program)));
    }
    println!("SP1 program saved with ID: {}", program_id);

    // Build our application with a route
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
