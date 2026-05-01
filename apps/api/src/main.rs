use anyhow::Result;
use axum::Router;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod cache;
mod config;
mod error;
mod models;
mod routes;
mod sources;

use cache::Cache;
use config::Config;
use sources::SourceRegistry;

#[derive(Clone)]
pub struct AppState {
    pub sources: Arc<SourceRegistry>,
    pub cache: Arc<Cache>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "vindicate_api=info,tower_http=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();
    let config = Config::from_env()?;

    let cache = Arc::new(Cache::new(&config.redis_url).await?);
    let sources = Arc::new(SourceRegistry::build());

    let state = AppState { sources, cache };

    let app = Router::new()
        .nest("/v1", routes::router())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("VINdicate API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
