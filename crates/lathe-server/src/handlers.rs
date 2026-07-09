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

#[cfg(test)]
mod tests {
    use super::*;
    use lathe_core::executor::Executor;
    use lathe_core::graph::port::{Connection, Port};
    use lathe_core::graph::{GraphDefinition, GraphVersion, LatheGraph};
    use lathe_core::node_defs::{EndNodeDef, NodeKind, StartNodeDef};
    use lathe_core::registry;

    fn test_state() -> Arc<ServerState> {
        let start = StartNodeDef {
            id: "start".to_string(),
            ..Default::default()
        };
        let end = EndNodeDef {
            id: "end".to_string(),
            out_pointers: vec!["/message".to_string()],
            ..Default::default()
        };
        let connection = Connection {
            from: Port {
                node_id: start.id.clone(),
                name: "out".to_string(),
            },
            to: Port {
                node_id: end.id.clone(),
                name: "in".to_string(),
            },
        };

        let definition = GraphDefinition {
            graph_version: GraphVersion::V1,
            name: "test-pipeline".to_string(),
            nodes: vec![NodeKind::Start(start), NodeKind::End(end)],
            connections: vec![connection],
            provider_configs: Default::default(),
        };

        let graph = LatheGraph::from_def(definition, true).unwrap();
        let nodes = registry::materialize(&graph.definition.nodes, &graph.definition.provider_configs).unwrap();
        let executor = Executor::new(graph, nodes);

        Arc::new(ServerState {
            executor,
            pipeline_name: "test-pipeline".to_string(),
        })
    }

    #[tokio::test]
    async fn health_reports_ok_and_pipeline_name() {
        let response = health(State(test_state())).await;
        assert_eq!(response.0["status"], "ok");
        assert_eq!(response.0["pipeline"], "test-pipeline");
    }

    #[tokio::test]
    async fn invoke_runs_pipeline_and_returns_selected_state() {
        let body = serde_json::json!({"message": "hello"});
        let result = invoke(State(test_state()), Json(body)).await.unwrap();
        assert_eq!(result.0["message"], "hello");
    }

    #[tokio::test]
    async fn invoke_rejects_non_object_body() {
        let body = serde_json::json!("just a string");
        let (status, _) = invoke(State(test_state()), Json(body)).await.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn invoke_rejects_empty_object_body() {
        let body = serde_json::json!({});
        let (status, _) = invoke(State(test_state()), Json(body)).await.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn invoke_returns_server_error_when_pipeline_execution_fails() {
        // "end" requires /message but the request omits it, so EndNode::execute fails.
        let body = serde_json::json!({"other": "value"});
        let (status, _) = invoke(State(test_state()), Json(body)).await.unwrap_err();
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    }
}
