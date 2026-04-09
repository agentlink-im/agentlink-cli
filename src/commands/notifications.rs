use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;

use crate::api::ApiClient;
use crate::config::Config;
use crate::utils::output::{print_error, print_success, print_table};

#[derive(Subcommand)]
pub enum NotificationCommands {
    /// 列出通知
    List {
        #[arg(short, long)]
        unread: bool,
    },

    /// 标记通知为已读
    MarkRead {
        /// 通知 ID，不指定则标记全部已读
        id: Option<String>,
    },

    /// 实时监听通知
    Watch,
}

pub async fn execute(
    command: NotificationCommands,
    config: &Config,
    format: crate::OutputFormat,
) -> Result<()> {
    ensure_authenticated(config)?;
    let client = ApiClient::new(config)?;

    match command {
        NotificationCommands::List { unread } => match client.list_notifications(unread).await {
            Ok(notifications) => {
                if notifications.is_empty() {
                    if unread {
                        println!("{}", "No unread notifications.".green());
                    } else {
                        println!("{}", "No notifications.".yellow());
                    }
                    return Ok(());
                }

                match format {
                    crate::OutputFormat::Json => {
                        println!("{}", serde_json::to_string_pretty(&notifications)?);
                    }
                    crate::OutputFormat::Yaml => {
                        println!("{}", serde_yaml::to_string(&notifications)?);
                    }
                    _ => {
                        let title = if unread {
                            "Unread Notifications"
                        } else {
                            "Notifications"
                        };
                        println!("\n{}:\n", title.bold().underline());

                        let data: Vec<Vec<String>> = notifications
                            .iter()
                            .map(|notification| {
                                let status = if notification.is_read {
                                    "✓".green().to_string()
                                } else {
                                    "○".yellow().to_string()
                                };

                                vec![
                                    status,
                                    format!("{:?}", notification.kind).to_lowercase(),
                                    notification.title.clone(),
                                    notification.created_at.format("%Y-%m-%d %H:%M").to_string(),
                                ]
                            })
                            .collect();

                        print_table(vec!["", "Kind", "Title", "Received"], data);
                    }
                }
                Ok(())
            }
            Err(error) => {
                print_error(&format!("Failed to list notifications: {}", error));
                Ok(())
            }
        },
        NotificationCommands::MarkRead { id } => {
            let result = if let Some(notification_id) = id {
                client
                    .mark_notification_read(&notification_id)
                    .await
                    .map(|_| 1_u64)
            } else {
                client
                    .mark_all_notifications_read()
                    .await
                    .map(|response| response.updated)
            };

            match result {
                Ok(updated) => {
                    print_success(&format!("Marked {} notification(s) as read.", updated));
                    Ok(())
                }
                Err(error) => {
                    print_error(&format!("Failed to mark notifications as read: {}", error));
                    Ok(())
                }
            }
        }
        NotificationCommands::Watch => {
            println!("{}", "Starting notification watcher...".cyan());
            println!("Press Ctrl+C to exit.\n");
            println!(
                "{}",
                "WebSocket notification support coming soon...".yellow()
            );
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
