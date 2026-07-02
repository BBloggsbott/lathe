use crate::node_defs::llm::{LLMProvider, LlmNodeDef};
use crate::nodes::LatheNode;
use crate::state::AgentState;
use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::FutureExt;
use futures::future::BoxFuture;
use rig_core::completion::Prompt;
use rig_core::prelude::CompletionClient;
use rig_core::providers;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use uuid::Uuid;

const LLM_NODE_DEFAULT_LABEL: &str = "LLMNode";
const LLM_NODE_DEFAULT_SYSTEM_PROMPT: &str = "You are a helpful assistant";

type LLMCaller = Box<dyn Fn(String) -> BoxFuture<'static, Result<String>> + Send + Sync>;

pub struct LLMNode {
    id: String,
    label: String,
    provider: LLMProvider, // Currently
    model: String,
    system_prompt: String,
    caller: LLMCaller,
}

impl LLMNode {
    pub fn new(
        id: &str,
        label: &str,
        provider: &LLMProvider,
        base_url: Option<&String>,
        model: &String,
        system_prompt: &str,
    ) -> Self {
        Self {
            id: if !id.is_empty() {
                id.to_owned()
            } else {
                Uuid::new_v4().to_string()
            },
            label: if !label.is_empty() {
                label.to_owned()
            } else {
                LLM_NODE_DEFAULT_LABEL.to_string()
            },
            provider: provider.clone(),
            model: model.clone(),
            system_prompt: if !system_prompt.is_empty() {
                system_prompt.to_owned()
            } else {
                LLM_NODE_DEFAULT_SYSTEM_PROMPT.to_string()
            },
            caller: build_llm_caller(provider, base_url, model, system_prompt)
                .expect("Unable to build llm caller"),
        }
    }

    pub fn from_node_def(def: &LlmNodeDef) -> Self {
        Self::new(
            &def.id,
            &def.label,
            &def.provider,
            def.base_url.as_ref(),
            &def.model,
            &def.system_prompt,
        )
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

    // todo: Not implemented. Doing this just to make clippy happy
    async fn execute(&self, agent_state: AgentState) -> Result<AgentState> {
        Ok(agent_state)
    }
}

fn build_llm_caller(
    provider: &LLMProvider,
    base_url: Option<&String>,
    model: &String,
    system_prompt: &str,
) -> Result<LLMCaller> {
    match provider {
        LLMProvider::OpenAI => build_openai_closure(base_url, None, model, system_prompt),
        LLMProvider::LMStudio => build_lmstudio_closure(base_url, model, system_prompt),
    }
}

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

    Ok(Box::new(move |prompt: String| {
        let agent = Arc::clone(&agent);
        async move {
            agent
                .prompt(prompt.as_str())
                .await
                .map_err(anyhow::Error::from)
        }
        .boxed()
    }))
}

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

    Ok(Box::new(move |prompt: String| {
        let agent = Arc::clone(&agent);
        async move {
            agent
                .prompt(prompt.as_str())
                .await
                .map_err(anyhow::Error::from)
        }
        .boxed()
    }))
}
