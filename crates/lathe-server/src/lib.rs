//! HTTP server exposing a Lathe [`Executor`] over a small Axum API (`POST /invoke`,
//! `GET /health`). Used by the `lathe server` CLI subcommand via [`serve`].

use crate::state::ServerState;
use anyhow::Result;
use axum::Router;
use axum::routing::{get, post};
use lathe_core::executor::Executor;
use std::sync::Arc;

mod handlers;
mod state;

/// Builds the Axum [`Router`] for `executor`, wiring up `/invoke` and `/health`.
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

/// Builds the app for `executor` and serves it over HTTP at `host:port` until the process
/// exits.
pub async fn serve(executor: Executor, pipeline_name: String, host: &str, port: u16) -> Result<()> {
    let app = app(executor, pipeline_name);
    let listener = tokio::net::TcpListener::bind((host, port)).await?;
    tracing::info!("Lathe server listening on {host}:{port}");
    axum::serve(listener, app).await?;
    Ok(())
}
