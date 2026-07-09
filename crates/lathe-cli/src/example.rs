use anyhow::Result;
use clap::ValueEnum;
use lathe_core::graph::port::{Connection, Port};
use lathe_core::graph::{GraphDefinition, GraphVersion, LatheGraph};
use lathe_core::node_defs::llm::LlmNodeDef;
use lathe_core::node_defs::{EndNodeDef, NodeKind, StartNodeDef};
use lathe_core::provider::{LLMProvider, LLMProviderConfig, LLMProviderConfigs};
use lathe_core::yaml;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Clone, ValueEnum)]
pub enum ExampleType {
    Simple,
    Explainer,
    None,
}

pub fn create_simple_agent(provider: LLMProvider, model: String) -> Result<()> {
    let start_node_def = StartNodeDef {
        id: "start-node".to_string(),
        ..Default::default()
    };
    let end_node_def = EndNodeDef {
        id: "end-node".to_string(),
        out_pointers: vec!["/output_message".to_string()],
        ..Default::default()
    };

    let mut provider_config = LLMProviderConfig::default(&provider);
    provider_config.id = "my-model".to_string();
    // todo: Using my local model fro dev-testing. Update to a generic example.

    let llm_node_def = LlmNodeDef {
        id: "llm-node".to_string(),
        label: "Simple Assistant LLM Node".to_string(),
        provider: LLMProvider::LMStudio,
        model,
        system_prompt: "You are a helpful assistant".to_string(),
        input_key: "/message".to_string(),
        output_key: "/output_message".to_string(),
        provider_config_id: provider_config.id.clone(),
    };

    let connect_start_llm = Connection {
        from: Port {
            node_id: start_node_def.id.clone(),
            name: format!("to {}", llm_node_def.label),
        },
        to: Port {
            node_id: llm_node_def.id.clone(),
            name: format!("from {}", start_node_def.label),
        },
    };

    let connect_llm_end = Connection {
        from: Port {
            node_id: llm_node_def.id.clone(),
            name: format!("to {}", end_node_def.label),
        },
        to: Port {
            node_id: end_node_def.id.clone(),
            name: format!("from {}", llm_node_def.label),
        },
    };

    let mut provider_configs = LLMProviderConfigs::new();
    provider_configs.insert(provider_config.id.clone(), provider_config);

    let graph_definition = GraphDefinition {
        graph_version: GraphVersion::V1,
        name: "Example Lathe Graph".to_string(),
        nodes: vec![
            NodeKind::Start(start_node_def),
            NodeKind::LLMNode(llm_node_def),
            NodeKind::End(end_node_def),
        ],
        connections: vec![connect_start_llm, connect_llm_end],
        provider_configs,
    };

    let lathe_graph = LatheGraph::from_def(graph_definition, true)?;
    let mut out_path = PathBuf::from_str(".")?;
    out_path.push("examples");
    out_path.push("simple_agent.yaml");
    tracing::info!("Writing to {}", out_path.to_str().unwrap());
    yaml::save(&lathe_graph, &out_path)
    // Ok(())
}

pub fn create_explainer_agent(provider: LLMProvider, model: String) -> Result<()> {
    let start_node_def = StartNodeDef {
        id: "start-node".to_string(),
        ..Default::default()
    };
    let end_node_def = EndNodeDef {
        id: "end-node".to_string(),
        out_pointers: vec![
            "/explanation".to_string(),
            "/summary".to_string(),
            "/title".to_string(),
        ],
        ..Default::default()
    };

    let mut provider_config = LLMProviderConfig::default(&provider);
    provider_config.id = "my-model".to_string();
    // todo: Using my local model fro dev-testing. Update to a generic example.

    let llm_explainer_node_def = LlmNodeDef {
        id: "llm-explainer-node".to_string(),
        label: "Explainer LLM Node".to_string(),
        provider: LLMProvider::LMStudio,
        model: model.clone(),
        system_prompt: "You are a knowledgeable assistant who explains topics clearly. For the given question or topic, provide a detailed, well-structured explanation that builds from foundational concepts to more nuanced points, using concrete examples where helpful.".to_string(),
        input_key: "/message".to_string(),
        output_key: "/explanation".to_string(),
        provider_config_id: provider_config.id.clone(),
    };

    let llm_summarizer_node_def = LlmNodeDef {
        id: "llm-summarizer-node".to_string(),
        label: "Explainer LLM Node".to_string(),
        provider: LLMProvider::LMStudio,
        model: model.clone(),
        system_prompt: "You are an expert in {{/message}} who summarizes text. Given the text, produce a concise summary of two to three sentences that captures the key points while preserving the original meaning.".to_string(),
        input_key: "/explanation".to_string(),
        output_key: "/summary".to_string(),
        provider_config_id: provider_config.id.clone(),
    };

    let llm_topic_generator_node_def = LlmNodeDef {
        id: "llm-topic-generator-node".to_string(),
        label: "Explainer LLM Node".to_string(),
        provider: LLMProvider::LMStudio,
        model: model.clone(),
        system_prompt: "You are an expert in {{/message}} who writes titles for text. Given some text, generate a short, descriptive title (five words or fewer) that captures its essence.".to_string(),
        input_key: "/explanation".to_string(),
        output_key: "/title".to_string(),
        provider_config_id: provider_config.id.clone(),
    };

    let connect_start_explainer = Connection {
        from: Port {
            node_id: start_node_def.id.clone(),
            name: format!("to {}", llm_explainer_node_def.label),
        },
        to: Port {
            node_id: llm_explainer_node_def.id.clone(),
            name: format!("from {}", start_node_def.label),
        },
    };

    let connect_explainer_summarizer = Connection {
        from: Port {
            node_id: llm_explainer_node_def.id.clone(),
            name: format!("to {}", llm_summarizer_node_def.label),
        },
        to: Port {
            node_id: llm_summarizer_node_def.id.clone(),
            name: format!("from {}", llm_explainer_node_def.label),
        },
    };

    let connect_explainer_topic_generator = Connection {
        from: Port {
            node_id: llm_explainer_node_def.id.clone(),
            name: format!("to {}", llm_topic_generator_node_def.label),
        },
        to: Port {
            node_id: llm_topic_generator_node_def.id.clone(),
            name: format!("from {}", llm_explainer_node_def.label),
        },
    };

    let connect_summarizer_end = Connection {
        from: Port {
            node_id: llm_summarizer_node_def.id.clone(),
            name: format!("to {}", end_node_def.label),
        },
        to: Port {
            node_id: end_node_def.id.clone(),
            name: format!("from {}", llm_summarizer_node_def.label),
        },
    };

    let connect_topic_generator_end = Connection {
        from: Port {
            node_id: llm_topic_generator_node_def.id.clone(),
            name: format!("to {}", end_node_def.label),
        },
        to: Port {
            node_id: end_node_def.id.clone(),
            name: format!("from {}", llm_topic_generator_node_def.label),
        },
    };

    let mut provider_configs = LLMProviderConfigs::new();
    provider_configs.insert(provider_config.id.clone(), provider_config);

    let graph_definition = GraphDefinition {
        graph_version: GraphVersion::V1,
        name: "Example Lathe Graph".to_string(),
        nodes: vec![
            NodeKind::Start(start_node_def),
            NodeKind::LLMNode(llm_explainer_node_def),
            NodeKind::LLMNode(llm_summarizer_node_def),
            NodeKind::LLMNode(llm_topic_generator_node_def),
            NodeKind::End(end_node_def),
        ],
        connections: vec![
            connect_start_explainer,
            connect_explainer_summarizer,
            connect_explainer_topic_generator,
            connect_summarizer_end,
            connect_topic_generator_end,
        ],
        provider_configs,
    };

    let lathe_graph = LatheGraph::from_def(graph_definition, true)?;
    let mut out_path = PathBuf::from_str(".")?;
    out_path.push("examples");
    out_path.push("explainer_agent.yaml");
    tracing::info!("Writing to {}", out_path.to_str().unwrap());
    yaml::save(&lathe_graph, &out_path)
}
