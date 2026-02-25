mod config;
mod db;
mod error;
mod handlers;
mod middleware;
mod models;
mod state;

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

use config::Config;
use handlers::{get_orders, get_portfolio, get_stocks, login, oauth_login, place_order, register};
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = Config::from_env()?;
    let pool = db::create_pool(&config).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let state = AppState {
        pool,
        config: config.clone(),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/auth/oauth", post(oauth_login))
        .route("/api/stocks", get(get_stocks))
        .route("/api/portfolio", get(get_portfolio))
        .route("/api/orders", get(get_orders))
        .route("/api/orders", post(place_order))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server listening on http://{}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;

    Ok(())
}

async fn health() -> &'static str {
    "OK"
}
