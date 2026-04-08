use anyhow::{Context, Result};
use reqwest::{Client, Method, RequestBuilder, Response};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

use crate::config::Config;
use crate::models::ApiResponse;

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
            auth_token: config.api_key.clone(),
        })
    }

    pub fn with_auth_token(mut self, auth_token: String) -> Self {
        self.auth_token = Some(auth_token);
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

    pub async fn post_stream<B>(&self, path: &str, body: Option<B>) -> Result<Response>
    where
        B: Serialize,
    {
        let mut request = self.build_request(Method::POST, path);
        if let Some(body) = body {
            request = request.json(&body);
        }

        self.send_stream(request).await
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

    pub async fn delete<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.send_json(self.build_request(Method::DELETE, path))
            .await
    }

    pub async fn delete_no_data(&self, path: &str) -> Result<()> {
        self.send_without_data(self.build_request(Method::DELETE, path))
            .await
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

    pub async fn put_no_data<B>(&self, path: &str, body: Option<B>) -> Result<()>
    where
        B: Serialize,
    {
        let mut request = self.build_request(Method::PUT, path);
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

    async fn send_stream(&self, request: RequestBuilder) -> Result<Response> {
        let response = request
            .header("Accept", "text/event-stream")
            .send()
            .await
            .context("Failed to send request")?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Self::http_error(status.as_u16(), &body);
        }

        Ok(response)
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

    pub async fn get_user(&self, user_id: &str) -> Result<agentlink_protocol::user::UserResponse> {
        self.get_user_by_id(user_id).await
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

    pub async fn get_agent_profile(
        &self,
        agent_id: &str,
    ) -> Result<agentlink_protocol::agent::AgentProfileResponse> {
        self.get_profile(agent_id).await
    }

    pub async fn get_current_agent_profile(
        &self,
    ) -> Result<agentlink_protocol::agent::AgentProfileResponse> {
        let user = self.get_current_user().await?;
        self.get_profile(&user.id.to_string()).await
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

    #[test]
    fn test_api_client_creation() {
        let config = Config::default();
        let client = ApiClient::new(&config);
        assert!(client.is_ok());
    }
}
