use axum::{
    routing::{get, post},
    Router,
};
use sqlx::SqlitePool;

use crate::handlers::{ai_reply, create_lead, get_lead, reply_to_message, send_message};

pub fn create_router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/lead", post(create_lead))
        .route("/lead/{id}", get(get_lead))
        .route("/send", post(send_message))
        .route("/reply", post(reply_to_message))
        .route("/ai/reply", post(ai_reply))
        .with_state(pool)
}
