use std::sync::Arc;

use anyhow::Result;
use axum::body::Body;
use axum::extract::State;
use axum::http::{header, HeaderValue, Method, Request, StatusCode, Uri};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use diesel::prelude::*;
use diesel::sql_types::{Integer, Text};
use dioxus::server::{DioxusRouterExt, ServeConfig};
use serde_json::json;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::set_header::SetResponseHeaderLayer;

use crate::api_server;
use crate::assets;
use crate::auth_server::{self, AppState};
use crate::db;
use crate::jobs;
use crate::pdf_server;
use crate::sync_server;
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

fn web_origins() -> Vec<String> {
    std::env::var("PERSONA_WEB_ORIGINS")
        .unwrap_or_default()
        .split(',')
        .map(|origin| origin.trim().to_owned())
        .filter(|origin| !origin.is_empty())
        .collect()
}

fn cors_layer() -> Option<CorsLayer> {
    let origins: Vec<HeaderValue> = web_origins()
        .iter()
        .filter_map(|origin| HeaderValue::from_str(origin).ok())
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

fn same_origin(request: &Request<Body>) -> Option<String> {
    let host = request
        .headers()
        .get("x-forwarded-host")
        .or_else(|| request.headers().get(header::HOST))?
        .to_str()
        .ok()?
        .split(',')
        .next()?
        .trim();
    let scheme = request
        .headers()
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("http");
    Some(format!("{scheme}://{host}"))
}

fn loopback_origin(origin: &str) -> bool {
    let Ok(uri) = origin.parse::<Uri>() else {
        return false;
    };
    if !matches!(uri.scheme_str(), Some("http" | "https")) {
        return false;
    }
    let host = uri.host().unwrap_or_default().trim_matches(['[', ']']);
    host.eq_ignore_ascii_case("localhost")
        || host.ends_with(".localhost")
        || matches!(host, "127.0.0.1" | "::1" | "0.0.0.0")
}

fn dev_proxy_origin(request: &Request<Body>, origin: &str) -> bool {
    if request.headers().get("x-proxied-by-dioxus").is_none() || !loopback_origin(origin) {
        return false;
    }
    match std::env::var("DIOXUS_DEVSERVER_PORT")
        .ok()
        .and_then(|port| port.parse::<u16>().ok())
    {
        Some(port) => origin
            .parse::<Uri>()
            .ok()
            .and_then(|uri| uri.port_u16())
            .is_some_and(|origin_port| origin_port == port),
        None => false,
    }
}

async fn origin_guard(request: Request<Body>, next: Next) -> Response {
    if matches!(
        *request.method(),
        Method::POST | Method::PUT | Method::DELETE | Method::PATCH
    ) {
        if let Some(origin) = request.headers().get(header::ORIGIN) {
            if let Ok(origin) = origin.to_str() {
                let allowed = web_origins().iter().any(|item| item == origin)
                    || same_origin(&request).as_deref() == Some(origin)
                    || dev_proxy_origin(&request, origin);
                if !allowed {
                    return (
                        StatusCode::FORBIDDEN,
                        Json(json!({
                            "code": "origin_rejected",
                            "message": "Origin no permitido.",
                        })),
                    )
                        .into_response();
                }
            }
        }
    }
    next.run(request).await
}

fn security_headers(router: Router, cookie_secure: bool) -> Router {
    let csp = "default-src 'self'; script-src 'self' 'wasm-unsafe-eval' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; font-src 'self' data:; connect-src 'self'; object-src 'none'; base-uri 'self'; frame-ancestors 'self'";
    let router = router.layer(SetResponseHeaderLayer::overriding(
        header::HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static(csp),
    ));
    let router = router.layer(SetResponseHeaderLayer::overriding(
        header::HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    ));
    let router = router.layer(SetResponseHeaderLayer::overriding(
        header::HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("same-origin"),
    ));
    let router = router.layer(SetResponseHeaderLayer::overriding(
        header::HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("SAMEORIGIN"),
    ));
    if cookie_secure {
        router.layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("strict-transport-security"),
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ))
    } else {
        router
    }
}

fn url_host(value: &str) -> Option<String> {
    value
        .parse::<Uri>()
        .ok()
        .and_then(|uri| uri.host().map(|host| host.trim_matches(['[', ']']).to_owned()))
}

fn url_scheme(value: &str) -> Option<String> {
    value
        .parse::<Uri>()
        .ok()
        .and_then(|uri| uri.scheme_str().map(str::to_owned))
}

fn is_loopback_host(host: &str) -> bool {
    host.eq_ignore_ascii_case("localhost")
        || host.ends_with(".localhost")
        || matches!(host, "127.0.0.1" | "::1" | "0.0.0.0")
}

fn env_non_empty(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().trim_end_matches('/').to_owned())
        .filter(|value| !value.is_empty())
}

fn production_config_error() -> Option<String> {
    if cfg!(debug_assertions) {
        return None;
    }
    let production = std::env::var("PERSONA_ENV")
        .map(|value| value.eq_ignore_ascii_case("production"))
        .unwrap_or(false);
    let cookie_secure = std::env::var("PERSONA_COOKIE_SECURE")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let public_base = env_non_empty("PUBLIC_BASE_URL");
    let assets_public = env_non_empty("ASSETS_PUBLIC_ENDPOINT");
    if production {
        let Some(base) = public_base.as_deref() else {
            return Some("PERSONA_ENV=production requires PUBLIC_BASE_URL to be set".to_owned());
        };
        let Some(host) = url_host(base) else {
            return Some(format!("PUBLIC_BASE_URL {base} is not a valid URL"));
        };
        if is_loopback_host(&host) {
            return Some(format!(
                "PERSONA_ENV=production requires PUBLIC_BASE_URL to be a public host, got {base}"
            ));
        }
        if url_scheme(base).as_deref() != Some("https") {
            return Some(format!(
                "PERSONA_ENV=production requires PUBLIC_BASE_URL to use https, got {base}"
            ));
        }
        if !cookie_secure {
            return Some(
                "PERSONA_ENV=production requires PERSONA_COOKIE_SECURE=true".to_owned(),
            );
        }
        if let Some(assets) = assets_public.as_deref() {
            if let Some(assets_host) = url_host(assets) {
                if is_loopback_host(&assets_host) {
                    return Some(format!(
                        "ASSETS_PUBLIC_ENDPOINT {assets} is loopback while the app is public"
                    ));
                }
            }
        }
        return None;
    }
    if let Some(base) = public_base.as_deref() {
        if let Some(host) = url_host(base) {
            if !is_loopback_host(&host) {
                if !cookie_secure {
                    return Some(
                        "PUBLIC_BASE_URL is public but PERSONA_COOKIE_SECURE is not true".to_owned(),
                    );
                }
                if let Some(assets) = assets_public.as_deref() {
                    if let Some(assets_host) = url_host(assets) {
                        if is_loopback_host(&assets_host) {
                            return Some(format!(
                                "ASSETS_PUBLIC_ENDPOINT {assets} is loopback while PUBLIC_BASE_URL {base} is public"
                            ));
                        }
                    }
                }
            }
        }
    }
    None
}

pub async fn router() -> Result<Router> {
    if let Some(error) = production_config_error() {
        return Err(anyhow::anyhow!(error));
    }
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
    jobs::spawn(state.clone());

    let auth_router = Router::new()
        .route("/api/auth/sign-up/email", post(auth_server::sign_up))
        .route("/api/auth/sign-in/email", post(auth_server::sign_in))
        .route("/api/auth/get-session", get(auth_server::get_session))
        .route("/api/auth/sign-out", post(auth_server::sign_out))
        .route("/api/auth/mfa/verify", post(auth_server::mfa_verify))
        .route("/api/auth/mfa/enroll", post(auth_server::mfa_enroll))
        .route("/api/auth/mfa/confirm", post(auth_server::mfa_confirm))
        .route("/api/auth/mfa/disable", post(auth_server::mfa_disable))
        .route("/api/auth/mfa/status", get(auth_server::mfa_status))
        .route(
            "/api/auth/change-password",
            post(auth_server::change_password),
        )
        .with_state(state.clone());

    let app_router = Router::new()
        .route("/health", get(health))
        .with_state(state.clone())
        .merge(auth_router)
        .merge(api_server::router(state.clone()))
        .merge(jobs::router(state.clone()))
        .merge(pdf_server::router(state.clone()))
        .merge(sync_server::router(state.clone()))
        .merge(verify_server::router(state));
    #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
    let app_router = {
        println!(
            "dev sqlite browser mounted at /dev/db for {}",
            db::database_path()
        );
        app_router.merge(crate::dev_db::router())
    };
    let app_router = match cors_layer() {
        Some(cors) => app_router.layer(cors),
        None => app_router,
    };
    let app_router = app_router.layer(middleware::from_fn(origin_guard));
    let auth_limiter = Arc::new(auth_server::AuthRateLimiter::new());
    let app_router = app_router.layer(middleware::from_fn_with_state(
        auth_limiter,
        auth_server::auth_rate_limit,
    ));

    let limiter = std::sync::Arc::new(verify_server::VerifyRateLimiter::new());
    let router = Router::new()
        .serve_dioxus_application(ServeConfig::new(), crate::App)
        .merge(app_router);
    let router = security_headers(router, cookie_secure);
    Ok(verify_server::layer(router, limiter))
}
