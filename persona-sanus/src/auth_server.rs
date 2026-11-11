use argon2::password_hash::{phc::PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::Argon2;
use axum::extract::State;
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::{Duration as ChronoDuration, SecondsFormat, Utc};
use diesel::prelude::*;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::db::{self, DbPool, NewSession, NewUser};

const SESSION_COOKIE: &str = "persona_session";
const SESSION_DAYS: i64 = 30;

#[derive(Debug, Clone)]
pub struct AppState {
    pub pool: DbPool,
    pub cookie_secure: bool,
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

#[derive(Debug)]
pub enum AuthError {
    BadRequest(String),
    Conflict(String),
    Unauthorized(String),
    Internal,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            Self::Conflict(message) => (StatusCode::CONFLICT, message),
            Self::Unauthorized(message) => (StatusCode::UNAUTHORIZED, message),
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_owned(),
            ),
        };
        (status, Json(json!({ "message": message }))).into_response()
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

    let created_user = with_conn(&state.pool, move |connection| {
        connection.transaction::<db::User, AuthError, _>(|connection| {
            if db::find_user_by_email(connection, &email)?.is_some() {
                return Err(AuthError::Conflict(
                    "An account with this email already exists.".to_owned(),
                ));
            }
            db::insert_user(
                connection,
                &NewUser {
                    id: &user_id,
                    email: &email,
                    password_hash: &password_hash,
                    display_name: &name,
                    created_at: &created_at,
                    updated_at: &created_at,
                },
            )?;
            db::insert_session(
                connection,
                &NewSession {
                    id: &session_id,
                    token_hash: &token_hash,
                    user_id: &user_id,
                    expires_at: &expires_at,
                    created_at: &created_at,
                },
            )?;
            Ok(db::User {
                id: user_id,
                email,
                password_hash,
                display_name: name,
                created_at: created_at.clone(),
                updated_at: created_at,
            })
        })
    })
    .await?;

    Ok(session_response(&token, &created_user, &state))
}

pub async fn sign_in(
    State(state): State<AppState>,
    Json(request): Json<SignInRequest>,
) -> Result<Response, AuthError> {
    let email = request.email.trim().to_lowercase();
    let user = with_conn(&state.pool, move |connection| {
        db::find_user_by_email(connection, &email).map_err(AuthError::from)
    })
    .await?;

    let Some(user) = user else {
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
        return Err(AuthError::Unauthorized(
            "Invalid email or password.".to_owned(),
        ));
    }

    let session_id = Uuid::now_v7().to_string();
    let token = new_token();
    let token_hash = hash_token(&token);
    let created_at = now();
    let expires_at = session_expiry();
    let user_id = user.id.clone();

    with_conn(&state.pool, move |connection| {
        db::insert_session(
            connection,
            &NewSession {
                id: &session_id,
                token_hash: &token_hash,
                user_id: &user_id,
                expires_at: &expires_at,
                created_at: &created_at,
            },
        )?;
        Ok(())
    })
    .await?;

    Ok(session_response(&token, &user, &state))
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
        db::delete_expired_sessions(connection, &now)?;
        db::find_session_by_token_hash(connection, &token_hash).map_err(AuthError::from)
    })
    .await?;

    let Some((session_id, user_id, expires_at)) = session else {
        return Ok(Json(Value::Null).into_response());
    };

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

async fn with_conn<T, F>(pool: &DbPool, operation: F) -> Result<T, AuthError>
where
    T: Send + 'static,
    F: FnOnce(&mut SqliteConnection) -> Result<T, AuthError> + Send + 'static,
{
    let pool = pool.clone();
    tokio::task::spawn_blocking(move || {
        use diesel::connection::SimpleConnection;
        let mut connection = pool.get()?;
        connection.batch_execute("PRAGMA foreign_keys = ON")?;
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

fn hash_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

fn now() -> String {
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
        "token": token,
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

fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
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
