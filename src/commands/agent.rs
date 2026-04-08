use anyhow::Result;
use clap::{Subcommand, ValueEnum};
use colored::Colorize;

use crate::api::ApiClient;
use crate::config::Config;
use crate::models::CreateServiceRequest;
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

#[derive(Subcommand)]
pub enum AgentCommands {
    /// 查看当前 Agent 状态
    Status,

    /// 设置当前 Agent 可用性
    SetAvailability {
        #[arg(value_enum)]
        status: AvailabilityArg,
    },

    /// 查看当前 Agent 统计
    Stats,

    /// 列出当前 Agent 服务
    Services,

    /// 为当前 Agent 添加服务
    AddService {
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
    ensure_authenticated(config)?;
    let client = ApiClient::new(config)?;

    match command {
        AgentCommands::Status => match client.get_current_agent_profile().await {
            Ok(profile) => {
                match format {
                    crate::OutputFormat::Json => {
                        println!("{}", serde_json::to_string_pretty(&profile)?);
                    }
                    crate::OutputFormat::Yaml => {
                        println!("{}", serde_yaml::to_string(&profile)?);
                    }
                    _ => {
                        println!("\n{}:\n", "Agent Status".bold().underline());
                        println!("{}: {}", "Agent ID".bold(), profile.user_id);
                        println!("{}: {}", "LinkID".bold(), profile.linkid);
                        println!(
                            "{}: {}",
                            "Available".bold(),
                            if profile.is_available { "yes" } else { "no" }
                        );
                        println!(
                            "{}: {}",
                            "Display Name".bold(),
                            profile.display_name.unwrap_or_default()
                        );
                        println!(
                            "{}: {}",
                            "Specialty".bold(),
                            profile.specialty.unwrap_or_default()
                        );
                    }
                }
                Ok(())
            }
            Err(error) => {
                print_error(&format!("Failed to get agent status: {}", error));
                Ok(())
            }
        },
        AgentCommands::SetAvailability { status } => {
            let user = client.get_current_user().await?;

            match client
                .update_agent_availability(&user.id.to_string(), status.as_bool())
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
        AgentCommands::Stats => match client.get_current_agent_profile().await {
            Ok(profile) => {
                match format {
                    crate::OutputFormat::Json => {
                        println!("{}", serde_json::to_string_pretty(&profile)?);
                    }
                    crate::OutputFormat::Yaml => {
                        println!("{}", serde_yaml::to_string(&profile)?);
                    }
                    _ => {
                        println!("\n{}:\n", "Agent Statistics".bold().underline());
                        println!("{}: {}", "Rating".bold(), profile.rating);
                        println!("{}: {}", "Completed Tasks".bold(), profile.completed_tasks);
                        println!("{}: {}", "Services".bold(), profile.services.len());
                        println!("{}: {}", "Expertise".bold(), profile.expertise.len());
                        println!("{}: {}", "Works".bold(), profile.works.len());
                    }
                }
                Ok(())
            }
            Err(error) => {
                print_error(&format!("Failed to get agent stats: {}", error));
                Ok(())
            }
        },
        AgentCommands::Services => match client.get_current_agent_profile().await {
            Ok(profile) => {
                if profile.services.is_empty() {
                    println!("{}", "No services found.".yellow());
                    return Ok(());
                }

                match format {
                    crate::OutputFormat::Json => {
                        println!("{}", serde_json::to_string_pretty(&profile.services)?);
                    }
                    crate::OutputFormat::Yaml => {
                        println!("{}", serde_yaml::to_string(&profile.services)?);
                    }
                    _ => {
                        println!("\n{}:\n", "Services".bold().underline());
                        for service in profile.services {
                            let price = service
                                .price
                                .map(|value| value.to_string())
                                .unwrap_or_else(|| "-".to_string());
                            let currency = service.currency.unwrap_or_else(|| "-".to_string());
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
                }
                Ok(())
            }
            Err(error) => {
                print_error(&format!("Failed to list services: {}", error));
                Ok(())
            }
        },
        AgentCommands::AddService {
            name,
            price,
            currency,
            days,
            description,
        } => {
            let user = client.get_current_user().await?;
            let body = CreateServiceRequest {
                name,
                description,
                price: price.and_then(rust_decimal::Decimal::from_f64_retain),
                currency,
                delivery_days: days,
            };

            match client
                .create_agent_service(&user.id.to_string(), body)
                .await
            {
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

fn ensure_authenticated(config: &Config) -> Result<()> {
    if config.is_authenticated() {
        Ok(())
    } else {
        anyhow::bail!("Not authenticated. Run 'agentlink auth login' first.")
    }
}
