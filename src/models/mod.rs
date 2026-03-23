use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// API 标准响应格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

/// 认证响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub user: User,
    pub token: String,
    #[serde(rename = "expiresIn")]
    pub expires_in: i64,
    #[serde(rename = "isNewUser")]
    pub is_new_user: bool,
    #[serde(rename = "needsOnboarding")]
    pub needs_onboarding: bool,
}

/// 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "userType")]
    pub user_type: String,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
    pub status: String,
    #[serde(rename = "isVerified")]
    pub is_verified: bool,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}

/// 任务信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    #[serde(rename = "posterId")]
    pub poster_id: String,
    pub title: String,
    pub description: String,
    pub kind: String,
    pub status: String,
    #[serde(rename = "budgetMin")]
    pub budget_min: Option<i64>,
    #[serde(rename = "budgetMax")]
    pub budget_max: Option<i64>,
    pub currency: String,
    pub deadline: Option<DateTime<Utc>>,
    #[serde(rename = "locationType")]
    pub location_type: String,
    pub skills: Vec<Skill>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

/// 技能
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
}

/// 申请信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
    pub id: String,
    #[serde(rename = "taskId")]
    pub task_id: String,
    #[serde(rename = "applicantId")]
    pub applicant_id: String,
    pub status: String,
    #[serde(rename = "coverLetter")]
    pub cover_letter: Option<String>,
    #[serde(rename = "proposedBudget")]
    pub proposed_budget: Option<i64>,
    #[serde(rename = "estimatedDays")]
    pub estimated_days: Option<i32>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}

/// 会话信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub participants: Vec<Participant>,
    #[serde(rename = "lastMessage")]
    pub last_message: Option<LastMessage>,
    #[serde(rename = "unreadCount")]
    pub unread_count: i32,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastMessage {
    pub content: String,
    #[serde(rename = "senderId")]
    pub sender_id: String,
    #[serde(rename = "sentAt")]
    pub sent_at: DateTime<Utc>,
}

/// 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    #[serde(rename = "conversationId")]
    pub conversation_id: String,
    #[serde(rename = "senderId")]
    pub sender_id: String,
    #[serde(rename = "senderName")]
    pub sender_name: String,
    #[serde(rename = "senderAvatar")]
    pub sender_avatar: Option<String>,
    pub content: String,
    #[serde(rename = "contentType")]
    pub content_type: String,
    #[serde(rename = "sentAt")]
    pub sent_at: DateTime<Utc>,
    #[serde(rename = "readAt")]
    pub read_at: Option<DateTime<Utc>>,
}

/// 通知
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    #[serde(rename = "type")]
    pub notification_type: String,
    pub title: String,
    pub content: Option<String>,
    #[serde(rename = "relatedId")]
    pub related_id: Option<String>,
    #[serde(rename = "isRead")]
    pub is_read: bool,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}

/// 人脉连接
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: String,
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
    #[serde(rename = "userType")]
    pub user_type: String,
    #[serde(rename = "connectedAt")]
    pub connected_at: DateTime<Utc>,
}

/// 人脉请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionRequest {
    pub id: String,
    #[serde(rename = "fromUser")]
    pub from_user: Participant,
    pub message: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}

/// API Key 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: String,
    #[serde(rename = "userId")]
    pub user_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    #[serde(rename = "apiKeyPreview")]
    pub api_key_preview: String,
    #[serde(rename = "isActive")]
    pub is_active: bool,
    #[serde(rename = "lastUsedAt")]
    pub last_used_at: Option<DateTime<Utc>>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<DateTime<Utc>>,
}

/// 分页元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub page: i64,
    #[serde(rename = "per_page")]
    pub per_page: i64,
    pub total: i64,
    #[serde(rename = "total_pages")]
    pub total_pages: i64,
}

/// 分页响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub meta: PaginationMeta,
}

/// Agent 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
    pub capabilities: Vec<String>,
    #[serde(rename = "successRate")]
    pub success_rate: f64,
    #[serde(rename = "completedTasks")]
    pub completed_tasks: i32,
    #[serde(rename = "responseTime")]
    pub response_time: Option<String>,
}

/// 服务信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub price: i64,
    pub unit: String,
}
