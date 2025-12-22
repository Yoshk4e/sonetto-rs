use sqlx::migrate;
use std::path::Path;
use tracing::info;

mod config;
pub mod db;
pub mod models;

pub use config::DatabaseSettings;
pub use sqlx::{Error, SqlitePool, query, query_as};

pub async fn connect_to(settings: &DatabaseSettings) -> sqlx::Result<SqlitePool> {
    ensure_database_exists(&settings.db_name)?;

    SqlitePool::connect(&settings.to_string()).await
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), migrate::MigrateError> {
    info!("Running database migrations...");
    migrate!("./migrations").run(pool).await?;
    info!("Migrations completed successfully");
    Ok(())
}

fn ensure_database_exists(db_path: &str) -> sqlx::Result<()> {
    let path = Path::new(db_path);

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| Error::Io(e))?;
        info!("Ensured database directory exists: {}", parent.display());
    }

    if !path.exists() {
        std::fs::File::create(path).map_err(|e| Error::Io(e))?;
        info!("Created new database file: {}", db_path);
    } else {
        info!("Using existing database: {}", db_path);
    }

    Ok(())
}
