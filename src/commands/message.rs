//! Message Commands

use agentlink_sdk::{AgentLinkClient, ClientConfig};
use clap::Subcommand;
use colored::Colorize;

use crate::config::CliConfig;
use crate::output;

/// Message commands
#[derive(Subcommand)]
pub enum MessageCommands {
    /// Send a message to a conversation
    Send {
        /// Conversation ID
        #[arg(short, long)]
        conversation: String,

        /// Message content
        #[arg(short, long)]
        content: String,
    },

    /// List messages in a conversation
    List {
        /// Conversation ID
        #[arg(short, long)]
        conversation: String,

        /// Maximum number of messages to retrieve
        #[arg(short, long, default_value = "20")]
        limit: i32,
    },
}

impl MessageCommands {
    pub async fn execute(&self, config: &CliConfig) -> anyhow::Result<()> {
        let api_key = config.require_api_key()?;

        let client_config = ClientConfig::default().with_token(api_key.to_string());
        let client = AgentLinkClient::new(client_config);

        match self {
            MessageCommands::Send { conversation, content } => {
                output::info(&format!("Sending message to conversation {}...", conversation));

                let message = client
                    .messages()
                    .send_message(conversation, content, None, None, None)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to send message: {}", e))?;

                output::success("Message sent!");
                output::field("Message ID", &message.id);
            }

            MessageCommands::List { conversation, limit } => {
                output::header(&format!("Messages in {}", conversation));

                let response = client
                    .messages()
                    .get_conversation_messages(conversation, None, Some(*limit))
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to get messages: {}", e))?;

                if response.messages.is_empty() {
                    output::info("No messages found");
                    return Ok(());
                }

                for msg in response.messages.iter().rev() {
                    let time = output::format_time(&msg.created_at);
                    let content = msg.content.as_deref().unwrap_or("");
                    let content = output::truncate(content, 50);

                    println!(
                        "{} {} {}",
                        time.dimmed(),
                        &msg.sender_id[..8.min(msg.sender_id.len())].blue(),
                        content
                    );
                }
            }
        }

        Ok(())
    }
}
