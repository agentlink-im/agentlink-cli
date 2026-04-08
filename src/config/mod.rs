use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// CLI 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 基础 API 地址
    #[serde(default = "default_server_url")]
    pub server_url: String,

    /// WebSocket 地址
    #[serde(default = "default_websocket_url")]
    pub websocket_url: String,

    /// 持久化的人类用户 token（jwt_*）
    /// 兼容历史字段名 `api_key`
    #[serde(default, alias = "api_key")]
    pub user_token: Option<String>,

    /// 默认输出格式
    #[serde(default)]
    pub defaults: Defaults,

    /// 运行时 Agent API Key（不落盘）
    #[serde(skip)]
    pub runtime_agent_api_key: Option<String>,

    /// 当前配置文件路径（不落盘）
    #[serde(skip)]
    config_path: Option<PathBuf>,
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

fn default_server_url() -> String {
    "https://beta-api.agentlink.chat/".to_string()
}

fn default_websocket_url() -> String {
    "wss://beta-api.agentlink.chat/".to_string()
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
            user_token: None,
            defaults: Defaults::default(),
            runtime_agent_api_key: None,
            config_path: None,
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

        let mut config = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config file: {:?}", config_path))?;

            toml::from_str(&content)
                .with_context(|| format!("Failed to parse config file: {:?}", config_path))?
        } else {
            Config::default()
        };

        config.config_path = Some(config_path);
        Ok(config)
    }

    /// 保存配置
    pub fn save(&self) -> Result<()> {
        let config_path = if let Some(path) = &self.config_path {
            path.clone()
        } else {
            Self::default_config_path()?
        };

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

    /// 获取当前生效配置文件路径
    pub fn current_config_path(&self) -> Result<PathBuf> {
        if let Some(path) = &self.config_path {
            Ok(path.clone())
        } else {
            Self::default_config_path()
        }
    }

    /// 将持久化配置重置为默认值，但保留当前配置文件路径和运行时 Agent API Key
    pub fn reset_to_defaults(&mut self) {
        let config_path = self.config_path.clone();
        let runtime_agent_api_key = self.runtime_agent_api_key.clone();
        *self = Self::default();
        self.config_path = config_path;
        self.runtime_agent_api_key = runtime_agent_api_key;
    }

    /// 检查是否有任意可用认证（用户 token 或运行时 agent key）
    pub fn is_authenticated(&self) -> bool {
        self.user_token.is_some() || self.runtime_agent_api_key.is_some()
    }

    /// 检查是否有用户登录态
    pub fn has_user_token(&self) -> bool {
        self.user_token.is_some()
    }

    /// 设置运行时 Agent API Key（不会写入配置文件）
    pub fn set_runtime_agent_api_key(&mut self, api_key: Option<String>) {
        self.runtime_agent_api_key = api_key;
    }

    /// 设置人类用户 token
    pub fn set_user_token(&mut self, token: String) {
        self.user_token = Some(token);
    }

    /// 清除人类用户 token
    pub fn clear_user_token(&mut self) {
        self.user_token = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.server_url, "https://beta-api.agentlink.chat/");
        assert_eq!(config.websocket_url, "wss://beta-api.agentlink.chat/");
        assert!(config.user_token.is_none());
        assert!(config.runtime_agent_api_key.is_none());
    }

    #[test]
    fn test_config_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let mut config = Config::default();
        config.server_url = "https://test.example.com".to_string();
        config.user_token = Some("jwt_test_token".to_string());
        config.runtime_agent_api_key = Some("sk_runtime_only".to_string());

        let content = toml::to_string_pretty(&config).unwrap();
        std::fs::write(&config_path, content).unwrap();

        let loaded = Config::load(Some(config_path.to_str().unwrap())).unwrap();
        assert_eq!(loaded.server_url, "https://test.example.com");
        assert_eq!(loaded.user_token, Some("jwt_test_token".to_string()));
        assert!(loaded.runtime_agent_api_key.is_none());
    }

    #[test]
    fn test_config_load_legacy_api_key_field() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        std::fs::write(
            &config_path,
            r#"
server_url = "https://legacy.example.com"
api_key = "jwt_legacy_token"
"#,
        )
        .unwrap();

        let loaded = Config::load(Some(config_path.to_str().unwrap())).unwrap();
        assert_eq!(loaded.server_url, "https://legacy.example.com");
        assert_eq!(loaded.user_token, Some("jwt_legacy_token".to_string()));
    }
}
