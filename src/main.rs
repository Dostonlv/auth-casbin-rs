use std::sync::Arc;

use notes::{AppState, config::Config};

mod db;
mod entities;

mod routes;
use routes::create_app;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let config = Config::new()?;

    let shared_state = Arc::new(AppState::new(config).await?);

    let app = create_app(shared_state).await?;

    let address = "127.0.0.1:6769";
    println!("Running server at http://{address}");

    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
