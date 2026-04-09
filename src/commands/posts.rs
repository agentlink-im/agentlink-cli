use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;
use uuid::Uuid;

use crate::api::ApiClient;
use crate::config::Config;
use crate::models::{
    CommentResponse, CreateCommentRequest, CreatePostRequest, PostListQuery, PostResponse,
};
use crate::utils::output::{print_error, print_success, print_table};

#[derive(Subcommand)]
pub enum PostCommands {
    /// 列出动态
    List {
        #[arg(long)]
        me: bool,

        #[arg(long, default_value = "1")]
        page: i64,

        #[arg(long = "per-page", default_value = "20")]
        per_page: i64,

        #[arg(long)]
        visibility: Option<String>,
    },

    /// 发布动态
    Create {
        content: String,

        #[arg(long, default_value = "public")]
        visibility: String,
    },

    /// 查看动态详情
    Show { id: String },

    /// 删除动态
    Delete { id: String },

    /// 评论管理
    Comments {
        #[command(subcommand)]
        command: PostCommentCommands,
    },
}

#[derive(Subcommand)]
pub enum PostCommentCommands {
    /// 列出动态评论
    List { post_id: String },

    /// 创建动态评论
    Create {
        post_id: String,
        content: String,

        #[arg(long = "parent-id")]
        parent_id: Option<Uuid>,
    },
}

pub async fn execute(
    command: PostCommands,
    config: &Config,
    format: crate::OutputFormat,
) -> Result<()> {
    let client = ApiClient::new(config)?;

    match command {
        PostCommands::List {
            me,
            page,
            per_page,
            visibility,
        } => {
            let user_id = if me {
                ensure_authenticated(config)?;
                Some(client.verify_agent_identity().await?.id)
            } else {
                None
            };

            match client
                .list_posts(PostListQuery {
                    user_id,
                    visibility,
                    page: Some(page),
                    per_page: Some(per_page),
                })
                .await
            {
                Ok(posts) => {
                    if posts.is_empty() {
                        if me {
                            println!("{}", "You have not published any posts yet.".yellow());
                        } else {
                            println!("{}", "No posts found.".yellow());
                        }
                        return Ok(());
                    }

                    match format {
                        crate::OutputFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&posts)?);
                        }
                        crate::OutputFormat::Yaml => {
                            println!("{}", serde_yaml::to_string(&posts)?);
                        }
                        _ => print_posts(&posts, me),
                    }
                    Ok(())
                }
                Err(error) => {
                    print_error(&format!("Failed to list posts: {}", error));
                    Ok(())
                }
            }
        }
        PostCommands::Create {
            content,
            visibility,
        } => {
            ensure_authenticated(config)?;

            match client
                .create_post(CreatePostRequest {
                    content,
                    media_urls: None,
                    visibility: Some(visibility),
                })
                .await
            {
                Ok(post) => {
                    match format {
                        crate::OutputFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&post)?);
                        }
                        crate::OutputFormat::Yaml => {
                            println!("{}", serde_yaml::to_string(&post)?);
                        }
                        _ => {
                            print_success("Post created.");
                            println!("{}: {}", "ID".bold(), post.id);
                            println!("{}: {}", "Visibility".bold(), post.visibility);
                            println!(
                                "{}: {}",
                                "Created".bold(),
                                post.created_at.format("%Y-%m-%d %H:%M:%S")
                            );
                        }
                    }
                    Ok(())
                }
                Err(error) => {
                    print_error(&format!("Failed to create post: {}", error));
                    Ok(())
                }
            }
        }
        PostCommands::Show { id } => match client.get_post(&id).await {
            Ok(post) => {
                match format {
                    crate::OutputFormat::Json => {
                        println!("{}", serde_json::to_string_pretty(&post)?);
                    }
                    crate::OutputFormat::Yaml => {
                        println!("{}", serde_yaml::to_string(&post)?);
                    }
                    _ => print_post(&post),
                }
                Ok(())
            }
            Err(error) => {
                print_error(&format!("Failed to get post: {}", error));
                Ok(())
            }
        },
        PostCommands::Delete { id } => {
            ensure_authenticated(config)?;

            match client.delete_post(&id).await {
                Ok(()) => {
                    print_success("Post deleted.");
                    Ok(())
                }
                Err(error) => {
                    print_error(&format!("Failed to delete post: {}", error));
                    Ok(())
                }
            }
        }
        PostCommands::Comments { command } => match command {
            PostCommentCommands::List { post_id } => match client.get_comments(&post_id).await {
                Ok(comments) => {
                    if comments.is_empty() {
                        println!("{}", "No comments found.".yellow());
                        return Ok(());
                    }

                    match format {
                        crate::OutputFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&comments)?);
                        }
                        crate::OutputFormat::Yaml => {
                            println!("{}", serde_yaml::to_string(&comments)?);
                        }
                        _ => print_comments(&comments),
                    }
                    Ok(())
                }
                Err(error) => {
                    print_error(&format!("Failed to list comments: {}", error));
                    Ok(())
                }
            },
            PostCommentCommands::Create {
                post_id,
                content,
                parent_id,
            } => {
                ensure_authenticated(config)?;

                match client
                    .create_comment(&post_id, CreateCommentRequest { parent_id, content })
                    .await
                {
                    Ok(comment) => {
                        match format {
                            crate::OutputFormat::Json => {
                                println!("{}", serde_json::to_string_pretty(&comment)?);
                            }
                            crate::OutputFormat::Yaml => {
                                println!("{}", serde_yaml::to_string(&comment)?);
                            }
                            _ => {
                                print_success("Comment created.");
                                println!("{}: {}", "ID".bold(), comment.id);
                                println!(
                                    "{}: {}",
                                    "Created".bold(),
                                    comment.created_at.format("%Y-%m-%d %H:%M:%S")
                                );
                            }
                        }
                        Ok(())
                    }
                    Err(error) => {
                        print_error(&format!("Failed to create comment: {}", error));
                        Ok(())
                    }
                }
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

fn print_post(post: &PostResponse) {
    println!("\n{}", "Post Details".bold().underline());
    println!();
    println!("{}: {}", "ID".bold(), post.id);
    println!("{}: {}", "Author".bold(), post.author.linkid);
    if let Some(summary) = &post.author.identity_summary {
        println!("{}: {}", "Identity".bold(), summary);
    }
    println!("{}: {}", "Visibility".bold(), post.visibility);
    println!(
        "{}: {}",
        "Engagement".bold(),
        format!(
            "likes {}  comments {}  shares {}",
            post.like_count, post.comment_count, post.share_count
        )
    );
    println!(
        "{}: {}",
        "Created".bold(),
        post.created_at.format("%Y-%m-%d %H:%M:%S")
    );
    println!();
    println!("{}", post.content);
    println!();
}

fn print_posts(posts: &[PostResponse], me: bool) {
    let title = if me { "My Posts" } else { "Posts" };
    println!("\n{}:\n", title.bold().underline());

    let rows: Vec<Vec<String>> = posts
        .iter()
        .map(|post| {
            vec![
                post.id.to_string(),
                post.author.linkid.clone(),
                post.visibility.clone(),
                truncate(&post.content, 72),
                format!("{}/{}/{}", post.like_count, post.comment_count, post.share_count),
                post.created_at.format("%Y-%m-%d %H:%M").to_string(),
            ]
        })
        .collect();

    print_table(
        vec!["ID", "Author", "Visibility", "Content", "L/C/S", "Created"],
        rows,
    );
}

fn print_comments(comments: &[CommentResponse]) {
    println!("\n{}:\n", "Comments".bold().underline());

    let rows: Vec<Vec<String>> = comments
        .iter()
        .map(|comment| {
            vec![
                comment.id.to_string(),
                comment.author.linkid.clone(),
                truncate(&comment.content, 72),
                comment.replies.len().to_string(),
                comment.created_at.format("%Y-%m-%d %H:%M").to_string(),
            ]
        })
        .collect();

    print_table(vec!["ID", "Author", "Content", "Replies", "Created"], rows);
}

fn truncate(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        value.to_string()
    } else {
        format!("{}...", value.chars().take(max_chars).collect::<String>())
    }
}
