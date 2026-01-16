use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageStatus {
    Enqueued,
    Sent,
    Replied,
    AiEnqueued,
    AiReplied,
    FollowUp,
    Closed,
}

impl MessageStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageStatus::Enqueued => "enqueued",
            MessageStatus::Sent => "sent",
            MessageStatus::Replied => "replied",
            MessageStatus::AiEnqueued => "ai_enqueued",
            MessageStatus::AiReplied => "ai_replied",
            MessageStatus::FollowUp => "follow_up",
            MessageStatus::Closed => "closed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Lead {
    pub id: i64,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLeadRequest {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: i64,
    pub leads_id: i64,
    pub message_sent: Option<String>,
    pub sent_at: Option<String>,
    pub reply_received: Option<String>,
    pub reply_received_at: Option<String>,
    pub ai_reply: Option<String>,
    pub ai_reply_sent: Option<String>,
    pub created_at: String,
    pub status: String,
    pub follow_up_at: Option<String>,
    pub closed_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub lead_id: i64,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct ReplyRequest {
    pub message_id: i64,
    pub reply: String,
}

#[derive(Debug, Deserialize)]
pub struct AiReplyRequest {
    pub message_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OutreachLog {
    pub id: i64,
    pub message_id: i64,
    pub log_at: String,
    pub step: String,
}

#[derive(Debug, Serialize)]
pub struct LeadWithDetails {
    pub lead: Lead,
    pub messages: Vec<Message>,
    pub outreach_logs: Vec<OutreachLog>,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
}
