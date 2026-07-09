//! One-shot pipeline execution for the `lathe run` CLI subcommand.

use anyhow::Result;
use lathe_core::executor::Executor;
use lathe_core::state::AgentState;
use lathe_core::{registry, yaml};
use serde_json::{Map, Value};
use std::path::PathBuf;

/// Loads the pipeline YAML at `pipeline`, runs it once against `message` as the initial
/// `/message` agent state, and prints the resulting state as pretty-printed JSON.
pub async fn run_pipeline(pipeline: PathBuf, message: String) -> Result<()> {
    let graph = yaml::load(pipeline.as_path(), true)?;
    tracing::info!("Loaded pipeline: {}", graph.name);

    let nodes = registry::materialize(&graph.definition.nodes, &graph.definition.provider_configs)?;
    tracing::info!("Built executable nodes: {} nodes", nodes.len());

    let executor = Executor::new(graph, nodes);
    tracing::info!("Built Executor");

    let mut message_as_state = Map::new();
    message_as_state.insert("message".to_string(), Value::String(message.clone()));

    let initial = AgentState::new(message_as_state);
    let result = executor.run(initial).await?;

    println!("{}", result.pretty_string()?);
    Ok(())
}
