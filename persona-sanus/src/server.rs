use anyhow::Result;
use axum::extract::State;
use axum::http::{header, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use diesel::prelude::*;
use diesel::sql_types::{Integer, Text};
use dioxus::server::{DioxusRouterExt, ServeConfig};
use serde_json::json;
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::api_server;
use crate::assets;
use crate::auth_server::{self, AppState};
use crate::db;
use crate::pdf_server;
use crate::verify_server;

#[derive(QueryableByName)]
struct ForeignKeysPragma {
    #[diesel(sql_type = Integer)]
    foreign_keys: i32,
}

#[derive(QueryableByName)]
struct JournalModePragma {
    #[diesel(sql_type = Text)]
    journal_mode: String,
}

async fn health(State(state): State<AppState>) -> Response {
    let Ok(mut connection) = state.pool.get() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "ok": false })),
        )
            .into_response();
    };
    let foreign_keys =
        diesel::sql_query("PRAGMA foreign_keys").get_result::<ForeignKeysPragma>(&mut connection);
    let journal_mode =
        diesel::sql_query("PRAGMA journal_mode").get_result::<JournalModePragma>(&mut connection);
    match (foreign_keys, journal_mode) {
        (Ok(foreign_keys), Ok(journal_mode)) => Json(json!({
            "ok": true,
            "foreign_keys": foreign_keys.foreign_keys == 1,
            "journal_mode": journal_mode.journal_mode,
        }))
        .into_response(),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false })),
        )
            .into_response(),
    }
}

fn cors_layer() -> Option<CorsLayer> {
    let origins: Vec<HeaderValue> = std::env::var("PERSONA_WEB_ORIGINS")
        .ok()?
        .split(',')
        .filter_map(|origin| HeaderValue::from_str(origin.trim()).ok())
        .collect();
    if origins.is_empty() {
        return None;
    }
    Some(
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(origins))
            .allow_credentials(true)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([header::CONTENT_TYPE, header::COOKIE]),
    )
}

pub async fn router() -> Result<Router> {
    let cookie_secure = std::env::var("PERSONA_COOKIE_SECURE")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let storage = assets::AssetStorage::from_env();
    if let Some(storage) = storage.as_ref() {
        storage.initialize().await;
    }
    let state = AppState {
        pool: db::connect()?,
        cookie_secure,
        assets: storage.map(std::sync::Arc::new),
    };

    let auth_router = Router::new()
        .route("/api/auth/sign-up/email", post(auth_server::sign_up))
        .route("/api/auth/sign-in/email", post(auth_server::sign_in))
        .route("/api/auth/get-session", get(auth_server::get_session))
        .route("/api/auth/sign-out", post(auth_server::sign_out))
        .with_state(state.clone());

    let app_router = Router::new()
        .route("/health", get(health))
        .with_state(state.clone())
        .merge(auth_router)
        .merge(api_server::router(state.clone()))
        .merge(pdf_server::router(state.clone()))
        .merge(verify_server::router(state));
    let app_router = match cors_layer() {
        Some(cors) => app_router.layer(cors),
        None => app_router,
    };

    let limiter = std::sync::Arc::new(verify_server::VerifyRateLimiter::new());
    Ok(verify_server::layer(
        Router::new()
            .serve_dioxus_application(ServeConfig::new(), crate::App)
            .merge(app_router),
        limiter,
    ))
}
