use anyhow::Result;
use clap::Parser;
use lathe_core::executor::Executor;
use lathe_core::graph::port::{Connection, Port};
use lathe_core::graph::{GraphDefinition, LatheGraph};
use lathe_core::node_defs::llm::LlmNodeDef;
use lathe_core::node_defs::{EndNodeDef, NodeKind, StartNodeDef};
use lathe_core::provider::{LLMProvider, LLMProviderConfig, LLMProviderConfigs};
use lathe_core::state::AgentState;
use lathe_core::{registry, yaml};
use serde_json::{Map, Value};
use std::path::PathBuf;
use std::str::FromStr;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(name = "lathe", about = "Execute a Lathe pipeline from a YAML file")]
struct Args {
    /// Path to the pipeline YAML file
    #[arg(short, long)]
    pipeline: Option<PathBuf>,

    /// The user message to send into the pipeline
    #[arg(short, long)]
    message: Option<String>,

    /// Create an example graph yaml
    #[arg(short, long)]
    create_example: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env (OPENAI_API_KEY etc.)
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    if args.create_example {
        tracing::info!("Creating example yaml");
        return create_example_yaml();
    }

    let pipeline = args.pipeline.unwrap();
    let message = args.message.unwrap();

    let graph = yaml::load(&pipeline)?;
    tracing::info!("Loaded pipeline: {}", graph.name);

    let nodes = registry::inflate(&graph.definition.nodes, &graph.definition.provider_configs)?;

    let executor = Executor::new(graph, nodes);

    let mut message_as_state = Map::new();
    message_as_state.insert("message".to_string(), Value::String(message));

    let initial = AgentState::new(message_as_state);
    let result = executor.run(initial).await?;

    println!("\n=== Pipeline Output ===");
    println!("{:?}", result);

    Ok(())
}

fn create_example_yaml() -> Result<()> {
    let start_node_def = StartNodeDef::default();
    let mut end_node_def = EndNodeDef::default();
    end_node_def.out_pointers = vec!["/message".to_string()];

    let provider_config = LLMProviderConfig::default(&LLMProvider::LMStudio);
    // todo: Using my local model fro dev-testing. Update to a generic example.
    let model = "qwen2.5-0.5b-instruct-quantized".to_string();

    let llm_node_def = LlmNodeDef {
        id: Uuid::new_v4().to_string(),
        label: "Simple Assistant LLM Node".to_string(),
        provider: LLMProvider::LMStudio,
        model,
        system_prompt: "You are a helpful assistant".to_string(),
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
        name: "Example Lathe Graph".to_string(),
        nodes: vec![
            NodeKind::Start(start_node_def),
            NodeKind::LLMNode(llm_node_def),
            NodeKind::End(end_node_def),
        ],
        connections: vec![connect_start_llm, connect_llm_end],
        provider_configs,
    };

    let lathe_graph = LatheGraph::from_def(graph_definition)?;
    let mut out_path = PathBuf::from_str(".")?;
    out_path.push("examples");
    out_path.push("simple_lathe_graph.yaml");
    tracing::info!("Writing to {}", out_path.to_str().unwrap());
    yaml::save(&lathe_graph, &out_path)
    // Ok(())
}
