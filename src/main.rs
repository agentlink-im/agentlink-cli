use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use tracing::{debug, info};

mod api;
mod commands;
mod config;
mod models;
mod utils;

use commands::{
    agent::AgentCommands, api_key::ApiKeyCommands, config::ConfigCommands,
    messages::MessageCommands, notifications::NotificationCommands, tasks::TaskCommands,
    update::UpdateCommands,
};

/// AgentLink CLI - 面向 AI Agent 的 AgentLink 命令行工具
#[derive(Parser)]
#[command(
    name = "agentlink",
    about = "Agent-only CLI for AgentLink",
    long_about = None,
    version,
    author
)]
#[command(propagate_version = true)]
struct Cli {
    /// 配置文件路径
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,

    /// API 基础地址（默认 https://beta-api.agentlink.chat/）
    #[arg(short = 's', long = "base-url", env = "AGENTLINK_BASE_URL")]
    base_url: Option<String>,

    /// Agent API Key（sk_*；通过 Authorization: Bearer 发送）
    #[arg(long = "api-key", env = "AGENTLINK_API_KEY")]
    api_key: Option<String>,

    /// 输出格式
    #[arg(short, long, value_enum, default_value = "table")]
    format: OutputFormat,

    /// 详细输出
    #[arg(short, long)]
    verbose: bool,

    /// 静默模式
    #[arg(short, long)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Agent API Key 管理
    #[command(alias = "key")]
    ApiKey {
        #[command(subcommand)]
        command: ApiKeyCommands,
    },

    /// 配置管理
    #[command(alias = "c")]
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// 任务管理
    #[command(alias = "t")]
    Tasks {
        #[command(subcommand)]
        command: TaskCommands,
    },

    /// 消息管理
    #[command(alias = "m")]
    Messages {
        #[command(subcommand)]
        command: MessageCommands,
    },

    /// 通知管理
    #[command(alias = "n")]
    Notifications {
        #[command(subcommand)]
        command: NotificationCommands,
    },

    /// Agent 专属命令
    #[command(alias = "ag")]
    Agent {
        #[command(subcommand)]
        command: AgentCommands,
    },

    /// 生成自动补全脚本
    Completion {
        /// Shell 类型
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },

    /// 显示版本信息
    Version,

    /// 检查和更新 CLI 版本
    #[command(name = "self-update")]
    SelfUpdate {
        #[command(subcommand)]
        command: UpdateCommands,
    },
}

#[derive(Clone, Copy, Debug, Default, clap::ValueEnum)]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
    Yaml,
    Plain,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // 初始化日志
    let log_level = if cli.verbose {
        "debug"
    } else if cli.quiet {
        "error"
    } else {
        "info"
    };

    tracing_subscriber::fmt()
        .with_env_filter(format!("agentlink_cli={}", log_level))
        .init();

    debug!("Starting AgentLink CLI");

    // 加载配置
    let mut config = config::Config::load(cli.config.as_deref())?;

    // 命令行参数覆盖配置文件
    if let Some(base_url) = cli
        .base_url
        .or_else(|| std::env::var("AGENTLINK_SERVER").ok())
    {
        config.server_url = base_url;
    }
    if let Some(api_key) = cli.api_key {
        config.set_runtime_api_key(Some(api_key))?;
    }

    info!("Using server: {}", config.server_url);

    // 执行命令
    match cli.command {
        Commands::ApiKey { command } => commands::api_key::execute(command, &mut config).await,
        Commands::Config { command } => commands::config::execute(command, &mut config).await,
        Commands::Tasks { command } => commands::tasks::execute(command, &config, cli.format).await,
        Commands::Messages { command } => {
            commands::messages::execute(command, &config, cli.format).await
        }
        Commands::Notifications { command } => {
            commands::notifications::execute(command, &config, cli.format).await
        }
        Commands::Agent { command } => commands::agent::execute(command, &config, cli.format).await,
        Commands::Completion { shell } => {
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            clap_complete::generate(shell, &mut cmd, name, &mut std::io::stdout());
            Ok(())
        }
        Commands::Version => {
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Commands::SelfUpdate { command } => commands::update::execute(command).await,
    }
}
