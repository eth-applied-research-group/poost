mod common;
mod endpoints;
mod program;

use axum::{
    Router,
    routing::{get, post},
};
use common::{AppState, ProgramID, zkVMInstance, zkVMVendor};
use endpoints::{execute_program, get_server_info, prove_program, verify_proof};
use program::get_sp1_compiled_program;
use std::{fs, net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::trace::TraceLayer;

fn app(state: AppState) -> Router {
    Router::new()
        .route("/execute", post(execute_program))
        .route("/prove", post(prove_program))
        .route("/verify", post(verify_proof))
        .route("/info", get(get_server_info))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        // 10MB limit to account for the proof size
        // and the possibly large input size
        .layer(axum::extract::DefaultBodyLimit::max(10 * 1024 * 1024))
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

    let app = init_state().await;

    let addr: SocketAddr = "0.0.0.0:3000".parse()?;
    println!("Poost listening on {addr}");

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn init_state() -> Router {
    // Create programs directory if it doesn't exist
    let programs_dir = PathBuf::from("programs");
    fs::create_dir_all(&programs_dir).expect("Failed to create programs directory");

    let state = AppState {
        programs: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
    };

    // Compile the SP1 program at startup
    println!("Compiling SP1 program...");
    let sp1_zkvm = get_sp1_compiled_program();
    println!("SP1 program compiled successfully");

    // Save the compiled zkvm program instance in the app state with a fixed program ID
    let program_id = ProgramID::from(zkVMVendor::SP1);
    {
        let mut programs = state.programs.write().await;
        programs.insert(
            program_id.clone(),
            zkVMInstance::new(zkVMVendor::SP1, Arc::new(sp1_zkvm)),
        );
    }

    println!("SP1 program saved with ID: {:?}", program_id);

    // Build our application with a route
    let app = app(state);

    app
}

async fn shutdown_signal() {
    signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C handler");
    println!("graceful shutdown");
}
