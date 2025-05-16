use axum::{Json, extract::State, http::StatusCode};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use ere_sp1::EreSP1;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::instrument;
use zkvm_interface::{Input, zkVM};

use crate::common::{AppState, ProgramID};
use crate::program_input::ProgramInput;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteRequest {
    pub program_id: ProgramID,
    pub input: ProgramInput,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteResponse {
    pub program_id: ProgramID,
    pub total_num_cycles: u64,
    pub region_cycles: IndexMap<String, u64>,
    pub execution_time: f64,
}

#[axum::debug_handler]
#[instrument(skip_all)]
pub async fn execute_program(
    State(state): State<AppState>,
    Json(req): Json<ExecuteRequest>,
) -> Result<Json<ExecuteResponse>, (StatusCode, String)> {
    let program_id = req.program_id.clone();
    if let Some(program) = state.programs.read().await.get(&program_id) {
        // Check if it's SP1 and use ere-sp1
        match program {
            crate::common::Program::SP1(elf_bytes) => {
                let start = Instant::now();

                // Create input and execute using EreSP1
                let mut input = Input::new();
                input.write(&req.input.value1).map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to write value1: {}", e),
                    )
                })?;
                input.write(&req.input.value2).map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to write value2: {}", e),
                    )
                })?;

                let zkvm = EreSP1::new(elf_bytes.clone());
                let report = zkvm.execute(&input).map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to execute program: {}", e),
                    )
                })?;
                let execution_time = start.elapsed().as_secs_f64();

                Ok(Json(ExecuteResponse {
                    program_id,
                    total_num_cycles: report.total_num_cycles,
                    region_cycles: report.region_cycles,
                    execution_time,
                }))
            }
            _ => Err((
                StatusCode::NOT_IMPLEMENTED,
                "Only SP1 execution is supported".to_string(),
            )),
        }
    } else {
        Err((StatusCode::NOT_FOUND, "Program not found".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{Program, ProgramID};
    use crate::test_utils::get_sp1_compiled_program;

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
    async fn test_execute_program_success() {
        let (state, _temp_dir) = create_test_state();
        let program_id = ProgramID("sp1".to_string());

        let program = get_sp1_compiled_program();

        {
            let mut programs = state.programs.write().await;
            programs.insert(program_id.clone(), program);
        }

        let request = ExecuteRequest {
            program_id: program_id.clone(),
            input: ProgramInput {
                value1: 42,
                value2: 10,
            },
        };

        let result = execute_program(State(state), Json(request)).await;

        assert!(result.is_ok());
        let response = result.unwrap().0;
        assert_eq!(response.program_id, program_id);
        assert!(response.total_num_cycles > 0);
        assert!(response.execution_time > 0.0);
    }

    #[tokio::test]
    async fn test_execute_program_not_found() {
        let (state, _temp_dir) = create_test_state();

        let request = ExecuteRequest {
            program_id: ProgramID("non_existent".to_string()),
            input: ProgramInput {
                value1: 42,
                value2: 10,
            },
        };

        let result = execute_program(State(state), Json(request)).await;

        assert!(result.is_err());
        let (status, message) = result.unwrap_err();
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(message, "Program not found");
    }

    #[tokio::test]
    async fn test_execute_program_wrong_type() {
        let (state, _temp_dir) = create_test_state();
        let program_id = ProgramID("test_program".to_string());
        {
            let mut programs = state.programs.write().await;
            programs.insert(program_id.clone(), Program::Risc0("test".to_string()));
        }

        let request = ExecuteRequest {
            program_id: program_id.clone(),
            input: ProgramInput {
                value1: 42,
                value2: 10,
            },
        };

        let result = execute_program(State(state), Json(request)).await;

        assert!(result.is_err());
        let (status, _message) = result.unwrap_err();
        assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
    }
}
