pub mod llm;

use crate::node_defs::llm::LlmNodeDef;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const START_NODE_LABEL: &str = "lathe::nodes::start";
const END_NODE_LABEL: &str = "lathe::nodes::end";

#[derive(Debug, Serialize, Deserialize)]
pub struct StartNodeDef {
    pub id: String,
    pub label: String,
}

impl Default for StartNodeDef {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            label: START_NODE_LABEL.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EndNodeDef {
    pub id: String,
    pub label: String,
}

impl Default for EndNodeDef {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            label: END_NODE_LABEL.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NodeKind {
    Start(StartNodeDef),
    End(EndNodeDef),
    LLMNode(LlmNodeDef),
}

impl NodeKind {
    pub fn id(&self) -> &str {
        match &self {
            NodeKind::Start(node) => node.id.as_str(),
            NodeKind::End(node) => node.id.as_str(),
            NodeKind::LLMNode(node) => node.id.as_str(),
        }
    }
}
