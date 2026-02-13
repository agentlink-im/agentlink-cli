//! User Commands

use agentlink_sdk::{AgentLinkClient, ClientConfig};
use clap::Subcommand;

use crate::config::CliConfig;
use crate::output;

/// User commands
#[derive(Subcommand)]
pub enum UserCommands {
    /// Get current user profile
    Me,

    /// Set user's LinkID
    SetLinkId {
        /// New LinkID
        #[arg(short, long)]
        linkid: String,
    },

    /// Check if a LinkID is available
    CheckLinkId {
        /// LinkID to check
        #[arg(short, long)]
        linkid: String,
    },
}

impl UserCommands {
    pub async fn execute(&self, config: &CliConfig) -> anyhow::Result<()> {
        let api_key = config.require_api_key()?;

        let client_config = ClientConfig::default().with_token(api_key.to_string());
        let client = AgentLinkClient::new(client_config);

        match self {
            UserCommands::Me => {
                output::header("User Profile");

                let user = client
                    .users()
                    .get_me()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to get user: {}", e))?;

                output::field("ID", &user.id);
                output::field("Nickname", &user.nickname);
                if !user.linkid.is_empty() {
                    output::field("LinkID", &user.linkid);
                }
                if let Some(ref avatar) = user.avatar {
                    output::field("Avatar", avatar);
                }
                output::field("Created", &output::format_time(&user.created_at));
            }

            UserCommands::SetLinkId { linkid } => {
                output::info(&format!("Setting LinkID to '{}'...", linkid));

                client
                    .auth()
                    .set_linkid(linkid)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to set LinkID: {}", e))?;

                output::success("LinkID updated successfully!");
            }

            UserCommands::CheckLinkId { linkid } => {
                let response = client
                    .auth()
                    .check_linkid(linkid)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to check LinkID: {}", e))?;

                if response.available {
                    output::success(&format!("LinkID '{}' is available", linkid));
                } else {
                    output::warn(&format!("LinkID '{}' is not available", linkid));
                }
            }
        }

        Ok(())
    }
}
