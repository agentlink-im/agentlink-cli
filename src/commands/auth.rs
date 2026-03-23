use anyhow::{Context, Result};
use clap::Subcommand;
use colored::Colorize;

use crate::api::ApiClient;
use crate::config::Config;
use crate::utils::output::{print_success, print_user_info};

#[derive(Subcommand)]
pub enum AuthCommands {
    /// 使用 API Key 登录
    Login {
        /// API Key
        #[arg(short, long)]
        api_key: Option<String>,
    },

    /// 退出登录
    Logout,

    /// 查看当前登录状态
    Whoami,

    /// 验证 API Key 是否有效
    Verify,
}

pub async fn execute(command: AuthCommands, config: &mut Config) -> Result<()> {
    match command {
        AuthCommands::Login { api_key } => {
            let api_key = if let Some(key) = api_key {
                key
            } else {
                // 交互式输入
                dialoguer::Password::new()
                    .with_prompt("Enter your API key")
                    .interact()
                    .context("Failed to read API key")?
            };

            // 验证 API Key
            let client = ApiClient::new(config)?.with_api_key(api_key.clone());

            match client.verify_api_key().await {
                Ok(user) => {
                    config.set_api_key(api_key);
                    config.save()?;

                    print_success("Successfully authenticated!");
                    print_user_info(&user);
                    Ok(())
                }
                Err(e) => {
                    anyhow::bail!("Authentication failed: {}", e)
                }
            }
        }

        AuthCommands::Logout => {
            if !config.is_authenticated() {
                println!("{}", "You are not logged in.".yellow());
                return Ok(());
            }

            config.clear_auth();
            config.save()?;

            print_success("Successfully logged out.");
            Ok(())
        }

        AuthCommands::Whoami => {
            if !config.is_authenticated() {
                println!("{}", "You are not logged in.".yellow());
                println!("Run {} to authenticate.", "agentlink auth login".cyan());
                return Ok(());
            }

            let client = ApiClient::new(config)?;

            match client.verify_api_key().await {
                Ok(user) => {
                    print_user_info(&user);
                    Ok(())
                }
                Err(e) => {
                    println!("{}", format!("Failed to get user info: {}", e).red());
                    println!("Your API key may be invalid or expired.");
                    println!("Run {} to re-authenticate.", "agentlink auth login".cyan());
                    Ok(())
                }
            }
        }

        AuthCommands::Verify => {
            if !config.is_authenticated() {
                println!("{}", "You are not logged in.".yellow());
                return Ok(());
            }

            let client = ApiClient::new(config)?;

            match client.verify_api_key().await {
                Ok(_) => {
                    print_success("API key is valid.");
                    Ok(())
                }
                Err(e) => {
                    anyhow::bail!("API key is invalid: {}", e)
                }
            }
        }
    }
}
