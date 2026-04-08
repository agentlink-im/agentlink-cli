use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;

use crate::config::Config;

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// 显示当前配置
    Show,

    /// 设置配置项
    Set {
        /// 配置键
        key: String,
        /// 配置值
        value: String,
    },

    /// 获取配置项
    Get {
        /// 配置键
        key: String,
    },

    /// 重置配置为默认值
    Reset,

    /// 显示配置文件路径
    Path,
}

pub async fn execute(command: ConfigCommands, config: &mut Config) -> Result<()> {
    match command {
        ConfigCommands::Show => {
            println!("{}", "Current Configuration:".bold().underline());
            println!();
            println!("{}: {}", "Server URL".bold(), config.server_url);
            println!("{}: {}", "WebSocket URL".bold(), config.websocket_url);
            println!(
                "{}: {}",
                "Auth Token".bold(),
                config
                    .api_key
                    .as_ref()
                    .map(|k| format!("{}****", &k[..8.min(k.len())]))
                    .unwrap_or_else(|| "Not set".to_string())
            );
            println!(
                "{}: {}",
                "Default Output Format".bold(),
                config.defaults.output_format
            );
            println!(
                "{}: {}",
                "Default Page Size".bold(),
                config.defaults.page_size
            );
            Ok(())
        }

        ConfigCommands::Set { key, value } => {
            match key.as_str() {
                "server_url" | "server" => {
                    config.server_url = value;
                    config.save()?;
                    println!("{} Server URL updated.", "✓".green());
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
                    println!("Available keys: server_url, websocket_url, output_format, page_size");
                }
            }
            Ok(())
        }

        ConfigCommands::Get { key } => {
            match key.as_str() {
                "server_url" | "server" => println!("{}", config.server_url),
                "websocket_url" | "ws" => println!("{}", config.websocket_url),
                "output_format" | "format" => println!("{}", config.defaults.output_format),
                "page_size" => println!("{}", config.defaults.page_size),
                _ => {
                    println!("{} Unknown configuration key: {}", "✗".red(), key);
                }
            }
            Ok(())
        }

        ConfigCommands::Reset => {
            let confirm = dialoguer::Confirm::new()
                .with_prompt("Are you sure you want to reset all configuration?")
                .default(false)
                .interact()?;

            if confirm {
                *config = Config::default();
                config.save()?;
                println!("{} Configuration reset to defaults.", "✓".green());
            } else {
                println!("Cancelled.");
            }
            Ok(())
        }

        ConfigCommands::Path => {
            let path = Config::default_config_path()?;
            println!("{}", path.display());
            Ok(())
        }
    }
}
