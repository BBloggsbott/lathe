//! Loading and saving [`GraphDefinition`]s as YAML files.

use crate::graph::{GraphDefinition, LatheGraph};
use anyhow::{Context, Result};
use std::fs::File;
use std::path::Path;

/// Writes `graph`'s underlying [`GraphDefinition`] to `path` as YAML.
pub fn save(graph: &LatheGraph, path: &Path) -> Result<()> {
    let out_file = File::create(path).with_context(|| match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() && !parent.exists() => format!(
            "failed to create file at {}: directory {} does not exist",
            path.display(),
            parent.display()
        ),
        _ => format!("failed to create file at {}", path.display()),
    })?;
    serde_yaml::to_writer(out_file, &graph.definition)?;

    Ok(())
}

/// Reads a [`GraphDefinition`] from the YAML file at `path` and builds a [`LatheGraph`] from
/// it, optionally validating it (see [`LatheGraph::from_def`]).
pub fn load(path: &Path, validate: bool) -> Result<LatheGraph> {
    let file = File::open(path)?;
    let graph_definition: GraphDefinition = serde_yaml::from_reader(file)?;
    let graph = LatheGraph::from_def(graph_definition, validate)?;
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GraphVersion;
    use crate::graph::port::{Connection, Port};
    use crate::node_defs::{EndNodeDef, NodeKind, StartNodeDef};

    struct TempYamlPath(std::path::PathBuf);

    impl TempYamlPath {
        fn new(name: &str) -> Self {
            let mut path = std::env::temp_dir();
            path.push(format!(
                "lathe-yaml-test-{name}-{}.yaml",
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

    fn start_end_graph() -> LatheGraph {
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
            name: "yaml-test-graph".to_string(),
            nodes: vec![NodeKind::Start(start), NodeKind::End(end)],
            connections: vec![connection],
            provider_configs: Default::default(),
        };

        LatheGraph::from_def(definition, true).unwrap()
    }

    #[test]
    fn save_then_load_round_trips_the_graph_definition() {
        let temp = TempYamlPath::new("round-trip");
        let graph = start_end_graph();

        save(&graph, &temp.0).unwrap();
        let loaded = load(&temp.0, true).unwrap();

        assert_eq!(loaded.name, "yaml-test-graph");
        assert_eq!(loaded.definition.nodes.len(), 2);
        assert_eq!(loaded.definition.connections.len(), 1);
    }

    #[test]
    fn load_nonexistent_file_errors() {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "lathe-yaml-test-missing-{}.yaml",
            uuid::Uuid::new_v4()
        ));
        assert!(load(&path, true).is_err());
    }
}
