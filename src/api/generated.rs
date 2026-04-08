// This file is generated from protocol/src/http_surface.rs. Do not edit manually.
use super::ApiClient;
use anyhow::Result;

impl ApiClient {
    pub async fn get_platform_dashboard(
        &self,
        query: agentlink_protocol::analytics::DashboardQuery,
    ) -> Result<agentlink_protocol::analytics::PlatformDashboardResponse> {
        let path = "/api/v1/analytics/dashboard".to_string();
        self.get_with_query(&path, &query).await
    }
    pub async fn get_task_market_analysis(
        &self,
        query: agentlink_protocol::analytics::DashboardQuery,
    ) -> Result<agentlink_protocol::analytics::TaskMarketResponse> {
        let path = "/api/v1/analytics/task-market".to_string();
        self.get_with_query(&path, &query).await
    }
    pub async fn get_user_activity(
        &self,
        query: agentlink_protocol::analytics::DashboardQuery,
    ) -> Result<agentlink_protocol::analytics::UserActivityResponse> {
        let path = "/api/v1/analytics/user-activity".to_string();
        self.get_with_query(&path, &query).await
    }
    pub async fn get_skill_analysis(
        &self,
    ) -> Result<agentlink_protocol::analytics::SkillAnalysisResponse> {
        let path = "/api/v1/analytics/skills".to_string();
        self.get(&path).await
    }
    pub async fn check_data_freshness(
        &self,
    ) -> Result<agentlink_protocol::analytics::DataFreshnessResponse> {
        let path = "/api/v1/analytics/freshness".to_string();
        self.get(&path).await
    }
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
    pub async fn get_user_by_id(&self, id: &str) -> Result<agentlink_protocol::user::UserResponse> {
        let path = format!("/api/v1/users/{id}", id = id);
        self.get(&path).await
    }
    pub async fn get_social_stats(
        &self,
        id: &str,
    ) -> Result<agentlink_protocol::user::UserSocialStats> {
        let path = format!("/api/v1/users/{id}/social-stats", id = id);
        self.get(&path).await
    }
    pub async fn list_users(
        &self,
        query: agentlink_protocol::user::UserSearchQuery,
    ) -> Result<agentlink_protocol::PaginatedResponse<agentlink_protocol::user::UserResponse>> {
        let path = "/api/v1/users".to_string();
        self.get_with_query(&path, &query).await
    }
    pub async fn update_profile(
        &self,
        body: agentlink_protocol::user::UpdateProfileRequest,
    ) -> Result<agentlink_protocol::user::UserResponse> {
        let path = "/api/v1/users/me/profile".to_string();
        self.put(&path, Some(body)).await
    }
    pub async fn list_skills(&self) -> Result<Vec<agentlink_protocol::user::Skill>> {
        let path = "/api/v1/skills".to_string();
        self.get(&path).await
    }
    pub async fn update_my_skills(
        &self,
        body: agentlink_protocol::user::UpdateUserSkillsRequest,
    ) -> Result<Vec<agentlink_protocol::user::UserSkillResponse>> {
        let path = "/api/v1/users/me/skills".to_string();
        self.put(&path, Some(body)).await
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
    pub async fn create_task(
        &self,
        body: agentlink_protocol::task::CreateTaskRequest,
    ) -> Result<agentlink_protocol::task::TaskResponse> {
        let path = "/api/v1/tasks".to_string();
        self.post(&path, Some(body)).await
    }
    pub async fn update_task(
        &self,
        id: &str,
        body: agentlink_protocol::task::UpdateTaskRequest,
    ) -> Result<agentlink_protocol::task::TaskResponse> {
        let path = format!("/api/v1/tasks/{id}", id = id);
        self.put(&path, Some(body)).await
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
    pub async fn get_applications(
        &self,
        id: &str,
    ) -> Result<Vec<agentlink_protocol::task::ApplicationResponse>> {
        let path = format!("/api/v1/tasks/{id}/applications", id = id);
        self.get(&path).await
    }
    pub async fn get_task_comments(
        &self,
        id: &str,
    ) -> Result<Vec<agentlink_protocol::comment::CommentResponse>> {
        let path = format!("/api/v1/tasks/{id}/comments", id = id);
        self.get(&path).await
    }
    pub async fn create_task_comment(
        &self,
        id: &str,
        body: agentlink_protocol::comment::CreateCommentRequest,
    ) -> Result<agentlink_protocol::comment::CommentResponse> {
        let path = format!("/api/v1/tasks/{id}/comments", id = id);
        self.post(&path, Some(body)).await
    }
    pub async fn get_task_subscription(
        &self,
        id: &str,
    ) -> Result<agentlink_protocol::task::TaskSubscriptionStatusResponse> {
        let path = format!("/api/v1/tasks/{id}/subscription", id = id);
        self.get(&path).await
    }
    pub async fn subscribe_to_task(
        &self,
        id: &str,
    ) -> Result<agentlink_protocol::task::TaskSubscriptionStatusResponse> {
        let path = format!("/api/v1/tasks/{id}/subscribe", id = id);
        self.post(&path, None::<serde_json::Value>).await
    }
    pub async fn unsubscribe_from_task(
        &self,
        id: &str,
    ) -> Result<agentlink_protocol::task::TaskSubscriptionStatusResponse> {
        let path = format!("/api/v1/tasks/{id}/unsubscribe", id = id);
        self.delete(&path).await
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
    pub async fn get_notification_settings(
        &self,
    ) -> Result<agentlink_protocol::message::NotificationSettingsResponse> {
        let path = "/api/v1/notifications/settings".to_string();
        self.get(&path).await
    }
    pub async fn update_notification_settings(
        &self,
        body: agentlink_protocol::message::UpdateNotificationSettingsRequest,
    ) -> Result<agentlink_protocol::message::NotificationSettingsResponse> {
        let path = "/api/v1/notifications/settings".to_string();
        self.put(&path, Some(body)).await
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
    pub async fn follow_user(
        &self,
        body: agentlink_protocol::network::FollowRequest,
    ) -> Result<()> {
        let path = "/api/v1/network/follow".to_string();
        self.post_no_data(&path, Some(body)).await
    }
    pub async fn list_follows(
        &self,
        query: agentlink_protocol::network::FollowListQuery,
    ) -> Result<agentlink_protocol::PaginatedResponse<agentlink_protocol::network::FollowListItem>>
    {
        let path = "/api/v1/network/follows".to_string();
        self.get_with_query(&path, &query).await
    }
    pub async fn unfollow_user(&self, user_id: &str) -> Result<()> {
        let path = format!("/api/v1/network/unfollow/{user_id}", user_id = user_id);
        self.delete_no_data(&path).await
    }
    pub async fn get_stats(
        &self,
        query: agentlink_protocol::network::NetworkStatsQuery,
    ) -> Result<agentlink_protocol::network::NetworkStats> {
        let path = "/api/v1/network/stats".to_string();
        self.get_with_query(&path, &query).await
    }
    pub async fn get_platform_stats(&self) -> Result<agentlink_protocol::stats::PlatformStats> {
        let path = "/api/v1/stats/platform".to_string();
        self.get(&path).await
    }
    pub async fn get_trending_searches(
        &self,
        query: agentlink_protocol::stats::TrendingSearchesQuery,
    ) -> Result<Vec<agentlink_protocol::stats::TrendingSearch>> {
        let path = "/api/v1/stats/trending-searches".to_string();
        self.get_with_query(&path, &query).await
    }
    pub async fn reviews_list_reviews(
        &self,
        query: agentlink_protocol::review::ReviewListQuery,
    ) -> Result<Vec<agentlink_protocol::review::ReviewResponse>> {
        let path = "/api/v1/reviews".to_string();
        self.get_with_query(&path, &query).await
    }
    pub async fn create_review(
        &self,
        body: agentlink_protocol::review::CreateReviewRequest,
    ) -> Result<agentlink_protocol::review::ReviewResponse> {
        let path = "/api/v1/reviews".to_string();
        self.post(&path, Some(body)).await
    }
    pub async fn update_review(
        &self,
        id: &str,
        body: agentlink_protocol::review::UpdateReviewRequest,
    ) -> Result<agentlink_protocol::review::ReviewResponse> {
        let path = format!("/api/v1/reviews/{id}", id = id);
        self.put(&path, Some(body)).await
    }
    pub async fn delete_review(&self, id: &str) -> Result<()> {
        let path = format!("/api/v1/reviews/{id}", id = id);
        self.delete_no_data(&path).await
    }
    pub async fn get_review_stats(
        &self,
        id: &str,
    ) -> Result<agentlink_protocol::review::ReviewStats> {
        let path = format!("/api/v1/reviews/{id}/stats", id = id);
        self.get(&path).await
    }
    pub async fn list_posts(
        &self,
        query: agentlink_protocol::social::PostListQuery,
    ) -> Result<Vec<agentlink_protocol::social::PostResponse>> {
        let path = "/api/v1/posts".to_string();
        self.get_with_query(&path, &query).await
    }
    pub async fn create_post(
        &self,
        body: agentlink_protocol::social::CreatePostRequest,
    ) -> Result<agentlink_protocol::social::PostResponse> {
        let path = "/api/v1/posts".to_string();
        self.post(&path, Some(body)).await
    }
    pub async fn get_post(&self, id: &str) -> Result<agentlink_protocol::social::PostResponse> {
        let path = format!("/api/v1/posts/{id}", id = id);
        self.get(&path).await
    }
    pub async fn delete_post(&self, id: &str) -> Result<()> {
        let path = format!("/api/v1/posts/{id}", id = id);
        self.delete_no_data(&path).await
    }
    pub async fn like_post(&self, id: &str) -> Result<()> {
        let path = format!("/api/v1/posts/{id}/like", id = id);
        self.post_no_data(&path, None::<serde_json::Value>).await
    }
    pub async fn unlike_post(&self, id: &str) -> Result<()> {
        let path = format!("/api/v1/posts/{id}/unlike", id = id);
        self.delete_no_data(&path).await
    }
    pub async fn get_comments(
        &self,
        id: &str,
    ) -> Result<Vec<agentlink_protocol::comment::CommentResponse>> {
        let path = format!("/api/v1/posts/{id}/comments", id = id);
        self.get(&path).await
    }
    pub async fn create_comment(
        &self,
        id: &str,
        body: agentlink_protocol::comment::CreateCommentRequest,
    ) -> Result<agentlink_protocol::comment::CommentResponse> {
        let path = format!("/api/v1/posts/{id}/comments", id = id);
        self.post(&path, Some(body)).await
    }
    pub async fn create_share(
        &self,
        body: agentlink_protocol::social::CreateShareRequest,
    ) -> Result<agentlink_protocol::social::ShareActionResponse> {
        let path = "/api/v1/shares".to_string();
        self.post(&path, Some(body)).await
    }
    pub async fn list_favorites(
        &self,
        query: agentlink_protocol::social::FavoriteListQuery,
    ) -> Result<agentlink_protocol::social::FavoriteListResponse> {
        let path = "/api/v1/favorites".to_string();
        self.get_with_query(&path, &query).await
    }
    pub async fn create_favorite(
        &self,
        body: agentlink_protocol::social::CreateFavoriteRequest,
    ) -> Result<agentlink_protocol::social::FavoriteActionResponse> {
        let path = "/api/v1/favorites".to_string();
        self.post(&path, Some(body)).await
    }
    pub async fn delete_favorite(
        &self,
        target_type: &str,
        target_id: &str,
    ) -> Result<agentlink_protocol::social::FavoriteActionResponse> {
        let path = format!(
            "/api/v1/favorites/{target_type}/{target_id}",
            target_type = target_type,
            target_id = target_id
        );
        self.delete(&path).await
    }
    pub async fn get_feed(
        &self,
        query: agentlink_protocol::social::FeedQuery,
    ) -> Result<agentlink_protocol::social::FeedResponse> {
        let path = "/api/v1/feed".to_string();
        self.get_with_query(&path, &query).await
    }
    pub async fn list_api_keys(
        &self,
        agent_id: &str,
    ) -> Result<agentlink_protocol::agent_api_key::AgentApiKeyListResponse> {
        let path = format!("/api/v1/agents/{agent_id}/api-keys", agent_id = agent_id);
        self.get(&path).await
    }
    pub async fn create_api_key(
        &self,
        agent_id: &str,
        body: agentlink_protocol::agent_api_key::CreateAgentApiKeyRequest,
    ) -> Result<agentlink_protocol::agent_api_key::AgentApiKeyResponse> {
        let path = format!("/api/v1/agents/{agent_id}/api-keys", agent_id = agent_id);
        self.post(&path, Some(body)).await
    }
    pub async fn api_keys_get_api_key(
        &self,
        agent_id: &str,
        key_id: &str,
    ) -> Result<agentlink_protocol::agent_api_key::AgentApiKeyResponse> {
        let path = format!(
            "/api/v1/agents/{agent_id}/api-keys/{key_id}",
            agent_id = agent_id,
            key_id = key_id
        );
        self.get(&path).await
    }
    pub async fn api_keys_update_api_key(
        &self,
        agent_id: &str,
        key_id: &str,
        body: agentlink_protocol::agent_api_key::UpdateAgentApiKeyRequest,
    ) -> Result<agentlink_protocol::agent_api_key::AgentApiKeyConfigResponse> {
        let path = format!(
            "/api/v1/agents/{agent_id}/api-keys/{key_id}",
            agent_id = agent_id,
            key_id = key_id
        );
        self.put(&path, Some(body)).await
    }
    pub async fn api_keys_delete_api_key(
        &self,
        agent_id: &str,
        key_id: &str,
    ) -> Result<agentlink_protocol::agent_api_key::AgentApiKeyMessageResponse> {
        let path = format!(
            "/api/v1/agents/{agent_id}/api-keys/{key_id}",
            agent_id = agent_id,
            key_id = key_id
        );
        self.delete(&path).await
    }
    pub async fn regenerate_api_key(
        &self,
        agent_id: &str,
        key_id: &str,
        body: agentlink_protocol::agent_api_key::RegenerateAgentApiKeyRequest,
    ) -> Result<agentlink_protocol::agent_api_key::AgentApiKeyResponse> {
        let path = format!(
            "/api/v1/agents/{agent_id}/api-keys/{key_id}/regenerate",
            agent_id = agent_id,
            key_id = key_id
        );
        self.post(&path, Some(body)).await
    }
    pub async fn api_keys_get_api_key_stats(
        &self,
        agent_id: &str,
        key_id: &str,
    ) -> Result<agentlink_protocol::agent_api_key::AgentApiKeyStatsResponse> {
        let path = format!(
            "/api/v1/agents/{agent_id}/api-keys/{key_id}/stats",
            agent_id = agent_id,
            key_id = key_id
        );
        self.get(&path).await
    }
    pub async fn agent_api_keys_get_api_key(
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
    pub async fn agent_api_keys_update_api_key(
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
    pub async fn agent_api_keys_delete_api_key(
        &self,
        agent_id: &str,
    ) -> Result<agentlink_protocol::agent_api_key::AgentApiKeyMessageResponse> {
        let path = format!("/api/v1/agents/{agent_id}/api-key", agent_id = agent_id);
        self.delete(&path).await
    }
    pub async fn agent_api_keys_get_api_key_stats(
        &self,
        agent_id: &str,
    ) -> Result<agentlink_protocol::agent_api_key::AgentApiKeyStatsResponse> {
        let path = format!(
            "/api/v1/agents/{agent_id}/api-key/stats",
            agent_id = agent_id
        );
        self.get(&path).await
    }
    pub async fn get_my_point_account(
        &self,
    ) -> Result<agentlink_protocol::growth::PointAccountView> {
        let path = "/api/v1/points/me".to_string();
        self.get(&path).await
    }
    pub async fn list_my_point_ledger(
        &self,
        query: agentlink_protocol::growth::PaginationQuery,
    ) -> Result<
        agentlink_protocol::PaginatedResponse<agentlink_protocol::growth::PointLedgerEntryView>,
    > {
        let path = "/api/v1/points/ledger".to_string();
        self.get_with_query(&path, &query).await
    }
    pub async fn get_my_referrals(
        &self,
    ) -> Result<agentlink_protocol::growth::ReferralOverviewResponse> {
        let path = "/api/v1/referrals/me".to_string();
        self.get(&path).await
    }
    pub async fn get_my_referral_hub(
        &self,
    ) -> Result<agentlink_protocol::growth::ReferralHubResponse> {
        let path = "/api/v1/referrals/me/hub".to_string();
        self.get(&path).await
    }
    pub async fn bind_referral(
        &self,
        body: agentlink_protocol::growth::BindReferralRequest,
    ) -> Result<agentlink_protocol::growth::BindReferralResponse> {
        let path = "/api/v1/referrals/bind".to_string();
        self.post(&path, Some(body)).await
    }
    pub async fn list_grants(
        &self,
        query: agentlink_protocol::growth::AdminListQuery,
    ) -> Result<
        agentlink_protocol::PaginatedResponse<agentlink_protocol::growth::GrowthRewardGrantView>,
    > {
        let path = "/api/v1/admin/growth/grants".to_string();
        self.get_with_query(&path, &query).await
    }
    pub async fn get_dashboard(
        &self,
    ) -> Result<agentlink_protocol::growth::GrowthDashboardResponse> {
        let path = "/api/v1/admin/growth/dashboard".to_string();
        self.get(&path).await
    }
    pub async fn list_campaigns(
        &self,
        query: agentlink_protocol::growth::PaginationQuery,
    ) -> Result<agentlink_protocol::PaginatedResponse<agentlink_protocol::growth::GrowthCampaignView>>
    {
        let path = "/api/v1/admin/growth/campaigns".to_string();
        self.get_with_query(&path, &query).await
    }
    pub async fn create_campaign(
        &self,
        body: agentlink_protocol::growth::CreateCampaignRequest,
    ) -> Result<agentlink_protocol::growth::GrowthCampaignView> {
        let path = "/api/v1/admin/growth/campaigns".to_string();
        self.post(&path, Some(body)).await
    }
    pub async fn update_campaign(
        &self,
        campaign_code: &str,
        body: agentlink_protocol::growth::UpdateCampaignRequest,
    ) -> Result<agentlink_protocol::growth::GrowthCampaignView> {
        let path = format!(
            "/api/v1/admin/growth/campaigns/{campaign_code}",
            campaign_code = campaign_code
        );
        self.put(&path, Some(body)).await
    }
    pub async fn list_reward_rules(
        &self,
        query: agentlink_protocol::growth::PaginationQuery,
    ) -> Result<
        agentlink_protocol::PaginatedResponse<agentlink_protocol::growth::GrowthRewardRuleView>,
    > {
        let path = "/api/v1/admin/growth/rules".to_string();
        self.get_with_query(&path, &query).await
    }
    pub async fn create_reward_rule(
        &self,
        body: agentlink_protocol::growth::CreateRewardRuleRequest,
    ) -> Result<agentlink_protocol::growth::GrowthRewardRuleView> {
        let path = "/api/v1/admin/growth/rules".to_string();
        self.post(&path, Some(body)).await
    }
    pub async fn update_reward_rule(
        &self,
        rule_code: &str,
        body: agentlink_protocol::growth::UpdateRewardRuleRequest,
    ) -> Result<agentlink_protocol::growth::GrowthRewardRuleView> {
        let path = format!(
            "/api/v1/admin/growth/rules/{rule_code}",
            rule_code = rule_code
        );
        self.put(&path, Some(body)).await
    }
    pub async fn get_referrals_overview(
        &self,
    ) -> Result<agentlink_protocol::growth::ReferralSummary> {
        let path = "/api/v1/admin/growth/referrals/overview".to_string();
        self.get(&path).await
    }
    pub async fn list_accounts(
        &self,
        query: agentlink_protocol::growth::AdminListQuery,
    ) -> Result<
        agentlink_protocol::PaginatedResponse<agentlink_protocol::growth::AdminPointAccountView>,
    > {
        let path = "/api/v1/admin/growth/accounts".to_string();
        self.get_with_query(&path, &query).await
    }
    pub async fn list_admin_referrals(
        &self,
        query: agentlink_protocol::growth::AdminReferralListQuery,
    ) -> Result<
        agentlink_protocol::PaginatedResponse<agentlink_protocol::growth::AdminReferralItemView>,
    > {
        let path = "/api/v1/admin/growth/referrals".to_string();
        self.get_with_query(&path, &query).await
    }
    pub async fn growth_list_reviews(
        &self,
        query: agentlink_protocol::growth::PaginationQuery,
    ) -> Result<
        agentlink_protocol::PaginatedResponse<agentlink_protocol::growth::GrowthPendingReviewView>,
    > {
        let path = "/api/v1/admin/growth/reviews".to_string();
        self.get_with_query(&path, &query).await
    }
    pub async fn decide_review(
        &self,
        grant_id: &str,
        body: agentlink_protocol::growth::ReviewDecisionRequest,
    ) -> Result<agentlink_protocol::growth::GrowthRewardGrantView> {
        let path = format!(
            "/api/v1/admin/growth/reviews/{grant_id}/decision",
            grant_id = grant_id
        );
        self.post(&path, Some(body)).await
    }
    pub async fn create_manual_grant(
        &self,
        body: agentlink_protocol::growth::ManualGrantRequest,
    ) -> Result<agentlink_protocol::growth::GrowthRewardGrantView> {
        let path = "/api/v1/admin/growth/manual-grants".to_string();
        self.post(&path, Some(body)).await
    }
    pub async fn reverse_grant(
        &self,
        body: agentlink_protocol::growth::ReversalRequest,
    ) -> Result<agentlink_protocol::growth::GrowthRewardGrantView> {
        let path = "/api/v1/admin/growth/reversals".to_string();
        self.post(&path, Some(body)).await
    }
    pub async fn list_risk_audits(
        &self,
        query: agentlink_protocol::growth::AdminRiskAuditListQuery,
    ) -> Result<
        agentlink_protocol::PaginatedResponse<agentlink_protocol::growth::GrowthRiskAuditView>,
    > {
        let path = "/api/v1/admin/growth/risk-audits".to_string();
        self.get_with_query(&path, &query).await
    }
    pub async fn get_events_trace(
        &self,
    ) -> Result<agentlink_protocol::growth::GrowthEventsTraceResponse> {
        let path = "/api/v1/admin/growth/events-trace".to_string();
        self.get(&path).await
    }
    pub async fn list_agents(
        &self,
        query: agentlink_protocol::agent::ListAgentsQuery,
    ) -> Result<Vec<agentlink_protocol::agent::AgentSummaryResponse>> {
        let path = "/api/v1/agents".to_string();
        self.get_with_query(&path, &query).await
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
    pub async fn create_agent(
        &self,
        body: agentlink_protocol::agent::CreateAgentRequest,
    ) -> Result<agentlink_protocol::agent::CreateAgentResponse> {
        let path = "/api/v1/agents".to_string();
        self.post(&path, Some(body)).await
    }
    pub async fn update_agent(
        &self,
        agent_id: &str,
        body: agentlink_protocol::agent::UpdateAgentRequest,
    ) -> Result<agentlink_protocol::agent::AgentProfileResponse> {
        let path = format!("/api/v1/agents/{agent_id}", agent_id = agent_id);
        self.put(&path, Some(body)).await
    }
    pub async fn delete_agent(&self, agent_id: &str) -> Result<()> {
        let path = format!("/api/v1/agents/{agent_id}", agent_id = agent_id);
        self.delete_no_data(&path).await
    }
    pub async fn search(
        &self,
        query: agentlink_protocol::agent::AgentSearchQuery,
    ) -> Result<
        agentlink_protocol::PaginatedResponse<agentlink_protocol::agent::AgentProfileResponse>,
    > {
        let path = "/api/v1/agents/search".to_string();
        self.get_with_query(&path, &query).await
    }
    pub async fn get_workspace(
        &self,
        agent_id: &str,
    ) -> Result<agentlink_protocol::agent::AgentWorkspaceResponse> {
        let path = format!("/api/v1/agents/{agent_id}/workspace", agent_id = agent_id);
        self.get(&path).await
    }
    pub async fn list_services(
        &self,
        agent_id: &str,
    ) -> Result<Vec<agentlink_protocol::agent::AgentServiceResponse>> {
        let path = format!("/api/v1/agents/{agent_id}/services", agent_id = agent_id);
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
    pub async fn update_service(
        &self,
        agent_id: &str,
        service_id: &str,
        body: agentlink_protocol::agent::UpdateServiceRequest,
    ) -> Result<agentlink_protocol::agent::AgentServiceResponse> {
        let path = format!(
            "/api/v1/agents/{agent_id}/services/{service_id}",
            agent_id = agent_id,
            service_id = service_id
        );
        self.put(&path, Some(body)).await
    }
    pub async fn delete_service(&self, agent_id: &str, service_id: &str) -> Result<()> {
        let path = format!(
            "/api/v1/agents/{agent_id}/services/{service_id}",
            agent_id = agent_id,
            service_id = service_id
        );
        self.delete_no_data(&path).await
    }
    pub async fn list_expertise(
        &self,
        agent_id: &str,
    ) -> Result<Vec<agentlink_protocol::agent::AgentExpertiseResponse>> {
        let path = format!("/api/v1/agents/{agent_id}/expertise", agent_id = agent_id);
        self.get(&path).await
    }
    pub async fn add_expertise(
        &self,
        agent_id: &str,
        body: agentlink_protocol::agent::CreateExpertiseRequest,
    ) -> Result<agentlink_protocol::agent::AgentExpertiseResponse> {
        let path = format!("/api/v1/agents/{agent_id}/expertise", agent_id = agent_id);
        self.post(&path, Some(body)).await
    }
    pub async fn remove_expertise(&self, agent_id: &str, expertise_id: &str) -> Result<()> {
        let path = format!(
            "/api/v1/agents/{agent_id}/expertise/{expertise_id}",
            agent_id = agent_id,
            expertise_id = expertise_id
        );
        self.delete_no_data(&path).await
    }
    pub async fn list_works(
        &self,
        agent_id: &str,
    ) -> Result<Vec<agentlink_protocol::agent::AgentWorkResponse>> {
        let path = format!("/api/v1/agents/{agent_id}/works", agent_id = agent_id);
        self.get(&path).await
    }
    pub async fn create_work(
        &self,
        agent_id: &str,
        body: agentlink_protocol::agent::CreateWorkRequest,
    ) -> Result<agentlink_protocol::agent::AgentWorkResponse> {
        let path = format!("/api/v1/agents/{agent_id}/works", agent_id = agent_id);
        self.post(&path, Some(body)).await
    }
    pub async fn update_work(
        &self,
        agent_id: &str,
        work_id: &str,
        body: agentlink_protocol::agent::CreateWorkRequest,
    ) -> Result<agentlink_protocol::agent::AgentWorkResponse> {
        let path = format!(
            "/api/v1/agents/{agent_id}/works/{work_id}",
            agent_id = agent_id,
            work_id = work_id
        );
        self.put(&path, Some(body)).await
    }
    pub async fn delete_work(&self, agent_id: &str, work_id: &str) -> Result<()> {
        let path = format!(
            "/api/v1/agents/{agent_id}/works/{work_id}",
            agent_id = agent_id,
            work_id = work_id
        );
        self.delete_no_data(&path).await
    }
    pub async fn generate_task_description(
        &self,
        body: agentlink_protocol::ai::GenerateTaskDescriptionRequest,
    ) -> Result<reqwest::Response> {
        let path = "/api/v1/tasks/generate-description".to_string();
        self.post_stream(&path, Some(body)).await
    }
}
