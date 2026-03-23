use anyhow::{Context, Result};
use reqwest::{Client, Method, RequestBuilder, Response};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

use crate::config::Config;
use crate::models::ApiResponse;

/// API 客户端
#[derive(Debug, Clone)]
pub struct ApiClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
}

impl ApiClient {
    /// 创建新的 API 客户端
    pub fn new(config: &Config) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            base_url: config.server_url.clone(),
            api_key: config.api_key.clone(),
        })
    }

    /// 设置 API Key
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// 构建请求
    fn build_request(&self, method: Method, path: &str) -> RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client.request(method, &url);

        // 添加认证头
        // 支持两种 token 类型：
        // - sk_ 前缀：API Key，走 API Key 鉴权逻辑
        // - jwt_ 前缀：JWT Token，走 JWT 鉴权逻辑
        if let Some(ref token) = self.api_key {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        // 添加默认头
        request = request
            .header("Accept", "application/json")
            .header("User-Agent", format!("agentlink-cli/{}", env!("CARGO_PKG_VERSION")));

        request
    }

    /// 发送 GET 请求
    pub async fn get<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let request = self.build_request(Method::GET, path);
        self.send_request(request).await
    }

    /// 发送 POST 请求
    pub async fn post<T, B>(&self, path: &str, body: Option<B>) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let mut request = self.build_request(Method::POST, path);

        if let Some(body) = body {
            request = request.json(&body);
        }

        self.send_request(request).await
    }

    /// 发送 PUT 请求
    pub async fn put<T, B>(&self, path: &str, body: Option<B>) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let mut request = self.build_request(Method::PUT, path);

        if let Some(body) = body {
            request = request.json(&body);
        }

        self.send_request(request).await
    }

    /// 发送 DELETE 请求
    pub async fn delete<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let request = self.build_request(Method::DELETE, path);
        self.send_request(request).await
    }

    /// 发送请求并处理响应
    async fn send_request<T>(&self, request: RequestBuilder) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let response = request
            .send()
            .await
            .context("Failed to send request")?;

        self.handle_response(response).await
    }

    /// 处理响应
    async fn handle_response<T>(&self, response: Response) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let status = response.status();

        if status.is_success() {
            let api_response: ApiResponse<T> = response
                .json()
                .await
                .context("Failed to parse response")?;

            if api_response.success {
                api_response
                    .data
                    .context("Response data is empty")
            } else {
                let error = api_response
                    .error
                    .map(|e| e.message)
                    .unwrap_or_else(|| "Unknown error".to_string());
                anyhow::bail!("API error: {}", error)
            }
        } else {
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            match status.as_u16() {
                401 => anyhow::bail!("Authentication failed. Please check your API key."),
                403 => anyhow::bail!("Permission denied."),
                404 => anyhow::bail!("Resource not found."),
                422 => anyhow::bail!("Validation error: {}", text),
                429 => anyhow::bail!("Rate limit exceeded. Please try again later."),
                _ => anyhow::bail!("HTTP error {}: {}", status, text),
            }
        }
    }

    // ==================== 认证 API ====================

    /// 验证 API Key / JWT Token
    pub async fn verify_api_key(&self) -> Result<crate::models::User> {
        self.get("/api/v1/users/me").await
    }

    /// 发送验证码
    pub async fn send_verification_code(&self, email: &str) -> Result<serde_json::Value> {
        let body = serde_json::json!({
            "email": email,
        });
        self.post("/api/v1/auth/send-code", Some(body)).await
    }

    /// 一键登录/注册
    pub async fn magic_login(&self, email: &str, code: &str) -> Result<crate::models::AuthResponse> {
        let body = serde_json::json!({
            "email": email,
            "code": code,
        });
        self.post("/api/v1/auth/magic-login", Some(body)).await
    }

    /// 完成 Onboarding
    pub async fn complete_onboarding(&self, display_name: &str) -> Result<crate::models::User> {
        let body = serde_json::json!({
            "display_name": display_name,
        });
        self.post("/api/v1/auth/complete-onboarding", Some(body)).await
    }

    // ==================== 用户 API ====================

    /// 获取用户列表
    pub async fn list_users(
        &self,
        page: Option<i64>,
        per_page: Option<i64>,
    ) -> Result<crate::models::PaginatedResponse<crate::models::User>> {
        let mut path = "/api/v1/users".to_string();
        let mut params = vec![];

        if let Some(p) = page {
            params.push(format!("page={}", p));
        }
        if let Some(pp) = per_page {
            params.push(format!("per_page={}", pp));
        }

        if !params.is_empty() {
            path.push('?');
            path.push_str(&params.join("&"));
        }

        self.get(&path).await
    }

    /// 获取用户详情
    pub async fn get_user(&self, user_id: &str) -> Result<crate::models::User> {
        self.get(&format!("/api/v1/users/{}", user_id)).await
    }

    // ==================== 任务 API ====================

    /// 获取任务列表
    pub async fn list_tasks(
        &self,
        page: Option<i64>,
        per_page: Option<i64>,
    ) -> Result<crate::models::PaginatedResponse<crate::models::Task>> {
        let mut path = "/api/v1/tasks".to_string();
        let mut params = vec![];

        if let Some(p) = page {
            params.push(format!("page={}", p));
        }
        if let Some(pp) = per_page {
            params.push(format!("per_page={}", pp));
        }

        if !params.is_empty() {
            path.push('?');
            path.push_str(&params.join("&"));
        }

        self.get(&path).await
    }

    /// 获取任务详情
    pub async fn get_task(&self, task_id: &str) -> Result<crate::models::Task> {
        self.get(&format!("/api/v1/tasks/{}", task_id)).await
    }

    /// 申请任务
    pub async fn apply_to_task<B>(
        &self,
        task_id: &str,
        body: B,
    ) -> Result<crate::models::Application>
    where
        B: Serialize,
    {
        self.post(&format!("/api/v1/tasks/{}/apply", task_id), Some(body))
            .await
    }

    // ==================== 消息 API ====================

    /// 获取会话列表
    pub async fn list_conversations(
        &self,
    ) -> Result<Vec<crate::models::Conversation>> {
        self.get("/api/v1/conversations").await
    }

    /// 获取消息列表
    pub async fn get_messages(
        &self,
        conversation_id: &str,
    ) -> Result<Vec<crate::models::Message>> {
        self.get(&format!("/api/v1/conversations/{}/messages", conversation_id))
            .await
    }

    /// 发送消息
    pub async fn send_message<B>(
        &self,
        conversation_id: &str,
        body: B,
    ) -> Result<crate::models::Message>
    where
        B: Serialize,
    {
        self.post(&format!("/api/v1/conversations/{}/messages", conversation_id), Some(body))
            .await
    }

    // ==================== 通知 API ====================

    /// 获取通知列表
    pub async fn list_notifications(
        &self,
        unread_only: bool,
    ) -> Result<Vec<crate::models::Notification>> {
        let path = if unread_only {
            "/api/v1/notifications?unread_only=true"
        } else {
            "/api/v1/notifications"
        };
        self.get(path).await
    }

    /// 标记通知已读
    pub async fn mark_notification_read(&self, notification_id: &str) -> Result<()> {
        self.post::<serde_json::Value, _>(
            &format!("/api/v1/notifications/{}/read", notification_id),
            None::<serde_json::Value>,
        )
        .await?;
        Ok(())
    }

    // ==================== 人脉 API ====================

    /// 获取人脉列表
    pub async fn list_connections(&self) -> Result<Vec<crate::models::Connection>> {
        self.get("/api/v1/network/connections").await
    }

    /// 获取待处理请求
    pub async fn list_pending_requests(
        &self,
    ) -> Result<Vec<crate::models::ConnectionRequest>> {
        self.get("/api/v1/network/requests").await
    }

    /// 发送人脉请求
    pub async fn send_connection_request<B>(
        &self,
        body: B,
    ) -> Result<serde_json::Value>
    where
        B: Serialize,
    {
        self.post("/api/v1/network/requests", Some(body)).await
    }

    /// 响应人脉请求
    pub async fn respond_to_request<B>(
        &self,
        request_id: &str,
        body: B,
    ) -> Result<serde_json::Value>
    where
        B: Serialize,
    {
        self.post(&format!("/api/v1/network/requests/{}/respond", request_id), Some(body))
            .await
    }

    // ==================== Agent API ====================

    /// 获取 Agent 统计
    pub async fn get_agent_stats(&self) -> Result<serde_json::Value> {
        self.get("/api/v1/agents/me/stats").await
    }

    /// 更新 Agent 状态
    pub async fn update_agent_status<B>(&self, body: B) -> Result<serde_json::Value>
    where
        B: Serialize,
    {
        self.put("/api/v1/agents/me/status", Some(body)).await
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
