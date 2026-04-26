use anyhow::Result;
use clap::{Subcommand, ValueEnum};
use colored::Colorize;

use agentlink_protocol::message::{
    ConversationResponse, CreateConversationRequest, ParticipantResponse, SendMessageRequest,
};
use agentlink_protocol::ws_event::WsMessageCreatedPayload;
use agentlink_protocol::{ConversationType, MessageType};

use crate::config::Config;
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
        /// 对方 linkid (如: alice, agent_001)
        recipient: String,
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
    let client = config.to_client()?;

    match command {
        MessageCommands::List => match client
            .messages
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
            .messages
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
            recipient,
            message,
        } => {
            let conversation_id = resolve_conversation_id(&client, &recipient).await;
            let conversation_id = match conversation_id {
                Ok(id) => id,
                Err(error) => {
                    print_error(&format!("Failed to resolve recipient '{}': {}", recipient, error));
                    return Ok(());
                }
            };

            let body = SendMessageRequest {
                content: message,
                kind: Some(MessageType::Text),
                metadata: None,
                reply_to: None,
            };

            match client.messages.send_message(&conversation_id, body).await {
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
                .map(|s| s.to_string())
                .collect::<Vec<_>>();

            let conversation = client
                .messages
                .create_conversation(CreateConversationRequest {
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
            if let Err(error) = run_watch(config, conversation_id.as_deref()).await {
                print_error(&format!("WebSocket error: {}", error));
            }
            Ok(())
        }
    }
}

async fn resolve_conversation_id(
    client: &agentlink_rust_sdk::AgentLinkClient,
    recipient: &str,
) -> Result<String> {
    let user = client.users.get_user(recipient).await?;
    let target_user_id = user.id.to_string();

    let conversations = client
        .messages
        .list_conversations(agentlink_protocol::message::ConversationQuery {
            page: None,
            per_page: None,
        })
        .await?;

    // Find an existing direct conversation with this user
    for conversation in &conversations {
        if conversation.kind == ConversationType::Direct
            && conversation
                .participants
                .iter()
                .any(|p| p.user_id == user.id)
        {
            return Ok(conversation.id.to_string());
        }
    }

    // No existing conversation: create a new direct one
    let new_conversation = client
        .messages
        .create_conversation(CreateConversationRequest {
            kind: ConversationType::Direct,
            title: None,
            participant_ids: vec![target_user_id],
        })
        .await?;

    Ok(new_conversation.id.to_string())
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

/// 使用 SDK WebSocket 客户端实现消息监听。
async fn run_watch(
    config: &Config,
    conversation_filter: Option<&str>,
) -> anyhow::Result<()> {
    use agentlink_rust_sdk::websocket::WsEvent;
    use futures_util::StreamExt;

    let client = config.to_client()?;
    let mut ws_stream = client.websocket().connect().await?;

    println!("{}", "Starting message watcher...".cyan());
    println!("Press Ctrl+C to exit.\n");

    while let Some(event) = ws_stream.next().await {
        match event {
            Ok(WsEvent::MessageCreated(payload)) => {
                handle_message_created(&payload, conversation_filter);
            }
            Ok(WsEvent::ConnectionReady(payload)) => {
                println!(
                    "[{}] [READY] Connected as {}",
                    chrono::Utc::now().format("%H:%M:%S"),
                    payload.linkid
                );
            }
            Ok(WsEvent::EventNotification(payload)) => {
                println!("[NOTIFICATION] {}", payload.data.title);
                if !payload.data.content.is_empty() {
                    println!("  {}", payload.data.content);
                }
            }
            Ok(WsEvent::Pong(_)) => {}
            Ok(WsEvent::Error(payload)) => {
                eprintln!(
                    "[{}] [ERROR] {}: {}",
                    chrono::Utc::now().format("%H:%M:%S"),
                    payload.code,
                    payload.message
                );
            }
            Ok(WsEvent::Raw(text)) => {
                println!("[RAW] {}", text);
            }
            Err(e) => {
                eprintln!("[ERROR] WebSocket error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

fn handle_message_created(
    payload: &WsMessageCreatedPayload,
    filter_conversation_id: Option<&str>,
) {
    if let Some(filter) = filter_conversation_id {
        if payload.message.conversation_id.to_string() != filter {
            return;
        }
    }

    let msg = &payload.message;
    println!(
        "[{}] [MESSAGE] {}: {}",
        msg.created_at.format("%H:%M:%S"),
        msg.sender_name,
        msg.content
    );
}
