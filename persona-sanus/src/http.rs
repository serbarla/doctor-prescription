#[cfg(not(target_arch = "wasm32"))]
use std::sync::{LazyLock, RwLock};

use serde_json::Value;

#[cfg(target_arch = "wasm32")]
pub fn api_base() -> String {
    web_sys::window()
        .and_then(|window| window.location().origin().ok())
        .unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn api_base() -> String {
    std::env::var("PERSONA_SERVER_ORIGIN")
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
static SESSION_TOKEN: LazyLock<RwLock<Option<String>>> = LazyLock::new(|| RwLock::new(None));
#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn set_session_token(token: Option<String>) {
    if let Ok(mut guard) = SESSION_TOKEN.write() {
        *guard = token;
    }
}
#[cfg(not(target_arch = "wasm32"))]
pub fn current_session_token() -> Option<String> {
    SESSION_TOKEN.read().ok().and_then(|guard| guard.clone())
}

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
    match current_session_token() {
        Some(token) => builder.header("Cookie", format!("persona_session={token}")),
        None => builder,
    }
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

#[cfg(not(target_arch = "wasm32"))]
fn capture_session_cookie(response: &reqwest::Response) {
    for value in response
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
    {
        if let Some(pair) = value.split(';').next() {
            if let Some((name, token)) = pair.split_once('=') {
                if name.trim() == "persona_session" {
                    let token = token.trim().to_owned();
                    set_session_token(if token.is_empty() { None } else { Some(token) });
                }
            }
        }
    }
}

async fn send(builder: reqwest::RequestBuilder) -> Result<Value, String> {
    let response = builder
        .send()
        .await
        .map_err(|error| format!("Could not reach the server: {error}"))?;
    #[cfg(not(target_arch = "wasm32"))]
    capture_session_cookie(&response);
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
#[cfg(feature = "desktop")]
pub async fn get_bytes(url_str: &str) -> Result<Vec<u8>, String> {
    let response = client()
        .get(url_str)
        .send()
        .await
        .map_err(|error| format!("Could not reach the server: {error}"))?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("Request failed ({status})"));
    }
    response
        .bytes()
        .await
        .map(|bytes| bytes.to_vec())
        .map_err(|error| format!("Could not read the server response: {error}"))
}
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
pub async fn sse_response(path: &str) -> Result<reqwest::Response, String> {
    let response = with_credentials(client().get(url(path)))
        .send()
        .await
        .map_err(|error| format!("Could not reach the server: {error}"))?;
    if !response.status().is_success() {
        return Err(format!("Request failed ({})", response.status()));
    }
    Ok(response)
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

pub async fn post_file(path: &str, file_name: String, bytes: Vec<u8>) -> Result<Value, String> {
    let part = reqwest::multipart::Part::bytes(bytes).file_name(file_name);
    let form = reqwest::multipart::Form::new().part("file", part);
    send(with_credentials(client().post(url(path)).multipart(form))).await
}

#[cfg(target_arch = "wasm32")]
pub async fn is_server_reachable() -> bool {
    true
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn is_server_reachable() -> bool {
    client()
        .get(url("/health"))
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
        .map(|response| response.status().is_success())
        .unwrap_or(false)
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
