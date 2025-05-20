use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use zkvm_interface::{Input, zkVM};

use crate::common::{AppState, ProgramID};
use crate::program::ProgramInput;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProveRequest {
    pub program_id: ProgramID,
    pub input: ProgramInput,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProveResponse {
    pub program_id: ProgramID,
    pub proof: Vec<u8>,
    pub proving_time_milliseconds: u128,
}

#[axum::debug_handler]
#[instrument(skip_all)]
pub async fn prove_program(
    State(state): State<AppState>,
    Json(req): Json<ProveRequest>,
) -> Result<Json<ProveResponse>, (StatusCode, String)> {
    let program_id = req.program_id.clone();
    let programs = state.programs.read().await;

    let program = programs
        .get(&program_id)
        .ok_or((StatusCode::NOT_FOUND, "Program not found".to_string()))?;

    let input: Input = req.input.into();

    let (proof, report) = program.vm.prove(&input).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to generate proof: {}", e),
        )
    })?;

    Ok(Json(ProveResponse {
        program_id,
        proof,
        proving_time_milliseconds: report.proving_time.as_millis(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{zkVMInstance, zkVMVendor};
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
    async fn test_prove_program_success() {
        let mock_zkvm = MockZkVM::default();

        let (state, _temp_dir) = create_test_state();
        let program_id = ProgramID::from(zkVMVendor::SP1);
        {
            let mut programs = state.programs.write().await;
            programs.insert(
                program_id.clone(),
                zkVMInstance::new(zkVMVendor::SP1, Arc::new(mock_zkvm)),
            );
        }

        let request = ProveRequest {
            program_id: program_id.clone(),
            input: ProgramInput::test_input(),
        };

        let result = prove_program(State(state), Json(request)).await;

        assert!(result.is_ok());
        let response = result.unwrap().0;
        assert_eq!(response.program_id, program_id);
        assert!(!response.proof.is_empty()); // Check that the proof is not empty
    }

    #[tokio::test]
    async fn test_prove_program_not_found() {
        let (state, _temp_dir) = create_test_state();

        let request = ProveRequest {
            program_id: ProgramID("non_existent".to_string()),
            input: ProgramInput::test_input(),
        };

        let result = prove_program(State(state), Json(request)).await;

        assert!(result.is_err());
        let (status, message) = result.unwrap_err();
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(message, "Program not found");
    }
}
