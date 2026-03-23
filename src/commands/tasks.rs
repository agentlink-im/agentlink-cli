use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;

use crate::api::ApiClient;
use crate::config::Config;
use crate::models::Task;
use crate::utils::output::{print_error, print_success, print_table, OutputFormat};

#[derive(Subcommand)]
pub enum TaskCommands {
    /// 列出任务
    List {
        /// 页码
        #[arg(short, long, default_value = "1")]
        page: i64,

        /// 每页数量
        #[arg(short, long, default_value = "20")]
        per_page: i64,

        /// 搜索关键词
        #[arg(short, long)]
        query: Option<String>,
    },

    /// 查看任务详情
    Show {
        /// 任务 ID
        id: String,
    },

    /// 申请任务
    Apply {
        /// 任务 ID
        id: String,

        /// 申请说明
        #[arg(short, long)]
        cover_letter: Option<String>,

        /// 期望预算
        #[arg(short, long)]
        budget: Option<i64>,

        /// 预计完成天数
        #[arg(short, long)]
        days: Option<i32>,
    },

    /// 查看我的任务
    MyTasks {
        /// 页码
        #[arg(short, long, default_value = "1")]
        page: i64,

        /// 每页数量
        #[arg(short, long, default_value = "20")]
        per_page: i64,
    },
}

pub async fn execute(
    command: TaskCommands,
    config: &Config,
    format: crate::OutputFormat,
) -> Result<()> {
    // 检查是否已认证
    if !config.is_authenticated() {
        print_error("You must be logged in to use this command.");
        println!("Run {} to authenticate.", "agentlink auth login".cyan());
        return Ok(());
    }

    let client = ApiClient::new(config)?;

    match command {
        TaskCommands::List {
            page,
            per_page,
            query: _,
        } => {
            match client.list_tasks(Some(page), Some(per_page)).await {
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
                                response.meta.page,
                                response.meta.total_pages
                            );

                            let tasks: Vec<Vec<String>> = response
                                .data
                                .iter()
                                .map(|t| {
                                    vec![
                                        t.id.clone(),
                                        t.title.clone(),
                                        t.status.clone(),
                                        format_budget(&t),
                                        format_date(&t.created_at),
                                    ]
                                })
                                .collect();

                            print_table(
                                vec!["ID", "Title", "Status", "Budget", "Created"],
                                tasks,
                            );

                            println!(
                                "\nShowing {} of {} tasks",
                                response.data.len(),
                                response.meta.total
                            );
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    print_error(&format!("Failed to list tasks: {}", e));
                    Ok(())
                }
            }
        }

        TaskCommands::Show { id } => {
            match client.get_task(&id).await {
                Ok(task) => {
                    match format {
                        crate::OutputFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&task)?);
                        }
                        crate::OutputFormat::Yaml => {
                            println!("{}", serde_yaml::to_string(&task)?);
                        }
                        _ => {
                            print_task_details(&task);
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    print_error(&format!("Failed to get task: {}", e));
                    Ok(())
                }
            }
        }

        TaskCommands::Apply {
            id,
            cover_letter,
            budget,
            days,
        } => {
            let body = serde_json::json!({
                "coverLetter": cover_letter,
                "proposedBudget": budget,
                "estimatedDays": days,
            });

            match client.apply_to_task(&id, body).await {
                Ok(application) => {
                    print_success("Application submitted successfully!");
                    println!("  {}: {}", "Application ID".bold(), application.id);
                    println!("  {}: {}", "Status".bold(), application.status);
                    Ok(())
                }
                Err(e) => {
                    print_error(&format!("Failed to apply: {}", e));
                    Ok(())
                }
            }
        }

        TaskCommands::MyTasks { page, per_page } => {
            match client.list_tasks(Some(page), Some(per_page)).await {
                Ok(response) => {
                    if response.data.is_empty() {
                        println!("{}", "You have no tasks.".yellow());
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
                            println!("\n{}:\n", "My Tasks".bold().underline());

                            let tasks: Vec<Vec<String>> = response
                                .data
                                .iter()
                                .map(|t| {
                                    vec![
                                        t.id.clone(),
                                        t.title.clone(),
                                        t.status.clone(),
                                        format_budget(&t),
                                        format_date(&t.created_at),
                                    ]
                                })
                                .collect();

                            print_table(
                                vec!["ID", "Title", "Status", "Budget", "Created"],
                                tasks,
                            );
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    print_error(&format!("Failed to get my tasks: {}", e));
                    Ok(())
                }
            }
        }
    }
}

fn format_budget(task: &Task) -> String {
    match (task.budget_min, task.budget_max) {
        (Some(min), Some(max)) => format!("{}-{} {}", min, max, task.currency),
        (Some(min), None) => format!("{}+ {}", min, task.currency),
        (None, Some(max)) => format!("Up to {} {}", max, task.currency),
        _ => "Not specified".to_string(),
    }
}

fn format_date(date: &chrono::DateTime<chrono::Utc>) -> String {
    date.format("%Y-%m-%d").to_string()
}

fn print_task_details(task: &Task) {
    println!("\n{}", "Task Details".bold().underline());
    println!();
    println!("{}: {}", "ID".bold(), task.id);
    println!("{}: {}", "Title".bold(), task.title);
    println!("{}: {}", "Status".bold(), task.status);
    println!("{}: {}", "Type".bold(), task.kind);
    println!("{}: {}", "Budget".bold(), format_budget(task));
    println!(
        "{}: {}",
        "Location".bold(),
        task.location_type
    );
    if let Some(deadline) = task.deadline {
        println!("{}: {}", "Deadline".bold(), deadline.format("%Y-%m-%d %H:%M"));
    }
    println!();
    println!("{}", "Description:".bold());
    println!("{}", task.description);
    println!();
    println!("{}: {}", "Created".bold(), task.created_at.format("%Y-%m-%d %H:%M"));
    println!();

    if !task.skills.is_empty() {
        println!("{}:", "Required Skills".bold());
        for skill in &task.skills {
            println!("  • {}", skill.name);
        }
    }
}
