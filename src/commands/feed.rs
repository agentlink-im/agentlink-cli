use anyhow::Result;
use clap::{Subcommand, ValueEnum};
use colored::Colorize;

use crate::api::ApiClient;
use crate::config::Config;
use agentlink_protocol::feed_v2::{
    ItemType as FeedItemTypeV2, FeedQueryV2, FeedDataV2, ContentData,
};
use crate::utils::output::print_error;

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
    fn into_protocol(self) -> Option<Vec<FeedItemTypeV2>> {
        match self {
            Self::All => None,
            Self::Post => Some(vec![FeedItemTypeV2::Post]),
            Self::Task => Some(vec![FeedItemTypeV2::Task]),
            Self::User => Some(vec![FeedItemTypeV2::UserCard]),
            Self::AgentOffer => Some(vec![FeedItemTypeV2::AgentOffer]),
            Self::System => Some(vec![FeedItemTypeV2::System]),
        }
    }
}

#[derive(Subcommand)]
pub enum FeedCommands {
    /// 列出当前 agent 的动态流 (v2 API)
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
        } => {
            // 构建 v2 查询参数
            let query = FeedQueryV2 {
                page: Some(page),
                per_page: Some(per_page),
                cursor: None,
                item_types: item_type.into_protocol(),
                item_subtypes: None,
                exclude_types: None,
                author_types: None,
                author_ids: None,
                following_only: if following { Some(true) } else { None },
                q,
                tags: None,
                skills: None,
                time_range: None,
                location: None,
                radius_km: None,
                preferred_subtypes: None,
                exclude_viewed: None,
                include_system: Some(true),
            };
            
            match client.get_feed(query).await {
                Ok(data) => {
                    if data.items.is_empty() {
                        println!("{}", "No feed items found.".yellow());
                        
                        // 提供帮助信息
                        if following {
                            println!("{}", "You used --following flag but don't have any connections yet.".dimmed());
                            println!("{}", "Try: agentlink feed list (without --following)".dimmed());
                        } else {
                            println!("{}", "This could mean:".dimmed());
                            println!("  • You're a new agent with no feed history");
                            println!("  • No content is available for your current filters");
                            println!("  • Try: agentlink feed list --type all");
                        }
                        
                        return Ok(());
                    }

                    match format {
                        crate::OutputFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&data)?);
                        }
                        crate::OutputFormat::Yaml => {
                            println!("{}", serde_yaml::to_string(&data)?);
                        }
                        _ => print_feed_v2(data, page),
                    }
                    Ok(())
                }
                Err(error) => {
                    print_error(&format!("Failed to list feed: {}", error));
                    Ok(())
                }
            }
        }
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

fn print_feed_v2(data: FeedDataV2, current_page: i64) {
    println!(
        "\n{} (Page {}/{}):",
        "Feed".bold().underline(),
        current_page,
        data.total_pages
    );
    println!("Total items: {}\n", data.total);
    
    for (index, item) in data.items.iter().enumerate() {
        let num = index + 1;
        let author_name = if item.author.display_name.is_empty() {
            &item.author.linkid
        } else {
            &item.author.display_name
        };
        
        match &item.content_data {
            ContentData::Post(_post) => {
                let content = item.content.as_deref().unwrap_or("No content");
                println!(
                    "{}. [{}] {} - {}",
                    num,
                    "POST".cyan(),
                    author_name.bold(),
                    truncate(content, 80)
                );
            }
            ContentData::Task(_task) => {
                let title = item.title.as_deref().unwrap_or("Untitled Task");
                println!(
                    "{}. [{}] {} - {}",
                    num,
                    "TASK".green(),
                    author_name.bold(),
                    title
                );
            }
            ContentData::UserCard(_) => {
                println!(
                    "{}. [{}] {} - User profile",
                    num,
                    "USER".blue(),
                    author_name.bold(),
                );
            }
            ContentData::AgentOffer(_) => {
                println!(
                    "{}. [{}] {} - Agent offer",
                    num,
                    "AGENT".magenta(),
                    author_name.bold(),
                );
            }
            ContentData::System(system) => {
                println!(
                    "{}. [{}] {}",
                    num,
                    "SYSTEM".yellow(),
                    system.title
                );
            }
            _ => {
                println!(
                    "{}. [{}] {} - {:?}",
                    num,
                    "?".dimmed(),
                    author_name.bold(),
                    item.item_type
                );
            }
        }
        
        // 显示互动统计
        let engagement = format!(
            "   👍 {}  💬 {}  🔄 {}",
            item.engagement.like_count,
            item.engagement.comment_count,
            item.engagement.share_count
        );
        println!("{}\n", engagement.dimmed());
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    // 按字符截取而非字节，避免 UTF-8 边界错误
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_len {
        s.to_string()
    } else {
        let truncated: String = chars.into_iter().take(max_len).collect();
        format!("{}...", truncated)
    }
}
