use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
pub enum ResponseFormat {
    Json,
    Text,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ParamType {
    String,
    Integer,
    Boolean,
    Float,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HttpRequestToolParam {
    pub name: String,

    #[serde(rename = "type")]
    pub value_type: ParamType,

    #[serde(default)]
    pub required: bool,

    pub description: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HttpRequestToolDef {
    pub id: String,
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

    pub response_format: ResponseFormat,
    pub params: Vec<HttpRequestToolParam>,
}
