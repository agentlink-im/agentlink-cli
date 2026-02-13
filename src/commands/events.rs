//! Event Loop Command
//!
//! Monitor all AgentLink events in real-time.

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use agentlink_sdk::{
    events::{
        ServerEvent,
        MessageReceivedData, MessageDeliveredData, MessageReadData, MessageDeletedData,
        UnreadCountUpdatedData, OfflineMessagesBatchData,
        FriendRequestReceivedData, FriendRequestAcceptedData, FriendRequestRejectedData,
        FriendAddedData, FriendRemovedData,
        UserPresenceChangedData,
        UserBlockedData, UserUnblockedData,
        SyncConversationListData, SyncMessageHistoryData, SyncConversationUpdateData,
        SyncFriendUpdateData, SyncFriendRequestListData, SyncCompleteData,
        EVENT_MESSAGE_RECEIVED, EVENT_MESSAGE_DELIVERED, EVENT_MESSAGE_READ, EVENT_MESSAGE_DELETED,
        EVENT_UNREAD_COUNT_UPDATED, EVENT_OFFLINE_MESSAGES_BATCH,
        EVENT_FRIEND_REQUEST_RECEIVED, EVENT_FRIEND_REQUEST_ACCEPTED, EVENT_FRIEND_REQUEST_REJECTED,
        EVENT_FRIEND_ADDED, EVENT_FRIEND_REMOVED,
        EVENT_USER_PRESENCE_CHANGED,
        EVENT_USER_BLOCKED, EVENT_USER_UNBLOCKED,
        EVENT_SYNC_CONVERSATION_LIST, EVENT_SYNC_FRIEND_LIST, EVENT_SYNC_MESSAGE_HISTORY,
        EVENT_SYNC_CONVERSATION_UPDATE, EVENT_SYNC_FRIEND_UPDATE, EVENT_SYNC_FRIEND_REQUEST_LIST,
        EVENT_SYNC_COMPLETE,
    },
    protocols::friend::SyncFriendListData,
    AgentLinkClient, ClientConfig,
};
use colored::Colorize;
use chrono::{TimeZone, Utc};

use crate::config::CliConfig;
use crate::output;

/// Start event loop to monitor all events
pub async fn start(config: &CliConfig) -> anyhow::Result<()> {
    let api_key = config.require_api_key()?;

    println!();
    println!("{}", "╔════════════════════════════════════════╗".cyan());
    println!("{}", "║     AgentLink Event Monitor Mode       ║".cyan());
    println!("{}", "╚════════════════════════════════════════╝".cyan());
    println!();

    // Create client
    let client_config = ClientConfig::default().with_token(api_key.to_string());
    let client = AgentLinkClient::new(client_config);

    // Connect MQTT and start event loop
    output::info("Connecting to AgentLink...");
    client.connect_and_start().await?;
    output::success("Connected! Listening for events...");
    println!();
    println!("{}", "Press Ctrl+C or type 'q' to exit".dimmed());
    println!("{}", "─────────────────────────────────────────".dimmed());
    println!();

    // Register all event handlers
    register_event_handlers(&client).await;

    // Wait for exit signal
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    // Simple input loop for 'q' command
    while running.load(Ordering::SeqCst) {
        print!("{}", "> ".dimmed());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input == "q" || input == "quit" || input == "exit" {
            break;
        }
    }

    // Cleanup
    client.disconnect_mqtt().await?;
    output::info("Disconnected.");

    Ok(())
}

async fn register_event_handlers(client: &AgentLinkClient) {
    // ==================== Message Events ====================

    client.on(EVENT_MESSAGE_RECEIVED, |event: ServerEvent<MessageReceivedData>| {
        let data = event.data;
        let time = format_timestamp(event.timestamp);
        let content = data.content.as_deref().unwrap_or("<empty>");
        let sender_short = shorten_id(&data.sender_id);
        let conv_short = shorten_id(&data.conversation_id);

        println!("{}", format!("[{}] {} {}", time.dimmed(), "MESSAGE RECEIVED".green(), format!("from {}", sender_short).blue()));
        println!("  Conversation: {}", conv_short);
        println!("  Content: {}", content);
        println!();
        async {}
    }).await;

    client.on(EVENT_MESSAGE_DELIVERED, |event: ServerEvent<MessageDeliveredData>| {
        let data = event.data;
        let time = format_timestamp(event.timestamp);
        let msg_short = shorten_id(&data.message_id);
        let conv_short = shorten_id(&data.conversation_id);

        println!("{}", format!("[{}] {} to {}", time.dimmed(), "DELIVERED".yellow(), conv_short));
        println!("  Message: {}", msg_short);
        println!();
        async {}
    }).await;

    client.on(EVENT_MESSAGE_READ, |event: ServerEvent<MessageReadData>| {
        let data = event.data;
        let time = format_timestamp(event.timestamp);
        let msg_short = shorten_id(&data.message_id);
        let conv_short = shorten_id(&data.conversation_id);

        println!("{}", format!("[{}] {} in {}", time.dimmed(), "READ".cyan(), conv_short));
        println!("  Message: {}", msg_short);
        println!();
        async {}
    }).await;

    client.on(EVENT_MESSAGE_DELETED, |event: ServerEvent<MessageDeletedData>| {
        let data = event.data;
        let time = format_timestamp(event.timestamp);
        let msg_short = shorten_id(&data.message_id);

        println!("{}", format!("[{}] {} {}", time.dimmed(), "DELETED".red(), msg_short));
        println!();
        async {}
    }).await;

    client.on(EVENT_UNREAD_COUNT_UPDATED, |event: ServerEvent<UnreadCountUpdatedData>| {
        let data = event.data;
        let time = format_timestamp(event.timestamp);
        let conv_short = shorten_id(&data.conversation_id);

        println!("{}", format!("[{}] {} in {}", time.dimmed(), "UNREAD COUNT".magenta(), conv_short));
        println!("  Unread: {} | Total: {}", data.unread_count, data.total_unread);
        println!();
        async {}
    }).await;

    client.on(EVENT_OFFLINE_MESSAGES_BATCH, |event: ServerEvent<OfflineMessagesBatchData>| {
        let data = event.data;
        let time = format_timestamp(event.timestamp);

        println!("{}", format!("[{}] {} (batch {}/{})", time.dimmed(), "OFFLINE MESSAGES".green(), data.batch, data.total));
        for msg in &data.messages {
            let sender_short = shorten_id(&msg.sender_id);
            let content = msg.content.as_deref().unwrap_or("<empty>");
            println!("  {} {}: {}", "From".dimmed(), sender_short.blue(), content);
        }
        println!();
        async {}
    }).await;

    // ==================== Friend Events ====================

    client.on(EVENT_FRIEND_REQUEST_RECEIVED, |event: ServerEvent<FriendRequestReceivedData>| {
        let data = event.data;
        let time = format_timestamp(event.timestamp);
        let from_short = shorten_id(&data.from_user_id);

        println!("{}", format!("[{}] {} from {}", time.dimmed(), "FRIEND REQUEST".green(), from_short));
        if let Some(ref msg) = data.message {
            println!("  Message: {}", msg);
        }
        println!();
        async {}
    }).await;

    client.on(EVENT_FRIEND_REQUEST_ACCEPTED, |event: ServerEvent<FriendRequestAcceptedData>| {
        let data = event.data;
        let time = format_timestamp(event.timestamp);
        let friend_name = data.friend.nickname.clone();

        println!("{}", format!("[{}] {} {} accepted your request", time.dimmed(), "FRIEND".green(), friend_name.blue()));
        println!("  Friend ID: {}", shorten_id(&data.friend_id));
        println!();
        async {}
    }).await;

    client.on(EVENT_FRIEND_REQUEST_REJECTED, |event: ServerEvent<FriendRequestRejectedData>| {
        let data = event.data;
        let time = format_timestamp(event.timestamp);

        println!("{}", format!("[{}] {} rejected", time.dimmed(), "FRIEND REQUEST".red()));
        if let Some(ref msg) = data.message {
            println!("  Message: {}", msg);
        }
        println!();
        async {}
    }).await;

    client.on(EVENT_FRIEND_ADDED, |event: ServerEvent<FriendAddedData>| {
        let data = event.data;
        let time = format_timestamp(event.timestamp);
        let friend_short = shorten_id(&data.friend_id);

        println!("{}", format!("[{}] {} {}", time.dimmed(), "FRIEND ADDED".green(), friend_short));
        println!();
        async {}
    }).await;

    client.on(EVENT_FRIEND_REMOVED, |event: ServerEvent<FriendRemovedData>| {
        let data = event.data;
        let time = format_timestamp(event.timestamp);
        let friend_short = shorten_id(&data.friend_id);

        println!("{}", format!("[{}] {} {}", time.dimmed(), "FRIEND REMOVED".red(), friend_short));
        println!();
        async {}
    }).await;

    // ==================== User Presence Events ====================

    client.on(EVENT_USER_PRESENCE_CHANGED, |event: ServerEvent<UserPresenceChangedData>| {
        let data = event.data;
        let time = format_timestamp(event.timestamp);
        let user_short = shorten_id(&data.user_id);
        let status = if data.online { "ONLINE".green() } else { "OFFLINE".red() };

        println!("{}", format!("[{}] {} is {}", time.dimmed(), user_short.blue(), status));
        if let Some(ref device) = data.device_type {
            println!("  Device: {}", device);
        }
        println!();
        async {}
    }).await;

    // ==================== Block Events ====================

    client.on(EVENT_USER_BLOCKED, |event: ServerEvent<UserBlockedData>| {
        let data = event.data;
        let time = format_timestamp(event.timestamp);

        println!("{}", format!("[{}] {} {}", time.dimmed(), "BLOCKED".red(), data.blocked_user_nickname));
        println!();
        async {}
    }).await;

    client.on(EVENT_USER_UNBLOCKED, |event: ServerEvent<UserUnblockedData>| {
        let data = event.data;
        let time = format_timestamp(event.timestamp);
        let user_short = shorten_id(&data.unblocked_user_id);

        println!("{}", format!("[{}] {} {}", time.dimmed(), "UNBLOCKED".green(), user_short));
        println!();
        async {}
    }).await;

    // ==================== Sync Events ====================

    client.on(EVENT_SYNC_CONVERSATION_LIST, |event: ServerEvent<SyncConversationListData>| {
        let data = event.data;
        let time = format_timestamp(event.timestamp);

        println!("{}", format!("[{}] {} ({} conversations)", time.dimmed(), "SYNC CONVERSATIONS".cyan(), data.total));
        for conv in &data.conversations {
            let name = conv.name.as_deref().unwrap_or("Direct");
            println!("  {} {}", shorten_id(&conv.id).dimmed(), name);
        }
        println!();
        async {}
    }).await;

    client.on(EVENT_SYNC_FRIEND_LIST, |event: ServerEvent<SyncFriendListData>| {
        let data = event.data;
        let time = format_timestamp(event.timestamp);

        println!("{}", format!("[{}] {} ({} friends)", time.dimmed(), "SYNC FRIENDS".cyan(), data.friends.len()));
        for friendship in &data.friends {
            if let Some(ref friend) = friendship.friend {
                let name = friendship.remark.as_ref().unwrap_or(&friend.nickname);
                println!("  {} {}", shorten_id(&friend.id).dimmed(), name.blue());
            }
        }
        println!();
        async {}
    }).await;

    client.on(EVENT_SYNC_MESSAGE_HISTORY, |event: ServerEvent<SyncMessageHistoryData>| {
        let data = event.data;
        let time = format_timestamp(event.timestamp);
        let conv_short = shorten_id(&data.conversation_id);

        println!("{}", format!("[{}] {} for {}", time.dimmed(), "SYNC MESSAGES".cyan(), conv_short));
        println!("  Count: {} | Has more: {}", data.messages.len(), data.has_more);
        println!();
        async {}
    }).await;

    client.on(EVENT_SYNC_CONVERSATION_UPDATE, |event: ServerEvent<SyncConversationUpdateData>| {
        let data = event.data;
        let time = format_timestamp(event.timestamp);
        let conv_short = shorten_id(&data.id);

        println!("{}", format!("[{}] {} {}", time.dimmed(), "CONVERSATION UPDATE".cyan(), conv_short));
        if let Some(ref name) = data.name {
            println!("  Name: {}", name);
        }
        println!();
        async {}
    }).await;

    client.on(EVENT_SYNC_FRIEND_UPDATE, |event: ServerEvent<SyncFriendUpdateData>| {
        let data = event.data;
        let time = format_timestamp(event.timestamp);

        println!("{}", format!("[{}] {} for {}", time.dimmed(), "FRIEND UPDATE".cyan(), data.friend.nickname));
        println!("  Type: {}", data.update_type);
        println!();
        async {}
    }).await;

    client.on(EVENT_SYNC_FRIEND_REQUEST_LIST, |event: ServerEvent<SyncFriendRequestListData>| {
        let data = event.data;
        let time = format_timestamp(event.timestamp);

        println!("{}", format!("[{}] {} ({} requests)", time.dimmed(), "SYNC FRIEND REQUESTS".cyan(), data.total));
        for req in &data.requests {
            let status_color = match req.status.as_str() {
                "pending" => "pending".yellow(),
                "accepted" => "accepted".green(),
                "rejected" => "rejected".red(),
                _ => req.status.normal(),
            };
            println!("  {} from {} | {}", shorten_id(&req.id).dimmed(), shorten_id(&req.from_user_id).blue(), status_color);
        }
        println!();
        async {}
    }).await;

    client.on(EVENT_SYNC_COMPLETE, |event: ServerEvent<SyncCompleteData>| {
        let data = event.data;
        let time = format_timestamp(event.timestamp);

        println!("{}", format!("[{}] {} ({})", time.dimmed(), "SYNC COMPLETE".green().bold(), data.sync_type));
        println!("  Items: {}", data.item_count);
        println!();
        async {}
    }).await;
}

/// Format Unix timestamp to human-readable time
fn format_timestamp(ts: i64) -> String {
    Utc.timestamp_opt(ts, 0)
        .single()
        .map(|dt| dt.format("%H:%M:%S").to_string())
        .unwrap_or_else(|| ts.to_string())
}

/// Shorten ID for display (first 8 chars)
fn shorten_id(id: &str) -> String {
    id[..8.min(id.len())].to_string()
}
