use anyhow::{Context, Result};
use clap::Subcommand;
use colored::Colorize;

use crate::config::Config;
use crate::utils::output::{print_error, print_success};

#[derive(Subcommand)]
pub enum WebhookCommands {
    /// 配置 Webhook 转发地址
    ///
    /// CLI 通过 poll 接收到的所有 WebSocket 事件，
    /// 会实时 HTTP POST 到该地址。
    Set {
        /// Webhook URL（如 http://localhost:3000/hooks/agentlink）
        url: String,
    },

    /// 查看当前 Webhook 配置
    Get,

    /// 删除 Webhook 配置
    Delete,

    /// 发送测试事件到 Webhook
    Test {
        /// 事件类型
        #[arg(short, long, default_value = "message.created")]
        event: String,
    },
}

pub async fn execute(command: WebhookCommands, config: &mut Config) -> Result<()> {
    match command {
        WebhookCommands::Set { url } => {
            config.webhook_url = Some(url.clone());
            config.save()?;
            print_success("Webhook URL configured.");
            println!("{}: {}", "URL".bold(), url);
            println!(
                "{}",
                "Run `agentlink poll start` to begin forwarding events."
                    .dimmed()
            );
            Ok(())
        }
        WebhookCommands::Get => {
            match &config.webhook_url {
                Some(url) => {
                    println!("{}: {}", "Webhook URL".bold(), url);
                    println!(
                        "{}",
                        "Events received via poll will be forwarded to this URL."
                            .dimmed()
                    );
                }
                None => {
                    println!(
                        "{}",
                        "No webhook URL configured.".yellow()
                    );
                    println!(
                        "{}",
                        "Run `agentlink webhook set <url>` to configure.".dimmed()
                    );
                }
            }
            Ok(())
        }
        WebhookCommands::Delete => {
            config.webhook_url = None;
            config.save()?;
            print_success("Webhook URL removed.");
            Ok(())
        }
        WebhookCommands::Test { event } => {
            let url = match &config.webhook_url {
                Some(url) => url.clone(),
                None => {
                    print_error("No webhook URL configured. Run `agentlink webhook set <url>` first.");
                    return Ok(());
                }
            };

            let payload = format!(
                r#"{{"event":"{}","timestamp":"{}","payload":{{"test":true}}}}"#,
                event,
                chrono::Utc::now().to_rfc3339()
            );

            match forward_to_webhook(&url, &payload).await {
                Ok(()) => {
                    print_success("Test event delivered.");
                }
                Err(e) => {
                    print_error(&format!("Failed to deliver test event: {}", e));
                }
            }
            Ok(())
        }
    }
}

/// 将事件 JSON POST 到 webhook URL
pub async fn forward_to_webhook(url: &str, payload: &str) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .context("Failed to create HTTP client")?;

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("User-Agent", format!("agentlink-cli/{} webhook-forwarder", env!("CARGO_PKG_VERSION")))
        .body(payload.to_string())
        .send()
        .await
        .context("Failed to send webhook request")?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Webhook returned {}: {}", status, body);
    }

    Ok(())
}
