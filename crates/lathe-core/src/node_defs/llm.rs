use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LLMProvider {
    OpenAI,
    LMStudio,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmNodeDef {
    pub id: String,
    pub label: String,
    pub provider: LLMProvider,
    pub model: String,
    pub system_prompt: String,
    pub base_url: Option<String>,
}
