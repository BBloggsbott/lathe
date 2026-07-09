//! Converts serializable [`NodeKind`] definitions into runtime [`LatheNode`]s that an
//! [`crate::executor::Executor`] can run.

use crate::node_defs::NodeKind;
use crate::nodes::LatheNode;
use crate::nodes::end::EndNode;
use crate::nodes::llm::LLMNode;
use crate::nodes::start::StartNode;
use crate::provider::LLMProviderConfigs;
use anyhow::Result;
use std::collections::HashMap;

/// Runtime nodes keyed by node ID, ready to be handed to [`crate::executor::Executor::new`].
pub type ExecutableNodes = HashMap<String, Box<dyn LatheNode>>;

/// Builds an [`ExecutableNodes`] map by instantiating the concrete runtime node for each
/// [`NodeKind`] in `nodes`, resolving any LLM nodes' providers against `provider_configs`.
pub fn materialize(
    nodes: &[NodeKind],
    provider_configs: &LLMProviderConfigs,
) -> Result<ExecutableNodes> {
    let mut executable_nodes: ExecutableNodes = HashMap::new();

    for kind in nodes {
        match kind {
            NodeKind::Start(def) => {
                executable_nodes.insert(
                    def.id.clone(),
                    Box::new(StartNode::new(&def.id, &def.label)),
                );
            }
            NodeKind::End(def) => {
                executable_nodes.insert(
                    def.id.clone(),
                    Box::new(EndNode::new(&def.id, &def.label, &def.out_pointers)),
                );
            }
            NodeKind::LLMNode(def) => {
                executable_nodes.insert(
                    def.id.clone(),
                    Box::new(LLMNode::from_node_def(def, provider_configs)),
                );
            }
        }
    }

    Ok(executable_nodes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node_defs::llm::LlmNodeDef;
    use crate::node_defs::{EndNodeDef, StartNodeDef};
    use crate::provider::LLMProvider;

    #[test]
    fn materialize_builds_a_node_for_each_definition() {
        let nodes = vec![
            NodeKind::Start(StartNodeDef {
                id: "start".to_string(),
                ..Default::default()
            }),
            NodeKind::LLMNode(LlmNodeDef {
                id: "llm".to_string(),
                label: "LLM".to_string(),
                provider: LLMProvider::LMStudio,
                model: "test-model".to_string(),
                system_prompt: "".to_string(),
                input_key: "/message".to_string(),
                output_key: "/response".to_string(),
                provider_config_id: "".to_string(),
            }),
            NodeKind::End(EndNodeDef {
                id: "end".to_string(),
                ..Default::default()
            }),
        ];

        let executable = materialize(&nodes, &LLMProviderConfigs::new()).unwrap();

        assert_eq!(executable.len(), 3);
        assert_eq!(executable.get("start").unwrap().id(), "start");
        assert_eq!(executable.get("llm").unwrap().id(), "llm");
        assert_eq!(executable.get("end").unwrap().id(), "end");
    }

    #[test]
    fn materialize_of_empty_nodes_returns_empty_map() {
        let executable = materialize(&[], &LLMProviderConfigs::new()).unwrap();
        assert!(executable.is_empty());
    }
}
