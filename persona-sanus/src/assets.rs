use std::io::Cursor;
use std::time::Duration;

use chrono::Utc;
use image::{ImageFormat, ImageReader, Limits};
use reqwest::{Client, Method, StatusCode, Url};
use sha2::{Digest, Sha256};

use crate::auth_server::AuthError;

pub const MAX_IMAGE_BYTES: usize = 10_485_760;
const MAX_BODY_BYTES: usize = 11_534_336;
const MAX_DIMENSION: u32 = 12_000;
const MAX_ALLOC: u64 = 256 * 1024 * 1024;
const STARTUP_ATTEMPTS: u32 = 6;

#[derive(Debug, Clone)]
pub struct ProcessedImage {
    pub bytes: Vec<u8>,
    pub content_type: &'static str,
    pub width: u32,
    pub height: u32,
    pub sha256: String,
}

#[derive(Debug)]
pub struct AssetStorage {
    client: Client,
    endpoint: Url,
    public_endpoint: String,
    public_host: String,
    host: String,
    bucket: String,
    region: String,
    access_key_id: String,
    secret_access_key: String,
    ttl_seconds: i64,
}

pub fn object_key(clinic_id: &str, asset_id: &str) -> String {
    format!("clinics/{clinic_id}/images/{asset_id}")
}

pub fn body_limit() -> usize {
    MAX_BODY_BYTES
}

pub fn process_image(bytes: &[u8]) -> Result<ProcessedImage, AuthError> {
    let format = detect_format(bytes).ok_or_else(|| {
        AuthError::UnsupportedMedia("Solo se aceptan imágenes JPEG o PNG.".to_owned())
    })?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_DIMENSION);
    limits.max_image_height = Some(MAX_DIMENSION);
    limits.max_alloc = Some(MAX_ALLOC);
    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    reader.limits(limits);
    let decoded = reader.decode().map_err(|_| {
        AuthError::UnsupportedMedia(
            "No se pudo leer la imagen. Verifica que sea un JPEG o PNG válido.".to_owned(),
        )
    })?;
    let width = decoded.width();
    let height = decoded.height();
    if width == 0 || height == 0 {
        return Err(AuthError::UnsupportedMedia(
            "La imagen no tiene dimensiones válidas.".to_owned(),
        ));
    }
    let mut output = Vec::new();
    let content_type = match format {
        ImageFormat::Jpeg => {
            image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, 85)
                .encode_image(&decoded)
                .map_err(|_| {
                    AuthError::UnsupportedMedia("No se pudo procesar la imagen.".to_owned())
                })?;
            "image/jpeg"
        }
        _ => {
            decoded
                .write_to(&mut Cursor::new(&mut output), ImageFormat::Png)
                .map_err(|_| {
                    AuthError::UnsupportedMedia("No se pudo procesar la imagen.".to_owned())
                })?;
            "image/png"
        }
    };
    if output.len() > MAX_IMAGE_BYTES {
        return Err(AuthError::TooLarge(
            "La imagen supera el límite de 10 MB.".to_owned(),
        ));
    }
    let sha256 = hex_encode(&Sha256::digest(&output));
    Ok(ProcessedImage {
        bytes: output,
        content_type,
        width,
        height,
        sha256,
    })
}

fn detect_format(bytes: &[u8]) -> Option<ImageFormat> {
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some(ImageFormat::Jpeg);
    }
    if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Some(ImageFormat::Png);
    }
    None
}

impl AssetStorage {
    pub fn from_env() -> Option<Self> {
        let endpoint = std::env::var("ASSETS_ENDPOINT").ok()?;
        let bucket = std::env::var("ASSETS_BUCKET").ok()?;
        let access_key_id = std::env::var("ASSETS_ACCESS_KEY_ID").ok()?;
        let secret_access_key = std::env::var("ASSETS_SECRET_ACCESS_KEY").ok()?;
        let endpoint = Url::parse(endpoint.trim_end_matches('/')).ok()?;
        let region = std::env::var("ASSETS_REGION").unwrap_or_else(|_| "us-east-1".to_owned());
        let public_endpoint = std::env::var("ASSETS_PUBLIC_ENDPOINT")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| endpoint.to_string());
        let public_endpoint = public_endpoint.trim_end_matches('/').to_owned();
        let public_url = Url::parse(&public_endpoint).ok()?;
        let ttl_seconds = std::env::var("ASSETS_SIGNED_URL_TTL_SECONDS")
            .ok()
            .and_then(|value| value.parse::<i64>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(900);
        let client = Client::builder()
            .timeout(Duration::from_secs(20))
            .build()
            .ok()?;
        Some(Self {
            client,
            endpoint: endpoint.clone(),
            public_endpoint,
            public_host: authority(&public_url),
            host: authority(&endpoint),
            bucket,
            region,
            access_key_id,
            secret_access_key,
            ttl_seconds,
        })
    }

    pub fn ttl_seconds(&self) -> i64 {
        self.ttl_seconds
    }

    pub async fn initialize(&self) {
        let mut delay = Duration::from_millis(200);
        for _ in 0..STARTUP_ATTEMPTS {
            if self.ensure_ready().await.is_ok() {
                return;
            }
            tokio::time::sleep(delay).await;
            delay = (delay * 2).min(Duration::from_secs(3));
        }
        eprintln!("assets: object storage is not reachable; /api/assets will answer 503");
    }

    pub async fn ensure_ready(&self) -> Result<(), AuthError> {
        if let Ok(response) = self.send(Method::HEAD, "", None, None).await {
            if response.status().is_success() {
                return Ok(());
            }
        }
        match self.send(Method::PUT, "", None, None).await {
            Ok(response)
                if response.status().is_success() || response.status() == StatusCode::CONFLICT =>
            {
                Ok(())
            }
            _ => Err(self.unavailable()),
        }
    }

    pub async fn put_object(
        &self,
        key: &str,
        content_type: &str,
        bytes: &[u8],
    ) -> Result<(), AuthError> {
        let path = format!("/{key}");
        let response = self
            .send(Method::PUT, &path, Some(bytes), Some(content_type))
            .await?;
        if response.status().is_success() {
            return Ok(());
        }
        Err(self.unavailable())
    }

    pub async fn get_object(&self, key: &str) -> Result<Option<Vec<u8>>, AuthError> {
        let path = format!("/{key}");
        let response = self.send(Method::GET, &path, None, None).await?;
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !response.status().is_success() {
            return Err(self.unavailable());
        }
        let bytes = response.bytes().await.map_err(|_| self.unavailable())?;
        Ok(Some(bytes.to_vec()))
    }

    pub async fn delete_object(&self, key: &str) -> Result<(), AuthError> {
        let path = format!("/{key}");
        let response = self.send(Method::DELETE, &path, None, None).await?;
        if response.status().is_success() || response.status() == StatusCode::NOT_FOUND {
            return Ok(());
        }
        Err(self.unavailable())
    }

    pub fn presigned_get(&self, key: &str) -> Result<String, AuthError> {
        let now = Utc::now();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let date = now.format("%Y%m%d").to_string();
        let scope = format!("{date}/{}/s3/aws4_request", self.region);
        let canonical_uri = uri_encode(&format!("/{}/{key}", self.bucket), true);
        let mut params = vec![
            ("X-Amz-Algorithm".to_owned(), "AWS4-HMAC-SHA256".to_owned()),
            (
                "X-Amz-Credential".to_owned(),
                format!("{}/{scope}", self.access_key_id),
            ),
            ("X-Amz-Date".to_owned(), amz_date.clone()),
            ("X-Amz-Expires".to_owned(), self.ttl_seconds.to_string()),
            ("X-Amz-SignedHeaders".to_owned(), "host".to_owned()),
        ];
        params.sort();
        let canonical_query = canonical_query(&params);
        let canonical_headers = format!("host:{}\n", self.public_host);
        let canonical_request = format!(
            "GET\n{canonical_uri}\n{canonical_query}\n{canonical_headers}\nhost\nUNSIGNED-PAYLOAD"
        );
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{amz_date}\n{scope}\n{}",
            hex_encode(&Sha256::digest(canonical_request.as_bytes()))
        );
        let signature = hex_encode(&self.signing_key(&date, &string_to_sign));
        Ok(format!(
            "{}{canonical_uri}?{canonical_query}&X-Amz-Signature={signature}",
            self.public_endpoint
        ))
    }

    async fn send(
        &self,
        method: Method,
        path: &str,
        body: Option<&[u8]>,
        content_type: Option<&str>,
    ) -> Result<reqwest::Response, AuthError> {
        let now = Utc::now();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let date = now.format("%Y%m%d").to_string();
        let scope = format!("{date}/{}/s3/aws4_request", self.region);
        let canonical_uri = uri_encode(&format!("/{}{path}", self.bucket), true);
        let payload_hash = hex_encode(&Sha256::digest(body.unwrap_or_default()));
        let canonical_headers = format!(
            "host:{}\nx-amz-content-sha256:{payload_hash}\nx-amz-date:{amz_date}\n",
            self.host
        );
        let signed_headers = "host;x-amz-content-sha256;x-amz-date";
        let canonical_request = format!(
            "{}\n{canonical_uri}\n\n{canonical_headers}\n{signed_headers}\n{payload_hash}",
            method.as_str()
        );
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{amz_date}\n{scope}\n{}",
            hex_encode(&Sha256::digest(canonical_request.as_bytes()))
        );
        let signature = hex_encode(&self.signing_key(&date, &string_to_sign));
        let authorization = format!(
            "AWS4-HMAC-SHA256 Credential={}/{scope}, SignedHeaders={signed_headers}, Signature={signature}",
            self.access_key_id
        );
        let url = format!(
            "{}{canonical_uri}",
            self.endpoint.as_str().trim_end_matches('/')
        );
        let mut request = self
            .client
            .request(method, url)
            .header("x-amz-content-sha256", payload_hash)
            .header("x-amz-date", amz_date)
            .header("authorization", authorization);
        if let Some(content_type) = content_type {
            request = request.header("content-type", content_type);
        }
        if let Some(body) = body {
            request = request.body(body.to_vec());
        }
        request.send().await.map_err(|_| self.unavailable())
    }

    fn signing_key(&self, date: &str, string_to_sign: &str) -> Vec<u8> {
        let date_key = hmac_sha256(
            format!("AWS4{}", self.secret_access_key).as_bytes(),
            date.as_bytes(),
        );
        let region_key = hmac_sha256(&date_key, self.region.as_bytes());
        let service_key = hmac_sha256(&region_key, b"s3");
        let signing_key = hmac_sha256(&service_key, b"aws4_request");
        hmac_sha256(&signing_key, string_to_sign.as_bytes())
    }

    fn unavailable(&self) -> AuthError {
        AuthError::StorageUnavailable(
            "El almacenamiento de imágenes no está disponible.".to_owned(),
        )
    }
}

fn authority(url: &Url) -> String {
    match (url.host_str(), url.port()) {
        (Some(host), Some(port)) => format!("{host}:{port}"),
        (Some(host), None) => host.to_owned(),
        _ => String::new(),
    }
}

fn canonical_query(params: &[(String, String)]) -> String {
    params
        .iter()
        .map(|(key, value)| format!("{}={}", uri_encode(key, false), uri_encode(value, false)))
        .collect::<Vec<String>>()
        .join("&")
}

fn uri_encode(value: &str, keep_slash: bool) -> String {
    let mut encoded = String::new();
    for byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(*byte as char)
            }
            b'/' if keep_slash => encoded.push('/'),
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut block = [0u8; 64];
    if key.len() > 64 {
        block[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        block[..key.len()].copy_from_slice(key);
    }
    let mut inner_pad = [0x36u8; 64];
    let mut outer_pad = [0x5Cu8; 64];
    for index in 0..64 {
        inner_pad[index] ^= block[index];
        outer_pad[index] ^= block[index];
    }
    let mut inner = Sha256::new();
    inner.update(inner_pad);
    inner.update(data);
    let inner_hash = inner.finalize();
    let mut outer = Sha256::new();
    outer.update(outer_pad);
    outer.update(inner_hash);
    outer.finalize().to_vec()
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}
