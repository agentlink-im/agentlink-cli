use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// CLI 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 服务器地址
    #[serde(default = "default_server_url")]
    pub server_url: String,

    /// WebSocket 地址
    #[serde(default = "default_websocket_url")]
    pub websocket_url: String,

    /// API Key
    pub api_key: Option<String>,

    /// 默认输出格式
    #[serde(default)]
    pub defaults: Defaults,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Defaults {
    /// 默认输出格式
    #[serde(default = "default_output_format")]
    pub output_format: String,

    /// 默认分页大小
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

/// API 路径前缀 - 强制规范，不可配置
pub const API_PREFIX: &str = "/api/v1";
/// WebSocket 路径 - 强制规范，不可配置
pub const WS_PATH: &str = "/ws";

fn default_server_url() -> String {
    "https://api.agentlink.example.com".to_string()
}

fn default_websocket_url() -> String {
    "wss://ws.agentlink.example.com".to_string()
}

fn default_output_format() -> String {
    "table".to_string()
}

fn default_page_size() -> u32 {
    20
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_url: default_server_url(),
            websocket_url: default_websocket_url(),
            api_key: None,
            defaults: Defaults::default(),
        }
    }
}

impl Config {
    /// 加载配置
    pub fn load(path: Option<&str>) -> Result<Self> {
        let config_path = if let Some(p) = path {
            PathBuf::from(p)
        } else {
            Self::default_config_path()?
        };

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config file: {:?}", config_path))?;

            let config: Config = toml::from_str(&content)
                .with_context(|| format!("Failed to parse config file: {:?}", config_path))?;

            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    /// 保存配置
    pub fn save(&self) -> Result<()> {
        let config_path = Self::default_config_path()?;

        // 确保配置目录存在
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;

        std::fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config file: {:?}", config_path))?;

        Ok(())
    }

    /// 获取默认配置文件路径
    pub fn default_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().context("Failed to get config directory")?;

        Ok(config_dir.join("agentlink").join("config.toml"))
    }

    /// 获取配置目录
    pub fn config_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().context("Failed to get config directory")?;

        Ok(config_dir.join("agentlink"))
    }

    /// 检查是否已登录
    pub fn is_authenticated(&self) -> bool {
        self.api_key.is_some()
    }

    /// 获取 API Key
    pub fn get_api_key(&self) -> Result<&str> {
        self.api_key
            .as_deref()
            .context("Not authenticated. Please run 'agentlink auth login' first.")
    }

    /// 设置 API Key
    pub fn set_api_key(&mut self, api_key: String) {
        self.api_key = Some(api_key);
    }

    /// 清除认证信息
    pub fn clear_auth(&mut self) {
        self.api_key = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.server_url, "https://api.agentlink.example.com");
        assert_eq!(config.websocket_url, "wss://ws.agentlink.example.com");
        assert!(config.api_key.is_none());
    }

    #[test]
    fn test_config_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let mut config = Config::default();
        config.server_url = "https://test.example.com".to_string();
        config.api_key = Some("test_api_key".to_string());

        // 保存到临时文件
        let content = toml::to_string_pretty(&config).unwrap();
        std::fs::write(&config_path, content).unwrap();

        // 加载配置
        let loaded = Config::load(Some(config_path.to_str().unwrap())).unwrap();
        assert_eq!(loaded.server_url, "https://test.example.com");
        assert_eq!(loaded.api_key, Some("test_api_key".to_string()));
    }
}
