use anyhow::Context;

pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub redis_url: String,
}

impl Config {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .context("DATABASE_URL -> environment variable not found")?,
            jwt_secret: std::env::var("JWT_SECRET")
                .context("JWT_SECRET -> environment variable not found")?,
            redis_url: std::env::var("REDIS_URL")
                .context("REDIS_URL -> environment variable not found")?,
        })
    }
}
