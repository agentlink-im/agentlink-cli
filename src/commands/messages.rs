use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;

use crate::api::ApiClient;
use crate::config::Config;
use crate::utils::output::{print_error, print_success, print_table};

#[derive(Subcommand)]
pub enum MessageCommands {
    /// 列出所有会话
    List,

    /// 查看会话消息
    Show {
        /// 会话 ID
        conversation_id: String,
    },

    /// 发送消息
    Send {
        /// 会话 ID
        conversation_id: String,

        /// 消息内容
        message: String,
    },

    /// 创建新会话
    Create {
        /// 参与者用户 ID（多个用逗号分隔）
        #[arg(short, long)]
        participants: String,
    },

    /// 实时监听消息（WebSocket）
    Watch {
        /// 会话 ID（可选，不指定则监听所有消息）
        conversation_id: Option<String>,
    },
}

pub async fn execute(
    command: MessageCommands,
    config: &Config,
    format: crate::OutputFormat,
) -> Result<()> {
    if !config.is_authenticated() {
        print_error("You must be logged in to use this command.");
        println!("Run {} to authenticate.", "agentlink auth login".cyan());
        return Ok(());
    }

    let client = ApiClient::new(config)?;

    match command {
        MessageCommands::List => {
            match client.list_conversations().await {
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
                        _ => {
                            println!("\n{}:\n", "Conversations".bold().underline());

                            let data: Vec<Vec<String>> = conversations
                                .iter()
                                .map(|c| {
                                    let last_msg = c
                                        .last_message
                                        .as_ref()
                                        .map(|m| m.content.chars().take(50).collect::<String>() + "...")
                                        .unwrap_or_else(|| "No messages".to_string());

                                    vec![
                                        c.id.clone(),
                                        format_participants(&c.participants),
                                        last_msg,
                                        c.unread_count.to_string(),
                                        c.updated_at.format("%Y-%m-%d %H:%M").to_string(),
                                    ]
                                })
                                .collect();

                            print_table(
                                vec!["ID", "Participants", "Last Message", "Unread", "Updated"],
                                data,
                            );
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    print_error(&format!("Failed to list conversations: {}", e));
                    Ok(())
                }
            }
        }

        MessageCommands::Show { conversation_id } => {
            match client.get_messages(&conversation_id).await {
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

                            for msg in messages {
                                let is_me = msg.sender_id == "me"; // 简化处理
                                let prefix = if is_me { "You".green() } else { msg.sender_name.cyan() };
                                let time = msg.sent_at.format("%H:%M").to_string().dimmed();

                                println!("{} {}: {}", prefix, time, msg.content);
                            }
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    print_error(&format!("Failed to get messages: {}", e));
                    Ok(())
                }
            }
        }

        MessageCommands::Send {
            conversation_id,
            message,
        } => {
            let body = serde_json::json!({
                "content": message,
                "contentType": "text",
            });

            match client.send_message(&conversation_id, body).await {
                Ok(msg) => {
                    print_success("Message sent!");
                    println!("  {}: {}", "ID".bold(), msg.id);
                    println!("  {}: {}", "Sent at".bold(), msg.sent_at.format("%Y-%m-%d %H:%M:%S"));
                    Ok(())
                }
                Err(e) => {
                    print_error(&format!("Failed to send message: {}", e));
                    Ok(())
                }
            }
        }

        MessageCommands::Create { participants } => {
            let participant_ids: Vec<String> = participants
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();

            let body = serde_json::json!({
                "participantIds": participant_ids,
            });

            match client.post("/api/v1/conversations", Some(body)).await::<serde_json::Value>().await {
                Ok(response) => {
                    print_success("Conversation created!");
                    println!("  {}: {}", "ID".bold(), response["id"]);
                    Ok(())
                }
                Err(e) => {
                    print_error(&format!("Failed to create conversation: {}", e));
                    Ok(())
                }
            }
        }

        MessageCommands::Watch { conversation_id } => {
            println!("{}", "Starting message watcher...".cyan());
            println!("Press Ctrl+C to exit.\n");

            // 这里应该实现 WebSocket 连接
            // 简化版本，实际实现需要使用 tokio-tungstenite
            if let Some(id) = conversation_id {
                println!("Watching conversation: {}", id);
            } else {
                println!("Watching all conversations");
            }

            // TODO: 实现 WebSocket 监听
            println!("\n{}", "WebSocket support coming soon...".yellow());

            Ok(())
        }
    }
}

fn format_participants(participants: &[crate::models::Participant]) -> String {
    if participants.len() <= 2 {
        participants
            .iter()
            .map(|p| p.display_name.clone())
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        format!("{} and {} others", participants[0].display_name, participants.len() - 1)
    }
}
