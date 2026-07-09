use serde::{Deserialize, Serialize};

/// One endpoint of a [`Connection`]: the node it belongs to and a human-readable port name.
#[derive(Debug, Serialize, Deserialize)]
pub struct Port {
    pub node_id: String,
    pub name: String,
}

/// A directed edge between two nodes in a [`crate::graph::GraphDefinition`], executed in
/// topological order from `from` to `to`.
#[derive(Debug, Serialize, Deserialize)]
pub struct Connection {
    pub from: Port,
    pub to: Port,
}
