use crate::node_defs::NodeKind;
use crate::port::Connection;
use anyhow::Result;
use petgraph::algo::toposort;
use petgraph::graph::{DiGraph, NodeIndex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct GraphDefinition {
    pub name: String,
    pub nodes: Vec<NodeKind>,
    pub connections: Vec<Connection>,
}

pub struct LatheGraph {
    pub definition: GraphDefinition,
    pub node_index: HashMap<String, NodeIndex>,
    pub digraph: DiGraph<String, ()>,
}

// todo: Currently implemented as an acyclic graph. But Agentic Graphs needs cycles for retries and such.
//  Need to explore alternative way to implement cycles in the graph
impl LatheGraph {
    pub fn from_def(definition: GraphDefinition) -> Result<Self> {
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

        Ok(Self {
            definition,
            node_index: node_index_map,
            digraph,
        })
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
}
