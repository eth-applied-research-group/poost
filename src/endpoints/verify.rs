use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::common::{AppState, ZkVMType};

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyRequest {
    pub proof: String,
    #[serde(deserialize_with = "crate::endpoints::register::deserialize_zkvm_type")]
    pub zkvm: ZkVMType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyResponse {
    pub program_id: String,
    pub verified: bool,
}

#[axum::debug_handler]
#[instrument(skip_all)]
pub async fn verify_proof(
    Path(program_id): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, (StatusCode, String)> {
    if let Some(program) = state.programs.read().await.get(&program_id) {
        // TODO: In a real implementation, verify the proof using the appropriate ZKVM verifier
        // based on req.zkvm
        Ok(Json(VerifyResponse {
            program_id,
            verified: true,
        }))
    } else {
        Err((StatusCode::NOT_FOUND, "Program not found".to_string()))
    }
}
