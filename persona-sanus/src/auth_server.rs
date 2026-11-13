use std::collections::{HashMap, VecDeque};
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

use argon2::password_hash::{phc::PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::Argon2;
use axum::body::Body;
use axum::extract::{ConnectInfo, State};
use axum::http::{header, HeaderMap, HeaderName, HeaderValue, Method, Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::{Duration as ChronoDuration, SecondsFormat, Utc};
use diesel::prelude::*;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::assets::AssetStorage;
use crate::db::{self, DbPool, NewSession};
use crate::service::{self, RegisterUser, Role};

pub(crate) const SESSION_COOKIE: &str = "persona_session";
const SESSION_DAYS: i64 = 30;
const MFA_TICKET_MINUTES: i64 = 5;
const TOTP_PERIOD_SECS: i64 = 30;

static DUMMY_HASH: LazyLock<String> = LazyLock::new(|| {
    Argon2::default()
        .hash_password(b"persona-sanus-dummy-credential")
        .map(|hash| hash.to_string())
        .unwrap_or_default()
});

#[derive(Debug, Clone)]
pub struct AppState {
    pub pool: DbPool,
    pub cookie_secure: bool,
    pub assets: Option<Arc<AssetStorage>>,
}

#[derive(Debug, Deserialize)]
pub struct SignUpRequest {
    name: String,
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
pub struct SignInRequest {
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
pub struct MfaVerifyRequest {
    ticket: String,
    code: String,
}

#[derive(Debug, Deserialize)]
pub struct MfaCodeRequest {
    code: String,
}

#[derive(Debug, Deserialize)]
pub struct MfaDisableRequest {
    password: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    #[serde(alias = "currentPassword")]
    current_password: String,
    #[serde(alias = "newPassword")]
    new_password: String,
}

#[derive(Debug)]
pub enum AuthError {
    BadRequest(String),
    Conflict(String),
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    Unprocessable(String),
    DuplicateNationalId,
    ImageQuotaExceeded,
    AssetInUse,
    TooLarge(String),
    UnsupportedMedia(String),
    StorageUnavailable(String),
    UnacknowledgedWarnings(Vec<String>),
    RateLimited,
    Internal,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, json!({ "message": message })),
            Self::Conflict(message) => (StatusCode::CONFLICT, json!({ "message": message })),
            Self::Unauthorized(message) => {
                (StatusCode::UNAUTHORIZED, json!({ "message": message }))
            }
            Self::Forbidden(message) => (StatusCode::FORBIDDEN, json!({ "message": message })),
            Self::NotFound(message) => (StatusCode::NOT_FOUND, json!({ "message": message })),
            Self::Unprocessable(message) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                json!({ "message": message }),
            ),
            Self::DuplicateNationalId => (
                StatusCode::CONFLICT,
                json!({
                    "code": "duplicate_national_id",
                    "message": "Ya existe un paciente con esta cédula.",
                }),
            ),
            Self::ImageQuotaExceeded => (
                StatusCode::CONFLICT,
                json!({
                    "code": "image_quota_exceeded",
                    "message": "La biblioteca de la clínica ya tiene su imagen. Elimínala antes de subir otra.",
                }),
            ),
            Self::AssetInUse => (
                StatusCode::CONFLICT,
                json!({
                    "code": "asset_in_use",
                    "message": "La imagen está fijada por una versión publicada o una receta retenida.",
                }),
            ),
            Self::TooLarge(message) => (
                StatusCode::PAYLOAD_TOO_LARGE,
                json!({ "code": "image_too_large", "message": message }),
            ),
            Self::UnsupportedMedia(message) => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                json!({ "code": "unsupported_media_type", "message": message }),
            ),
            Self::StorageUnavailable(message) => (
                StatusCode::SERVICE_UNAVAILABLE,
                json!({ "code": "storage_unavailable", "message": message }),
            ),
            Self::UnacknowledgedWarnings(keys) => (
                StatusCode::CONFLICT,
                json!({
                    "code": "unacknowledged_warnings",
                    "message": "La plantilla tiene variables desconocidas.",
                    "unknownKeys": keys,
                }),
            ),
            Self::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                json!({
                    "code": "rate_limited",
                    "message": "Demasiadas solicitudes. Inténtalo de nuevo en un minuto.",
                }),
            ),
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "message": "Internal server error" }),
            ),
        };
        (status, Json(body)).into_response()
    }
}

impl From<diesel::result::Error> for AuthError {
    fn from(error: diesel::result::Error) -> Self {
        match error {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) => Self::Conflict("An account with this email already exists.".to_owned()),
            _ => Self::Internal,
        }
    }
}

impl From<diesel::r2d2::PoolError> for AuthError {
    fn from(_: diesel::r2d2::PoolError) -> Self {
        Self::Internal
    }
}

pub struct AuthRateLimiter {
    limit: u32,
    windows: Mutex<HashMap<IpAddr, VecDeque<Instant>>>,
}

impl AuthRateLimiter {
    pub fn new() -> Self {
        let limit = std::env::var("AUTH_RATE_LIMIT_PER_MINUTE")
            .ok()
            .and_then(|value| value.trim().parse::<u32>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(20);
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

pub async fn auth_rate_limit(
    State(limiter): State<Arc<AuthRateLimiter>>,
    ConnectInfo(connect): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if !request.uri().path().starts_with("/api/auth/") || request.method() != Method::POST {
        return next.run(request).await;
    }
    let ip = forwarded_ip(request.headers())
        .filter(|_| connect.ip() == IpAddr::from([127, 0, 0, 1]) || connect.ip().is_loopback())
        .unwrap_or(connect.ip());
    if limiter.allowed(ip) {
        return next.run(request).await;
    }
    AuthError::RateLimited.into_response()
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

async fn audit_auth(
    pool: &DbPool,
    actor_id: Option<String>,
    action: &str,
    entity_id: &str,
    metadata: Value,
) {
    let pool = pool.clone();
    let action = action.to_owned();
    let entity_id = entity_id.to_owned();
    let at = now();
    let _ = with_conn(&pool, move |connection| {
        service::record_audit(
            connection,
            actor_id.as_deref(),
            &action,
            "auth",
            &entity_id,
            Some(metadata),
            &at,
        )
    })
    .await;
}

async fn actor_profile_id(pool: &DbPool, user_id: &str) -> Option<String> {
    let user_id = user_id.to_owned();
    with_conn(pool, move |connection| {
        Ok(db::find_profile_by_auth_user_id(connection, &user_id)?.map(|profile| profile.id))
    })
    .await
    .ok()
    .flatten()
}

pub async fn sign_up(
    State(state): State<AppState>,
    Json(request): Json<SignUpRequest>,
) -> Result<Response, AuthError> {
    let email = request.email.trim().to_lowercase();
    let name = request.name.trim().to_owned();

    if email.len() < 3 || !email.contains('@') {
        return Err(AuthError::BadRequest(
            "Enter a valid email address.".to_owned(),
        ));
    }
    if name.is_empty() {
        return Err(AuthError::BadRequest("Enter your name.".to_owned()));
    }
    if request.password.chars().count() < 8 {
        return Err(AuthError::BadRequest(
            "Password must be at least 8 characters.".to_owned(),
        ));
    }

    let password_hash = hash_password_async(request.password).await?;
    let user_id = Uuid::now_v7().to_string();
    let session_id = Uuid::now_v7().to_string();
    let token = new_token();
    let token_hash = hash_token(&token);
    let created_at = now();
    let expires_at = session_expiry();

    let created_user = service::register_user(
        &state.pool,
        RegisterUser {
            user_id,
            profile_id: Uuid::now_v7().to_string(),
            session_id,
            email: email.clone(),
            display_name: name,
            password_hash,
            token_hash,
            created_at,
            expires_at,
        },
    )
    .await?;

    audit_auth(
        &state.pool,
        actor_profile_id(&state.pool, &created_user.id).await,
        "sign_up",
        &created_user.id,
        json!({ "email": email }),
    )
    .await;

    Ok(session_response(&token, &created_user, &state))
}

pub async fn sign_in(
    State(state): State<AppState>,
    Json(request): Json<SignInRequest>,
) -> Result<Response, AuthError> {
    let email = request.email.trim().to_lowercase();
    let email_lookup = email.clone();
    let user = with_conn(&state.pool, move |connection| {
        db::find_user_by_email(connection, &email_lookup).map_err(AuthError::from)
    })
    .await?;

    let Some(user) = user else {
        let _ = tokio::task::spawn_blocking(|| {
            verify_password(&DUMMY_HASH, "persona-sanus-dummy-attempt")
        })
        .await;
        audit_auth(
            &state.pool,
            None,
            "sign_in_failed",
            &email,
            json!({ "email": email }),
        )
        .await;
        return Err(AuthError::Unauthorized(
            "Invalid email or password.".to_owned(),
        ));
    };

    let password_hash = user.password_hash.clone();
    let password = request.password;
    let valid = tokio::task::spawn_blocking(move || verify_password(&password_hash, &password))
        .await
        .map_err(|_| AuthError::Internal)?;
    if !valid {
        audit_auth(
            &state.pool,
            actor_profile_id(&state.pool, &user.id).await,
            "sign_in_failed",
            &user.id,
            json!({ "email": user.email.clone() }),
        )
        .await;
        return Err(AuthError::Unauthorized(
            "Invalid email or password.".to_owned(),
        ));
    }

    let user_id = user.id.clone();
    let (role_needs_mfa, mfa_enabled) = with_conn(&state.pool, move |connection| {
        let role = db::find_profile_by_auth_user_id(connection, &user_id)?.and_then(|profile| {
            db::find_membership_by_profile(connection, &profile.id)
                .ok()
                .flatten()
                .and_then(|member| Role::parse(&member.role))
        });
        let enabled = db::find_user_mfa(connection, &user_id)?.is_some_and(|mfa| mfa.enabled == 1);
        Ok::<_, AuthError>((role, enabled))
    })
    .await?;
    let needs_mfa =
        matches!(role_needs_mfa, Some(Role::Administrator | Role::Doctor)) && mfa_enabled;

    let session_id = Uuid::now_v7().to_string();
    let token = new_token();
    let token_hash = hash_token(&token);
    let created_at = now();
    let expires_at = if needs_mfa {
        (Utc::now() + ChronoDuration::minutes(MFA_TICKET_MINUTES))
            .to_rfc3339_opts(SecondsFormat::Secs, true)
    } else {
        session_expiry()
    };
    let user_id = user.id.clone();
    let pending: i32 = i32::from(needs_mfa);

    with_conn(&state.pool, move |connection| {
        db::insert_session(
            connection,
            &NewSession {
                id: &session_id,
                token_hash: &token_hash,
                user_id: &user_id,
                expires_at: &expires_at,
                created_at: &created_at,
                device_id: None,
                mfa_pending: pending,
            },
        )?;
        Ok(())
    })
    .await?;

    if needs_mfa {
        audit_auth(
            &state.pool,
            actor_profile_id(&state.pool, &user.id).await,
            "mfa_challenge",
            &user.id,
            json!({ "email": user.email.clone() }),
        )
        .await;
        return Ok(Json(json!({
            "mfaRequired": true,
            "mfaTicket": token,
            "user": user_json(&user),
        }))
        .into_response());
    }

    audit_auth(
        &state.pool,
        actor_profile_id(&state.pool, &user.id).await,
        "sign_in",
        &user.id,
        json!({ "email": user.email.clone() }),
    )
    .await;

    Ok(session_response(&token, &user, &state))
}

pub async fn mfa_verify(
    State(state): State<AppState>,
    Json(request): Json<MfaVerifyRequest>,
) -> Result<Response, AuthError> {
    let ticket = request.ticket.trim().to_owned();
    let code = request.code.trim().to_owned();
    if ticket.is_empty() || code.is_empty() {
        return Err(AuthError::BadRequest("Enter the MFA code.".to_owned()));
    }
    let token_hash = hash_token(&ticket);
    let at = now();
    let session = with_conn(&state.pool, move |connection| {
        db::delete_expired_sessions(connection, &at)?;
        Ok(db::find_session_by_token_hash(connection, &token_hash)?)
    })
    .await?;
    let Some(session) = session else {
        return Err(AuthError::Unauthorized("Invalid MFA code.".to_owned()));
    };
    if session.mfa_pending == 0 {
        return Err(AuthError::Unauthorized("Invalid MFA code.".to_owned()));
    }
    let user_id = session.user_id.clone();
    let user_id_lookup = user_id.clone();
    let secret = with_conn(&state.pool, move |connection| {
        Ok(db::find_user_mfa(connection, &user_id_lookup)?
            .filter(|mfa| mfa.enabled == 1)
            .map(|mfa| mfa.secret))
    })
    .await?;
    let Some(secret) = secret else {
        return Err(AuthError::Unauthorized("Invalid MFA code.".to_owned()));
    };
    if !totp_verify(&secret, &code) {
        audit_auth(
            &state.pool,
            actor_profile_id(&state.pool, &user_id).await,
            "mfa_verify_failed",
            &user_id,
            json!({}),
        )
        .await;
        return Err(AuthError::Unauthorized("Invalid MFA code.".to_owned()));
    }
    let session_id = session.id.clone();
    let expires_at = session_expiry();
    with_conn(&state.pool, move |connection| {
        db::promote_session(connection, &session_id, &expires_at)?;
        Ok(())
    })
    .await?;
    let user = with_conn(&state.pool, move |connection| {
        db::find_user_by_id(connection, &user_id).map_err(AuthError::from)
    })
    .await?
    .ok_or(AuthError::Internal)?;
    audit_auth(
        &state.pool,
        actor_profile_id(&state.pool, &user.id).await,
        "mfa_verify",
        &user.id,
        json!({ "email": user.email.clone() }),
    )
    .await;
    Ok(session_response(&ticket, &user, &state))
}

async fn require_mfa_actor(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<service::ActorContext, AuthError> {
    service::resolve_actor(&state.pool, headers)
        .await?
        .ok_or_else(|| AuthError::Unauthorized("Sign in required.".to_owned()))
}

fn require_mfa_role(actor: &service::ActorContext) -> Result<(), AuthError> {
    match actor.role() {
        Some(Role::Administrator | Role::Doctor) => Ok(()),
        _ => Err(AuthError::Forbidden(
            "MFA is available for Doctor and Administrator roles.".to_owned(),
        )),
    }
}

pub async fn mfa_enroll(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, AuthError> {
    let actor = require_mfa_actor(&state, &headers).await?;
    require_mfa_role(&actor)?;
    let secret = generate_secret();
    let secret_store = secret.clone();
    let at = now();
    let user_id = actor.user.id.clone();
    with_conn(&state.pool, move |connection| {
        db::upsert_user_mfa(connection, &user_id, &secret_store, 0, &at)?;
        Ok(())
    })
    .await?;
    audit_auth(
        &state.pool,
        Some(actor.profile.id.clone()),
        "mfa_enroll",
        &actor.user.id,
        json!({}),
    )
    .await;
    Ok(Json(json!({
        "secret": secret,
        "uri": otpauth_uri(&actor.user.email, &secret),
        "enabled": false,
    }))
    .into_response())
}

pub async fn mfa_confirm(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<MfaCodeRequest>,
) -> Result<Response, AuthError> {
    let actor = require_mfa_actor(&state, &headers).await?;
    require_mfa_role(&actor)?;
    let code = request.code.trim().to_owned();
    let user_id = actor.user.id.clone();
    let user_id_lookup = user_id.clone();
    let secret = with_conn(&state.pool, move |connection| {
        Ok(db::find_user_mfa(connection, &user_id_lookup)?.map(|mfa| mfa.secret))
    })
    .await?;
    let Some(secret) = secret else {
        return Err(AuthError::BadRequest("Enroll MFA first.".to_owned()));
    };
    if !totp_verify(&secret, &code) {
        return Err(AuthError::Unauthorized("Invalid MFA code.".to_owned()));
    }
    let at = now();
    with_conn(&state.pool, move |connection| {
        db::set_user_mfa_enabled(connection, &user_id, 1, &at)?;
        Ok(())
    })
    .await?;
    audit_auth(
        &state.pool,
        Some(actor.profile.id.clone()),
        "mfa_confirm",
        &actor.user.id,
        json!({}),
    )
    .await;
    Ok(Json(json!({ "enabled": true })).into_response())
}

pub async fn mfa_status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, AuthError> {
    let actor = require_mfa_actor(&state, &headers).await?;
    let user_id = actor.user.id.clone();
    let enabled = with_conn(&state.pool, move |connection| {
        Ok(db::find_user_mfa(connection, &user_id)?.is_some_and(|mfa| mfa.enabled == 1))
    })
    .await?;
    Ok(Json(json!({ "enabled": enabled })).into_response())
}

pub async fn mfa_disable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<MfaDisableRequest>,
) -> Result<Response, AuthError> {
    let actor = require_mfa_actor(&state, &headers).await?;
    require_mfa_role(&actor)?;
    let password_hash = actor.user.password_hash.clone();
    let password = request.password;
    let valid = tokio::task::spawn_blocking(move || verify_password(&password_hash, &password))
        .await
        .map_err(|_| AuthError::Internal)?;
    if !valid {
        return Err(AuthError::Unauthorized(
            "Invalid email or password.".to_owned(),
        ));
    }
    let user_id = actor.user.id.clone();
    with_conn(&state.pool, move |connection| {
        db::delete_user_mfa(connection, &user_id)?;
        Ok(())
    })
    .await?;
    audit_auth(
        &state.pool,
        Some(actor.profile.id.clone()),
        "mfa_disable",
        &actor.user.id,
        json!({}),
    )
    .await;
    Ok(Json(json!({ "enabled": false })).into_response())
}

pub async fn change_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<Response, AuthError> {
    let actor = require_mfa_actor(&state, &headers).await?;
    if request.new_password.chars().count() < 8 {
        return Err(AuthError::BadRequest(
            "Password must be at least 8 characters.".to_owned(),
        ));
    }
    let password_hash = actor.user.password_hash.clone();
    let current = request.current_password;
    let valid = tokio::task::spawn_blocking(move || verify_password(&password_hash, &current))
        .await
        .map_err(|_| AuthError::Internal)?;
    if !valid {
        return Err(AuthError::Unauthorized(
            "Invalid email or password.".to_owned(),
        ));
    }
    let new_hash = hash_password_async(request.new_password).await?;
    let token = new_token();
    let token_hash = hash_token(&token);
    let session_id = Uuid::now_v7().to_string();
    let created_at = now();
    let expires_at = session_expiry();
    let user_id = actor.user.id.clone();
    let device_id = actor.device_id.clone();
    let new_hash_ref = new_hash.clone();
    let profile_id = actor.profile.id.clone();
    let actor_email = actor.user.email.clone();
    with_conn(&state.pool, move |connection| {
        db::immediate_transaction(connection, |connection| {
            db::update_user_password(connection, &user_id, &new_hash_ref, &created_at)?;
            db::delete_sessions_by_user(connection, &user_id)?;
            db::insert_session(
                connection,
                &NewSession {
                    id: &session_id,
                    token_hash: &token_hash,
                    user_id: &user_id,
                    expires_at: &expires_at,
                    created_at: &created_at,
                    device_id: device_id.as_deref(),
                    mfa_pending: 0,
                },
            )?;
            service::record_audit(
                connection,
                Some(&profile_id),
                "change_password",
                "auth",
                &user_id,
                Some(json!({ "email": actor_email })),
                &created_at,
            )?;
            Ok(())
        })
    })
    .await?;
    Ok(session_response(&token, &actor.user, &state))
}

pub async fn get_session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, AuthError> {
    let Some(token) = cookie_value(&headers, SESSION_COOKIE) else {
        return Ok(Json(Value::Null).into_response());
    };

    let token_hash = hash_token(&token);
    let now = now();
    let session = with_conn(&state.pool, move |connection| {
        let Some(session) = db::find_session_by_token_hash(connection, &token_hash)? else {
            return Ok(None);
        };
        if session.expires_at < now {
            return Ok(None);
        };
        if session.mfa_pending != 0 {
            return Ok(None);
        }
        if let Some(device_id) = session.device_id.as_deref() {
            let valid = db::find_device_by_id(connection, device_id)?
                .is_some_and(|device| device.revoked_at.is_none());
            if !valid {
                db::delete_session_by_token_hash(connection, &token_hash)?;
                return Ok(None);
            }
        }
        Ok(Some(session))
    })
    .await?;

    let Some(session) = session else {
        return Ok(Json(Value::Null).into_response());
    };
    let session_id = session.id;
    let user_id = session.user_id;
    let expires_at = session.expires_at;

    let user = with_conn(&state.pool, move |connection| {
        db::find_user_by_id(connection, &user_id).map_err(AuthError::from)
    })
    .await?;

    let Some(user) = user else {
        let token_hash = hash_token(&token);
        with_conn(&state.pool, move |connection| {
            db::delete_session_by_token_hash(connection, &token_hash)?;
            Ok(())
        })
        .await?;
        return Ok(Json(Value::Null).into_response());
    };

    Ok(Json(json!({
        "session": {
            "id": session_id,
            "userId": user.id,
            "expiresAt": expires_at,
        },
        "user": user_json(&user),
    }))
    .into_response())
}

pub async fn sign_out(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, AuthError> {
    if let Some(token) = cookie_value(&headers, SESSION_COOKIE) {
        let token_hash = hash_token(&token);
        with_conn(&state.pool, move |connection| {
            db::delete_session_by_token_hash(connection, &token_hash)?;
            Ok(())
        })
        .await?;
    }

    let mut response = Json(json!({ "success": true })).into_response();
    add_cookie(&mut response, expired_cookie(state.cookie_secure));
    Ok(response)
}

pub(crate) async fn with_conn<T, F>(pool: &DbPool, operation: F) -> Result<T, AuthError>
where
    T: Send + 'static,
    F: FnOnce(&mut SqliteConnection) -> Result<T, AuthError> + Send + 'static,
{
    let pool = pool.clone();
    tokio::task::spawn_blocking(move || {
        let mut connection = pool.get()?;
        operation(&mut connection)
    })
    .await
    .map_err(|_| AuthError::Internal)?
}

async fn hash_password_async(password: String) -> Result<String, AuthError> {
    tokio::task::spawn_blocking(move || hash_password(&password))
        .await
        .map_err(|_| AuthError::Internal)?
}

fn hash_password(password: &str) -> Result<String, AuthError> {
    Argon2::default()
        .hash_password(password.as_bytes())
        .map(|hash| hash.to_string())
        .map_err(|_| AuthError::Internal)
}

fn verify_password(hash: &str, password: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

fn new_token() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

pub(crate) fn hash_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

pub(crate) fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn session_expiry() -> String {
    (Utc::now() + ChronoDuration::days(SESSION_DAYS)).to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn user_json(user: &db::User) -> Value {
    json!({
        "id": user.id,
        "email": user.email,
        "name": user.display_name,
        "createdAt": user.created_at,
        "updatedAt": user.updated_at,
    })
}

fn session_response(token: &str, user: &db::User, state: &AppState) -> Response {
    let mut response = Json(json!({
        "user": user_json(user),
    }))
    .into_response();
    add_cookie(&mut response, session_cookie(token, state.cookie_secure));
    response
}

fn add_cookie(response: &mut Response, cookie: String) {
    if let Ok(value) = HeaderValue::from_str(&cookie) {
        response.headers_mut().append(header::SET_COOKIE, value);
    }
}

fn session_cookie(token: &str, secure: bool) -> String {
    let max_age = SESSION_DAYS * 24 * 60 * 60;
    let secure = if secure { "; Secure" } else { "" };
    format!("{SESSION_COOKIE}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}{secure}")
}

fn expired_cookie(secure: bool) -> String {
    let secure = if secure { "; Secure" } else { "" };
    format!("{SESSION_COOKIE}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0{secure}")
}

pub(crate) fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|part| {
            let (key, value) = part.trim().split_once('=')?;
            (key == name).then(|| value.to_owned())
        })
}

fn generate_secret() -> String {
    let mut bytes = Vec::with_capacity(20);
    bytes.extend_from_slice(&Uuid::new_v4().into_bytes());
    bytes.extend_from_slice(&Uuid::new_v4().into_bytes()[..4]);
    base32_encode(&bytes)
}

fn otpauth_uri(email: &str, secret: &str) -> String {
    format!(
        "otpauth://totp/PersonaSanus%3A{email}?secret={secret}&issuer=PersonaSanus&algorithm=SHA256&digits=6&period=30"
    )
}

const BASE32_ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

fn base32_encode(bytes: &[u8]) -> String {
    let mut output = String::new();
    let mut buffer: u32 = 0;
    let mut bits = 0;
    for &byte in bytes {
        buffer = (buffer << 8) | u32::from(byte);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            let index = ((buffer >> bits) & 0x1f) as usize;
            output.push(BASE32_ALPHABET[index] as char);
        }
    }
    if bits > 0 {
        let index = ((buffer << (5 - bits)) & 0x1f) as usize;
        output.push(BASE32_ALPHABET[index] as char);
    }
    output
}

fn base32_decode(text: &str) -> Option<Vec<u8>> {
    let mut output = Vec::new();
    let mut buffer: u32 = 0;
    let mut bits = 0;
    for ch in text.trim().trim_end_matches('=').chars() {
        let value = match ch.to_ascii_uppercase() {
            'A'..='Z' => ch.to_ascii_uppercase() as u32 - 'A' as u32,
            '2'..='7' => ch as u32 - '2' as u32 + 26,
            _ => return None,
        };
        buffer = (buffer << 5) | value;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            output.push(((buffer >> bits) & 0xff) as u8);
        }
    }
    Some(output)
}

fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    let mut key_block = [0u8; 64];
    if key.len() > 64 {
        let hashed = Sha256::digest(key);
        key_block[..hashed.len()].copy_from_slice(&hashed);
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }
    let mut inner = Vec::with_capacity(64 + message.len());
    inner.extend(key_block.iter().map(|byte| byte ^ 0x36));
    inner.extend_from_slice(message);
    let inner_hash = Sha256::digest(&inner);
    let mut outer = Vec::with_capacity(64 + inner_hash.len());
    outer.extend(key_block.iter().map(|byte| byte ^ 0x5c));
    outer.extend_from_slice(&inner_hash);
    Sha256::digest(&outer).into()
}

fn hotp(secret: &[u8], counter: u64) -> u32 {
    let digest = hmac_sha256(secret, &counter.to_be_bytes());
    let offset = usize::from(digest[31] & 0x0f);
    let code = u32::from(digest[offset] & 0x7f) << 24
        | u32::from(digest[offset + 1]) << 16
        | u32::from(digest[offset + 2]) << 8
        | u32::from(digest[offset + 3]);
    code % 1_000_000
}

fn totp_verify(secret: &str, code: &str) -> bool {
    let Some(key) = base32_decode(secret) else {
        return false;
    };
    let Ok(candidate) = code.trim().parse::<u32>() else {
        return false;
    };
    if code.trim().chars().count() != 6 {
        return false;
    }
    let counter = Utc::now().timestamp() / TOTP_PERIOD_SECS;
    for step in -1i64..=1 {
        let value = hotp(&key, (counter + step) as u64);
        if value == candidate {
            return true;
        }
    }
    false
}
