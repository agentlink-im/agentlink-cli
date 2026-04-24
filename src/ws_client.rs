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
        conversation_id.as_deref(),
    )
    .await
}

/// 启动 WebSocket 事件流，支持 JSON Lines 输出、事件过滤和自动重连。
///
/// 当 `json` 为 true 时，每行输出一个 JSON 对象，格式如下：
/// ```json
/// {"event":"message.created","timestamp":"2026-01-01T00:00:00Z","payload":{...}}
/// ```
///
/// 当 `reconnect` 为 true 时，连接断开后会按指数退避策略自动重连，
/// 直到用户按下 Ctrl+C。
pub async fn run_poll(
    config: &Config,
    json: bool,
    filters: Vec<String>,
    reconnect: bool,
    max_backoff_secs: u64,
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
        match connect_and_listen(&url, json, filter_set.as_ref(), conversation_filter).await {
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
                                if !handle_envelope(&envelope, json, filter_set, conversation_filter) {
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
fn handle_envelope(
    envelope: &WsServerEnvelope,
    json: bool,
    filter_set: Option<&std::collections::HashSet<String>>,
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

    if json {
        // JSON Lines 输出：每行一个 JSON 对象
        let line = match &envelope.message {
            WsServerMessage::ConnectionReady(payload) => json_line(
                "connection.ready",
                &envelope.timestamp,
                payload,
            ),
            WsServerMessage::MessageCreated(payload) => {
                if let Some(filter) = conversation_filter {
                    if payload.message.conversation_id.to_string() != filter {
                        return true;
                    }
                }
                json_line("message.created", &envelope.timestamp, payload)
            }
            WsServerMessage::EventNotification(payload) => {
                json_line("event.notification", &envelope.timestamp, payload)
            }
            WsServerMessage::Pong(payload) => {
                json_line("pong", &envelope.timestamp, payload)
            }
            WsServerMessage::Error(payload) => {
                json_line("error", &envelope.timestamp, payload)
            }
        };
        println!("{}", line);
    } else {
        // 人类可读格式
        match &envelope.message {
            WsServerMessage::ConnectionReady(payload) => {
                println!("[{}] [READY] Connected as {}", format_time(envelope.timestamp), payload.linkid);
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
                eprintln!("[{}] [ERROR] {}: {}", format_time(envelope.timestamp), payload.code, payload.message);
            }
        }
    }
    true
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

fn json_line(event: &str, timestamp: &DateTime<Utc>, payload: &impl serde::Serialize) -> String {
    match serde_json::to_string(payload) {
        Ok(payload_json) => format!(
            r#"{{"event":"{}","timestamp":"{}","payload":{}}}"#,
            event,
            timestamp.to_rfc3339(),
            payload_json
        ),
        Err(_) => format!(
            r#"{{"event":"{}","timestamp":"{}"}}"#,
            event,
            timestamp.to_rfc3339()
        ),
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
    fn test_json_line_format() {
        let payload = WsConnectionReadyPayload {
            user_id: Uuid::nil(),
            linkid: "agent_1".to_string(),
        };
        let ts = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let line = json_line("connection.ready", &ts, &payload);
        assert!(line.contains("\"event\":\"connection.ready\""));
        assert!(line.contains("\"linkid\":\"agent_1\""));
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
        assert!(handle_envelope(&envelope, false, None, None));

        // 有过滤且匹配：应该处理
        let mut set = std::collections::HashSet::new();
        set.insert("connectionready".to_string());
        assert!(handle_envelope(&envelope, false, Some(&set), None));

        // 有过滤但不匹配：应该跳过（返回 true 表示继续循环）
        let mut set2 = std::collections::HashSet::new();
        set2.insert("messagecreated".to_string());
        assert!(handle_envelope(&envelope, false, Some(&set2), None));
    }

    #[test]
    fn test_message_matches_filter_without_filter() {
        let payload = WsMessageCreatedPayload {
            message: sample_message_response(),
            client_message_id: None,
        };
        assert!(handle_envelope(
            &WsServerEnvelope {
                message: WsServerMessage::MessageCreated(payload),
                timestamp: Utc::now(),
            },
            false,
            None,
            None
        ));
    }

    #[test]
    fn test_message_matches_filter_with_matching_id() {
        let payload = WsMessageCreatedPayload {
            message: sample_message_response(),
            client_message_id: None,
        };
        assert!(handle_envelope(
            &WsServerEnvelope {
                message: WsServerMessage::MessageCreated(payload),
                timestamp: Utc::now(),
            },
            false,
            None,
            Some("00000000-0000-0000-0000-000000000000")
        ));
    }

    #[test]
    fn test_message_matches_filter_with_mismatching_id() {
        let payload = WsMessageCreatedPayload {
            message: sample_message_response(),
            client_message_id: None,
        };
        assert!(handle_envelope(
            &WsServerEnvelope {
                message: WsServerMessage::MessageCreated(payload),
                timestamp: Utc::now(),
            },
            false,
            None,
            Some("11111111-1111-1111-1111-111111111111")
        ));
    }
}
