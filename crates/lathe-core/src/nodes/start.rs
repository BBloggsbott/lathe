use crate::nodes::LatheNode;
use crate::state::AgentState;
use anyhow::bail;
use async_trait::async_trait;

/// The graph's entry point node. Passes the initial agent state through unchanged, failing if
/// it is empty. Every graph has exactly one, corresponding to a
/// [`crate::node_defs::StartNodeDef`].
#[derive(Debug)]
pub struct StartNode {
    id: String,
    label: String,
}

#[async_trait]
impl LatheNode for StartNode {
    fn label(&self) -> &str {
        self.label.as_str()
    }

    fn id(&self) -> &str {
        self.id.as_str()
    }

    /// Logs and returns `agent_state` unchanged; errors if it is empty.
    async fn execute(&self, agent_state: AgentState) -> anyhow::Result<AgentState> {
        tracing::info!(
            "Starting Graph execution with agent state: {:?}",
            agent_state
        );
        if agent_state.is_empty() {
            bail!("Empty Agent State. Nothing to process")
        }
        Ok(agent_state)
    }
}

impl StartNode {
    /// Creates a new [`StartNode`] with the given ID and label.
    pub fn new(id: &str, label: &str) -> Self {
        Self {
            id: id.to_owned(),
            label: label.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn id_and_label_are_exposed() {
        let node = StartNode::new("start-1", "Start");
        assert_eq!(node.id(), "start-1");
        assert_eq!(node.label(), "Start");
    }

    #[tokio::test]
    async fn execute_passes_through_non_empty_state_unchanged() {
        let node = StartNode::new("start-1", "Start");
        let state = AgentState::try_from(json!({"message": "hi"})).unwrap();
        let result = node.execute(state).await.unwrap();
        assert_eq!(result.get("/message"), Some(&json!("hi")));
    }

    #[tokio::test]
    async fn execute_errors_on_empty_state() {
        let node = StartNode::new("start-1", "Start");
        let state = AgentState::default();
        assert!(node.execute(state).await.is_err());
    }
}
