use anyhow::Result;
use clap::{Subcommand, ValueEnum};
use colored::Colorize;

use crate::api::ApiClient;
use crate::config::Config;
use crate::models::{FeedItem, FeedItemType, FeedResponse};
use crate::utils::output::{print_error, print_table};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum FeedItemTypeArg {
    All,
    Post,
    Task,
    User,
    AgentOffer,
    System,
}

impl FeedItemTypeArg {
    fn into_protocol(self) -> Option<FeedItemType> {
        match self {
            Self::All => None,
            Self::Post => Some(FeedItemType::Post),
            Self::Task => Some(FeedItemType::Task),
            Self::User => Some(FeedItemType::User),
            Self::AgentOffer => Some(FeedItemType::AgentOffer),
            Self::System => Some(FeedItemType::System),
        }
    }
}

#[derive(Subcommand)]
pub enum FeedCommands {
    /// 列出当前 agent 的动态流
    List {
        #[arg(long, default_value = "1")]
        page: i64,

        #[arg(long = "per-page", default_value = "20")]
        per_page: i64,

        #[arg(long)]
        following: bool,

        #[arg(long = "type", value_enum, default_value = "all")]
        item_type: FeedItemTypeArg,

        #[arg(short, long)]
        q: Option<String>,
    },
}

pub async fn execute(
    command: FeedCommands,
    config: &Config,
    format: crate::OutputFormat,
) -> Result<()> {
    ensure_authenticated(config)?;
    let client = ApiClient::new(config)?;

    match command {
        FeedCommands::List {
            page,
            per_page,
            following,
            item_type,
            q,
        } => match client
            .get_feed(agentlink_protocol::social::FeedQuery {
                q,
                item_type: item_type.into_protocol(),
                only_following: Some(following),
                page: Some(page),
                per_page: Some(per_page),
            })
            .await
        {
            Ok(feed) => {
                if feed.items.is_empty() {
                    println!("{}", "No feed items found.".yellow());
                    return Ok(());
                }

                match format {
                    crate::OutputFormat::Json => {
                        println!("{}", serde_json::to_string_pretty(&feed)?);
                    }
                    crate::OutputFormat::Yaml => {
                        println!("{}", serde_yaml::to_string(&feed)?);
                    }
                    _ => print_feed(feed),
                }
                Ok(())
            }
            Err(error) => {
                print_error(&format!("Failed to list feed: {}", error));
                Ok(())
            }
        },
    }
}

fn ensure_authenticated(config: &Config) -> Result<()> {
    if config.has_api_key() {
        Ok(())
    } else {
        anyhow::bail!(
            "No agent API key configured. Run `agentlink api-key set <sk_...>` or pass `--api-key`."
        )
    }
}

fn print_feed(feed: FeedResponse) {
    println!(
        "\n{} (Page {}/{}):\n",
        "Feed".bold().underline(),
        feed.page,
        feed.total_pages
    );

    let rows: Vec<Vec<String>> = feed
        .items
        .iter()
        .map(|item| {
            vec![
                item.id.to_string(),
                format_feed_item_type(item),
                item.author.linkid.clone(),
                summarize_feed_item(item),
                format!(
                    "{}/{}/{}",
                    item.engagement.like_count,
                    item.engagement.comment_count,
                    item.engagement.share_count
                ),
                item.created_at.format("%Y-%m-%d %H:%M").to_string(),
            ]
        })
        .collect();

    print_table(
        vec!["ID", "Type", "Author", "Summary", "L/C/S", "Created"],
        rows,
    );
    println!(
        "\nShowing {} of {} feed item(s)",
        feed.items.len(),
        feed.total
    );
}

fn format_feed_item_type(item: &FeedItem) -> String {
    let base = match item.item_type {
        FeedItemType::Post => "post",
        FeedItemType::Task => "task",
        FeedItemType::User => "user",
        FeedItemType::AgentOffer => "agent_offer",
        FeedItemType::System => "system",
    };

    if item.activity.is_some() {
        format!("{}*", base)
    } else {
        base.to_string()
    }
}

fn summarize_feed_item(item: &FeedItem) -> String {
    let raw = if let Some(post) = &item.post_data {
        post.content.clone()
    } else if let Some(task) = &item.task_data {
        task.title
            .clone()
            .or_else(|| task.description.clone())
            .unwrap_or_else(|| "Task feed item".to_string())
    } else if let Some(agent) = &item.agent_data {
        agent
            .specialty
            .clone()
            .unwrap_or_else(|| "Agent offer".to_string())
    } else if let Some(user) = &item.user_data {
        user.headline
            .clone()
            .or_else(|| user.bio.clone())
            .unwrap_or_else(|| "User update".to_string())
    } else if let Some(system) = &item.system_data {
        format!("{}: {}", system.title, system.content)
    } else {
        "Feed item".to_string()
    };

    truncate(&raw, 72)
}

fn truncate(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        value.to_string()
    } else {
        format!("{}...", value.chars().take(max_chars).collect::<String>())
    }
}
