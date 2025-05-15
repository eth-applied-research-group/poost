use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::common::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProveRequest {
    /// TODO: Maybe we can make this a Vec<Vec<u8>>
    /// TODO so each input is already serialized properly
    pub input: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProveResponse {
    pub program_id: String,
    pub proof: String,
}

#[axum::debug_handler]
#[instrument(skip_all)]
pub async fn prove_program(
    Path(program_id): Path<String>,
    State(state): State<AppState>,
    Json(_req): Json<ProveRequest>,
) -> Result<Json<ProveResponse>, (StatusCode, String)> {
    if let Some(_program) = state.programs.read().await.get(&program_id) {
        // TODO: In a real implementation, generate a proof based on the program type
        Ok(Json(ProveResponse {
            program_id,
            proof: "Stub proof data".to_string(),
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
        endpoints::{prove::ProveRequest, prove_program},
    };

    #[tokio::test]
    async fn test_prove_program() {
        let state = AppState {
            programs_dir: temp_dir(),
            programs: Arc::new(RwLock::new(HashMap::new())),
        };

        let req = ProveRequest {
            input: "test_input".to_string(),
        };

        let result = prove_program(Path("non_existent".to_string()), State(state), Json(req)).await;

        assert!(result.is_err());
        let (status, _) = result.err().unwrap();
        assert_eq!(status, axum::http::StatusCode::NOT_FOUND);
    }
}
