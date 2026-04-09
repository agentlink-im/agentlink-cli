use anyhow::{Context, Result};
use clap::Subcommand;
use colored::Colorize;

use crate::api::ApiClient;
use crate::config::Config;
use crate::utils::output::{print_success, print_user_info, print_warning};

#[derive(Subcommand)]
pub enum ApiKeyCommands {
    /// 保存 Agent API Key 到本地配置
    Set {
        /// 直接提供 API Key；未提供时会交互式输入
        value: Option<String>,
    },

    /// 查看当前 API Key 摘要
    Show,

    /// 清除本地保存的 API Key
    Clear,

    /// 校验当前 API Key 并显示当前 agent 身份
    Verify,
}

pub async fn execute(command: ApiKeyCommands, config: &mut Config) -> Result<()> {
    match command {
        ApiKeyCommands::Set { value } => {
            let value = match value {
                Some(value) => value,
                None => dialoguer::Password::new()
                    .with_prompt("Enter Agent API key")
                    .interact()
                    .context("Failed to read API key")?,
            };

            config.set_api_key(value)?;
            config.save()?;

            print_success("Agent API key saved.");
            println!(
                "{}: {}",
                "Config Path".bold(),
                config.current_config_path()?.display()
            );
            Ok(())
        }
        ApiKeyCommands::Show => {
            println!("{}", "Agent API Key".bold().underline());
            println!();
            println!(
                "{}: {}",
                "Saved".bold(),
                config
                    .saved_api_key_preview()
                    .unwrap_or_else(|| "Not set".to_string())
            );
            println!(
                "{}: {}",
                "Runtime Override".bold(),
                config
                    .runtime_api_key_preview()
                    .unwrap_or_else(|| "Not set".to_string())
            );
            println!(
                "{}: {}",
                "Config Path".bold(),
                config.current_config_path()?.display()
            );
            Ok(())
        }
        ApiKeyCommands::Clear => {
            if config.api_key.is_none() {
                print_warning("No persisted agent API key found.");
                return Ok(());
            }

            config.clear_api_key();
            config.save()?;
            print_success("Agent API key cleared.");
            Ok(())
        }
        ApiKeyCommands::Verify => {
            config.require_api_key()?;

            let client = ApiClient::new(config)?;
            let user = client.verify_agent_identity().await?;

            print_success("Agent API key is valid.");
            print_user_info(&user);
            Ok(())
        }
    }
}
