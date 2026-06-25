use crate::state::AgentState;
use anyhow::{Result, bail};
use serde_json::Value;

pub fn validate(template: &str, agent_state: &AgentState) -> Result<()> {
    let mut processing_template = template;

    while let Some(open) = processing_template.find("{{") {
        processing_template = &processing_template[open + 2..];

        let close = processing_template
            .find("}}")
            .ok_or_else(|| anyhow::anyhow!("Unclosed state pointer `{{` in template"))?;

        let pointer = processing_template[..close].trim();
        processing_template = &processing_template[close + 2..];

        if !pointer.starts_with("/") {
            bail!("template pointer must start with `/`: {}", pointer)
        }

        if !agent_state.exists(pointer) {
            bail!("pointer {} not in agent state", pointer)
        }
    }

    Ok(())
}

/// Fills in the values for a templated string from the agent state.
pub fn resolve(template: &str, agent_state: &AgentState) -> Result<String> {
    let mut result = String::with_capacity(template.len());
    let mut processing_template = template;

    while let Some(open) = processing_template.find("{{") {
        result.push_str(&processing_template[..open]);
        processing_template = &processing_template[open + 2..];

        let close = processing_template
            .find("}}")
            .ok_or_else(|| anyhow::anyhow!("Unclosed state pointer `{{` in template"))?;

        let pointer = processing_template[..close].trim();
        processing_template = &processing_template[close + 2..];

        if !pointer.starts_with("/") {
            bail!("template pointer must start with `/`: {}", pointer)
        }

        let resolved_pointer_value = agent_state
            .get(pointer)
            .ok_or_else(|| anyhow::anyhow!("pointer {} not in agent state", pointer))?;

        match resolved_pointer_value {
            Value::String(s) => result.push_str(s),
            other => result.push_str(&other.to_string()),
        }
    }

    Ok(result)
}
