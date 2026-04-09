use anyhow::Result;
use clap::{Args, Subcommand, ValueEnum};
use colored::Colorize;

use agentlink_protocol::agent::{AgentWorkspaceResponse, CreateServiceRequest};

use crate::api::ApiClient;
use crate::config::Config;
use crate::utils::output::{print_error, print_success};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum AvailabilityArg {
    Available,
    Unavailable,
}

impl AvailabilityArg {
    fn as_bool(self) -> bool {
        matches!(self, Self::Available)
    }
}

#[derive(Args, Clone, Debug, Default)]
pub(crate) struct AgentTargetArgs {
    /// 指定 agent_id；默认使用当前 API Key 对应的 agent
    #[arg(long)]
    agent_id: Option<String>,
}

#[derive(Subcommand)]
pub enum AgentCommands {
    /// 查看当前 agent 工作台状态
    Status(AgentTargetArgs),

    /// 设置当前 agent 可用性
    SetAvailability {
        #[command(flatten)]
        target: AgentTargetArgs,

        #[arg(value_enum)]
        status: AvailabilityArg,
    },

    /// 查看当前 agent 统计
    Stats(AgentTargetArgs),

    /// 列出当前 agent 服务
    Services(AgentTargetArgs),

    /// 为当前 agent 添加服务
    AddService {
        #[command(flatten)]
        target: AgentTargetArgs,

        name: String,

        #[arg(long)]
        price: Option<f64>,

        #[arg(long)]
        currency: Option<String>,

        #[arg(long)]
        days: Option<i32>,

        #[arg(short, long)]
        description: Option<String>,
    },
}

pub async fn execute(
    command: AgentCommands,
    config: &Config,
    format: crate::OutputFormat,
) -> Result<()> {
    ensure_agent_authenticated(config)?;
    let client = ApiClient::new(config)?;

    match command {
        AgentCommands::Status(target) => {
            match client.get_agent_workspace(target.agent_id.as_deref()).await {
                Ok(workspace) => {
                    match format {
                        crate::OutputFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&workspace)?);
                        }
                        crate::OutputFormat::Yaml => {
                            println!("{}", serde_yaml::to_string(&workspace)?);
                        }
                        _ => print_agent_status(&workspace),
                    }
                    Ok(())
                }
                Err(error) => {
                    print_error(&format!("Failed to get agent status: {}", error));
                    Ok(())
                }
            }
        }
        AgentCommands::SetAvailability { target, status } => {
            let agent_id = client.resolve_agent_id(target.agent_id.as_deref()).await?;

            match client
                .update_agent_availability(&agent_id, status.as_bool())
                .await
            {
                Ok(_) => {
                    print_success("Agent availability updated.");
                    Ok(())
                }
                Err(error) => {
                    print_error(&format!("Failed to update availability: {}", error));
                    Ok(())
                }
            }
        }
        AgentCommands::Stats(target) => {
            match client.get_agent_workspace(target.agent_id.as_deref()).await {
                Ok(workspace) => {
                    match format {
                        crate::OutputFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&workspace.access_stats)?);
                        }
                        crate::OutputFormat::Yaml => {
                            println!("{}", serde_yaml::to_string(&workspace.access_stats)?);
                        }
                        _ => print_agent_stats(&workspace),
                    }
                    Ok(())
                }
                Err(error) => {
                    print_error(&format!("Failed to get agent stats: {}", error));
                    Ok(())
                }
            }
        }
        AgentCommands::Services(target) => {
            match client.get_agent_workspace(target.agent_id.as_deref()).await {
                Ok(workspace) => {
                    if workspace.services.is_empty() {
                        println!("{}", "No services found.".yellow());
                        return Ok(());
                    }

                    match format {
                        crate::OutputFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&workspace.services)?);
                        }
                        crate::OutputFormat::Yaml => {
                            println!("{}", serde_yaml::to_string(&workspace.services)?);
                        }
                        _ => print_services(&workspace),
                    }
                    Ok(())
                }
                Err(error) => {
                    print_error(&format!("Failed to list services: {}", error));
                    Ok(())
                }
            }
        }
        AgentCommands::AddService {
            target,
            name,
            price,
            currency,
            days,
            description,
        } => {
            let agent_id = client.resolve_agent_id(target.agent_id.as_deref()).await?;
            let body = CreateServiceRequest {
                name,
                description,
                price: price.and_then(rust_decimal::Decimal::from_f64_retain),
                currency,
                delivery_days: days,
            };

            match client.create_agent_service(&agent_id, body).await {
                Ok(_) => {
                    print_success("Service added.");
                    Ok(())
                }
                Err(error) => {
                    print_error(&format!("Failed to add service: {}", error));
                    Ok(())
                }
            }
        }
    }
}

fn ensure_agent_authenticated(config: &Config) -> Result<()> {
    config.require_api_key().map(|_| ())
}

fn print_agent_status(workspace: &AgentWorkspaceResponse) {
    println!("\n{}:\n", "Agent Status".bold().underline());
    println!("{}: {}", "Agent ID".bold(), workspace.agent.id);
    println!("{}: {}", "LinkID".bold(), workspace.agent.linkid);
    println!(
        "{}: {}",
        "Available".bold(),
        if workspace.agent.is_available {
            "yes"
        } else {
            "no"
        }
    );
    println!(
        "{}: {}",
        "Display Name".bold(),
        workspace.agent.display_name.clone().unwrap_or_default()
    );
    println!(
        "{}: {}",
        "Specialty".bold(),
        workspace.agent.specialty.clone().unwrap_or_default()
    );
    println!(
        "{}: {}",
        "API Key".bold(),
        workspace
            .api_key
            .api_key_preview
            .clone()
            .unwrap_or_else(|| "Not configured".to_string())
    );
}

fn print_agent_stats(workspace: &AgentWorkspaceResponse) {
    println!("\n{}:\n", "Agent Statistics".bold().underline());
    println!("{}: {}", "Rating".bold(), workspace.agent.rating);
    println!(
        "{}: {}",
        "Completed Tasks".bold(),
        workspace.agent.completed_tasks
    );
    println!("{}: {}", "Services".bold(), workspace.services.len());
    println!("{}: {}", "Expertise".bold(), workspace.expertise.len());
    println!("{}: {}", "Works".bold(), workspace.works.len());
    println!(
        "{}: {}",
        "Requests 24h".bold(),
        workspace.access_stats.requests_24h
    );
    println!(
        "{}: {}",
        "Requests 7d".bold(),
        workspace.access_stats.requests_7d
    );
    println!(
        "{}: {}",
        "Avg Response (ms)".bold(),
        workspace
            .access_stats
            .avg_response_time_ms
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string())
    );
}

fn print_services(workspace: &AgentWorkspaceResponse) {
    println!("\n{}:\n", "Services".bold().underline());
    for service in &workspace.services {
        let price = service
            .price
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string());
        let currency = service.currency.clone().unwrap_or_else(|| "-".to_string());
        let days = service
            .delivery_days
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string());

        println!(
            "• {} | price: {} {} | delivery_days: {} | active: {}",
            service.name,
            price,
            currency,
            days,
            if service.is_active { "yes" } else { "no" }
        );
    }
}
