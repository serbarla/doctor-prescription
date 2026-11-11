use anyhow::Result;
use axum::http::header;
use axum::routing::{get, post};
use axum::Router;
use dioxus::server::{DioxusRouterExt, ServeConfig};

use crate::auth_server::{self, AppState};
use crate::db;

async fn health() -> ([(header::HeaderName, &'static str); 1], &'static str) {
    (
        [(header::CONTENT_TYPE, "application/json")],
        "{\"ok\":true}",
    )
}

pub async fn router() -> Result<Router> {
    let cookie_secure = std::env::var("PERSONA_COOKIE_SECURE")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let state = AppState {
        pool: db::connect()?,
        cookie_secure,
    };

    let auth_router = Router::new()
        .route("/api/auth/sign-up/email", post(auth_server::sign_up))
        .route("/api/auth/sign-in/email", post(auth_server::sign_in))
        .route("/api/auth/get-session", get(auth_server::get_session))
        .route("/api/auth/sign-out", post(auth_server::sign_out))
        .with_state(state);

    let app_router = Router::new()
        .route("/health", get(health))
        .merge(auth_router);

    Ok(Router::new()
        .serve_dioxus_application(ServeConfig::new(), crate::App)
        .merge(app_router))
}
