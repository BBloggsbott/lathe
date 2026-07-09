pub mod example;
pub mod run;
pub mod server;

use crate::example::ExampleType;
use anyhow::Result;
use clap::{Parser, Subcommand};
use lathe_core::provider::LLMProvider;
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

    /// Launch the Lathe Server
    Server {
        /// Path to the pipeline YAML file
        #[arg(short, long)]
        pipeline: PathBuf,

        /// Host for the server
        #[arg(short = 'H', long, default_value = "127.0.0.1")]
        host: String,

        /// Port for the server
        #[arg(short = 'P', long, default_value = "8080")]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env (OPENAI_API_KEY etc.)
    dotenvy::dotenv().ok();

    let args = Args::parse();

    match args.command {
        Commands::Example {
            name: example_type,
            provider,
            model,
        } => {
            tracing_subscriber::fmt()
                .without_time()
                .with_target(false)
                .with_level(true)
                .compact()
                .init();
            example::create_example(example_type, provider, model)?;
        }
        Commands::Run { pipeline, message } => {
            run::run_pipeline(pipeline, message).await?;
        }
        Commands::Server {
            pipeline,
            host,
            port,
        } => {
            tracing_subscriber::fmt().init();
            server::start_server(pipeline, host.as_str(), port).await?;
        }
    }

    Ok(())
}
