use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;

use crate::api::ApiClient;
use crate::config::Config;

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// 显示当前配置（包括环境变量和配置文件）
    Show,

    /// 设置配置项到配置文件
    /// 
    /// 示例:
    ///   agentlink config set base_url https://api.example.com
    ///   agentlink config set api_key sk_xxx
    Set {
        /// 配置键 (base_url, api_key, websocket_url, output_format, page_size)
        key: String,
        /// 配置值
        value: String,
    },

    /// 获取配置项的值
    Get {
        /// 配置键
        key: String,
    },

    /// 列出所有可用的配置键
    List,

    /// 重置配置为默认值（保留配置文件路径）
    Reset,

    /// 显示配置文件路径
    Path,

    /// 验证当前 API Key 并显示身份信息
    Whoami,
}

pub async fn execute(command: ConfigCommands, config: &mut Config) -> Result<()> {
    match command {
        ConfigCommands::Show => {
            let config_path = config.current_config_path()?;
            
            println!("{}", "Current Configuration:".bold().underline());
            println!();
            println!("{}: {}", "Configuration File".bold(), config_path.display());
            println!();
            println!("{}: {}", "Base URL".bold(), config.server_url);
            println!("{}: {}", "WebSocket URL".bold(), config.websocket_url);
            
            println!();
            println!("{}", "API Keys:".bold());
            println!(
                "  {}: {}",
                "Saved (in config file)".bold(),
                config
                    .saved_api_key_preview()
                    .unwrap_or_else(|| "Not set".to_string())
            );
            
            println!(
                "  {}: {}",
                "Runtime (not saved)".bold(),
                config
                    .runtime_api_key_preview()
                    .unwrap_or_else(|| "Not set".to_string())
            );
            
            println!();
            println!("{}", "Defaults:".bold());
            println!("  {}: {}", "Output Format".bold(), config.defaults.output_format);
            println!("  {}: {}", "Page Size".bold(), config.defaults.page_size);
            Ok(())
        }

        ConfigCommands::Set { key, value } => {
            match key.as_str() {
                "base_url" | "server_url" | "server" => {
                    config.server_url = value;
                    config.save()?;
                    println!("{} Base URL updated.", "✓".green());
                }
                "api_key" => {
                    config.set_api_key(value)?;
                    config.save()?;
                    println!("{} Agent API key updated.", "✓".green());
                }
                "websocket_url" | "ws" => {
                    config.websocket_url = value;
                    config.save()?;
                    println!("{} WebSocket URL updated.", "✓".green());
                }
                "output_format" | "format" => {
                    config.defaults.output_format = value;
                    config.save()?;
                    println!("{} Default output format updated.", "✓".green());
                }
                "page_size" => {
                    config.defaults.page_size = value.parse()?;
                    config.save()?;
                    println!("{} Default page size updated.", "✓".green());
                }
                _ => {
                    println!("{} Unknown configuration key: {}", "✗".red(), key);
                    println!(
                        "Available keys: base_url, api_key, websocket_url, output_format, page_size"
                    );
                }
            }
            Ok(())
        }

        ConfigCommands::Get { key } => {
            match key.as_str() {
                "base_url" | "server_url" | "server" => println!("{}", config.server_url),
                "api_key" => println!(
                    "{}",
                    config
                        .saved_api_key_preview()
                        .unwrap_or_else(|| "Not set".to_string())
                ),
                "websocket_url" | "ws" => println!("{}", config.websocket_url),
                "output_format" | "format" => println!("{}", config.defaults.output_format),
                "page_size" => println!("{}", config.defaults.page_size),
                _ => {
                    println!("{} Unknown configuration key: {}", "✗".red(), key);
                    println!("Run `agentlink config list` to see available keys.");
                }
            }
            Ok(())
        }

        ConfigCommands::List => {
            println!("{}", "Available Configuration Keys:".bold().underline());
            println!();
            println!("{}:", "General".bold());
            println!("  {} - Base API URL", "base_url".cyan());
            println!("  {} - WebSocket URL", "websocket_url".cyan());
            println!();
            println!("{}:", "Authentication".bold());
            println!("  {} - Agent API Key (sk_*)", "api_key".cyan());
            println!();
            println!("{}:", "Defaults".bold());
            println!("  {} - Default output format (table, json, yaml, plain)", "output_format".cyan());
            println!("  {} - Default page size for list commands", "page_size".cyan());
            println!();
            println!("{}", "Configuration Priority:".bold().underline());
            println!("  1. CLI arguments (highest)");
            println!("  2. Config file");
            println!("  3. Default values (lowest)");
            Ok(())
        }

        ConfigCommands::Reset => {
            let confirm = dialoguer::Confirm::new()
                .with_prompt("Are you sure you want to reset all configuration?")
                .default(false)
                .interact()?;

            if confirm {
                config.reset_to_defaults();
                config.save()?;
                println!("{} Configuration reset to defaults.", "✓".green());
            } else {
                println!("Cancelled.");
            }
            Ok(())
        }

        ConfigCommands::Path => {
            let path = config.current_config_path()?;
            println!("{}", path.display());
            Ok(())
        }

        ConfigCommands::Whoami => {
            // 验证 API Key 并获取身份信息
            match config.require_api_key() {
                Ok(api_key) => {
                    println!("{}", "Verifying API key...".dimmed());
                    println!("{}: {}", "Using API Key".dimmed(), mask_api_key(api_key));
                    println!("{}: {}", "Server URL".dimmed(), config.server_url);
                    
                    match ApiClient::new(config) {
                        Ok(client) => {
                            match client.get_current_user().await {
                                Ok(user) => {
                                    println!();
                                    println!("{}", "Authentication Successful".green().bold());
                                    println!();
                                    println!("{}: {}", "User Type".bold(), format!("{:?}", user.user_type).cyan());
                                    println!("{}: {}", "LinkID".bold(), user.linkid);
                                    println!("{}: {}", "Display Name".bold(), user.display_name.unwrap_or_else(|| "N/A".to_string()));
                                    println!("{}: {}", "Verified".bold(), if user.is_verified { "Yes".green() } else { "No".yellow() });
                                    println!();
                                    println!("{}: {}", "API Key".bold(), mask_api_key(api_key));
                                }
                                Err(e) => {
                                    println!();
                                    println!("{}: {}", "Authentication Failed".red().bold(), e);
                                    println!();
                                    println!("{}", "Possible causes:".yellow());
                                    println!("  • API key is invalid or expired");
                                    println!("  • API key has been revoked");
                                    println!("  • Server is unreachable");
                                    println!();
                                    println!("{}", "To fix:".yellow());
                                    println!("  1. Verify your API key: agentlink config get api_key");
                                    println!("  2. Set a new API key: agentlink config set api_key <sk_...>");
                                    println!("  3. Check server URL: agentlink config get base_url");
                                }
                            }
                        }
                        Err(e) => {
                            println!("{}: {}", "Failed to create API client".red(), e);
                        }
                    }
                }
                Err(e) => {
                    println!("{}: {}", "No API key configured".red(), e);
                }
            }
            Ok(())
        }
    }
}

fn mask_api_key(api_key: &str) -> String {
    let visible_len = api_key.len().min(12);
    format!("{}****", &api_key[..visible_len])
}
