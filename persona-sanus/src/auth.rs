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

pub async fn sign_up(name: &str, email: &str, password: &str) -> Result<Value, String> {
    post_json(
        "/api/auth/sign-up/email",
        json!({
            "name": name,
            "email": email,
            "password": password,
            "rememberMe": true
        }),
    )
    .await
}
pub async fn sign_in(email: &str, password: &str) -> Result<Value, String> {
    post_json(
        "/api/auth/sign-in/email",
        json!({
            "email": email,
            "password": password,
            "rememberMe": true
        }),
    )
    .await
}
pub async fn sign_out() -> Result<(), String> {
    post_json("/api/auth/sign-out", json!({})).await?;
    Ok(())
}

pub fn is_mfa_required(value: &Value) -> bool {
    value
        .get("mfaRequired")
        .and_then(|flag| flag.as_bool())
        .unwrap_or(false)
}

pub fn mfa_ticket(value: &Value) -> Option<String> {
    string_field(value, &["mfaTicket"])
}

pub async fn verify_mfa(ticket: &str, code: &str) -> Result<Value, String> {
    post_json(
        "/api/auth/mfa/verify",
        json!({ "ticket": ticket, "code": code }),
    )
    .await
}

pub async fn enroll_mfa() -> Result<Value, String> {
    post_json("/api/auth/mfa/enroll", json!({})).await
}

pub async fn confirm_mfa(code: &str) -> Result<Value, String> {
    post_json("/api/auth/mfa/confirm", json!({ "code": code })).await
}

pub async fn disable_mfa(password: &str) -> Result<Value, String> {
    post_json("/api/auth/mfa/disable", json!({ "password": password })).await
}

pub async fn mfa_status() -> Result<bool, String> {
    let value = get_json("/api/auth/mfa/status").await?;
    Ok(value
        .get("enabled")
        .and_then(|flag| flag.as_bool())
        .unwrap_or(false))
}

pub async fn change_password(current: &str, new: &str) -> Result<Value, String> {
    post_json(
        "/api/auth/change-password",
        json!({ "currentPassword": current, "newPassword": new }),
    )
    .await
}
