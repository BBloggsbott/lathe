mod llm;
pub mod start;

use crate::nodes::llm::LLMNode;
use crate::nodes::start::StartNode;
use crate::state::AgentState;
use anyhow::{Result, bail};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[async_trait]
pub trait LatheNode: Send + Sync {
    fn label(&self) -> &str;

    fn id(&self) -> &str;

    async fn execute(&self, agent_state: AgentState) -> Result<AgentState>;
}

const END_NODE_LABEL: &str = "lathe::nodes::end";

#[derive(Debug, Serialize, Deserialize)]
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

impl Default for EndNode {
    fn default() -> Self {
        Self::new()
    }
}

impl EndNode {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            label: END_NODE_LABEL.to_string(),
            out_pointers: vec![],
        }
    }

    pub fn set_out_pointers(&mut self, out_pointers: Vec<String>) {
        self.out_pointers = out_pointers;
    }

    pub fn out_pointers(&self) -> &Vec<String> {
        &self.out_pointers
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NodeKind {
    Start(StartNode),
    End(EndNode),
    LLMNode(LLMNode),
}

impl NodeKind {
    pub fn id(&self) -> &str {
        match &self {
            NodeKind::Start(node) => node.id(),
            NodeKind::End(node) => node.id(),
            NodeKind::LLMNode(node) => node.id(),
        }
    }
}
