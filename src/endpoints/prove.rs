use axum::{Json, extract::State, http::StatusCode};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use ere_sp1::EreSP1;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use zkvm_interface::{Input, zkVM};

use crate::common::AppState;
use crate::program_input::ProgramInput;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProveRequest {
    pub program_id: String,
    pub input: ProgramInput,
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
        // Check if it's SP1 and use ere-sp1
        match program {
            crate::common::Program::SP1(elf_base64) => {
                // Decode the ELF file
                let elf_bytes = BASE64.decode(elf_base64).map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to decode ELF: {}", e),
                    )
                })?;

                // Create input and generate proof using EreSP1
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

                let zkvm = EreSP1::new(elf_bytes);
                let (proof, _report) = zkvm.prove(&input).map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to generate proof: {}", e),
                    )
                })?;

                Ok(Json(ProveResponse { program_id, proof }))
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
    use ere_sp1::RV32_IM_SUCCINCT_ZKVM_ELF;
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Arc;
    use tempfile::TempDir;
    use tokio::sync::RwLock;
    use zkvm_interface::Compiler;

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

    // Helper function to ensure test program is compiled
    fn ensure_test_program_compiled() -> Vec<u8> {
        let program_dir = PathBuf::from("programs/sp1");
        let target_dir =
            program_dir.join("target/elf-compilation/riscv32im-succinct-zkvm-elf/release");
        let elf_path = target_dir.join("ere-test-sp1-guest");

        if !elf_path.exists() {
            println!("Compiling test program...");
            RV32_IM_SUCCINCT_ZKVM_ELF::compile(&program_dir)
                .expect("Failed to compile test program");
        }

        fs::read(&elf_path).expect("Failed to read compiled ELF")
    }

    #[tokio::test]
    async fn test_prove_program_success() {
        let elf_bytes = ensure_test_program_compiled();
        let (state, _temp_dir) = create_test_state();
        let program_id = "sp1".to_string();
        {
            let mut programs = state.programs.write().await;
            let elf_base64 = BASE64.encode(&elf_bytes);
            programs.insert(program_id.clone(), Program::SP1(elf_base64));
        }

        let request = ProveRequest {
            program_id: program_id.clone(),
            input: ProgramInput {
                value1: 42,
                value2: 10,
            },
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
            program_id: "non_existent".to_string(),
            input: ProgramInput {
                value1: 42,
                value2: 10,
            },
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
            input: ProgramInput {
                value1: 42,
                value2: 10,
            },
        };

        let result = prove_program(State(state), Json(request)).await;

        assert!(result.is_err());
        let (status, _message) = result.unwrap_err();
        assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
    }

    #[tokio::test]
    #[should_panic]
    async fn test_prove_program_fails_with_no_input() {
        let elf_bytes = ensure_test_program_compiled();
        let (state, _temp_dir) = create_test_state();
        let program_id = "test_program".to_string();
        {
            let mut programs = state.programs.write().await;
            let elf_base64 = BASE64.encode(&elf_bytes);
            programs.insert(program_id.clone(), Program::SP1(elf_base64));
        }

        // Provide zero input
        let request = ProveRequest {
            program_id: program_id.clone(),
            input: ProgramInput {
                value1: 0,
                value2: 0,
            },
        };

        // Call the handler directly
        let _ = prove_program(State(state), Json(request)).await;
    }
}
