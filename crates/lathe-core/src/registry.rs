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
