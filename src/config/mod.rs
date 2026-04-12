use anyhow::{anyhow, Context, Result};
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

    /// 持久化的 Agent API Key（sk_*）
    /// 兼容历史字段名 `user_token`
    #[serde(default, alias = "user_token")]
    pub api_key: Option<String>,

    /// 默认输出格式
    #[serde(default)]
    pub defaults: Defaults,

    /// 运行时覆盖的 Agent API Key（不落盘）
    #[serde(skip)]
    pub runtime_api_key: Option<String>,

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
            api_key: None,
            defaults: Defaults::default(),
            runtime_api_key: None,
            config_path: None,
        }
    }
}

impl Config {
    /// 加载配置（仅从配置文件读取）
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

    /// 将持久化配置重置为默认值，但保留当前配置文件路径
    pub fn reset_to_defaults(&mut self) {
        let config_path = self.config_path.clone();
        let runtime_api_key = self.runtime_api_key.clone();
        *self = Self::default();
        self.config_path = config_path;
        self.runtime_api_key = runtime_api_key;
    }

    /// 检查是否存在可用的 Agent API Key
    pub fn has_api_key(&self) -> bool {
        self.require_api_key().is_ok()
    }

    /// 返回生效的 Agent API Key；若配置格式非法则返回错误
    pub fn require_api_key(&self) -> Result<&str> {
        let api_key = self
            .runtime_api_key
            .as_deref()
            .or(self.api_key.as_deref())
            .ok_or_else(|| missing_api_key_error())?;
        validate_api_key_value(api_key)?;
        Ok(api_key)
    }

    /// 设置 Agent API Key
    pub fn set_api_key(&mut self, api_key: String) -> Result<()> {
        let api_key = api_key.trim().to_string();
        validate_api_key_value(&api_key)?;
        self.api_key = Some(api_key);
        Ok(())
    }

    /// 清除 Agent API Key
    pub fn clear_api_key(&mut self) {
        self.api_key = None;
    }

    /// 设置运行时 Agent API Key（不写入配置文件）
    pub fn set_runtime_api_key(&mut self, api_key: Option<String>) -> Result<()> {
        let api_key = api_key
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        if let Some(value) = api_key.as_deref() {
            validate_api_key_value(value)?;
        }

        self.runtime_api_key = api_key;
        Ok(())
    }

    /// 返回持久化配置中的 API Key 预览
    pub fn saved_api_key_preview(&self) -> Option<String> {
        self.api_key.as_deref().map(mask_api_key)
    }

    /// 返回运行时覆盖的 API Key 预览
    pub fn runtime_api_key_preview(&self) -> Option<String> {
        self.runtime_api_key.as_deref().map(mask_api_key)
    }
}

pub fn validate_api_key_value(api_key: &str) -> Result<()> {
    let api_key = api_key.trim();

    if api_key.is_empty() {
        return Err(missing_api_key_error());
    }

    if !api_key.starts_with("sk_") {
        return Err(anyhow!(
            "AgentLink CLI only supports agent API keys (`sk_*`). Run `agentlink api-key set <sk_...>` or pass `--api-key`."
        ));
    }

    Ok(())
}

pub fn mask_api_key(api_key: &str) -> String {
    let visible_len = api_key.len().min(8);
    format!("{}****", &api_key[..visible_len])
}

fn missing_api_key_error() -> anyhow::Error {
    anyhow!(
        "No agent API key configured. Run `agentlink api-key set <sk_...>` or pass `--api-key`."
    )
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
        assert!(config.api_key.is_none());
        assert!(config.runtime_api_key.is_none());
    }

    #[test]
    fn test_config_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let mut config = Config::default();
        config.server_url = "https://test.example.com".to_string();
        config.api_key = Some("sk_test_token".to_string());

        let content = toml::to_string_pretty(&config).unwrap();
        std::fs::write(&config_path, content).unwrap();

        let loaded = Config::load(Some(config_path.to_str().unwrap())).unwrap();
        assert_eq!(loaded.server_url, "https://test.example.com");
        assert_eq!(loaded.api_key, Some("sk_test_token".to_string()));
    }

    #[test]
    fn test_config_load_legacy_user_token_field() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        std::fs::write(
            &config_path,
            r#"
server_url = "https://legacy.example.com"
user_token = "sk_legacy_token"
"#,
        )
        .unwrap();

        let loaded = Config::load(Some(config_path.to_str().unwrap())).unwrap();
        assert_eq!(loaded.server_url, "https://legacy.example.com");
        assert_eq!(loaded.api_key, Some("sk_legacy_token".to_string()));
    }

    #[test]
    fn test_require_api_key_rejects_non_agent_tokens() {
        let mut config = Config::default();
        config.api_key = Some("jwt_legacy_token".to_string());

        let error = config.require_api_key().unwrap_err();
        assert!(error
            .to_string()
            .contains("only supports agent API keys (`sk_*`)"));
    }
}
