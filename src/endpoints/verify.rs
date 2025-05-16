use axum::{Json, extract::State, http::StatusCode};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use zkvm_interface::zkVM;

use crate::common::{AppState, Program, ProgramID};

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyRequest {
    pub program_id: ProgramID,
    pub proof: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyResponse {
    pub program_id: ProgramID,
    pub verified: bool,
}

#[axum::debug_handler]
#[instrument(skip_all)]
pub async fn verify_proof(
    State(state): State<AppState>,
    Json(req): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, (StatusCode, String)> {
    if let Some(program) = state.programs.read().await.get(&req.program_id) {
        match program {
            Program::SP1(zkvm) => {
                // Decode the proof
                let proof_bytes = BASE64.decode(&req.proof).map_err(|e| {
                    (
                        StatusCode::BAD_REQUEST,
                        format!("Failed to decode proof: {}", e),
                    )
                })?;

                zkvm.verify(&proof_bytes).map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Verification failed: {}", e),
                    )
                })?;

                Ok(Json(VerifyResponse {
                    program_id: req.program_id,
                    verified: true,
                }))
            }
            _ => Err((
                StatusCode::NOT_IMPLEMENTED,
                "Only SP1 verification is supported".to_string(),
            )),
        }
    } else {
        Err((StatusCode::NOT_FOUND, "Program not found".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        common::{Program, ZkVMType},
        endpoints::{prove::ProveRequest, prove_program},
        program::{ProgramInput, get_sp1_compiled_program},
    };
    use std::collections::HashMap;
    use std::fs;
    use std::sync::Arc;
    use tempfile::TempDir;
    use tokio::sync::RwLock;

    // Helper function to create a test AppState
    fn create_test_state() -> (AppState, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let programs_dir = temp_dir.path().join("programs");
        fs::create_dir_all(&programs_dir).unwrap();

        let state = AppState {
            programs: Arc::new(RwLock::new(HashMap::new())),
        };

        (state, temp_dir)
    }

    #[tokio::test]
    async fn test_verify_proof_success() {
        let program_id = ProgramID::from(ZkVMType::SP1);
        let sp1_zkvm = get_sp1_compiled_program();

        let state = AppState {
            programs: Arc::new(RwLock::new(HashMap::new())),
        };
        {
            let mut programs = state.programs.write().await;
            programs.insert(program_id.clone(), Program::SP1(sp1_zkvm));
        }

        let request = ProveRequest {
            program_id: program_id.clone(),
            input: ProgramInput {
                value1: 42,
                value2: 10,
            },
        };

        let result = prove_program(State(state.clone()), Json(request))
            .await
            .unwrap();

        // Create a request
        let request = VerifyRequest {
            program_id: program_id.clone(),
            proof: base64::engine::general_purpose::STANDARD.encode(&result.proof),
        };

        // Call the handler
        let response = verify_proof(State(state), Json(request)).await.unwrap();

        // Verify the response
        assert_eq!(response.program_id, program_id);
        assert!(response.verified);
    }

    #[tokio::test]
    async fn test_verify_proof_invalid_proof() {
        let (state, _temp_dir) = create_test_state();
        let program_id = ProgramID::from(ZkVMType::SP1);

        // Read and encode the fixed program's ELF
        let sp1_zkvm = get_sp1_compiled_program();
        {
            let mut programs = state.programs.write().await;
            programs.insert(program_id.clone(), Program::SP1(sp1_zkvm));
        }

        let request = VerifyRequest {
            program_id: program_id.clone(),
            proof: BASE64.encode("invalid_proof"),
        };

        let result = verify_proof(State(state), Json(request)).await;

        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_verify_proof_not_found() {
        let (state, _temp_dir) = create_test_state();

        let request = VerifyRequest {
            program_id: ProgramID("non_existent".to_string()),
            proof: BASE64.encode("example_proof"),
        };

        let result = verify_proof(State(state), Json(request)).await;

        assert!(result.is_err());
        let (status, message) = result.unwrap_err();
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(message, "Program not found");
    }

    #[tokio::test]
    async fn test_verify_proof_wrong_type() {
        let (state, _temp_dir) = create_test_state();
        let program_id = ProgramID("test_program".to_string());
        {
            let mut programs = state.programs.write().await;
            programs.insert(program_id.clone(), Program::Risc0("test".to_string()));
        }

        let request = VerifyRequest {
            program_id: program_id.clone(),
            proof: BASE64.encode("example_proof"),
        };

        let result = verify_proof(State(state), Json(request)).await;

        assert!(result.is_err());
        let (status, _message) = result.unwrap_err();
        assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
    }
}
