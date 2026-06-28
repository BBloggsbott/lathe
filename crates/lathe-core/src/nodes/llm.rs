use crate::nodes::LatheNode;
use crate::state::AgentState;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const LLM_NODE_DEFAULT_LABEL: &str = "LLMNode";
const LLM_NODE_DEFAULT_SYSTEM_PROMPT: &str = " You are a helpful assistant";

#[derive(Debug, Serialize, Deserialize)]
pub struct LLMNode {
    id: String,
    label: String,
    provider: String,
    model: String,
    system_prompt: String,
}

#[async_trait]
impl LatheNode for LLMNode {
    fn label(&self) -> &str {
        self.label.as_str()
    }

    fn id(&self) -> &str {
        self.id.as_str()
    }

    // todo: Not implemented. Doing this just to make clippy happy
    async fn execute(&self, agent_state: AgentState) -> Result<AgentState> {
        Ok(agent_state)
    }
}

impl LLMNode {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            label: LLM_NODE_DEFAULT_LABEL.to_string(),
            provider: String::new(),
            model: String::new(),
            system_prompt: LLM_NODE_DEFAULT_SYSTEM_PROMPT.to_string(),
        }
    }

    pub fn set_label(&mut self, label: &str) {
        self.label = label.to_string()
    }

    pub fn set_provider(&mut self, provider: &str) {
        self.provider = provider.to_string()
    }

    pub fn set_model(&mut self, model: &str) {
        self.model = model.to_string()
    }

    pub fn set_system_prompt(&mut self, system_prompt: &str) {
        self.system_prompt = system_prompt.to_string()
    }
}
