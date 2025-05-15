use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::common::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProveRequest {
    pub program_id: String,
    pub input: Vec<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProveResponse {
    pub program_id: String,
    pub proof: Vec<u8>,
}

#[axum::debug_handler]
#[instrument(skip_all)]
pub async fn prove_program(
    State(state): State<AppState>,
    Json(req): Json<ProveRequest>,
) -> Result<Json<ProveResponse>, (StatusCode, String)> {
    let program_id = req.program_id.clone();
    if let Some(program) = state.programs.read().await.get(&program_id) {
        match program {
            crate::common::Program::SP1(_) => {
                // TODO: Use ere-sp1 to generate proof
                Ok(Json(ProveResponse {
                    program_id,
                    proof: vec![0, 1, 2], // Example proof bytes
                }))
            }
            _ => Err((
                StatusCode::NOT_IMPLEMENTED,
                format!("unsupported zkvm {:?}", program),
            )),
        }
    } else {
        Err((StatusCode::NOT_FOUND, "Program not found".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::Program;
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
            programs_dir,
        };

        (state, temp_dir)
    }

    #[tokio::test]
    async fn test_prove_program_success() {
        let (state, _temp_dir) = create_test_state();
        let program_id = "test_program".to_string();
        {
            let mut programs = state.programs.write().await;
            programs.insert(program_id.clone(), Program::SP1("test".to_string()));
        }

        let request = ProveRequest {
            program_id: program_id.clone(),
            input: vec![vec![1, 2, 3], vec![4, 5, 6]],
        };

        let result = prove_program(State(state), Json(request)).await;

        assert!(result.is_ok());
        let response = result.unwrap().0;
        assert_eq!(response.program_id, program_id);
        assert_eq!(response.proof, vec![0, 1, 2]); // Check against our stub proof
    }

    #[tokio::test]
    async fn test_prove_program_not_found() {
        let (state, _temp_dir) = create_test_state();

        let request = ProveRequest {
            program_id: "non_existent".to_string(),
            input: vec![vec![1, 2, 3]],
        };

        let result = prove_program(State(state), Json(request)).await;

        assert!(result.is_err());
        let (status, message) = result.unwrap_err();
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(message, "Program not found");
    }

    #[tokio::test]
    async fn test_prove_program_wrong_type() {
        let (state, _temp_dir) = create_test_state();
        let program_id = "test_program".to_string();
        {
            let mut programs = state.programs.write().await;
            programs.insert(program_id.clone(), Program::Risc0("test".to_string()));
        }

        let request = ProveRequest {
            program_id: program_id.clone(),
            input: vec![vec![1, 2, 3]],
        };

        let result = prove_program(State(state), Json(request)).await;

        assert!(result.is_err());
        let (status, _message) = result.unwrap_err();
        assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
    }
}
