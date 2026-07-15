use crate::node_defs::NodeKind;
use crate::provider::LLMProviderConfigs;
use crate::tool_defs::ToolKind;
use anyhow::Result;
use petgraph::algo::toposort;
use petgraph::graph::{DiGraph, NodeIndex};
use port::Connection;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub mod port;

/// Serializable, storage-format description of a pipeline: its nodes, the connections between
/// them, and the provider configs those nodes reference. This is what gets read from and
/// written to YAML via [`crate::yaml`]; use [`LatheGraph::from_def`] to turn it into a runnable,
/// validated graph.
//todo: Do I need to have this in my memory after the graph is loaded?
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct GraphDefinition {
    pub graph_version: GraphVersion,
    pub name: String,
    pub nodes: Vec<NodeKind>,
    pub connections: Vec<Connection>,
    pub provider_configs: LLMProviderConfigs,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub tools: HashMap<String, ToolKind>,
}

/// Schema version of a [`GraphDefinition`], used by [`crate::executor::Executor`] to pick the
/// correct execution strategy.
#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy)]
pub enum GraphVersion {
    #[default]
    V1,
}

/// A [`GraphDefinition`] indexed into a [`petgraph`] directed graph, ready for topological
/// execution. Node IDs are used as vertices; edges come from the definition's connections.
pub struct LatheGraph {
    pub graph_version: GraphVersion,
    pub name: String,
    pub definition: GraphDefinition,
    pub node_index: HashMap<String, NodeIndex>,
    pub digraph: DiGraph<String, ()>,
}

// todo: Currently implemented as an acyclic graph. But Agentic Graphs needs cycles for retries and such.
//  Need to explore alternative way to implement cycles in the graph
impl LatheGraph {
    /// Builds a [`LatheGraph`] from a [`GraphDefinition`], indexing nodes and connections into
    /// a digraph. When `validate` is `true`, also runs [`Self::validate`] and returns its error
    /// (if any) instead of the constructed graph.
    pub fn from_def(definition: GraphDefinition, validate: bool) -> Result<Self> {
        let mut digraph: DiGraph<String, ()> = DiGraph::new();
        let mut node_index_map: HashMap<String, NodeIndex> = HashMap::new();

        for node in &definition.nodes {
            let id = node.id().to_string();
            let idx = digraph.add_node(id.clone());
            node_index_map.insert(id, idx);
        }

        for connection in &definition.connections {
            let from = node_index_map
                .get(&connection.from.node_id)
                .copied()
                .ok_or_else(|| {
                    anyhow::anyhow!("Unknown node for id: {}", connection.from.node_id)
                })?;

            let to = node_index_map
                .get(&connection.to.node_id)
                .copied()
                .ok_or_else(|| {
                    anyhow::anyhow!("Unknown node for id: {}", connection.from.node_id)
                })?;

            digraph.add_edge(from, to, ());
        }

        if validate {
            let lathe_graph = Self {
                graph_version: definition.graph_version,
                name: definition.name.clone(),
                definition,
                node_index: node_index_map,
                digraph,
            };
            lathe_graph.validate()?;
            Ok(lathe_graph)
        } else {
            Ok(Self {
                graph_version: definition.graph_version,
                name: definition.name.clone(),
                definition,
                node_index: node_index_map,
                digraph,
            })
        }
    }

    /// Returns node IDs in topological (dependency-respecting) execution order.
    /// Errors if the graph contains a cycle.
    pub fn topological_order(&self) -> Result<Vec<String>> {
        let sorted = toposort(&self.digraph, None).map_err(|cycle| {
            anyhow::anyhow!("graph contains a cycle: {}", self.digraph[cycle.node_id()])
        })?;

        Ok(sorted
            .into_iter()
            .map(|idx| self.digraph[idx].clone())
            .collect())
    }

    /// Checks that the graph's leaf nodes (no outgoing edges) exactly match its declared
    /// [`NodeKind::End`] nodes. Every end node must be a leaf and vice versa.
    pub fn validate(&self) -> Result<()> {
        let end_node_ids = self.end_node_ids();
        let leaf_ids: HashSet<&str> = self
            .digraph
            .node_indices()
            .filter(|&idx| {
                self.digraph
                    .neighbors_directed(idx, petgraph::Direction::Outgoing)
                    .count()
                    == 0
            })
            .map(|idx| self.digraph[idx].as_str())
            .collect();

        let end_ids: HashSet<&str> = end_node_ids.into_iter().collect();

        if leaf_ids != end_ids {
            let mut issues: Vec<String> = vec![];
            if leaf_ids.difference(&end_ids).count() > 0 {
                issues.push(format!(
                    "Found leaf nodes that are not end nodes: {:?}. Every end node must be a leaf, and every leaf must be an end node",
                    leaf_ids.difference(&end_ids)
                ));
            }

            if end_ids.difference(&leaf_ids).count() > 0 {
                issues.push(format!(
                    "Found end nodes that are not leaf nodes: {:?}. Every end node must be a leaf, and every leaf must be an end node",
                    end_ids.difference(&leaf_ids)
                ));
            }

            return Err(anyhow::anyhow!(issues.join("\n")));
        }

        Ok(())
    }

    /// IDs of all [`NodeKind::End`] nodes in the underlying definition.
    fn end_node_ids(&self) -> Vec<&str> {
        let node_ids: Vec<&str> = self
            .definition
            .nodes
            .iter()
            .filter_map(|node| match node {
                NodeKind::End(node) => Some(node.id.as_str()),
                _ => None,
            })
            .collect();

        node_ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node_defs::{EndNodeDef, StartNodeDef};
    use port::Port;

    fn connection(from: &str, to: &str) -> Connection {
        Connection {
            from: Port {
                node_id: from.to_string(),
                name: "out".to_string(),
            },
            to: Port {
                node_id: to.to_string(),
                name: "in".to_string(),
            },
        }
    }

    fn start_end_definition() -> GraphDefinition {
        GraphDefinition {
            graph_version: GraphVersion::V1,
            name: "test-graph".to_string(),
            nodes: vec![
                NodeKind::Start(StartNodeDef {
                    id: "start".to_string(),
                    ..Default::default()
                }),
                NodeKind::End(EndNodeDef {
                    id: "end".to_string(),
                    ..Default::default()
                }),
            ],
            connections: vec![connection("start", "end")],
            provider_configs: Default::default(),
            tools: Default::default(),
        }
    }

    #[test]
    fn from_def_indexes_nodes_and_edges() {
        let graph = LatheGraph::from_def(start_end_definition(), false).unwrap();
        assert_eq!(graph.node_index.len(), 2);
        assert_eq!(graph.digraph.edge_count(), 1);
    }

    #[test]
    fn from_def_unknown_source_node_errors() {
        let mut def = start_end_definition();
        def.connections = vec![connection("missing", "end")];
        assert!(LatheGraph::from_def(def, false).is_err());
    }

    #[test]
    fn from_def_unknown_target_node_errors() {
        let mut def = start_end_definition();
        def.connections = vec![connection("start", "missing")];
        assert!(LatheGraph::from_def(def, false).is_err());
    }

    #[test]
    fn topological_order_respects_dependencies() {
        let graph = LatheGraph::from_def(start_end_definition(), false).unwrap();
        let order = graph.topological_order().unwrap();
        assert_eq!(order, vec!["start".to_string(), "end".to_string()]);
    }

    #[test]
    fn topological_order_errors_on_cycle() {
        let mut def = start_end_definition();
        def.connections.push(connection("end", "start"));
        let graph = LatheGraph::from_def(def, false).unwrap();
        assert!(graph.topological_order().is_err());
    }

    #[test]
    fn validate_passes_when_leaves_match_end_nodes() {
        let graph = LatheGraph::from_def(start_end_definition(), false).unwrap();
        assert!(graph.validate().is_ok());
    }

    #[test]
    fn from_def_with_validate_true_runs_validation() {
        assert!(LatheGraph::from_def(start_end_definition(), true).is_ok());
    }

    #[test]
    fn validate_fails_when_leaf_is_not_declared_end_node() {
        // "end" is a leaf but only declared as a Start node, so it's not in end_node_ids.
        let mut def = start_end_definition();
        def.nodes = vec![
            NodeKind::Start(StartNodeDef {
                id: "start".to_string(),
                ..Default::default()
            }),
            NodeKind::Start(StartNodeDef {
                id: "end".to_string(),
                ..Default::default()
            }),
        ];
        let graph = LatheGraph::from_def(def, false).unwrap();
        assert!(graph.validate().is_err());
    }

    #[test]
    fn validate_fails_when_declared_end_node_is_not_a_leaf() {
        // "middle" is declared as an End node but has an outgoing edge to "end", so it isn't
        // actually a leaf.
        let def = GraphDefinition {
            graph_version: GraphVersion::V1,
            name: "test-graph".to_string(),
            nodes: vec![
                NodeKind::Start(StartNodeDef {
                    id: "start".to_string(),
                    ..Default::default()
                }),
                NodeKind::End(EndNodeDef {
                    id: "middle".to_string(),
                    ..Default::default()
                }),
                NodeKind::End(EndNodeDef {
                    id: "end".to_string(),
                    ..Default::default()
                }),
            ],
            connections: vec![connection("start", "middle"), connection("middle", "end")],
            provider_configs: Default::default(),
            tools: Default::default(),
        };
        let graph = LatheGraph::from_def(def, false).unwrap();
        assert!(graph.validate().is_err());
    }
}
