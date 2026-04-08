use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;

use crate::api::ApiClient;
use crate::config::Config;
use crate::models::{CreateApplicationRequest, TaskResponse};
use crate::utils::output::{print_error, print_success, print_table};

#[derive(Subcommand)]
pub enum TaskCommands {
    /// 列出任务
    List {
        #[arg(short, long, default_value = "1")]
        page: i64,

        #[arg(short, long, default_value = "20")]
        per_page: i64,

        #[arg(short, long)]
        query: Option<String>,
    },

    /// 查看任务详情
    Show { id: String },

    /// 申请任务
    Apply {
        id: String,

        #[arg(short, long)]
        cover_letter: Option<String>,

        #[arg(short, long)]
        budget: Option<f64>,

        #[arg(short, long)]
        days: Option<i32>,
    },

    /// 查看我发布的任务
    MyTasks,
}

pub async fn execute(
    command: TaskCommands,
    config: &Config,
    format: crate::OutputFormat,
) -> Result<()> {
    let client = ApiClient::new(config)?;

    match command {
        TaskCommands::List {
            page,
            per_page,
            query,
        } => match client
            .list_tasks(agentlink_protocol::task::TaskSearchQuery {
                q: query,
                task_type: None,
                status: None,
                budget_min: None,
                budget_max: None,
                skill_ids: None,
                page: Some(page),
                per_page: Some(per_page),
            })
            .await
        {
            Ok(response) => {
                if response.data.is_empty() {
                    println!("{}", "No tasks found.".yellow());
                    return Ok(());
                }

                match format {
                    crate::OutputFormat::Json => {
                        println!("{}", serde_json::to_string_pretty(&response.data)?);
                    }
                    crate::OutputFormat::Yaml => {
                        println!("{}", serde_yaml::to_string(&response.data)?);
                    }
                    _ => {
                        println!(
                            "\n{} (Page {}/{}):\n",
                            "Available Tasks".bold().underline(),
                            response.page,
                            response.total_pages
                        );

                        let tasks: Vec<Vec<String>> = response
                            .data
                            .iter()
                            .map(|task| {
                                vec![
                                    task.id.to_string(),
                                    task.title.clone(),
                                    format!("{:?}", task.status).to_lowercase(),
                                    format_budget(task),
                                    format_date(&task.created_at),
                                ]
                            })
                            .collect();

                        print_table(vec!["ID", "Title", "Status", "Budget", "Created"], tasks);

                        println!(
                            "\nShowing {} of {} tasks",
                            response.data.len(),
                            response.total
                        );
                    }
                }

                Ok(())
            }
            Err(error) => {
                print_error(&format!("Failed to list tasks: {}", error));
                Ok(())
            }
        },
        TaskCommands::Show { id } => match client.get_task(&id).await {
            Ok(task) => {
                match format {
                    crate::OutputFormat::Json => {
                        println!("{}", serde_json::to_string_pretty(&task)?);
                    }
                    crate::OutputFormat::Yaml => {
                        println!("{}", serde_yaml::to_string(&task)?);
                    }
                    _ => print_task_details(&task),
                }
                Ok(())
            }
            Err(error) => {
                print_error(&format!("Failed to get task: {}", error));
                Ok(())
            }
        },
        TaskCommands::Apply {
            id,
            cover_letter,
            budget,
            days,
        } => {
            ensure_authenticated(config)?;

            let body = CreateApplicationRequest {
                task_id: None,
                cover_letter,
                proposed_budget: budget.and_then(rust_decimal::Decimal::from_f64_retain),
                estimated_days: days,
            };

            match client.apply_to_task(&id, body).await {
                Ok(application) => {
                    print_success("Application submitted successfully.");
                    println!("{}: {}", "Application ID".bold(), application.id);
                    println!(
                        "{}: {}",
                        "Status".bold(),
                        format!("{:?}", application.status).to_lowercase()
                    );
                    Ok(())
                }
                Err(error) => {
                    print_error(&format!("Failed to apply: {}", error));
                    Ok(())
                }
            }
        }
        TaskCommands::MyTasks => {
            ensure_authenticated(config)?;

            match client.get_my_tasks().await {
                Ok(response) => {
                    if response.tasks.is_empty() {
                        println!("{}", "You have no tasks.".yellow());
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
                            println!("\n{}:\n", "My Tasks".bold().underline());

                            let tasks: Vec<Vec<String>> = response
                                .tasks
                                .iter()
                                .map(|task| {
                                    vec![
                                        task.id.to_string(),
                                        task.title.clone(),
                                        format!("{:?}", task.status).to_lowercase(),
                                        format_budget(task),
                                        format_date(&task.created_at),
                                    ]
                                })
                                .collect();

                            print_table(vec!["ID", "Title", "Status", "Budget", "Created"], tasks);

                            println!(
                                "\nOpen: {}  In Progress: {}  Completed: {}",
                                response.stats.open,
                                response.stats.in_progress,
                                response.stats.completed
                            );
                        }
                    }
                    Ok(())
                }
                Err(error) => {
                    print_error(&format!("Failed to get my tasks: {}", error));
                    Ok(())
                }
            }
        }
    }
}

fn ensure_authenticated(config: &Config) -> Result<()> {
    if config.is_authenticated() {
        Ok(())
    } else {
        anyhow::bail!("Not authenticated. Run 'agentlink auth login' first.")
    }
}

fn format_budget(task: &TaskResponse) -> String {
    match (&task.budget_min, &task.budget_max) {
        (Some(min), Some(max)) => format!("{}-{} {}", min, max, task.currency),
        (Some(min), None) => format!("{}+ {}", min, task.currency),
        (None, Some(max)) => format!("Up to {} {}", max, task.currency),
        (None, None) => "Not specified".to_string(),
    }
}

fn format_date(date: &chrono::DateTime<chrono::Utc>) -> String {
    date.format("%Y-%m-%d").to_string()
}

fn print_task_details(task: &TaskResponse) {
    println!("\n{}", "Task Details".bold().underline());
    println!();
    println!("{}: {}", "ID".bold(), task.id);
    println!("{}: {}", "Title".bold(), task.title);
    println!(
        "{}: {}",
        "Status".bold(),
        format!("{:?}", task.status).to_lowercase()
    );
    println!(
        "{}: {}",
        "Type".bold(),
        format!("{:?}", task.task_type).to_lowercase()
    );
    println!("{}: {}", "Budget".bold(), format_budget(task));
    println!("{}: {}", "Location".bold(), task.location_type);
    if let Some(deadline) = task.deadline {
        println!(
            "{}: {}",
            "Deadline".bold(),
            deadline.format("%Y-%m-%d %H:%M")
        );
    }
    println!("{}: {}", "Applications".bold(), task.application_count);
    println!();
    println!("{}", "Description:".bold());
    println!("{}", task.description);
    println!();

    if let Some(creator) = &task.creator {
        println!(
            "{}: {} ({})",
            "Creator".bold(),
            creator.linkid,
            creator.user_type
        );
    }

    if !task.skills.is_empty() {
        println!();
        println!("{}:", "Required Skills".bold());
        for skill in &task.skills {
            println!("  • {} ({})", skill.name, skill.category);
        }
    }
}
