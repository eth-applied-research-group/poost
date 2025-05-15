use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::common::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteRequest {
    pub input: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteResponse {
    pub program_id: String,
    pub output: String,
    pub execution_time: f64,
}

#[axum::debug_handler]
#[instrument(skip_all)]
pub async fn execute_program(
    Path(program_id): Path<String>,
    State(state): State<AppState>,
    Json(_req): Json<ExecuteRequest>,
) -> Result<Json<ExecuteResponse>, (StatusCode, String)> {
    if let Some(_program) = state.programs.read().await.get(&program_id) {
        // TODO: stub program, real program will check the zkvm type and then
        // TODO: choose the zkvm to execute the program
        Ok(Json(ExecuteResponse {
            program_id,
            output: "Stub execution output".to_string(),
            execution_time: 0.1,
        }))
    } else {
        Err((StatusCode::NOT_FOUND, "Program not found".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        env::temp_dir,
        sync::Arc,
    };
    use tokio::sync::RwLock;

    use axum::{
        Json,
        extract::{Path, State},
    };

    use crate::{
        common::AppState,
        endpoints::{execute::ExecuteRequest, execute_program},
    };

    #[tokio::test]
    async fn test_execute_program() {
        let state = AppState {
            programs_dir: temp_dir(),
            programs: Arc::new(RwLock::new(HashMap::new())),
        };

        let req = ExecuteRequest {
            input: "input_to_program".to_string(),
        };

        let result =
            execute_program(Path("non_existent".to_string()), State(state), Json(req)).await;

        assert!(result.is_err());
        let (status, _) = result.err().unwrap();
        assert_eq!(status, axum::http::StatusCode::NOT_FOUND);
    }
}
