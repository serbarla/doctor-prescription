use serde_json::{json, Value};

use crate::http::{get_json, post_json, string_field};

#[derive(Debug, Clone, PartialEq)]
pub struct SessionUser {
    pub id: String,
    pub email: String,
    pub name: String,
}

pub fn parse_session_user(value: &Value) -> Option<SessionUser> {
    if value.is_null() {
        return None;
    }
    let candidate = match value.get("user") {
        Some(user) if user.is_object() => user,
        _ if value.get("id").is_some() || value.get("email").is_some() => value,
        _ => return None,
    };
    let email = string_field(candidate, &["email"])?;
    let id = string_field(candidate, &["id", "userId", "_id"]).unwrap_or_else(|| email.clone());
    let name = string_field(candidate, &["name", "displayName"]).unwrap_or_else(|| email.clone());
    Some(SessionUser { id, email, name })
}

pub async fn fetch_session() -> Option<SessionUser> {
    match get_json("/api/auth/get-session").await {
        Ok(value) => parse_session_user(&value),
        Err(_) => None,
    }
}

pub async fn sign_up(
    name: &str,
    email: &str,
    password: &str,
) -> Result<Option<SessionUser>, String> {
    let value = post_json(
        "/api/auth/sign-up/email",
        json!({
            "name": name,
            "email": email,
            "password": password,
            "rememberMe": true
        }),
    )
    .await?;
    Ok(parse_session_user(&value))
}

pub async fn sign_in(email: &str, password: &str) -> Result<Option<SessionUser>, String> {
    let value = post_json(
        "/api/auth/sign-in/email",
        json!({
            "email": email,
            "password": password,
            "rememberMe": true
        }),
    )
    .await?;
    Ok(parse_session_user(&value))
}

pub async fn sign_out() -> Result<(), String> {
    post_json("/api/auth/sign-out", json!({})).await?;
    Ok(())
}
