//! AgentLink CLI
//!
//! Command-line interface for AgentLink IM service.

mod chat;
mod commands;
mod config;
mod output;

use clap::{Parser, Subcommand};
use commands::{ChatCommands, FriendCommands, MessageCommands, UserCommands};
use config::CliConfig;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// AgentLink CLI - Command-line interface for AgentLink IM service
#[derive(Parser)]
#[command(name = "agentlink")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// API Key (or set AGENTLINK_API_KEY env var)
    #[arg(short = 'k', long, env = "AGENTLINK_API_KEY")]
    api_key: Option<String>,

    /// API URL (or set AGENTLINK_API_URL env var)
    #[arg(short = 'u', long, env = "AGENTLINK_API_URL")]
    api_url: Option<String>,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Output format (text, json)
    #[arg(long, default_value = "text")]
    format: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// User commands
    #[command(subcommand)]
    User(UserCommands),

    /// Message commands
    #[command(subcommand)]
    Msg(MessageCommands),

    /// Conversation/Chat commands
    #[command(subcommand)]
    Chat(ChatCommands),

    /// Friend commands
    #[command(subcommand)]
    Friend(FriendCommands),

    /// Start interactive chat mode
    Interactive {
        /// Conversation ID to join
        #[arg(short, long)]
        conversation: Option<String>,
    },

    /// Start event loop to monitor all events
    Events,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    if cli.verbose {
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new("debug"))
            .with(tracing_subscriber::fmt::layer())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new("warn"))
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    // Load configuration
    let config = CliConfig::from_args(&cli);

    // Execute command
    match cli.command {
        Commands::User(cmd) => cmd.execute(&config).await?,
        Commands::Msg(cmd) => cmd.execute(&config).await?,
        Commands::Chat(cmd) => cmd.execute(&config).await?,
        Commands::Friend(cmd) => cmd.execute(&config).await?,
        Commands::Interactive { conversation } => {
            chat::interactive::start(&config, conversation).await?;
        }
        Commands::Events => {
            commands::start_event_loop(&config).await?;
        }
    }

    Ok(())
}
