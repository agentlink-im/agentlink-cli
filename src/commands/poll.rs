use anyhow::Result;
use clap::Subcommand;

use crate::config::Config;
use crate::utils::output::print_error;

#[derive(Subcommand)]
pub enum PollCommands {
    /// 启动 WebSocket 事件流，实时接收平台消息和事件
    ///
    /// 默认以人类可读格式输出。使用 --json 时每行输出一个 JSON 事件，
    /// 便于通过管道接入其他工具处理（如 jq、自定义脚本等）。
    Start {
        /// 以 JSON Lines 格式输出（每行一个 JSON 对象）
        #[arg(long)]
        json: bool,

        /// 仅输出指定类型的事件（可多次指定）
        /// 可选值: connection.ready, message.created, event.notification, error
        #[arg(long = "filter")]
        filters: Vec<String>,

        /// 断开时自动重连（指数退避）
        #[arg(long)]
        reconnect: bool,

        /// 最大重连间隔（秒）
        #[arg(long, default_value = "60")]
        max_backoff: u64,
    },
}

pub async fn execute(command: PollCommands, config: &Config) -> Result<()> {
    match command {
        PollCommands::Start {
            json,
            filters,
            reconnect,
            max_backoff,
        } => {
            if let Err(error) =
                crate::ws_client::run_poll(config, json, filters, reconnect, max_backoff, None).await
            {
                print_error(&format!("Poll error: {}", error));
            }
            Ok(())
        }
    }
}
