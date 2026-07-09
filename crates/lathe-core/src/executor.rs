use crate::graph::{GraphVersion, LatheGraph};
use crate::nodes::LatheNode;
use crate::state::AgentState;
use anyhow::Result;
use std::collections::HashMap;

/// Drives a [`LatheGraph`] end-to-end by executing its runtime `nodes` in topological order,
/// threading the [`AgentState`] from one node to the next.
pub struct Executor {
    graph: LatheGraph,
    nodes: HashMap<String, Box<dyn LatheNode>>,
}

impl Executor {
    /// Pairs a validated graph with its inflated executable nodes (see
    /// [`crate::registry::materialize`]).
    pub fn new(graph: LatheGraph, nodes: HashMap<String, Box<dyn LatheNode>>) -> Self {
        Self { graph, nodes }
    }

    /// Executes nodes in topological order for a [`GraphVersion::V1`] graph,
    /// passing the resulting state from each node into the next.
    async fn run_v1_graph(&self, agent_state: AgentState) -> Result<AgentState> {
        let order = self.graph.topological_order()?;
        let mut state = agent_state;

        for node_id in &order {
            let node = self.nodes.get(node_id).ok_or_else(|| {
                anyhow::anyhow!("Could not find runtime node with id {node_id} in LatheGraph")
            })?;

            state = node.execute(state).await?;
        }

        Ok(state)
    }

    /// Runs the pipeline against the given initial `agent_state`, dispatching on the graph's
    /// [`GraphVersion`], and returns the final state.
    pub async fn run(&self, agent_state: AgentState) -> Result<AgentState> {
        match self.graph.graph_version {
            GraphVersion::V1 => self.run_v1_graph(agent_state).await,
        }
    }
}
