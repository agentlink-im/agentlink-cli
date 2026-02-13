//! Friend Commands

use agentlink_sdk::{AgentLinkClient, ClientConfig};
use clap::Subcommand;
use colored::Colorize;

use crate::config::CliConfig;
use crate::output;

/// Friend commands
#[derive(Subcommand)]
pub enum FriendCommands {
    /// List all friends
    List,

    /// List pending friend requests
    Requests,

    /// Send a friend request
    Add {
        /// Target user ID
        #[arg(short, long)]
        user: String,

        /// Optional message
        #[arg(short, long)]
        message: Option<String>,
    },

    /// Accept a friend request
    Accept {
        /// Request ID
        #[arg(short, long)]
        request: String,
    },

    /// Reject a friend request
    Reject {
        /// Request ID
        #[arg(short, long)]
        request: String,
    },

    /// Remove a friend
    Remove {
        /// Friend user ID
        #[arg(short, long)]
        user: String,
    },
}

impl FriendCommands {
    pub async fn execute(&self, config: &CliConfig) -> anyhow::Result<()> {
        let api_key = config.require_api_key()?;

        let client_config = ClientConfig::default().with_token(api_key.to_string());
        let client = AgentLinkClient::new(client_config);

        match self {
            FriendCommands::List => {
                output::header("Friends");

                let response = client
                    .friends()
                    .get_friends()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to get friends: {}", e))?;

                if response.friends.is_empty() {
                    output::info("No friends yet");
                    return Ok(());
                }

                output::separator();
                for friendship in &response.friends {
                    if let Some(ref friend) = friendship.friend {
                        let name = friendship.remark.clone().unwrap_or_else(|| friend.nickname.clone());
                        println!("{} ({})", name.blue(), friend.id.dimmed());

                        if !friend.linkid.is_empty() {
                            println!("   LinkID: {}", friend.linkid);
                        }
                        output::separator();
                    }
                }

                output::info(&format!("Total: {} friends", response.total));
            }

            FriendCommands::Requests => {
                output::header("Friend Requests");

                let response = client
                    .friends()
                    .get_pending_requests()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to get friend requests: {}", e))?;

                if response.requests.is_empty() {
                    output::info("No pending requests");
                    return Ok(());
                }

                output::info(&format!("Pending: {} requests", response.pending));
                println!();

                for req in &response.requests {
                    let status_color = match req.status.as_str() {
                        "pending" => "pending".yellow(),
                        "accepted" => "accepted".green(),
                        "rejected" => "rejected".red(),
                        _ => req.status.normal(),
                    };

                    println!("{}: from {} | {}", req.id.dimmed(), req.from_user_id.blue(), status_color);
                    if let Some(ref msg) = req.message {
                        println!("   Message: {}", msg);
                    }
                    println!();
                }
            }

            FriendCommands::Add { user, message } => {
                output::info(&format!("Sending friend request to {}...", user));

                let _request = client
                    .friends()
                    .send_friend_request(user, message.as_deref())
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to send friend request: {}", e))?;

                output::success("Friend request sent!");
            }

            FriendCommands::Accept { request } => {
                output::info(&format!("Accepting friend request {}...", request));

                client
                    .friends()
                    .respond_friend_request(request, true, None)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to accept friend request: {}", e))?;

                output::success("Friend request accepted!");
            }

            FriendCommands::Reject { request } => {
                client
                    .friends()
                    .respond_friend_request(request, false, None)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to reject friend request: {}", e))?;

                output::success("Friend request rejected");
            }

            FriendCommands::Remove { user } => {
                client
                    .friends()
                    .delete_friend(user)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to remove friend: {}", e))?;

                output::success("Friend removed");
            }
        }

        Ok(())
    }
}
