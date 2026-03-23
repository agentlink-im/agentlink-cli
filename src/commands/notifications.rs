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
        /// 仅显示未读通知
        #[arg(short, long)]
        unread: bool,
    },

    /// 标记通知为已读
    MarkRead {
        /// 通知 ID（不指定则标记所有为已读）
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
    if !config.is_authenticated() {
        print_error("You must be logged in to use this command.");
        println!("Run {} to authenticate.", "agentlink auth login".cyan());
        return Ok(());
    }

    let client = ApiClient::new(config)?;

    match command {
        NotificationCommands::List { unread } => {
            match client.list_notifications(unread).await {
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
                                .map(|n| {
                                    let status = if n.is_read {
                                        "✓".green().to_string()
                                    } else {
                                        "○".yellow().to_string()
                                    };

                                    vec![
                                        status,
                                        n.notification_type.clone(),
                                        n.title.clone(),
                                        n.created_at.format("%Y-%m-%d %H:%M").to_string(),
                                    ]
                                })
                                .collect();

                            print_table(
                                vec!["", "Type", "Title", "Received"],
                                data,
                            );

                            let unread_count = notifications.iter().filter(|n| !n.is_read).count();
                            if unread_count > 0 {
                                println!("\n{} unread notifications", unread_count.to_string().yellow());
                            }
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    print_error(&format!("Failed to list notifications: {}", e));
                    Ok(())
                }
            }
        }

        NotificationCommands::MarkRead { id } => {
            if let Some(notification_id) = id {
                match client.mark_notification_read(&notification_id).await {
                    Ok(_) => {
                        print_success("Notification marked as read.");
                        Ok(())
                    }
                    Err(e) => {
                        print_error(&format!("Failed to mark notification as read: {}", e));
                        Ok(())
                    }
                }
            } else {
                // 标记所有为已读
                match client.list_notifications(true).await {
                    Ok(notifications) => {
                        let mut success_count = 0;
                        for n in notifications {
                            if let Ok(_) = client.mark_notification_read(&n.id).await {
                                success_count += 1;
                            }
                        }
                        print_success(&format!("Marked {} notifications as read.", success_count));
                        Ok(())
                    }
                    Err(e) => {
                        print_error(&format!("Failed to get notifications: {}", e));
                        Ok(())
                    }
                }
            }
        }

        NotificationCommands::Watch => {
            println!("{}", "Starting notification watcher...".cyan());
            println!("Press Ctrl+C to exit.\n");

            // TODO: 实现 WebSocket 监听通知
            println!("{}", "WebSocket notification support coming soon...".yellow());

            Ok(())
        }
    }
}
