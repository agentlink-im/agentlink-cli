use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, Utc};
use colored::Colorize;
use dialoguer::{Confirm, Editor, Input, MultiSelect, Select};
use rust_decimal::Decimal;
use std::path::PathBuf;
use uuid::Uuid;

use crate::api::ApiClient;
use crate::config::Config;
use crate::utils::output::{print_error, print_success};

/// 任务发布向导
pub struct TaskPublishWizard {
    draft: TaskDraft,
    config: Config,
}

/// 任务草稿
#[derive(Debug, Clone, Default)]
pub struct TaskDraft {
    pub title: Option<String>,
    pub kind: Option<TaskType>,
    pub description: Option<String>,
    pub budget_min: Option<Decimal>,
    pub budget_max: Option<Decimal>,
    pub currency: String,
    pub location_type: Option<String>,
    pub deadline: Option<DateTime<Utc>>,
    pub skill_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskType {
    OneTime,
    Project,
    LongTerm,
    Consultation,
}

impl TaskType {
    fn as_str(&self) -> &'static str {
        match self {
            TaskType::OneTime => "one_time",
            TaskType::Project => "project",
            TaskType::LongTerm => "long_term",
            TaskType::Consultation => "consultation",
        }
    }

    fn display_name(&self) -> &'static str {
        match self {
            TaskType::OneTime => "One Time (一次性任务)",
            TaskType::Project => "Project (项目制)",
            TaskType::LongTerm => "Long Term (长期合作)",
            TaskType::Consultation => "Consultation (咨询)",
        }
    }

    fn all() -> Vec<TaskType> {
        vec![TaskType::OneTime, TaskType::Project, TaskType::LongTerm, TaskType::Consultation]
    }
}

/// 草稿存储路径
fn draft_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().context("Failed to get config directory")?;
    Ok(config_dir.join("agentlink").join("task_draft.toml"))
}

impl TaskDraft {
    /// 保存草稿到文件
    pub fn save(&self) -> Result<()> {
        let path = draft_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// 从文件加载草稿
    pub fn load() -> Result<Option<Self>> {
        let path = draft_path()?;
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)?;
        let draft: TaskDraft = toml::from_str(&content)?;
        Ok(Some(draft))
    }

    /// 清除草稿
    pub fn clear() -> Result<()> {
        let path = draft_path()?;
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// 检查是否有草稿
    pub fn exists() -> bool {
        draft_path().map(|p| p.exists()).unwrap_or(false)
    }
}

impl TaskPublishWizard {
    pub fn new(config: Config) -> Self {
        Self {
            draft: TaskDraft::default(),
            config,
        }
    }

    #[allow(dead_code)]
    pub fn with_draft(config: Config, draft: TaskDraft) -> Self {
        Self { draft, config }
    }

    /// 运行发布向导
    pub async fn run(&mut self) -> Result<()> {
        println!("{}", "\n📝 Task Publish Wizard\n".bold().underline());
        println!("{}", "This wizard will guide you through creating a new task.\n".dimmed());

        // 检查是否有草稿
        if TaskDraft::exists() {
            let should_resume = Confirm::new()
                .with_prompt("A draft was found. Do you want to resume from where you left off?")
                .default(true)
                .interact()?;

            if should_resume {
                if let Some(draft) = TaskDraft::load()? {
                    self.draft = draft;
                    println!("{}", "✓ Draft loaded.\n".green());
                }
            } else {
                TaskDraft::clear()?;
            }
        }

        // Step 1: 基本信息
        if self.draft.title.is_none() {
            self.collect_basic_info().await?;
            self.draft.save()?;
        }

        // Step 2: 任务描述
        if self.draft.description.is_none() {
            self.collect_description().await?;
            self.draft.save()?;
        }

        // Step 3: 预算设置
        if self.draft.budget_min.is_none() && self.draft.budget_max.is_none() {
            self.collect_budget().await?;
            self.draft.save()?;
        }

        // Step 4: 工作设置
        if self.draft.location_type.is_none() {
            self.collect_work_settings().await?;
            self.draft.save()?;
        }

        // Step 5: 技能选择
        self.collect_skills().await?;
        self.draft.save()?;

        // Step 6: 确认和发布
        let should_publish = self.review_and_confirm().await?;

        if should_publish {
            self.publish_task().await?;
            TaskDraft::clear()?;
        } else {
            self.draft.save()?;
            println!("\n{}", "Draft saved. You can resume later.".yellow());
        }

        Ok(())
    }

    /// Step 1: 收集基本信息
    async fn collect_basic_info(&mut self) -> Result<()> {
        println!("{}", "\n📋 Step 1: Basic Information\n".bold());

        // 标题
        let title: String = Input::new()
            .with_prompt("Task title")
            .validate_with(|input: &String| {
                let len = input.trim().len();
                if len < 5 {
                    Err("Title must be at least 5 characters".to_string())
                } else if len > 200 {
                    Err("Title must be at most 200 characters".to_string())
                } else {
                    Ok(())
                }
            })
            .interact_text()?;

        // 任务类型
        let task_types = TaskType::all();
        let type_names: Vec<&str> = task_types.iter().map(|t| t.display_name()).collect();
        let type_index = Select::new()
            .with_prompt("Task type")
            .items(&type_names)
            .default(0)
            .interact()?;

        self.draft.title = Some(title.trim().to_string());
        self.draft.kind = Some(task_types[type_index]);

        Ok(())
    }

    /// Step 2: 收集任务描述
    async fn collect_description(&mut self) -> Result<()> {
        println!("{}", "\n📝 Step 2: Task Description\n".bold());
        println!("{}", "You can write the description in your preferred editor.".dimmed());
        println!("{}", "The description should be at least 20 characters.\n".dimmed());

        loop {
            let description = Editor::new()
                .extension(".md")
                .edit("")?
                .unwrap_or_default();

            let trimmed = description.trim();
            if trimmed.len() < 20 {
                println!(
                    "{}",
                    format!("Description is too short ({} chars). Minimum is 20 characters.", trimmed.len()).red()
                );
                let retry = Confirm::new()
                    .with_prompt("Try again?")
                    .default(true)
                    .interact()?;
                if !retry {
                    anyhow::bail!("Task description is required");
                }
                continue;
            }

            // 预览
            println!("\n{}", "Description preview:".bold());
            println!("{}", "─".repeat(60).dimmed());
            println!("{}", trimmed);
            println!("{}", "─".repeat(60).dimmed());

            let confirm = Confirm::new()
                .with_prompt("Is this description correct?")
                .default(true)
                .interact()?;

            if confirm {
                self.draft.description = Some(trimmed.to_string());
                break;
            }
        }

        Ok(())
    }

    /// Step 3: 收集预算信息
    async fn collect_budget(&mut self) -> Result<()> {
        println!("{}", "\n💰 Step 3: Budget Settings\n".bold());

        // 货币
        let currencies = vec!["USD", "CNY", "EUR", "GBP"];
        let currency_index = Select::new()
            .with_prompt("Currency")
            .items(&currencies)
            .default(0)
            .interact()?;
        self.draft.currency = currencies[currency_index].to_string();

        // 是否设置预算
        let has_budget = Confirm::new()
            .with_prompt("Do you want to set a budget range?")
            .default(true)
            .interact()?;

        if has_budget {
            // 最低预算
            let min_budget: f64 = Input::new()
                .with_prompt("Minimum budget")
                .default(100.0)
                .interact_text()?;

            // 最高预算
            let max_budget: f64 = Input::new()
                .with_prompt("Maximum budget")
                .default(min_budget * 1.5)
                .validate_with(|input: &f64| {
                    if *input < min_budget {
                        Err("Maximum budget must be greater than or equal to minimum budget".to_string())
                    } else {
                        Ok(())
                    }
                })
                .interact_text()?;

            self.draft.budget_min = Decimal::from_f64_retain(min_budget);
            self.draft.budget_max = Decimal::from_f64_retain(max_budget);
        }

        Ok(())
    }

    /// Step 4: 收集工作设置
    async fn collect_work_settings(&mut self) -> Result<()> {
        println!("{}", "\n🏢 Step 4: Work Settings\n".bold());

        // 工作地点
        let locations = vec![
            ("remote", "Remote (远程)"),
            ("onsite", "Onsite (现场)"),
            ("hybrid", "Hybrid (混合)"),
        ];
        let location_names: Vec<&str> = locations.iter().map(|(_, name)| *name).collect();
        let location_index = Select::new()
            .with_prompt("Location type")
            .items(&location_names)
            .default(0)
            .interact()?;
        self.draft.location_type = Some(locations[location_index].0.to_string());

        // 截止日期
        let has_deadline = Confirm::new()
            .with_prompt("Do you want to set a deadline?")
            .default(false)
            .interact()?;

        if has_deadline {
            loop {
                let date_str: String = Input::new()
                    .with_prompt("Deadline (YYYY-MM-DD)")
                    .interact_text()?;

                match NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
                    Ok(date) => {
                        let datetime = date.and_hms_opt(23, 59, 59).unwrap();
                        let utc_datetime = DateTime::<Utc>::from_naive_utc_and_offset(datetime, Utc);
                        
                        if utc_datetime < Utc::now() {
                            println!("{}", "Deadline must be in the future".red());
                            continue;
                        }
                        
                        self.draft.deadline = Some(utc_datetime);
                        break;
                    }
                    Err(_) => {
                        println!("{}", "Invalid date format. Please use YYYY-MM-DD".red());
                    }
                }
            }
        }

        Ok(())
    }

    /// Step 5: 收集技能
    async fn collect_skills(&mut self) -> Result<()> {
        println!("{}", "\n🎯 Step 5: Required Skills\n".bold());

        let client = ApiClient::new(&self.config)?;
        
        match client.list_skills().await {
            Ok(skills) => {
                if skills.is_empty() {
                    println!("{}", "No skills available in the catalog.".yellow());
                    return Ok(());
                }

                let skill_names: Vec<String> = skills
                    .iter()
                    .map(|s| format!("{} ({})", s.name, s.category))
                    .collect();

                let selected = MultiSelect::new()
                    .with_prompt("Select required skills (Space to select, Enter to confirm)")
                    .items(&skill_names)
                    .interact()?;

                self.draft.skill_ids = selected.iter().map(|&i| skills[i].id).collect();
            }
            Err(e) => {
                println!("{}", format!("Warning: Failed to load skills: {}", e).yellow());
                let skip = Confirm::new()
                    .with_prompt("Continue without skills?")
                    .default(true)
                    .interact()?;
                if !skip {
                    anyhow::bail!("Skill selection is required");
                }
            }
        }

        Ok(())
    }

    /// Step 6: 审核和确认
    async fn review_and_confirm(&mut self) -> Result<bool> {
        loop {
            println!("{}", "\n👁️  Step 6: Review & Confirm\n".bold());

            let kind = self.draft.kind.as_ref().unwrap();
            let title = self.draft.title.as_ref().unwrap();
            let description = self.draft.description.as_ref().unwrap();
            let location = self.draft.location_type.as_deref().unwrap_or("Not specified");

            println!("{}", "Task Summary:".bold().underline());
            println!("  {}: {}", "Title".bold(), title);
            println!("  {}: {}", "Type".bold(), kind.display_name());
            println!(
                "  {}: {}",
                "Budget".bold(),
                match (self.draft.budget_min, self.draft.budget_max) {
                    (Some(min), Some(max)) => format!("{} {} - {}", min, self.draft.currency, max),
                    (Some(min), None) => format!("{} {}+", min, self.draft.currency),
                    (None, Some(max)) => format!("Up to {} {}", max, self.draft.currency),
                    (None, None) => "Not specified".to_string(),
                }
            );
            println!("  {}: {}", "Location".bold(), location);
            println!(
                "  {}: {}",
                "Deadline".bold(),
                self.draft
                    .deadline
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| "Not specified".to_string())
            );
            println!(
                "  {}: {}",
                "Skills".bold(),
                if self.draft.skill_ids.is_empty() {
                    "None".to_string()
                } else {
                    format!("{} selected", self.draft.skill_ids.len())
                }
            );
            println!();
            println!("{}", "Description:".bold());
            println!("{}", "─".repeat(60).dimmed());
            // 截断长描述
            let desc_preview: String = description
                .chars()
                .take(500)
                .collect::<String>()
                .lines()
                .take(10)
                .collect::<Vec<_>>()
                .join("\n");
            println!("{}", desc_preview);
            if description.len() > 500 || description.lines().count() > 10 {
                println!("\n{}...", "(truncated)".dimmed());
            }
            println!("{}", "─".repeat(60).dimmed());
            println!();

            let options = vec!["Publish now", "Save as draft and exit", "Edit basic info", "Edit description", "Edit budget", "Edit work settings", "Edit skills"];
            let choice = Select::new()
                .with_prompt("What would you like to do?")
                .items(&options)
                .default(0)
                .interact()?;

            match choice {
                0 => return Ok(true),  // Publish
                1 => return Ok(false), // Save draft and exit
                2 => {
                    // Edit basic info
                    self.collect_basic_info().await?;
                    self.draft.save()?;
                }
                3 => {
                    // Edit description
                    self.collect_description().await?;
                    self.draft.save()?;
                }
                4 => {
                    // Edit budget
                    self.collect_budget().await?;
                    self.draft.save()?;
                }
                5 => {
                    // Edit work settings
                    self.collect_work_settings().await?;
                    self.draft.save()?;
                }
                6 => {
                    // Edit skills
                    self.collect_skills().await?;
                    self.draft.save()?;
                }
                _ => unreachable!(),
            }
        }
    }

    /// 发布任务
    async fn publish_task(&self) -> Result<()> {
        println!("\n{}", "Publishing task...".dimmed());

        let client = ApiClient::new(&self.config)?;

        let request = agentlink_protocol::task::CreateTaskRequest {
            title: self.draft.title.clone().unwrap(),
            description: self.draft.description.clone().unwrap(),
            kind: match self.draft.kind.unwrap() {
                TaskType::OneTime => agentlink_protocol::TaskType::OneTime,
                TaskType::Project => agentlink_protocol::TaskType::Project,
                TaskType::LongTerm => agentlink_protocol::TaskType::LongTerm,
                TaskType::Consultation => agentlink_protocol::TaskType::Consultation,
            },
            budget_min: self.draft.budget_min,
            budget_max: self.draft.budget_max,
            currency: Some(self.draft.currency.clone()),
            deadline: self.draft.deadline,
            location_type: self.draft.location_type.clone(),
            skill_ids: if self.draft.skill_ids.is_empty() {
                None
            } else {
                Some(self.draft.skill_ids.clone())
            },
        };

        match client.create_task(request).await {
            Ok(task) => {
                print_success("Task published successfully!");
                println!("\n  {}: {}", "Task ID".bold(), task.id);
                println!("  {}: {}", "Title".bold(), task.title);
                println!(
                    "  {}: https://agentlink.chat/tasks/{}",
                    "View".bold(),
                    task.id
                );
                Ok(())
            }
            Err(e) => {
                print_error(&format!("Failed to publish task: {}", e));
                Err(e)
            }
        }
    }
}

// 为 TaskDraft 实现序列化/反序列化
impl serde::Serialize for TaskDraft {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("TaskDraft", 9)?;
        state.serialize_field("title", &self.title)?;
        state.serialize_field("kind", &self.kind.map(|k| k.as_str()))?;
        state.serialize_field("description", &self.description)?;
        state.serialize_field("budget_min", &self.budget_min)?;
        state.serialize_field("budget_max", &self.budget_max)?;
        state.serialize_field("currency", &self.currency)?;
        state.serialize_field("location_type", &self.location_type)?;
        state.serialize_field("deadline", &self.deadline)?;
        state.serialize_field("skill_ids", &self.skill_ids)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for TaskDraft {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct TaskDraftHelper {
            title: Option<String>,
            kind: Option<String>,
            description: Option<String>,
            budget_min: Option<Decimal>,
            budget_max: Option<Decimal>,
            currency: Option<String>,
            location_type: Option<String>,
            deadline: Option<DateTime<Utc>>,
            skill_ids: Option<Vec<Uuid>>,
        }

        let helper = TaskDraftHelper::deserialize(deserializer)?;
        Ok(TaskDraft {
            title: helper.title,
            kind: helper.kind.and_then(|k| match k.as_str() {
                "one_time" => Some(TaskType::OneTime),
                "project" => Some(TaskType::Project),
                "long_term" => Some(TaskType::LongTerm),
                "consultation" => Some(TaskType::Consultation),
                _ => None,
            }),
            description: helper.description,
            budget_min: helper.budget_min,
            budget_max: helper.budget_max,
            currency: helper.currency.unwrap_or_else(|| "USD".to_string()),
            location_type: helper.location_type,
            deadline: helper.deadline,
            skill_ids: helper.skill_ids.unwrap_or_default(),
        })
    }
}
