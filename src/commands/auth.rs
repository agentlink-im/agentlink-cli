use anyhow::{Context, Result};
use clap::Subcommand;
use colored::Colorize;

use crate::api::ApiClient;
use crate::config::Config;
use crate::utils::output::{print_success, print_user_info, print_warning};

#[derive(Subcommand)]
pub enum AuthCommands {
    /// 登录并保存认证 token
    Login {
        /// 直接提供 jwt_/sk_ token
        #[arg(long)]
        token: Option<String>,

        /// 使用邮箱验证码登录
        #[arg(long)]
        email: Option<String>,
    },

    /// 发送邮箱验证码
    SendCode {
        /// 登录邮箱
        email: String,
    },

    /// 查看 onboarding 状态
    OnboardingStatus,

    /// 完成 onboarding，设置 linkid
    CompleteOnboarding {
        /// 3-30 位 linkid
        linkid: String,
    },

    /// 退出登录
    Logout,

    /// 查看当前登录状态
    Whoami,

    /// 验证当前 token 是否有效
    Verify,
}

pub async fn execute(command: AuthCommands, config: &mut Config) -> Result<()> {
    match command {
        AuthCommands::Login { token, email } => login(config, token, email).await,
        AuthCommands::SendCode { email } => {
            let client = ApiClient::new(config)?;
            let response = client
                .send_verification_code(agentlink_protocol::auth::SendVerificationCodeRequest {
                    email,
                })
                .await?;
            print_success(&response.message);
            if let Some(code) = response.code {
                println!("{}: {}", "Verification Code".bold(), code);
            }
            Ok(())
        }
        AuthCommands::OnboardingStatus => {
            ensure_authenticated(config)?;
            let client = ApiClient::new(config)?;
            let response = client.get_onboarding_status().await?;
            println!(
                "{}: {}",
                "Needs Onboarding".bold(),
                if response.needs_onboarding {
                    "yes"
                } else {
                    "no"
                }
            );
            Ok(())
        }
        AuthCommands::CompleteOnboarding { linkid } => {
            ensure_authenticated(config)?;
            let client = ApiClient::new(config)?;
            let response = client
                .complete_onboarding(agentlink_protocol::auth::UpdateLinkidRequest { linkid })
                .await?;
            print_success(&response.message);
            print_user_info(&response.user);
            Ok(())
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
            match client.verify_token().await {
                Ok(user) => {
                    print_user_info(&user);
                    Ok(())
                }
                Err(error) => {
                    println!("{}", format!("Failed to get user info: {}", error).red());
                    println!("Run {} to re-authenticate.", "agentlink auth login".cyan());
                    Ok(())
                }
            }
        }
        AuthCommands::Verify => {
            ensure_authenticated(config)?;
            let client = ApiClient::new(config)?;
            client.verify_token().await?;
            print_success("Token is valid.");
            Ok(())
        }
    }
}

async fn login(config: &mut Config, token: Option<String>, email: Option<String>) -> Result<()> {
    if let Some(token) = token {
        let client = ApiClient::new(config)?.with_auth_token(token.clone());
        let user = client.verify_token().await?;
        config.set_api_key(token);
        config.save()?;
        print_success("Successfully authenticated.");
        print_user_info(&user);
        return Ok(());
    }

    let email = match email {
        Some(email) => email,
        None => dialoguer::Input::new()
            .with_prompt("Enter your email")
            .interact_text()
            .context("Failed to read email")?,
    };

    let client = ApiClient::new(config)?;
    let code_response = client
        .send_verification_code(agentlink_protocol::auth::SendVerificationCodeRequest {
            email: email.clone(),
        })
        .await?;
    print_success(&code_response.message);
    if let Some(code) = code_response.code {
        println!("{}: {}", "Verification Code".bold(), code);
    }

    let code: String = dialoguer::Input::new()
        .with_prompt("Enter verification code")
        .interact_text()
        .context("Failed to read verification code")?;

    let login_response = client
        .magic_login(agentlink_protocol::auth::MagicLoginRequest { email, code })
        .await?;
    config.set_api_key(login_response.token.clone());
    config.save()?;

    print_success("Successfully authenticated.");
    print_user_info(&login_response.user);

    if login_response.needs_onboarding {
        print_warning("Onboarding is still required. Run `agentlink auth onboarding-status` and `agentlink auth complete-onboarding <linkid>`.");
    }

    Ok(())
}

fn ensure_authenticated(config: &Config) -> Result<()> {
    if config.is_authenticated() {
        Ok(())
    } else {
        anyhow::bail!("Not authenticated. Run 'agentlink auth login' first.")
    }
}
