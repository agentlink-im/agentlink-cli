use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::{Args, Subcommand, ValueEnum};
use colored::Colorize;

use agentlink_protocol::agent::{
    AgentSummaryResponse, AgentWorkspaceResponse, CreateServiceRequest,
};
use agentlink_protocol::agent_api_key::AgentApiKeyStatsResponse;
use agentlink_protocol::agent_api_key::{CreateAgentApiKeyRequest, UpdateAgentApiKeyRequest};

use crate::api::ApiClient;
use crate::config::Config;
use crate::utils::output::{print_error, print_success, print_table};

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

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum KeyStateArg {
    Active,
    Inactive,
}

impl KeyStateArg {
    fn as_bool(self) -> bool {
        matches!(self, Self::Active)
    }
}

#[derive(Args, Clone, Debug, Default)]
pub(crate) struct AgentTargetArgs {
    /// 指定 agent_id；未提供时会在仅有一个 Agent 的情况下自动选择
    #[arg(long)]
    agent_id: Option<String>,
}

#[derive(Subcommand)]
pub enum AgentApiKeyCommands {
    /// 查看主 API Key 摘要
    Show(AgentTargetArgs),

    /// 创建或重置主 API Key
    Reset {
        #[command(flatten)]
        target: AgentTargetArgs,

        #[arg(long)]
        name: Option<String>,

        /// 逗号分隔权限列表，默认 all
        #[arg(long)]
        permissions: Option<String>,

        #[arg(long = "rate-limit-per-minute", default_value_t = 100)]
        rate_limit_per_minute: i32,

        /// RFC3339 时间，如 2026-12-31T23:59:59Z
        #[arg(long)]
        expires_at: Option<String>,
    },

    /// 更新主 API Key 配置
    Update {
        #[command(flatten)]
        target: AgentTargetArgs,

        #[arg(long)]
        name: Option<String>,

        /// 逗号分隔权限列表
        #[arg(long)]
        permissions: Option<String>,

        #[arg(long = "rate-limit-per-minute")]
        rate_limit_per_minute: Option<i32>,

        #[arg(long, value_enum)]
        state: Option<KeyStateArg>,

        /// RFC3339 时间，如 2026-12-31T23:59:59Z
        #[arg(long)]
        expires_at: Option<String>,
    },

    /// 撤销主 API Key
    Revoke(AgentTargetArgs),

    /// 查看主 API Key 统计
    Stats(AgentTargetArgs),
}

#[derive(Subcommand)]
pub enum AgentCommands {
    /// 列出当前用户拥有的 Agents
    List,

    /// 查看当前 Agent 工作台状态
    Status(AgentTargetArgs),

    /// 设置当前 Agent 可用性
    SetAvailability {
        #[command(flatten)]
        target: AgentTargetArgs,

        #[arg(value_enum)]
        status: AvailabilityArg,
    },

    /// 查看当前 Agent 统计
    Stats(AgentTargetArgs),

    /// 列出当前 Agent 服务
    Services(AgentTargetArgs),

    /// 为当前 Agent 添加服务
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

    /// 管理主 API Key
    ApiKey {
        #[command(subcommand)]
        command: AgentApiKeyCommands,
    },
}

pub async fn execute(
    command: AgentCommands,
    config: &Config,
    format: crate::OutputFormat,
) -> Result<()> {
    ensure_user_authenticated(config)?;
    let client = ApiClient::new(config)?;

    match command {
        AgentCommands::List => match client.list_my_owned_agents().await {
            Ok(agents) => {
                if agents.is_empty() {
                    println!("{}", "No owned agents found.".yellow());
                    return Ok(());
                }

                match format {
                    crate::OutputFormat::Json => {
                        println!("{}", serde_json::to_string_pretty(&agents)?);
                    }
                    crate::OutputFormat::Yaml => {
                        println!("{}", serde_yaml::to_string(&agents)?);
                    }
                    _ => print_agent_list(&agents),
                }
                Ok(())
            }
            Err(error) => {
                print_error(&format!("Failed to list agents: {}", error));
                Ok(())
            }
        },
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
        AgentCommands::ApiKey { command } => execute_api_key(command, &client, format).await,
    }
}

async fn execute_api_key(
    command: AgentApiKeyCommands,
    client: &ApiClient,
    format: crate::OutputFormat,
) -> Result<()> {
    match command {
        AgentApiKeyCommands::Show(target) => {
            match client
                .get_primary_agent_api_key(target.agent_id.as_deref())
                .await
            {
                Ok(response) => {
                    match format {
                        crate::OutputFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&response)?);
                        }
                        crate::OutputFormat::Yaml => {
                            println!("{}", serde_yaml::to_string(&response)?);
                        }
                        _ => {
                            println!("\n{}:\n", "Primary API Key".bold().underline());
                            println!("{}: {}", "Has Key".bold(), response.has_key);
                            println!("{}: {}", "Agent ID".bold(), response.agent_id);
                            println!(
                                "{}: {}",
                                "Preview".bold(),
                                response
                                    .api_key_preview
                                    .clone()
                                    .unwrap_or_else(|| "-".to_string())
                            );
                            println!(
                                "{}: {}",
                                "Permissions".bold(),
                                response
                                    .permissions
                                    .map(|permissions| permissions.join(", "))
                                    .unwrap_or_else(|| "-".to_string())
                            );
                            println!(
                                "{}: {}",
                                "Active".bold(),
                                response
                                    .is_active
                                    .map(|value| if value { "yes" } else { "no" })
                                    .unwrap_or("-")
                            );
                            println!(
                                "{}: {}",
                                "Rate Limit".bold(),
                                response
                                    .rate_limit_per_minute
                                    .map(|value| value.to_string())
                                    .unwrap_or_else(|| "-".to_string())
                            );
                        }
                    }
                    Ok(())
                }
                Err(error) => {
                    print_error(&format!("Failed to get API key: {}", error));
                    Ok(())
                }
            }
        }
        AgentApiKeyCommands::Reset {
            target,
            name,
            permissions,
            rate_limit_per_minute,
            expires_at,
        } => {
            let body = CreateAgentApiKeyRequest {
                name,
                permissions: parse_permissions_or_default(permissions),
                rate_limit_per_minute,
                expires_at: parse_datetime(expires_at)?,
            };

            match client
                .create_or_reset_primary_agent_api_key(target.agent_id.as_deref(), body)
                .await
            {
                Ok(response) => {
                    print_success("Primary API key created or reset.");
                    println!("{}: {}", "Key ID".bold(), response.id);
                    println!("{}: {}", "API Key".bold(), response.api_key);
                    Ok(())
                }
                Err(error) => {
                    print_error(&format!("Failed to reset API key: {}", error));
                    Ok(())
                }
            }
        }
        AgentApiKeyCommands::Update {
            target,
            name,
            permissions,
            rate_limit_per_minute,
            state,
            expires_at,
        } => {
            let body = UpdateAgentApiKeyRequest {
                name,
                permissions: permissions.map(parse_permissions),
                rate_limit_per_minute,
                is_active: state.map(KeyStateArg::as_bool),
                expires_at: parse_datetime(expires_at)?,
            };

            match client
                .update_primary_agent_api_key(target.agent_id.as_deref(), body)
                .await
            {
                Ok(_) => {
                    print_success("Primary API key updated.");
                    Ok(())
                }
                Err(error) => {
                    print_error(&format!("Failed to update API key: {}", error));
                    Ok(())
                }
            }
        }
        AgentApiKeyCommands::Revoke(target) => {
            match client
                .revoke_primary_agent_api_key(target.agent_id.as_deref())
                .await
            {
                Ok(response) => {
                    print_success(&response.message);
                    Ok(())
                }
                Err(error) => {
                    print_error(&format!("Failed to revoke API key: {}", error));
                    Ok(())
                }
            }
        }
        AgentApiKeyCommands::Stats(target) => {
            match client
                .get_primary_agent_api_key_stats(target.agent_id.as_deref())
                .await
            {
                Ok(response) => {
                    match format {
                        crate::OutputFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&response)?);
                        }
                        crate::OutputFormat::Yaml => {
                            println!("{}", serde_yaml::to_string(&response)?);
                        }
                        _ => print_api_key_stats(&response),
                    }
                    Ok(())
                }
                Err(error) => {
                    print_error(&format!("Failed to get API key stats: {}", error));
                    Ok(())
                }
            }
        }
    }
}

fn ensure_user_authenticated(config: &Config) -> Result<()> {
    if config.has_user_token() {
        Ok(())
    } else {
        anyhow::bail!(
            "Agent management commands require a user session. Run 'agentlink auth login' first."
        )
    }
}

fn print_agent_list(agents: &[AgentSummaryResponse]) {
    println!("\n{}:\n", "My Agents".bold().underline());
    let rows = agents
        .iter()
        .map(|agent| {
            vec![
                agent.id.to_string(),
                agent.linkid.clone(),
                agent.display_name.clone().unwrap_or_default(),
                if agent.is_available {
                    "yes".to_string()
                } else {
                    "no".to_string()
                },
                agent.created_at.format("%Y-%m-%d").to_string(),
            ]
        })
        .collect();
    print_table(
        vec!["ID", "LinkID", "Display Name", "Available", "Created"],
        rows,
    );
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

fn print_api_key_stats(response: &AgentApiKeyStatsResponse) {
    println!("\n{}:\n", "API Key Stats".bold().underline());
    println!("{}: {}", "Agent ID".bold(), response.agent_id);
    println!("{}: {}", "Has Stats".bold(), response.has_stats);
    println!(
        "{}: {}",
        "Total Requests".bold(),
        response
            .total_requests
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    println!(
        "{}: {}",
        "Requests 24h".bold(),
        response
            .requests_24h
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    println!(
        "{}: {}",
        "Requests 7d".bold(),
        response
            .requests_7d
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    println!(
        "{}: {}",
        "Avg Response (ms)".bold(),
        response
            .avg_response_time_ms
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    if let Some(message) = &response.message {
        println!("{}: {}", "Message".bold(), message);
    }
}

fn parse_permissions_or_default(permissions: Option<String>) -> Vec<String> {
    permissions
        .map(parse_permissions)
        .filter(|values| !values.is_empty())
        .unwrap_or_else(|| vec!["all".to_string()])
}

fn parse_permissions(value: String) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>()
}

fn parse_datetime(value: Option<String>) -> Result<Option<DateTime<Utc>>> {
    value
        .map(|raw| {
            DateTime::parse_from_rfc3339(&raw)
                .with_context(|| format!("Invalid RFC3339 datetime: {}", raw))
                .map(|datetime| datetime.with_timezone(&Utc))
        })
        .transpose()
}
