//! Boots the HTTP server for the `lathe server` CLI subcommand.

use anyhow::Result;
use lathe_core::executor::Executor;
use lathe_core::{registry, yaml};
use lathe_server::serve;
use std::path::PathBuf;

/// Loads the pipeline YAML at `pipeline`, builds an [`Executor`] for it, and serves it over
/// HTTP at `host:port` (see [`serve`]).
pub async fn start_server(pipeline: PathBuf, host: &str, port: u16) -> Result<()> {
    let graph = yaml::load(pipeline.as_path(), true)?;
    let pipeline_name = graph.name.clone();
    tracing::info!("Loaded pipeline: {}", pipeline_name);

    let nodes = registry::materialize(&graph.definition.nodes, &graph.definition.provider_configs)?;
    tracing::info!("Built executable nodes: {} nodes", nodes.len());

    let executor = Executor::new(graph, nodes);
    tracing::info!("Built Executor");

    serve(executor, pipeline_name, host, port).await
}
