use crate::tool_defs::http_request::HttpRequestToolDef;
use serde::{Deserialize, Serialize};

pub mod http_request;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "kind")]
pub enum ToolKind {
    HttpRequest(HttpRequestToolDef),
}
