use serde_json::{json, Value};

pub const AUTH_BASE: &str = "http://127.0.0.1:8080/api/auth";

#[derive(Debug, Clone, PartialEq)]
pub struct SessionUser {
    pub id: String,
    pub email: String,
    pub name: String,
}

fn string_field(value: &Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        match value.get(key) {
            Some(Value::String(text)) if !text.is_empty() => return Some(text.clone()),
            Some(Value::Number(number)) => return Some(number.to_string()),
            _ => {}
        }
    }
    None
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

#[cfg(target_arch = "wasm32")]
fn with_credentials(builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    builder.fetch_credentials_include()
}

#[cfg(not(target_arch = "wasm32"))]
fn with_credentials(builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    builder
}

fn error_message(value: &Value, status: reqwest::StatusCode) -> String {
    if let Some(message) = string_field(value, &["message", "error", "code"]) {
        return message;
    }
    if let Some(error) = value.get("error") {
        if let Some(message) = string_field(error, &["message", "code"]) {
            return message;
        }
    }
    format!("Request failed ({status})")
}

async fn send(builder: reqwest::RequestBuilder) -> Result<Value, String> {
    let response = builder
        .send()
        .await
        .map_err(|error| format!("Could not reach the auth server: {error}"))?;
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|error| format!("Could not read the auth server response: {error}"))?;
    let value: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    if status.is_success() {
        Ok(value)
    } else {
        Err(error_message(&value, status))
    }
}

async fn get_json(path: &str) -> Result<Value, String> {
    let url = format!("{AUTH_BASE}{path}");
    send(with_credentials(reqwest::Client::new().get(url))).await
}

async fn post_json(path: &str, body: Value) -> Result<Value, String> {
    let url = format!("{AUTH_BASE}{path}");
    send(with_credentials(reqwest::Client::new().post(url)).json(&body)).await
}

pub async fn fetch_session() -> Option<SessionUser> {
    match get_json("/get-session").await {
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
        "/sign-up/email",
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
        "/sign-in/email",
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
    post_json("/sign-out", json!({})).await?;
    Ok(())
}
