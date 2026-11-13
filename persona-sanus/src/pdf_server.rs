use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, HeaderValue};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;
use unicode_normalization::UnicodeNormalization;

use crate::api_server::require_actor;
use crate::assets;
use crate::auth_server::{AppState, AuthError};
use crate::pdf::{self, PdfInput, RenderMode};
use crate::service::{self, PrescriptionDocument};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/prescriptions/{id}/pdf", get(prescription_pdf))
        .route("/api/prescriptions/{id}/preview", get(prescription_preview))
        .with_state(state)
}

pub async fn prescription_pdf(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let document = service::prescription_document(&state.pool, &actor, id, false).await?;
    let code = document.verification_code.clone().ok_or_else(|| {
        AuthError::Unprocessable("La receta no tiene un código de verificación.".to_owned())
    })?;
    let pinned = pinned_assets(&state, &document).await?;
    let base = public_base_url();
    let verify_url = format!("{base}/verify/{code}");
    let filename = pdf_filename(&document);
    let title = filename.trim_end_matches(".pdf").to_owned();
    let input = PdfInput {
        doc_json: document.doc_json.clone(),
        values: document.variables(),
        medications: document.medications(),
        assets: pinned,
        margin_mm: document.margin_mm,
        rx_number: document.rx_number,
        issued_at: Some(document.issued_display()),
        mode: RenderMode::Final { code, verify_url },
        title: title.clone(),
    };
    let bytes = tokio::task::spawn_blocking(move || pdf::render(&input))
        .await
        .map_err(|_| AuthError::Internal)?
        .map_err(|_| AuthError::Internal)?;
    let mut response = bytes.into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/pdf"),
    );
    let ascii = ascii_fallback(&filename);
    let encoded = percent_encode(&filename);
    let disposition = format!("attachment; filename=\"{ascii}\"; filename*=UTF-8''{encoded}");
    let value = HeaderValue::from_str(&disposition).map_err(|_| AuthError::Internal)?;
    response
        .headers_mut()
        .insert(header::CONTENT_DISPOSITION, value);
    Ok(response)
}

pub async fn prescription_preview(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let document = service::prescription_document(&state.pool, &actor, id, true).await?;
    let pinned = pinned_assets(&state, &document).await?;
    let base = public_base_url();
    let provisional = document.verification_code.is_none();
    let qr_data_url = document.verification_code.as_ref().and_then(|code| {
        let payload = format!("{base}/verify/{code}");
        pdf::qr_png(&payload).map(|png| data_url("image/png", &png))
    });
    let image_sources: Vec<_> = pinned
        .iter()
        .map(|(asset_id, bytes)| {
            json!({
                "assetId": asset_id,
                "dataUrl": data_url(content_type(bytes), bytes),
            })
        })
        .collect();
    let variables: Vec<_> = document
        .variables()
        .into_iter()
        .map(|(key, value)| json!({ "key": key, "value": value }))
        .collect();
    Ok(Json(json!({
        "id": document.prescription_id,
        "status": document.status,
        "rxNumber": document.rx_number,
        "verificationCode": document.verification_code,
        "issuedAt": document.issued_display(),
        "provisional": provisional,
        "docJson": document.doc_json,
        "marginMm": document.margin_mm,
        "variables": variables,
        "medications": document.medications(),
        "qrDataUrl": qr_data_url,
        "imageSources": image_sources,
    }))
    .into_response())
}

async fn pinned_assets(
    state: &AppState,
    document: &PrescriptionDocument,
) -> Result<Vec<(String, Vec<u8>)>, AuthError> {
    let mut pinned = Vec::new();
    if let Some(storage) = state.assets.as_ref() {
        for asset_id in &document.asset_ids {
            let key = assets::object_key(&document.clinic_id, asset_id);
            if let Some(bytes) = storage.get_object(&key).await? {
                pinned.push((asset_id.clone(), bytes));
            }
        }
    }
    Ok(pinned)
}

fn content_type(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "image/jpeg"
    } else {
        "image/png"
    }
}

fn data_url(content_type: &str, bytes: &[u8]) -> String {
    format!("data:{content_type};base64,{}", base64(bytes))
}

fn base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::new();
    for chunk in bytes.chunks(3) {
        let first = chunk[0] as u32;
        let second = chunk.get(1).copied().unwrap_or(0) as u32;
        let third = chunk.get(2).copied().unwrap_or(0) as u32;
        let value = (first << 16) | (second << 8) | third;
        output.push(TABLE[((value >> 18) & 0x3F) as usize] as char);
        output.push(TABLE[((value >> 12) & 0x3F) as usize] as char);
        output.push(if chunk.len() > 1 {
            TABLE[((value >> 6) & 0x3F) as usize] as char
        } else {
            '='
        });
        output.push(if chunk.len() > 2 {
            TABLE[(value & 0x3F) as usize] as char
        } else {
            '='
        });
    }
    output
}

fn public_base_url() -> String {
    std::env::var("PUBLIC_BASE_URL")
        .ok()
        .map(|value| value.trim().trim_end_matches('/').to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "http://127.0.0.1:8080".to_owned())
}

fn pdf_filename(document: &PrescriptionDocument) -> String {
    let patient = sanitize_part(&document.patient_name, 45);
    let doctor = sanitize_part(&document.doctor_name, 45);
    let name = format!("{}-receta-{}-{}.pdf", patient, document.file_date(), doctor);
    name.chars().take(120).collect()
}

fn sanitize_part(value: &str, limit: usize) -> String {
    let cleaned: String = value
        .chars()
        .filter(|character| {
            !matches!(
                character,
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'
            ) && !character.is_control()
        })
        .collect();
    let collapsed = cleaned.split_whitespace().collect::<Vec<&str>>().join(" ");
    let collapsed = collapsed.trim();
    let text = if collapsed.is_empty() {
        "Sin nombre"
    } else {
        collapsed
    };
    text.chars().take(limit).collect()
}

fn ascii_fallback(value: &str) -> String {
    let text: String = value
        .nfd()
        .filter(|character| character.is_ascii() && !character.is_control())
        .collect();
    if text.trim().is_empty() {
        "receta.pdf".to_owned()
    } else {
        text
    }
}

fn percent_encode(value: &str) -> String {
    let mut output = String::new();
    for byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'!'
            | b'#'
            | b'$'
            | b'&'
            | b'+'
            | b'-'
            | b'.'
            | b'^'
            | b'_'
            | b'`'
            | b'|'
            | b'~' => output.push(*byte as char),
            _ => output.push_str(&format!("%{byte:02X}")),
        }
    }
    output
}
