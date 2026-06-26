use anyhow::Context;
use casbin::{CoreApi, DefaultModel, Enforcer, MgmtApi};
use config::Config;
use sqlx::SqlitePool;

use rediscn::RedisAdaptor;
use sqlx_adapter::SqlxAdapter;
use tokio::sync::RwLock;

pub mod config;
pub mod rediscn;

pub struct AppState
where
    Self: Send + Sync,
{
    pub pool: SqlitePool,
    pub config: Config,
    pub redis_adaptor: RedisAdaptor,
    pub enforcer: RwLock<Enforcer>,
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
        let adapter = SqlxAdapter::new(config.database_url.clone(), 8).await?;
        let model = DefaultModel::from_file("config/model.conf").await?;
        let mut enforcer = Enforcer::new(model, adapter).await?;
        seed_policies(&mut enforcer).await?;

        Ok(Self {
            pool,
            config,
            redis_adaptor,
            enforcer: RwLock::new(enforcer),
        })
    }
}

async fn seed_policies(e: &mut Enforcer) -> anyhow::Result<()> {
    let policies = vec![
        vec!["user","/notes","GET"]
    ];

    for p in policies {
        let p: Vec<String> = p.into_iter().map(String::from).collect();
        if !e.has_policy(p.clone()) {
            e.add_policy(p).await?;
        }
    }

    Ok(())
}
