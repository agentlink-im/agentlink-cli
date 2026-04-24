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

        /// 收到事件时执行的外部命令（事件 JSON 通过 stdin 传入）
        ///
        /// 示例: --exec "python3 ./handler.py"
        /// 示例: --exec "./notify.sh"
        #[arg(long)]
        exec: Option<String>,

        /// Webhook 转发地址（覆盖配置文件中的设置）
        ///
        /// CLI 会将收到的每个事件实时 HTTP POST 到该地址。
        /// 也可通过 `agentlink webhook set <url>` 持久化配置。
        #[arg(long)]
        webhook_url: Option<String>,
    },
}

pub async fn execute(command: PollCommands, config: &Config) -> Result<()> {
    match command {
        PollCommands::Start {
            json,
            filters,
            reconnect,
            max_backoff,
            exec,
            webhook_url,
        } => {
            // 命令行参数优先，其次配置文件
            let resolved_webhook = webhook_url.as_ref()
                .or(config.webhook_url.as_ref())
                .cloned();

            if let Err(error) = crate::ws_client::run_poll(
                config, json, filters, reconnect, max_backoff, exec, resolved_webhook, None,
            ).await
            {
                print_error(&format!("Poll error: {}", error));
            }
            Ok(())
        }
    }
}
