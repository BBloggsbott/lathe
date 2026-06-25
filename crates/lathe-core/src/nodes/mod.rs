use crate::state::AgentState;
use anyhow::{Result, bail};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait LatheNode: Send + Sync {
    fn label(&self) -> &str;

    fn id(&self) -> &str;

    async fn execute(&self, agent_state: AgentState) -> Result<AgentState>;
}

const START_NODE_LABEL: &str = "lathe::nodes::start";

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

    async fn execute(&self, agent_state: AgentState) -> Result<AgentState> {
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

const END_NODE_LABEL: &str = "lathe::nodes::end";

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

    async fn execute(&self, agent_state: AgentState) -> Result<AgentState> {
        if agent_state.is_empty() {
            bail!("Empty Agent State. Nothing to process")
        }
        agent_state.select(&self.out_pointers)
    }
}

impl EndNode {
    pub fn new(out_pointers: Vec<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            label: END_NODE_LABEL.to_string(),
            out_pointers,
        }
    }

    pub fn set_out_pointers(&mut self, out_pointers: Vec<String>) {
        self.out_pointers = out_pointers;
    }

    pub fn out_pointers(&self) -> &Vec<String> {
        &self.out_pointers
    }
}
