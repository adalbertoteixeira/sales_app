use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Sqlite, SqlitePool};
use tracing::info;

pub async fn init_db(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    info!("Connecting to database: {}", database_url);

    if !Sqlite::database_exists(database_url).await.unwrap_or(false) {
        info!("Creating database: {}", database_url);
        Sqlite::create_database(database_url).await?;
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    info!("Running migrations");
    sqlx::migrate!("./migrations").run(&pool).await?;
    info!("Migrations completed successfully");

    Ok(pool)
}
