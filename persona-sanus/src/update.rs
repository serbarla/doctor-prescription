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
    pub signature: String,
}

#[derive(Debug)]
pub enum UpdateError {
    BadSignature,
    NotNewer,
    NotInRollout,
    Fetch(String),
    Io(String),
}

pub fn canonical_manifest(version: &str, rollout: u8, url: &str) -> String {
    format!("{version}|{rollout}|{url}")
}

pub fn verify(manifest: &Manifest, public_key_b64: &str) -> bool {
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
    let payload = canonical_manifest(&manifest.version, manifest.rollout_percent, &manifest.url);
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

pub fn evaluate(
    manifest: &Manifest,
    public_key_b64: &str,
    current_version: &str,
    device_id: &str,
) -> Result<(), UpdateError> {
    if !verify(manifest, public_key_b64) {
        return Err(UpdateError::BadSignature);
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
    let bytes = reqwest::blocking::get(&manifest.url)
        .map_err(|error| UpdateError::Fetch(error.to_string()))?;
    let bytes = bytes
        .bytes()
        .map_err(|error| UpdateError::Fetch(error.to_string()))?;
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

pub fn run_check() {
    let Ok(url) = std::env::var("UPDATE_MANIFEST_URL") else {
        return;
    };
    let Ok(public_key) = std::env::var("UPDATE_PUBLIC_KEY") else {
        return;
    };
    let device_id = std::env::var("PERSONA_DEVICE_ID").unwrap_or_default();
    let manifest =
        match reqwest::blocking::get(url.trim()).and_then(|response| response.json::<Manifest>()) {
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
