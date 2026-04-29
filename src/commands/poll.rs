use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;

use agentlink_rust_sdk::websocket::WsEvent;

use crate::config::Config;
use crate::utils::output::print_error;

#[derive(Subcommand)]
pub enum PollCommands {
    /// 启动 WebSocket 事件流，实时接收平台消息和事件
    ///
    /// 默认以人类可读格式输出。使用 --json 时每行输出一个 JSON 事件，
    /// 便于通过管道接入其他工具处理（如 jq、自定义脚本等）。
    Start {
        /// 以 JSON Lines 格式输出（每行一个 JSON 对象）
        #[arg(long)]
        json: bool,

        /// 仅输出指定类型的事件（可多次指定）
        /// 可选值: connection.ready, message.created, event.notification, error
        #[arg(long = "filter")]
        filters: Vec<String>,

        /// 断开时自动重连（指数退避）
        #[arg(long)]
        reconnect: bool,

        /// 最大重连间隔（秒）
        #[arg(long, default_value = "60")]
        max_backoff: u64,

        /// 收到事件时执行的外部命令（事件 JSON 通过 stdin 传入）
        ///
        /// 示例: --exec "python3 ./handler.py"
        /// 示例: --exec "./notify.sh"
        #[arg(long)]
        exec: Option<String>,

        /// Webhook 转发地址（覆盖配置文件中的设置）
        ///
        /// CLI 会将收到的每个事件实时 HTTP POST 到该地址。
        /// 也可通过 `agentlink webhook set <url>` 持久化配置。
        #[arg(long)]
        webhook_url: Option<String>,
    },
}

pub async fn execute(command: PollCommands, config: &Config) -> Result<()> {
    match command {
        PollCommands::Start {
            json,
            filters,
            reconnect,
            max_backoff,
            exec,
            webhook_url,
        } => {
            // 命令行参数优先，其次配置文件
            let resolved_webhook = webhook_url
                .as_ref()
                .or(config.webhook_url.as_ref())
                .cloned();

            if let Err(error) = run_poll(
                config, json, filters, reconnect, max_backoff, exec, resolved_webhook,
            )
            .await
            {
                print_error(&format!("Poll error: {}", error));
            }
            Ok(())
        }
    }
}

async fn run_poll(
    config: &Config,
    json: bool,
    filters: Vec<String>,
    _reconnect: bool,
    _max_backoff: u64,
    exec_command: Option<String>,
    webhook_url: Option<String>,
) -> anyhow::Result<()> {
    use futures_util::StreamExt;

    let client = config.to_client()?;
    let mut builder = client.websocket();

    if !filters.is_empty() {
        builder = builder.filter_events(filters);
    }

    let mut ws_stream = builder.connect().await?;

    if !json {
        println!("{}", "Starting event poll...".cyan());
        println!("Press Ctrl+C to exit.\n");
    }

    while let Some(event) = ws_stream.next().await {
        let event = match event {
            Ok(e) => e,
            Err(e) => {
                eprintln!("[ERROR] WebSocket error: {}", e);
                continue;
            }
        };

        let event_json = build_event_json(&event);

        // 输出到 stdout
        if json {
            println!("{}", event_json);
        } else {
            print_event_human(&event);
        }

        // 触发外部命令回调
        if let Some(cmd) = &exec_command {
            let cmd = cmd.clone();
            let event_json = event_json.clone();
            tokio::spawn(async move {
                if let Err(e) = run_exec_command(&cmd, &event_json).await {
                    eprintln!("[EXEC ERROR] {}: {}", cmd, e);
                }
            });
        }

        // 转发到 webhook
        if let Some(url) = &webhook_url {
            let url = url.clone();
            let event_json = event_json.clone();
            tokio::spawn(async move {
                if let Err(e) = crate::commands::webhook::forward_to_webhook(&url, &event_json).await
                {
                    eprintln!("[WEBHOOK ERROR] {}", e);
                }
            });
        }
    }

    Ok(())
}

fn build_event_json(event: &WsEvent) -> String {
    let (event_name, payload_json) = match event {
        WsEvent::ConnectionReady(payload) => (
            "connection.ready",
            serde_json::to_string(payload),
        ),
        WsEvent::MessageCreated(payload) => (
            "message.created",
            serde_json::to_string(payload),
        ),
        WsEvent::EventNotification(payload) => (
            "event.notification",
            serde_json::to_string(payload),
        ),
        WsEvent::PresenceChanged(payload) => (
            "user.presence_changed",
            serde_json::to_string(payload),
        ),
        WsEvent::Pong(payload) => ("pong", serde_json::to_string(payload)),
        WsEvent::Error(payload) => ("error", serde_json::to_string(payload)),
        WsEvent::Raw(text) => ("raw", Ok(text.clone())),
    };

    match payload_json {
        Ok(payload) => format!(
            r#"{{"event":"{}","timestamp":"{}","payload":{}}}"#,
            event_name,
            chrono::Utc::now().to_rfc3339(),
            payload
        ),
        Err(_) => format!(
            r#"{{"event":"{}","timestamp":"{}"}}"#,
            event_name,
            chrono::Utc::now().to_rfc3339()
        ),
    }
}

fn print_event_human(event: &WsEvent) {
    match event {
        WsEvent::ConnectionReady(payload) => {
            println!(
                "[{}] [READY] Connected as {}",
                chrono::Utc::now().format("%H:%M:%S"),
                payload.linkid
            );
        }
        WsEvent::MessageCreated(payload) => {
            let msg = &payload.message;
            println!(
                "[{}] [MESSAGE] {}: {}",
                msg.created_at.format("%H:%M:%S"),
                msg.sender_name,
                msg.content
            );
        }
        WsEvent::EventNotification(payload) => {
            println!("[NOTIFICATION] {}", payload.data.title);
            if !payload.data.content.is_empty() {
                println!("  {}", payload.data.content);
            }
        }
        WsEvent::PresenceChanged(payload) => {
            println!(
                "[{}] [PRESENCE] {} is now {:?}",
                chrono::Utc::now().format("%H:%M:%S"),
                payload.linkid,
                payload.status
            );
        }
        WsEvent::Pong(_) => {}
        WsEvent::Error(payload) => {
            eprintln!(
                "[{}] [ERROR] {}: {}",
                chrono::Utc::now().format("%H:%M:%S"),
                payload.code,
                payload.message
            );
        }
        WsEvent::Raw(text) => {
            println!("[RAW] {}", text);
        }
    }
}

async fn run_exec_command(command: &str, event_json: &str) -> anyhow::Result<()> {
    let mut child = if cfg!(target_os = "windows") {
        tokio::process::Command::new("cmd")
            .arg("/C")
            .arg(command)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to spawn exec command: {}", e))?
    } else {
        tokio::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to spawn exec command: {}", e))?
    };

    if let Some(stdin) = child.stdin.take() {
        let mut stdin = tokio::io::BufWriter::new(stdin);
        tokio::io::AsyncWriteExt::write_all(&mut stdin, event_json.as_bytes()).await?;
        tokio::io::AsyncWriteExt::write_all(&mut stdin, b"\n").await?;
    }

    let timeout = std::time::Duration::from_secs(30);
    let result = tokio::time::timeout(timeout, child.wait()).await;

    match result {
        Ok(Ok(status)) => {
            if !status.success() {
                let code = status.code().map_or("?".to_string(), |c| c.to_string());
                anyhow::bail!("Exit code: {}", code);
            }
            Ok(())
        }
        Ok(Err(e)) => Err(anyhow::anyhow!("Process error: {}", e)),
        Err(_) => {
            let _ = child.start_kill();
            anyhow::bail!("Timeout after {}s", timeout.as_secs())
        }
    }
}
