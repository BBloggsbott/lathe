pub mod end;
pub mod llm;
pub mod start;

use crate::state::AgentState;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait LatheNode: Send + Sync {
    fn label(&self) -> &str;

    fn id(&self) -> &str;

    async fn execute(&self, agent_state: AgentState) -> Result<AgentState>;
}
