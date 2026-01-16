use chrono::{Duration, Utc};
use sqlx::SqlitePool;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info, warn};

use crate::handlers::log_outreach;
use crate::models::MessageStatus;

pub async fn start_scheduler(pool: SqlitePool) -> Result<JobScheduler, Box<dyn std::error::Error>> {
    info!("Starting scheduler");

    let sched = JobScheduler::new().await?;

    let pool_clone = pool.clone();
    let process_enqueued_job = Job::new_async("0 * * * * *", move |_uuid, _l| {
        let pool = pool_clone.clone();
        Box::pin(async move {
            process_enqueued_messages(&pool).await;
        })
    })?;

    let pool_clone = pool.clone();
    let process_ai_enqueued_job = Job::new_async("0 * * * * *", move |_uuid, _l| {
        let pool = pool_clone.clone();
        Box::pin(async move {
            process_ai_enqueued_messages(&pool).await;
        })
    })?;

    let pool_clone = pool.clone();
    let process_follow_up_job = Job::new_async("0 * * * * *", move |_uuid, _l| {
        let pool = pool_clone.clone();
        Box::pin(async move {
            process_follow_up_messages(&pool).await;
        })
    })?;

    let pool_clone = pool.clone();
    let process_closed_job = Job::new_async("0 * * * * *", move |_uuid, _l| {
        let pool = pool_clone.clone();
        Box::pin(async move {
            process_closed_messages(&pool).await;
        })
    })?;

    sched.add(process_enqueued_job).await?;
    sched.add(process_ai_enqueued_job).await?;
    sched.add(process_follow_up_job).await?;
    sched.add(process_closed_job).await?;

    sched.start().await?;

    info!("Scheduler started with cron jobs");

    Ok(sched)
}

async fn process_enqueued_messages(pool: &SqlitePool) {
    info!("Processing enqueued messages");

    let messages: Vec<(i64,)> =
        sqlx::query_as("SELECT id FROM messages WHERE status = ?")
            .bind(MessageStatus::Enqueued.as_str())
            .fetch_all(pool)
            .await
            .unwrap_or_default();

    if messages.is_empty() {
        info!("No enqueued messages to process");
        return;
    }

    info!("Found {} enqueued messages to process", messages.len());

    let now = Utc::now().to_rfc3339();
    let new_status = MessageStatus::Sent.as_str();

    for (message_id,) in messages {
        let result = sqlx::query("UPDATE messages SET status = ?, sent_at = ? WHERE id = ?")
            .bind(new_status)
            .bind(&now)
            .bind(message_id)
            .execute(pool)
            .await;

        match result {
            Ok(_) => {
                log_outreach(pool, message_id, MessageStatus::Sent).await;
                info!("Message {} status updated to sent", message_id);
            }
            Err(e) => {
                error!("Failed to update message {}: {}", message_id, e);
            }
        }
    }
}

async fn process_ai_enqueued_messages(pool: &SqlitePool) {
    info!("Processing AI enqueued messages");

    let messages: Vec<(i64,)> =
        sqlx::query_as("SELECT id FROM messages WHERE status = ?")
            .bind(MessageStatus::AiEnqueued.as_str())
            .fetch_all(pool)
            .await
            .unwrap_or_default();

    if messages.is_empty() {
        info!("No AI enqueued messages to process");
        return;
    }

    info!("Found {} AI enqueued messages to process", messages.len());

    let now = Utc::now().to_rfc3339();
    let new_status = MessageStatus::AiReplied.as_str();

    for (message_id,) in messages {
        let result = sqlx::query("UPDATE messages SET status = ?, ai_reply_sent = ? WHERE id = ?")
            .bind(new_status)
            .bind(&now)
            .bind(message_id)
            .execute(pool)
            .await;

        match result {
            Ok(_) => {
                log_outreach(pool, message_id, MessageStatus::AiReplied).await;
                info!("Message {} status updated to ai_replied", message_id);
            }
            Err(e) => {
                error!("Failed to update message {}: {}", message_id, e);
            }
        }
    }
}

async fn process_follow_up_messages(pool: &SqlitePool) {
    info!("Processing messages for follow-up (sent_at > 24h with no reply)");

    let cutoff = (Utc::now() - Duration::hours(24)).to_rfc3339();
    info!("Follow-up cutoff time: {}", cutoff);

    let messages: Vec<(i64,)> = sqlx::query_as(
        r#"
        SELECT id FROM messages
        WHERE sent_at IS NOT NULL
          AND sent_at < ?
          AND reply_received IS NULL
          AND reply_received_at IS NULL
          AND follow_up_at IS NULL
          AND closed_at IS NULL
        "#,
    )
    .bind(&cutoff)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    if messages.is_empty() {
        info!("No messages require follow-up");
        return;
    }

    info!(
        "Found {} messages that need follow-up (sent over 24h ago with no reply)",
        messages.len()
    );

    let now = Utc::now().to_rfc3339();
    let new_status = MessageStatus::FollowUp.as_str();

    for (message_id,) in messages {
        info!(
            "Processing follow-up for message_id: {}",
            message_id
        );

        let result = sqlx::query("UPDATE messages SET status = ?, follow_up_at = ? WHERE id = ?")
            .bind(new_status)
            .bind(&now)
            .bind(message_id)
            .execute(pool)
            .await;

        match result {
            Ok(_) => {
                log_outreach(pool, message_id, MessageStatus::FollowUp).await;
                info!(
                    "Message {} marked for follow-up at {}",
                    message_id, now
                );
            }
            Err(e) => {
                error!(
                    "Failed to mark message {} for follow-up: {}",
                    message_id, e
                );
            }
        }
    }

    info!("Finished processing follow-up messages");
}

async fn process_closed_messages(pool: &SqlitePool) {
    info!("Processing messages for closing (follow_up_at > 24h with no reply)");

    let cutoff = (Utc::now() - Duration::hours(24)).to_rfc3339();
    info!("Closed cutoff time: {}", cutoff);

    let messages: Vec<(i64,)> = sqlx::query_as(
        r#"
        SELECT id FROM messages
        WHERE follow_up_at IS NOT NULL
          AND follow_up_at < ?
          AND reply_received IS NULL
          AND reply_received_at IS NULL
          AND closed_at IS NULL
        "#,
    )
    .bind(&cutoff)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    if messages.is_empty() {
        info!("No messages require closing");
        return;
    }

    warn!(
        "Found {} messages to close (follow-up sent over 24h ago with no reply)",
        messages.len()
    );

    let now = Utc::now().to_rfc3339();
    let new_status = MessageStatus::Closed.as_str();

    for (message_id,) in messages {
        info!("Processing closure for message_id: {}", message_id);

        let result = sqlx::query("UPDATE messages SET status = ?, closed_at = ? WHERE id = ?")
            .bind(new_status)
            .bind(&now)
            .bind(message_id)
            .execute(pool)
            .await;

        match result {
            Ok(_) => {
                log_outreach(pool, message_id, MessageStatus::Closed).await;
                warn!("Message {} closed at {} (no response after follow-up)", message_id, now);
            }
            Err(e) => {
                error!("Failed to close message {}: {}", message_id, e);
            }
        }
    }

    info!("Finished processing closed messages");
}
