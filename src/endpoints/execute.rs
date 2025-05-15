use axum::{Json, extract::State, http::StatusCode};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use ere_sp1::EreSP1;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::instrument;
use zkvm_interface::{Input, zkVM};

use crate::common::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteRequest {
    pub program_id: String,
    pub input: Vec<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteResponse {
    pub program_id: String,
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
            crate::common::Program::SP1(elf_base64) => {
                let start = Instant::now();

                // Decode the ELF file
                let elf_bytes = BASE64.decode(elf_base64).map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to decode ELF: {}", e),
                    )
                })?;

                // Create input and execute using EreSP1
                let mut input = Input::new();
                for slice in &req.input {
                    input.write(slice).map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Failed to write input: {}", e),
                        )
                    })?;
                }

                let zkvm = EreSP1::new(elf_bytes);
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
    async fn test_execute_program_success() {
        let (state, _temp_dir) = create_test_state();
        let program_id = "test_program".to_string();

        // Read and encode the example program's ELF
        let elf_path = "tests/sp1/execute/basic/target/elf-compilation/riscv32im-succinct-zkvm-elf/release/ere-test-sp1-guest";
        let elf_bytes = fs::read(elf_path).expect("Failed to read ELF file");
        let elf_base64 = BASE64.encode(elf_bytes);

        {
            let mut programs = state.programs.write().await;
            programs.insert(program_id.clone(), Program::SP1(elf_base64));
        }

        let mut input = Input::new();
        input.write(&(10 as u32)).unwrap();
        input.write(&(20 as u16)).unwrap();
        let chunked_inputs: Vec<_> = input.chunked_iter().map(|chunk| chunk.to_vec()).collect();

        let request = ExecuteRequest {
            program_id: program_id.clone(),
            input: chunked_inputs,
        };

        let result = execute_program(State(state), Json(request)).await;

        assert!(result.is_ok());
        let response = result.unwrap().0;
        assert_eq!(response.program_id, program_id);
        // The program adds the inputs and multiplies by 2: (1 + 2) * 2 = 6
        assert!(response.total_num_cycles > 0);
        assert!(response.execution_time > 0.0);
    }

    #[tokio::test]
    async fn test_execute_program_not_found() {
        let (state, _temp_dir) = create_test_state();

        let request = ExecuteRequest {
            program_id: "non_existent".to_string(),
            input: vec![vec![1, 2, 3]],
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
        let program_id = "test_program".to_string();
        {
            let mut programs = state.programs.write().await;
            programs.insert(program_id.clone(), Program::Risc0("test".to_string()));
        }

        let request = ExecuteRequest {
            program_id: program_id.clone(),
            input: vec![vec![1, 2, 3]],
        };

        let result = execute_program(State(state), Json(request)).await;

        assert!(result.is_err());
        let (status, message) = result.unwrap_err();
        assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
        assert_eq!(message, "Only SP1 execution is supported");
    }
}
