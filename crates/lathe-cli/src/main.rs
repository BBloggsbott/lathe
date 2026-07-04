use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
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

#[derive(Parser, Debug)]
#[command(name = "lathe", about = "Execute a Lathe pipeline from a YAML file")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create an example Graph's yaml
    Example {
        /// Name of the pre-defined example
        name: ExampleType,

        /// LLM Provider to use for the example
        #[arg(short, long)]
        provider: LLMProvider,

        /// Model name to use for the example
        #[arg(short, long)]
        model: String,
    },

    /// Run a pipeline from the yaml
    Run {
        /// Path to the pipeline YAML file
        #[arg(short, long)]
        pipeline: PathBuf,

        /// The user message to send into the pipeline
        #[arg(short, long)]
        message: String,
    },
}

#[derive(Debug, Clone, ValueEnum)]
enum ExampleType {
    Simple,
    None,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env (OPENAI_API_KEY etc.)
    dotenvy::dotenv().ok();

    let args = Args::parse();

    match args.command {
        Commands::Example {
            name,
            provider,
            model,
        } => {
            tracing_subscriber::fmt()
                .without_time()
                .with_target(false)
                .with_level(true)
                .compact()
                .init();
            match name {
                ExampleType::Simple => {
                    tracing::info!("Creating example yaml");
                    return create_example_yaml(provider, model);
                }

                ExampleType::None => {
                    tracing::info!("Not creating example yaml");
                }
            }
        }
        Commands::Run { pipeline, message } => {
            tracing_subscriber::fmt()
                .without_time()
                .with_target(false)
                .with_level(true)
                .compact()
                .init();
            let graph = yaml::load(pipeline.as_path())?;
            tracing::info!("Loaded pipeline: {}", graph.name);

            let nodes =
                registry::inflate(&graph.definition.nodes, &graph.definition.provider_configs)?;

            let executor = Executor::new(graph, nodes);

            let mut message_as_state = Map::new();
            message_as_state.insert("message".to_string(), Value::String(message.clone()));

            let initial = AgentState::new(message_as_state);
            let result = executor.run(initial).await?;

            println!("\nPipeline Output:");
            println!("{:?}", result);
        }
    }

    Ok(())
}

fn create_example_yaml(provider: LLMProvider, model: String) -> Result<()> {
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
    provider_config.id = "my-lm-studio-model".to_string();
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
