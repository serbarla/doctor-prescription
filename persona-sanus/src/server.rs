use std::env;
use std::str::FromStr;

use anyhow::Result;
use axum::http::header;
use axum::routing::get;
use axum::Router;
use dioxus::server::{DioxusRouterExt, ServeConfig};
use rustauth::prelude::*;
use rustauth::sqlx::SqliteAdapter;
use rustauth_axum::{RustAuthAxumExt, RustAuthAxumOptions};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

async fn health() -> ([(header::HeaderName, &'static str); 1], &'static str) {
    (
        [(header::CONTENT_TYPE, "application/json")],
        "{\"ok\":true}",
    )
}

pub async fn router() -> Result<Router> {
    let secret = env::var("RUSTAUTH_SECRET")
        .unwrap_or_else(|_| "dev-only-secret-at-least-32-chars-long!!".to_owned());
    let base_url = env::var("RUSTAUTH_BASE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8080/api/auth".to_owned());
    let base_path = env::var("RUSTAUTH_AUTH_BASE_PATH").unwrap_or_else(|_| "/api/auth".to_owned());
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://app.db".to_owned());

    let connect_options = SqliteConnectOptions::from_str(&database_url)?.create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .connect_with(connect_options)
        .await?;

    let auth = RustAuth::builder()
        .options(
            RustAuthOptions::new().trusted_origins(TrustedOriginOptions::Static(vec![
                "http://127.0.0.1:8080".to_owned(),
                "http://localhost:8080".to_owned(),
            ])),
        )
        .secret(secret)
        .base_url(base_url)
        .base_path(base_path)
        .email_password(
            EmailPasswordOptions::new()
                .enabled(true)
                .auto_sign_in(true)
                .require_email_verification(false),
        )
        .rate_limit(RateLimitOptions::memory().enabled(true))
        .adapter(SqliteAdapter::new(pool))
        .build()
        .await?;

    auth.create_schema(None).await?;

    let auth_router = Router::new()
        .route("/health", get(health))
        .merge(auth.mount_at_base_path(RustAuthAxumOptions::new())?);

    Ok(Router::new()
        .serve_dioxus_application(ServeConfig::new(), crate::App)
        .merge(auth_router))
}
