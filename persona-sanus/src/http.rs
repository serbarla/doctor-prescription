#[cfg(not(target_arch = "wasm32"))]
use std::sync::LazyLock;

use serde_json::Value;

#[cfg(target_arch = "wasm32")]
pub fn api_base() -> String {
    web_sys::window()
        .and_then(|window| window.location().origin().ok())
        .unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn api_base() -> String {
    std::env::var("PERSONA_API_BASE")
        .unwrap_or_else(|_| "http://127.0.0.1:8080".to_owned())
        .trim_end_matches('/')
        .to_owned()
}

#[cfg(not(target_arch = "wasm32"))]
static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap_or_default()
});

#[cfg(not(target_arch = "wasm32"))]
fn client() -> reqwest::Client {
    CLIENT.clone()
}

#[cfg(target_arch = "wasm32")]
fn client() -> reqwest::Client {
    reqwest::Client::new()
}

fn url(path: &str) -> String {
    format!("{}{}", api_base(), path)
}

#[cfg(target_arch = "wasm32")]
fn with_credentials(builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    builder.fetch_credentials_include()
}

#[cfg(not(target_arch = "wasm32"))]
fn with_credentials(builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    builder
}

pub fn string_field(value: &Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        match value.get(key) {
            Some(Value::String(text)) if !text.is_empty() => return Some(text.clone()),
            Some(Value::Number(number)) => return Some(number.to_string()),
            _ => {}
        }
    }
    None
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
        .map_err(|error| format!("Could not reach the server: {error}"))?;
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|error| format!("Could not read the server response: {error}"))?;
    let value: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    if status.is_success() {
        Ok(value)
    } else {
        Err(error_message(&value, status))
    }
}

pub async fn get_json(path: &str) -> Result<Value, String> {
    send(with_credentials(client().get(url(path)))).await
}

pub async fn post_json(path: &str, body: Value) -> Result<Value, String> {
    send(with_credentials(client().post(url(path)).json(&body))).await
}

pub async fn put_json(path: &str, body: Value) -> Result<Value, String> {
    send(with_credentials(client().put(url(path)).json(&body))).await
}

pub async fn delete_json(path: &str) -> Result<Value, String> {
    send(with_credentials(client().delete(url(path)))).await
}

pub fn encode_query(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(*byte as char)
            }
            b' ' => encoded.push('+'),
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}
