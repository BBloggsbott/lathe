//! `lathe` CLI entry point: generate example pipelines, run a pipeline once from the terminal,
//! or serve one over HTTP.

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

/// Top-level `lathe` subcommands.
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

/// Parses CLI args, loads `.env`, and dispatches to the selected [`Commands`] variant.
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
                .with_level(false)
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn clap_command_definition_is_valid() {
        // Catches malformed clap attributes (conflicting args, bad defaults, etc.) that would
        // otherwise only surface at runtime via a panic.
        Args::command().debug_assert();
    }

    #[test]
    fn example_subcommand_parses_name_provider_and_model() {
        let args = Args::parse_from([
            "lathe", "example", "simple", "-p", "open-ai", "-m", "gpt-5.5",
        ]);
        match args.command {
            Commands::Example {
                name,
                provider,
                model,
            } => {
                assert!(matches!(name, ExampleType::Simple));
                assert!(matches!(provider, LLMProvider::OpenAI));
                assert_eq!(model, "gpt-5.5");
            }
            _ => panic!("expected Example subcommand"),
        }
    }

    #[test]
    fn run_subcommand_parses_pipeline_and_message() {
        let args = Args::parse_from(["lathe", "run", "-p", "pipeline.yaml", "-m", "hi"]);
        match args.command {
            Commands::Run { pipeline, message } => {
                assert_eq!(pipeline, PathBuf::from("pipeline.yaml"));
                assert_eq!(message, "hi");
            }
            _ => panic!("expected Run subcommand"),
        }
    }

    #[test]
    fn server_subcommand_defaults_host_and_port() {
        let args = Args::parse_from(["lathe", "server", "-p", "pipeline.yaml"]);
        match args.command {
            Commands::Server { host, port, .. } => {
                assert_eq!(host, "127.0.0.1");
                assert_eq!(port, 8080);
            }
            _ => panic!("expected Server subcommand"),
        }
    }
}
