//! Minimal `{{/pointer}}` templating over [`AgentState`], used to fill LLM system prompts with
//! values from prior pipeline steps.

use crate::state::AgentState;
use anyhow::{Result, bail};
use serde_json::Value;

/// Checks that every `{{/pointer}}` placeholder in `template` is well-formed (a `/`-prefixed
/// JSON Pointer, properly closed) and resolves against `agent_state`, without building the
/// resolved string.
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

    result.push_str(processing_template);

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn state() -> AgentState {
        AgentState::try_from(json!({"name": "world", "count": 3})).unwrap()
    }

    #[test]
    fn resolve_with_no_placeholders_returns_template_unchanged() {
        let result = resolve("hello there", &state()).unwrap();
        assert_eq!(result, "hello there");
    }

    #[test]
    fn resolve_substitutes_string_pointer() {
        let result = resolve("hello {{/name}}", &state()).unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn resolve_substitutes_multiple_pointers() {
        let result = resolve("{{/name}} has {{/count}}", &state()).unwrap();
        assert_eq!(result, "world has 3");
    }

    #[test]
    fn resolve_keeps_text_after_the_last_placeholder() {
        let result = resolve("hello {{/name}}!", &state()).unwrap();
        assert_eq!(result, "hello world!");
    }

    #[test]
    fn resolve_stringifies_non_string_values() {
        let result = resolve("count={{/count}}", &state()).unwrap();
        assert_eq!(result, "count=3");
    }

    #[test]
    fn resolve_trims_whitespace_inside_braces() {
        let result = resolve("hello {{ /name }}", &state()).unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn resolve_unclosed_placeholder_errors() {
        assert!(resolve("hello {{/name", &state()).is_err());
    }

    #[test]
    fn resolve_pointer_missing_leading_slash_errors() {
        assert!(resolve("hello {{name}}", &state()).is_err());
    }

    #[test]
    fn resolve_missing_pointer_errors() {
        assert!(resolve("hello {{/missing}}", &state()).is_err());
    }

    #[test]
    fn validate_accepts_well_formed_template() {
        assert!(validate("hello {{/name}}", &state()).is_ok());
    }

    #[test]
    fn validate_rejects_unclosed_placeholder() {
        assert!(validate("hello {{/name", &state()).is_err());
    }

    #[test]
    fn validate_rejects_pointer_without_leading_slash() {
        assert!(validate("hello {{name}}", &state()).is_err());
    }

    #[test]
    fn validate_rejects_missing_pointer() {
        assert!(validate("hello {{/missing}}", &state()).is_err());
    }
}
