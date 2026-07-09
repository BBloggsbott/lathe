use crate::provider::LLMProvider;
use serde::{Deserialize, Serialize};

/// Serializable definition of an LLM-backed node: which provider/model to call, the system
/// prompt (which may reference agent state via `{{/pointer}}` templates), and which agent
/// state keys to read the user message from and write the response to. Turned into a runnable
/// [`crate::nodes::llm::LLMNode`] via [`crate::registry::materialize`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmNodeDef {
    pub id: String,
    pub label: String,
    pub provider: LLMProvider,
    pub model: String,
    pub system_prompt: String,
    pub input_key: String,
    pub output_key: String,
    pub provider_config_id: String,
}
