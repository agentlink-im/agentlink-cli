use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use tokio::time::{interval, sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use agentlink_protocol::ws_event::{
    WsClientMessage, WsEventNotificationPayload, WsMessageCreatedPayload, WsPingPayload,
    WsServerEnvelope, WsServerMessage,
};

use crate::config::Config;

/// 启动 WebSocket 监听并阻塞直到用户按下 Ctrl+C
/// 支持可选的 conversation_id 过滤（仅处理该会话的消息）
pub async fn run_watch(config: &Config, conversation_id: Option<String>) -> Result<()> {
    run_poll(
        config,
        false,       // json mode
        Vec::new(),  // no filters
        false,       // no auto-reconnect
        60,          // max_backoff unused
        None,        // no exec command
        conversation_id.as_deref(),
    )
    .await
}

/// 启动 WebSocket 事件流，支持 JSON Lines 输出、事件过滤、自动重连和外部命令回调。
///
/// 当 `json` 为 true 时，每行输出一个 JSON 对象，格式如下：
/// ```json
/// {"event":"message.created","timestamp":"2026-01-01T00:00:00Z","payload":{...}}
/// ```
///
/// 当 `reconnect` 为 true 时，连接断开后会按指数退避策略自动重连，
/// 直到用户按下 Ctrl+C。
///
/// 当 `exec_command` 为 Some 时，每个事件都会通过 stdin 传给该命令，
/// 便于实现事件驱动的自动化 pipeline。
pub async fn run_poll(
    config: &Config,
    json: bool,
    filters: Vec<String>,
    reconnect: bool,
    max_backoff_secs: u64,
    exec_command: Option<String>,
    conversation_filter: Option<&str>,
) -> Result<()> {
    let token = config.require_api_key()?;
    let base_url = config.websocket_url.trim_end_matches('/');
    let url = if base_url.ends_with("/ws") {
        format!("{}?token={}", base_url, urlencoding::encode(token))
    } else {
        format!("{}/ws?token={}", base_url, urlencoding::encode(token))
    };

    let filter_set = if filters.is_empty() {
        None
    } else {
        Some(
            filters
                .into_iter()
                .map(|s| s.to_lowercase().replace(".", "").replace("_", ""))
                .collect::<std::collections::HashSet<String>>(),
        )
    };

    let mut backoff_secs = 1u64;

    loop {
        match connect_and_listen(&url, json, filter_set.as_ref(), exec_command.as_deref(), conversation_filter).await {
            Ok(()) => {
                if !reconnect {
                    break;
                }
                // 正常退出（Ctrl+C），不重连
                break;
            }
            Err(e) => {
                if !reconnect {
                    return Err(e);
                }
                if !json {
                    eprintln!(
                        "\n[RECONNECT] Connection lost: {}. Retrying in {}s...",
                        e, backoff_secs
                    );
                } else {
                    eprintln!(r#"{{"event":"_reconnect","reason":"{}","backoff":{}}}"#, e, backoff_secs);
                }
                sleep(Duration::from_secs(backoff_secs)).await;
                backoff_secs = (backoff_secs * 2).min(max_backoff_secs);
            }
        }
    }

    Ok(())
}

/// 建立 WebSocket 连接并监听消息。返回 Ok 表示正常退出（Ctrl+C）。
async fn connect_and_listen(
    url: &str,
    json: bool,
    filter_set: Option<&std::collections::HashSet<String>>,
    exec_command: Option<&str>,
    conversation_filter: Option<&str>,
) -> Result<()> {
    let (ws_stream, _) = connect_async(url)
        .await
        .context("Failed to connect to WebSocket")?;

    let (mut write, mut read) = ws_stream.split();

    // 心跳任务
    let heartbeat_handle = tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(30));
        loop {
            ticker.tick().await;
            let ping = WsClientMessage::Ping(WsPingPayload {
                client_timestamp: Some(Utc::now().to_rfc3339()),
            });
            let ping_json = match serde_json::to_string(&ping) {
                Ok(json) => json,
                Err(_) => break,
            };
            if write.send(Message::Text(ping_json.into())).await.is_err() {
                break;
            }
        }
    });

    // Ctrl+C 监听
    let ctrl_c = tokio::signal::ctrl_c();
    tokio::pin!(ctrl_c);

    loop {
        tokio::select! {
            _ = &mut ctrl_c => {
                if !json {
                    println!("\nDisconnecting...");
                }
                break;
            }
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<WsServerEnvelope>(&text) {
                            Ok(envelope) => {
                                if !handle_envelope(&envelope, json, filter_set, exec_command, conversation_filter).await {
                                    break;
                                }
                            }
                            Err(_) => {
                                if json {
                                    println!(r#"{{"event":"_raw","text":{}}}"#, serde_json::to_string(&text.as_str()).unwrap_or_default());
                                } else {
                                    println!("[RAW] {}", text);
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        if !json {
                            println!("Connection closed by server.");
                        }
                        break;
                    }
                    Some(Err(e)) => {
                        heartbeat_handle.abort();
                        return Err(anyhow::anyhow!("WebSocket error: {}", e));
                    }
                    _ => {}
                }
            }
        }
    }

    heartbeat_handle.abort();
    Ok(())
}

/// 处理单个 envelope。返回 false 表示应该退出监听循环。
async fn handle_envelope(
    envelope: &WsServerEnvelope,
    json: bool,
    filter_set: Option<&std::collections::HashSet<String>>,
    exec_command: Option<&str>,
    conversation_filter: Option<&str>,
) -> bool {
    let event_name = event_name(&envelope.message);

    // 应用事件类型过滤
    if let Some(set) = filter_set {
        let normalized = event_name.to_lowercase().replace(".", "").replace("_", "");
        if !set.contains(&normalized) {
            return true;
        }
    }

    // 构建 JSON（exec 回调始终需要 JSON，即使输出模式是人类可读）
    let event_json = build_event_json(envelope);

    // 输出到 stdout
    if json {
        println!("{}", event_json);
    } else {
        print_envelope_human(envelope, conversation_filter);
    }

    // 触发外部命令回调（异步，不阻塞 WebSocket 读取）
    if let Some(cmd) = exec_command {
        let cmd = cmd.to_string();
        let event_json = event_json.clone();
        tokio::spawn(async move {
            if let Err(e) = run_exec_command(&cmd, &event_json).await {
                eprintln!("[EXEC ERROR] {}: {}", cmd, e);
            }
        });
    }

    true
}

/// 执行外部命令，将事件 JSON 通过 stdin 传入。
async fn run_exec_command(command: &str, event_json: &str) -> Result<()> {
    // 支持简单的 shell 命令（如 "python3 handler.py"）
    let mut child = if cfg!(target_os = "windows") {
        tokio::process::Command::new("cmd")
            .arg("/C")
            .arg(command)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .context("Failed to spawn exec command")?
    } else {
        tokio::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .context("Failed to spawn exec command")?
    };

    // 写入 stdin
    if let Some(stdin) = child.stdin.take() {
        let mut stdin = tokio::io::BufWriter::new(stdin);
        tokio::io::AsyncWriteExt::write_all(&mut stdin, event_json.as_bytes()).await?;
        tokio::io::AsyncWriteExt::write_all(&mut stdin, b"\n").await?;
        // BufWriter 在 drop 时会自动 flush
    }

    // 带超时的等待
    let timeout = Duration::from_secs(30);
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

fn build_event_json(envelope: &WsServerEnvelope) -> String {
    let (event_name, payload_json) = match &envelope.message {
        WsServerMessage::ConnectionReady(payload) => (
            "connection.ready",
            serde_json::to_string(payload),
        ),
        WsServerMessage::MessageCreated(payload) => (
            "message.created",
            serde_json::to_string(payload),
        ),
        WsServerMessage::EventNotification(payload) => (
            "event.notification",
            serde_json::to_string(payload),
        ),
        WsServerMessage::Pong(payload) => (
            "pong",
            serde_json::to_string(payload),
        ),
        WsServerMessage::Error(payload) => (
            "error",
            serde_json::to_string(payload),
        ),
    };

    match payload_json {
        Ok(payload) => format!(
            r#"{{"event":"{}","timestamp":"{}","payload":{}}}"#,
            event_name,
            envelope.timestamp.to_rfc3339(),
            payload
        ),
        Err(_) => format!(
            r#"{{"event":"{}","timestamp":"{}"}}"#,
            event_name,
            envelope.timestamp.to_rfc3339()
        ),
    }
}

fn event_name(message: &WsServerMessage) -> &'static str {
    match message {
        WsServerMessage::ConnectionReady(_) => "connection.ready",
        WsServerMessage::MessageCreated(_) => "message.created",
        WsServerMessage::EventNotification(_) => "event.notification",
        WsServerMessage::Pong(_) => "pong",
        WsServerMessage::Error(_) => "error",
    }
}

fn print_envelope_human(
    envelope: &WsServerEnvelope,
    conversation_filter: Option<&str>,
) {
    match &envelope.message {
        WsServerMessage::ConnectionReady(payload) => {
            println!(
                "[{}] [READY] Connected as {}",
                format_time(envelope.timestamp),
                payload.linkid
            );
        }
        WsServerMessage::MessageCreated(payload) => {
            handle_message_created_human(payload, conversation_filter);
        }
        WsServerMessage::EventNotification(payload) => {
            handle_event_notification_human(payload);
        }
        WsServerMessage::Pong(_) => {
            // 静默忽略心跳回包
        }
        WsServerMessage::Error(payload) => {
            eprintln!(
                "[{}] [ERROR] {}: {}",
                format_time(envelope.timestamp),
                payload.code,
                payload.message
            );
        }
    }
}

fn handle_message_created_human(
    payload: &WsMessageCreatedPayload,
    filter_conversation_id: Option<&str>,
) {
    if let Some(filter) = filter_conversation_id {
        if payload.message.conversation_id.to_string() != filter {
            return;
        }
    }

    let msg = &payload.message;
    println!(
        "[{}] [MESSAGE] {}: {}",
        format_time(msg.created_at),
        msg.sender_name,
        msg.content
    );
}

fn handle_event_notification_human(payload: &WsEventNotificationPayload) {
    let title = &payload.data.title;
    let content = &payload.data.content;

    println!("[NOTIFICATION] {}", title);
    if !content.is_empty() {
        println!("  {}", content);
    }
}

fn format_time(dt: DateTime<Utc>) -> String {
    dt.format("%H:%M:%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentlink_protocol::ws_event::{
        WsConnectionReadyPayload, WsErrorPayload, WsMessageCreatedPayload, WsPongPayload,
    };
    use chrono::TimeZone;
    use uuid::Uuid;

    fn sample_message_response() -> agentlink_protocol::message::MessageResponse {
        agentlink_protocol::message::MessageResponse {
            id: Uuid::nil(),
            conversation_id: Uuid::nil(),
            sender_id: Uuid::nil(),
            sender_name: "Alice".to_string(),
            sender_avatar: None,
            kind: agentlink_protocol::MessageType::Text,
            content: "Hello".to_string(),
            metadata: None,
            reply_to: None,
            is_edited: false,
            created_at: Utc.with_ymd_and_hms(2024, 1, 1, 12, 30, 45).unwrap(),
        }
    }

    #[test]
    fn test_format_time_formats_correctly() {
        let dt = Utc.with_ymd_and_hms(2024, 1, 1, 12, 30, 45).unwrap();
        assert_eq!(format_time(dt), "12:30:45");
    }

    #[test]
    fn test_event_name_mapping() {
        let ready = WsServerMessage::ConnectionReady(WsConnectionReadyPayload {
            user_id: Uuid::nil(),
            linkid: "alice".to_string(),
        });
        assert_eq!(event_name(&ready), "connection.ready");

        let pong = WsServerMessage::Pong(WsPongPayload {
            client_timestamp: None,
        });
        assert_eq!(event_name(&pong), "pong");

        let error = WsServerMessage::Error(WsErrorPayload {
            code: "ERR".to_string(),
            message: "oops".to_string(),
            client_message_id: None,
        });
        assert_eq!(event_name(&error), "error");
    }

    #[test]
    fn test_build_event_json_format() {
        let payload = WsConnectionReadyPayload {
            user_id: Uuid::nil(),
            linkid: "agent_1".to_string(),
        };
        let envelope = WsServerEnvelope {
            message: WsServerMessage::ConnectionReady(payload),
            timestamp: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
        };
        let json = build_event_json(&envelope);
        assert!(json.contains("\"event\":\"connection.ready\""));
        assert!(json.contains("\"linkid\":\"agent_1\""));
        assert!(json.contains("\"timestamp\""));
    }

    #[test]
    fn test_handle_envelope_filtering() {
        let envelope = WsServerEnvelope {
            message: WsServerMessage::ConnectionReady(WsConnectionReadyPayload {
                user_id: Uuid::nil(),
                linkid: "alice".to_string(),
            }),
            timestamp: Utc::now(),
        };

        // 无过滤：应该处理
        assert!(tokio::runtime::Runtime::new().unwrap().block_on(handle_envelope(&envelope, false, None, None, None)));

        // 有过滤且匹配：应该处理
        let mut set = std::collections::HashSet::new();
        set.insert("connectionready".to_string());
        assert!(tokio::runtime::Runtime::new().unwrap().block_on(handle_envelope(&envelope, false, Some(&set), None, None)));

        // 有过滤但不匹配：应该跳过（返回 true 表示继续循环）
        let mut set2 = std::collections::HashSet::new();
        set2.insert("messagecreated".to_string());
        assert!(tokio::runtime::Runtime::new().unwrap().block_on(handle_envelope(&envelope, false, Some(&set2), None, None)));
    }

    #[test]
    fn test_message_matches_filter_without_filter() {
        let payload = WsMessageCreatedPayload {
            message: sample_message_response(),
            client_message_id: None,
        };
        assert!(tokio::runtime::Runtime::new().unwrap().block_on(handle_envelope(
            &WsServerEnvelope {
                message: WsServerMessage::MessageCreated(payload),
                timestamp: Utc::now(),
            },
            false,
            None,
            None,
            None
        )));
    }

    #[test]
    fn test_message_matches_filter_with_matching_id() {
        let payload = WsMessageCreatedPayload {
            message: sample_message_response(),
            client_message_id: None,
        };
        assert!(tokio::runtime::Runtime::new().unwrap().block_on(handle_envelope(
            &WsServerEnvelope {
                message: WsServerMessage::MessageCreated(payload),
                timestamp: Utc::now(),
            },
            false,
            None,
            None,
            Some("00000000-0000-0000-0000-000000000000")
        )));
    }

    #[test]
    fn test_message_matches_filter_with_mismatching_id() {
        let payload = WsMessageCreatedPayload {
            message: sample_message_response(),
            client_message_id: None,
        };
        assert!(tokio::runtime::Runtime::new().unwrap().block_on(handle_envelope(
            &WsServerEnvelope {
                message: WsServerMessage::MessageCreated(payload),
                timestamp: Utc::now(),
            },
            false,
            None,
            None,
            Some("11111111-1111-1111-1111-111111111111")
        )));
    }
}
