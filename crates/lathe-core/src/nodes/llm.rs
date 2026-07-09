use crate::node_defs::llm::LlmNodeDef;
use crate::nodes::LatheNode;
use crate::provider::{LLMProvider, LLMProviderConfig, LLMProviderConfigs};
use crate::state::AgentState;
use crate::template;
use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::FutureExt;
use futures::future::BoxFuture;
use rig_core::completion::{Message, Prompt};
use rig_core::prelude::CompletionClient;
use rig_core::providers;
use serde_json::Value;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use uuid::Uuid;

const LLM_NODE_DEFAULT_LABEL: &str = "LLMNode";
const LLM_NODE_DEFAULT_SYSTEM_PROMPT: &str = "You are a helpful assistant";

/// A closure that prompts an LLM with `(system_prompt, user_message)` and resolves to the
/// model's text response. Built once per node by [`build_llm_caller`] so the underlying client
/// and agent are reused across calls.
type LLMCaller = Box<dyn Fn(String, String) -> BoxFuture<'static, Result<String>> + Send + Sync>;

/// A node that calls out to an LLM. Reads its user message from `input_key` in the agent
/// state, resolves the system prompt's `{{/pointer}}` templates against that same state, and
/// writes the model's response to `output_key`. Built from an [`LlmNodeDef`] via
/// [`Self::from_node_def`].
pub struct LLMNode {
    id: String,
    label: String,
    provider: LLMProvider, // Currently
    model: String,
    system_prompt: String,
    input_key: String,
    output_key: String,
    caller: LLMCaller,
}

impl LLMNode {
    /// Builds an [`LLMNode`] from its serializable definition, resolving `def.provider_config_id`
    /// against `provider_configs` (falling back to [`LLMProviderConfig::default`] if not found)
    /// and eagerly constructing the underlying LLM client/agent. Missing `id`/`label`/
    /// `system_prompt` fall back to generated/default values. Panics if the client cannot be
    /// built (e.g. missing API key).
    pub fn from_node_def(def: &LlmNodeDef, provider_configs: &LLMProviderConfigs) -> Self {
        let provider_config = provider_configs.get(&def.provider_config_id);
        Self {
            id: if !&def.id.is_empty() {
                def.id.to_owned()
            } else {
                Uuid::new_v4().to_string()
            },
            label: if !&def.label.is_empty() {
                def.label.to_owned()
            } else {
                LLM_NODE_DEFAULT_LABEL.to_string()
            },
            provider: def.provider.clone(),
            model: def.model.clone(),
            system_prompt: if !&def.system_prompt.is_empty() {
                def.system_prompt.to_owned()
            } else {
                LLM_NODE_DEFAULT_SYSTEM_PROMPT.to_string()
            },
            input_key: def.input_key.to_owned(),
            output_key: def.output_key.to_owned(),
            caller: build_llm_caller(
                &def.provider,
                &def.model,
                &def.system_prompt,
                provider_config,
            )
            .expect("Unable to build llm caller"),
        }
    }
}

impl Debug for LLMNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LLMNode")
            .field("id", &self.id)
            .field("label", &self.label)
            .field("provider", &self.provider)
            .field("model", &self.model)
            .field(
                "system_prompt",
                &if self.system_prompt.len() > 10 {
                    format!("{}...", &self.system_prompt[..7])
                } else {
                    self.system_prompt.clone()
                },
            )
            .field("caller", &"<async function>")
            .finish()
    }
}

#[async_trait]
impl LatheNode for LLMNode {
    fn label(&self) -> &str {
        self.label.as_str()
    }

    fn id(&self) -> &str {
        self.id.as_str()
    }

    /// Resolves the system prompt template against `agent_state`, prompts the LLM with the
    /// value at `input_key`, and writes the response to `output_key`.
    // todo: Not implemented. Doing this just to make clippy happy
    async fn execute(&self, mut agent_state: AgentState) -> Result<AgentState> {
        let user_message = agent_state
            .get(&self.input_key)
            .context("Cannot find input key in agent state")?
            .to_string();
        let response = (self.caller)(
            template::resolve(self.system_prompt.as_str(), &agent_state)?,
            user_message,
        )
        .await?;
        agent_state.set(&self.output_key, Value::String(response))?;
        Ok(agent_state)
    }
}

/// Dispatches to the provider-specific closure builder for `provider`, falling back to
/// [`LLMProviderConfig::default`] when no config was resolved for the node.
fn build_llm_caller(
    provider: &LLMProvider,
    model: &String,
    system_prompt: &str,
    provider_config: Option<&LLMProviderConfig>,
) -> Result<LLMCaller> {
    let provider_config = match provider_config {
        None => &LLMProviderConfig::default(provider),
        Some(config) => config,
    };
    match provider {
        LLMProvider::OpenAI => build_openai_closure(
            provider_config.base_url.as_ref(),
            provider_config.api_key.as_ref(),
            model,
            system_prompt,
        ),
        LLMProvider::LMStudio => {
            build_lmstudio_closure(provider_config.base_url.as_ref(), model, system_prompt)
        }
    }
}

/// Builds an [`LLMCaller`] backed by the OpenAI-compatible `rig_core` client. Reads the API key
/// from `openai_api_key` or, if absent, the `OPENAI_API_KEY` env var; uses `base_url` if given.
// todo: Move default handling to Provider config
fn build_openai_closure(
    base_url: Option<&String>,
    openai_api_key: Option<&String>,
    model: &String,
    system_prompt: &str,
) -> Result<LLMCaller> {
    let api_key = match openai_api_key {
        None => std::env::var("OPENAI_API_KEY")
            .context("Could not find openai API key as param or in env")?,
        Some(key) => key.clone(),
    };

    let client = match base_url {
        None => providers::openai::Client::new(api_key)?,
        Some(url) => providers::openai::Client::builder()
            .base_url(url.as_str())
            .api_key(api_key)
            .build()?,
    };

    let agent = Arc::new(client.agent(model).preamble(system_prompt).build());

    Ok(Box::new(
        move |system_prompt: String, user_message: String| {
            let agent = Arc::clone(&agent);
            async move {
                agent
                    .prompt(user_message.as_str())
                    .with_history(vec![Message::system(system_prompt)])
                    .await
                    .map_err(anyhow::Error::from)
            }
            .boxed()
        },
    ))
}

/// Builds an [`LLMCaller`] backed by a local LM Studio server via the OpenAI-compatible
/// `rig_core` client. Defaults `base_url` to `http://localhost:1234/v1` and uses a placeholder
/// API key, since LM Studio does not require authentication.
// todo: Move default handling to Provider config
fn build_lmstudio_closure(
    base_url: Option<&String>,
    model: &String,
    system_prompt: &str,
) -> Result<LLMCaller> {
    let api_key = "lmstudio";

    let base_url = match base_url {
        None => "http://localhost:1234/v1",
        Some(url) => url,
    };

    let client = providers::openai::Client::builder()
        .base_url(base_url)
        .api_key(api_key)
        .build()?;

    let agent = Arc::new(client.agent(model).preamble(system_prompt).build());

    Ok(Box::new(
        move |system_prompt: String, user_message: String| {
            let agent = Arc::clone(&agent);
            async move {
                agent
                    .prompt(user_message.as_str())
                    .with_history(vec![Message::system(system_prompt)])
                    .await
                    .map_err(anyhow::Error::from)
            }
            .boxed()
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::LLMProviderConfigs;

    fn lmstudio_def() -> LlmNodeDef {
        LlmNodeDef {
            id: "".to_string(),
            label: "".to_string(),
            provider: LLMProvider::LMStudio,
            model: "test-model".to_string(),
            system_prompt: "".to_string(),
            input_key: "/message".to_string(),
            output_key: "/response".to_string(),
            provider_config_id: "unknown-config".to_string(),
        }
    }

    #[test]
    fn from_node_def_fills_in_defaults_for_blank_fields() {
        let def = lmstudio_def();
        let node = LLMNode::from_node_def(&def, &LLMProviderConfigs::new());

        assert!(Uuid::parse_str(&node.id).is_ok());
        assert_eq!(node.label, LLM_NODE_DEFAULT_LABEL);
        assert_eq!(node.system_prompt, LLM_NODE_DEFAULT_SYSTEM_PROMPT);
        assert_eq!(node.input_key, "/message");
        assert_eq!(node.output_key, "/response");
    }

    #[test]
    fn from_node_def_preserves_explicit_fields() {
        let mut def = lmstudio_def();
        def.id = "llm-1".to_string();
        def.label = "My LLM".to_string();
        def.system_prompt = "Be terse".to_string();

        let node = LLMNode::from_node_def(&def, &LLMProviderConfigs::new());

        assert_eq!(node.id, "llm-1");
        assert_eq!(node.label, "My LLM");
        assert_eq!(node.system_prompt, "Be terse");
    }

    #[test]
    fn id_and_label_are_exposed() {
        let mut def = lmstudio_def();
        def.id = "llm-1".to_string();
        def.label = "My LLM".to_string();
        let node = LLMNode::from_node_def(&def, &LLMProviderConfigs::new());

        assert_eq!(node.id(), "llm-1");
        assert_eq!(node.label(), "My LLM");
    }

    #[test]
    fn debug_impl_truncates_long_system_prompt() {
        let mut def = lmstudio_def();
        def.system_prompt = "this is a fairly long system prompt".to_string();
        let node = LLMNode::from_node_def(&def, &LLMProviderConfigs::new());

        let debug_str = format!("{node:?}");
        assert!(debug_str.contains("this is..."));
        assert!(!debug_str.contains("fairly long"));
    }

    #[test]
    fn debug_impl_shows_short_system_prompt_in_full() {
        let mut def = lmstudio_def();
        def.system_prompt = "short".to_string();
        let node = LLMNode::from_node_def(&def, &LLMProviderConfigs::new());

        let debug_str = format!("{node:?}");
        assert!(debug_str.contains("short"));
    }
}
