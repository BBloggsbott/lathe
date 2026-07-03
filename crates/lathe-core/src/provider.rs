use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LLMProvider {
    OpenAI,
    LMStudio,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMProviderConfig {
    pub id: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
}

impl LLMProviderConfig {
    pub fn default(provider: &LLMProvider) -> Self {
        match provider {
            LLMProvider::OpenAI => Self {
                id: Uuid::new_v4().to_string(),
                base_url: None,
                api_key: None,
            },
            LLMProvider::LMStudio => Self {
                id: Uuid::new_v4().to_string(),
                base_url: None,
                api_key: None,
            },
        }
    }
}

pub type LLMProviderConfigs = HashMap<String, LLMProviderConfig>;
