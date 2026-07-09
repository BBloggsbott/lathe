use std::path::PathBuf;
use serde_json::{Map, Value};
use lathe_core::{registry, yaml};
use lathe_core::executor::Executor;
use lathe_core::state::AgentState;
use anyhow::Result;

pub async fn run_pipeline(pipeline: PathBuf, message: String) -> Result<()> {
    let graph = yaml::load(pipeline.as_path(), true)?;
    tracing::info!("Loaded pipeline: {}", graph.name);

    let nodes =
        registry::inflate(&graph.definition.nodes, &graph.definition.provider_configs)?;
    tracing::info!("Executable nodes built: {} nodes", nodes.len());

    let executor = Executor::new(graph, nodes);

    let mut message_as_state = Map::new();
    message_as_state.insert("message".to_string(), Value::String(message.clone()));

    let initial = AgentState::new(message_as_state);
    let result = executor.run(initial).await?;

    println!("{}", result.pretty_string()?);
    Ok(())
}