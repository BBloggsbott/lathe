//! Runtime, executable node implementations, as opposed to the serializable definitions in
//! [`crate::node_defs`]. Built from those definitions via [`crate::registry::materialize`] and run
//! in sequence by [`crate::executor::Executor`].

pub mod end;
pub mod llm;
pub mod start;

use crate::state::AgentState;
use anyhow::Result;
use async_trait::async_trait;

/// A single executable step in a Lathe graph. Implementations take the current agent state,
/// perform their work, and return the (possibly updated) state for the next node.
#[async_trait]
pub trait LatheNode: Send + Sync {
    /// Human-readable label for logging/debugging.
    fn label(&self) -> &str;

    /// Unique node ID matching the corresponding node definition and graph vertex.
    fn id(&self) -> &str;

    /// Runs this node's behavior against the given state, returning the resulting state.
    async fn execute(&self, agent_state: AgentState) -> Result<AgentState>;
}
