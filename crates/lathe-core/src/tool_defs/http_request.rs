use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

fn default_timeout_ms() -> u64 {
    5000
}

#[derive(Serialize, Deserialize, Debug)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ResponseType {
    Json,
    Text,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ParamType {
    Text,
    Integer,
    Boolean,
    Float,
}

impl Display for ParamType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParamType::Text => write!(f, "String"),
            ParamType::Integer => write!(f, "Integer"),
            ParamType::Boolean => write!(f, "Boolean"),
            ParamType::Float => write!(f, "Float"),
        }
    }
}

impl ParamType {
    pub fn matches(&self, value: &serde_json::Value) -> bool {
        match self {
            ParamType::Text => value.is_string(),
            ParamType::Integer => value.is_i64(),
            ParamType::Boolean => value.is_boolean(),
            ParamType::Float => value.is_f64(),
        }
    }
}

// todo: Currently all Tool params are mandatory. Introduce optional params by
//  building the query params and request body during runtime to enable optional params.
// todo: Add a param source field that will allow resolving params from agent states or from
//  the llm.
#[derive(Serialize, Deserialize, Debug)]
pub struct HttpRequestToolParam {
    #[serde(rename = "type")]
    pub value_type: ParamType,

    pub description: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HttpRequestToolDef {
    pub name: String,
    pub description: String,
    pub method: HttpMethod,
    pub url: String,

    #[serde(default)]
    pub headers: HashMap<String, String>,

    #[serde(default)]
    pub body: Option<String>,

    #[serde(default = "default_timeout_ms")]
    pub timeout: u64,

    pub response_type: ResponseType,

    #[serde(default)]
    pub params: HashMap<String, HttpRequestToolParam>,
}
