use anyhow::{Context, Ok};
use config::Config;
use sqlx::SqlitePool;

pub mod config;

pub struct AppState {
    pub pool: SqlitePool,
    pub config: Config,
}

impl AppState {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let pool = SqlitePool::connect(&config.database_url)
            .await
            .context("failed on creating database pooling")?;
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .context("migration failed")?;
        Ok(Self { pool: pool, config })
    }
}
