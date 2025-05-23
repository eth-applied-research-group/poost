use axum::{Json, extract::State, http::StatusCode};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tracing::instrument;
use zkvm_interface::{Input, zkVM};

use crate::common::{AppState, ProgramID};
use crate::program::ProgramInput;

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
    pub execution_time_duration: Duration,
}

#[axum::debug_handler]
#[instrument(skip_all)]
pub async fn execute_program(
    State(state): State<AppState>,
    Json(req): Json<ExecuteRequest>,
) -> Result<Json<ExecuteResponse>, (StatusCode, String)> {
    let program_id = req.program_id.clone();
    let programs = state.programs.read().await;

    let program = programs
        .get(&program_id)
        .ok_or((StatusCode::NOT_FOUND, "Program not found".to_string()))?;

    let input: Input = req.input.into();

    let start = Instant::now();
    let report = program.vm.execute(&input).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to execute program: {}", e),
        )
    })?;
    let execution_time_duration = start.elapsed();

    Ok(Json(ExecuteResponse {
        program_id,
        total_num_cycles: report.total_num_cycles,
        region_cycles: report.region_cycles,
        execution_time_duration,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{ProgramID, zkVMInstance};
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
    async fn test_execute_program_success() {
        let (state, _temp_dir) = create_test_state();
        let program_id = ProgramID("sp1".to_string());

        let mock_zkvm = MockZkVM::default();

        {
            let mut programs = state.programs.write().await;
            programs.insert(
                program_id.clone(),
                zkVMInstance::new(crate::common::zkVMVendor::SP1, Arc::new(mock_zkvm)),
            );
        }

        let request = ExecuteRequest {
            program_id: program_id.clone(),
            input: ProgramInput::test_input(),
        };

        let result = execute_program(State(state), Json(request)).await;

        assert!(result.is_ok());
        let response = result.unwrap().0;
        assert_eq!(response.program_id, program_id);
        assert!(response.total_num_cycles > 0);
        assert!(response.execution_time_duration.as_millis() > 0);
    }

    #[tokio::test]
    async fn test_execute_program_not_found() {
        let (state, _temp_dir) = create_test_state();

        let request = ExecuteRequest {
            program_id: ProgramID("non_existent".to_string()),
            input: ProgramInput::test_input(),
        };

        let result = execute_program(State(state), Json(request)).await;

        assert!(result.is_err());
        let (status, message) = result.unwrap_err();
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(message, "Program not found");
    }
}
