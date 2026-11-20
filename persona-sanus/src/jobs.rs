use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use chrono::{Duration as ChronoDuration, Utc};
use diesel::prelude::*;
use ed25519_dalek::Signer;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::assets::AssetStorage;
use crate::auth_server::{self, AppState};
use crate::db::{self, DbPool};
use crate::update::canonical_manifest;

const STALE_PENDING_ASSET: ChronoDuration = ChronoDuration::hours(1);

fn env_seconds(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

fn retention_enabled() -> bool {
    !matches!(
        std::env::var("RETENTION_TRIM_ENABLED")
            .unwrap_or_else(|_| "true".to_owned())
            .trim()
            .to_lowercase()
            .as_str(),
        "0" | "false" | "no" | "off"
    )
}

fn audit(
    connection: &mut SqliteConnection,
    action: &str,
    entity: &str,
    entity_id: &str,
    metadata: Value,
    at: &str,
) -> diesel::result::QueryResult<()> {
    let id = Uuid::now_v7().to_string();
    let metadata_json = serde_json::to_string(&metadata).ok();
    db::insert_audit(
        connection,
        &db::NewAuditLog {
            id: &id,
            actor_id: None,
            action,
            entity,
            entity_id,
            metadata_json: metadata_json.as_deref(),
            at,
        },
    )?;
    Ok(())
}

async fn purge_expired_sessions(pool: &DbPool) {
    let pool = pool.clone();
    let _ = tokio::task::spawn_blocking(move || {
        let mut connection = match pool.get() {
            Ok(connection) => connection,
            Err(_) => return,
        };
        let now = auth_server::now();
        let deleted = db::delete_expired_sessions(&mut connection, &now).unwrap_or(0);
        let _ = audit(
            &mut connection,
            "session_purge",
            "session",
            "expired",
            json!({ "deleted": deleted }),
            &now,
        );
    })
    .await;
}

async fn cleanup_stale_assets(pool: &DbPool, storage: Option<Arc<AssetStorage>>) {
    let lookup_pool = pool.clone();
    let stale: Vec<(String, String)> = tokio::task::spawn_blocking(move || {
        let mut connection = match lookup_pool.get() {
            Ok(connection) => connection,
            Err(_) => return Vec::new(),
        };
        let cutoff =
            (Utc::now() - STALE_PENDING_ASSET).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        db::list_stale_pending_assets(&mut connection, &cutoff)
            .unwrap_or_default()
            .into_iter()
            .map(|asset| (asset.id, asset.object_key))
            .collect()
    })
    .await
    .unwrap_or_default();
    let mut removed = 0_usize;
    for (id, object_key) in &stale {
        if let Some(storage) = storage.as_ref() {
            let _ = storage.delete_object(object_key).await;
        }
        let delete_pool = pool.clone();
        let id = id.clone();
        let object_key = object_key.clone();
        let at = auth_server::now();
        let outcome = tokio::task::spawn_blocking(move || {
            let mut connection = match delete_pool.get() {
                Ok(connection) => connection,
                Err(_) => return false,
            };
            db::hard_delete_asset(&mut connection, &id).is_ok()
                && audit(
                    &mut connection,
                    "asset_cleanup",
                    "asset",
                    &id,
                    json!({ "objectKey": object_key }),
                    &at,
                )
                .is_ok()
        })
        .await
        .unwrap_or(false);
        if outcome {
            removed += 1;
        }
    }
    if removed == 0 {
        let audit_pool = pool.clone();
        let _ = tokio::task::spawn_blocking(move || {
            let mut connection = match audit_pool.get() {
                Ok(connection) => connection,
                Err(_) => return,
            };
            let now = auth_server::now();
            let _ = audit(
                &mut connection,
                "asset_cleanup",
                "asset",
                "stale_pending",
                json!({ "deleted": 0 }),
                &now,
            );
        })
        .await;
    }
}

async fn retention_trim(pool: &DbPool) {
    let pool = pool.clone();
    let _ = tokio::task::spawn_blocking(move || {
        let mut connection = match pool.get() {
            Ok(connection) => connection,
            Err(_) => return,
        };
        if !retention_enabled() {
            return;
        }
        let now = auth_server::now();
        let clinics = db::list_clinics_past_grace(&mut connection, &now).unwrap_or_default();
        if clinics.is_empty() {
            let _ = audit(
                &mut connection,
                "retention_trim",
                "clinic",
                "past_grace",
                json!({ "clinics": 0 }),
                &now,
            );
            return;
        }
        for clinic in clinics {
            if db::immediate_transaction(&mut connection, |connection| {
                let counts =
                    db::trim_clinic(connection, &clinic.id, &clinic.server_retention_until)?;
                let _ = audit(
                    connection,
                    "retention_trim",
                    "clinic",
                    &clinic.id,
                    json!({
                        "prescriptionItems": counts.prescription_items,
                        "verificationCodes": counts.verification_codes,
                        "prescriptions": counts.prescriptions,
                        "patients": counts.patients,
                        "templateVersionAssets": counts.template_version_assets,
                        "templateVersions": counts.template_versions,
                        "templates": counts.templates,
                        "assets": counts.assets,
                    }),
                    &now,
                );
                Ok::<_, diesel::result::Error>(())
            })
            .is_err()
            {
                eprintln!("jobs: retention trim failed for clinic {}", clinic.id);
            }
        }
    })
    .await;
}

fn set_private(path: &std::path::Path, mode: u32) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
            .map_err(|error| error.to_string())?;
    }
    #[cfg(not(unix))]
    let _ = (path, mode);
    Ok(())
}

fn wipe_and_remove(path: &std::path::Path) -> Result<(), String> {
    if let Ok(mut file) = std::fs::OpenOptions::new().write(true).open(path) {
        let mut remaining = file.metadata().map(|m| m.len()).unwrap_or(0);
        let chunk = [0u8; 65536];
        while remaining > 0 {
            let n = remaining.min(chunk.len() as u64) as usize;
            let _ = file.write_all(&chunk[..n]);
            remaining -= n as u64;
        }
        let _ = file.sync_all();
    }
    std::fs::remove_file(path).map_err(|error| error.to_string())
}

struct TempGuard(Option<PathBuf>);

impl Drop for TempGuard {
    fn drop(&mut self) {
        if let Some(path) = self.0.take() {
            let _ = wipe_and_remove(&path);
        }
    }
}

fn backup_paths() -> (PathBuf, PathBuf, PathBuf, String) {
    let dir = std::env::var("BACKUP_DIR").unwrap_or_else(|_| "./backups".to_owned());
    let date = Utc::now().format("%Y%m%d").to_string();
    let stamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let unique = format!("{stamp}-{}-{}", std::process::id(), Uuid::new_v4().simple());
    let plain_tmp = PathBuf::from(&dir).join(format!(".app-{unique}.db.tmp"));
    let encrypted_tmp = PathBuf::from(&dir).join(format!(".app-{unique}.db.age.tmp"));
    let encrypted_final = PathBuf::from(&dir).join(format!("app-{date}.db.age"));

    (
        PathBuf::from(dir),
        plain_tmp,
        encrypted_tmp,
        encrypted_final.to_string_lossy().into_owned(),
    )
}
fn run_checked(program: &str, args: &[&str], stage: &str) -> Result<(), String> {
    let output = std::process::Command::new(program)
        .args(args)
        .output()
        .map_err(|error| format!("{stage}: failed to run {program}: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let tail: String = stderr
            .chars()
            .rev()
            .take(500)
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        return Err(format!(
            "{stage}: {program} exited {}: {tail}",
            output.status
        ));
    }
    Ok(())
}

fn sweep_stale_plain(dir: &std::path::Path) -> Result<(), String> {
    let mut stale = Vec::new();
    let entries = std::fs::read_dir(dir).map_err(|error| error.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if name.ends_with(".db") || name.ends_with(".db.tmp") || name.ends_with(".age.tmp") {
            stale.push(path);
        }
    }
    if !stale.is_empty() {
        let mut names = Vec::new();
        for path in &stale {
            let _ = wipe_and_remove(path);
            names.push(
                path.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            );
        }
        return Err(format!(
            "stale plaintext backup file(s) {} found and removed; rerun the backup",
            names.join(", ")
        ));
    }
    Ok(())
}

async fn daily_backup(pool: &DbPool, state: AppState) {
    let backup_pool = pool.clone();
    let outcome = tokio::task::spawn_blocking(move || -> Result<(String, Option<String>), String> {
        let (dir, plain, encrypted_tmp, encrypted) = backup_paths();
        let recipient = std::env::var("BACKUP_AGE_RECIPIENT").unwrap_or_default();
        if recipient.trim().is_empty() {
            return Err("BACKUP_AGE_RECIPIENT is not set".to_owned());
        }
        std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
        set_private(&dir, 0o700)?;
        sweep_stale_plain(&dir)?;
        let plain_guard = TempGuard(Some(plain.clone()));
        let encrypted_guard = TempGuard(Some(encrypted_tmp.clone()));
        let mut connection = backup_pool.get().map_err(|error| error.to_string())?;
        let literal = plain.to_string_lossy().replace('\'', "''");
        diesel::sql_query(format!("VACUUM INTO '{literal}'"))
            .execute(&mut connection)
            .map_err(|error| error.to_string())?;
        set_private(&plain, 0o600)?;
        if let Ok(file) = std::fs::OpenOptions::new().read(true).open(&plain) {
            let _ = file.sync_all();
        }
        let status = std::process::Command::new("age")
            .args(["-r", recipient.trim(), "-o"])
            .arg(&encrypted_tmp)
            .arg(&plain)
            .status()
            .map_err(|error| error.to_string())?;
        if !status.success() {
            return Err("age encryption failed".to_owned());
        }
        set_private(&encrypted_tmp, 0o600)?;
        if let Ok(file) = std::fs::OpenOptions::new().read(true).open(&encrypted_tmp) {
            let _ = file.sync_all();
        }
        drop(plain_guard);
        std::fs::rename(&encrypted_tmp, &encrypted).map_err(|error| error.to_string())?;
        let mut encrypted_guard = encrypted_guard;
        encrypted_guard.0 = None;
        if let Ok(dir_file) = std::fs::OpenOptions::new().read(true).open(&dir) {
            let _ = dir_file.sync_all();
        }
        let mut mirror_dir: Option<PathBuf> = None;
        if state.assets.is_some() {
            let endpoint = std::env::var("ASSETS_ENDPOINT").unwrap_or_default();
            let bucket = std::env::var("ASSETS_BUCKET").unwrap_or_default();
            let access = std::env::var("ASSETS_ACCESS_KEY_ID").unwrap_or_default();
            let secret = std::env::var("ASSETS_SECRET_ACCESS_KEY").unwrap_or_default();
            if endpoint.is_empty() || bucket.is_empty() || access.is_empty() || secret.is_empty() {
                return Err(format!(
                    "mirror: asset storage is enabled but ASSETS_ENDPOINT/ASSETS_BUCKET/ASSETS_ACCESS_KEY_ID/ASSETS_SECRET_ACCESS_KEY is incomplete (sqlite backup at {encrypted})"
                ));
            }
            let dir_path = dir.join(format!("minio-{}", Utc::now().format("%Y%m%d")));
            std::fs::create_dir_all(&dir_path).map_err(|error| format!("mirror: {error}"))?;
            run_checked(
                "mc",
                &["alias", "set", "persona-backup", &endpoint, &access, &secret],
                "alias",
            )
            .map_err(|error| format!("{error} (sqlite backup at {encrypted})"))?;
            run_checked(
                "mc",
                &["mirror", &format!("persona-backup/{bucket}"), dir_path.to_string_lossy().as_ref()],
                "mirror",
            )
            .map_err(|error| format!("{error} (sqlite backup at {encrypted})"))?;
            mirror_dir = Some(dir_path);
        }
        Ok((encrypted, mirror_dir.map(|path| path.to_string_lossy().into_owned())))
    })
    .await;
    let outcome: Result<(String, Option<String>), String> =
        outcome.unwrap_or_else(|error| Err(error.to_string()));
    let pool = pool.clone();
    let _ = tokio::task::spawn_blocking(move || {
        let mut connection = match pool.get() {
            Ok(connection) => connection,
            Err(_) => return,
        };
        let now = auth_server::now();
        match outcome {
            Ok((path, mirror)) => {
                let mut metadata = json!({ "path": path, "encrypted": true });
                if let Some(mirror) = mirror {
                    metadata["mirror"] = json!(mirror);
                }
                let _ = audit(
                    &mut connection,
                    "backup_daily",
                    "backup",
                    "sqlite",
                    metadata,
                    &now,
                );
            }
            Err(error) => {
                eprintln!("jobs: daily backup failed: {error}");
                let _ = audit(
                    &mut connection,
                    "backup_daily",
                    "backup",
                    "sqlite",
                    json!({ "error": error }),
                    &now,
                );
            }
        }
    })
    .await;
}

pub fn spawn(state: AppState) {
    let interval = Duration::from_secs(env_seconds("JOBS_INTERVAL_SECONDS", 3600));
    let backup_interval = Duration::from_secs(env_seconds("BACKUP_INTERVAL_SECONDS", 86400));
    tokio::spawn(async move {
        let mut backup_tick = tokio::time::interval(backup_interval);
        loop {
            purge_expired_sessions(&state.pool).await;
            cleanup_stale_assets(&state.pool, state.assets.clone()).await;
            retention_trim(&state.pool).await;
            tokio::select! {
                _ = tokio::time::sleep(interval) => {}
                _ = backup_tick.tick() => {
                    daily_backup(&state.pool, state.clone()).await;
                }
            }
        }
    });
}

pub fn manifest_signing_key() -> Option<ed25519_dalek::SigningKey> {
    let value = std::env::var("UPDATE_SIGNING_KEY").ok()?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(value.trim())
        .ok()?;
    let array: [u8; 32] = bytes.try_into().ok()?;
    Some(ed25519_dalek::SigningKey::from_bytes(&array))
}

pub async fn update_manifest(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> axum::response::Response {
    let _ = state;
    let Some(key) = manifest_signing_key() else {
        return axum::response::IntoResponse::into_response((
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            axum::Json(json!({ "message": "Update manifest is not configured." })),
        ));
    };
    let version = std::env::var("UPDATE_MANIFEST_VERSION").unwrap_or_default();
    let url = std::env::var("UPDATE_MANIFEST_DOWNLOAD_URL").unwrap_or_default();
    let rollout = std::env::var("UPDATE_MANIFEST_ROLLOUT")
        .ok()
        .and_then(|value| value.parse::<u8>().ok())
        .unwrap_or(100);
    let payload = canonical_manifest(&version, rollout, &url);
    let signature = key.sign(payload.as_bytes());
    axum::response::IntoResponse::into_response(axum::Json(json!({
        "version": version,
        "rolloutPercent": rollout,
        "url": url,
        "signature": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, signature.to_bytes()),
    })))
}

pub fn router(state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/api/updates/manifest", axum::routing::get(update_manifest))
        .with_state(state)
}
