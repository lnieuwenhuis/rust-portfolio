pub mod auth;
pub mod config;
pub mod csrf;
pub mod db;
pub mod error;
pub mod markdown;
pub mod models;
pub mod routes;
pub mod seed;

use std::{net::SocketAddr, sync::Arc};

use axum::Router;
use tokio::net::TcpListener;
use tower_http::{compression::CompressionLayer, services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{EnvFilter, fmt};

use crate::{auth::SessionStore, config::Config, db::DbPool};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub pool: DbPool,
    pub sessions: SessionStore,
}

pub fn app(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(routes::public::router())
        .merge(routes::admin::router())
        .nest_service(
            "/static",
            ServeDir::new("static").append_index_html_on_directories(false),
        )
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

pub async fn run() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = Config::from_env()?;
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(config.rust_log.clone())),
        )
        .init();

    let pool = db::connect(&config.database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    seed::seed_if_empty(&pool).await?;

    let state = Arc::new(AppState {
        config,
        pool,
        sessions: SessionStore::new(),
    });

    let addr: SocketAddr = state.config.bind_addr()?;
    tracing::info!(%addr, "starting lars-portfolio");
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app(state))
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use sqlx::postgres::PgPoolOptions;
    use tower::ServiceExt;

    use crate::{AppState, auth::SessionStore, config::Config};

    #[tokio::test]
    async fn health_route_returns_ok() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://postgres:postgres@localhost/lars_portfolio")
            .expect("lazy pool should be created");
        let state = Arc::new(AppState {
            config: Config {
                database_url: "postgres://postgres:postgres@localhost/lars_portfolio".to_string(),
                admin_username: "lars".to_string(),
                admin_password_hash: "unused".to_string(),
                session_secret: "0123456789012345678901234567890123456789012345678901234567890123"
                    .to_string(),
                base_url: "http://localhost:3000".to_string(),
                rust_log: "info".to_string(),
                port: 3000,
                secure_cookies: false,
            },
            pool,
            sessions: SessionStore::new(),
        });

        let response = super::app(state)
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("request should complete");

        assert_eq!(response.status(), StatusCode::OK);
    }
}
