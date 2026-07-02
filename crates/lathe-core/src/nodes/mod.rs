pub mod end;
mod llm;
pub mod start;

use crate::nodes::end::EndNode;
use crate::nodes::llm::LLMNode;
use crate::nodes::start::StartNode;
use crate::state::AgentState;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait LatheNode: Send + Sync {
    fn label(&self) -> &str;

    fn id(&self) -> &str;

    async fn execute(&self, agent_state: AgentState) -> Result<AgentState>;
}

#[derive(Debug)]
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
