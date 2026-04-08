// This file is generated from protocol/src/http_surface.rs.
// Regenerate with: cargo run --manifest-path protocol/Cargo.toml --bin generate_http_artifacts
// Do not edit manually.
#![allow(dead_code)]
use super::ApiClient;
use anyhow::Result;

impl ApiClient {
    pub async fn send_verification_code(
        &self,
        body: agentlink_protocol::auth::SendVerificationCodeRequest,
    ) -> Result<agentlink_protocol::auth::SendVerificationCodeResponse> {
        let path = "/api/v1/auth/send-code".to_string();
        self.post(&path, Some(body)).await
    }
    pub async fn magic_login(
        &self,
        body: agentlink_protocol::auth::MagicLoginRequest,
    ) -> Result<agentlink_protocol::auth::MagicLoginResponse> {
        let path = "/api/v1/auth/magic-login".to_string();
        self.post(&path, Some(body)).await
    }
    pub async fn get_onboarding_status(
        &self,
    ) -> Result<agentlink_protocol::auth::OnboardingStatusResponse> {
        let path = "/api/v1/auth/onboarding-status".to_string();
        self.get(&path).await
    }
    pub async fn complete_onboarding(
        &self,
        body: agentlink_protocol::auth::UpdateLinkidRequest,
    ) -> Result<agentlink_protocol::auth::CompleteOnboardingResponse> {
        let path = "/api/v1/auth/complete-onboarding".to_string();
        self.post(&path, Some(body)).await
    }
    pub async fn get_current_user(&self) -> Result<agentlink_protocol::user::UserResponse> {
        let path = "/api/v1/users/me".to_string();
        self.get(&path).await
    }
    pub async fn list_tasks(
        &self,
        query: agentlink_protocol::task::TaskSearchQuery,
    ) -> Result<agentlink_protocol::PaginatedResponse<agentlink_protocol::task::TaskResponse>> {
        let path = "/api/v1/tasks".to_string();
        self.get_with_query(&path, &query).await
    }
    pub async fn get_task_by_id(&self, id: &str) -> Result<agentlink_protocol::task::TaskResponse> {
        let path = format!("/api/v1/tasks/{id}", id = id);
        self.get(&path).await
    }
    pub async fn apply_to_task(
        &self,
        id: &str,
        body: agentlink_protocol::task::CreateApplicationRequest,
    ) -> Result<agentlink_protocol::task::ApplicationResponse> {
        let path = format!("/api/v1/tasks/{id}/apply", id = id);
        self.post(&path, Some(body)).await
    }
    pub async fn get_my_tasks(&self) -> Result<agentlink_protocol::task::MyTasksResponse> {
        let path = "/api/v1/users/me/tasks".to_string();
        self.get(&path).await
    }
    pub async fn list_conversations(
        &self,
        query: agentlink_protocol::message::ConversationQuery,
    ) -> Result<Vec<agentlink_protocol::message::ConversationResponse>> {
        let path = "/api/v1/conversations".to_string();
        self.get_with_query(&path, &query).await
    }
    pub async fn create_conversation(
        &self,
        body: agentlink_protocol::message::CreateConversationRequest,
    ) -> Result<agentlink_protocol::message::ConversationResponse> {
        let path = "/api/v1/conversations".to_string();
        self.post(&path, Some(body)).await
    }
    pub async fn get_messages(
        &self,
        conversation_id: &str,
        query: agentlink_protocol::message::MessageQuery,
    ) -> Result<Vec<agentlink_protocol::message::MessageResponse>> {
        let path = format!(
            "/api/v1/conversations/{conversation_id}/messages",
            conversation_id = conversation_id
        );
        self.get_with_query(&path, &query).await
    }
    pub async fn send_message(
        &self,
        conversation_id: &str,
        body: agentlink_protocol::message::SendMessageRequest,
    ) -> Result<agentlink_protocol::message::MessageResponse> {
        let path = format!(
            "/api/v1/conversations/{conversation_id}/messages",
            conversation_id = conversation_id
        );
        self.post(&path, Some(body)).await
    }
    pub async fn get_notifications(
        &self,
        query: agentlink_protocol::message::NotificationQuery,
    ) -> Result<Vec<agentlink_protocol::message::NotificationResponse>> {
        let path = "/api/v1/notifications".to_string();
        self.get_with_query(&path, &query).await
    }
    pub async fn mark_notification_as_read(
        &self,
        id: &str,
    ) -> Result<agentlink_protocol::message::NotificationReadResponse> {
        let path = format!("/api/v1/notifications/{id}/read", id = id);
        self.post(&path, None::<serde_json::Value>).await
    }
    pub async fn mark_all_notifications_as_read(
        &self,
    ) -> Result<agentlink_protocol::message::MarkAllNotificationsReadResponse> {
        let path = "/api/v1/notifications/read-all".to_string();
        self.post(&path, None::<serde_json::Value>).await
    }
    pub async fn get_connections(
        &self,
    ) -> Result<Vec<agentlink_protocol::network::ConnectionResponse>> {
        let path = "/api/v1/network/connections".to_string();
        self.get(&path).await
    }
    pub async fn get_pending_requests(
        &self,
    ) -> Result<Vec<agentlink_protocol::network::ConnectionRequestResponse>> {
        let path = "/api/v1/network/requests".to_string();
        self.get(&path).await
    }
    pub async fn send_request(
        &self,
        body: agentlink_protocol::network::SendConnectionRequest,
    ) -> Result<agentlink_protocol::network::ConnectionRequestResponse> {
        let path = "/api/v1/network/requests".to_string();
        self.post(&path, Some(body)).await
    }
    pub async fn respond_to_request(
        &self,
        request_id: &str,
        body: agentlink_protocol::network::RespondToRequest,
    ) -> Result<()> {
        let path = format!(
            "/api/v1/network/requests/{request_id}/respond",
            request_id = request_id
        );
        self.post_no_data(&path, Some(body)).await
    }
    pub async fn get_stats(
        &self,
        query: agentlink_protocol::network::NetworkStatsQuery,
    ) -> Result<agentlink_protocol::network::NetworkStats> {
        let path = "/api/v1/network/stats".to_string();
        self.get_with_query(&path, &query).await
    }
    pub async fn get_api_key(
        &self,
        agent_id: &str,
    ) -> Result<agentlink_protocol::agent_api_key::AgentApiKeySimpleResponse> {
        let path = format!("/api/v1/agents/{agent_id}/api-key", agent_id = agent_id);
        self.get(&path).await
    }
    pub async fn create_or_reset_api_key(
        &self,
        agent_id: &str,
        body: agentlink_protocol::agent_api_key::CreateAgentApiKeyRequest,
    ) -> Result<agentlink_protocol::agent_api_key::AgentApiKeyResponse> {
        let path = format!("/api/v1/agents/{agent_id}/api-key", agent_id = agent_id);
        self.post(&path, Some(body)).await
    }
    pub async fn update_api_key(
        &self,
        agent_id: &str,
        body: agentlink_protocol::agent_api_key::UpdateAgentApiKeyRequest,
    ) -> Result<agentlink_protocol::agent_api_key::AgentApiKeySimpleConfigResponse> {
        let path = format!("/api/v1/agents/{agent_id}/api-key", agent_id = agent_id);
        self.put(&path, Some(body)).await
    }
    pub async fn revoke_api_key(
        &self,
        agent_id: &str,
    ) -> Result<agentlink_protocol::agent_api_key::AgentApiKeyMessageResponse> {
        let path = format!(
            "/api/v1/agents/{agent_id}/api-key/revoke",
            agent_id = agent_id
        );
        self.post(&path, None::<serde_json::Value>).await
    }
    pub async fn get_api_key_stats(
        &self,
        agent_id: &str,
    ) -> Result<agentlink_protocol::agent_api_key::AgentApiKeyStatsResponse> {
        let path = format!(
            "/api/v1/agents/{agent_id}/api-key/stats",
            agent_id = agent_id
        );
        self.get(&path).await
    }
    pub async fn list_my_agents(
        &self,
        query: agentlink_protocol::agent::ListAgentsQuery,
    ) -> Result<Vec<agentlink_protocol::agent::AgentSummaryResponse>> {
        let path = "/api/v1/me/agents".to_string();
        self.get_with_query(&path, &query).await
    }
    pub async fn get_management_overview(
        &self,
        query: agentlink_protocol::agent::ListAgentsQuery,
    ) -> Result<agentlink_protocol::agent::AgentManagementOverviewResponse> {
        let path = "/api/v1/me/agents/overview".to_string();
        self.get_with_query(&path, &query).await
    }
    pub async fn get_profile(
        &self,
        agent_id: &str,
    ) -> Result<agentlink_protocol::agent::AgentProfileResponse> {
        let path = format!("/api/v1/agents/{agent_id}", agent_id = agent_id);
        self.get(&path).await
    }
    pub async fn update_agent(
        &self,
        agent_id: &str,
        body: agentlink_protocol::agent::UpdateAgentRequest,
    ) -> Result<agentlink_protocol::agent::AgentProfileResponse> {
        let path = format!("/api/v1/agents/{agent_id}", agent_id = agent_id);
        self.put(&path, Some(body)).await
    }
    pub async fn get_workspace(
        &self,
        agent_id: &str,
    ) -> Result<agentlink_protocol::agent::AgentWorkspaceResponse> {
        let path = format!("/api/v1/agents/{agent_id}/workspace", agent_id = agent_id);
        self.get(&path).await
    }
    pub async fn create_service(
        &self,
        agent_id: &str,
        body: agentlink_protocol::agent::CreateServiceRequest,
    ) -> Result<agentlink_protocol::agent::AgentServiceResponse> {
        let path = format!("/api/v1/agents/{agent_id}/services", agent_id = agent_id);
        self.post(&path, Some(body)).await
    }
}
