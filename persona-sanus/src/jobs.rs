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

fn backup_paths() -> (PathBuf, PathBuf, String) {
    let dir = std::env::var("BACKUP_DIR").unwrap_or_else(|_| "./backups".to_owned());
    let date = Utc::now().format("%Y%m%d").to_string();
    let plain = PathBuf::from(&dir).join(format!("app-{date}.db"));
    let encrypted = plain.with_extension("db.age");
    (
        PathBuf::from(dir),
        plain,
        encrypted.to_string_lossy().into_owned(),
    )
}

async fn daily_backup(pool: &DbPool, state: AppState) {
    let backup_pool = pool.clone();
    let outcome = tokio::task::spawn_blocking(move || -> Result<String, String> {
        let (dir, plain, encrypted) = backup_paths();
        let recipient = std::env::var("BACKUP_AGE_RECIPIENT").unwrap_or_default();
        if recipient.trim().is_empty() {
            return Err("BACKUP_AGE_RECIPIENT is not set".to_owned());
        }
        std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
        let mut connection = backup_pool.get().map_err(|error| error.to_string())?;
        let literal = plain.to_string_lossy().replace('\'', "''");
        diesel::sql_query(format!("VACUUM INTO '{literal}'"))
            .execute(&mut connection)
            .map_err(|error| error.to_string())?;
        let status = std::process::Command::new("age")
            .args(["-r", recipient.trim(), "-o", &encrypted])
            .arg(&plain)
            .status()
            .map_err(|error| error.to_string())?;
        let _ = std::fs::remove_file(&plain);
        if !status.success() {
            return Err("age encryption failed".to_owned());
        }
        if state.assets.is_some() {
            let mirror_dir = dir.join(format!("minio-{}", Utc::now().format("%Y%m%d")));
            let _ = std::fs::create_dir_all(&mirror_dir);
            let endpoint = std::env::var("ASSETS_ENDPOINT").unwrap_or_default();
            let bucket = std::env::var("ASSETS_BUCKET").unwrap_or_default();
            let access = std::env::var("ASSETS_ACCESS_KEY_ID").unwrap_or_default();
            let secret = std::env::var("ASSETS_SECRET_ACCESS_KEY").unwrap_or_default();
            if !endpoint.is_empty() && !bucket.is_empty() {
                let _ = std::process::Command::new("mc")
                    .args([
                        "alias",
                        "set",
                        "persona-backup",
                        &endpoint,
                        &access,
                        &secret,
                    ])
                    .status();
                let _ = std::process::Command::new("mc")
                    .args([
                        "mirror",
                        &format!("persona-backup/{bucket}"),
                        mirror_dir.to_string_lossy().as_ref(),
                    ])
                    .status();
            }
        }
        Ok(encrypted)
    })
    .await;
    let outcome: Result<String, String> = outcome.unwrap_or_else(|error| Err(error.to_string()));
    let pool = pool.clone();
    let _ = tokio::task::spawn_blocking(move || {
        let mut connection = match pool.get() {
            Ok(connection) => connection,
            Err(_) => return,
        };
        let now = auth_server::now();
        match outcome {
            Ok(path) => {
                let _ = audit(
                    &mut connection,
                    "backup_daily",
                    "backup",
                    "sqlite",
                    json!({ "path": path, "encrypted": true }),
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
