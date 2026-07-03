use crate::nodes::LatheNode;
use crate::state::AgentState;
use anyhow::bail;
use async_trait::async_trait;

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
    pub fn new(id: &str, label: &str) -> Self {
        Self {
            id: id.to_owned(),
            label: label.to_owned(),
        }
    }
}
