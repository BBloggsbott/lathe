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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GraphDefinition;
    use crate::graph::port::{Connection, Port};
    use crate::node_defs::{EndNodeDef, NodeKind, StartNodeDef};
    use crate::registry;
    use serde_json::json;

    fn start_end_graph(out_pointers: Vec<String>) -> LatheGraph {
        let start = StartNodeDef {
            id: "start".to_string(),
            ..Default::default()
        };
        let end = EndNodeDef {
            id: "end".to_string(),
            out_pointers,
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
            name: "test-graph".to_string(),
            nodes: vec![NodeKind::Start(start), NodeKind::End(end)],
            connections: vec![connection],
            provider_configs: Default::default(),
        };

        LatheGraph::from_def(definition, true).unwrap()
    }

    #[tokio::test]
    async fn run_threads_state_through_nodes_and_selects_output() {
        let graph = start_end_graph(vec!["/message".to_string()]);
        let nodes = registry::materialize(&graph.definition.nodes, &graph.definition.provider_configs).unwrap();
        let executor = Executor::new(graph, nodes);

        let initial = AgentState::try_from(json!({"message": "hello"})).unwrap();
        let result = executor.run(initial).await.unwrap();

        assert_eq!(result.get("/message"), Some(&json!("hello")));
    }

    #[tokio::test]
    async fn run_errors_when_a_node_is_missing_from_the_executable_map() {
        let graph = start_end_graph(vec!["/message".to_string()]);
        // Deliberately materialize an empty node map so the graph's own nodes can't be found.
        let executor = Executor::new(graph, HashMap::new());

        let initial = AgentState::try_from(json!({"message": "hello"})).unwrap();
        assert!(executor.run(initial).await.is_err());
    }
}
