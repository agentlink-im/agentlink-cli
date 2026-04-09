use anyhow::{Context, Result};
use clap::Subcommand;
use colored::Colorize;
use serde::Deserialize;
use std::process::Command;

const GITHUB_REPO: &str = "agentlink-im/agentlink-cli";
const GITHUB_API_URL: &str =
    "https://api.github.com/repos/agentlink-im/agentlink-cli/releases/latest";

#[derive(Subcommand)]
pub enum UpdateCommands {
    /// 检查是否有新版本
    Check,

    /// 更新到最新版本
    #[command(name = "update")]
    Update {
        /// 强制更新，即使已经是最新版本
        #[arg(short, long)]
        force: bool,

        /// 指定版本号（例如: v0.2.0）
        #[arg(short, long)]
        version: Option<String>,
    },
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct GithubRelease {
    tag_name: String,
    name: String,
    body: String,
    published_at: String,
    html_url: String,
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

/// 获取当前版本
fn current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// 获取最新版本信息
async fn get_latest_release() -> Result<GithubRelease> {
    let client = reqwest::Client::builder()
        .user_agent("agentlink-cli-update")
        .build()?;

    let release: GithubRelease = client.get(GITHUB_API_URL).send().await?.json().await?;

    Ok(release)
}

/// 比较版本号
/// 返回值: -1 表示 current < latest, 0 表示相等, 1 表示 current > latest
fn compare_versions(current: &str, latest: &str) -> i32 {
    let current = current.trim_start_matches('v');
    let latest = latest.trim_start_matches('v');

    let current_parts: Vec<u32> = current.split('.').filter_map(|s| s.parse().ok()).collect();

    let latest_parts: Vec<u32> = latest.split('.').filter_map(|s| s.parse().ok()).collect();

    for i in 0..std::cmp::max(current_parts.len(), latest_parts.len()) {
        let current_part = current_parts.get(i).copied().unwrap_or(0);
        let latest_part = latest_parts.get(i).copied().unwrap_or(0);

        if current_part < latest_part {
            return -1;
        } else if current_part > latest_part {
            return 1;
        }
    }

    0
}

/// 确定当前平台的资产名称
fn get_platform_asset_name() -> Option<String> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let asset_name = match (os, arch) {
        ("linux", "x86_64") => "agentlink-linux-x86_64",
        ("linux", "aarch64") => "agentlink-linux-aarch64",
        ("macos", "x86_64") => "agentlink-macos-x86_64",
        ("macos", "aarch64") => "agentlink-macos-aarch64",
        ("windows", "x86_64") => "agentlink-windows-x86_64.exe",
        _ => return None,
    };

    Some(asset_name.to_string())
}

/// 检查更新
async fn check_update() -> Result<()> {
    let current = current_version();
    println!("{} {}", "当前版本:".bold(), current.cyan());

    let release = get_latest_release().await?;
    let latest = release.tag_name.trim_start_matches('v');

    println!("{} {}", "最新版本:".bold(), latest.green());
    println!("{} {}", "发布日期:".bold(), release.published_at);
    println!(
        "{} {}",
        "发布页面:".bold(),
        release.html_url.blue().underline()
    );

    match compare_versions(&current, latest) {
        -1 => {
            println!("\n{}", "✓ 发现新版本!".green().bold());
            println!("\n{}", "更新内容:".bold());
            println!("{}", release.body);
            println!("\n{}", "运行以下命令更新:".yellow());
            println!("  agentlink self-update update");
        }
        0 => {
            println!("\n{}", "✓ 已经是最新版本".green().bold());
        }
        1 => {
            println!("\n{}", "⚠ 当前版本比最新版本还新".yellow().bold());
            println!("   你可能在使用开发版本");
        }
        _ => {}
    }

    Ok(())
}

/// 执行更新
async fn perform_update(force: bool, specific_version: Option<String>) -> Result<()> {
    let current = current_version();

    // 获取目标版本
    let (target_version, download_url) = if let Some(version) = specific_version {
        let version = if version.starts_with('v') {
            version
        } else {
            format!("v{}", version)
        };
        let url = format!(
            "https://github.com/{}/releases/download/{}/{}",
            GITHUB_REPO,
            version,
            get_platform_asset_name().context("不支持当前平台")?
        );
        (version, url)
    } else {
        let release = get_latest_release().await?;
        let latest = release.tag_name.clone();

        if !force && compare_versions(&current, &latest) >= 0 {
            println!("{}", "✓ 已经是最新版本，使用 --force 强制更新".green());
            return Ok(());
        }

        let asset_name = get_platform_asset_name().context("不支持当前平台")?;
        let asset = release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .context(format!("未找到适用于当前平台的二进制文件: {}", asset_name))?;

        (latest, asset.browser_download_url.clone())
    };

    println!(
        "{} {} -> {}",
        "正在更新:".bold(),
        current.cyan(),
        target_version.green()
    );
    println!("{} {}", "下载地址:".bold(), download_url.blue().underline());

    // 使用安装脚本进行更新
    let status = if cfg!(target_os = "windows") {
        Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "Invoke-WebRequest -Uri {} -UseBasicParsing | Invoke-Expression",
                    "https://raw.githubusercontent.com/agentlink-im/agentlink-cli/main/install.ps1"
                ),
            ])
            .status()?
    } else {
        Command::new("sh")
            .arg("-c")
            .arg("curl -sSL https://raw.githubusercontent.com/agentlink-im/agentlink-cli/main/install.sh | sh")
            .status()?
    };

    if status.success() {
        println!("\n{}", "✓ 更新成功!".green().bold());
        println!(
            "{} 请重新启动终端或运行: source ~/.bashrc (或 ~/.zshrc)",
            "提示:".yellow()
        );
    } else {
        anyhow::bail!(
            "更新失败，请手动安装: https://github.com/agentlink-im/agentlink-cli/releases"
        );
    }

    Ok(())
}

pub async fn execute(command: UpdateCommands) -> Result<()> {
    match command {
        UpdateCommands::Check => check_update().await,
        UpdateCommands::Update { force, version } => perform_update(force, version).await,
    }
}
