mod api;

#[cfg(feature = "server")]
mod api_server;
mod auth;
#[cfg(feature = "desktop")]
mod desktop_local;
mod device;

#[cfg(feature = "server")]
mod assets;
#[cfg(feature = "server")]
mod auth_server;
#[cfg(feature = "server")]
mod db;
mod forms;
mod http;
#[cfg(feature = "server")]
mod jobs;
#[cfg(any(feature = "server", feature = "desktop"))]
mod pdf;
#[cfg(feature = "server")]
mod pdf_server;
#[cfg(feature = "server")]
mod schema;
#[cfg(feature = "server")]
mod server;
#[cfg(feature = "server")]
mod service;
#[cfg(feature = "server")]
mod sync_server;
#[cfg(feature = "server")]
mod verify_server;
#[cfg(any(feature = "server", feature = "desktop"))]
#[allow(dead_code)]
mod update;
use api::Me;
use auth::SessionUser;
use dioxus::prelude::*;
use dioxus::router::{router, Navigator};
#[cfg(feature = "desktop")]
use serde_json::Value;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/verify/:code")]
    Verify { code: String },
    #[layout(Navbar)]
    #[route("/")]
    Home {},
    #[route("/login")]
    AuthPage {},
    #[route("/dashboard")]
    Dashboard {},
    #[route("/patients")]
    Patients {},
    #[route("/patients/new")]
    PatientNew {},
    #[route("/patients/:id")]
    PatientDetail { id: String },
    #[route("/patients/:id/edit")]
    PatientEdit { id: String },
    #[route("/settings")]
    Settings {},
    #[route("/settings/users")]
    SettingsUsers {},
    #[route("/settings/profile")]
    SettingsProfile {},
    #[route("/settings/clinic")]
    SettingsClinic {},
    #[route("/audit")]
    Audit {},
    #[route("/templates")]
    Templates {},
    #[route("/templates/new")]
    TemplateNew {},
    #[route("/templates/:id/edit")]
    TemplateEdit { id: String },
    #[route("/templates/:id/preview")]
    TemplatePreview { id: String },
    #[route("/prescriptions?:status")]
    Prescriptions { status: Option<String> },
    #[route("/prescriptions/new?:patient_id&:template_id")]
    PrescriptionNew {
        patient_id: Option<String>,
        template_id: Option<String>,
    },
    #[route("/prescriptions/:id")]
    PrescriptionDetail { id: String },
    #[route("/prescriptions/:id/preview")]
    PrescriptionPreview { id: String },
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

#[cfg(not(feature = "server"))]
fn main() {
    #[cfg(feature = "desktop")]
    std::thread::spawn(update::run_check);
    dioxus::launch(App);
}

#[cfg(feature = "server")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = dioxus::cli_config::fullstack_address_or_localhost();
    let router = server::router().await?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
struct SyncStatus {
    online: bool,
    pending: i64,
    failed: i64,
}

fn sync_status_label(status: &SyncStatus) -> &'static str {
    if !status.online {
        return "Sin conexión";
    }
    if status.pending > 0 || status.failed > 0 {
        return "Pendientes";
    }
    "En línea"
}

#[cfg(feature = "desktop")]
fn local_pending_count() -> i64 {
    desktop_local::open_local_db()
        .and_then(|mut conn| desktop_local::pending_count(&mut conn))
        .unwrap_or(0)
}
#[cfg(feature = "desktop")]
fn local_failed_count() -> i64 {
    desktop_local::open_local_db()
        .and_then(|mut conn| desktop_local::failed_count(&mut conn))
        .unwrap_or(0)
}
#[cfg(feature = "desktop")]
fn persist_local_session(token: Option<String>, user: &SessionUser, profile: Option<&Me>) {
    let Some(token) = token else {
        return;
    };
    http::set_session_token(Some(token.clone()));
    let user_value = serde_json::json!({
        "id": user.id,
        "email": user.email,
        "name": user.name,
    });
    let profile_value = profile.and_then(|profile| serde_json::to_value(profile).ok());
    let expires_at = (chrono::Utc::now() + chrono::Duration::days(30))
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let _ =
        desktop_local::save_local_session(&token, &user_value, profile_value.as_ref(), &expires_at);
}
#[cfg(feature = "desktop")]
fn restore_local_session() -> Option<(SessionUser, Option<Me>)> {
    let stored = desktop_local::load_local_session()?;
    let user = auth::parse_session_user(&stored.user)?;
    let profile = stored
        .profile
        .as_ref()
        .and_then(|value| serde_json::from_value(value.clone()).ok());
    http::set_session_token(Some(stored.token));
    Some((user, profile))
}
#[cfg(feature = "desktop")]
fn refresh_local_profile(user: &SessionUser, profile: Option<&Me>) {
    let user_value = serde_json::json!({
        "id": user.id,
        "email": user.email,
        "name": user.name,
    });
    let profile_value = profile.and_then(|profile| serde_json::to_value(profile).ok());
    let _ = desktop_local::refresh_local_session(&user_value, profile_value.as_ref());
}
#[cfg(feature = "desktop")]
fn clear_persisted_session() {
    http::set_session_token(None);
    desktop_local::clear_local_session();
}
#[cfg(not(feature = "desktop"))]
fn clear_persisted_session() {}
#[cfg(not(feature = "desktop"))]
fn restore_local_session() -> Option<(SessionUser, Option<Me>)> {
    None
}

#[cfg(not(feature = "desktop"))]
fn local_pending_count() -> i64 {
    0
}
#[cfg(not(feature = "desktop"))]
fn local_failed_count() -> i64 {
    0
}

#[cfg(feature = "desktop")]
fn ensure_sync_loop() {
    start_sync_loop();
}
#[cfg(not(feature = "desktop"))]
fn ensure_sync_loop() {}

async fn probe_sync_status() -> SyncStatus {
    let online = http::is_server_reachable().await;
    if online {
        ensure_sync_loop();
        #[cfg(feature = "desktop")]
        drain_outbox().await;
    }
    SyncStatus {
        online,
        pending: local_pending_count(),
        failed: local_failed_count(),
    }
}

#[cfg(feature = "desktop")]
fn populate_local_cache(profile: &Me) {
    let clinic_id = profile
        .clinic
        .as_ref()
        .map(|clinic| clinic.id.clone())
        .unwrap_or_default();
    let actor_id = profile.profile.id.clone();
    spawn(async move {
        drain_outbox().await;
        let mut conn = match desktop_local::open_local_db() {
            Ok(conn) => conn,
            Err(_) => return,
        };
        let mut synced = false;
        let settings = api::fetch_clinic_settings().await.ok();
        if let Ok(me) = api::fetch_me().await {
            synced = true;
            if let Some(clinic) = me.clinic.as_ref() {
                let value = serde_json::json!({
                    "id": clinic.id,
                    "name": clinic.name,
                    "address": settings.as_ref().and_then(|row| row.address.clone()),
                    "phone": settings.as_ref().and_then(|row| row.phone.clone()),
                    "logo_asset_id": Value::Null,
                    "timezone": clinic.timezone,
                    "llm_url_template": clinic.llm_url_template,
                    "retention_days": clinic.retention_days.unwrap_or(30),
                    "rx_block_size": clinic.rx_block_size.unwrap_or(50),
                    "created_at": "1970-01-01T00:00:00Z",
                    "updated_at": "1970-01-01T00:00:00Z",
                });
                let _ = desktop_local::cache_clinic(&mut conn, &value);
            }
        }
        if let Ok(patients) = api::fetch_patients("").await {
            for patient in patients {
                let value = serde_json::json!({
                    "id": patient.id,
                    "clinic_id": clinic_id,
                    "full_name": patient.full_name,
                    "national_id": patient.national_id,
                    "birth_date": patient.birth_date,
                    "sex": patient.sex,
                    "phone": patient.phone,
                    "notes": patient.notes,
                    "client_id": Value::Null,
                    "deleted_at": Value::Null,
                    "created_at": patient.created_at,
                    "updated_at": patient.updated_at,
                    "created_by": actor_id,
                });
                let _ = desktop_local::cache_patient(&mut conn, &value);
            }
        }
        if let Ok(templates) = api::fetch_templates().await {
            for template in templates {
                let value = serde_json::json!({
                    "id": template.id,
                    "owner_type": template.owner_type,
                    "owner_id": template.owner_id,
                    "name": template.name,
                    "current_version_id": template.current_version_id,
                    "status": template.status,
                    "client_id": Value::Null,
                    "created_at": template.created_at,
                    "updated_at": template.updated_at,
                    "created_by": actor_id,
                });
                let _ = desktop_local::cache_template(&mut conn, &value);
                let mut versions = template.versions.clone();
                if let Some(current) = template.current_version.as_ref() {
                    if !versions.iter().any(|version| version.id == current.id) {
                        versions.push(current.clone());
                    }
                }
                for version in &versions {
                    let version_value = serde_json::json!({
                        "id": version.id,
                        "template_id": template.id,
                        "version_no": version.version_no,
                        "doc_json": version.doc_json,
                        "variables_spec": version.variables_spec,
                        "theme": version.theme,
                        "status": version.status,
                        "created_at": version.created_at,
                        "updated_at": version.created_at,
                        "created_by": actor_id,
                    });
                    let _ = desktop_local::cache_template_version(&mut conn, &version_value);
                    let asset_ids = desktop_local::doc_asset_ids(&version.doc_json);
                    let _ = desktop_local::upsert_pinned_assets(&mut conn, &version.id, &asset_ids);
                }
            }
        }
        if let Ok(prescriptions) = api::fetch_prescriptions("", "", "").await {
            for prescription in prescriptions {
                let mut value = serde_json::json!({
                    "id": prescription.id,
                    "patient_id": prescription.patient_id,
                    "doctor_id": prescription.doctor_id,
                    "clinic_id": prescription.clinic_id,
                    "template_version_id": prescription.template_version_id,
                    "status": prescription.status,
                    "rx_number": prescription.rx_number,
                    "rx_block_id": prescription.rx_block_id,
                    "diagnosis": prescription.diagnosis,
                    "validity": prescription.validity,
                    "instructions": prescription.instructions,
                    "signed_at": prescription.signed_at,
                    "synced_at": Value::Null,
                    "voided_at": prescription.voided_at,
                    "void_reason": prescription.void_reason,
                    "client_id": prescription.client_id,
                    "supersedes_id": prescription.supersedes_id,
                    "created_at": prescription.created_at,
                    "updated_at": prescription.updated_at,
                    "created_by": prescription.created_by,
                });
                if prescription.status != "draft" {
                    value["synced_at"] = serde_json::json!("1970-01-01T00:00:00Z");
                }
                let pending =
                    desktop_local::has_pending_outbox(&mut conn, &prescription.id).unwrap_or(false);
                if !pending {
                    let _ = desktop_local::clear_prescription_items(&mut conn, &prescription.id);
                    let _ = desktop_local::cache_prescription(&mut conn, &value);
                    if let Ok(detail) = api::fetch_prescription(&prescription.id).await {
                        for item in &detail.items {
                            let item_value = serde_json::json!({
                                "id": item.id,
                                "prescription_id": item.prescription_id,
                                "position": item.position,
                                "med_name": item.med_name,
                                "dose": item.dose,
                                "route": item.route,
                                "frequency": item.frequency,
                                "duration": item.duration,
                                "quantity": item.quantity,
                                "notes": item.notes,
                                "created_at": detail.created_at,
                                "updated_at": detail.updated_at,
                                "created_by": actor_id,
                            });
                            let _ = desktop_local::cache_prescription_item(&mut conn, &item_value);
                        }
                    }
                }
            }
        }
        let mut asset_rows: Vec<api::AssetRow> = Vec::new();
        if let Ok(library) = api::fetch_assets().await {
            asset_rows.extend(library.assets);
        }
        if let Some(settings) = settings {
            if let Some(logo) = settings.logo {
                if !asset_rows.iter().any(|row| row.id == logo.id) {
                    asset_rows.push(logo);
                }
            }
        }
        let pinned: Vec<String> =
            desktop_local::cached_pinned_asset_ids(&mut conn).unwrap_or_default();
        for asset in &asset_rows {
            let meta = serde_json::json!({
                "id": asset.id,
                "clinic_id": clinic_id,
                "object_key": Value::Null,
                "owner_type": Value::Null,
                "owner_id": Value::Null,
                "kind": asset.kind,
                "original_name": asset.original_name,
                "content_type": asset.content_type,
                "size_bytes": asset.size_bytes,
                "sha256": asset.sha256,
                "width": asset.width,
                "height": asset.height,
                "status": asset.status,
                "deleted_at": Value::Null,
                "created_at": asset.created_at,
                "updated_at": asset.created_at,
                "created_by": actor_id,
            });
            let _ = desktop_local::cache_asset_meta(&mut conn, &meta);
            if pinned.contains(&asset.id) {
                if let Some(url) = asset.url.as_deref() {
                    if let Ok(bytes) = http::get_bytes(url).await {
                        let _ =
                            desktop_local::cache_image(&mut conn, &asset.id, &bytes, &asset.sha256);
                    }
                }
            }
        }
        if let Ok(cached) = desktop_local::cached_asset_ids(&mut conn) {
            for asset_id in cached {
                if !pinned.contains(&asset_id) {
                    let _ = desktop_local::evict_asset_cache(&mut conn, &asset_id);
                }
            }
        }
        let _ = desktop_local::set_meta(&mut conn, "image_quota", "1");
        if let Ok(status) = api::fetch_rx_block_status().await {
            if let Some(block_id) = status.block_id.as_deref() {
                let value = serde_json::json!({
                    "id": block_id,
                    "clinic_id": clinic_id,
                    "device_id": status.device_id,
                    "start_number": status.start_number,
                    "end_number": status.end_number,
                    "consumed": status.consumed,
                    "status": if status.exhausted { "exhausted" } else { "active" },
                    "created_by": actor_id,
                });
                let _ = desktop_local::cache_rx_block_status(&mut conn, &value);
            }
        }
        if synced {
            let _ = desktop_local::set_meta(
                &mut conn,
                "last_synced_at",
                &chrono::Utc::now().to_rfc3339(),
            );
            let _ = desktop_local::evict_expired(&mut conn);
        }
    });
}

#[cfg(feature = "desktop")]
fn payload_text(payload: &serde_json::Value, key: &str) -> String {
    payload
        .get(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

#[cfg(feature = "desktop")]
fn payload_number(payload: &serde_json::Value, key: &str) -> i64 {
    payload
        .get(key)
        .and_then(serde_json::Value::as_i64)
        .unwrap_or_default()
}

#[cfg(feature = "desktop")]
async fn replay_outbox_row(operation: &str, payload: &serde_json::Value) -> Result<(), String> {
    match operation {
        "create_patient" => {
            let form = api::PatientForm {
                full_name: payload_text(payload, "fullName"),
                national_id: payload_text(payload, "nationalId"),
                birth_date: payload_text(payload, "birthDate"),
                sex: payload_text(payload, "sex"),
                phone: payload_text(payload, "phone"),
                notes: payload_text(payload, "notes"),
            };
            match api::create_patient_with_client_id(&form, &payload_text(payload, "clientId"))
                .await?
            {
                api::PatientCreateOutcome::Created(_) => Ok(()),
                api::PatientCreateOutcome::LikelyDuplicate(ids) => {
                    Err(format!("Posible duplicado: {}.", ids.join(", ")))
                }
            }
        }
        "update_patient" => {
            let form = api::PatientForm {
                full_name: payload_text(payload, "fullName"),
                national_id: payload_text(payload, "nationalId"),
                birth_date: payload_text(payload, "birthDate"),
                sex: payload_text(payload, "sex"),
                phone: payload_text(payload, "phone"),
                notes: payload_text(payload, "notes"),
            };
            api::update_patient(&payload_text(payload, "patientId"), &form).await?;
            Ok(())
        }
        "create_template" => {
            api::create_template_with_client_id(
                &payload_text(payload, "name"),
                &payload_text(payload, "ownerType"),
                &payload_text(payload, "clientId"),
            )
            .await?;
            Ok(())
        }
        "publish_template_version" => {
            api::publish_template(
                &payload_text(payload, "templateId"),
                &payload_text(payload, "versionId"),
            )
            .await?;
            Ok(())
        }
        "create_prescription" => {
            api::create_prescription(
                &payload_text(payload, "patientId"),
                &payload_text(payload, "templateVersionId"),
                &payload_text(payload, "clientId"),
                &payload_text(payload, "diagnosis"),
                &payload_text(payload, "validity"),
                &payload_text(payload, "instructions"),
            )
            .await?;
            Ok(())
        }
        "update_prescription" => {
            api::update_prescription(
                &payload_text(payload, "prescriptionId"),
                &payload_text(payload, "diagnosis"),
                &payload_text(payload, "validity"),
                &payload_text(payload, "instructions"),
            )
            .await?;
            Ok(())
        }
        "upsert_prescription_item" => {
            let form = api::PrescriptionItemForm {
                id: Some(payload_text(payload, "itemId")),
                med_name: payload_text(payload, "medName"),
                dose: payload_text(payload, "dose"),
                route: payload_text(payload, "route"),
                frequency: payload_text(payload, "frequency"),
                duration: payload_text(payload, "duration"),
                quantity: payload_text(payload, "quantity"),
                notes: payload_text(payload, "notes"),
            };
            api::upsert_prescription_item(&payload_text(payload, "prescriptionId"), &form).await?;
            Ok(())
        }
        "sign_prescription" => {
            api::reconcile_sign_prescription(
                &payload_text(payload, "prescriptionId"),
                payload_number(payload, "rxNumber"),
                &payload_text(payload, "rxBlockId"),
            )
            .await?;
            Ok(())
        }
        "void_prescription" => {
            api::void_prescription(
                &payload_text(payload, "prescriptionId"),
                &payload_text(payload, "voidReason"),
            )
            .await?;
            Ok(())
        }
        "supersede_prescription" => {
            api::reconcile_supersede_prescription(
                &payload_text(payload, "prescriptionId"),
                &payload_text(payload, "clientId"),
                &payload_text(payload, "newPrescriptionId"),
                payload_number(payload, "rxNumber"),
                &payload_text(payload, "rxBlockId"),
            )
            .await?;
            Ok(())
        }
        _ => Err(format!("Operación desconocida: {operation}")),
    }
}

#[cfg(feature = "desktop")]
async fn drain_outbox() {
    let mut conn = match desktop_local::open_local_db() {
        Ok(conn) => conn,
        Err(_) => return,
    };
    let rows = desktop_local::pending_outbox_rows(&mut conn).unwrap_or_default();
    for row in rows {
        let id = row
            .get("id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let operation = row
            .get("operation")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let attempts = row
            .get("attempts")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0);
        let payload: serde_json::Value = row
            .get("payloadJson")
            .and_then(serde_json::Value::as_str)
            .and_then(|text| serde_json::from_str(text).ok())
            .unwrap_or(serde_json::Value::Null);
        match replay_outbox_row(&operation, &payload).await {
            Ok(()) => {
                let _ = desktop_local::mark_outbox_synced(&mut conn, &id);
            }
            Err(error)
                if error.starts_with("Could not reach")
                    || error.starts_with("Could not read the server") =>
            {
                let _ = desktop_local::mark_outbox_retry(&mut conn, &id, attempts, &error);
                return;
            }
            Err(error) => {
                let _ = desktop_local::mark_outbox_failed(&mut conn, &id, &error);
            }
        }
    }
}

#[cfg(feature = "desktop")]
static SYNC_LOOP_STARTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[cfg(feature = "desktop")]
fn sync_cursor() -> i64 {
    desktop_local::open_local_db()
        .and_then(|mut conn| desktop_local::get_meta(&mut conn, "sync_cursor"))
        .ok()
        .flatten()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}

#[cfg(feature = "desktop")]
async fn refresh_sync_cache(cursor: i64) {
    let mut conn = match desktop_local::open_local_db() {
        Ok(conn) => conn,
        Err(_) => return,
    };
    let _ = desktop_local::set_meta(&mut conn, "sync_cursor", &cursor.to_string());
    if let Ok(status) = api::fetch_rx_block_status().await {
        if let Some(block_id) = status.block_id.as_deref() {
            let clinic_id = desktop_local::rx_block_clinic_id(&mut conn, block_id)
                .ok()
                .flatten()
                .unwrap_or_default();
            let value = serde_json::json!({
                "id": block_id,
                "clinic_id": clinic_id,
                "device_id": status.device_id,
                "start_number": status.start_number,
                "end_number": status.end_number,
                "consumed": status.consumed,
                "status": if status.exhausted { "exhausted" } else { "active" },
                "created_by": "",
            });
            let _ = desktop_local::cache_rx_block_status(&mut conn, &value);
        }
    }
    let _ = desktop_local::set_meta(
        &mut conn,
        "last_synced_at",
        &chrono::Utc::now().to_rfc3339(),
    );
}

#[cfg(feature = "desktop")]
async fn poll_sync_state() {
    let cursor = sync_cursor();
    if let Ok(state) = api::fetch_sync_state(cursor).await {
        refresh_sync_cache(state.cursor).await;
    }
}

#[cfg(feature = "desktop")]
async fn run_sse_once() -> Result<(), String> {
    let cursor = sync_cursor();
    let mut response = http::sse_response(&format!("/api/sync/events?since={cursor}")).await?;
    let mut buffer = String::new();
    loop {
        let chunk = response
            .chunk()
            .await
            .map_err(|error| format!("Could not read the server response: {error}"))?;
        let Some(chunk) = chunk else {
            return Ok(());
        };
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(position) = buffer.find('\n') {
            let line = buffer[..position].trim_end().to_owned();
            buffer.drain(..=position);
            if let Some(data) = line.strip_prefix("data: ") {
                if let Ok(event) = serde_json::from_str::<api::SyncEvent>(data) {
                    refresh_sync_cache(event.cursor).await;
                }
            }
        }
    }
}

#[cfg(feature = "desktop")]
fn start_sync_loop() {
    if SYNC_LOOP_STARTED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        return;
    }
    spawn(async move {
        loop {
            match run_sse_once().await {
                Ok(()) => {}
                Err(_) => {
                    poll_sync_state().await;
                    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                }
            }
        }
    });
}

#[cfg(not(feature = "desktop"))]
fn populate_local_cache(_profile: &Me) {}

#[cfg(feature = "desktop")]
fn sign_prescription_offline(
    prescription: &api::Prescription,
    profile: &Me,
) -> Result<api::Prescription, String> {
    let mut conn = desktop_local::open_local_db()?;
    let value = serde_json::json!({
        "id": prescription.id,
        "patient_id": prescription.patient_id,
        "doctor_id": prescription.doctor_id,
        "clinic_id": prescription.clinic_id,
        "template_version_id": prescription.template_version_id,
        "status": prescription.status,
        "rx_number": prescription.rx_number,
        "rx_block_id": prescription.rx_block_id,
        "diagnosis": prescription.diagnosis,
        "validity": prescription.validity,
        "instructions": prescription.instructions,
        "signed_at": prescription.signed_at,
        "synced_at": Value::Null,
        "voided_at": prescription.voided_at,
        "void_reason": prescription.void_reason,
        "client_id": prescription.client_id,
        "supersedes_id": prescription.supersedes_id,
        "created_at": prescription.created_at,
        "updated_at": prescription.updated_at,
        "created_by": prescription.created_by,
    });
    desktop_local::cache_prescription(&mut conn, &value)?;
    let doctor_id = profile
        .doctor
        .as_ref()
        .map(|doctor| doctor.id.clone())
        .or_else(|| prescription.doctor_id.clone())
        .ok_or_else(|| "El perfil no tiene ficha de médico.".to_owned())?;
    let actor_id = profile.profile.id.clone();
    let device_id = crate::device::device_id();
    let result = desktop_local::offline_sign(
        &mut conn,
        &prescription.id,
        &device_id,
        &doctor_id,
        &actor_id,
    )?;
    let mut signed = prescription.clone();
    signed.status = "signed".to_owned();
    signed.rx_number = Some(result.rx_number as i32);
    signed.rx_block_id = Some(result.rx_block_id);
    signed.doctor_id = Some(doctor_id);
    signed.signed_at = Some(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
    Ok(signed)
}

#[cfg(feature = "desktop")]
fn update_prescription_offline(
    prescription: &api::Prescription,
    profile: &Me,
    diagnosis: &str,
    validity: &str,
    instructions: &str,
) -> Result<api::Prescription, String> {
    let mut conn = desktop_local::open_local_db()?;
    let value = serde_json::json!({
        "id": prescription.id,
        "patient_id": prescription.patient_id,
        "doctor_id": prescription.doctor_id,
        "clinic_id": prescription.clinic_id,
        "template_version_id": prescription.template_version_id,
        "status": prescription.status,
        "rx_number": prescription.rx_number,
        "rx_block_id": prescription.rx_block_id,
        "diagnosis": prescription.diagnosis,
        "validity": prescription.validity,
        "instructions": prescription.instructions,
        "signed_at": prescription.signed_at,
        "synced_at": Value::Null,
        "voided_at": prescription.voided_at,
        "void_reason": prescription.void_reason,
        "client_id": prescription.client_id,
        "supersedes_id": prescription.supersedes_id,
        "created_at": prescription.created_at,
        "updated_at": prescription.updated_at,
        "created_by": prescription.created_by,
    });
    desktop_local::cache_prescription(&mut conn, &value)?;
    desktop_local::update_local_prescription(
        &mut conn,
        &prescription.id,
        diagnosis,
        validity,
        instructions,
        &profile.profile.id,
    )?;
    let mut updated = prescription.clone();
    updated.diagnosis = {
        let trimmed = diagnosis.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_owned())
        }
    };
    updated.validity = {
        let trimmed = validity.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_owned())
        }
    };
    updated.instructions = {
        let trimmed = instructions.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_owned())
        }
    };
    Ok(updated)
}

#[cfg(feature = "desktop")]
fn upsert_prescription_item_offline(
    prescription: &api::Prescription,
    profile: &Me,
    form: &api::PrescriptionItemForm,
) -> Result<(), String> {
    let mut conn = desktop_local::open_local_db()?;
    let value = serde_json::json!({
        "id": prescription.id,
        "patient_id": prescription.patient_id,
        "doctor_id": prescription.doctor_id,
        "clinic_id": prescription.clinic_id,
        "template_version_id": prescription.template_version_id,
        "status": prescription.status,
        "rx_number": prescription.rx_number,
        "rx_block_id": prescription.rx_block_id,
        "diagnosis": prescription.diagnosis,
        "validity": prescription.validity,
        "instructions": prescription.instructions,
        "signed_at": prescription.signed_at,
        "synced_at": Value::Null,
        "voided_at": prescription.voided_at,
        "void_reason": prescription.void_reason,
        "client_id": prescription.client_id,
        "supersedes_id": prescription.supersedes_id,
        "created_at": prescription.created_at,
        "updated_at": prescription.updated_at,
        "created_by": prescription.created_by,
    });
    desktop_local::cache_prescription(&mut conn, &value)?;
    desktop_local::upsert_local_prescription_item(
        &mut conn,
        &prescription.id,
        form.id.as_deref(),
        &form.med_name,
        &form.dose,
        &form.route,
        &form.frequency,
        &form.duration,
        &form.quantity,
        &form.notes,
        &profile.profile.id,
    )?;
    Ok(())
}

async fn sign_prescription_dispatch(
    id: &str,
    acknowledge_warnings: bool,
    fallback: Option<(api::Prescription, Me)>,
) -> Result<api::Prescription, String> {
    if http::is_server_reachable().await {
        return api::sign_prescription(id, acknowledge_warnings).await;
    }
    #[cfg(feature = "desktop")]
    {
        let Some((prescription, profile)) = fallback else {
            return Err("Sin conexión.".to_owned());
        };
        return sign_prescription_offline(&prescription, &profile);
    }
    #[cfg(not(feature = "desktop"))]
    {
        let _ = fallback;
        Err("Sin conexión.".to_owned())
    }
}
#[cfg(feature = "desktop")]
fn update_patient_offline(
    patient_id: &str,
    form: &api::PatientForm,
    profile: &Me,
) -> Result<api::Patient, String> {
    let mut conn = desktop_local::open_local_db()?;
    desktop_local::update_local_patient(
        &mut conn,
        patient_id,
        &serde_json::json!({
            "fullName": form.full_name,
            "nationalId": form.national_id,
            "birthDate": form.birth_date,
            "sex": form.sex,
            "phone": form.phone,
            "notes": form.notes,
        }),
        &profile.profile.id,
    )?;
    Ok(api::Patient {
        id: patient_id.to_owned(),
        full_name: form.full_name.clone(),
        national_id: (!form.national_id.is_empty()).then(|| form.national_id.clone()),
        birth_date: (!form.birth_date.is_empty()).then(|| form.birth_date.clone()),
        sex: (!form.sex.is_empty()).then(|| form.sex.clone()),
        phone: (!form.phone.is_empty()).then(|| form.phone.clone()),
        notes: (!form.notes.is_empty()).then(|| form.notes.clone()),
        created_at: String::new(),
        updated_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    })
}

async fn update_patient_dispatch(
    patient_id: &str,
    form: &api::PatientForm,
    fallback: Option<Me>,
) -> Result<api::Patient, String> {
    if http::is_server_reachable().await {
        return api::update_patient(patient_id, form).await;
    }
    #[cfg(feature = "desktop")]
    {
        let Some(profile) = fallback else {
            return Err("Sin conexión.".to_owned());
        };
        return update_patient_offline(patient_id, form, &profile);
    }
    #[cfg(not(feature = "desktop"))]
    {
        let _ = fallback;
        Err("Sin conexión.".to_owned())
    }
}

#[cfg(feature = "desktop")]
fn create_template_offline(
    name: &str,
    owner_type: &str,
    profile: &Me,
) -> Result<api::Template, String> {
    let owner_id = if owner_type == "clinic" {
        profile
            .clinic
            .as_ref()
            .map(|clinic| clinic.id.clone())
            .ok_or_else(|| "Sin clínica asignada.".to_owned())?
    } else {
        profile
            .doctor
            .as_ref()
            .map(|doctor| doctor.id.clone())
            .ok_or_else(|| "El perfil no tiene ficha de médico.".to_owned())?
    };
    let mut conn = desktop_local::open_local_db()?;
    let created = desktop_local::create_local_template(
        &mut conn,
        name,
        owner_type,
        &owner_id,
        &profile.profile.id,
    )?;
    Ok(api::Template {
        id: created.id,
        owner_type: owner_type.to_owned(),
        owner_id,
        name: name.trim().to_owned(),
        status: "draft".to_owned(),
        current_version_id: None,
        current_version: None,
        versions: Vec::new(),
        created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        updated_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    })
}

async fn create_template_dispatch(
    name: &str,
    owner_type: &str,
    fallback: Option<Me>,
) -> Result<api::Template, String> {
    if http::is_server_reachable().await {
        return api::create_template(name, owner_type).await;
    }
    #[cfg(feature = "desktop")]
    {
        let Some(profile) = fallback else {
            return Err("Sin conexión.".to_owned());
        };
        return create_template_offline(name, owner_type, &profile);
    }
    #[cfg(not(feature = "desktop"))]
    {
        let _ = fallback;
        Err("Sin conexión.".to_owned())
    }
}

#[cfg(feature = "desktop")]
fn publish_template_offline(
    template: &api::Template,
    version_id: &str,
    profile: &Me,
) -> Result<api::Template, String> {
    let mut conn = desktop_local::open_local_db()?;
    desktop_local::publish_local_template(
        &mut conn,
        &template.id,
        version_id,
        &profile.profile.id,
    )?;
    let mut published = template.clone();
    published.status = "published".to_owned();
    published.current_version_id = Some(version_id.to_owned());
    Ok(published)
}

async fn publish_template_dispatch(
    template_id: &str,
    version_id: &str,
    fallback: Option<(api::Template, Me)>,
) -> Result<api::Template, String> {
    if http::is_server_reachable().await {
        return api::publish_template(template_id, version_id).await;
    }
    #[cfg(feature = "desktop")]
    {
        let Some((template, profile)) = fallback else {
            return Err("Sin conexión.".to_owned());
        };
        return publish_template_offline(&template, version_id, &profile);
    }
    #[cfg(not(feature = "desktop"))]
    {
        let _ = fallback;
        Err("Sin conexión.".to_owned())
    }
}

#[cfg(feature = "desktop")]
fn void_prescription_offline(
    prescription: &api::Prescription,
    profile: &Me,
    reason: &str,
) -> Result<api::Prescription, String> {
    let mut conn = desktop_local::open_local_db()?;
    let value = serde_json::json!({
        "id": prescription.id,
        "patient_id": prescription.patient_id,
        "doctor_id": prescription.doctor_id,
        "clinic_id": prescription.clinic_id,
        "template_version_id": prescription.template_version_id,
        "status": prescription.status,
        "rx_number": prescription.rx_number,
        "rx_block_id": prescription.rx_block_id,
        "diagnosis": prescription.diagnosis,
        "validity": prescription.validity,
        "instructions": prescription.instructions,
        "signed_at": prescription.signed_at,
        "synced_at": Value::Null,
        "voided_at": prescription.voided_at,
        "void_reason": prescription.void_reason,
        "client_id": prescription.client_id,
        "supersedes_id": prescription.supersedes_id,
        "created_at": prescription.created_at,
        "updated_at": prescription.updated_at,
        "created_by": prescription.created_by,
    });
    desktop_local::cache_prescription(&mut conn, &value)?;
    desktop_local::offline_void(&mut conn, &prescription.id, reason, &profile.profile.id)?;
    let mut voided = prescription.clone();
    voided.status = "voided".to_owned();
    voided.voided_at = Some(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
    voided.void_reason = Some(reason.trim().to_owned());
    Ok(voided)
}

async fn void_prescription_dispatch(
    id: &str,
    reason: &str,
    fallback: Option<(api::Prescription, Me)>,
) -> Result<api::Prescription, String> {
    if http::is_server_reachable().await {
        return api::void_prescription(id, reason).await;
    }
    #[cfg(feature = "desktop")]
    {
        let Some((prescription, profile)) = fallback else {
            return Err("Sin conexión.".to_owned());
        };
        return void_prescription_offline(&prescription, &profile, reason);
    }
    #[cfg(not(feature = "desktop"))]
    {
        let _ = fallback;
        Err("Sin conexión.".to_owned())
    }
}

#[cfg(feature = "desktop")]
fn supersede_prescription_offline(
    prescription: &api::Prescription,
    profile: &Me,
) -> Result<api::Prescription, String> {
    let mut conn = desktop_local::open_local_db()?;
    let value = serde_json::json!({
        "id": prescription.id,
        "patient_id": prescription.patient_id,
        "doctor_id": prescription.doctor_id,
        "clinic_id": prescription.clinic_id,
        "template_version_id": prescription.template_version_id,
        "status": prescription.status,
        "rx_number": prescription.rx_number,
        "rx_block_id": prescription.rx_block_id,
        "diagnosis": prescription.diagnosis,
        "validity": prescription.validity,
        "instructions": prescription.instructions,
        "signed_at": prescription.signed_at,
        "synced_at": Value::Null,
        "voided_at": prescription.voided_at,
        "void_reason": prescription.void_reason,
        "client_id": prescription.client_id,
        "supersedes_id": prescription.supersedes_id,
        "created_at": prescription.created_at,
        "updated_at": prescription.updated_at,
        "created_by": prescription.created_by,
    });
    desktop_local::cache_prescription(&mut conn, &value)?;
    let doctor_id = profile
        .doctor
        .as_ref()
        .map(|doctor| doctor.id.clone())
        .or_else(|| prescription.doctor_id.clone())
        .ok_or_else(|| "El perfil no tiene ficha de médico.".to_owned())?;
    let actor_id = profile.profile.id.clone();
    let device_id = crate::device::device_id();
    let result = desktop_local::offline_supersede(
        &mut conn,
        &prescription.id,
        &device_id,
        &doctor_id,
        &actor_id,
    )?;
    let mut corrected = prescription.clone();
    corrected.id = result.id;
    corrected.status = "signed".to_owned();
    corrected.rx_number = Some(result.rx_number as i32);
    corrected.rx_block_id = Some(result.rx_block_id);
    corrected.doctor_id = Some(doctor_id);
    corrected.client_id = corrected.id.clone();
    corrected.supersedes_id = Some(prescription.id.clone());
    corrected.signed_at =
        Some(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
    Ok(corrected)
}

async fn supersede_prescription_dispatch(
    id: &str,
    fallback: Option<(api::Prescription, Me)>,
) -> Result<api::Prescription, String> {
    if http::is_server_reachable().await {
        return api::supersede_prescription(id).await;
    }
    #[cfg(feature = "desktop")]
    {
        let Some((prescription, profile)) = fallback else {
            return Err("Sin conexión.".to_owned());
        };
        return supersede_prescription_offline(&prescription, &profile);
    }
    #[cfg(not(feature = "desktop"))]
    {
        let _ = fallback;
        Err("Sin conexión.".to_owned())
    }
}
#[component]
fn App() -> Element {
    let mut user = use_signal(|| None::<SessionUser>);
    let mut actor = use_signal(|| None::<Me>);
    let mut checked = use_signal(|| false);
    let mut sync = use_signal(|| SyncStatus {
        online: true,
        pending: 0,
        failed: 0,
    });
    use_context_provider(|| user);
    use_context_provider(|| actor);
    use_context_provider(|| checked);
    use_context_provider(|| sync);

    use_effect(move || {
        spawn(async move {
            let persisted = restore_local_session();
            let reachable = http::is_server_reachable().await;
            let (session, profile) = if reachable {
                let live = auth::fetch_session().await;
                match live {
                    Some(found) => {
                        let _ = api::register_current_device().await;
                        let live_profile = api::fetch_me().await.ok();
                        if let Some(live_profile) = &live_profile {
                            populate_local_cache(live_profile);
                        }
                        #[cfg(feature = "desktop")]
                        refresh_local_profile(&found, live_profile.as_ref());
                        (Some(found), live_profile)
                    }
                    None => {
                        #[cfg(feature = "desktop")]
                        clear_persisted_session();
                        (None, None)
                    }
                }
            } else {
                match persisted {
                    Some((found, found_profile)) => (Some(found), found_profile),
                    None => (None, None),
                }
            };
            let status = probe_sync_status().await;
            sync.set(status);
            actor.set(profile);
            user.set(session);
            checked.set(true);
        });
    });

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        div {
            id: "hero",
            h1 { "PersonaSanus" }
            p { "Your prescriptions and health records, kept organized." }
            div {
                id: "links",
                Link { to: Route::AuthPage {}, "Sign in" }
                Link { to: Route::Dashboard {}, "Dashboard" }
            }
        }
    }
}
#[component]
fn Verify(code: String) -> Element {
    let navigator = use_navigator();
    let mut result = use_signal(|| None::<api::VerificationResult>);
    let mut message = use_signal(String::new);
    let mut loaded = use_signal(|| false);
    let mut typed = use_signal(String::new);
    let requested = use_signal(|| code.clone());

    use_effect(move || {
        let load_code = requested();
        spawn(async move {
            match api::fetch_verification(&load_code).await {
                Ok(found) => result.set(Some(found)),
                Err(error) => message.set(error),
            }
            loaded.set(true);
        });
    });

    let title = match result().as_ref().map(|item| item.status.as_str()) {
        Some("valid") => "Receta auténtica",
        Some("voided") => "Receta anulada",
        Some("superseded") => "Receta reemplazada",
        Some("not found") => "Receta no encontrada",
        _ => "Verificación de receta",
    };
    let doctor = result()
        .as_ref()
        .and_then(|item| item.doctor_name.clone())
        .unwrap_or_else(|| "—".to_owned());
    let date = result()
        .as_ref()
        .and_then(|item| item.issued_at.clone())
        .map(|value| format_date(&value))
        .unwrap_or_else(|| "—".to_owned());
    let number = result()
        .as_ref()
        .and_then(|item| item.rx_number)
        .map(|value| format!("{value:06}"))
        .unwrap_or_else(|| "—".to_owned());

    rsx! {
        div {
            id: "verify-page",
            h1 { "{title}" }
            if !loaded() {
                p { "Cargando..." }
            } else if result().is_none() && !message().is_empty() {
                p { class: "error", "{message}" }
            } else {
                div {
                    class: "detail-grid",
                    div { class: "field", span { "Doctor" }, span { "{doctor}" } }
                    div { class: "field", span { "Fecha" }, span { "{date}" } }
                    div { class: "field", span { "No." }, span { "{number}" } }
                }
            }
            form {
                onsubmit: move |event| {
                    event.prevent_default();
                    let value = typed().trim().to_uppercase();
                    if !value.is_empty() {
                        navigator.push(Route::Verify { code: value });
                    }
                },
                div {
                    class: "field",
                    span { "Código de verificación" }
                    input {
                        value: typed(),
                        oninput: move |event| typed.set(event.value()),
                    }
                }
                button { class: "button", type: "submit", "Verificar" }
            }
        }
    }
}

#[component]
fn Navbar() -> Element {
    let mut user = use_context::<Signal<Option<SessionUser>>>();
    let mut actor = use_context::<Signal<Option<Me>>>();
    let mut sync = use_context::<Signal<SyncStatus>>();
    let navigator = use_navigator();
    let current = user();
    let profile = actor();
    let status = sync();
    let router = router();
    use_effect(move || {
        let _route: Route = router.current();
        spawn(async move {
            let current_status = probe_sync_status().await;
            sync.set(current_status);
        });
    });

    rsx! {
        div {
            id: "navbar",
            Link { to: Route::Home {}, "Home" }
            if current.is_some() {
                Link { to: Route::Dashboard {}, "Dashboard" }
                if profile
                    .as_ref()
                    .is_some_and(|actor| actor.can_read_patients())
                {
                    Link { to: Route::Patients {}, "Pacientes" }
                }
                if profile
                    .as_ref()
                    .is_some_and(|actor| actor.can_read_prescriptions())
                {
                    Link { to: Route::Prescriptions { status: None }, "Recetas" }
                }
                if profile
                    .as_ref()
                    .is_some_and(|actor| actor.can_manage_templates())
                {
                    Link { to: Route::Templates {}, "Plantillas" }
                }
                Link { to: Route::SettingsProfile {}, "Mi perfil" }
                if profile.as_ref().is_some_and(|actor| actor.is_administrator()) {
                    Link { to: Route::SettingsUsers {}, "Usuarios" }
                    Link { to: Route::Audit {}, "Auditoría" }
                }
                Link { to: Route::Settings {}, "Configuración" }
                span {
                    class: "sync-status",
                    "{sync_status_label(&status)}"
                    if status.pending > 0 {
                        " · {status.pending} pendientes"
                    }
                }
                if status.failed > 0 {
                    Link {
                        class: "sync-link",
                        to: Route::Prescriptions { status: Some("failed".to_owned()) },
                        "{status.failed} con error"
                    }
                }
            }
            if let Some(session) = current {
                span { "{session.email}" }
                button {
                    onclick: move |_| {
                        spawn(async move {
                            let _ = auth::sign_out().await;
                            clear_persisted_session();
                            actor.set(None);
                            user.set(None);
                            navigator.push(Route::Home {});
                        });
                    },
                    "Sign out"
                }
            } else {
                Link { to: Route::AuthPage {}, "Sign in" }
            }
        }
        Outlet::<Route> {}
    }
}

#[component]
fn AuthPage() -> Element {
    let mut user = use_context::<Signal<Option<SessionUser>>>();
    let mut actor = use_context::<Signal<Option<Me>>>();
    let mut checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();
    let mut sign_up_mode = use_signal(|| false);
    let mut name = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(String::new);
    let mut submitting = use_signal(|| false);
    let mut mfa_ticket = use_signal(String::new);
    let mut mfa_code = use_signal(String::new);
    let current = user();

    if let Some(session) = current {
        return rsx! {
            div {
                id: "echo",
                h4 { "Signed in" }
                p { "{session.name}" }
                p { "{session.email}" }
                button {
                    onclick: move |_| {
                        spawn(async move {
                            let _ = auth::sign_out().await;
                            clear_persisted_session();
                            actor.set(None);
                            user.set(None);
                            navigator.push(Route::Home {});
                        });
                    },
                    "Sign out"
                }
            }
        };
    }

    let submit = move |_| {
        if submitting() {
            return;
        }
        let name_value = name().trim().to_string();
        let email_value = email().trim().to_string();
        let password_value = password();
        if email_value.is_empty()
            || password_value.is_empty()
            || (sign_up_mode() && name_value.is_empty())
        {
            error.set("Fill in every field.".to_string());
            return;
        }
        error.set(String::new());
        submitting.set(true);
        spawn(async move {
            let result = if sign_up_mode() {
                auth::sign_up(&name_value, &email_value, &password_value).await
            } else {
                auth::sign_in(&email_value, &password_value).await
            };
            match result {
                Ok(value) => {
                    if auth::is_mfa_required(&value) {
                        if let Some(ticket) = auth::mfa_ticket(&value) {
                            mfa_ticket.set(ticket);
                        }
                        submitting.set(false);
                        error.set("Ingresa el código de verificación.".to_string());
                        return;
                    }
                    #[cfg(feature = "desktop")]
                    let token = http::current_session_token();
                    #[cfg(not(feature = "desktop"))]
                    let token: Option<String> = None;
                    let from_response = auth::parse_session_user(&value);
                    let refreshed = auth::fetch_session().await;
                    let session = refreshed.or(from_response);
                    let profile = match session {
                        Some(_) => {
                            let _ = api::register_current_device().await;
                            api::fetch_me().await.ok()
                        }
                        None => None,
                    };
                    if let Some(found) = &session {
                        #[cfg(feature = "desktop")]
                        persist_local_session(token.clone(), found, profile.as_ref());
                        #[cfg(not(feature = "desktop"))]
                        {
                            let _ = found;
                            let _ = &token;
                        }
                    }
                    actor.set(profile);
                    user.set(session.clone());
                    checked.set(true);
                    submitting.set(false);
                    if session.is_some() {
                        navigator.push(Route::Dashboard {});
                    } else {
                        error.set("The auth server did not return an active session.".to_string());
                    }
                }
                Err(message) => {
                    submitting.set(false);
                    error.set(message);
                }
            }
        });
    };

    let verify_mfa = move |_| {
        if submitting() {
            return;
        }
        let ticket = mfa_ticket();
        let code = mfa_code().trim().to_string();
        if ticket.is_empty() || code.is_empty() {
            error.set("Ingresa el código de verificación.".to_string());
            return;
        }
        error.set(String::new());
        submitting.set(true);
        spawn(async move {
            match auth::verify_mfa(&ticket, &code).await {
                Ok(value) => {
                    #[cfg(feature = "desktop")]
                    let token = http::current_session_token();
                    #[cfg(not(feature = "desktop"))]
                    let token: Option<String> = None;
                    let from_response = auth::parse_session_user(&value);
                    let refreshed = auth::fetch_session().await;
                    let session = refreshed.or(from_response);
                    let profile = match session {
                        Some(_) => {
                            let _ = api::register_current_device().await;
                            api::fetch_me().await.ok()
                        }
                        None => None,
                    };
                    if let Some(found) = &session {
                        #[cfg(feature = "desktop")]
                        persist_local_session(token.clone(), found, profile.as_ref());
                        #[cfg(not(feature = "desktop"))]
                        {
                            let _ = found;
                            let _ = &token;
                        }
                    }
                    actor.set(profile);
                    user.set(session.clone());
                    checked.set(true);
                    submitting.set(false);
                    if session.is_some() {
                        navigator.push(Route::Dashboard {});
                    } else {
                        error.set("The auth server did not return an active session.".to_string());
                    }
                }
                Err(message) => {
                    submitting.set(false);
                    error.set(message);
                }
            }
        });
    };

    rsx! {
        div {
            id: "echo",
            h4 {
                if sign_up_mode() { "Create your account" } else { "Welcome back" }
            }
            div {
                button {
                    onclick: move |_| {
                        sign_up_mode.set(false);
                        error.set(String::new());
                    },
                    "Sign in"
                }
                button {
                    onclick: move |_| {
                        sign_up_mode.set(true);
                        error.set(String::new());
                    },
                    "Sign up"
                }
            }
            if sign_up_mode() {
                input {
                    placeholder: "Name",
                    value: "{name}",
                    oninput: move |event| name.set(event.value()),
                }
            }
            input {
                r#type: "email",
                placeholder: "Email",
                value: "{email}",
                oninput: move |event| email.set(event.value()),
            }
            input {
                r#type: "password",
                placeholder: "Password",
                value: "{password}",
                oninput: move |event| password.set(event.value()),
            }
            if !mfa_ticket().is_empty() {
                input {
                    placeholder: "Código de verificación",
                    value: "{mfa_code}",
                    oninput: move |event| mfa_code.set(event.value()),
                }
                button {
                    disabled: submitting(),
                    onclick: verify_mfa,
                    "Verificar código"
                }
            }
            if !error().is_empty() {
                p { "{error}" }
            }
            button {
                disabled: submitting(),
                onclick: submit,
                if submitting() {
                    "Please wait..."
                } else if sign_up_mode() {
                    "Sign up"
                } else {
                    "Sign in"
                }
            }
        }
    }
}

#[component]
fn Dashboard() -> Element {
    let mut user = use_context::<Signal<Option<SessionUser>>>();
    let mut actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();

    use_effect(move || {
        if checked() && user().is_none() {
            navigator.replace(Route::AuthPage {});
        }
    });

    if !checked() {
        return rsx! {
            div {
                id: "echo",
                p { "Loading..." }
            }
        };
    }

    match user() {
        Some(session) => rsx! {
            div {
                id: "echo",
                h4 { "Dashboard" }
                p { "{session.name}" }
                p { "{session.email}" }
                if let Some(profile) = actor() {
                    p { "Rol: {role_label(profile.role.as_deref())}" }
                    if let Some(clinic) = profile.clinic {
                        p { "Clínica: {clinic.name}" }
                    } else {
                        p { "Sin clínica asignada." }
                    }
                }
                button {
                    onclick: move |_| {
                        spawn(async move {
                            let _ = auth::sign_out().await;
                            clear_persisted_session();
                            actor.set(None);
                            user.set(None);
                            navigator.push(Route::Home {});
                        });
                    },
                    "Sign out"
                }
            }
        },
        None => rsx! {
            div {
                id: "echo",
                p { "Redirecting to sign in..." }
            }
        },
    }
}

#[component]
fn Settings() -> Element {
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();

    use_effect(move || {
        if checked() {
            match actor() {
                None => {
                    navigator.replace(Route::AuthPage {});
                }
                Some(profile) if !profile.is_administrator() => {
                    navigator.replace(Route::SettingsProfile {});
                }
                _ => {}
            }
        }
    });

    let profile = actor();
    if !checked() || profile.is_none() {
        return rsx! {
            div { id: "echo", p { "Cargando..." } }
        };
    }
    match profile {
        Some(profile) if profile.is_administrator() => rsx! {
            div {
                id: "echo",
                h4 { "Configuración" }
                p { "Administra el acceso del personal y tu perfil." }
                Link { to: Route::SettingsUsers {}, "Usuarios y roles" }
                Link { to: Route::SettingsClinic {}, "Clínica y biblioteca" }
                Link { to: Route::SettingsProfile {}, "Mi perfil" }
            }
        },
        _ => rsx! {
            div { id: "echo", p { "Redirigiendo..." } }
        },
    }
}

#[component]
fn SettingsClinic() -> Element {
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();
    let mut name = use_signal(String::new);
    let mut address = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut llm_url = use_signal(String::new);
    let mut message = use_signal(String::new);
    let mut saving = use_signal(|| false);
    let mut uploading = use_signal(|| false);
    let mut generation = use_signal(|| 0_u32);
    let mut settings = use_signal(|| None::<api::ClinicSettings>);
    let mut library = use_signal(|| None::<api::AssetLibrary>);

    use_effect(move || {
        let _ = generation();
        if !checked() {
            return;
        }
        match actor() {
            None => {
                navigator.replace(Route::AuthPage {});
                return;
            }
            Some(profile) if !profile.is_administrator() => {
                navigator.replace(Route::SettingsProfile {});
                return;
            }
            _ => {}
        }
        spawn(async move {
            match api::fetch_clinic_settings().await {
                Ok(found) => {
                    name.set(found.name.clone());
                    address.set(found.address.clone().unwrap_or_default());
                    phone.set(found.phone.clone().unwrap_or_default());
                    llm_url.set(found.llm_url_template.clone().unwrap_or_default());
                    settings.set(Some(found));
                }
                Err(error) => message.set(error),
            }
            match api::fetch_assets().await {
                Ok(loaded) => library.set(Some(loaded)),
                Err(error) => message.set(error),
            }
        });
    });

    if !checked() || actor().is_none() {
        return rsx! {
            div { id: "settings-page", p { "Cargando..." } }
        };
    }
    if let Some(profile) = actor() {
        if !profile.is_administrator() {
            return rsx! {
                div { id: "settings-page", p { "Redirigiendo..." } }
            };
        }
    }

    let clinic = settings();
    let logo_url = clinic
        .as_ref()
        .and_then(|clinic| clinic.logo.as_ref())
        .and_then(|logo| logo.url.clone());
    let library_state = library();
    let quota = library_state
        .as_ref()
        .map(|library| (library.quota.used, library.quota.limit));
    let library_assets: Vec<api::AssetRow> = library_state
        .as_ref()
        .map(|library| {
            library
                .assets
                .iter()
                .filter(|asset| asset.kind == "image")
                .cloned()
                .collect()
        })
        .unwrap_or_default();

    rsx! {
        div {
            id: "settings-page",
            div {
                class: "page-head",
                div {
                    h4 { "Clínica" }
                    p { "Datos de la clínica, logo y biblioteca de imágenes." }
                }
                Link { class: "button secondary", to: Route::Settings {}, "Volver" }
            }
            if !message().is_empty() {
                p { class: "error", "{message}" }
            }
            section {
                class: "editor-section",
                h4 { "Datos de la clínica" }
                div {
                    class: "form-grid",
                    label {
                        class: "field",
                        span { "Nombre" }
                        input {
                            value: "{name}",
                            oninput: move |event| name.set(event.value()),
                        }
                    }
                    label {
                        class: "field",
                        span { "Dirección" }
                        input {
                            value: "{address}",
                            oninput: move |event| address.set(event.value()),
                        }
                    }
                    label {
                        class: "field",
                        span { "Teléfono" }
                        input {
                            value: "{phone}",
                            oninput: move |event| phone.set(event.value()),
                        }
                    }
                    label {
                        class: "field wide",
                        span { "Plantilla del enlace LLM" }
                        input {
                            placeholder: "https://chat.openai.com/?q=",
                            value: "{llm_url}",
                            oninput: move |event| llm_url.set(event.value()),
                        }
                    }
                }
                div {
                    class: "actions",
                    button {
                        disabled: saving(),
                        onclick: move |_| {
                            if saving() {
                                return;
                            }
                            saving.set(true);
                            let value = name();
                            let street = address();
                            let telephone = phone();
                            let llm = llm_url();
                            spawn(async move {
                                match api::update_clinic_settings(&value, &street, &telephone, &llm).await {
                                    Ok(_) => {
                                        message.set("Datos de la clínica actualizados.".to_owned());
                                        generation += 1;
                                    }
                                    Err(error) => message.set(error),
                                }
                                saving.set(false);
                            });
                        },
                        if saving() { "Guardando..." } else { "Guardar datos" }
                    }
                }
            }
            section {
                class: "editor-section",
                h4 { "Logo" }
                if let Some(url) = logo_url {
                    img { class: "clinic-logo", src: "{url}", alt: "Logo de la clínica" }
                } else {
                    p { class: "hint", "La clínica todavía no tiene logo." }
                }
                label {
                    class: "field",
                    span { "Subir logo (JPEG o PNG, máximo 10 MB)" }
                    input {
                        r#type: "file",
                        accept: "image/jpeg,image/png",
                        disabled: uploading(),
                        onchange: move |event| {
                            let Some(file) = event.files().first().cloned() else {
                                return;
                            };
                            if uploading() {
                                return;
                            }
                            uploading.set(true);
                            let file_name = file.name();
                            spawn(async move {
                                if !http::is_server_reachable().await {
                                    message.set("Subir imágenes requiere conexión.".to_owned());
                                    uploading.set(false);
                                    return;
                                }
                                match file.read_bytes().await {
                                    Ok(bytes) => {
                                        match api::upload_logo(file_name, bytes.to_vec()).await {
                                            Ok(_) => {
                                                message.set("Logo actualizado.".to_owned());
                                                generation += 1;
                                            }
                                            Err(error) => message.set(error),
                                        }
                                    }
                                    Err(_) => message.set("No se pudo leer el archivo.".to_owned()),
                                }
                                uploading.set(false);
                            });
                        },
                    }
                }
            }
            section {
                class: "editor-section",
                h4 { "Biblioteca de imágenes" }
                if let Some((used, limit)) = quota {
                    p { class: "hint", "Uso de la biblioteca: {used} de {limit}." }
                }
                for asset in library_assets.iter() {
                    div {
                        key: "{asset.id}",
                        class: "asset-row",
                        div {
                            strong {
                                "{asset.original_name.clone().unwrap_or_else(|| asset.id.clone())}"
                            }
                            p {
                                class: "hint",
                                "{asset.width.unwrap_or(0)} × {asset.height.unwrap_or(0)} · {asset.size_bytes} bytes · sha256 {asset.sha256.chars().take(12).collect::<String>()}"
                            }
                        }
                        button {
                            class: "danger",
                            onclick: {
                                let asset_id = asset.id.clone();
                                move |_| {
                                    let asset_id = asset_id.clone();
                                    spawn(async move {
                                        match api::delete_asset(&asset_id).await {
                                            Ok(()) => {
                                                message.set("Imagen eliminada.".to_owned());
                                                generation += 1;
                                            }
                                            Err(error) => message.set(error),
                                        }
                                    });
                                }
                            },
                            "Eliminar"
                        }
                    }
                }
                label {
                    class: "field",
                    span { "Subir imagen de la biblioteca (JPEG o PNG, máximo 10 MB)" }
                    input {
                        r#type: "file",
                        accept: "image/jpeg,image/png",
                        disabled: uploading(),
                        onchange: move |event| {
                            let Some(file) = event.files().first().cloned() else {
                                return;
                            };
                            if uploading() {
                                return;
                            }
                            uploading.set(true);
                            let file_name = file.name();
                            spawn(async move {
                                if !http::is_server_reachable().await {
                                    message.set("Subir imágenes requiere conexión.".to_owned());
                                    uploading.set(false);
                                    return;
                                }
                                match file.read_bytes().await {
                                    Ok(bytes) => {
                                        match api::upload_asset(file_name, bytes.to_vec()).await {
                                            Ok(_) => {
                                                message.set("Imagen agregada a la biblioteca.".to_owned());
                                                generation += 1;
                                            }
                                            Err(error) => message.set(error),
                                        }
                                    }
                                    Err(_) => message.set("No se pudo leer el archivo.".to_owned()),
                                }
                                uploading.set(false);
                            });
                        },
                    }
                }
            }
            section {
                class: "editor-section",
                h4 { "Exportación de datos" }
                if let Some(retention_until) = clinic.as_ref().and_then(|value| value.server_retention_until.clone()) {
                    p { class: "hint", "Retención en servidor hasta: {retention_until}" }
                }
                if let Some(grace) = clinic.as_ref().and_then(|value| value.data_grace_ends_at.clone()) {
                    p { class: "hint", "Fin del período de gracia para exportación: {grace}" }
                }
                p { class: "hint", "Exporte los datos de la clínica (JSON: pacientes, recetas y medicamentos) antes de que finalice el período de gracia. La exportación queda registrada en la auditoría." }
                div {
                    class: "actions",
                    a {
                        class: "button secondary",
                        href: "/api/clinic/export",
                        "Exportar datos (JSON)"
                    }
                }
            }
        }
    }
}

#[component]
fn SettingsUsers() -> Element {
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();
    let mut users = use_signal(Vec::<api::UserRow>::new);
    let mut devices = use_signal(Vec::<api::DeviceRow>::new);
    let refresh = use_signal(|| 0);
    let mut message = use_signal(String::new);

    use_effect(move || {
        let _ = refresh();
        if !checked() {
            return;
        }
        let Some(profile) = actor() else {
            navigator.replace(Route::AuthPage {});
            return;
        };
        if !profile.is_administrator() {
            return;
        }
        spawn(async move {
            match api::fetch_users().await {
                Ok(rows) => users.set(rows),
                Err(error) => message.set(error),
            }
            match api::fetch_devices().await {
                Ok(rows) => devices.set(rows),
                Err(error) => message.set(error),
            }
        });
    });

    let profile = actor();
    if !checked() {
        return rsx! {
            div { id: "echo", p { "Cargando..." } }
        };
    }
    if !profile
        .as_ref()
        .is_some_and(|profile| profile.is_administrator())
    {
        return rsx! {
            div {
                id: "echo",
                h4 { "Acceso denegado" }
                p { "Solo el administrador puede gestionar usuarios y roles." }
                Link { to: Route::SettingsProfile {}, "Ir a mi perfil" }
            }
        };
    }

    let rows = users();
    let device_rows = devices();
    rsx! {
        div {
            id: "echo",
            h4 { "Usuarios y roles" }
            p { "Asigna un rol o deshabilita el acceso de cada persona." }
            if !message().is_empty() {
                p { "{message}" }
            }
            for row in rows {
                div {
                    key: "{row.profile_id}",
                    p { "{row.display_name}" }
                    p { "{row.email}" }
                    p { "Alta: {format_date(&row.created_at)}" }
                    p { "Estado: {status_label(&row.status)}" }
                    select {
                        value: row.role.clone().unwrap_or_default(),
                        onchange: {
                            let profile_id = row.profile_id.clone();
                            let mut refresh = refresh;
                            let mut message = message;
                            move |event| {
                                let value = event.value();
                                if value.is_empty() {
                                    return;
                                }
                                let profile_id = profile_id.clone();
                                spawn(async move {
                                    match api::set_role(&profile_id, &value).await {
                                        Ok(()) => refresh += 1,
                                        Err(error) => message.set(error),
                                    }
                                });
                            }
                        },
                        option { value: "", disabled: true, "Sin rol" }
                        option { value: "administrator", "Administrador" }
                        option { value: "doctor", "Doctor" }
                        option { value: "nursing", "Enfermería" }
                        option { value: "secretary", "Secretaría" }
                    }
                    if row.role.is_some() {
                        button {
                            onclick: {
                                let profile_id = row.profile_id.clone();
                                let mut refresh = refresh;
                                let mut message = message;
                                move |_| {
                                    let profile_id = profile_id.clone();
                                    spawn(async move {
                                        match api::remove_member(&profile_id).await {
                                            Ok(()) => refresh += 1,
                                            Err(error) => message.set(error),
                                        }
                                    });
                                }
                            },
                            "Quitar del equipo"
                        }
                    }
                    if row.status == "active" {
                        button {
                            onclick: {
                                let profile_id = row.profile_id.clone();
                                let mut refresh = refresh;
                                let mut message = message;
                                move |_| {
                                    let profile_id = profile_id.clone();
                                    spawn(async move {
                                        match api::disable_profile(&profile_id, true).await {
                                            Ok(()) => refresh += 1,
                                            Err(error) => message.set(error),
                                        }
                                    });
                                }
                            },
                            "Deshabilitar"
                        }
                    } else {
                        button {
                            onclick: {
                                let profile_id = row.profile_id.clone();
                                let mut refresh = refresh;
                                let mut message = message;
                                move |_| {
                                    let profile_id = profile_id.clone();
                                    spawn(async move {
                                        match api::disable_profile(&profile_id, false).await {
                                            Ok(()) => refresh += 1,
                                            Err(error) => message.set(error),
                                        }
                                    });
                                }
                            },
                            "Habilitar"
                        }
                    }
                }
            }
            if !device_rows.is_empty() {
                div {
                    h4 { "Dispositivos" }
                    p { "Equipos registrados y su último acceso." }
                    for device in device_rows {
                        div {
                            key: "{device.id}",
                            p { "{device.profile_name} · {device_name_label(&device)}" }
                            p {
                                "{platform_label(&device.platform)} · Última conexión: {last_seen_label(device.last_seen_at.as_deref())}"
                            }
                            if device.app_version.is_some() {
                                p { "Versión: {device.app_version.clone().unwrap_or_default()}" }
                            }
                            if device.revoked_at.is_some() {
                                p { "Estado: Revocado" }
                            } else {
                                button {
                                    class: "danger",
                                    onclick: {
                                        let device_id = device.id.clone();
                                        let mut refresh = refresh;
                                        let mut message = message;
                                        move |_| {
                                            let device_id = device_id.clone();
                                            spawn(async move {
                                                match api::revoke_device(&device_id).await {
                                                    Ok(()) => refresh += 1,
                                                    Err(error) => message.set(error),
                                                }
                                            });
                                        }
                                    },
                                    "Revocar"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SettingsProfile() -> Element {
    let mut actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();
    let mut initialized = use_signal(|| false);
    let mut display_name = use_signal(String::new);
    let mut specialty = use_signal(String::new);
    let mut license_no = use_signal(String::new);
    let mut national_id = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut message = use_signal(String::new);
    let mut saving = use_signal(|| false);
    let mut mfa_enabled = use_signal(|| false);
    let mut mfa_secret = use_signal(String::new);
    let mut mfa_uri = use_signal(String::new);
    let mut mfa_code = use_signal(String::new);
    let mut mfa_message = use_signal(String::new);
    let mut mfa_password = use_signal(String::new);
    let mut current_password = use_signal(String::new);
    let mut new_password = use_signal(String::new);
    let mut password_message = use_signal(String::new);

    use_effect(move || {
        if initialized() {
            return;
        }
        if !checked() {
            return;
        }
        let Some(profile) = actor() else {
            navigator.replace(Route::AuthPage {});
            return;
        };
        display_name.set(profile.profile.display_name);
        if let Some(doctor) = profile.doctor {
            specialty.set(doctor.specialty.unwrap_or_default());
            license_no.set(doctor.license_no.unwrap_or_default());
            national_id.set(doctor.national_id.unwrap_or_default());
            phone.set(doctor.phone.unwrap_or_default());
            email.set(doctor.email.unwrap_or_default());
        }
        spawn(async move {
            if let Ok(enabled) = auth::mfa_status().await {
                mfa_enabled.set(enabled);
            }
        });
        initialized.set(true);
    });

    let current = actor();
    if !checked() {
        return rsx! {
            div { id: "echo", p { "Cargando..." } }
        };
    }
    let Some(profile) = current else {
        return rsx! {
            div { id: "echo", p { "Redirigiendo..." } }
        };
    };
    let is_doctor = profile.is_doctor();
    let is_admin = profile.is_administrator();

    rsx! {
        div {
            id: "echo",
            h4 { "Mi perfil" }
            if let Some(clinic) = profile.clinic {
                p { "Clínica: {clinic.name}" }
            } else {
                p { "Sin clínica asignada. Un administrador debe asignarte un rol." }
            }
            p { "Rol: {role_label(profile.role.as_deref())}" }
            div {
                input {
                    placeholder: "Nombre visible",
                    value: "{display_name}",
                    oninput: move |event| display_name.set(event.value()),
                }
            }
            if is_doctor {
                div {
                    input {
                        placeholder: "Especialidad",
                        value: "{specialty}",
                        oninput: move |event| specialty.set(event.value()),
                    }
                    input {
                        placeholder: "Número de licencia",
                        value: "{license_no}",
                        oninput: move |event| license_no.set(event.value()),
                    }
                    input {
                        placeholder: "Cédula",
                        value: "{national_id}",
                        oninput: move |event| national_id.set(event.value()),
                    }
                    input {
                        placeholder: "Teléfono",
                        value: "{phone}",
                        oninput: move |event| phone.set(event.value()),
                    }
                    input {
                        placeholder: "Correo",
                        value: "{email}",
                        oninput: move |event| email.set(event.value()),
                    }
                }
            }
            if !message().is_empty() {
                p { "{message}" }
            }
            button {
                disabled: saving(),
                onclick: move |_| {
                    if saving() {
                        return;
                    }
                    saving.set(true);
                    message.set(String::new());
                    spawn(async move {
                        let result = api::update_profile(
                            &display_name(),
                            &specialty(),
                            &license_no(),
                            &national_id(),
                            &phone(),
                            &email(),
                        )
                        .await;
                        saving.set(false);
                        match result {
                            Ok(updated) => {
                                actor.set(Some(updated));
                                message.set("Perfil guardado.".to_string());
                            }
                            Err(error) => message.set(error),
                        }
                    });
                },
                if saving() { "Guardando..." } else { "Guardar" }
            }
            if is_doctor || is_admin {
                div {
                    h4 { "Verificación en dos pasos" }
                    p {
                        if mfa_enabled() {
                            "Activada. Se pedirá el código al iniciar sesión."
                        } else {
                            "Sin activar. Actívala para proteger tu cuenta."
                        }
                    }
                    if !mfa_secret().is_empty() {
                        p { "Secreto: {mfa_secret}" }
                        p { "URI: {mfa_uri}" }
                        input {
                            placeholder: "Código de 6 dígitos",
                            value: "{mfa_code}",
                            oninput: move |event| mfa_code.set(event.value()),
                        }
                        button {
                            onclick: move |_| {
                                spawn(async move {
                                    match auth::confirm_mfa(&mfa_code()).await {
                                        Ok(_) => {
                                            mfa_enabled.set(true);
                                            mfa_secret.set(String::new());
                                            mfa_message.set("Verificación activada.".to_string());
                                        }
                                        Err(error) => mfa_message.set(error),
                                    }
                                });
                            },
                            "Confirmar activación"
                        }
                    } else {
                        button {
                            onclick: move |_| {
                                spawn(async move {
                                    match auth::enroll_mfa().await {
                                        Ok(value) => {
                                            if let Some(secret) = http::string_field(&value, &["secret"]) {
                                                mfa_secret.set(secret);
                                            }
                                            if let Some(uri) = http::string_field(&value, &["uri"]) {
                                                mfa_uri.set(uri);
                                            }
                                            mfa_message.set("Escanea el URI o copia el secreto en tu app de autenticación.".to_string());
                                        }
                                        Err(error) => mfa_message.set(error),
                                    }
                                });
                            },
                            "Activar verificación"
                        }
                    }
                    if mfa_enabled() {
                        input {
                            r#type: "password",
                            placeholder: "Contraseña para desactivar",
                            value: "{mfa_password}",
                            oninput: move |event| mfa_password.set(event.value()),
                        }
                        button {
                            onclick: move |_| {
                                spawn(async move {
                                    match auth::disable_mfa(&mfa_password()).await {
                                        Ok(_) => {
                                            mfa_enabled.set(false);
                                            mfa_message.set("Verificación desactivada.".to_string());
                                        }
                                        Err(error) => mfa_message.set(error),
                                    }
                                });
                            },
                            "Desactivar verificación"
                        }
                    }
                    if !mfa_message().is_empty() {
                        p { "{mfa_message}" }
                    }
                }
            }
            div {
                h4 { "Cambiar contraseña" }
                p { "Al cambiarla se invalidan las demás sesiones." }
                input {
                    r#type: "password",
                    placeholder: "Contraseña actual",
                    value: "{current_password}",
                    oninput: move |event| current_password.set(event.value()),
                }
                input {
                    r#type: "password",
                    placeholder: "Nueva contraseña",
                    value: "{new_password}",
                    oninput: move |event| new_password.set(event.value()),
                }
                if !password_message().is_empty() {
                    p { "{password_message}" }
                }
                button {
                    onclick: move |_| {
                        spawn(async move {
                            match auth::change_password(&current_password(), &new_password()).await {
                                Ok(_) => {
                                    current_password.set(String::new());
                                    new_password.set(String::new());
                                    password_message.set("Contraseña cambiada. Sesión renovada.".to_string());
                                }
                                Err(error) => password_message.set(error),
                            }
                        });
                    },
                    "Cambiar contraseña"
                }
            }
        }
    }
}

#[component]
fn Audit() -> Element {
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();
    let mut action = use_signal(String::new);
    let mut entity = use_signal(String::new);
    let mut q = use_signal(String::new);
    let mut rows = use_signal(Vec::<api::AuditRow>::new);
    let mut message = use_signal(String::new);
    let mut generation = use_signal(|| 0_u32);

    use_effect(move || {
        let _ = generation();
        if !checked() {
            return;
        }
        match actor() {
            None => {
                navigator.replace(Route::AuthPage {});
                return;
            }
            Some(profile) if !profile.is_administrator() => {
                message.set(
                    "Acceso denegado. Solo el administrador puede ver la auditoría.".to_string(),
                );
                return;
            }
            _ => {}
        }
        let action_value = action();
        let entity_value = entity();
        let q_value = q();
        spawn(async move {
            match api::fetch_audit(&action_value, &entity_value, &q_value).await {
                Ok(found) => {
                    message.set(String::new());
                    rows.set(found);
                }
                Err(error) => message.set(error),
            }
        });
    });

    if !checked() {
        return rsx! {
            div { id: "audit-page", p { "Cargando..." } }
        };
    }
    let is_admin = actor().is_some_and(|profile| profile.is_administrator());
    if !is_admin {
        return rsx! {
            div {
                id: "audit-page",
                div { class: "notice",
                    h4 { "Acceso denegado" }
                    p { "{message}" }
                }
            }
        };
    }

    rsx! {
        div {
            id: "audit-page",
            div {
                class: "page-head",
                div {
                    h4 { "Auditoría" }
                    p { "Registro de solo anexación con retención de 5 años. Filtra por acción, entidad o texto." }
                }
            }
            div {
                input {
                    placeholder: "Acción (p. ej. sensitive_read)",
                    value: "{action}",
                    oninput: move |event| action.set(event.value()),
                }
                input {
                    placeholder: "Entidad (p. ej. patient)",
                    value: "{entity}",
                    oninput: move |event| entity.set(event.value()),
                }
                input {
                    placeholder: "Buscar en entidad o metadatos",
                    value: "{q}",
                    oninput: move |event| q.set(event.value()),
                }
                button {
                    onclick: move |_| generation.set(generation() + 1),
                    "Filtrar"
                }
            }
            if !message().is_empty() {
                p { "{message}" }
            }
            table {
                thead {
                    tr {
                        th { "Fecha" }
                        th { "Actor" }
                        th { "Acción" }
                        th { "Entidad" }
                        th { "ID entidad" }
                        th { "Metadatos" }
                    }
                }
                tbody {
                    for row in rows() {
                        tr {
                            td { "{row.at}" }
                            td { "{row.actor_name.clone().unwrap_or_else(|| row.actor_id.clone().unwrap_or_default())}" }
                            td { "{row.action}" }
                            td { "{row.entity}" }
                            td { "{row.entity_id}" }
                            td { "{row.metadata}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Patients() -> Element {
    let user = use_context::<Signal<Option<SessionUser>>>();
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();
    let mut query = use_signal(String::new);
    let mut patients = use_signal(Vec::<api::Patient>::new);
    let mut message = use_signal(String::new);

    use_effect(move || {
        if !checked() {
            return;
        }
        if user().is_none() {
            navigator.replace(Route::AuthPage {});
            return;
        }
        let term = query();
        spawn(async move {
            match api::fetch_patients(&term).await {
                Ok(found) => {
                    message.set(String::new());
                    patients.set(found);
                }
                Err(error) => {
                    message.set(error);
                    #[cfg(feature = "desktop")]
                    if let Ok(mut conn) = desktop_local::open_local_db() {
                        if let Ok(found) = desktop_local::list_patients(&mut conn) {
                            let mut mapped = Vec::new();
                            for value in found {
                                if let Ok(patient) = serde_json::from_value::<api::Patient>(value) {
                                    mapped.push(patient);
                                }
                            }
                            patients.set(mapped);
                        }
                    }
                }
            }
        });
    });

    if !checked() {
        return rsx! {
            div { id: "patients-page", p { "Cargando..." } }
        };
    }
    let Some(profile) = actor() else {
        return rsx! {
            div { id: "patients-page", p { "Redirigiendo..." } }
        };
    };
    if !profile.can_read_patients() {
        return rsx! {
            div {
                id: "patients-page",
                div { class: "notice",
                    h4 { "Acceso denegado" }
                    p { "Necesitas un rol en la clínica para consultar pacientes." }
                }
            }
        };
    }

    rsx! {
        div {
            id: "patients-page",
            div {
                class: "page-head",
                div {
                    h4 { "Pacientes" }
                    p { "Busca por nombre o cédula dentro de tu clínica." }
                }
                if profile.can_write_patients() {
                    Link { class: "button", to: Route::PatientNew {}, "Nuevo paciente" }
                }
            }
            div {
                class: "search-row",
                input {
                    r#type: "search",
                    placeholder: "Nombre o cédula",
                    value: "{query}",
                    oninput: move |event| query.set(event.value()),
                }
            }
            if !message().is_empty() {
                p { class: "error", "{message}" }
            }
            div {
                class: "patient-list",
                if patients().is_empty() {
                    p { class: "hint", "No hay pacientes para esta búsqueda." }
                }
                for patient in patients() {
                    div {
                        key: "{patient.id}",
                        class: "patient-row",
                        div {
                            p { class: "patient-name", "{patient.full_name}" }
                            p {
                                class: "patient-meta",
                                "Cédula: {patient.national_id.clone().unwrap_or_else(|| \"—\".to_string())}"
                            }
                            if let Some(birth_date) = patient.birth_date.clone() {
                                p { class: "patient-meta", "Nacimiento: {format_date(&birth_date)}" }
                            }
                        }
                        Link {
                            class: "button secondary",
                            to: Route::PatientDetail { id: patient.id.clone() },
                            "Ver ficha"
                        }
                    }
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn submit_new_patient(
    mut saving: Signal<bool>,
    mut message: Signal<String>,
    mut warning_ids: Signal<Vec<String>>,
    navigator: Navigator,
    sync: Signal<SyncStatus>,
    form: api::PatientForm,
    confirm: bool,
    profile: Me,
) {
    let _ = &profile;
    let _ = &sync;
    if saving() {
        return;
    }
    if form.full_name.trim().is_empty() {
        message.set("Ingresa el nombre del paciente.".to_string());
        return;
    }
    if !form.birth_date.is_empty() && !valid_birth_date(&form.birth_date) {
        message.set("Ingresa una fecha de nacimiento válida.".to_string());
        return;
    }
    saving.set(true);
    message.set(String::new());
    spawn(async move {
        if !http::is_server_reachable().await {
            #[cfg(feature = "desktop")]
            {
                saving.set(false);
                match desktop_local::open_local_db().and_then(|mut conn| {
                    desktop_local::create_local_patient(
                        &mut conn,
                        &serde_json::json!({
                            "clinicId": profile.clinic.as_ref().map(|clinic| clinic.id.clone()).unwrap_or_default(),
                            "fullName": form.full_name,
                            "nationalId": form.national_id,
                            "birthDate": form.birth_date,
                            "sex": form.sex,
                            "phone": form.phone,
                            "notes": form.notes,
                        }),
                        &profile.profile.id,
                    )
                }) {
                    Ok(created) => {
                        let mut sync = sync;
                        sync.set(probe_sync_status().await);
                        navigator.push(Route::PatientDetail { id: created.id });
                    }
                    Err(error) => message.set(error),
                }
                return;
            }
        }
        match api::create_patient(&form, confirm).await {
            Ok(api::PatientCreateOutcome::Created(patient)) => {
                saving.set(false);
                navigator.push(Route::PatientDetail { id: patient.id });
            }
            Ok(api::PatientCreateOutcome::LikelyDuplicate(ids)) => {
                saving.set(false);
                warning_ids.set(ids);
                message.set("Ya existe un paciente con un nombre similar.".to_string());
            }
            Err(error) => {
                saving.set(false);
                message.set(error);
            }
        }
    });
}

#[component]
fn PatientNew() -> Element {
    let user = use_context::<Signal<Option<SessionUser>>>();
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let sync = use_context::<Signal<SyncStatus>>();
    let navigator = use_navigator();
    let mut full_name = use_signal(String::new);
    let mut national_id = use_signal(String::new);
    let mut birth_date = use_signal(String::new);
    let mut sex = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut notes = use_signal(String::new);
    let warning_ids = use_signal(Vec::<String>::new);
    let message = use_signal(String::new);
    let saving = use_signal(|| false);

    use_effect(move || {
        if checked() && user().is_none() {
            navigator.replace(Route::AuthPage {});
        }
    });

    if !checked() {
        return rsx! {
            div { id: "patients-page", p { "Cargando..." } }
        };
    }
    let Some(profile) = actor() else {
        return rsx! {
            div { id: "patients-page", p { "Redirigiendo..." } }
        };
    };
    if !profile.can_write_patients() {
        return rsx! {
            div {
                id: "patients-page",
                div { class: "notice",
                    h4 { "Acceso denegado" }
                    p { "Solo doctor, enfermería y secretaría pueden crear pacientes." }
                    Link { to: Route::Patients {}, "Volver a pacientes" }
                }
            }
        };
    }

    let form = move || api::PatientForm {
        full_name: full_name(),
        national_id: national_id(),
        birth_date: birth_date(),
        sex: sex(),
        phone: phone(),
        notes: notes(),
    };

    let profile_for_duplicate = profile.clone();

    rsx! {
        div {
            id: "patients-page",
            div {
                class: "page-head",
                div {
                    h4 { "Nuevo paciente" }
                    p { "Los datos se guardan en la clínica actual." }
                }
                Link { class: "button secondary", to: Route::Patients {}, "Cancelar" }
            }
            div {
                class: "form-grid",
                label {
                    class: "field",
                    span { "Nombres y apellidos" }
                    input {
                        value: "{full_name}",
                        oninput: move |event| full_name.set(event.value()),
                    }
                }
                label {
                    class: "field",
                    span { "Cédula" }
                    input {
                        value: "{national_id}",
                        oninput: move |event| national_id.set(event.value()),
                    }
                }
                label {
                    class: "field",
                    span { "Fecha de nacimiento" }
                    input {
                        r#type: "date",
                        value: "{birth_date}",
                        oninput: move |event| birth_date.set(event.value()),
                    }
                }
                label {
                    class: "field",
                    span { "Sexo" }
                    select {
                        value: "{sex}",
                        onchange: move |event| sex.set(event.value()),
                        option { value: "", "Sin especificar" }
                        option { value: "female", "Femenino" }
                        option { value: "male", "Masculino" }
                        option { value: "other", "Otro" }
                        option { value: "unspecified", "Prefiere no decir" }
                    }
                }
                label {
                    class: "field",
                    span { "Teléfono" }
                    input {
                        r#type: "tel",
                        value: "{phone}",
                        oninput: move |event| phone.set(event.value()),
                    }
                }
                label {
                    class: "field wide",
                    span { "Notas" }
                    textarea {
                        value: "{notes}",
                        oninput: move |event| notes.set(event.value()),
                    }
                }
            }
            if !message().is_empty() {
                p { class: "error", "{message}" }
            }
            if !warning_ids().is_empty() {
                div {
                    class: "notice",
                    p { "Posible duplicado por nombre. Coincidencias: {warning_ids().join(\", \")}" }
                    button {
                        disabled: saving(),
                        onclick: move |_| {
                            submit_new_patient(
                                saving,
                                message,
                                warning_ids,
                                navigator,
                                sync,
                                form(),
                                true,
                                profile_for_duplicate.clone(),
                            );
                        },
                        "Crear de todas formas"
                    }
                }
            }
            div {
                class: "actions",
                button {
                    disabled: saving(),
                    onclick: move |_| {
                    submit_new_patient(
                        saving,
                        message,
                        warning_ids,
                        navigator,
                        sync,
                        form(),
                        false,
                        profile.clone(),
                    );
                    },
                    if saving() { "Guardando..." } else { "Guardar paciente" }
                }
            }
        }
    }
}

#[component]
fn PatientDetail(id: String) -> Element {
    let user = use_context::<Signal<Option<SessionUser>>>();
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();
    let mut patient = use_signal(|| None::<api::Patient>);
    let mut message = use_signal(String::new);
    let mut confirm_delete = use_signal(|| false);
    let mut deleting = use_signal(|| false);
    let load_id = id.clone();

    use_effect(move || {
        if !checked() {
            return;
        }
        if user().is_none() {
            navigator.replace(Route::AuthPage {});
            return;
        }
        let load_id = load_id.clone();
        spawn(async move {
            match api::fetch_patient(&load_id).await {
                Ok(found) => patient.set(Some(found)),
                Err(error) => {
                    #[cfg(feature = "desktop")]
                    if let Ok(mut conn) = desktop_local::open_local_db() {
                        if let Ok(found) = desktop_local::list_patients(&mut conn) {
                            for value in found {
                                if value.get("id").and_then(Value::as_str) == Some(load_id.as_str())
                                {
                                    if let Ok(mapped) =
                                        serde_json::from_value::<api::Patient>(value)
                                    {
                                        patient.set(Some(mapped));
                                    }
                                    break;
                                }
                            }
                        }
                    }
                    if patient().is_none() {
                        message.set(error);
                    }
                }
            }
        });
    });

    if !checked() {
        return rsx! {
            div { id: "patients-page", p { "Cargando..." } }
        };
    }
    let Some(profile) = actor() else {
        return rsx! {
            div { id: "patients-page", p { "Redirigiendo..." } }
        };
    };
    if !profile.can_read_patients() {
        return rsx! {
            div {
                id: "patients-page",
                div { class: "notice",
                    h4 { "Acceso denegado" }
                    p { "Necesitas un rol en la clínica para consultar pacientes." }
                }
            }
        };
    }
    let Some(patient) = patient() else {
        return rsx! {
            div {
                id: "patients-page",
                if message().is_empty() {
                    p { "Cargando..." }
                } else {
                    p { class: "error", "{message}" }
                    Link { to: Route::Patients {}, "Volver a pacientes" }
                }
            }
        };
    };

    rsx! {
        div {
            id: "patients-page",
            div {
                class: "page-head",
                div {
                    h4 { "{patient.full_name}" }
                    p { "Ficha del paciente" }
                }
                div {
                    class: "actions",
                    if profile.can_write_patients() {
                        Link {
                            class: "button secondary",
                            to: Route::PatientEdit { id: patient.id.clone() },
                            "Editar"
                        }
                    } else {
                        span { class: "badge", "Solo lectura" }
                    }
                }
            }
            div {
                class: "detail-grid",
                div { class: "field", span { "Cédula" }, span { "{patient.national_id.clone().unwrap_or_else(|| \"—\".to_string())}" } }
                div { class: "field", span { "Fecha de nacimiento" }, span { "{optional_date(patient.birth_date.as_deref())}" } }
                div { class: "field", span { "Sexo" }, span { "{sex_label(patient.sex.as_deref())}" } }
                div { class: "field", span { "Teléfono" }, span { "{patient.phone.clone().unwrap_or_else(|| \"—\".to_string())}" } }
                div { class: "field wide", span { "Notas" }, span { class: "notes", "{patient.notes.clone().unwrap_or_else(|| \"—\".to_string())}" } }
            }
            div {
                class: "actions",
                if profile.can_create_prescriptions() {
                    Link {
                        class: "button",
                        to: Route::PrescriptionNew {
                            patient_id: Some(patient.id.clone()),
                            template_id: None,
                        },
                        "Nueva receta"
                    }
                }
                Link {
                    class: "button secondary",
                    to: Route::Prescriptions { status: None },
                    "Historial de recetas"
                }
            }
            if !message().is_empty() {
                p { class: "error", "{message}" }
            }
            if confirm_delete() {
                div {
                    class: "notice",
                    p { "¿Eliminar este paciente? La ficha se oculta de la búsqueda y queda registrada en auditoría." }
                    div {
                        class: "actions",
                        button {
                            disabled: deleting(),
                            onclick: {
                                let patient_id = patient.id.clone();
                                move |_| {
                                    if deleting() {
                                        return;
                                    }
                                    deleting.set(true);
                                    let patient_id = patient_id.clone();
                                    spawn(async move {
                                        match api::delete_patient(&patient_id).await {
                                            Ok(()) => {
                                                navigator.push(Route::Patients {});
                                            }
                                            Err(error) => {
                                                deleting.set(false);
                                                message.set(error);
                                            }
                                        }
                                    });
                                }
                            },
                            if deleting() { "Eliminando..." } else { "Confirmar eliminación" }
                        }
                        button {
                            class: "secondary",
                            disabled: deleting(),
                            onclick: move |_| confirm_delete.set(false),
                            "Cancelar"
                        }
                    }
                }
            } else if profile.can_write_patients() {
                div {
                    class: "actions",
                    button {
                        class: "danger",
                        onclick: move |_| confirm_delete.set(true),
                        "Eliminar paciente"
                    }
                }
            }
            div {
                class: "actions",
                Link { class: "button secondary", to: Route::Patients {}, "Volver a pacientes" }
            }
        }
    }
}

fn submit_edit_patient(
    mut saving: Signal<bool>,
    mut message: Signal<String>,
    navigator: Navigator,
    patient_id: String,
    form: api::PatientForm,
    profile: Me,
    sync: Signal<SyncStatus>,
) {
    if saving() {
        return;
    }
    if form.full_name.trim().is_empty() {
        message.set("Ingresa el nombre del paciente.".to_string());
        return;
    }
    if !form.birth_date.is_empty() && !valid_birth_date(&form.birth_date) {
        message.set("Ingresa una fecha de nacimiento válida.".to_string());
        return;
    }
    saving.set(true);
    message.set(String::new());
    spawn(async move {
        match update_patient_dispatch(&patient_id, &form, Some(profile)).await {
            Ok(patient) => {
                saving.set(false);
                let mut sync = sync;
                sync.set(probe_sync_status().await);
                navigator.push(Route::PatientDetail { id: patient.id });
            }
            Err(error) => {
                saving.set(false);
                message.set(error);
            }
        }
    });
}

#[component]
fn PatientEdit(id: String) -> Element {
    let user = use_context::<Signal<Option<SessionUser>>>();
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let sync = use_context::<Signal<SyncStatus>>();
    let navigator = use_navigator();
    let mut full_name = use_signal(String::new);
    let mut national_id = use_signal(String::new);
    let mut birth_date = use_signal(String::new);
    let mut sex = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut notes = use_signal(String::new);
    let mut message = use_signal(String::new);
    let saving = use_signal(|| false);
    let mut initialized = use_signal(|| false);
    let patient_id = id.clone();

    use_effect({
        let patient_id = patient_id.clone();
        move || {
            if initialized() || !checked() {
                return;
            }
            if user().is_none() {
                navigator.replace(Route::AuthPage {});
                return;
            }
            let patient_id = patient_id.clone();
            spawn(async move {
                match api::fetch_patient(&patient_id).await {
                    Ok(patient) => {
                        full_name.set(patient.full_name);
                        national_id.set(patient.national_id.unwrap_or_default());
                        birth_date.set(patient.birth_date.unwrap_or_default());
                        sex.set(patient.sex.unwrap_or_default());
                        phone.set(patient.phone.unwrap_or_default());
                        notes.set(patient.notes.unwrap_or_default());
                        initialized.set(true);
                    }
                    Err(error) => {
                        #[cfg(feature = "desktop")]
                        if let Ok(mut conn) = desktop_local::open_local_db() {
                            if let Ok(found) = desktop_local::list_patients(&mut conn) {
                                for value in found {
                                    if value.get("id").and_then(Value::as_str)
                                        == Some(patient_id.as_str())
                                    {
                                        if let Ok(mapped) =
                                            serde_json::from_value::<api::Patient>(value)
                                        {
                                            full_name.set(mapped.full_name);
                                            national_id.set(mapped.national_id.unwrap_or_default());
                                            birth_date.set(mapped.birth_date.unwrap_or_default());
                                            sex.set(mapped.sex.unwrap_or_default());
                                            phone.set(mapped.phone.unwrap_or_default());
                                            notes.set(mapped.notes.unwrap_or_default());
                                            initialized.set(true);
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                        if !initialized() {
                            message.set(error);
                        }
                    }
                }
            });
        }
    });

    if !checked() {
        return rsx! {
            div { id: "patients-page", p { "Cargando..." } }
        };
    }
    let Some(profile) = actor() else {
        return rsx! {
            div { id: "patients-page", p { "Redirigiendo..." } }
        };
    };
    if !profile.can_write_patients() {
        return rsx! {
            div {
                id: "patients-page",
                div { class: "notice",
                    h4 { "Acceso denegado" }
                    p { "Solo doctor, enfermería y secretaría pueden editar pacientes." }
                    Link { to: Route::PatientDetail { id: patient_id.clone() }, "Ver ficha" }
                }
            }
        };
    }

    let form = move || api::PatientForm {
        full_name: full_name(),
        national_id: national_id(),
        birth_date: birth_date(),
        sex: sex(),
        phone: phone(),
        notes: notes(),
    };

    rsx! {
        div {
            id: "patients-page",
            div {
                class: "page-head",
                div {
                    h4 { "Editar paciente" }
                    p { "Los cambios se auditan en la ficha del paciente." }
                }
                Link {
                    class: "button secondary",
                    to: Route::PatientDetail { id: patient_id.clone() },
                    "Cancelar"
                }
            }
            if !initialized() && message().is_empty() {
                p { "Cargando..." }
            }
            div {
                class: "form-grid",
                label {
                    class: "field",
                    span { "Nombres y apellidos" }
                    input {
                        value: "{full_name}",
                        oninput: move |event| full_name.set(event.value()),
                    }
                }
                label {
                    class: "field",
                    span { "Cédula" }
                    input {
                        value: "{national_id}",
                        oninput: move |event| national_id.set(event.value()),
                    }
                }
                label {
                    class: "field",
                    span { "Fecha de nacimiento" }
                    input {
                        r#type: "date",
                        value: "{birth_date}",
                        oninput: move |event| birth_date.set(event.value()),
                    }
                }
                label {
                    class: "field",
                    span { "Sexo" }
                    select {
                        value: "{sex}",
                        onchange: move |event| sex.set(event.value()),
                        option { value: "", "Sin especificar" }
                        option { value: "female", "Femenino" }
                        option { value: "male", "Masculino" }
                        option { value: "other", "Otro" }
                        option { value: "unspecified", "Prefiere no decir" }
                    }
                }
                label {
                    class: "field",
                    span { "Teléfono" }
                    input {
                        r#type: "tel",
                        value: "{phone}",
                        oninput: move |event| phone.set(event.value()),
                    }
                }
                label {
                    class: "field wide",
                    span { "Notas" }
                    textarea {
                        value: "{notes}",
                        oninput: move |event| notes.set(event.value()),
                    }
                }
            }
            if !message().is_empty() {
                p { class: "error", "{message}" }
            }
            div {
                class: "actions",
                button {
                    disabled: saving(),
                    onclick: {
                        let patient_id = patient_id.clone();
                        move |_| {
                            submit_edit_patient(
                                saving,
                                message,
                                navigator,
                                patient_id.clone(),
                                form(),
                                profile.clone(),
                                sync,
                            );
                        }
                    },
                    if saving() { "Guardando..." } else { "Guardar cambios" }
                }
            }
        }
    }
}

#[component]
fn Templates() -> Element {
    let user = use_context::<Signal<Option<SessionUser>>>();
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();
    let mut templates = use_signal(Vec::<api::Template>::new);
    let mut message = use_signal(String::new);
    let refresh = use_signal(|| 0);

    use_effect(move || {
        let _ = refresh();
        if !checked() {
            return;
        }
        if user().is_none() {
            navigator.replace(Route::AuthPage {});
            return;
        }
        spawn(async move {
            match api::fetch_templates().await {
                Ok(rows) => {
                    message.set(String::new());
                    templates.set(rows);
                }
                Err(error) => message.set(error),
            }
        });
    });

    if !checked() {
        return rsx! {
            div { id: "templates-page", p { "Cargando..." } }
        };
    }
    let Some(profile) = actor() else {
        return rsx! {
            div { id: "templates-page", p { "Redirigiendo..." } }
        };
    };
    if !profile.can_manage_templates() {
        return rsx! {
            div {
                id: "templates-page",
                div { class: "notice",
                    h4 { "Acceso denegado" }
                    p { "Solo doctor y administrador pueden gestionar plantillas." }
                }
            }
        };
    }
    let rows = templates();

    rsx! {
        div {
            id: "templates-page",
            div {
                class: "page-head",
                div {
                    h4 { "Plantillas" }
                    p { "Plantillas del médico y maestras de la clínica, con su versión vigente." }
                }
                Link { class: "button", to: Route::TemplateNew {}, "Nueva plantilla" }
            }
            if !message().is_empty() {
                p { class: "error", "{message}" }
            }
            div {
                class: "template-list",
                if rows.is_empty() {
                    p { class: "hint", "Todavía no hay plantillas." }
                }
                for template in rows {
                    div {
                        key: "{template.id}",
                        class: "template-row",
                        div {
                            p { class: "patient-name", "{template.name}" }
                            p {
                                class: "patient-meta",
                                "{template_owner_label(&template.owner_type)} · {template_status_label(&template.status)}"
                            }
                            if let Some(version) = template.current_version.clone() {
                                p {
                                    class: "patient-meta",
                                    "Versión vigente: {version.version_no} ({version_status_label(&version.status)})"
                                }
                            } else {
                                p { class: "patient-meta", "Sin versión publicada." }
                            }
                        }
                        if template.status != "archived" {
                            div {
                                class: "actions",
                                Link {
                                    class: "button secondary",
                                    to: Route::TemplateEdit { id: template.id.clone() },
                                    "Editar"
                                }
                                Link {
                                    class: "button secondary",
                                    to: Route::TemplatePreview { id: template.id.clone() },
                                    "Vista previa"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TemplateNew() -> Element {
    let user = use_context::<Signal<Option<SessionUser>>>();
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();
    let mut name = use_signal(String::new);
    let mut message = use_signal(String::new);
    let mut saving = use_signal(|| false);

    use_effect(move || {
        if checked() && user().is_none() {
            navigator.replace(Route::AuthPage {});
        }
    });

    if !checked() {
        return rsx! {
            div { id: "templates-page", p { "Cargando..." } }
        };
    }
    let Some(profile) = actor() else {
        return rsx! {
            div { id: "templates-page", p { "Redirigiendo..." } }
        };
    };
    if !profile.can_manage_templates() {
        return rsx! {
            div {
                id: "templates-page",
                div { class: "notice",
                    h4 { "Acceso denegado" }
                    p { "Solo doctor y administrador pueden crear plantillas." }
                    Link { to: Route::Templates {}, "Volver a plantillas" }
                }
            }
        };
    }
    let owner = if profile.is_administrator() {
        "Maestra de la clínica"
    } else {
        "Plantilla del médico"
    };

    rsx! {
        div {
            id: "templates-page",
            div {
                class: "page-head",
                div {
                    h4 { "Nueva plantilla" }
                    p { "Se crea un borrador; el contenido se define en el editor." }
                }
                Link { class: "button secondary", to: Route::Templates {}, "Cancelar" }
            }
            label {
                class: "field",
                span { "Nombre" }
                input {
                    placeholder: "Nombre de la plantilla",
                    value: "{name}",
                    oninput: move |event| name.set(event.value()),
                }
            }
            p { class: "hint", "Tipo: {owner}" }
            if !message().is_empty() {
                p { class: "error", "{message}" }
            }
            div {
                class: "actions",
                button {
                    disabled: saving(),
                    onclick: move |_| {
                        if saving() {
                            return;
                        }
                        let value = name().trim().to_owned();
                        if value.is_empty() {
                            message.set("Escribe el nombre de la plantilla.".to_owned());
                            return;
                        }
                        let owner_type = if profile.is_administrator() {
                            "clinic"
                        } else {
                            "doctor"
                        };
                        saving.set(true);
                        let profile = profile.clone();
                        message.set(String::new());
                        spawn(async move {
                            match create_template_dispatch(&value, owner_type, Some(profile.clone())).await {
                                Ok(template) => {
                                    navigator.push(Route::TemplateEdit { id: template.id });
                                }
                                Err(error) => {
                                    saving.set(false);
                                    message.set(error);
                                }
                            }
                        });
                    },
                    if saving() { "Creando..." } else { "Crear plantilla" }
                }
            }
        }
    }
}

#[component]
fn TemplateEdit(id: String) -> Element {
    let user = use_context::<Signal<Option<SessionUser>>>();
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();
    let mut template = use_signal(|| None::<api::Template>);
    let mut saved_draft = use_signal(|| None::<String>);
    let mut confirming_archive = use_signal(|| false);
    let mut message = use_signal(String::new);
    let mut generation = use_signal(|| 0);
    let load_id = id.clone();
    let template_id = id.clone();

    use_effect(move || {
        let _ = generation();
        if !checked() {
            return;
        }
        if user().is_none() {
            navigator.replace(Route::AuthPage {});
            return;
        }
        let load_id = load_id.clone();
        spawn(async move {
            match api::fetch_template(&load_id).await {
                Ok(found) => {
                    let draft = found
                        .versions
                        .iter()
                        .find(|version| version.status == "draft")
                        .map(|version| version.id.clone());
                    saved_draft.set(draft);
                    template.set(Some(found));
                }
                Err(error) => {
                    #[cfg(feature = "desktop")]
                    if let Ok(mut conn) = desktop_local::open_local_db() {
                        if let Ok(found) = desktop_local::list_templates(&mut conn) {
                            for value in found {
                                if value.get("id").and_then(Value::as_str) != Some(load_id.as_str())
                                {
                                    continue;
                                }
                                if let Ok(mut mapped) =
                                    serde_json::from_value::<api::Template>(value)
                                {
                                    if mapped.versions.is_empty() {
                                        if let Ok(versions) = desktop_local::list_template_versions(
                                            &mut conn, &load_id,
                                        ) {
                                            for version in versions {
                                                let version_id = version
                                                    .get("id")
                                                    .and_then(Value::as_str)
                                                    .unwrap_or_default()
                                                    .to_owned();
                                                if version_id.is_empty() {
                                                    continue;
                                                }
                                                if let Ok(Some(full)) =
                                                    desktop_local::get_template_version(
                                                        &mut conn,
                                                        &version_id,
                                                    )
                                                {
                                                    if let Ok(parsed) = serde_json::from_value::<
                                                        api::TemplateVersion,
                                                    >(
                                                        full
                                                    ) {
                                                        mapped.versions.push(parsed);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    let draft = mapped
                                        .versions
                                        .iter()
                                        .find(|version| version.status == "draft")
                                        .map(|version| version.id.clone());
                                    saved_draft.set(draft);
                                    template.set(Some(mapped));
                                }
                                break;
                            }
                        }
                    }
                    if template().is_none() {
                        message.set(error);
                    }
                }
            }
        });
    });

    if !checked() {
        return rsx! {
            div { id: "templates-page", p { "Cargando..." } }
        };
    }
    let Some(profile) = actor() else {
        return rsx! {
            div { id: "templates-page", p { "Redirigiendo..." } }
        };
    };
    if !profile.can_manage_templates() {
        return rsx! {
            div {
                id: "templates-page",
                div { class: "notice",
                    h4 { "Acceso denegado" }
                    p { "Solo doctor y administrador pueden editar plantillas." }
                }
            }
        };
    }
    let Some(model) = template() else {
        return rsx! {
            div {
                id: "templates-page",
                if message().is_empty() {
                    p { "Cargando..." }
                } else {
                    p { class: "error", "{message}" }
                    Link { to: Route::Templates {}, "Volver a plantillas" }
                }
            }
        };
    };
    let draft = model
        .versions
        .iter()
        .find(|version| version.status == "draft");
    let base = draft
        .or(model.current_version.as_ref())
        .or(model.versions.first());
    let initial_doc = base
        .map(|version| version.doc_json.clone())
        .unwrap_or_else(forms::default_document_json);
    let initial_margin = base
        .map(|version| forms::template_theme_margin(version.theme.as_deref()))
        .unwrap_or(15);
    let version_id = draft.map(|version| version.id.clone());
    let archived = model.status == "archived";

    rsx! {
        div {
            id: "templates-page",
            div {
                class: "page-head",
                div {
                    h4 { "{model.name}" }
                    p { "{template_owner_label(&model.owner_type)} · {template_status_label(&model.status)}" }
                }
                div {
                    class: "actions",
                    Link {
                        class: "button secondary",
                        to: Route::TemplatePreview { id: model.id.clone() },
                        "Vista previa"
                    }
                    Link { class: "button secondary", to: Route::Templates {}, "Volver" }
                }
            }
            if archived {
                div {
                    class: "notice",
                    p { "La plantilla está archivada y no admite cambios." }
                }
            } else {
                forms::TemplateDocumentForm {
                    key: "{model.id}-{generation}",
                    template_id: template_id.clone(),
                    version_id,
                    initial_doc,
                    initial_margin_mm: initial_margin,
                    on_saved: move |(version_id, _, _)| {
                        saved_draft.set(Some(version_id));
                        message.set("Borrador guardado.".to_owned());
                    },
                }
                if !message().is_empty() {
                    p { class: "error", "{message}" }
                }
                div {
                    class: "actions",
                    button {
                        disabled: saved_draft().is_none(),
                        onclick: {
                            let template_id = template_id.clone();
                            move |_| {
                                let Some(version_id) = saved_draft() else {
                                    return;
                                };
                                let template_id = template_id.clone();
                                let profile = profile.clone();
                                spawn(async move {
                                    match publish_template_dispatch(&template_id, &version_id, template().map(|found| (found, profile.clone()))).await {
                                        Ok(published) => {
                                            saved_draft.set(None);
                                            generation += 1;
                                            template.set(Some(published));
                                            message.set("Versión publicada e inmutable.".to_owned());
                                        }
                                        Err(error) => message.set(error),
                                    }
                                });
                            }
                        },
                        "Publicar versión"
                    }
                    button {
                        class: "secondary",
                        disabled: !matches!(model.status.as_str(), "draft" | "published"),
                        onclick: move |_| {
                            generation += 1;
                        },
                        "Recargar"
                    }
                    if confirming_archive() {
                        button {
                            class: "danger",
                            onclick: {
                                let template_id = template_id.clone();
                                move |_| {
                                    let template_id = template_id.clone();
                                    spawn(async move {
                                        match api::archive_template(&template_id).await {
                                            Ok(archived) => {
                                                saved_draft.set(None);
                                                confirming_archive.set(false);
                                                template.set(Some(archived));
                                                message.set("Plantilla archivada.".to_owned());
                                            }
                                            Err(error) => message.set(error),
                                        }
                                    });
                                }
                            },
                            "Confirmar archivo"
                        }
                        button {
                            class: "secondary",
                            onclick: move |_| confirming_archive.set(false),
                            "Cancelar"
                        }
                    } else {
                        button {
                            class: "danger",
                            onclick: move |_| confirming_archive.set(true),
                            "Archivar plantilla"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TemplatePreview(id: String) -> Element {
    let user = use_context::<Signal<Option<SessionUser>>>();
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();
    let mut template = use_signal(|| None::<api::Template>);
    let mut selected = use_signal(|| None::<String>);
    let mut margin = use_signal(|| 15_u32);
    let mut message = use_signal(String::new);
    let load_id = id.clone();

    use_effect(move || {
        if !checked() {
            return;
        }
        if user().is_none() {
            navigator.replace(Route::AuthPage {});
            return;
        }
        let load_id = load_id.clone();
        spawn(async move {
            match api::fetch_template(&load_id).await {
                Ok(found) => {
                    let chosen = found.current_version.as_ref().or(found.versions.first());
                    selected.set(chosen.map(|version| version.id.clone()));
                    margin.set(
                        chosen
                            .map(|version| forms::template_theme_margin(version.theme.as_deref()))
                            .unwrap_or(15),
                    );
                    template.set(Some(found));
                }
                Err(error) => message.set(error),
            }
        });
    });

    if !checked() {
        return rsx! {
            div { id: "templates-page", p { "Cargando..." } }
        };
    }
    let Some(profile) = actor() else {
        return rsx! {
            div { id: "templates-page", p { "Redirigiendo..." } }
        };
    };
    if !profile.can_manage_templates() {
        return rsx! {
            div {
                id: "templates-page",
                div { class: "notice",
                    h4 { "Acceso denegado" }
                    p { "Solo doctor y administrador pueden ver la vista previa." }
                }
            }
        };
    }
    let Some(model) = template() else {
        return rsx! {
            div {
                id: "templates-page",
                if message().is_empty() {
                    p { "Cargando..." }
                } else {
                    p { class: "error", "{message}" }
                    Link { to: Route::Templates {}, "Volver a plantillas" }
                }
            }
        };
    };
    let version = selected()
        .and_then(|id| {
            model
                .versions
                .iter()
                .find(|version| version.id == id)
                .cloned()
        })
        .or(model.current_version.clone())
        .or_else(|| model.versions.first().cloned());
    let Some(version) = version else {
        return rsx! {
            div {
                id: "templates-page",
                div { class: "notice",
                    p { "La plantilla todavía no tiene versiones para previsualizar." }
                    Link { to: Route::TemplateEdit { id: model.id.clone() }, "Abrir editor" }
                }
            }
        };
    };
    let warnings = forms::parse_document(&version.doc_json)
        .map(|document| forms::document_warnings(&document))
        .unwrap_or_default();
    let warnings_joined = warnings.join(", ");
    let versions = model.versions.clone();

    rsx! {
        div {
            id: "templates-page",
            div {
                class: "page-head",
                div {
                    h4 { "Vista previa A4 — {model.name}" }
                    p { "Proyección 210 × 297 mm con valores de ejemplo." }
                }
                div {
                    class: "actions",
                    Link { to: Route::Templates {}, "Volver a plantillas" }
                    Link {
                        class: "button secondary",
                        to: Route::TemplateEdit { id: model.id.clone() },
                        "Abrir editor"
                    }
                }
            }
            div {
                class: "preview-controls",
                label {
                    class: "field",
                    span { "Versión" }
                    select {
                        value: "{selected().unwrap_or_default()}",
                        onchange: move |event| {
                            let value = event.value();
                            if let Some(version) = model.versions.iter().find(|version| version.id == value) {
                                margin.set(forms::template_theme_margin(version.theme.as_deref()));
                            }
                            selected.set(Some(value));
                        },
                        for version in versions {
                            option {
                                value: "{version.id}",
                                "Versión {version.version_no} — {version_status_label(&version.status)}"
                            }
                        }
                    }
                }
                label {
                    class: "field",
                    span { "Márgenes (mm)" }
                    input {
                        r#type: "number",
                        min: "5",
                        max: "40",
                        value: "{margin}",
                        oninput: move |event| {
                            if let Ok(value) = event.value().parse::<u32>() {
                                margin.set(value.clamp(5, 40));
                            }
                        },
                    }
                }
            }
            if !warnings.is_empty() {
                div {
                    class: "notice",
                    p { "Variables no válidas: {warnings_joined}" }
                    p { class: "hint", "En la proyección se marcan como texto de advertencia, nunca como {{clave}}." }
                }
            }
            div {
                class: "preview-scroll",
                forms::A4Preview {
                    doc_json: version.doc_json.clone(),
                    values: forms::sample_values(),
                    margin_mm: margin(),
                }
            }
        }
    }
}

fn template_status_label(status: &str) -> &'static str {
    match status {
        "draft" => "Borrador",
        "published" => "Publicada",
        "archived" => "Archivada",
        _ => "Desconocido",
    }
}

fn version_status_label(status: &str) -> &'static str {
    match status {
        "draft" => "Borrador",
        "published" => "Publicada",
        "archived" => "Archivada",
        _ => "Desconocido",
    }
}

fn template_owner_label(owner_type: &str) -> &'static str {
    match owner_type {
        "doctor" => "Plantilla del médico",
        "clinic" => "Maestra de la clínica",
        _ => "Plantilla",
    }
}

fn valid_birth_date(value: &str) -> bool {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok()
}

fn optional_date(value: Option<&str>) -> String {
    match value {
        Some(date) => format_date(date),
        None => "—".to_string(),
    }
}

fn sex_label(sex: Option<&str>) -> &'static str {
    match sex {
        Some("female") => "Femenino",
        Some("male") => "Masculino",
        Some("other") => "Otro",
        Some("unspecified") => "Prefiere no decir",
        _ => "Sin especificar",
    }
}

fn role_label(role: Option<&str>) -> &'static str {
    match role {
        Some("administrator") => "Administrador",
        Some("doctor") => "Doctor",
        Some("nursing") => "Enfermería",
        Some("secretary") => "Secretaría",
        _ => "Sin rol",
    }
}

fn status_label(status: &str) -> &'static str {
    if status == "active" {
        "Activo"
    } else {
        "Deshabilitado"
    }
}

fn platform_label(platform: &str) -> &'static str {
    match platform {
        "macos" => "macOS",
        "windows" => "Windows",
        "web" => "Web",
        _ => "Desconocido",
    }
}

fn device_name_label(device: &api::DeviceRow) -> String {
    device
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| platform_label(&device.platform).to_owned())
}

fn last_seen_label(value: Option<&str>) -> String {
    match value {
        Some(value) => format_timestamp(value),
        None => "Nunca".to_owned(),
    }
}

fn format_timestamp(value: &str) -> String {
    match chrono::DateTime::parse_from_rfc3339(value) {
        Ok(parsed) => parsed.format("%d-%m-%Y %H:%M").to_string(),
        Err(_) => value.to_owned(),
    }
}

fn format_date(value: &str) -> String {
    let date = value.get(..10).unwrap_or(value);
    match chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        Ok(parsed) => parsed.format("%d-%m-%Y").to_string(),
        Err(_) => value.to_owned(),
    }
}

fn prescription_status_label(status: &str) -> &'static str {
    match status {
        "draft" => "Borrador",
        "signed" => "Firmada",
        "superseded" => "Reemplazada",
        "voided" => "Anulada",
        _ => "Desconocida",
    }
}

fn llm_explain_url(template: Option<&str>, medication: &str) -> Option<String> {
    let name = medication.trim();
    if name.is_empty() {
        return None;
    }
    let encoded = crate::http::encode_query(name);
    match template.map(str::trim).filter(|value| !value.is_empty()) {
        Some(template) => Some(
            template
                .replace("{med}", &encoded)
                .replace("{medication}", &encoded),
        ),
        None => Some(format!("https://chat.openai.com/?q={encoded}")),
    }
}

fn new_prescription_item_form() -> api::PrescriptionItemForm {
    api::PrescriptionItemForm::default()
}

#[component]
fn Prescriptions(status: Option<String>) -> Element {
    let user = use_context::<Signal<Option<SessionUser>>>();
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();
    let mut rows = use_signal(Vec::<api::Prescription>::new);
    let mut query = use_signal(String::new);
    let mut status = use_signal(|| status.clone().unwrap_or_default());
    let mut date = use_signal(String::new);
    let mut message = use_signal(String::new);
    #[allow(unused_mut)]
    let mut failed_rows = use_signal(Vec::<(String, String)>::new);
    let mut loading = use_signal(|| true);

    use_effect(move || {
        if !checked() {
            return;
        }
        if user().is_none() {
            navigator.replace(Route::AuthPage {});
            return;
        }
        let selected_status = status();
        let selected_date = date();
        loading.set(true);
        spawn(async move {
            #[cfg(feature = "desktop")]
            if selected_status == "pending_sync" {
                if let Ok(mut conn) = desktop_local::open_local_db() {
                    if let Ok(found) = desktop_local::list_prescriptions(&mut conn, "pending_sync")
                    {
                        let mut mapped = Vec::new();
                        for value in found {
                            if let Ok(prescription) =
                                serde_json::from_value::<api::Prescription>(value)
                            {
                                mapped.push(prescription);
                            }
                        }
                        rows.set(mapped);
                        message.set(String::new());
                        loading.set(false);
                        return;
                    }
                }
            }
            #[cfg(feature = "desktop")]
            if selected_status == "failed" {
                if let Ok(mut conn) = desktop_local::open_local_db() {
                    if let Ok(found) = desktop_local::list_prescriptions(&mut conn, "failed") {
                        let mut mapped = Vec::new();
                        for value in found {
                            if let Ok(prescription) =
                                serde_json::from_value::<api::Prescription>(value)
                            {
                                mapped.push(prescription);
                            }
                        }
                        rows.set(mapped);
                        let mut errors = Vec::new();
                        if let Ok(all) = desktop_local::outbox_rows(&mut conn) {
                            for value in all {
                                if value.get("status").and_then(serde_json::Value::as_str)
                                    == Some("failed")
                                {
                                    let detail = value
                                        .get("lastError")
                                        .and_then(serde_json::Value::as_str)
                                        .unwrap_or_default()
                                        .to_owned();
                                    let operation = value
                                        .get("operation")
                                        .and_then(serde_json::Value::as_str)
                                        .unwrap_or_default()
                                        .to_owned();
                                    errors.push((operation, detail));
                                }
                            }
                        }
                        failed_rows.set(errors);
                        loading.set(false);
                        return;
                    }
                }
            }
            match api::fetch_prescriptions("", &selected_status, &selected_date).await {
                Ok(found) => {
                    rows.set(found);
                    message.set(String::new());
                }
                Err(error) => {
                    message.set(error);
                    #[cfg(feature = "desktop")]
                    if let Ok(mut conn) = desktop_local::open_local_db() {
                        if let Ok(found) = desktop_local::list_prescriptions(&mut conn, "") {
                            let mut mapped = Vec::new();
                            for value in found {
                                if let Ok(prescription) =
                                    serde_json::from_value::<api::Prescription>(value)
                                {
                                    mapped.push(prescription);
                                }
                            }
                            rows.set(mapped);
                        }
                    }
                }
            }
            loading.set(false);
        });
    });

    if !checked() {
        return rsx! {
            div { id: "prescriptions-page", p { "Cargando..." } }
        };
    }
    let Some(profile) = actor() else {
        return rsx! {
            div { id: "prescriptions-page", p { "Redirigiendo..." } }
        };
    };
    if !profile.can_read_prescriptions() {
        return rsx! {
            div {
                id: "prescriptions-page",
                div { class: "notice",
                    h4 { "Acceso denegado" }
                    p { "Necesitas un rol en la clínica para consultar recetas." }
                }
            }
        };
    }
    let search = query().trim().to_lowercase();
    let visible = rows()
        .into_iter()
        .filter(|row| search.is_empty() || row.patient_name.to_lowercase().contains(&search))
        .collect::<Vec<_>>();

    rsx! {
        div {
            id: "prescriptions-page",
            div {
                class: "page-head",
                div {
                    h4 { "Recetas" }
                    p { "Historial con filtros por paciente, fecha y estado." }
                }
                if profile.can_create_prescriptions() {
                    Link {
                        class: "button",
                        to: Route::PrescriptionNew { patient_id: None, template_id: None },
                        "Nueva receta"
                    }
                }
            }
            div {
                class: "prescription-filters",
                input {
                    r#type: "search",
                    placeholder: "Paciente",
                    value: "{query}",
                    oninput: move |event| query.set(event.value()),
                }
                select {
                    value: "{status}",
                    onchange: move |event| status.set(event.value()),
                    option { value: "", "Todos los estados" }
                    option { value: "draft", "Borradores" }
                    option { value: "signed", "Firmadas" }
                    option { value: "superseded", "Reemplazadas" }
                    option { value: "voided", "Anuladas" }
                    option { value: "pending_sync", "Pendientes de sincronización" }
                    option { value: "failed", "Con error de sincronización" }
                }
                input {
                    r#type: "date",
                    value: "{date}",
                    oninput: move |event| date.set(event.value()),
                }
            }
            if !message().is_empty() {
                p { class: "error", "{message}" }
            }
            div {
                class: "patient-list",
                if loading() {
                    p { class: "hint", "Cargando recetas..." }
                } else if visible.is_empty() {
                    p { class: "hint", "No hay recetas para estos filtros." }
                }
                for row in visible {
                    div {
                        key: "{row.id}",
                        class: "patient-row",
                        div {
                            p { class: "patient-name", "{row.patient_name}" }
                            p { class: "patient-meta",
                                "{prescription_status_label(&row.status)} · {format_date(&row.created_at)}"
                                if let Some(number) = row.rx_number {
                                    " · No. {number}"
                                }
                            }
                            if status() == "pending_sync" {
                                span { class: "badge", "Provisional · PENDIENTE DE SINCRONIZACIÓN" }
                            }
                        }
                        Link {
                            class: "button secondary",
                            to: Route::PrescriptionDetail { id: row.id.clone() },
                            "Abrir"
                        }
                    }
                }
            }
            if status() == "failed" {
                div {
                    class: "patient-list",
                    for (at, (operation, detail)) in failed_rows().into_iter().enumerate() {
                        div {
                            key: "{at}",
                            class: "patient-row",
                            div {
                                p { class: "patient-name", "{operation}" }
                                p { class: "patient-meta", "{detail}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PrescriptionNew(patient_id: Option<String>, template_id: Option<String>) -> Element {
    let user = use_context::<Signal<Option<SessionUser>>>();
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    #[cfg(feature = "desktop")]
    let mut sync = use_context::<Signal<SyncStatus>>();
    let navigator = use_navigator();
    let mut patients = use_signal(Vec::<api::Patient>::new);
    let mut templates = use_signal(Vec::<api::Template>::new);
    let mut selected_patient = use_signal(|| patient_id.clone().unwrap_or_default());
    let mut selected_template = use_signal(|| template_id.clone().unwrap_or_default());
    let mut diagnosis = use_signal(String::new);
    let mut validity = use_signal(String::new);
    let mut instructions = use_signal(String::new);
    let mut message = use_signal(String::new);
    let mut saving = use_signal(|| false);
    let mut loaded = use_signal(|| false);

    use_effect(move || {
        if loaded() || !checked() {
            return;
        }
        if user().is_none() {
            navigator.replace(Route::AuthPage {});
            return;
        }
        spawn(async move {
            if let Ok(found) = api::fetch_patients("").await {
                patients.set(found)
            } else {
                #[cfg(feature = "desktop")]
                if let Ok(mut conn) = desktop_local::open_local_db() {
                    if let Ok(found) = desktop_local::list_patients(&mut conn) {
                        let mut mapped = Vec::new();
                        for value in found {
                            if let Ok(patient) = serde_json::from_value::<api::Patient>(value) {
                                mapped.push(patient);
                            }
                        }
                        patients.set(mapped);
                    }
                }
            }
            if let Ok(found) = api::fetch_templates().await {
                templates.set(found)
            } else {
                #[cfg(feature = "desktop")]
                if let Ok(mut conn) = desktop_local::open_local_db() {
                    if let Ok(found) = desktop_local::list_templates(&mut conn) {
                        let mut mapped = Vec::new();
                        for value in found {
                            if let Ok(template) = serde_json::from_value::<api::Template>(value) {
                                mapped.push(template);
                            }
                        }
                        templates.set(mapped);
                    }
                }
            }
            loaded.set(true);
        });
    });

    if !checked() {
        return rsx! {
            div { id: "prescriptions-page", p { "Cargando..." } }
        };
    }
    let Some(profile) = actor() else {
        return rsx! {
            div { id: "prescriptions-page", p { "Redirigiendo..." } }
        };
    };
    if !profile.can_create_prescriptions() {
        return rsx! {
            div {
                id: "prescriptions-page",
                div { class: "notice",
                    h4 { "Acceso denegado" }
                    p { "Solo doctor o enfermería pueden crear recetas." }
                    Link { to: Route::Prescriptions { status: None }, "Volver a recetas" }
                }
            }
        };
    }

    let published = templates()
        .into_iter()
        .filter(|template| {
            template.current_version_id.is_some()
                && template
                    .current_version
                    .as_ref()
                    .is_some_and(|version| version.status == "published")
        })
        .collect::<Vec<_>>();

    rsx! {
        div {
            id: "prescriptions-page",
            div {
                class: "page-head",
                div {
                    h4 { "Nueva receta" }
                    p { "Paciente y plantilla publicada." }
                }
                Link { class: "button secondary", to: Route::Prescriptions { status: None }, "Cancelar" }
            }
            div {
                class: "form-grid",
                label {
                    class: "field",
                    span { "Paciente" }
                    select {
                        value: "{selected_patient}",
                        onchange: move |event| selected_patient.set(event.value()),
                        option { value: "", "Selecciona un paciente" }
                        for patient in patients() {
                            option { value: "{patient.id}", "{patient.full_name}" }
                        }
                    }
                }
                label {
                    class: "field",
                    span { "Plantilla" }
                    select {
                        value: "{selected_template}",
                        onchange: move |event| selected_template.set(event.value()),
                        option { value: "", "Selecciona una plantilla" }
                        for template in published {
                            option {
                                value: "{template.current_version_id.clone().unwrap_or_default()}",
                                "{template.name}"
                            }
                        }
                    }
                }
                label {
                    class: "field",
                    span { "Diagnóstico" }
                    textarea {
                        value: "{diagnosis}",
                        oninput: move |event| diagnosis.set(event.value()),
                    }
                }
                label {
                    class: "field",
                    span { "Vigencia" }
                    input {
                        value: "{validity}",
                        oninput: move |event| validity.set(event.value()),
                    }
                }
                label {
                    class: "field wide",
                    span { "Indicaciones generales" }
                    textarea {
                        value: "{instructions}",
                        oninput: move |event| instructions.set(event.value()),
                    }
                }
            }
            if !message().is_empty() {
                p { class: "error", "{message}" }
            }
            div {
                class: "actions",
                button {
                    disabled: saving(),
                    onclick: move |_| {
                        if saving() {
                            return;
                        }
                        let patient = selected_patient();
                        let template = selected_template();
                        if patient.is_empty() || template.is_empty() {
                            message.set("Selecciona paciente y plantilla.".to_string());
                            return;
                        }
                        saving.set(true);
                        message.set(String::new());
                        let diagnosis = diagnosis();
                        let validity = validity();
                        let instructions = instructions();
                        let profile_clone = profile.clone();
                        let _ = &profile_clone;
                        spawn(async move {
                            if !http::is_server_reachable().await {
                                #[cfg(feature = "desktop")]
                                {
                                    saving.set(false);
                                    match desktop_local::open_local_db().and_then(|mut conn| {
                                        desktop_local::create_local_prescription(
                                            &mut conn,
                                            &patient,
                                            &template,
                                            &profile_clone.clinic.as_ref().map(|clinic| clinic.id.clone()).unwrap_or_default(),
                                            profile_clone.doctor.as_ref().map(|doctor| doctor.id.clone()).as_deref(),
                                            &diagnosis,
                                            &validity,
                                            &instructions,
                                            &profile_clone.profile.id,
                                        )
                                    }) {
                                        Ok(created) => {
                                            sync.set(probe_sync_status().await);
                                            navigator.push(Route::PrescriptionDetail { id: created.id });
                                        }
                                        Err(error) => message.set(error),
                                    }
                                    return;
                                }
                            }
                            match api::create_prescription(
                                &patient,
                                &template,
                                "",
                                &diagnosis,
                                &validity,
                                &instructions,
                            )
                            .await
                            {
                                Ok(prescription) => {
                                    saving.set(false);
                                    navigator.push(Route::PrescriptionDetail {
                                        id: prescription.id,
                                    });
                                }
                                Err(error) => {
                                    saving.set(false);
                                    message.set(error);
                                }
                            }
                        });
                    },
                    if saving() { "Creando..." } else { "Crear borrador" }
                }
            }
        }
    }
}

fn reload_prescription(
    mut prescription: Signal<Option<api::Prescription>>,
    mut items: Signal<Vec<api::PrescriptionItemForm>>,
    mut message: Signal<String>,
    id: String,
) {
    spawn(async move {
        match api::fetch_prescription(&id).await {
            Ok(found) => {
                items.set(
                    found
                        .items
                        .iter()
                        .map(|item| api::PrescriptionItemForm {
                            id: Some(item.id.clone()),
                            med_name: item.med_name.clone(),
                            dose: item.dose.clone().unwrap_or_default(),
                            route: item.route.clone().unwrap_or_default(),
                            frequency: item.frequency.clone().unwrap_or_default(),
                            duration: item.duration.clone().unwrap_or_default(),
                            quantity: item.quantity.clone().unwrap_or_default(),
                            notes: item.notes.clone().unwrap_or_default(),
                        })
                        .collect(),
                );
                prescription.set(Some(found));
            }
            Err(error) => {
                #[cfg(feature = "desktop")]
                if let Ok(mut conn) = desktop_local::open_local_db() {
                    if let Ok(found) = desktop_local::list_prescriptions(&mut conn, "") {
                        for value in found {
                            if value.get("id").and_then(Value::as_str) != Some(id.as_str()) {
                                continue;
                            }
                            if let Ok(mapped) = serde_json::from_value::<api::Prescription>(value) {
                                let local_items =
                                    desktop_local::local_prescription_items(&mut conn, &id)
                                        .ok()
                                        .unwrap_or_default();
                                items.set(
                                    local_items
                                        .into_iter()
                                        .map(|item| api::PrescriptionItemForm {
                                            id: item
                                                .get("id")
                                                .and_then(Value::as_str)
                                                .map(str::to_owned),
                                            med_name: item
                                                .get("medName")
                                                .and_then(Value::as_str)
                                                .unwrap_or_default()
                                                .to_owned(),
                                            dose: item
                                                .get("dose")
                                                .and_then(Value::as_str)
                                                .unwrap_or_default()
                                                .to_owned(),
                                            route: item
                                                .get("route")
                                                .and_then(Value::as_str)
                                                .unwrap_or_default()
                                                .to_owned(),
                                            frequency: item
                                                .get("frequency")
                                                .and_then(Value::as_str)
                                                .unwrap_or_default()
                                                .to_owned(),
                                            duration: item
                                                .get("duration")
                                                .and_then(Value::as_str)
                                                .unwrap_or_default()
                                                .to_owned(),
                                            quantity: item
                                                .get("quantity")
                                                .and_then(Value::as_str)
                                                .unwrap_or_default()
                                                .to_owned(),
                                            notes: item
                                                .get("notes")
                                                .and_then(Value::as_str)
                                                .unwrap_or_default()
                                                .to_owned(),
                                        })
                                        .collect(),
                                );
                                prescription.set(Some(mapped));
                            }
                            break;
                        }
                    }
                }
                if prescription().is_none() {
                    message.set(error);
                }
            }
        }
    });
}

#[component]
fn PrescriptionPreview(id: String) -> Element {
    let user = use_context::<Signal<Option<SessionUser>>>();
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let mut sync = use_context::<Signal<SyncStatus>>();
    let navigator = use_navigator();
    let mut preview = use_signal(|| None::<api::PrescriptionPreview>);
    let mut prescription = use_signal(|| None::<api::Prescription>);
    let mut message = use_signal(String::new);
    let mut signing = use_signal(|| false);
    let mut needs_ack = use_signal(|| false);
    let mut generation = use_signal(|| 0_u32);
    let prescription_id = id.clone();

    use_effect(move || {
        if !checked() {
            return;
        }
        if user().is_none() {
            navigator.replace(Route::AuthPage {});
            return;
        }
        let _ = generation();
        let load_id = prescription_id.clone();
        spawn(async move {
            if let Ok(found) = api::fetch_prescription(&load_id).await {
                prescription.set(Some(found));
            } else {
                #[cfg(feature = "desktop")]
                if let Ok(mut conn) = desktop_local::open_local_db() {
                    if let Ok(Some(found)) = desktop_local::get_prescription(&mut conn, &load_id) {
                        if let Ok(parsed) = serde_json::from_value(found) {
                            prescription.set(Some(parsed));
                        }
                    }
                }
            }
            match api::fetch_prescription_preview(&load_id).await {
                Ok(found) => preview.set(Some(found)),
                Err(error) => {
                    #[cfg(feature = "desktop")]
                    if let Ok(mut conn) = desktop_local::open_local_db() {
                        if let Ok(Some(found)) =
                            desktop_local::get_prescription(&mut conn, &load_id)
                        {
                            let status = found
                                .get("status")
                                .and_then(Value::as_str)
                                .unwrap_or("draft")
                                .to_owned();
                            let rx_number = found
                                .get("rxNumber")
                                .and_then(Value::as_i64)
                                .map(|value| value as i32);
                            let version_id = found
                                .get("templateVersionId")
                                .and_then(Value::as_str)
                                .unwrap_or_default()
                                .to_owned();
                            let doc_json =
                                desktop_local::get_template_version(&mut conn, &version_id)
                                    .ok()
                                    .flatten()
                                    .and_then(|version| {
                                        version
                                            .get("docJson")
                                            .and_then(Value::as_str)
                                            .map(str::to_owned)
                                    })
                                    .unwrap_or_else(forms::default_document_json);
                            let patient_id = found
                                .get("patientId")
                                .and_then(Value::as_str)
                                .unwrap_or_default()
                                .to_owned();
                            let patient_name = desktop_local::list_patients(&mut conn)
                                .ok()
                                .unwrap_or_default()
                                .into_iter()
                                .find(|patient| {
                                    patient.get("id").and_then(Value::as_str)
                                        == Some(patient_id.as_str())
                                })
                                .and_then(|patient| {
                                    patient
                                        .get("fullName")
                                        .and_then(Value::as_str)
                                        .map(str::to_owned)
                                })
                                .unwrap_or_else(|| "Paciente".to_owned());
                            let issued_at = found
                                .get("signedAt")
                                .and_then(Value::as_str)
                                .or_else(|| found.get("createdAt").and_then(Value::as_str))
                                .unwrap_or_default()
                                .to_owned();
                            let medications =
                                desktop_local::local_prescription_items(&mut conn, &load_id)
                                    .ok()
                                    .unwrap_or_default()
                                    .into_iter()
                                    .map(|item| forms::MedicationBlock {
                                        name: item
                                            .get("medName")
                                            .and_then(Value::as_str)
                                            .unwrap_or_default()
                                            .to_owned(),
                                        dose: item
                                            .get("dose")
                                            .and_then(Value::as_str)
                                            .unwrap_or_default()
                                            .to_owned(),
                                        route: item
                                            .get("route")
                                            .and_then(Value::as_str)
                                            .unwrap_or_default()
                                            .to_owned(),
                                        frequency: item
                                            .get("frequency")
                                            .and_then(Value::as_str)
                                            .unwrap_or_default()
                                            .to_owned(),
                                        duration: item
                                            .get("duration")
                                            .and_then(Value::as_str)
                                            .unwrap_or_default()
                                            .to_owned(),
                                        quantity: item
                                            .get("quantity")
                                            .and_then(Value::as_str)
                                            .unwrap_or_default()
                                            .to_owned(),
                                        notes: item
                                            .get("notes")
                                            .and_then(Value::as_str)
                                            .unwrap_or_default()
                                            .to_owned(),
                                    })
                                    .collect();
                            let local_preview = api::PrescriptionPreview {
                                id: load_id.clone(),
                                status,
                                rx_number,
                                verification_code: None,
                                issued_at,
                                provisional: true,
                                doc_json,
                                margin_mm: 12,
                                variables: vec![api::PreviewVariable {
                                    key: "patient.name".to_owned(),
                                    value: patient_name,
                                }],
                                medications,
                                qr_data_url: None,
                                image_sources: Vec::new(),
                            };
                            preview.set(Some(local_preview));
                            message.set(String::new());
                        }
                    }
                    if preview().is_none() {
                        message.set(error);
                    }
                }
            }
        });
    });

    if !checked() {
        return rsx! {
            div { id: "prescriptions-page", p { "Cargando..." } }
        };
    }
    let Some(profile) = actor() else {
        return rsx! {
            div { id: "prescriptions-page", p { "Redirigiendo..." } }
        };
    };
    if !profile.can_read_prescriptions() {
        return rsx! {
            div {
                id: "prescriptions-page",
                div { class: "notice",
                    h4 { "Acceso denegado" }
                    p { "No tienes permiso para ver esta receta." }
                }
            }
        };
    }
    let Some(current) = preview() else {
        return rsx! {
            div {
                id: "prescriptions-page",
                if message().is_empty() {
                    p { "Cargando vista previa..." }
                } else {
                    p { class: "error", "{message}" }
                    Link { to: Route::Prescriptions { status: None }, "Volver a recetas" }
                }
            }
        };
    };
    let is_draft = current.status == "draft";
    let can_sign = is_draft && profile.can_sign_prescriptions();
    let profile_for_ack = profile.clone();
    let patient_name = current
        .variables
        .iter()
        .find(|variable| variable.key == "patient.name")
        .map(|variable| variable.value.clone())
        .unwrap_or_else(|| "Receta".to_owned());
    let values: Vec<(String, String)> = current
        .variables
        .iter()
        .map(|variable| (variable.key.clone(), variable.value.clone()))
        .collect();
    let image_sources: Vec<(String, String)> = current
        .image_sources
        .iter()
        .map(|source| (source.asset_id.clone(), source.data_url.clone()))
        .collect();
    let meta = forms::DocumentMeta {
        verification_code: current.verification_code.clone(),
        rx_number: current.rx_number,
        issued_at: Some(current.issued_at.clone()),
        provisional: current.provisional,
        qr_data_url: current.qr_data_url.clone(),
    };
    let pdf_url = format!("/api/prescriptions/{}/pdf", current.id);
    let preview_id = current.id.clone();
    let pdf_doc = current.doc_json.clone();
    let pdf_values = values.clone();
    let pdf_meds = current.medications.clone();
    let pdf_rx = current.rx_number;
    let pdf_issued = current.issued_at.clone();

    rsx! {
        div {
            id: "prescriptions-page",
            div {
                class: "page-head",
                div {
                    h4 { "Vista previa A4 — {patient_name}" }
                    p {
                        "{prescription_status_label(&current.status)}"
                        if let Some(number) = current.rx_number {
                            " · No. {number}"
                        }
                        " · {current.issued_at}"
                    }
                }
                div {
                    class: "actions",
                    Link {
                        class: "button secondary",
                        to: Route::PrescriptionDetail { id: current.id.clone() },
                        "Volver a la receta"
                    }
                    if is_draft {
                        if can_sign {
                            button {
                                disabled: signing(),
                                onclick: {
                                    let preview_id = preview_id.clone();
                                    move |_| {
                                        if signing() {
                                            return;
                                        }
                                        signing.set(true);
                                        needs_ack.set(false);
                                        let preview_id = preview_id.clone();
                                        let offline = prescription()
                                            .map(|found| (found, profile.clone()));
                                        spawn(async move {
                                            match sign_prescription_dispatch(
                                                &preview_id, false, offline,
                                            )
                                            .await
                                            {
                                                Ok(signed) => {
                                                    signing.set(false);
                                                    sync.set(probe_sync_status().await);
                                                    prescription.set(Some(signed));
                                                    generation += 1;
                                                }
                                                Err(error) => {
                                                    signing.set(false);
                                                    if error.contains("variables desconocidas") {
                                                        needs_ack.set(true);
                                                    }
                                                    message.set(error);
                                                }
                                            }
                                        });
                                    }
                                },
                                if signing() { "Firmando..." } else { "Firmar y emitir" }
                            }
                            if needs_ack() {
                                button {
                                    class: "secondary",
                                    onclick: {
                                        let preview_id = preview_id.clone();
                                        move |_| {
                                            let preview_id = preview_id.clone();
                                            let offline = prescription().map(|found| {
                                                (found, profile_for_ack.clone())
                                            });
                                            spawn(async move {
                                                match sign_prescription_dispatch(
                                                    &preview_id, true, offline,
                                                )
                                                .await
                                                {
                                                    Ok(signed) => {
                                                        needs_ack.set(false);
                                                        sync.set(probe_sync_status().await);
                                                        prescription.set(Some(signed));
                                                        generation += 1;
                                                    }
                                                    Err(error) => message.set(error),
                                                }
                                            });
                                        }
                                    },
                                    "Firmar reconociendo las advertencias"
                                }
                            }
                        }
                    } else {
                        a {
                            class: "button",
                            href: "{pdf_url}",
                            "Descargar PDF"
                        }
                        a {
                            class: "button secondary",
                            href: "{pdf_url}",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            "Imprimir"
                        }
                        if current.provisional {
                            button {
                                class: "secondary",
                                onclick: move |_| {
                                    let inputs_doc = pdf_doc.clone();
                                    let inputs_values = pdf_values.clone();
                                    let inputs_meds = pdf_meds.clone();
                                    let inputs_number = pdf_rx;
                                    let inputs_issued = pdf_issued.clone();
                                    let inputs_title = format!("Receta-{preview_id}.pdf");
                                    spawn(async move {
                                        #[cfg(any(feature = "server", feature = "desktop"))]
                                        {
                                            #[cfg(feature = "desktop")]
                                            let cached_assets =
                                                desktop_local::cached_doc_assets(&inputs_doc);
                                            #[cfg(not(feature = "desktop"))]
                                            let cached_assets: Vec<(String, Vec<u8>)> = Vec::new();
                                            let input = pdf::PdfInput {
                                                doc_json: inputs_doc,
                                                values: inputs_values,
                                                medications: inputs_meds,
                                                assets: cached_assets,
                                                margin_mm: 12,
                                                rx_number: inputs_number,
                                                issued_at: Some(inputs_issued),
                                                mode: pdf::RenderMode::Provisional,
                                                title: inputs_title,
                                            };
                                            match pdf::render(&input) {
                                                Ok(bytes) => {
                                                    message.set(format!(
                                                        "PDF provisional generado ({} bytes) con marca PENDIENTE DE SINCRONIZACIÓN.",
                                                        bytes.len()
                                                    ));
                                                }
                                                Err(error) => message.set(error),
                                            }
                                        }
                                    });
                                },
                                "PDF provisional (local)"
                            }
                        }
                        {
                            #[cfg(target_arch = "wasm32")]
                            rsx! {
                                button {
                                    class: "secondary",
                                    onclick: {
                                        let pdf_url = pdf_url.clone();
                                        move |_| {
                                            let pdf_url = pdf_url.clone();
                                            spawn(async move {
                                                if let Some(window) = web_sys::window() {
                                                    let origin = window
                                                        .location()
                                                        .origin()
                                                        .unwrap_or_default();
                                                    let data = web_sys::ShareData::new();
                                                    data.set_title("Receta médica");
                                                    data.set_url(&format!("{origin}{pdf_url}"));
                                                    let _ = window.navigator().share_with_data(&data);
                                                }
                                            });
                                        }
                                    },
                                    "Compartir"
                                }
                            }
                        }
                    }
                }
            }
            if current.provisional {
                div {
                    class: "notice",
                    p { "Documento provisional: se muestra sin QR escaneable hasta sincronizar." }
                }
            }
            if !message().is_empty() {
                p { class: "error", "{message}" }
            }
            div {
                class: "preview-scroll",
                forms::A4Preview {
                    doc_json: current.doc_json.clone(),
                    values,
                    margin_mm: current.margin_mm,
                    medications: Some(current.medications.clone()),
                    meta: Some(meta),
                    image_sources,
                }
            }
        }
    }
}

#[component]
fn PrescriptionDetail(id: String) -> Element {
    let user = use_context::<Signal<Option<SessionUser>>>();
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let mut sync = use_context::<Signal<SyncStatus>>();
    let navigator = use_navigator();
    let mut prescription = use_signal(|| None::<api::Prescription>);
    let mut items = use_signal(Vec::<api::PrescriptionItemForm>::new);
    let mut block = use_signal(|| None::<api::RxBlockStatus>);
    let mut message = use_signal(String::new);
    let mut draft_diagnosis = use_signal(String::new);
    let mut draft_validity = use_signal(String::new);
    let mut draft_instructions = use_signal(String::new);
    let mut initialized = use_signal(|| false);
    let mut saving = use_signal(|| false);
    let mut signing = use_signal(|| false);
    let mut needs_ack = use_signal(|| false);
    let mut void_reason = use_signal(String::new);
    let mut show_void = use_signal(|| false);
    let prescription_id = use_signal(|| id.clone());

    use_effect(move || {
        if !checked() {
            return;
        }
        if user().is_none() {
            navigator.replace(Route::AuthPage {});
            return;
        }
        let load_id = prescription_id();
        spawn(async move {
            match api::fetch_prescription(&load_id).await {
                Ok(found) => {
                    draft_diagnosis.set(found.diagnosis.clone().unwrap_or_default());
                    draft_validity.set(found.validity.clone().unwrap_or_default());
                    draft_instructions.set(found.instructions.clone().unwrap_or_default());
                    items.set(
                        found
                            .items
                            .iter()
                            .map(|item| api::PrescriptionItemForm {
                                id: Some(item.id.clone()),
                                med_name: item.med_name.clone(),
                                dose: item.dose.clone().unwrap_or_default(),
                                route: item.route.clone().unwrap_or_default(),
                                frequency: item.frequency.clone().unwrap_or_default(),
                                duration: item.duration.clone().unwrap_or_default(),
                                quantity: item.quantity.clone().unwrap_or_default(),
                                notes: item.notes.clone().unwrap_or_default(),
                            })
                            .collect(),
                    );
                    prescription.set(Some(found));
                }
                Err(error) => {
                    #[cfg(feature = "desktop")]
                    if let Ok(mut conn) = desktop_local::open_local_db() {
                        if let Ok(found) = desktop_local::list_prescriptions(&mut conn, "") {
                            for value in found {
                                if value.get("id").and_then(Value::as_str) != Some(load_id.as_str())
                                {
                                    continue;
                                }
                                if let Ok(mapped) =
                                    serde_json::from_value::<api::Prescription>(value)
                                {
                                    draft_diagnosis
                                        .set(mapped.diagnosis.clone().unwrap_or_default());
                                    draft_validity.set(mapped.validity.clone().unwrap_or_default());
                                    draft_instructions
                                        .set(mapped.instructions.clone().unwrap_or_default());
                                    let local_items = desktop_local::local_prescription_items(
                                        &mut conn, &load_id,
                                    )
                                    .ok()
                                    .unwrap_or_default();
                                    items.set(
                                        local_items
                                            .into_iter()
                                            .map(|item| api::PrescriptionItemForm {
                                                id: item
                                                    .get("id")
                                                    .and_then(Value::as_str)
                                                    .map(str::to_owned),
                                                med_name: item
                                                    .get("medName")
                                                    .and_then(Value::as_str)
                                                    .unwrap_or_default()
                                                    .to_owned(),
                                                dose: item
                                                    .get("dose")
                                                    .and_then(Value::as_str)
                                                    .unwrap_or_default()
                                                    .to_owned(),
                                                route: item
                                                    .get("route")
                                                    .and_then(Value::as_str)
                                                    .unwrap_or_default()
                                                    .to_owned(),
                                                frequency: item
                                                    .get("frequency")
                                                    .and_then(Value::as_str)
                                                    .unwrap_or_default()
                                                    .to_owned(),
                                                duration: item
                                                    .get("duration")
                                                    .and_then(Value::as_str)
                                                    .unwrap_or_default()
                                                    .to_owned(),
                                                quantity: item
                                                    .get("quantity")
                                                    .and_then(Value::as_str)
                                                    .unwrap_or_default()
                                                    .to_owned(),
                                                notes: item
                                                    .get("notes")
                                                    .and_then(Value::as_str)
                                                    .unwrap_or_default()
                                                    .to_owned(),
                                            })
                                            .collect(),
                                    );
                                    prescription.set(Some(mapped));
                                }
                                break;
                            }
                        }
                    }
                    if prescription().is_none() {
                        message.set(error);
                    }
                }
            }
            if let Ok(status) = api::fetch_rx_block_status().await {
                block.set(Some(status));
            }
            initialized.set(true);
        });
    });

    if !checked() {
        return rsx! {
            div { id: "prescriptions-page", p { "Cargando..." } }
        };
    }
    let Some(profile) = actor() else {
        return rsx! {
            div { id: "prescriptions-page", p { "Redirigiendo..." } }
        };
    };
    let Some(current) = prescription() else {
        return rsx! {
            div {
                id: "prescriptions-page",
                if message().is_empty() && !initialized() {
                    p { "Cargando..." }
                } else {
                    p { class: "error", "{message}" }
                    Link { to: Route::Prescriptions { status: None }, "Volver a recetas" }
                }
            }
        };
    };
    let is_draft = current.status == "draft";
    let profile_for_ack = profile.clone();
    let profile_for_allocate = profile.clone();
    let profile_for_supersede = profile.clone();
    let profile_for_void = profile.clone();
    let can_edit = is_draft && profile.can_create_prescriptions();
    let can_sign = is_draft
        && profile.can_sign_prescriptions()
        && current.doctor_id.as_ref().is_none_or(|owner| {
            profile
                .doctor
                .as_ref()
                .is_some_and(|doctor| &doctor.id == owner)
        });
    let can_transition = !is_draft
        && current.status == "signed"
        && profile.can_sign_prescriptions()
        && current.doctor_id.as_ref().is_some_and(|owner| {
            profile
                .doctor
                .as_ref()
                .is_some_and(|doctor| &doctor.id == owner)
        });
    let block_state = block();
    let llm_template = profile
        .clinic
        .as_ref()
        .and_then(|clinic| clinic.llm_url_template.clone());

    rsx! {
        div {
            id: "prescriptions-page",
            div {
                class: "page-head",
                div {
                    h4 { "{current.patient_name}" }
                    p {
                        "{prescription_status_label(&current.status)}"
                        if let Some(number) = current.rx_number {
                            " · No. {number}"
                        }
                        " · {format_date(&current.created_at)}"
                    }
                }
                div {
                    class: "actions",
                    Link {
                        class: "button secondary",
                        to: Route::PrescriptionPreview { id: current.id.clone() },
                        "Vista previa A4"
                    }
                    Link { class: "button secondary", to: Route::Prescriptions { status: None }, "Volver" }
                }
            }
            div {
                class: "block-indicator",
                if let Some(status) = block_state.as_ref() {
                    if status.active {
                        span { "Números disponibles: {status.remaining} de {status.limit}" }
                    } else {
                        span { "Sin bloque de números activo." }
                        button {
                            class: "secondary",
                            onclick: move |_| {
                                let profile_clone = profile_for_allocate.clone();
                                let _ = &profile_clone;
                                spawn(async move {
                                    match api::allocate_rx_block().await {
                                        Ok(info) => {
                                            let _ = &info;
                                            #[cfg(feature = "desktop")]
                                            {
                                                if let Ok(mut conn) = desktop_local::open_local_db() {
                                                    let value = serde_json::json!({
                                                        "id": info.id,
                                                        "clinic_id": profile_clone.clinic.as_ref().map(|clinic| clinic.id.clone()).unwrap_or_default(),
                                                        "device_id": info.device_id,
                                                        "start_number": info.start_number,
                                                        "end_number": info.end_number,
                                                        "consumed": info.consumed,
                                                        "status": info.status,
                                                        "allocated_at": info.allocated_at,
                                                        "created_at": info.allocated_at,
                                                        "updated_at": info.allocated_at,
                                                        "created_by": profile_clone.profile.id,
                                                    });
                                                    let _ = desktop_local::cache_rx_block(&mut conn, &value);
                                                    let _ = desktop_local::set_meta(&mut conn, "active_rx_block", &info.id);
                                                }
                                            }
                                            if let Ok(status) = api::fetch_rx_block_status().await {
                                                block.set(Some(status));
                                            }
                                        }
                                        Err(error) => message.set(error),
                                    }
                                });
                            },
                            "Reservar bloque"
                        }
                    }
                    if status.exhausted {
                        p { class: "warning", "Bloque agotado. Reserva otro antes de firmar." }
                    }
                }
            }
            if !message().is_empty() {
                p { class: "error", "{message}" }
            }
            if can_edit {
                div {
                    class: "form-grid",
                    label {
                        class: "field",
                        span { "Diagnóstico" }
                        textarea {
                            value: "{draft_diagnosis}",
                            oninput: move |event| draft_diagnosis.set(event.value()),
                        }
                    }
                    label {
                        class: "field",
                        span { "Vigencia" }
                        input {
                            value: "{draft_validity}",
                            oninput: move |event| draft_validity.set(event.value()),
                        }
                    }
                    label {
                        class: "field wide",
                        span { "Indicaciones generales" }
                        textarea {
                            value: "{draft_instructions}",
                            oninput: move |event| draft_instructions.set(event.value()),
                        }
                    }
                }
                div {
                    class: "actions",
                    button {
                        disabled: saving(),
                        onclick: {
                            let current = current.clone();
                            let profile = profile.clone();
                            move |_| {
                                if saving() {
                                    return;
                                }
                                saving.set(true);
                                let current = current.clone();
                                let profile = profile.clone();
                                #[cfg(not(feature = "desktop"))]
                                let _ = &profile;
                                let diagnosis = draft_diagnosis();
                                let validity = draft_validity();
                                let instructions = draft_instructions();
                                spawn(async move {
                                    match api::update_prescription(
                                        &current.id,
                                        &diagnosis,
                                        &validity,
                                        &instructions,
                                    )
                                    .await
                                    {
                                        Ok(updated) => {
                                            saving.set(false);
                                            prescription.set(Some(updated));
                                        }
                                        Err(error) => {
                                            #[cfg(feature = "desktop")]
                                            match update_prescription_offline(
                                                &current,
                                                &profile,
                                                &diagnosis,
                                                &validity,
                                                &instructions,
                                            ) {
                                                Ok(updated) => {
                                                    saving.set(false);
                                                    prescription.set(Some(updated));
                                                    sync.set(probe_sync_status().await);
                                                    return;
                                                }
                                                Err(_) => {}
                                            }
                                            saving.set(false);
                                            message.set(error);
                                        }
                                    }
                                });
                            }
                        },
                        if saving() { "Guardando..." } else { "Guardar borrador" }
                    }
                }
            } else {
                div {
                    class: "detail-grid",
                    div { class: "field", span { "Diagnóstico" }, span { "{current.diagnosis.clone().unwrap_or_else(|| \"—\".to_string())}" } }
                    div { class: "field", span { "Vigencia" }, span { "{current.validity.clone().unwrap_or_else(|| \"—\".to_string())}" } }
                    div { class: "field wide", span { "Indicaciones" }, span { class: "notes", "{current.instructions.clone().unwrap_or_else(|| \"—\".to_string())}" } }
                    if let Some(verification) = current.verification.as_ref() {
                        div { class: "field", span { "Código de verificación" }, span { "{verification.code}" } }
                        div { class: "field", span { "Estado del código" }, span { "{verification.status}" } }
                    }
                    if let Some(reason) = current.void_reason.as_ref() {
                        div { class: "field wide", span { "Motivo de anulación" }, span { "{reason}" } }
                    }
                }
            }
            div {
                class: "medication-list",
                h4 { "Medicamentos" }
                if items().is_empty() {
                    p { class: "hint", "Sin medicamentos." }
                }
                for (index, item) in items().into_iter().enumerate() {
                    div {
                        key: "{index}",
                        class: "item-card",
                        if can_edit {
                            div {
                                class: "form-grid",
                                label {
                                    class: "field",
                                    span { "Medicamento" }
                                    input {
                                        value: "{item.med_name}",
                                        oninput: move |event| {
                                            let value = event.value();
                                            items.with_mut(|list| {
                                                if let Some(entry) = list.get_mut(index) {
                                                    entry.med_name = value;
                                                }
                                            });
                                        },
                                    }
                                }
                                label {
                                    class: "field",
                                    span { "Dosis" }
                                    input {
                                        value: "{item.dose}",
                                        oninput: move |event| {
                                            let value = event.value();
                                            items.with_mut(|list| {
                                                if let Some(entry) = list.get_mut(index) {
                                                    entry.dose = value;
                                                }
                                            });
                                        },
                                    }
                                }
                                label {
                                    class: "field",
                                    span { "Vía" }
                                    input {
                                        value: "{item.route}",
                                        oninput: move |event| {
                                            let value = event.value();
                                            items.with_mut(|list| {
                                                if let Some(entry) = list.get_mut(index) {
                                                    entry.route = value;
                                                }
                                            });
                                        },
                                    }
                                }
                                label {
                                    class: "field",
                                    span { "Frecuencia" }
                                    input {
                                        value: "{item.frequency}",
                                        oninput: move |event| {
                                            let value = event.value();
                                            items.with_mut(|list| {
                                                if let Some(entry) = list.get_mut(index) {
                                                    entry.frequency = value;
                                                }
                                            });
                                        },
                                    }
                                }
                                label {
                                    class: "field",
                                    span { "Duración" }
                                    input {
                                        value: "{item.duration}",
                                        oninput: move |event| {
                                            let value = event.value();
                                            items.with_mut(|list| {
                                                if let Some(entry) = list.get_mut(index) {
                                                    entry.duration = value;
                                                }
                                            });
                                        },
                                    }
                                }
                                label {
                                    class: "field",
                                    span { "Cantidad" }
                                    input {
                                        value: "{item.quantity}",
                                        oninput: move |event| {
                                            let value = event.value();
                                            items.with_mut(|list| {
                                                if let Some(entry) = list.get_mut(index) {
                                                    entry.quantity = value;
                                                }
                                            });
                                        },
                                    }
                                }
                                label {
                                    class: "field wide",
                                    span { "Notas" }
                                    input {
                                        value: "{item.notes}",
                                        oninput: move |event| {
                                            let value = event.value();
                                            items.with_mut(|list| {
                                                if let Some(entry) = list.get_mut(index) {
                                                    entry.notes = value;
                                                }
                                            });
                                        },
                                    }
                                }
                            }
                            div {
                                class: "actions",
                                button {
                                    onclick: {
                                        let id = prescription_id();
                                        let form = item.clone();
                                        let current = current.clone();
                                        let profile = profile.clone();
                                        move |_| {
                                            let id = id.clone();
                                            let form = form.clone();
                                            let current = current.clone();
                                            let profile = profile.clone();
                                            #[cfg(not(feature = "desktop"))]
                                            {
                                                let _ = &current;
                                                let _ = &profile;
                                            }
                                            spawn(async move {
                                                match api::upsert_prescription_item(&id, &form).await {
                                                    Ok(_) => reload_prescription(
                                                        prescription,
                                                        items,
                                                        message,
                                                        id,
                                                    ),
                                                    Err(error) => {
                                                        #[cfg(feature = "desktop")]
                                                        match upsert_prescription_item_offline(
                                                            &current, &profile, &form,
                                                        ) {
                                                            Ok(()) => {
                                                                sync.set(probe_sync_status().await);
                                                                reload_prescription(
                                                                    prescription,
                                                                    items,
                                                                    message,
                                                                    id,
                                                                );
                                                                return;
                                                            }
                                                            Err(_) => {}
                                                        }
                                                        message.set(error);
                                                    }
                                                }
                                            });
                                        }
                                    },
                                    "Guardar medicamento"
                                }
                                button {
                                    class: "secondary",
                                    disabled: index == 0,
                                    onclick: {
                                        let id = prescription_id();
                                        move |_| {
                                            let mut order = items()
                                                .iter()
                                                .filter_map(|entry| entry.id.clone())
                                                .collect::<Vec<_>>();
                                            if index == 0 || order.len() <= index {
                                                return;
                                            }
                                            order.swap(index, index - 1);
                                            let id = id.clone();
                                            spawn(async move {
                                                match api::reorder_prescription_items(&id, &order).await {
                                                    Ok(_) => reload_prescription(
                                                        prescription,
                                                        items,
                                                        message,
                                                        id,
                                                    ),
                                                    Err(error) => message.set(error),
                                                }
                                            });
                                        }
                                    },
                                    "Subir"
                                }
                                button {
                                    class: "secondary",
                                    disabled: index + 1 >= items().len(),
                                    onclick: {
                                        let id = prescription_id();
                                        move |_| {
                                            let mut order = items()
                                                .iter()
                                                .filter_map(|entry| entry.id.clone())
                                                .collect::<Vec<_>>();
                                            if index + 1 >= order.len() {
                                                return;
                                            }
                                            order.swap(index, index + 1);
                                            let id = id.clone();
                                            spawn(async move {
                                                match api::reorder_prescription_items(&id, &order).await {
                                                    Ok(_) => reload_prescription(
                                                        prescription,
                                                        items,
                                                        message,
                                                        id,
                                                    ),
                                                    Err(error) => message.set(error),
                                                }
                                            });
                                        }
                                    },
                                    "Bajar"
                                }
                                button {
                                    class: "danger",
                                    onclick: {
                                        let id = prescription_id();
                                        let item_id = item.id.clone().unwrap_or_default();
                                        move |_| {
                                            if item_id.is_empty() {
                                                return;
                                            }
                                            let id = id.clone();
                                            let item_id = item_id.clone();
                                            spawn(async move {
                                                match api::delete_prescription_item(&id, &item_id).await {
                                                    Ok(()) => reload_prescription(
                                                        prescription,
                                                        items,
                                                        message,
                                                        id,
                                                    ),
                                                    Err(error) => message.set(error),
                                                }
                                            });
                                        }
                                    },
                                    "Eliminar"
                                }
                            }
                        } else {
                            div {
                                class: "item-read",
                                p { class: "patient-name", "{item.med_name}" }
                                p { class: "patient-meta",
                                    "{item.dose} · {item.route} · {item.frequency} · {item.duration}"
                                }
                                if !item.notes.is_empty() {
                                    p { class: "patient-meta", "{item.notes}" }
                                }
                            }
                        }
                        if let Some(url) = llm_explain_url(llm_template.as_deref(), &item.med_name) {
                            div {
                                class: "llm-link",
                                a {
                                    href: "{url}",
                                    target: "_blank",
                                    rel: "noopener noreferrer",
                                    "Explicar con LLM"
                                }
                                p { class: "hint", "Se abre un sitio externo con el nombre del medicamento. No se envían datos del paciente." }
                            }
                        }
                    }
                }
                if can_edit {
                    div {
                        class: "actions",
                        button {
                            class: "secondary",
                            onclick: move |_| {
                                items.with_mut(|list| list.push(new_prescription_item_form()));
                            },
                            "Agregar medicamento"
                        }
                    }
                }
            }
            if can_sign {
                div {
                    class: "actions",
                    button {
                        disabled: signing(),
                        onclick: move |_| {
                            if signing() {
                                return;
                            }
                            signing.set(true);
                            needs_ack.set(false);
                            let id = prescription_id();
                            let offline =
                                prescription().map(|found| (found, actor().unwrap_or(profile.clone())));
                            spawn(async move {
                                match sign_prescription_dispatch(&id, false, offline).await {
                                    Ok(signed) => {
                                        signing.set(false);
                                        sync.set(probe_sync_status().await);
                                        prescription.set(Some(signed));
                                    }
                                    Err(error) => {
                                        signing.set(false);
                                        if error.contains("variables desconocidas") {
                                            needs_ack.set(true);
                                        }
                                        message.set(error);
                                    }
                                }
                            });
                        },
                        if signing() { "Firmando..." } else { "Firmar receta" }
                    }
                    if needs_ack() {
                        button {
                            class: "secondary",
                            onclick: move |_| {
                                let id = prescription_id();
                            let offline =
                                prescription().map(|found| (found, actor().unwrap_or(profile_for_ack.clone())));
                            spawn(async move {
                                match sign_prescription_dispatch(&id, true, offline).await {
                                    Ok(signed) => {
                                        needs_ack.set(false);
                                        prescription.set(Some(signed));
                                    }
                                    Err(error) => message.set(error),
                                }
                            });
                            },
                            "Firmar reconociendo las advertencias"
                        }
                    }
                }
                if block_state.as_ref().is_some_and(|status| status.exhausted) {
                    p { class: "warning", "Sin números disponibles: reserva un bloque para firmar." }
                }
            }
            if can_transition {
                div {
                    class: "actions",
                    button {
                        class: "secondary",
                        onclick: move |_| {
                            let id = prescription_id();
                            let profile = profile_for_supersede.clone();
                            spawn(async move {
                                match supersede_prescription_dispatch(&id, prescription().map(|found| (found, profile.clone()))).await {
                                    Ok(corrected) => {
                                        navigator.push(Route::PrescriptionDetail {
                                            id: corrected.id,
                                        });
                                    }
                                    Err(error) => message.set(error),
                                }
                            });
                        },
                        "Reemplazar (corrección)"
                    }
                    button {
                        class: "danger",
                        onclick: move |_| show_void.set(true),
                        "Anular"
                    }
                }
                if show_void() {
                    div {
                        class: "notice",
                        label {
                            class: "field",
                            span { "Motivo de anulación" }
                            input {
                                value: "{void_reason}",
                                oninput: move |event| void_reason.set(event.value()),
                            }
                        }
                        div {
                            class: "actions",
                            button {
                                class: "danger",
                                onclick: move |_| {
                                    let id = prescription_id();
                                    let reason = void_reason();
                                    let profile = profile_for_void.clone();
                                    spawn(async move {
                                        match void_prescription_dispatch(&id, &reason, prescription().map(|found| (found, profile.clone()))).await {
                                            Ok(voided) => {
                                                show_void.set(false);
                                                prescription.set(Some(voided));
                                            }
                                            Err(error) => message.set(error),
                                        }
                                    });
                                },
                                "Confirmar anulación"
                            }
                            button {
                                class: "secondary",
                                onclick: move |_| show_void.set(false),
                                "Cancelar"
                            }
                        }
                    }
                }
            }
        }
    }
}
