use crate::graph::{GraphVersion, LatheGraph};
use crate::nodes::LatheNode;
use crate::state::AgentState;
use anyhow::Result;
use std::collections::HashMap;

pub struct Executor {
    graph: LatheGraph,
    nodes: HashMap<String, Box<dyn LatheNode>>,
}

impl Executor {
    pub fn new(graph: LatheGraph, nodes: HashMap<String, Box<dyn LatheNode>>) -> Self {
        Self { graph, nodes }
    }

    pub async fn run_v1_graph(&self, agent_state: AgentState) -> Result<AgentState> {
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

    pub async fn run(&self, agent_state: AgentState) -> Result<AgentState> {
        match self.graph.graph_version {
            GraphVersion::V1 => self.run_v1_graph(agent_state).await,
        }
    }
}
