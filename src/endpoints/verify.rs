use axum::{Json, extract::State, http::StatusCode};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use ere_sp1::EreSP1;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use zkvm_interface::zkVM;

use crate::common::{AppState, Program};

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyRequest {
    pub program_id: String,
    pub proof: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyResponse {
    pub program_id: String,
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
            Program::SP1(elf_base64) => {
                // Decode the ELF file
                let elf_bytes = BASE64.decode(elf_base64).map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to decode ELF: {}", e),
                    )
                })?;

                // Decode the proof
                let proof_bytes = BASE64.decode(&req.proof).map_err(|e| {
                    (
                        StatusCode::BAD_REQUEST,
                        format!("Failed to decode proof: {}", e),
                    )
                })?;

                // Create EreSP1 instance and verify
                let zkvm = EreSP1::new(elf_bytes);
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
    use crate::common::Program;
    use crate::endpoints::prove::ProveRequest;
    use crate::endpoints::prove::prove_program;
    use std::collections::HashMap;
    use std::fs;
    use std::sync::Arc;
    use tempfile::TempDir;
    use tokio::sync::RwLock;
    use zkvm_interface::Input;

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
    async fn test_verify_proof_success() {
        let (state, _temp_dir) = create_test_state();
        let program_id = "test_program".to_string();

        // Read and encode the example program's ELF
        let elf_path = "tests/sp1/execute/basic/target/elf-compilation/riscv32im-succinct-zkvm-elf/release/ere-test-sp1-guest";
        let elf_bytes = fs::read(elf_path).expect("Failed to read ELF file");
        let elf_base64 = BASE64.encode(&elf_bytes);

        // Generate input data
        let mut input = Input::new();
        let n: u8 = 42;
        let a: u8 = 42;
        input.write(&n).expect("Failed to write n");
        input.write(&a).expect("Failed to write a");
        let input_chunks: Vec<Vec<u8>> = input.chunked_iter().map(|c| c.to_vec()).collect();

        {
            let mut programs = state.programs.write().await;
            programs.insert(program_id.clone(), Program::SP1(elf_base64));
        }

        // First generate a proof using the prove handler
        let prove_request = ProveRequest {
            program_id: program_id.clone(),
            input: input_chunks,
        };

        let prove_result = prove_program(State(state.clone()), Json(prove_request)).await;

        assert!(prove_result.is_ok());
        let proof = prove_result.unwrap().0.proof;
        let proof_base64 = BASE64.encode(&proof);

        // Now verify the proof
        let verify_request = VerifyRequest {
            program_id: program_id.clone(),
            proof: proof_base64,
        };

        let result = verify_proof(State(state), Json(verify_request)).await;

        assert!(result.is_ok());
        let response = result.unwrap().0;
        assert_eq!(response.program_id, program_id);
        assert!(response.verified);
    }

    #[tokio::test]
    async fn test_verify_proof_invalid_proof() {
        let (state, _temp_dir) = create_test_state();
        let program_id = "test_program".to_string();

        // Read and encode the example program's ELF
        let elf_path = "tests/sp1/execute/basic/target/elf-compilation/riscv32im-succinct-zkvm-elf/release/ere-test-sp1-guest";
        let elf_bytes = fs::read(elf_path).expect("Failed to read ELF file");
        let elf_base64 = BASE64.encode(&elf_bytes);

        {
            let mut programs = state.programs.write().await;
            programs.insert(program_id.clone(), Program::SP1(elf_base64));
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
            program_id: "non_existent".to_string(),
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
        let program_id = "test_program".to_string();
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
