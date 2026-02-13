//! Chat/Conversation Commands

use agentlink_sdk::{
    protocols::conversation::CreateConversationRequest,
    AgentLinkClient, ClientConfig,
};
use clap::Subcommand;
use colored::Colorize;

use crate::config::CliConfig;
use crate::output;

/// Chat/Conversation commands
#[derive(Subcommand)]
pub enum ChatCommands {
    /// List all conversations
    List,

    /// Get conversation details
    Get {
        /// Conversation ID
        #[arg(short, long)]
        id: String,
    },

    /// Create a new direct conversation with a user
    CreateDirect {
        /// Target user ID
        #[arg(short, long)]
        user: String,
    },

    /// Create a new group conversation
    CreateGroup {
        /// Group name
        #[arg(short, long)]
        name: String,

        /// Member user IDs (comma-separated)
        #[arg(short, long)]
        members: String,
    },
}

impl ChatCommands {
    pub async fn execute(&self, config: &CliConfig) -> anyhow::Result<()> {
        let api_key = config.require_api_key()?;

        let client_config = ClientConfig::default().with_token(api_key.to_string());
        let client = AgentLinkClient::new(client_config);

        match self {
            ChatCommands::List => {
                output::header("Conversations");

                let response = client
                    .conversations()
                    .get_conversations()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to get conversations: {}", e))?;

                if response.conversations.is_empty() {
                    output::info("No conversations found");
                    return Ok(());
                }

                output::separator();
                for conv in &response.conversations {
                    let name = if let Some(ref n) = conv.name {
                        n.clone()
                    } else {
                        "Direct".to_string()
                    };

                    println!("{} {}", conv.id.dimmed(), name);
                    if let Some(ref last_msg) = conv.last_message {
                        let time = output::format_time(&last_msg.created_at);
                        let content = last_msg.content.as_deref().unwrap_or("");
                    println!("  {} {}", time.dimmed(), output::truncate(content, 50));
                    }
                    output::separator();
                }
            }

            ChatCommands::Get { id } => {
                output::header("Conversation Details");

                let conv = client
                    .conversations()
                    .get_conversation(id)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to get conversation: {}", e))?;

                output::field("ID", &conv.id);
                if let Some(ref name) = conv.name {
                    output::field("Name", name);
                }
                output::field("Type", &conv.conversation_type);
                output::field("Created", &output::format_time(&conv.created_at));
            }

            ChatCommands::CreateDirect { user } => {
                output::info(&format!("Creating direct conversation with user {}...", user));

                let conv = client
                    .conversations()
                    .get_or_create_direct_conversation(user)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to create conversation: {}", e))?;

                output::success("Conversation ready!");
                output::field("ID", &conv.id);
            }

            ChatCommands::CreateGroup { name, members } => {
                let member_ids: Vec<&str> = members.split(',').map(|s| s.trim()).collect();

                output::info(&format!("Creating group '{}' with {} members...", name, member_ids.len()));

                let request = CreateConversationRequest {
                    conversation_type: "group".to_string(),
                    name: Some(name.clone()),
                    member_ids: member_ids.iter().map(|s| s.to_string()).collect(),
                };

                let response = client
                    .conversations()
                    .create_conversation(&request)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to create group: {}", e))?;

                output::success("Group created!");
                output::field("ID", &response.conversation_id);
                output::field("Name", name);
            }
        }

        Ok(())
    }
}
