//! Generates ready-to-run example pipeline YAML files under `./examples`, used by the
//! `lathe example` CLI subcommand to demonstrate graph construction.

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

/// Which built-in example pipeline to generate.
#[derive(Debug, Clone, ValueEnum)]
pub enum ExampleType {
    Simple,
    Explainer,
    None,
}

/// Writes the YAML for `example_type` to `./examples`, configured to use `provider`/`model`.
/// `ExampleType::None` is a no-op.
pub fn create_example(
    example_type: ExampleType,
    provider: LLMProvider,
    model: String,
) -> Result<()> {
    if !matches!(example_type, ExampleType::None) {
        std::fs::create_dir_all("examples")?;
    }

    match example_type {
        ExampleType::Simple => {
            tracing::info!("Creating simple agent example");
            create_simple_agent(provider, model)
        }

        ExampleType::Explainer => {
            tracing::info!("Creating explainer agent example");
            create_explainer_agent(provider, model)
        }

        ExampleType::None => {
            tracing::info!("Not creating example yaml");
            Ok(())
        }
    }
}

/// Builds and saves a minimal Start -> LLM -> End pipeline to `examples/simple_agent.yaml`.
fn create_simple_agent(provider: LLMProvider, model: String) -> Result<()> {
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

    let llm_node_def = LlmNodeDef {
        id: "llm-node".to_string(),
        label: "Simple Assistant LLM Node".to_string(),
        provider,
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
        name: "Example Lathe Graph - Simple".to_string(),
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

/// Builds and saves a fan-out pipeline (Start -> explainer LLM -> summarizer + title-generator
/// LLMs -> End) to `examples/explainer_agent.yaml`.
fn create_explainer_agent(provider: LLMProvider, model: String) -> Result<()> {
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

    let llm_explainer_node_def = LlmNodeDef {
        id: "llm-explainer-node".to_string(),
        label: "Explainer LLM Node".to_string(),
        provider: provider.clone(),
        model: model.clone(),
        system_prompt: "You are a knowledgeable assistant who explains topics clearly. For the given question or topic, provide a detailed, well-structured explanation that builds from foundational concepts to more nuanced points, using concrete examples where helpful.".to_string(),
        input_key: "/message".to_string(),
        output_key: "/explanation".to_string(),
        provider_config_id: provider_config.id.clone(),
    };

    let llm_summarizer_node_def = LlmNodeDef {
        id: "llm-summarizer-node".to_string(),
        label: "Explainer LLM Node".to_string(),
        provider: provider.clone(),
        model: model.clone(),
        system_prompt: "You are an expert in {{/message}} who summarizes text. Given the text, produce a concise summary of two to three sentences that captures the key points while preserving the original meaning.".to_string(),
        input_key: "/explanation".to_string(),
        output_key: "/summary".to_string(),
        provider_config_id: provider_config.id.clone(),
    };

    let llm_topic_generator_node_def = LlmNodeDef {
        id: "llm-topic-generator-node".to_string(),
        label: "Explainer LLM Node".to_string(),
        provider: provider.clone(),
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
        name: "Example Lathe Graph - Explainer".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Both `create_simple_agent` and `create_explainer_agent` write to hardcoded paths under
    /// `./examples`, so these tests clean that directory up afterwards to avoid leaving stray
    /// artifacts. `EXAMPLES_DIR_LOCK` serializes the two tests that touch the shared directory
    /// (tests within this binary otherwise run concurrently), so it's always safe to remove the
    /// whole directory afterwards if this test is the one that created it.
    static EXAMPLES_DIR_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    struct ExamplesDirCleanup {
        dir: PathBuf,
        remove_whole_dir: bool,
        _guard: std::sync::MutexGuard<'static, ()>,
    }

    impl ExamplesDirCleanup {
        fn setup() -> Self {
            let guard = EXAMPLES_DIR_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            let dir = PathBuf::from("examples");
            let remove_whole_dir = !dir.exists();
            std::fs::create_dir_all(&dir).unwrap();
            Self {
                dir,
                remove_whole_dir,
                _guard: guard,
            }
        }
    }

    impl Drop for ExamplesDirCleanup {
        fn drop(&mut self) {
            if self.remove_whole_dir {
                let _ = std::fs::remove_dir_all(&self.dir);
            } else {
                let _ = std::fs::remove_file(self.dir.join("simple_agent.yaml"));
                let _ = std::fs::remove_file(self.dir.join("explainer_agent.yaml"));
            }
        }
    }

    #[test]
    fn create_example_simple_writes_a_loadable_pipeline() {
        let cleanup = ExamplesDirCleanup::setup();
        let path = cleanup.dir.join("simple_agent.yaml");

        create_example(
            ExampleType::Simple,
            LLMProvider::LMStudio,
            "test-model".to_string(),
        )
        .unwrap();

        let graph = yaml::load(&path, true).unwrap();
        assert_eq!(graph.name, "Example Lathe Graph - Simple");
        assert_eq!(graph.definition.nodes.len(), 3);
    }

    #[test]
    fn create_example_explainer_writes_a_loadable_pipeline() {
        let cleanup = ExamplesDirCleanup::setup();
        let path = cleanup.dir.join("explainer_agent.yaml");

        create_example(
            ExampleType::Explainer,
            LLMProvider::LMStudio,
            "test-model".to_string(),
        )
        .unwrap();

        let graph = yaml::load(&path, true).unwrap();
        assert_eq!(graph.name, "Example Lathe Graph - Explainer");
        assert_eq!(graph.definition.nodes.len(), 5);
    }

    /// Collects the `provider` of every `LLMNode` in a graph definition.
    fn llm_node_providers(graph: &lathe_core::graph::LatheGraph) -> Vec<LLMProvider> {
        graph
            .definition
            .nodes
            .iter()
            .filter_map(|node| match node {
                NodeKind::LLMNode(llm_node) => Some(llm_node.provider.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn create_example_simple_uses_requested_provider_for_llm_nodes() {
        let cleanup = ExamplesDirCleanup::setup();
        let path = cleanup.dir.join("simple_agent.yaml");

        create_example(
            ExampleType::Simple,
            LLMProvider::OpenAI,
            "test-model".to_string(),
        )
        .unwrap();

        let graph = yaml::load(&path, true).unwrap();
        let providers = llm_node_providers(&graph);
        assert!(!providers.is_empty());
        assert!(
            providers
                .iter()
                .all(|provider| matches!(provider, LLMProvider::OpenAI))
        );
    }

    #[test]
    fn create_example_explainer_uses_requested_provider_for_llm_nodes() {
        let cleanup = ExamplesDirCleanup::setup();
        let path = cleanup.dir.join("explainer_agent.yaml");

        create_example(
            ExampleType::Explainer,
            LLMProvider::OpenAI,
            "test-model".to_string(),
        )
        .unwrap();

        let graph = yaml::load(&path, true).unwrap();
        let providers = llm_node_providers(&graph);
        assert_eq!(providers.len(), 3);
        assert!(
            providers
                .iter()
                .all(|provider| matches!(provider, LLMProvider::OpenAI))
        );
    }

    #[test]
    fn create_example_none_is_a_no_op() {
        assert!(
            create_example(
                ExampleType::None,
                LLMProvider::LMStudio,
                "test-model".to_string()
            )
            .is_ok()
        );
    }
}
