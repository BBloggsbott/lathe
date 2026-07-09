use crate::node_defs::NodeKind;
use crate::provider::LLMProviderConfigs;
use anyhow::Result;
use petgraph::algo::toposort;
use petgraph::graph::{DiGraph, NodeIndex};
use port::Connection;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub mod port;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct GraphDefinition {
    pub graph_version: GraphVersion,
    pub name: String,
    pub nodes: Vec<NodeKind>,
    pub connections: Vec<Connection>,
    pub provider_configs: LLMProviderConfigs,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy)]
pub enum GraphVersion {
    #[default]
    V1,
}

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

    pub fn topological_order(&self) -> Result<Vec<String>> {
        let sorted = toposort(&self.digraph, None).map_err(|cycle| {
            anyhow::anyhow!("graph contains a cycle: {}", &self.digraph[cycle.node_id()])
        })?;

        Ok(sorted
            .into_iter()
            .map(|idx| self.digraph[idx].clone())
            .collect())
    }

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
