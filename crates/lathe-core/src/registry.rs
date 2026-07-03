use crate::node_defs::NodeKind;
use crate::nodes::LatheNode;
use crate::nodes::end::EndNode;
use crate::nodes::llm::LLMNode;
use crate::nodes::start::StartNode;
use crate::provider::LLMProviderConfigs;
use anyhow::Result;
use std::collections::HashMap;

pub type ExecutableNodes = HashMap<String, Box<dyn LatheNode>>;

pub fn inflate(
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
