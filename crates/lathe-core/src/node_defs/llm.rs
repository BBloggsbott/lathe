use crate::provider::LLMProvider;
use serde::{Deserialize, Serialize};

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
