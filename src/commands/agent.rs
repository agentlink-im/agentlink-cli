use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;

use crate::api::ApiClient;
use crate::config::Config;
use crate::utils::output::{print_error, print_success};

#[derive(Subcommand)]
pub enum AgentCommands {
    /// 查看当前 Agent 状态
    Status,

    /// 更新 Agent 状态
    SetStatus {
        /// 状态
        status: String,

        /// 当前负载
        #[arg(short, long)]
        load: Option<i32>,

        /// 最大容量
        #[arg(short, long)]
        capacity: Option<i32>,
    },

    /// 查看 Agent 统计
    Stats,

    /// 列出服务
    Services,

    /// 添加服务
    AddService {
        /// 服务名称
        name: String,

        /// 价格
        price: i64,

        /// 单位
        unit: String,

        /// 描述
        #[arg(short, long)]
        description: Option<String>,
    },
}

pub async fn execute(
    command: AgentCommands,
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
        AgentCommands::Status => {
            match client.get_agent_stats().await {
                Ok(stats) => {
                    match format {
                        crate::OutputFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&stats)?);
                        }
                        _ => {
                            println!("\n{}:\n", "Agent Status".bold().underline());
                            if let Some(status) = stats.get("availabilityStatus") {
                                println!("{}: {}", "Status".bold(), status);
                            }
                            if let Some(load) = stats.get("currentLoad") {
                                println!("{}: {}", "Current Load".bold(), load);
                            }
                            if let Some(capacity) = stats.get("maxCapacity") {
                                println!("{}: {}", "Max Capacity".bold(), capacity);
                            }
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    print_error(&format!("Failed to get agent status: {}", e));
                    Ok(())
                }
            }
        }

        AgentCommands::SetStatus { status, load, capacity } => {
            let body = serde_json::json!({
                "availabilityStatus": status,
                "currentLoad": load,
                "maxCapacity": capacity,
            });

            match client.update_agent_status(body).await {
                Ok(_) => {
                    print_success("Agent status updated!");
                    Ok(())
                }
                Err(e) => {
                    print_error(&format!("Failed to update agent status: {}", e));
                    Ok(())
                }
            }
        }

        AgentCommands::Stats => {
            match client.get_agent_stats().await {
                Ok(stats) => {
                    match format {
                        crate::OutputFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&stats)?);
                        }
                        _ => {
                            println!("\n{}:\n", "Agent Statistics".bold().underline());
                            if let Some(completed) = stats.get("totalTasksCompleted") {
                                println!("{}: {}", "Tasks Completed".bold(), completed);
                            }
                            if let Some(success_rate) = stats.get("successRate") {
                                println!("{}: {}%", "Success Rate".bold(), success_rate);
                            }
                            if let Some(response_time) = stats.get("avgResponseTime") {
                                println!("{}: {} min", "Avg Response Time".bold(), response_time);
                            }
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    print_error(&format!("Failed to get agent stats: {}", e));
                    Ok(())
                }
            }
        }

        AgentCommands::Services => {
            match client.get::<Vec<serde_json::Value>>("/api/v1/agents/me/services").await {
                Ok(services) => {
                    if services.is_empty() {
                        println!("{}", "No services found.".yellow());
                        return Ok(());
                    }

                    match format {
                        crate::OutputFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&services)?);
                        }
                        _ => {
                            println!("\n{}:\n", "Services".bold().underline());
                            for service in services {
                                println!("• {} - {} {}/{}",
                                    service["name"].as_str().unwrap_or("Unknown"),
                                    service["price"].as_i64().unwrap_or(0),
                                    service["currency"].as_str().unwrap_or("CNY"),
                                    service["unit"].as_str().unwrap_or("unit")
                                );
                            }
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    print_error(&format!("Failed to list services: {}", e));
                    Ok(())
                }
            }
        }

        AgentCommands::AddService { name, price, unit, description } => {
            let body = serde_json::json!({
                "name": name,
                "price": price,
                "unit": unit,
                "description": description,
            });

            match client.post::<serde_json::Value, _>("/api/v1/agents/me/services", Some(body)).await {
                Ok(response) => {
                    print_success("Service added!");
                    println!("  {}: {}", "ID".bold(), response["id"]);
                    Ok(())
                }
                Err(e) => {
                    print_error(&format!("Failed to add service: {}", e));
                    Ok(())
                }
            }
        }
    }
}
