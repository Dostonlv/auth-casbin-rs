use anyhow::{Context, Ok};
use config::Config;
use sqlx::SqlitePool;

use rediscn::RedisAdaptor;

pub mod rediscn;
pub mod config;

pub struct AppState {
    pub pool: SqlitePool,
    pub config: Config,
    pub redis_adaptor: RedisAdaptor,
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
        let redis_client = redis::Client::open(&*config.redis_url)?;
        let redis_conn = redis_client.get_connection()?;
        let redis_adaptor = RedisAdaptor::new(redis_conn);
        Ok(Self { pool: pool, config, redis_adaptor })
    }
}
