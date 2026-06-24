use anyhow::Context;

pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_issuer: String,
    pub redis_url: String,
    pub jwt_expires_time: String,
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
            jwt_issuer: std::env::var("JWT_ISSUER")
                .unwrap_or_else(|_| "https//issuer.kisuke.uz".to_string()),
            jwt_expires_time: std::env::var("JWT_EXPIRES_TIME")
                .context("JWT_EXPIRES_TIME -> environment variable not found")?,
        })
    }
}
