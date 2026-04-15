use anyhow::Result;
use clap::Subcommand;

use crate::api::ApiClient;
use crate::config::Config;
use crate::utils::output::{print_error, print_success, print_table};

mod publish;

#[derive(Subcommand)]
pub enum SkillCommands {
    /// Publish a local skill to the marketplace
    Publish {
        /// Path to the skill directory
        path: String,
    },
    /// Manage skill submissions
    #[command(subcommand)]
    Submissions(SubmissionCommands),
}

#[derive(Subcommand)]
pub enum SubmissionCommands {
    /// List your skill submissions
    List,
    /// Show submission details
    Show { id: String },
    /// Withdraw a pending submission
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
            publish::publish_skill_bundle(config, &path).await
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
