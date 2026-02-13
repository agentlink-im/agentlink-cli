//! Interactive Chat Mode

use std::io::{self, Write};

use agentlink_sdk::{
    events::{ServerEvent, MessageReceivedData},
    AgentLinkClient, ClientConfig, EVENT_MESSAGE_RECEIVED,
};
use colored::Colorize;
use dialoguer::Select;

use crate::config::CliConfig;
use crate::output;

/// Start interactive chat mode
pub async fn start(config: &CliConfig, conversation_id: Option<String>) -> anyhow::Result<()> {
    let api_key = config.require_api_key()?;

    println!();
    println!("{}", "╔════════════════════════════════════════╗".cyan());
    println!("{}", "║     AgentLink Interactive Chat Mode    ║".cyan());
    println!("{}", "╚════════════════════════════════════════╝".cyan());
    println!();

    // Create client
    let client_config = ClientConfig::default().with_token(api_key.to_string());
    let client = AgentLinkClient::new(client_config);

    // Connect MQTT and start event loop
    output::info("Connecting to AgentLink...");
    client.connect_and_start().await?;
    output::success("Connected!");

    // Select or create conversation
    let conversation_id = if let Some(id) = conversation_id {
        id
    } else {
        select_conversation(&client).await?
    };

    println!();
    output::info(&format!("Joined conversation: {}", conversation_id));
    println!();
    println!("{}", "Commands: /quit, /list, /switch, /help".dimmed());
    println!("{}", "─────────────────────────────────────────".dimmed());
    println!();

    // Subscribe to conversation topic
    let topic = format!("conversations/{}/messages", conversation_id);
    client.subscribe(&topic).await?;

    // Register message handler
    let conversation_id_clone = conversation_id.clone();
    client.on(EVENT_MESSAGE_RECEIVED, move |event: ServerEvent<MessageReceivedData>| {
        let conv_id = conversation_id_clone.clone();
        async move {
            let data = &event.data;
            if data.conversation_id == conv_id {
                let sender_short = &data.sender_id[..8.min(data.sender_id.len())];
                let content = data.content.as_deref().unwrap_or("");
                println!("{} {}: {}", chrono::Local::now().format("%H:%M").to_string().dimmed(), sender_short.blue(), content);
            }
        }
    }).await;

    // Main input loop
    loop {
        print!("{}", "> ".green());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        // Handle commands
        if input.starts_with('/') {
            match input {
                "/quit" | "/q" => {
                    println!();
                    output::info("Goodbye!");
                    break;
                }
                "/list" | "/l" => {
                    list_conversations(&client).await?;
                }
                "/help" | "/h" => {
                    println!();
                    println!("Commands:");
                    println!("  /quit, /q    - Exit chat mode");
                    println!("  /list, /l    - List conversations");
                    println!("  /help, /h    - Show this help");
                    println!();
                }
                _ => {
                    output::warn(&format!("Unknown command: {}", input));
                }
            }
            continue;
        }

        // Send message
        if let Err(e) = client.messages().send_message(&conversation_id, input, None, None, None).await {
            output::error(&format!("Failed to send: {}", e));
        }
    }

    // Cleanup
    client.disconnect_mqtt().await?;

    Ok(())
}

async fn select_conversation(client: &AgentLinkClient) -> anyhow::Result<String> {
    let response = client
        .conversations()
        .get_conversations()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get conversations: {}", e))?;

    if response.conversations.is_empty() {
        output::info("No conversations found. Create one with:");
        println!("  agentlink chat create-direct --user <user-id>");
        return Err(anyhow::anyhow!("No conversations available"));
    }

    let items: Vec<String> = response
        .conversations
        .iter()
        .map(|c| {
            let name = c.name.clone().unwrap_or_else(|| "Direct".to_string());
            format!("{} - {}", c.id[..8.min(c.id.len())].to_string().dimmed(), name)
        })
        .collect();

    let selection = Select::new()
        .with_prompt("Select a conversation")
        .items(&items)
        .default(0)
        .interact()?;

    Ok(response.conversations[selection].id.clone())
}

async fn list_conversations(client: &AgentLinkClient) -> anyhow::Result<()> {
    println!();
    output::header("Conversations");

    let response = client
        .conversations()
        .get_conversations()
        .await?;

    for conv in &response.conversations {
        let name = conv.name.clone().unwrap_or_else(|| "Direct".to_string());
        println!("{} {}", &conv.id[..8.min(conv.id.len())].dimmed(), name);
    }

    println!();
    Ok(())
}
