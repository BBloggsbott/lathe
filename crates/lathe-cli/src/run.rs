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

#[cfg(test)]
mod tests {
    use super::*;
    use lathe_core::graph::port::{Connection, Port};
    use lathe_core::graph::{GraphDefinition, GraphVersion, LatheGraph};
    use lathe_core::node_defs::{EndNodeDef, NodeKind, StartNodeDef};

    struct TempYamlPath(PathBuf);

    impl TempYamlPath {
        fn new(name: &str) -> Self {
            let mut path = std::env::temp_dir();
            path.push(format!(
                "lathe-run-test-{name}-{}.yaml",
                uuid::Uuid::new_v4()
            ));
            Self(path)
        }
    }

    impl Drop for TempYamlPath {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    fn write_start_end_pipeline(path: &std::path::Path) {
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
            name: "run-test-pipeline".to_string(),
            nodes: vec![NodeKind::Start(start), NodeKind::End(end)],
            connections: vec![connection],
            provider_configs: Default::default(),
            tools: Default::default(),
        };

        let graph = LatheGraph::from_def(definition, true).unwrap();
        lathe_core::yaml::save(&graph, path).unwrap();
    }

    #[tokio::test]
    async fn run_pipeline_succeeds_for_a_valid_pipeline() {
        let temp = TempYamlPath::new("valid");
        write_start_end_pipeline(&temp.0);

        assert!(
            run_pipeline(temp.0.clone(), "hello".to_string())
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn run_pipeline_errors_for_a_missing_file() {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "lathe-run-test-missing-{}.yaml",
            uuid::Uuid::new_v4()
        ));

        assert!(run_pipeline(path, "hello".to_string()).await.is_err());
    }
}
