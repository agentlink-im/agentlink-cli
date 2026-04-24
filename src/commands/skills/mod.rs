use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;
use std::path::PathBuf;

use crate::api::ApiClient;
use crate::config::Config;
use crate::utils::output::{print_error, print_success, print_table};

mod publish;
mod sync;

#[derive(Subcommand)]
pub enum SkillCommands {
    /// Publish a new skill to the marketplace from a local directory
    ///
    /// The directory must contain a SKILL.md with frontmatter (name, version, description).
    /// This creates a new submission that requires admin approval before it becomes public.
    Publish {
        /// Path to the skill directory
        path: String,
    },
    /// Update an existing skill by publishing a new version
    ///
    /// Works the same as `publish`, but intended for skills that are already approved.
    /// Bump the version in SKILL.md before updating. If a pending submission exists for
    /// the same skill name, withdraw it first with `agentlink skills submissions withdraw`.
    Update {
        /// Path to the skill directory
        path: String,
    },
    /// Manage your skill submissions (list, show details, withdraw)
    #[command(subcommand)]
    Submissions(SubmissionCommands),
    /// Sync installed skills to local directory
    ///
    /// Fetches all skills currently installed for your agent from the server
    /// and writes them to the local skills directory (default: ~/.local/share/agentlink/skills/).
    Sync {
        /// Custom directory to sync skills to (optional)
        #[arg(short, long)]
        dir: Option<String>,
    },
    /// Show today's skill upload stats and list today's uploads
    Today {
        /// Page number
        #[arg(short, long, default_value = "1")]
        page: i32,

        /// Items per page
        #[arg(long = "per-page", default_value = "20")]
        per_page: i32,
    },
}

#[derive(Subcommand)]
pub enum SubmissionCommands {
    /// List all your skill submissions and their current status
    List,
    /// Show detailed information about a specific submission
    Show { id: String },
    /// Withdraw a pending submission so you can re-submit a new version
    Withdraw { id: String },
}

pub async fn execute(
    command: SkillCommands,
    config: &Config,
    format: crate::OutputFormat,
) -> Result<()> {
    match command {
        SkillCommands::Publish { path } => {
            ensure_authenticated(config)?;
            publish::publish_skill_bundle(config, &path, false).await
        }
        SkillCommands::Update { path } => {
            ensure_authenticated(config)?;
            publish::publish_skill_bundle(config, &path, true).await
        }
        SkillCommands::Sync { dir } => {
            ensure_authenticated(config)?;
            let dir_path = dir.map(PathBuf::from);
            sync::sync_skills(config, dir_path).await
        }
        SkillCommands::Submissions(sub_cmd) => {
            ensure_authenticated(config)?;
            let client = ApiClient::new(config)?;
            match sub_cmd {
                SubmissionCommands::List => {
                    match client.list_skill_submissions().await {
                        Ok(submissions) => {
                            if submissions.is_empty() {
                                println!("No skill submissions found.");
                            } else {
                                match format {
                                    crate::OutputFormat::Json => {
                                        println!("{}", serde_json::to_string_pretty(&submissions)?);
                                    }
                                    crate::OutputFormat::Yaml => {
                                        println!("{}", serde_yaml::to_string(&submissions)?);
                                    }
                                    _ => {
                                        let headers = vec!["ID", "Name", "Version", "Status", "Created"];
                                        let mut rows = Vec::new();
                                        for s in submissions {
                                            rows.push(vec![
                                                s.id.to_string(),
                                                s.name,
                                                s.version,
                                                format!("{:?}", s.status),
                                                s.created_at.to_rfc3339(),
                                            ]);
                                        }
                                        print_table(headers, rows);
                                    }
                                }
                            }
                            Ok(())
                        }
                        Err(e) => {
                            print_error(&format!("Failed to list submissions: {}", e));
                            Ok(())
                        }
                    }
                }
                SubmissionCommands::Show { id } => {
                    match client.get_skill_submission(&id).await {
                        Ok(submission) => {
                            match format {
                                crate::OutputFormat::Json => {
                                    println!("{}", serde_json::to_string_pretty(&submission)?);
                                }
                                crate::OutputFormat::Yaml => {
                                    println!("{}", serde_yaml::to_string(&submission)?);
                                }
                                _ => {
                                    println!("Submission: {}", submission.name);
                                    println!("  ID:        {}", submission.id);
                                    println!("  Version:   {}", submission.version);
                                    println!("  Status:    {:?}", submission.status);
                                    println!("  Category:  {}", submission.category.unwrap_or_default());
                                    if let Some(notes) = submission.review_notes {
                                        println!("  Review:    {}", notes);
                                    }
                                }
                            }
                            Ok(())
                        }
                        Err(e) => {
                            print_error(&format!("Failed to get submission: {}", e));
                            Ok(())
                        }
                    }
                }
                SubmissionCommands::Withdraw { id } => {
                    match client.withdraw_skill_submission(&id).await {
                        Ok(_) => {
                            print_success("Submission withdrawn successfully.");
                            Ok(())
                        }
                        Err(e) => {
                            print_error(&format!("Failed to withdraw submission: {}", e));
                            Ok(())
                        }
                    }
                }
            }
        }
        SkillCommands::Today { page, per_page } => {
            ensure_authenticated(config)?;
            let client = ApiClient::new(config)?;

            // Fetch stats
            match client.get_skill_upload_stats().await {
                Ok(stats) => {
                    println!("\n{}", "Skill Upload Stats".bold().underline());
                    println!("  {}: {}", "Total Uploads".bold(), stats.total_uploads);
                    println!("  {}: {}", "Today Uploads".bold(), stats.today_uploads);
                }
                Err(e) => {
                    print_error(&format!("Failed to get upload stats: {}", e));
                    return Ok(());
                }
            }

            // Fetch today's uploads
            match client
                .list_today_skill_uploads(agentlink_protocol::skill::TodaySkillUploadQuery {
                    page: Some(page),
                    per_page: Some(per_page),
                })
                .await
            {
                Ok(response) => {
                    if response.data.is_empty() {
                        println!("\n{}", "No skill uploads today.".yellow());
                        return Ok(());
                    }

                    match format {
                        crate::OutputFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&response)?);
                        }
                        crate::OutputFormat::Yaml => {
                            println!("{}", serde_yaml::to_string(&response)?);
                        }
                        _ => {
                            println!("\n{}", "Today's Uploads".bold().underline());
                            let headers = vec!["ID", "Name", "Version", "Status", "Created"];
                            let rows: Vec<Vec<String>> = response
                                .data
                                .iter()
                                .map(|item| {
                                    vec![
                                        item.id.to_string(),
                                        item.name.clone(),
                                        item.version.clone(),
                                        format!("{:?}", item.status).to_lowercase(),
                                        item.created_at.format("%Y-%m-%d %H:%M").to_string(),
                                    ]
                                })
                                .collect();
                            print_table(headers, rows);
                            println!(
                                "\nPage {}/{} | Total: {}",
                                response.page, response.total_pages, response.total
                            );
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    print_error(&format!("Failed to list today's uploads: {}", e));
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
