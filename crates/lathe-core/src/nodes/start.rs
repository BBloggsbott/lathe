use crate::nodes::LatheNode;
use crate::state::AgentState;
use anyhow::bail;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const START_NODE_LABEL: &str = "lathe::nodes::start";

#[derive(Debug, Serialize, Deserialize)]
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
        if agent_state.is_empty() {
            bail!("Empty Agent State. Nothing to process")
        }
        Ok(agent_state)
    }
}

impl Default for StartNode {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            label: START_NODE_LABEL.to_string(),
        }
    }
}
