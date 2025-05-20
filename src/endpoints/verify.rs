use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use zkvm_interface::zkVM;

use crate::common::{AppState, ProgramID};
use crate::{
    common::{zkVMInstance, zkVMVendor},
    endpoints::{prove::ProveRequest, prove_program},
    program::ProgramInput,
};

use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyRequest {
    pub program_id: ProgramID,
    pub proof: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyResponse {
    pub program_id: ProgramID,
    pub verified: bool,
    // Empty if verification returned true
    pub failure_reason: String,
}

#[axum::debug_handler]
#[instrument(skip_all)]
pub async fn verify_proof(
    State(state): State<AppState>,
    Json(req): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, (StatusCode, String)> {
    // Check if the program_id is correct
    let programs = state.programs.read().await;

    let program = programs
        .get(&req.program_id)
        .ok_or((StatusCode::NOT_FOUND, "Program not found".to_string()))?;

    // Verify the proof
    let (verified, failure_reason) = match program.vm.verify(&req.proof) {
        Ok(_) => (true, String::default()),
        Err(err) => (false, format!("{}", err)),
    };

    Ok(Json(VerifyResponse {
        program_id: req.program_id,
        verified,
        failure_reason,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_zkvm::MockZkVM;
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
        let program_id = ProgramID::from(zkVMVendor::SP1);
        let mock_zkvm = MockZkVM::default();

        let state = AppState {
            programs: Arc::new(RwLock::new(HashMap::new())),
        };
        {
            let mut programs = state.programs.write().await;
            programs.insert(
                program_id.clone(),
                zkVMInstance::new(zkVMVendor::SP1, Arc::new(mock_zkvm)),
            );
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
            program_id: result.program_id.clone(),
            proof: result.proof.clone(),
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
        let program_id = ProgramID::from(zkVMVendor::SP1);

        let mock_zkvm = MockZkVM::default();
        {
            let mut programs = state.programs.write().await;
            programs.insert(
                program_id.clone(),
                zkVMInstance::new(zkVMVendor::SP1, Arc::new(mock_zkvm)),
            );
        }

        let request = VerifyRequest {
            program_id: program_id.clone(),
            proof: b"invalid_proof".to_vec(),
        };

        let result = verify_proof(State(state), Json(request)).await;
        // The endpoint returns a result if the verification fails.
        // We need to check the proof response to know whether it failed
        // verification and for what reason.
        assert!(result.is_ok());
        assert!(!result.unwrap().verified);
    }

    #[tokio::test]
    async fn test_verify_proof_not_found() {
        let (state, _temp_dir) = create_test_state();

        let request = VerifyRequest {
            program_id: ProgramID("non_existent".to_string()),
            proof: b"example_proof".to_vec(),
        };

        let result = verify_proof(State(state), Json(request)).await;

        assert!(result.is_err());
        let (status, message) = result.unwrap_err();
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(message, "Program not found");
    }
}
