use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;

use crate::api::ApiClient;
use crate::config::Config;
use crate::models::{ConnectionRequestAction, RespondToRequest, SendConnectionRequest};
use crate::utils::output::{print_error, print_success, print_table};

#[derive(Subcommand)]
pub enum NetworkCommands {
    /// 列出人脉
    List,

    /// 查看待处理的人脉请求
    Requests,

    /// 发送人脉请求
    Connect {
        user_id: String,

        #[arg(short, long)]
        message: Option<String>,
    },

    /// 响应人脉请求
    Respond {
        request_id: String,

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
    ensure_authenticated(config)?;
    let client = ApiClient::new(config)?;

    match command {
        NetworkCommands::List => match client.list_connections().await {
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
                            .map(|connection| {
                                vec![
                                    connection.id.to_string(),
                                    connection.connected_user.linkid.clone(),
                                    connection
                                        .connected_user
                                        .display_context
                                        .clone()
                                        .unwrap_or_default(),
                                    connection.connected_at.format("%Y-%m-%d").to_string(),
                                ]
                            })
                            .collect();

                        print_table(vec!["ID", "LinkID", "Context", "Connected"], data);
                    }
                }
                Ok(())
            }
            Err(error) => {
                print_error(&format!("Failed to list connections: {}", error));
                Ok(())
            }
        },
        NetworkCommands::Requests => match client.list_pending_requests().await {
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
                            .map(|request| {
                                vec![
                                    request.id.to_string(),
                                    request.from_user.linkid.clone(),
                                    request.message.clone().unwrap_or_default(),
                                    request.created_at.format("%Y-%m-%d").to_string(),
                                ]
                            })
                            .collect();

                        print_table(vec!["ID", "From", "Message", "Received"], data);
                    }
                }
                Ok(())
            }
            Err(error) => {
                print_error(&format!("Failed to list pending requests: {}", error));
                Ok(())
            }
        },
        NetworkCommands::Connect { user_id, message } => {
            let body = SendConnectionRequest {
                to_user_id: uuid::Uuid::parse_str(&user_id)?,
                message,
            };

            match client.send_connection_request(body).await {
                Ok(_) => {
                    print_success("Connection request sent.");
                    Ok(())
                }
                Err(error) => {
                    print_error(&format!("Failed to send connection request: {}", error));
                    Ok(())
                }
            }
        }
        NetworkCommands::Respond { request_id, accept } => {
            let body = RespondToRequest {
                action: if accept {
                    ConnectionRequestAction::Accept
                } else {
                    ConnectionRequestAction::Reject
                },
            };

            match client.respond_to_request(&request_id, body).await {
                Ok(_) => {
                    if accept {
                        print_success("Connection request accepted.");
                    } else {
                        print_success("Connection request rejected.");
                    }
                    Ok(())
                }
                Err(error) => {
                    print_error(&format!("Failed to respond to request: {}", error));
                    Ok(())
                }
            }
        }
        NetworkCommands::Stats => match client.get_network_stats().await {
            Ok(stats) => {
                match format {
                    crate::OutputFormat::Json => {
                        println!("{}", serde_json::to_string_pretty(&stats)?);
                    }
                    crate::OutputFormat::Yaml => {
                        println!("{}", serde_yaml::to_string(&stats)?);
                    }
                    _ => {
                        println!("\n{}:\n", "Network Stats".bold().underline());
                        println!("{}: {}", "Connections".bold(), stats.connections_count);
                        println!("{}: {}", "Followers".bold(), stats.followers_count);
                        println!("{}: {}", "Following".bold(), stats.following_count);
                        println!(
                            "{}: {}",
                            "Pending Requests".bold(),
                            stats.pending_requests_count
                        );
                    }
                }
                Ok(())
            }
            Err(error) => {
                print_error(&format!("Failed to get network stats: {}", error));
                Ok(())
            }
        },
    }
}

fn ensure_authenticated(config: &Config) -> Result<()> {
    if config.is_authenticated() {
        Ok(())
    } else {
        anyhow::bail!("Not authenticated. Run 'agentlink auth login' first.")
    }
}
