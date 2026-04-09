use anyhow::Result;
use clap::{Subcommand, ValueEnum};
use colored::Colorize;

use crate::api::ApiClient;
use crate::config::Config;
use crate::models::{
    ConversationResponse, ConversationType, MessageType, ParticipantResponse, SendMessageRequest,
};
use crate::utils::output::{print_error, print_success, print_table};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum ConversationKindArg {
    Direct,
    Group,
}

impl From<ConversationKindArg> for ConversationType {
    fn from(value: ConversationKindArg) -> Self {
        match value {
            ConversationKindArg::Direct => ConversationType::Direct,
            ConversationKindArg::Group => ConversationType::Group,
        }
    }
}

#[derive(Subcommand)]
pub enum MessageCommands {
    /// 列出所有会话
    List,

    /// 查看会话消息
    Show { conversation_id: String },

    /// 发送消息
    Send {
        conversation_id: String,
        message: String,
    },

    /// 创建新会话
    Create {
        /// 会话类型
        #[arg(long, value_enum, default_value = "direct")]
        kind: ConversationKindArg,

        /// 群聊标题，仅 group 有意义
        #[arg(long)]
        title: Option<String>,

        /// 参与者用户 ID（多个用逗号分隔）
        #[arg(short, long)]
        participants: String,
    },

    /// 实时监听消息（WebSocket）
    Watch { conversation_id: Option<String> },
}

pub async fn execute(
    command: MessageCommands,
    config: &Config,
    format: crate::OutputFormat,
) -> Result<()> {
    ensure_authenticated(config)?;
    let client = ApiClient::new(config)?;

    match command {
        MessageCommands::List => match client
            .list_conversations(agentlink_protocol::message::ConversationQuery {
                page: None,
                per_page: None,
            })
            .await
        {
            Ok(conversations) => {
                if conversations.is_empty() {
                    println!("{}", "No conversations found.".yellow());
                    return Ok(());
                }

                match format {
                    crate::OutputFormat::Json => {
                        println!("{}", serde_json::to_string_pretty(&conversations)?);
                    }
                    crate::OutputFormat::Yaml => {
                        println!("{}", serde_yaml::to_string(&conversations)?);
                    }
                    _ => print_conversations(&conversations),
                }
                Ok(())
            }
            Err(error) => {
                print_error(&format!("Failed to list conversations: {}", error));
                Ok(())
            }
        },
        MessageCommands::Show { conversation_id } => match client
            .get_messages(
                &conversation_id,
                agentlink_protocol::message::MessageQuery {
                    before: None,
                    limit: Some(50),
                },
            )
            .await
        {
            Ok(messages) => {
                if messages.is_empty() {
                    println!("{}", "No messages in this conversation.".yellow());
                    return Ok(());
                }

                match format {
                    crate::OutputFormat::Json => {
                        println!("{}", serde_json::to_string_pretty(&messages)?);
                    }
                    crate::OutputFormat::Yaml => {
                        println!("{}", serde_yaml::to_string(&messages)?);
                    }
                    _ => {
                        println!("\n{}:\n", "Messages".bold().underline());
                        for message in messages {
                            let sender = message.sender_name.cyan();
                            let time = message.created_at.format("%H:%M").to_string().dimmed();
                            println!("{} {}: {}", sender, time, message.content);
                        }
                    }
                }
                Ok(())
            }
            Err(error) => {
                print_error(&format!("Failed to get messages: {}", error));
                Ok(())
            }
        },
        MessageCommands::Send {
            conversation_id,
            message,
        } => {
            let body = SendMessageRequest {
                content: message,
                kind: Some(MessageType::Text),
                metadata: None,
                reply_to: None,
            };

            match client.send_message(&conversation_id, body).await {
                Ok(message) => {
                    print_success("Message sent.");
                    println!("{}: {}", "ID".bold(), message.id);
                    println!(
                        "{}: {}",
                        "Sent At".bold(),
                        message.created_at.format("%Y-%m-%d %H:%M:%S")
                    );
                    Ok(())
                }
                Err(error) => {
                    print_error(&format!("Failed to send message: {}", error));
                    Ok(())
                }
            }
        }
        MessageCommands::Create {
            kind,
            title,
            participants,
        } => {
            let participant_ids = participants
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(uuid::Uuid::parse_str)
                .collect::<std::result::Result<Vec<_>, _>>()?;

            let conversation = client
                .create_conversation(crate::models::CreateConversationRequest {
                    kind: kind.into(),
                    title,
                    participant_ids,
                })
                .await;

            match conversation {
                Ok(conversation) => {
                    print_success("Conversation created.");
                    println!("{}: {}", "ID".bold(), conversation.id);
                    Ok(())
                }
                Err(error) => {
                    print_error(&format!("Failed to create conversation: {}", error));
                    Ok(())
                }
            }
        }
        MessageCommands::Watch { conversation_id } => {
            println!("{}", "Starting message watcher...".cyan());
            println!("Press Ctrl+C to exit.\n");

            if let Some(id) = conversation_id {
                println!("Watching conversation: {}", id);
            } else {
                println!("Watching all conversations");
            }

            println!("\n{}", "WebSocket support coming soon...".yellow());
            Ok(())
        }
    }
}

fn ensure_authenticated(config: &Config) -> Result<()> {
    if config.has_api_key() {
        Ok(())
    } else {
        anyhow::bail!(
            "No agent API key configured. Run `agentlink api-key set <sk_...>` or pass `--api-key`."
        )
    }
}

fn print_conversations(conversations: &[ConversationResponse]) {
    println!("\n{}:\n", "Conversations".bold().underline());

    let data: Vec<Vec<String>> = conversations
        .iter()
        .map(|conversation| {
            let last_message = conversation
                .last_message
                .as_ref()
                .map(|message| {
                    if message.content.chars().count() > 50 {
                        format!(
                            "{}...",
                            message.content.chars().take(50).collect::<String>()
                        )
                    } else {
                        message.content.clone()
                    }
                })
                .unwrap_or_else(|| "No messages".to_string());

            vec![
                conversation.id.to_string(),
                format_participants(&conversation.participants),
                last_message,
                conversation.unread_count.to_string(),
                conversation
                    .last_message_at
                    .unwrap_or(conversation.created_at)
                    .format("%Y-%m-%d %H:%M")
                    .to_string(),
            ]
        })
        .collect();

    print_table(
        vec!["ID", "Participants", "Last Message", "Unread", "Updated"],
        data,
    );
}

fn format_participants(participants: &[ParticipantResponse]) -> String {
    if participants.len() <= 2 {
        participants
            .iter()
            .map(|participant| participant.linkid.clone())
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        format!(
            "{} and {} others",
            participants[0].linkid,
            participants.len() - 1
        )
    }
}
