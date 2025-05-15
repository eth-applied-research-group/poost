use axum::{Json, extract::State, http::StatusCode};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use std::{fs, str::FromStr};
use tracing::instrument;
use uuid::Uuid;

use crate::common::{AppState, Program, ZkVMType};

#[derive(Debug, Deserialize)]
pub struct RegisterProgramRequest {
    pub program_name: String,
    #[serde(deserialize_with = "deserialize_zkvm_type")]
    pub zkvm: ZkVMType,
    pub elf_file: String,         // Base64-encoded ELF file
    pub compiler_version: String, // Version of the compiler used
}

#[derive(Debug, Serialize)]
pub struct RegisterProgramResponse {
    pub program_id: String,
    pub status: String,
}

pub fn deserialize_zkvm_type<'de, D>(deserializer: D) -> Result<ZkVMType, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    ZkVMType::from_str(&s).map_err(|e| serde::de::Error::custom(e))
}

#[axum::debug_handler]
#[instrument(skip_all)]
pub async fn register_program(
    State(state): State<AppState>,
    Json(req): Json<RegisterProgramRequest>,
) -> Result<Json<RegisterProgramResponse>, (StatusCode, String)> {
    let program_id = Uuid::new_v4().to_string();
    let program_dir = state.programs_dir.join(&program_id);

    // Create directory for the program
    fs::create_dir_all(&program_dir).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create program directory: {e}"),
        )
    })?;

    // Decode the ELF file
    let elf_bytes = BASE64.decode(&req.elf_file).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Invalid base64 encoding: {e}"),
        )
    })?;

    // Write the ELF file
    let elf_path = program_dir.join("program.elf");
    fs::write(&elf_path, elf_bytes).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to write ELF file: {e}"),
        )
    })?;

    // Write compiler version info
    let version_path = program_dir.join("compiler_version.txt");
    fs::write(&version_path, &req.compiler_version).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to write compiler version: {e}"),
        )
    })?;

    let program = match req.zkvm {
        ZkVMType::Risc0 => Program::Risc0(req.program_name.clone()),
        ZkVMType::SP1 => Program::SP1(req.program_name.clone()),
    };
    {
        let mut programs = state.programs.write().await;
        programs.insert(program_id.clone(), program);
    }

    Ok(Json(RegisterProgramResponse {
        program_id,
        status: format!(
            "registered with {} (compiler version: {})",
            req.zkvm, req.compiler_version
        ),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ere_sp1::RV32_IM_SUCCINCT_ZKVM_ELF;
    use std::collections::HashMap;
    use std::fs;
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

    /// Compiles the basic example program and returns the base64-encoded ELF file as a String
    fn compile_and_encode_elf() -> String {
        use std::path::PathBuf;

        let example_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("sp1")
            .join("compile")
            .join("basic");

        let elf_bytes =
            RV32_IM_SUCCINCT_ZKVM_ELF::compile(&example_dir).expect("Failed to compile example");
        BASE64.encode(&elf_bytes)
    }

    #[tokio::test]
    async fn test_register_sp1_program() {
        let (state, _temp_dir) = create_test_state();
        let elf_file = compile_and_encode_elf();
        let request = RegisterProgramRequest {
            program_name: "basic".to_string(),
            zkvm: ZkVMType::SP1,
            elf_file,
            compiler_version: "0.1.0".to_string(),
        };
        let result = register_program(State(state.clone()), Json(request)).await;
        assert!(result.is_ok());
        let response = result.unwrap().0;
        assert!(!response.program_id.is_empty());
        assert!(response.status.contains("sp1"));
        assert!(response.status.contains("0.1.0"));
        let programs = state.programs.read().await;
        assert!(programs.contains_key(&response.program_id));
        match programs.get(&response.program_id).unwrap() {
            Program::SP1(name) => assert_eq!(name, "basic"),
            _ => panic!("Expected SP1 program"),
        }
    }

    #[tokio::test]
    async fn test_register_invalid_base64() {
        let (state, _temp_dir) = create_test_state();

        let request = RegisterProgramRequest {
            program_name: "test_program".to_string(),
            zkvm: ZkVMType::SP1,
            elf_file: "invalid base64".to_string(),
            compiler_version: "0.1.0".to_string(),
        };

        let result = register_program(State(state), Json(request)).await;
        assert!(result.is_err());

        let (status, message) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(message.contains("Invalid base64 encoding"));
    }
}
