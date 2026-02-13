//! CLI Configuration

use agentlink_sdk::{DEFAULT_API_URL, ENV_API_KEY, ENV_API_URL};
use std::env;

/// CLI configuration
#[allow(dead_code)]
pub struct CliConfig {
    pub api_key: Option<String>,
    pub api_url: String,
    pub format: OutputFormat,
    pub verbose: bool,
}

/// Output format
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Text,
    Json,
}

#[allow(dead_code)]
impl CliConfig {
    /// Create configuration from CLI arguments
    pub fn from_args(cli: &crate::Cli) -> Self {
        let api_key = cli.api_key.clone().or_else(|| env::var(ENV_API_KEY).ok());

        let api_url = cli
            .api_url
            .clone()
            .or_else(|| env::var(ENV_API_URL).ok())
            .unwrap_or_else(|| DEFAULT_API_URL.to_string());

        let format = match cli.format.as_str() {
            "json" => OutputFormat::Json,
            _ => OutputFormat::Text,
        };

        Self {
            api_key,
            api_url,
            format,
            verbose: cli.verbose,
        }
    }

    /// Check if API key is set
    pub fn has_api_key(&self) -> bool {
        self.api_key.is_some()
    }

    /// Get API key or error
    pub fn require_api_key(&self) -> anyhow::Result<&str> {
        self.api_key
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("API key is required. Set AGENTLINK_API_KEY or use --api-key"))
    }
}
