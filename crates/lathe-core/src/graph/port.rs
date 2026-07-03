use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Port {
    pub node_id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Connection {
    pub from: Port,
    pub to: Port,
}
