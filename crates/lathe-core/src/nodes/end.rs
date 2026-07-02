use crate::nodes::LatheNode;
use crate::state::AgentState;
use anyhow::bail;
use async_trait::async_trait;

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

    async fn execute(&self, agent_state: AgentState) -> anyhow::Result<AgentState> {
        if agent_state.is_empty() {
            bail!("Empty Agent State. Nothing to process")
        }
        agent_state.select(&self.out_pointers)
    }
}

impl EndNode {
    pub fn new(id: &str, label: &str, out_pointers: &[String]) -> Self {
        Self {
            id: id.to_owned(),
            label: label.to_owned(),
            out_pointers: out_pointers.to_owned(),
        }
    }

    pub fn out_pointers(&self) -> &Vec<String> {
        &self.out_pointers
    }
}
