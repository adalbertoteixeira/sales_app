use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use sqlx::SqlitePool;
use tracing::{error, info};

use crate::models::{
    AiReplyRequest, ApiError, CreateLeadRequest, Lead, LeadWithDetails, Message, MessageStatus,
    OutreachLog, ReplyRequest, SendMessageRequest,
};

type ApiResult<T> = Result<(StatusCode, Json<T>), (StatusCode, Json<ApiError>)>;

fn api_error(status: StatusCode, message: &str) -> (StatusCode, Json<ApiError>) {
    (
        status,
        Json(ApiError {
            error: message.to_string(),
        }),
    )
}

pub async fn create_lead(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateLeadRequest>,
) -> ApiResult<Lead> {
    info!("Creating lead: {:?}", payload);

    if payload.name.trim().is_empty() {
        return Err(api_error(StatusCode::BAD_REQUEST, "Name is required"));
    }

    if payload.email.is_none() && payload.phone.is_none() {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "At least one of email or phone is required",
        ));
    }

    let result = sqlx::query_as::<_, Lead>(
        r#"
        INSERT INTO leads (name, email, phone)
        VALUES (?, ?, ?)
        RETURNING id, name, email, phone
        "#,
    )
    .bind(&payload.name)
    .bind(&payload.email)
    .bind(&payload.phone)
    .fetch_one(&pool)
    .await;

    match result {
        Ok(lead) => {
            info!("Lead created with id: {}", lead.id);
            Ok((StatusCode::CREATED, Json(lead)))
        }
        Err(e) => {
            error!("Failed to create lead: {}", e);
            Err(api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create lead",
            ))
        }
    }
}

pub async fn send_message(
    State(pool): State<SqlitePool>,
    Json(payload): Json<SendMessageRequest>,
) -> ApiResult<Message> {
    info!("Enqueueing message for lead_id: {}", payload.lead_id);

    let lead_exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM leads WHERE id = ?")
        .bind(payload.lead_id)
        .fetch_one(&pool)
        .await;

    match lead_exists {
        Ok(count) if count == 0 => {
            return Err(api_error(StatusCode::NOT_FOUND, "Lead not found"));
        }
        Err(e) => {
            error!("Failed to check lead existence: {}", e);
            return Err(api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error",
            ));
        }
        _ => {}
    }

    let now = Utc::now().to_rfc3339();
    let status = MessageStatus::Enqueued.as_str();

    let result = sqlx::query_as::<_, Message>(
        r#"
        INSERT INTO messages (leads_id, message_sent, created_at, status)
        VALUES (?, ?, ?, ?)
        RETURNING id, leads_id, message_sent, sent_at, reply_received, reply_received_at, ai_reply, ai_reply_sent, created_at, status, follow_up_at, closed_at
        "#,
    )
    .bind(payload.lead_id)
    .bind(&payload.message)
    .bind(&now)
    .bind(status)
    .fetch_one(&pool)
    .await;

    match result {
        Ok(message) => {
            log_outreach(&pool, message.id, MessageStatus::Enqueued).await;
            info!("Message enqueued with id: {}", message.id);
            Ok((StatusCode::CREATED, Json(message)))
        }
        Err(e) => {
            error!("Failed to create message: {}", e);
            Err(api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create message",
            ))
        }
    }
}

pub async fn reply_to_message(
    State(pool): State<SqlitePool>,
    Json(payload): Json<ReplyRequest>,
) -> ApiResult<Message> {
    info!(
        "Recording reply for message_id: {}, reply: {}",
        payload.message_id, payload.reply
    );

    let now = Utc::now().to_rfc3339();
    let status = MessageStatus::Replied.as_str();

    let result = sqlx::query_as::<_, Message>(
        r#"
        UPDATE messages
        SET reply_received = ?, reply_received_at = ?, status = ?
        WHERE id = ?
        RETURNING id, leads_id, message_sent, sent_at, reply_received, reply_received_at, ai_reply, ai_reply_sent, created_at, status, follow_up_at, closed_at
        "#,
    )
    .bind(&payload.reply)
    .bind(&now)
    .bind(status)
    .bind(payload.message_id)
    .fetch_optional(&pool)
    .await;

    match result {
        Ok(Some(message)) => {
            log_outreach(&pool, message.id, MessageStatus::Replied).await;
            info!("Reply recorded for message_id: {}", message.id);
            Ok((StatusCode::OK, Json(message)))
        }
        Ok(None) => Err(api_error(StatusCode::NOT_FOUND, "Message not found")),
        Err(e) => {
            error!("Failed to update message: {}", e);
            Err(api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update message",
            ))
        }
    }
}

pub async fn ai_reply(
    State(pool): State<SqlitePool>,
    Json(payload): Json<AiReplyRequest>,
) -> ApiResult<Message> {
    info!("Generating AI reply for message_id: {}", payload.message_id);

    let ai_response = "Thank you for your interest! Our team will follow up shortly.";
    let status = MessageStatus::AiEnqueued.as_str();

    let result = sqlx::query_as::<_, Message>(
        r#"
        UPDATE messages
        SET ai_reply = ?, status = ?
        WHERE id = ?
        RETURNING id, leads_id, message_sent, sent_at, reply_received, reply_received_at, ai_reply, ai_reply_sent, created_at, status, follow_up_at, closed_at
        "#,
    )
    .bind(ai_response)
    .bind(status)
    .bind(payload.message_id)
    .fetch_optional(&pool)
    .await;

    match result {
        Ok(Some(message)) => {
            log_outreach(&pool, message.id, MessageStatus::AiEnqueued).await;
            info!("AI reply enqueued for message_id: {}", message.id);
            Ok((StatusCode::OK, Json(message)))
        }
        Ok(None) => Err(api_error(StatusCode::NOT_FOUND, "Message not found")),
        Err(e) => {
            error!("Failed to update message: {}", e);
            Err(api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update message",
            ))
        }
    }
}

pub async fn get_lead(
    State(pool): State<SqlitePool>,
    Path(lead_id): Path<i64>,
) -> ApiResult<LeadWithDetails> {
    info!("Fetching lead with id: {}", lead_id);

    let lead = sqlx::query_as::<_, Lead>("SELECT id, name, email, phone FROM leads WHERE id = ?")
        .bind(lead_id)
        .fetch_optional(&pool)
        .await;

    let lead = match lead {
        Ok(Some(lead)) => lead,
        Ok(None) => return Err(api_error(StatusCode::NOT_FOUND, "Lead not found")),
        Err(e) => {
            error!("Failed to fetch lead: {}", e);
            return Err(api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error",
            ));
        }
    };

    let messages = sqlx::query_as::<_, Message>(
        r#"
        SELECT id, leads_id, message_sent, sent_at, reply_received, reply_received_at, ai_reply, ai_reply_sent, created_at, status, follow_up_at, closed_at
        FROM messages
        WHERE leads_id = ?
        ORDER BY created_at DESC
        "#,
    )
    .bind(lead_id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let message_ids: Vec<i64> = messages.iter().map(|m| m.id).collect();

    let outreach_logs = if !message_ids.is_empty() {
        let placeholders = message_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");
        let query = format!(
            "SELECT id, message_id, log_at, step FROM outreach_log WHERE message_id IN ({}) ORDER BY log_at DESC",
            placeholders
        );

        let mut query_builder = sqlx::query_as::<_, OutreachLog>(&query);
        for id in &message_ids {
            query_builder = query_builder.bind(id);
        }

        query_builder.fetch_all(&pool).await.unwrap_or_default()
    } else {
        vec![]
    };

    info!(
        "Found {} messages and {} outreach logs for lead_id: {}",
        messages.len(),
        outreach_logs.len(),
        lead_id
    );

    Ok((
        StatusCode::OK,
        Json(LeadWithDetails {
            lead,
            messages,
            outreach_logs,
        }),
    ))
}

pub async fn log_outreach(pool: &SqlitePool, message_id: i64, status: MessageStatus) {
    let now = Utc::now().to_rfc3339();
    let step = status.as_str();

    let result = sqlx::query(
        "INSERT INTO outreach_log (message_id, log_at, step) VALUES (?, ?, ?)",
    )
    .bind(message_id)
    .bind(&now)
    .bind(step)
    .execute(pool)
    .await;

    if let Err(e) = result {
        error!(
            "Failed to log outreach for message_id {}: {}",
            message_id, e
        );
    }
}
