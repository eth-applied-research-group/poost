use axum::{Json, extract::State, http::StatusCode};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use std::{fs, str::FromStr};
use tracing::instrument;
use uuid::Uuid;

use crate::common::{AppState, Program, ZkVMType};

#[derive(Debug, Deserialize)]
pub struct RegisterProgramRequest {
    pub program_name: String,
    #[serde(deserialize_with = "deserialize_zkvm_type")]
    pub zkvm: ZkVMType,
    pub source_code: String, // Base64-encoded tar/zip of the program source
}

#[derive(Debug, Serialize)]
pub struct RegisterProgramResponse {
    pub program_id: String,
    pub status: String,
}

pub fn deserialize_zkvm_type<'de, D>(deserializer: D) -> Result<ZkVMType, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    ZkVMType::from_str(&s).map_err(|e| serde::de::Error::custom(e))
}

#[axum::debug_handler]
#[instrument(skip_all)]
pub async fn register_program(
    State(state): State<AppState>,
    Json(req): Json<RegisterProgramRequest>,
) -> Result<Json<RegisterProgramResponse>, (StatusCode, String)> {
    let program_id = Uuid::new_v4().to_string();
    let program_dir = state.programs_dir.join(&program_id);

    // Create directory for the program
    fs::create_dir_all(&program_dir).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create program directory: {e}"),
        )
    })?;

    // Decode and extract the source code
    let _source_bytes = BASE64.decode(&req.source_code).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Invalid base64 encoding: {e}"),
        )
    })?;

    // TODO: Extract the source code archive (tar/zip) into program_dir
    // This will depend on the format you choose (tar.gz, zip, etc.)

    let program = match req.zkvm {
        ZkVMType::Risc0 => Program::Risc0(req.program_name.clone()),
        ZkVMType::SP1 => Program::SP1(req.program_name.clone()),
    };
    {
        let mut programs = state.programs.write().unwrap();
        programs.insert(program_id.clone(), program);
    }

    Ok(Json(RegisterProgramResponse {
        program_id,
        status: format!("compiled with {}", req.zkvm),
    }))
}
