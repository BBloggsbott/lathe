//! Axum request handlers for the Lathe HTTP server.

use crate::state::ServerState;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use lathe_core::state::AgentState;
use serde_json::Value;
use std::sync::Arc;

/// `GET /health` - reports server liveness and the name of the loaded pipeline.
pub async fn health(State(state): State<Arc<ServerState>>) -> Json<Value> {
    Json(serde_json::json!({"status": "ok", "pipeline": state.pipeline_name}))
}

/// `POST /invoke` - runs the loaded pipeline against the request body as the initial agent
/// state, returning the resulting state as JSON. Responds `400` if the body isn't a non-empty
/// JSON object, or `500` if pipeline execution fails.
pub async fn invoke(
    State(state): State<Arc<ServerState>>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let agent_state = AgentState::try_from(body).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;

    match state.executor.run(agent_state).await {
        Ok(result) => Ok(Json(result.into_value())),
        Err(e) => {
            tracing::error!("pipeline execution failed: {e:#}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}
