//! Serializable node definitions used in a [`crate::graph::GraphDefinition`].
//!
//! These are the YAML-facing counterparts of the runtime nodes in [`crate::nodes`]; use
//! [`crate::registry::materialize`] to convert a slice of [`NodeKind`] into executable nodes.

pub mod llm;

use crate::node_defs::llm::LlmNodeDef;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const START_NODE_LABEL: &str = "lathe::nodes::start";
const END_NODE_LABEL: &str = "lathe::nodes::end";

/// Definition of the graph's entry point. Every graph has exactly one; it passes the initial
/// agent state through unchanged (see [`crate::nodes::start::StartNode`]).
#[derive(Debug, Serialize, Deserialize)]
pub struct StartNodeDef {
    pub id: String,
    pub label: String,
}

impl Default for StartNodeDef {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            label: START_NODE_LABEL.to_string(),
        }
    }
}

/// Definition of a graph exit point. `out_pointers` lists the agent state pointers to select
/// into the final output (see [`crate::nodes::end::EndNode`]). A graph is valid only if its
/// end nodes are exactly its leaf nodes (see [`crate::graph::LatheGraph::validate`]).
#[derive(Debug, Serialize, Deserialize)]
pub struct EndNodeDef {
    pub id: String,
    pub label: String,
    pub out_pointers: Vec<String>,
}

impl Default for EndNodeDef {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            label: END_NODE_LABEL.to_string(),
            out_pointers: vec![],
        }
    }
}

/// Union of all node definition types that can appear in a [`crate::graph::GraphDefinition`].
#[derive(Debug, Serialize, Deserialize)]
pub enum NodeKind {
    Start(StartNodeDef),
    End(EndNodeDef),
    LLMNode(LlmNodeDef),
}

impl NodeKind {
    /// The node's unique ID, regardless of its concrete kind.
    pub fn id(&self) -> &str {
        match &self {
            NodeKind::Start(node) => node.id.as_str(),
            NodeKind::End(node) => node.id.as_str(),
            NodeKind::LLMNode(node) => node.id.as_str(),
        }
    }
}
