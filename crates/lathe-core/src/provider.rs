use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// The LLM backend an [`crate::node_defs::llm::LlmNodeDef`] talks to. Also usable as a CLI
/// value via [`ValueEnum`].
#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
pub enum LLMProvider {
    OpenAI,
    LMStudio,
}

/// Connection details for a single named provider, referenced by
/// [`crate::node_defs::llm::LlmNodeDef::provider_config_id`] and resolved via
/// [`LLMProviderConfigs`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMProviderConfig {
    pub id: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub provider: LLMProvider,
}

impl LLMProviderConfig {
    /// Builds a fresh, unconfigured config (random ID, no base URL/API key override) for the
    /// given provider, used when a node's `provider_config_id` doesn't resolve to a stored
    /// config.
    pub fn default(provider: &LLMProvider) -> Self {
        match provider {
            LLMProvider::OpenAI => Self {
                id: Uuid::new_v4().to_string(),
                base_url: None,
                api_key: None,
                provider: provider.clone(),
            },
            LLMProvider::LMStudio => Self {
                id: Uuid::new_v4().to_string(),
                base_url: None,
                api_key: None,
                provider: provider.clone(),
            },
        }
    }
}

/// Provider configs for a graph, keyed by [`LLMProviderConfig::id`] (matching
/// [`crate::node_defs::llm::LlmNodeDef::provider_config_id`]).
pub type LLMProviderConfigs = HashMap<String, LLMProviderConfig>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_openai_config_has_no_overrides() {
        let config = LLMProviderConfig::default(&LLMProvider::OpenAI);
        assert!(Uuid::parse_str(&config.id).is_ok());
        assert!(config.base_url.is_none());
        assert!(config.api_key.is_none());
        assert!(matches!(config.provider, LLMProvider::OpenAI));
    }

    #[test]
    fn default_lmstudio_config_has_no_overrides() {
        let config = LLMProviderConfig::default(&LLMProvider::LMStudio);
        assert!(Uuid::parse_str(&config.id).is_ok());
        assert!(config.base_url.is_none());
        assert!(config.api_key.is_none());
        assert!(matches!(config.provider, LLMProvider::LMStudio));
    }

    #[test]
    fn default_configs_have_distinct_ids() {
        let a = LLMProviderConfig::default(&LLMProvider::OpenAI);
        let b = LLMProviderConfig::default(&LLMProvider::OpenAI);
        assert_ne!(a.id, b.id);
    }
}
