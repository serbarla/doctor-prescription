use std::collections::{HashMap, VecDeque};
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::extract::{ConnectInfo, Path, State};
use axum::http::{HeaderMap, HeaderName, Request, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;

use crate::auth_server::{AppState, AuthError};
use crate::service;

pub struct VerifyRateLimiter {
    limit: u32,
    windows: Mutex<HashMap<IpAddr, VecDeque<Instant>>>,
}

impl VerifyRateLimiter {
    pub fn new() -> Self {
        let limit = std::env::var("VERIFY_RATE_LIMIT_PER_MINUTE")
            .ok()
            .and_then(|value| value.trim().parse::<u32>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(10);
        Self {
            limit,
            windows: Mutex::new(HashMap::new()),
        }
    }

    fn allowed(&self, ip: IpAddr) -> bool {
        let now = Instant::now();
        let cutoff = now - Duration::from_secs(60);
        let mut windows = self
            .windows
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let entry = windows.entry(ip).or_default();
        while entry.front().is_some_and(|moment| *moment <= cutoff) {
            entry.pop_front();
        }
        if entry.len() >= self.limit as usize {
            return false;
        }
        entry.push_back(now);
        true
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/verify/{code}", get(verify))
        .with_state(state)
}

pub async fn verify(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Response, AuthError> {
    let result = service::verify_code(&state.pool, &code).await?;
    Ok(Json(json!({
        "status": result.status,
        "doctorName": result.doctor_name,
        "issuedAt": result.issued_at,
        "rxNumber": result.rx_number,
    }))
    .into_response())
}

pub async fn rate_limit(
    State(limiter): State<Arc<VerifyRateLimiter>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let path = request.uri().path();
    if !path.starts_with("/verify/") && !path.starts_with("/api/verify/") {
        return next.run(request).await;
    }
    let ip = client_ip(&request);
    if limiter.allowed(ip) {
        return next.run(request).await;
    }
    let mut response = Json(json!({
        "code": "rate_limited",
        "message": "Demasiadas solicitudes. Inténtalo de nuevo en un minuto.",
    }))
    .into_response();
    *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
    response
}

fn client_ip(request: &Request<Body>) -> IpAddr {
    if let Some(connect) = request.extensions().get::<ConnectInfo<SocketAddr>>() {
        return connect.0.ip();
    }
    forwarded_ip(request.headers()).unwrap_or(IpAddr::from([0, 0, 0, 0]))
}

fn forwarded_ip(headers: &HeaderMap) -> Option<IpAddr> {
    headers
        .get(HeaderName::from_static("x-forwarded-for"))?
        .to_str()
        .ok()?
        .split(',')
        .next()?
        .trim()
        .parse()
        .ok()
}

pub fn layer(router: Router, limiter: Arc<VerifyRateLimiter>) -> Router {
    router.layer(middleware::from_fn_with_state(limiter, rate_limit))
}
