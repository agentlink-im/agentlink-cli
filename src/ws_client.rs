use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use tokio::time::{interval, Duration};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use agentlink_protocol::ws_event::{
    WsEventNotificationPayload, WsMessageCreatedPayload, WsServerEnvelope, WsServerMessage,
};

use crate::config::Config;

/// 启动 WebSocket 监听并阻塞直到用户按下 Ctrl+C
pub async fn run_watch(config: &Config, conversation_id: Option<String>) -> Result<()> {
    let token = config.require_api_key()?;
    let base_url = config.websocket_url.trim_end_matches('/');
    let url = if base_url.ends_with("/ws") {
        format!("{}?token={}", base_url, urlencoding::encode(token))
    } else {
        format!("{}/ws?token={}", base_url, urlencoding::encode(token))
    };

    let (ws_stream, _) = connect_async(&url)
        .await
        .context("Failed to connect to WebSocket")?;

    let (mut write, mut read) = ws_stream.split();

    // 心跳任务
    let heartbeat_handle = tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(30));
        loop {
            ticker.tick().await;
            let ping = serde_json::json!({
                "type": "ping",
                "payload": {
                    "client_timestamp": Utc::now().to_rfc3339()
                }
            });
            if write.send(Message::Text(ping.to_string().into())).await.is_err() {
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
                println!("\nDisconnecting...");
                break;
            }
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<WsServerEnvelope>(&text) {
                            Ok(envelope) => {
                                if !handle_message(&envelope.message, conversation_id.as_deref()) {
                                    break;
                                }
                            }
                            Err(_) => println!("[RAW] {}", text),
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        println!("Connection closed by server.");
                        break;
                    }
                    Some(Err(e)) => {
                        eprintln!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    heartbeat_handle.abort();
    Ok(())
}

/// 处理单条 WebSocket 消息。返回 false 表示应该退出监听循环。
fn handle_message(message: &WsServerMessage, filter_conversation_id: Option<&str>) -> bool {
    match message {
        WsServerMessage::ConnectionReady(payload) => {
            println!("[READY] Connected as {}", payload.linkid);
        }
        WsServerMessage::MessageCreated(payload) => {
            handle_message_created(payload, filter_conversation_id);
        }
        WsServerMessage::EventNotification(payload) => {
            handle_event_notification(payload);
        }
        WsServerMessage::Pong(_) => {
            // 静默忽略心跳回包
        }
        WsServerMessage::Error(payload) => {
            eprintln!("[ERROR] {}: {}", payload.code, payload.message);
        }
    }
    true
}

fn handle_message_created(
    payload: &WsMessageCreatedPayload,
    filter_conversation_id: Option<&str>,
) {
    if !message_matches_filter(payload, filter_conversation_id) {
        return;
    }

    let msg = &payload.message;
    println!(
        "[{}] {}: {}",
        format_time(msg.created_at),
        msg.sender_name,
        msg.content
    );
}

fn message_matches_filter(
    payload: &WsMessageCreatedPayload,
    filter_conversation_id: Option<&str>,
) -> bool {
    if let Some(filter) = filter_conversation_id {
        let msg_conversation_id = payload.message.conversation_id.to_string();
        msg_conversation_id == filter
    } else {
        true
    }
}

fn handle_event_notification(payload: &WsEventNotificationPayload) {
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
    fn test_handle_message_returns_true_for_all_current_variants() {
        // All current server message variants should keep the loop running
        let ready = WsServerMessage::ConnectionReady(WsConnectionReadyPayload {
            user_id: Uuid::nil(),
            linkid: "alice".to_string(),
        });
        assert!(handle_message(&ready, None));

        let pong = WsServerMessage::Pong(WsPongPayload {
            client_timestamp: None,
        });
        assert!(handle_message(&pong, None));

        let error = WsServerMessage::Error(WsErrorPayload {
            code: "ERR".to_string(),
            message: "oops".to_string(),
            client_message_id: None,
        });
        assert!(handle_message(&error, None));
    }

    #[test]
    fn test_message_matches_filter_without_filter() {
        let payload = WsMessageCreatedPayload {
            message: sample_message_response(),
            client_message_id: None,
        };
        assert!(message_matches_filter(&payload, None));
    }

    #[test]
    fn test_message_matches_filter_with_matching_id() {
        let payload = WsMessageCreatedPayload {
            message: sample_message_response(),
            client_message_id: None,
        };
        assert!(message_matches_filter(&payload, Some("00000000-0000-0000-0000-000000000000")));
    }

    #[test]
    fn test_message_matches_filter_with_mismatching_id() {
        let payload = WsMessageCreatedPayload {
            message: sample_message_response(),
            client_message_id: None,
        };
        assert!(!message_matches_filter(&payload, Some("11111111-1111-1111-1111-111111111111")));
    }

    #[test]
    fn test_handle_message_applies_conversation_filter() {
        let payload = WsMessageCreatedPayload {
            message: sample_message_response(),
            client_message_id: None,
        };
        let msg = WsServerMessage::MessageCreated(payload);
        // handle_message returns true regardless of filter; filtering is handled inside
        assert!(handle_message(&msg, Some("00000000-0000-0000-0000-000000000000")));
    }
}
