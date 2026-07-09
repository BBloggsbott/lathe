use crate::nodes::LatheNode;
use crate::state::AgentState;
use anyhow::bail;
use async_trait::async_trait;

/// A graph exit point node. Selects `out_pointers` out of the final agent state to produce the
/// graph's output, corresponding to a [`crate::node_defs::EndNodeDef`]. Every end node must be
/// a leaf node in the graph (see [`crate::graph::LatheGraph::validate`]).
#[derive(Debug)]
pub struct EndNode {
    id: String,
    label: String,
    out_pointers: Vec<String>,
}

#[async_trait]
impl LatheNode for EndNode {
    fn label(&self) -> &str {
        self.label.as_str()
    }

    fn id(&self) -> &str {
        self.id.as_str()
    }

    /// Logs and selects `out_pointers` out of `agent_state`; errors if the state is empty or a
    /// pointer is missing.
    async fn execute(&self, agent_state: AgentState) -> anyhow::Result<AgentState> {
        tracing::info!(
            "Ending Graph execution with agent state: {:?} and out_pointers: {:?}",
            agent_state,
            self.out_pointers
        );
        if agent_state.is_empty() {
            bail!("Empty Agent State. Nothing to process")
        }
        agent_state.select(&self.out_pointers)
    }
}

impl EndNode {
    /// Creates a new [`EndNode`] with the given ID, label, and agent state pointers to select
    /// as output.
    pub fn new(id: &str, label: &str, out_pointers: &[String]) -> Self {
        Self {
            id: id.to_owned(),
            label: label.to_owned(),
            out_pointers: out_pointers.to_owned(),
        }
    }

    /// The agent state pointers this node selects into its output.
    pub fn out_pointers(&self) -> &Vec<String> {
        &self.out_pointers
    }
}
