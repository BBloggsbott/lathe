pub mod example;

use crate::example::{ExampleType, create_explainer_agent, create_simple_agent};
use anyhow::Result;
use clap::{Parser, Subcommand};
use lathe_core::executor::Executor;
use lathe_core::provider::LLMProvider;
use lathe_core::state::AgentState;
use lathe_core::{registry, yaml};
use serde_json::{Map, Value};
use std::path::PathBuf;

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
                    tracing::info!("Creating simple agent example");
                    return create_simple_agent(provider, model);
                }

                ExampleType::Explainer => {
                    tracing::info!("Creating explainer agent example");
                    return create_explainer_agent(provider, model);
                }

                ExampleType::None => {
                    tracing::info!("Not creating example yaml");
                }
            }
        }
        Commands::Run { pipeline, message } => {
            let graph = yaml::load(pipeline.as_path())?;
            tracing::info!("Loaded pipeline: {}", graph.name);

            let nodes =
                registry::inflate(&graph.definition.nodes, &graph.definition.provider_configs)?;
            tracing::info!("Executable nodes built: {} nodes", nodes.len());

            let executor = Executor::new(graph, nodes);

            let mut message_as_state = Map::new();
            message_as_state.insert("message".to_string(), Value::String(message.clone()));

            let initial = AgentState::new(message_as_state);
            let result = executor.run(initial).await?;

            println!("{}", result.pretty_string()?);
        }
    }

    Ok(())
}
