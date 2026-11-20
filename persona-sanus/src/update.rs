use base64::Engine;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub version: String,
    pub rollout_percent: u8,
    pub url: String,
    #[serde(default)]
    pub sha256: String,
    pub signature: String,
}

#[derive(Debug)]
pub enum UpdateError {
    BadSignature,
    NotNewer,
    NotInRollout,
    EmptyDeviceId,
    InsecureTransport,
    DigestMismatch,
    Fetch(String),
    Io(String),
}

pub fn canonical_manifest(version: &str, rollout: u8, url: &str, sha256: &str) -> String {
    format!("{version}|{rollout}|{url}|{}", sha256.trim().to_lowercase())
}

pub fn verify(manifest: &Manifest, public_key_b64: &str) -> bool {
    let digest = manifest.sha256.trim().to_lowercase();
    if digest.len() != 64 || !digest.chars().all(|c| c.is_ascii_hexdigit()) {
        return false;
    }
    let Ok(key_bytes) = base64::engine::general_purpose::STANDARD.decode(public_key_b64.trim())
    else {
        return false;
    };
    let Ok(public_key) = <[u8; 32]>::try_from(key_bytes) else {
        return false;
    };
    let Ok(sig_bytes) = base64::engine::general_purpose::STANDARD.decode(manifest.signature.trim())
    else {
        return false;
    };
    let Ok(sig) = <[u8; 64]>::try_from(sig_bytes) else {
        return false;
    };
    let Ok(verifying) = VerifyingKey::from_bytes(&public_key) else {
        return false;
    };
    let payload = canonical_manifest(
        &manifest.version,
        manifest.rollout_percent,
        &manifest.url,
        &manifest.sha256,
    );
    verifying
        .verify(payload.as_bytes(), &Signature::from_bytes(&sig))
        .is_ok()
}

pub fn version_is_newer(candidate: &str, current: &str) -> bool {
    let parse = |value: &str| -> Vec<u64> {
        value
            .split('.')
            .map(|part| part.parse::<u64>().unwrap_or(0))
            .collect()
    };
    parse(candidate).cmp(&parse(current)) == std::cmp::Ordering::Greater
}

pub fn in_rollout(device_id: &str, rollout_percent: u8) -> bool {
    if rollout_percent >= 100 {
        return true;
    }
    if rollout_percent == 0 {
        return false;
    }
    let hash = Sha256::digest(device_id.as_bytes());
    (hash[0] as u16 % 100) < rollout_percent as u16
}

fn insecure_http_allowed() -> bool {
    matches!(
        std::env::var("PERSONA_UPDATE_ALLOW_INSECURE_HTTP")
            .unwrap_or_default()
            .trim()
            .to_lowercase()
            .as_str(),
        "1" | "true"
    )
}

fn url_scheme_host(url: &str) -> Option<(String, String)> {
    let (scheme, rest) = url.trim().split_once("://")?;
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    let hostport = authority.rsplit('@').next().unwrap_or("");
    let host = if let Some(stripped) = hostport.strip_prefix('[') {
        stripped.split(']').next()?
    } else {
        hostport.split(':').next()?
    };
    Some((scheme.to_lowercase(), host.to_lowercase()))
}

fn is_loopback_host(host: &str) -> bool {
    matches!(host, "127.0.0.1" | "localhost" | "::1")
}

pub fn transport_allowed(url: &str) -> bool {
    let Some((scheme, host)) = url_scheme_host(url) else {
        return false;
    };
    if scheme == "https" {
        return true;
    }
    scheme == "http" && cfg!(debug_assertions) && is_loopback_host(&host) && insecure_http_allowed()
}

pub fn evaluate(
    manifest: &Manifest,
    public_key_b64: &str,
    current_version: &str,
    device_id: &str,
) -> Result<(), UpdateError> {
    if device_id.trim().is_empty() {
        return Err(UpdateError::EmptyDeviceId);
    }
    if !verify(manifest, public_key_b64) {
        return Err(UpdateError::BadSignature);
    }
    if !transport_allowed(&manifest.url) {
        return Err(UpdateError::InsecureTransport);
    }
    if !version_is_newer(&manifest.version, current_version) {
        return Err(UpdateError::NotNewer);
    }
    if !in_rollout(device_id, manifest.rollout_percent) {
        return Err(UpdateError::NotInRollout);
    }
    Ok(())
}

pub fn stage(manifest: &Manifest) -> Result<std::path::PathBuf, UpdateError> {
    if !transport_allowed(&manifest.url) {
        return Err(UpdateError::InsecureTransport);
    }
    let bytes = reqwest::blocking::get(&manifest.url)
        .map_err(|error| UpdateError::Fetch(error.to_string()))?;
    let bytes = bytes
        .bytes()
        .map_err(|error| UpdateError::Fetch(error.to_string()))?;
    let actual = hex::encode(Sha256::digest(bytes.as_ref()));
    if actual != manifest.sha256.trim().to_lowercase() {
        return Err(UpdateError::DigestMismatch);
    }
    let exe = std::env::current_exe().map_err(|error| UpdateError::Io(error.to_string()))?;
    let previous = exe.with_extension("prev");
    let staged = exe.with_extension("new");
    std::fs::copy(&exe, &previous).map_err(|error| UpdateError::Io(error.to_string()))?;
    std::fs::write(&staged, bytes).map_err(|error| UpdateError::Io(error.to_string()))?;
    Ok(staged)
}

pub fn rollback() -> Result<(), UpdateError> {
    let exe = std::env::current_exe().map_err(|error| UpdateError::Io(error.to_string()))?;
    let previous = exe.with_extension("prev");
    if !previous.exists() {
        return Err(UpdateError::Io("no previous version retained".to_owned()));
    }
    std::fs::copy(&previous, &exe).map_err(|error| UpdateError::Io(error.to_string()))?;
    Ok(())
}

pub fn resolve_device_id() -> String {
    match std::env::var("PERSONA_DEVICE_ID") {
        Ok(value) if !value.trim().is_empty() => value.trim().to_owned(),
        _ => crate::device::device_id(),
    }
}

pub fn run_check() {
    let Ok(url) = std::env::var("UPDATE_MANIFEST_URL") else {
        return;
    };
    let Ok(public_key) = std::env::var("UPDATE_PUBLIC_KEY") else {
        return;
    };
    let url = url.trim().to_owned();
    if !transport_allowed(&url) {
        eprintln!("update: manifest URL refused (not HTTPS)");
        return;
    }
    let device_id = resolve_device_id();
    let manifest =
        match reqwest::blocking::get(&url).and_then(|response| response.json::<Manifest>()) {
            Ok(manifest) => manifest,
            Err(error) => {
                eprintln!("update: manifest fetch failed: {error}");
                return;
            }
        };
    match evaluate(
        &manifest,
        &public_key,
        env!("CARGO_PKG_VERSION"),
        &device_id,
    ) {
        Ok(()) => match stage(&manifest) {
            Ok(path) => eprintln!("update: staged {} at {}", manifest.version, path.display()),
            Err(error) => eprintln!("update: staging failed: {error:?}"),
        },
        Err(UpdateError::BadSignature) => eprintln!("update: bad signature, refused"),
        Err(UpdateError::NotNewer) => {}
        Err(UpdateError::NotInRollout) => eprintln!("update: outside staged rollout"),
        Err(error) => eprintln!("update: {error:?}"),
    }
}
