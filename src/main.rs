mod db;
mod handlers;
mod models;
mod routes;
mod scheduler;

use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("sales_app=info".parse()?),
        )
        .init();

    info!("Starting Sales App");

    let database_url = "sqlite:sales_app.db?mode=rwc";
    let pool = db::init_db(database_url).await?;

    let _scheduler = scheduler::start_scheduler(pool.clone()).await?;

    let app = routes::create_router(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3010").await?;
    info!("Server listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
