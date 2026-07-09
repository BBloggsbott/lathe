use crate::state::ServerState;
use anyhow::Result;
use axum::Router;
use axum::routing::{get, post};
use lathe_core::executor::Executor;
use std::sync::Arc;

mod handlers;
mod state;

pub fn app(executor: Executor, pipeline_name: String) -> Router {
    let state = Arc::new(ServerState {
        executor,
        pipeline_name,
    });

    Router::new()
        .route("/invoke", post(handlers::invoke))
        .route("/health", get(handlers::health))
        .with_state(state)
}

pub async fn serve(executor: Executor, pipeline_name: String, host: &str, port: u16) -> Result<()> {
    let app = app(executor, pipeline_name);
    let listener = tokio::net::TcpListener::bind((host, port)).await?;
    tracing::info!("Lathe server listening on {host}:{port}");
    axum::serve(listener, app).await?;
    Ok(())
}
