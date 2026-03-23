use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;

use crate::api::ApiClient;
use crate::config::Config;
use crate::utils::output::{print_error, print_success, print_table};

#[derive(Subcommand)]
pub enum NetworkCommands {
    /// 列出人脉
    List,

    /// 查看待处理的人脉请求
    Requests,

    /// 发送人脉请求
    Connect {
        /// 用户 ID
        user_id: String,

        /// 附言
        #[arg(short, long)]
        message: Option<String>,
    },

    /// 响应人脉请求
    Respond {
        /// 请求 ID
        request_id: String,

        /// 接受请求
        #[arg(short, long)]
        accept: bool,
    },

    /// 查看人脉统计
    Stats,
}

pub async fn execute(
    command: NetworkCommands,
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
        NetworkCommands::List => {
            match client.list_connections().await {
                Ok(connections) => {
                    if connections.is_empty() {
                        println!("{}", "No connections found.".yellow());
                        return Ok(());
                    }

                    match format {
                        crate::OutputFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&connections)?);
                        }
                        crate::OutputFormat::Yaml => {
                            println!("{}", serde_yaml::to_string(&connections)?);
                        }
                        _ => {
                            println!("\n{}:\n", "Connections".bold().underline());

                            let data: Vec<Vec<String>> = connections
                                .iter()
                                .map(|c| {
                                    vec![
                                        c.id.clone(),
                                        c.display_name.clone(),
                                        c.user_type.clone(),
                                        c.connected_at.format("%Y-%m-%d").to_string(),
                                    ]
                                })
                                .collect();

                            print_table(
                                vec!["ID", "Name", "Type", "Connected"],
                                data,
                            );
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    print_error(&format!("Failed to list connections: {}", e));
                    Ok(())
                }
            }
        }

        NetworkCommands::Requests => {
            match client.list_pending_requests().await {
                Ok(requests) => {
                    if requests.is_empty() {
                        println!("{}", "No pending requests.".green());
                        return Ok(());
                    }

                    match format {
                        crate::OutputFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&requests)?);
                        }
                        crate::OutputFormat::Yaml => {
                            println!("{}", serde_yaml::to_string(&requests)?);
                        }
                        _ => {
                            println!("\n{}:\n", "Pending Requests".bold().underline());

                            let data: Vec<Vec<String>> = requests
                                .iter()
                                .map(|r| {
                                    vec![
                                        r.id.clone(),
                                        r.from_user.display_name.clone(),
                                        r.message.clone().unwrap_or_default(),
                                        r.created_at.format("%Y-%m-%d").to_string(),
                                    ]
                                })
                                .collect();

                            print_table(
                                vec!["ID", "From", "Message", "Received"],
                                data,
                            );
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    print_error(&format!("Failed to list pending requests: {}", e));
                    Ok(())
                }
            }
        }

        NetworkCommands::Connect { user_id, message } => {
            let body = serde_json::json!({
                "toUserId": user_id,
                "message": message,
            });

            match client.send_connection_request(body).await {
                Ok(_) => {
                    print_success("Connection request sent!");
                    Ok(())
                }
                Err(e) => {
                    print_error(&format!("Failed to send connection request: {}", e));
                    Ok(())
                }
            }
        }

        NetworkCommands::Respond { request_id, accept } => {
            let body = serde_json::json!({
                "accept": accept,
            });

            match client.respond_to_request(&request_id, body).await {
                Ok(_) => {
                    if accept {
                        print_success("Connection request accepted!");
                    } else {
                        print_success("Connection request rejected.");
                    }
                    Ok(())
                }
                Err(e) => {
                    print_error(&format!("Failed to respond to request: {}", e));
                    Ok(())
                }
            }
        }

        NetworkCommands::Stats => {
            match client.get::<serde_json::Value>("/api/v1/network/stats").await {
                Ok(stats) => {
                    match format {
                        crate::OutputFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&stats)?);
                        }
                        _ => {
                            println!("\n{}:\n", "Network Stats".bold().underline());
                            println!("{}: {}", "Connections".bold(), stats["connectionsCount"]);
                            println!("{}: {}", "Followers".bold(), stats["followersCount"]);
                            println!("{}: {}", "Following".bold(), stats["followingCount"]);
                            println!("{}: {}", "Pending Requests".bold(), stats["pendingRequestsCount"]);
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    print_error(&format!("Failed to get network stats: {}", e));
                    Ok(())
                }
            }
        }
    }
}
