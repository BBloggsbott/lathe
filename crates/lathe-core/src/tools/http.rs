use crate::tool_defs::http_request::{
    HttpMethod, HttpRequestToolDef, HttpRequestToolParam, ResponseType,
};
use anyhow::{Context, Result};
use reqwest::Method;
use std::collections::HashMap;
use std::time::Duration;

pub struct ToolResult {
    pub status: u16,
    pub body: serde_json::Value,
}

const PARAM_IDENTIFIER_OPEN: &str = "{{";
const PARAM_IDENTIFIER_CLOSE: &str = "}}";

pub struct ResolvedHttpTool {
    id: String,
    name: String,
    description: String,
    method: Method,
    url: String,
    headers: HashMap<String, String>,
    body: Option<String>,
    timeout: u64,
    response_type: ResponseType,
    params: HashMap<String, HttpRequestToolParam>,
    client: reqwest::Client,
}

impl ResolvedHttpTool {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn from_def(id: String, def: HttpRequestToolDef, client: reqwest::Client) -> Result<Self> {
        let mut headers: HashMap<String, String> = HashMap::new();

        for (key, value) in def.headers.iter() {
            let resolved_value = resolve_header_values_from_env(&id, value)?;
            headers.insert(key.to_string(), resolved_value);
        }

        let method = match def.method {
            HttpMethod::GET => Method::GET,
            HttpMethod::POST => Method::POST,
            HttpMethod::PUT => Method::PUT,
            HttpMethod::PATCH => Method::PATCH,
            HttpMethod::DELETE => Method::DELETE,
        };

        Ok(ResolvedHttpTool {
            id: id.clone(),
            name: def.name,
            description: def.description,
            method,
            url: def.url,
            headers,
            body: def.body,
            timeout: def.timeout,
            response_type: def.response_type,
            params: def.params,
            client,
        })
    }

    pub async fn call(&self, args: &HashMap<String, serde_json::Value>) -> Result<ToolResult> {
        self.validate_args(args)?;
        let url = self.resolve_string(self.url.as_str(), args)?;

        let mut request_builder = self
            .client
            .request(self.method.clone(), url)
            .timeout(Duration::from_millis(self.timeout));

        for (key, value) in self.headers.iter() {
            let resolved_value = self.resolve_string(value.as_str(), args)?;
            request_builder = request_builder.header(key.clone(), resolved_value);
        }

        request_builder = match &self.body {
            None => request_builder,
            Some(value) => {
                let resolved_body = self.resolve_string(value, args)?;
                request_builder.body(resolved_body)
            }
        };

        let request = request_builder.build()?;

        let response = self.client.execute(request).await?;

        let tool_response_status = response.status().as_u16();

        let response_body = response.text().await?;
        let tool_response_body: serde_json::Value = match self.response_type {
            ResponseType::Json => serde_json::from_str(response_body.as_str())?,
            ResponseType::Text => serde_json::Value::String(response_body),
        };

        Ok(ToolResult {
            status: tool_response_status,
            body: tool_response_body,
        })
    }

    fn validate_args(&self, args: &HashMap<String, serde_json::Value>) -> Result<()> {
        for (param_name, param_def) in self.params.iter() {
            anyhow::ensure!(
                args.contains_key(param_name),
                "Cannot find argument for param {} in tool {}",
                param_name,
                self.name
            );

            let arg_value = args.get(param_name).unwrap();

            anyhow::ensure!(
                param_def.value_type.matches(arg_value),
                "Invalid argument type for param {} in tool {}. Expected {}",
                param_name,
                self.name,
                param_def.value_type
            );
        }

        Ok(())
    }

    fn resolve_string(
        &self,
        string: &str,
        args: &HashMap<String, serde_json::Value>,
    ) -> Result<String> {
        let mut resolved_string = String::with_capacity(string.len());
        let mut processing_string = string;

        while let Some(open) = processing_string.find(PARAM_IDENTIFIER_OPEN) {
            resolved_string.push_str(&processing_string[..open]);

            processing_string = &processing_string[open + PARAM_IDENTIFIER_OPEN.len()..];

            let close = processing_string.find(PARAM_IDENTIFIER_CLOSE).ok_or_else(|| {
                anyhow::anyhow!("Unclosed param reference while resolving string `{}` in context of tool {}", string, self.id)
            })?;

            let param_name = &processing_string[..close];

            processing_string = &processing_string[close + PARAM_IDENTIFIER_CLOSE.len()..];

            if !self.params.contains_key(param_name) {
                resolved_string.push_str(
                    format!("{PARAM_IDENTIFIER_OPEN}{param_name}{PARAM_IDENTIFIER_CLOSE}").as_str(),
                )
            } else {
                let param_value = args.get(param_name).unwrap();
                let param_value_string = param_value.to_string();
                resolved_string.push_str(param_value_string.as_str())
            }
        }

        resolved_string.push_str(processing_string);

        Ok(resolved_string)
    }
}

fn resolve_header_values_from_env(tool_id: &str, value: &str) -> Result<String> {
    let mut resolved_value = String::with_capacity(value.len());
    let mut processing_value = value;

    while let Some(open) = processing_value.find("${") {
        resolved_value.push_str(&processing_value[..open]);

        processing_value = &processing_value[open + 2..];

        let close = processing_value.find("}").ok_or_else(|| {
            anyhow::anyhow!("Unclosed env variable reference in {tool_id}'s headers")
        })?;

        let env_var_name = &processing_value[..close];

        processing_value = &processing_value[close + 1..];

        let resolved_env_value =
            std::env::var(env_var_name).context(format!("Could not find {env_var_name} in env"))?;

        resolved_value.push_str(resolved_env_value.as_str());
    }

    resolved_value.push_str(processing_value);

    Ok(resolved_value)
}
