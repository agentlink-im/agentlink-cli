use agentlink_protocol::ApiResponse;
use anyhow::{Context, Result};
use reqwest::{Client, Method, RequestBuilder, Response};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

use crate::config::Config;

mod generated;

#[derive(Debug, Clone)]
pub struct ApiClient {
    client: Client,
    base_url: String,
    auth_token: Option<String>,
}

impl ApiClient {
    pub fn new(config: &Config) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            base_url: config.server_url.trim_end_matches('/').to_string(),
            auth_token: config
                .runtime_agent_api_key
                .clone()
                .or_else(|| config.user_token.clone()),
        })
    }

    pub fn with_bearer_token(mut self, token: String) -> Self {
        self.auth_token = Some(token);
        self
    }

    fn build_request(&self, method: Method, path: &str) -> RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client.request(method, &url);

        if let Some(token) = &self.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        request.header("Accept", "application/json").header(
            "User-Agent",
            format!("agentlink-cli/{}", env!("CARGO_PKG_VERSION")),
        )
    }

    pub async fn get<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.send_json(self.build_request(Method::GET, path)).await
    }

    pub async fn get_with_query<T, Q>(&self, path: &str, query: &Q) -> Result<T>
    where
        T: DeserializeOwned,
        Q: Serialize,
    {
        self.send_json(self.build_request(Method::GET, path).query(query))
            .await
    }

    pub async fn post<T, B>(&self, path: &str, body: Option<B>) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let mut request = self.build_request(Method::POST, path);
        if let Some(body) = body {
            request = request.json(&body);
        }

        self.send_json(request).await
    }

    pub async fn put<T, B>(&self, path: &str, body: Option<B>) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let mut request = self.build_request(Method::PUT, path);
        if let Some(body) = body {
            request = request.json(&body);
        }

        self.send_json(request).await
    }

    pub async fn post_no_data<B>(&self, path: &str, body: Option<B>) -> Result<()>
    where
        B: Serialize,
    {
        let mut request = self.build_request(Method::POST, path);
        if let Some(body) = body {
            request = request.json(&body);
        }

        self.send_without_data(request).await
    }

    async fn send_json<T>(&self, request: RequestBuilder) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let response = request.send().await.context("Failed to send request")?;

        self.handle_json_response(response).await
    }

    async fn send_without_data(&self, request: RequestBuilder) -> Result<()> {
        let response = request.send().await.context("Failed to send request")?;

        self.handle_empty_success(response).await
    }

    async fn handle_json_response<T>(&self, response: Response) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        if !status.is_success() {
            return Self::http_error(status.as_u16(), &body);
        }

        let api_response: ApiResponse<T> =
            serde_json::from_str(&body).context("Failed to parse API response")?;

        if api_response.success {
            api_response.data.context("Response data is empty")
        } else {
            let message = api_response
                .error
                .map(|error| error.message)
                .or(api_response.message)
                .unwrap_or_else(|| "Unknown API error".to_string());
            anyhow::bail!("API error: {}", message);
        }
    }

    async fn handle_empty_success(&self, response: Response) -> Result<()> {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        if !status.is_success() {
            return Self::http_error(status.as_u16(), &body);
        }

        if body.trim().is_empty() {
            return Ok(());
        }

        let api_response: ApiResponse<serde_json::Value> =
            serde_json::from_str(&body).context("Failed to parse API response")?;

        if api_response.success {
            Ok(())
        } else {
            let message = api_response
                .error
                .map(|error| error.message)
                .or(api_response.message)
                .unwrap_or_else(|| "Unknown API error".to_string());
            anyhow::bail!("API error: {}", message);
        }
    }

    fn http_error<T>(status: u16, body: &str) -> Result<T> {
        match status {
            401 => anyhow::bail!("Authentication failed. Please check your token."),
            403 => anyhow::bail!("Permission denied."),
            404 => anyhow::bail!("Resource not found."),
            422 => anyhow::bail!("Validation error: {}", body),
            429 => anyhow::bail!("Rate limit exceeded. Please try again later."),
            _ => anyhow::bail!("HTTP error {}: {}", status, body),
        }
    }

    pub async fn verify_token(&self) -> Result<agentlink_protocol::user::UserResponse> {
        self.get_current_user().await
    }

    pub async fn get_task(&self, task_id: &str) -> Result<agentlink_protocol::task::TaskResponse> {
        self.get_task_by_id(task_id).await
    }

    pub async fn list_connections(
        &self,
    ) -> Result<Vec<agentlink_protocol::network::ConnectionResponse>> {
        self.get_connections().await
    }

    pub async fn send_connection_request(
        &self,
        body: agentlink_protocol::network::SendConnectionRequest,
    ) -> Result<agentlink_protocol::network::ConnectionRequestResponse> {
        self.send_request(body).await
    }

    pub async fn list_pending_requests(
        &self,
    ) -> Result<Vec<agentlink_protocol::network::ConnectionRequestResponse>> {
        self.get_pending_requests().await
    }

    pub async fn get_network_stats(&self) -> Result<agentlink_protocol::network::NetworkStats> {
        self.get_stats(agentlink_protocol::network::NetworkStatsQuery { user_id: None })
            .await
    }

    pub async fn list_notifications(
        &self,
        unread_only: bool,
    ) -> Result<Vec<agentlink_protocol::message::NotificationResponse>> {
        self.get_notifications(agentlink_protocol::message::NotificationQuery {
            unread_only: Some(unread_only),
            page: None,
            per_page: None,
        })
        .await
    }

    pub async fn mark_notification_read(
        &self,
        notification_id: &str,
    ) -> Result<agentlink_protocol::message::NotificationReadResponse> {
        self.mark_notification_as_read(notification_id).await
    }

    pub async fn mark_all_notifications_read(
        &self,
    ) -> Result<agentlink_protocol::message::MarkAllNotificationsReadResponse> {
        self.mark_all_notifications_as_read().await
    }

    pub async fn list_my_owned_agents(
        &self,
    ) -> Result<Vec<agentlink_protocol::agent::AgentSummaryResponse>> {
        self.list_my_agents(agentlink_protocol::agent::ListAgentsQuery {
            limit: Some(100),
            page: Some(1),
        })
        .await
    }

    pub async fn resolve_agent_id(&self, explicit_agent_id: Option<&str>) -> Result<String> {
        if let Some(agent_id) = explicit_agent_id {
            return Ok(agent_id.to_string());
        }

        let agents = self.list_my_owned_agents().await?;
        match agents.as_slice() {
            [] => anyhow::bail!("No owned agents found. Create one first."),
            [single] => Ok(single.id.to_string()),
            _ => {
                let options = agents
                    .iter()
                    .map(|agent| format!("{} ({})", agent.id, agent.linkid))
                    .collect::<Vec<_>>()
                    .join(", ");
                anyhow::bail!(
                    "Multiple agents found. Please pass --agent-id. Available agents: {}",
                    options
                )
            }
        }
    }

    pub async fn get_agent_workspace(
        &self,
        explicit_agent_id: Option<&str>,
    ) -> Result<agentlink_protocol::agent::AgentWorkspaceResponse> {
        let agent_id = self.resolve_agent_id(explicit_agent_id).await?;
        self.get_workspace(&agent_id).await
    }

    pub async fn get_primary_agent_api_key(
        &self,
        explicit_agent_id: Option<&str>,
    ) -> Result<agentlink_protocol::agent_api_key::AgentApiKeySimpleResponse> {
        let agent_id = self.resolve_agent_id(explicit_agent_id).await?;
        self.get_api_key(&agent_id).await
    }

    pub async fn create_or_reset_primary_agent_api_key(
        &self,
        explicit_agent_id: Option<&str>,
        body: agentlink_protocol::agent_api_key::CreateAgentApiKeyRequest,
    ) -> Result<agentlink_protocol::agent_api_key::AgentApiKeyResponse> {
        let agent_id = self.resolve_agent_id(explicit_agent_id).await?;
        self.create_or_reset_api_key(&agent_id, body).await
    }

    pub async fn update_primary_agent_api_key(
        &self,
        explicit_agent_id: Option<&str>,
        body: agentlink_protocol::agent_api_key::UpdateAgentApiKeyRequest,
    ) -> Result<agentlink_protocol::agent_api_key::AgentApiKeySimpleConfigResponse> {
        let agent_id = self.resolve_agent_id(explicit_agent_id).await?;
        self.update_api_key(&agent_id, body).await
    }

    pub async fn revoke_primary_agent_api_key(
        &self,
        explicit_agent_id: Option<&str>,
    ) -> Result<agentlink_protocol::agent_api_key::AgentApiKeyMessageResponse> {
        let agent_id = self.resolve_agent_id(explicit_agent_id).await?;
        self.revoke_api_key(&agent_id).await
    }

    pub async fn get_primary_agent_api_key_stats(
        &self,
        explicit_agent_id: Option<&str>,
    ) -> Result<agentlink_protocol::agent_api_key::AgentApiKeyStatsResponse> {
        let agent_id = self.resolve_agent_id(explicit_agent_id).await?;
        self.get_api_key_stats(&agent_id).await
    }

    pub async fn create_agent_service(
        &self,
        agent_id: &str,
        body: agentlink_protocol::agent::CreateServiceRequest,
    ) -> Result<agentlink_protocol::agent::AgentServiceResponse> {
        self.create_service(agent_id, body).await
    }

    pub async fn update_agent_availability(
        &self,
        agent_id: &str,
        is_available: bool,
    ) -> Result<agentlink_protocol::agent::AgentProfileResponse> {
        self.update_agent(
            agent_id,
            agentlink_protocol::agent::UpdateAgentRequest {
                linkid: None,
                avatar_url: None,
                display_name: None,
                description: None,
                specialty: None,
                is_available: Some(is_available),
            },
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Matcher;
    use serde_json::json;

    fn build_config(
        server_url: String,
        user_token: Option<&str>,
        runtime_agent_api_key: Option<&str>,
    ) -> Config {
        let mut config = Config::default();
        config.server_url = server_url;
        config.user_token = user_token.map(ToString::to_string);
        config.runtime_agent_api_key = runtime_agent_api_key.map(ToString::to_string);
        config
    }

    #[test]
    fn test_api_client_creation() {
        let config = Config::default();
        let client = ApiClient::new(&config);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_runtime_sk_token_uses_bearer_header_only() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/echo")
            .match_header("authorization", "Bearer sk_runtime_token")
            .match_header("x-api-key", Matcher::Missing)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"success":true,"data":{"ok":true}}"#)
            .create_async()
            .await;

        let config = build_config(server.url(), None, Some("sk_runtime_token"));
        let client = ApiClient::new(&config).unwrap();

        let data: serde_json::Value = client.get("/echo").await.unwrap();
        assert_eq!(data.get("ok").and_then(|v| v.as_bool()), Some(true));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_jwt_token_uses_bearer_header_only() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/echo")
            .match_header("authorization", "Bearer jwt_user_token")
            .match_header("x-api-key", Matcher::Missing)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"success":true,"data":{"ok":true}}"#)
            .create_async()
            .await;

        let config = build_config(server.url(), Some("jwt_user_token"), None);
        let client = ApiClient::new(&config).unwrap();

        let data: serde_json::Value = client.get("/echo").await.unwrap();
        assert_eq!(data.get("ok").and_then(|v| v.as_bool()), Some(true));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_runtime_sk_token_takes_precedence_over_user_token() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/echo")
            .match_header("authorization", "Bearer sk_runtime_first")
            .match_header("x-api-key", Matcher::Missing)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"success":true,"data":{"ok":true}}"#)
            .create_async()
            .await;

        let config = build_config(
            server.url(),
            Some("jwt_should_not_be_used"),
            Some("sk_runtime_first"),
        );
        let client = ApiClient::new(&config).unwrap();

        let data: serde_json::Value = client.get("/echo").await.unwrap();
        assert_eq!(data.get("ok").and_then(|v| v.as_bool()), Some(true));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_agent_api_key_endpoints_use_singular_api_key_paths() {
        let mut server = mockito::Server::new_async().await;
        let agent_id = "00000000-0000-0000-0000-000000000111";
        let key_id = "00000000-0000-0000-0000-000000000222";
        let created_at = "2026-01-01T00:00:00Z";

        let show_path = format!("/api/v1/agents/{agent_id}/api-key");
        let revoke_path = format!("/api/v1/agents/{agent_id}/api-key/revoke");
        let stats_path = format!("/api/v1/agents/{agent_id}/api-key/stats");

        let show_mock = server
            .mock("GET", show_path.as_str())
            .match_header("authorization", "Bearer jwt_user_token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "success": true,
                    "data": {
                        "has_key": true,
                        "agent_id": agent_id,
                        "id": key_id,
                        "api_key_preview": "sk_test...1234",
                        "permissions": ["all"],
                        "is_active": true,
                        "rate_limit_per_minute": 100,
                        "last_used_at": null,
                        "expires_at": null,
                        "created_at": created_at
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let reset_mock = server
            .mock("POST", show_path.as_str())
            .match_header("authorization", "Bearer jwt_user_token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "success": true,
                    "data": {
                        "id": key_id,
                        "agent_id": agent_id,
                        "api_key": "sk_generated_token",
                        "name": "primary",
                        "permissions": ["all"],
                        "is_active": true,
                        "rate_limit_per_minute": 100,
                        "last_used_at": null,
                        "expires_at": null,
                        "created_at": created_at
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let update_mock = server
            .mock("PUT", show_path.as_str())
            .match_header("authorization", "Bearer jwt_user_token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "success": true,
                    "data": {
                        "id": key_id,
                        "agent_id": agent_id,
                        "permissions": ["tasks:read"],
                        "is_active": true,
                        "rate_limit_per_minute": 60,
                        "expires_at": null
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let revoke_mock = server
            .mock("POST", revoke_path.as_str())
            .match_header("authorization", "Bearer jwt_user_token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "success": true,
                    "data": {
                        "message": "revoked"
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let stats_mock = server
            .mock("GET", stats_path.as_str())
            .match_header("authorization", "Bearer jwt_user_token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "success": true,
                    "data": {
                        "key_id": key_id,
                        "agent_id": agent_id,
                        "has_stats": true,
                        "total_requests": 42,
                        "requests_24h": 12,
                        "requests_7d": 40,
                        "avg_response_time_ms": 120,
                        "last_used_at": created_at,
                        "message": null
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let config = build_config(server.url(), Some("jwt_user_token"), None);
        let client = ApiClient::new(&config).unwrap();

        let show = client.get_api_key(agent_id).await.unwrap();
        assert!(show.has_key);

        let reset = client
            .create_or_reset_api_key(
                agent_id,
                agentlink_protocol::agent_api_key::CreateAgentApiKeyRequest {
                    name: Some("primary".to_string()),
                    permissions: vec!["all".to_string()],
                    rate_limit_per_minute: 100,
                    expires_at: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(reset.api_key, "sk_generated_token");

        let update = client
            .update_api_key(
                agent_id,
                agentlink_protocol::agent_api_key::UpdateAgentApiKeyRequest {
                    name: None,
                    permissions: Some(vec!["tasks:read".to_string()]),
                    rate_limit_per_minute: Some(60),
                    is_active: Some(true),
                    expires_at: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(update.rate_limit_per_minute, 60);

        let revoke = client.revoke_api_key(agent_id).await.unwrap();
        assert_eq!(revoke.message, "revoked");

        let stats = client.get_api_key_stats(agent_id).await.unwrap();
        assert_eq!(stats.total_requests, Some(42));

        show_mock.assert_async().await;
        reset_mock.assert_async().await;
        update_mock.assert_async().await;
        revoke_mock.assert_async().await;
        stats_mock.assert_async().await;
    }
}
