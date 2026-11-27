use crate::desktop_local as local;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde_json::{json, Value};

fn database() -> SqliteConnection {
    let mut conn = SqliteConnection::establish(":memory:").unwrap();
    local::initialize_test_db(&mut conn);
    conn
}

fn identity(conn: &mut SqliteConnection, user: &str, role: &str, doctor: Option<&str>, clinic: &str) {
    local::set_meta(conn, "session_token", "token").unwrap();
    local::set_meta(conn, "session_expires_at", "2099-01-01T00:00:00Z").unwrap();
    local::set_meta(conn, "session_user", &json!({"id": user}).to_string()).unwrap();
    local::set_meta(conn, "session_profile", &json!({
        "profile": {"id": user}, "role": role, "clinic": {"id": clinic},
        "doctor": doctor.map(|id| json!({"id": id}))
    }).to_string()).unwrap();
}

fn prescription(id: &str, clinic: &str, doctor: Option<&str>, status: &str) -> Value {
    json!({"id": id, "patient_id": "patient", "clinic_id": clinic,
        "doctor_id": doctor, "template_version_id": "template", "status": status,
        "diagnosis": format!("diagnosis-{id}"), "client_id": id,
        "recetario_type": "ambulatorio", "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-01T00:00:00Z", "created_by": "author"})
}

fn cache(conn: &mut SqliteConnection, id: &str, doctor: Option<&str>, status: &str) {
    local::cache_prescription(conn, &prescription(id, "clinic", doctor, status)).unwrap();
}

#[test]
fn roles_identity_and_items_fail_closed() {
    let mut conn = database();
    identity(&mut conn, "nurse", "nursing", None, "clinic");
    cache(&mut conn, "a-draft", Some("a"), "draft");
    cache(&mut conn, "b-signed", Some("b"), "signed");
    cache(&mut conn, "unassigned", None, "draft");
    local::cache_prescription_item(&mut conn, &json!({"id":"item", "prescription_id":"a-draft", "position":1, "med_name":"medicine", "created_at":"2026", "updated_at":"2026"})).unwrap();
    assert_eq!(local::list_prescriptions(&mut conn, "").unwrap().len(), 3);
    assert_eq!(local::local_prescription_items(&mut conn, "a-draft").unwrap().len(), 1);
    assert!(local::offline_void(&mut conn, "b-signed", "reason", "nurse").is_err());
    identity(&mut conn, "doctor-a", "doctor", Some("a"), "clinic");
    assert!(local::list_prescriptions(&mut conn, "").unwrap().is_empty());
    assert!(local::local_prescription_items(&mut conn, "a-draft").is_err());
    cache(&mut conn, "a-draft", Some("a"), "draft");
    cache(&mut conn, "unassigned", None, "draft");
    assert!(local::cache_prescription(&mut conn, &prescription("b-signed", "clinic", Some("b"), "signed")).is_err());
    assert_eq!(local::list_prescriptions(&mut conn, "draft").unwrap().len(), 2);
    assert!(local::get_prescription(&mut conn, "b-signed").unwrap().is_none());
    assert!(local::update_local_prescription(&mut conn, "a-draft", "changed", "", "", None, "", "", "doctor-a").is_ok());
    assert!(local::update_local_prescription(&mut conn, "a-draft", "bad", "", "", None, "", "", "other").is_err());
    identity(&mut conn, "doctor-a", "doctor", None, "clinic");
    assert!(local::list_prescriptions(&mut conn, "").unwrap().is_empty());
    identity(&mut conn, "doctor-a", "doctor", Some("a"), "other-clinic");
    assert!(local::list_prescriptions(&mut conn, "").unwrap().is_empty());
    local::set_meta(&mut conn, "session_profile", "null").unwrap();
    assert!(local::list_prescriptions(&mut conn, "").unwrap().is_empty());
}

#[test]
fn unsynced_work_and_outbox_survive_identity_changes() {
    let mut conn = database();
    identity(&mut conn, "a", "doctor", Some("doctor-a"), "clinic");
    let created = local::create_local_prescription(&mut conn, "patient", "template", "clinic", Some("doctor-a"), "private", "", "", "a").unwrap();
    assert!(local::get_prescription(&mut conn, &created.id).unwrap().is_some());
    assert!(local::upsert_local_prescription_item(&mut conn, &created.id, None, "medicine", "", "", "", "", "", None, "", "a").is_ok());
    let before = local::outbox_rows(&mut conn).unwrap();
    identity(&mut conn, "b", "doctor", Some("doctor-b"), "clinic");
    assert!(local::get_prescription(&mut conn, &created.id).unwrap().is_none());
    assert!(local::update_local_prescription(&mut conn, &created.id, "bad", "", "", None, "", "", "b").is_err());
    assert!(local::offline_sign(&mut conn, &created.id, "device", "doctor-b", "b").is_err());
    assert_eq!(local::outbox_rows(&mut conn).unwrap(), before);
    identity(&mut conn, "a", "doctor", Some("doctor-a"), "clinic");
    assert_eq!(local::get_prescription(&mut conn, &created.id).unwrap().unwrap()["diagnosis"], "private");
    assert_eq!(local::local_prescription_items(&mut conn, &created.id).unwrap().len(), 1);
    local::mark_outbox_failed(&mut conn, &created.outbox_id, "rejected").unwrap();
    assert!(local::outbox_rows(&mut conn).unwrap().iter().any(|row| row["id"] == created.outbox_id && row["status"] == "failed" && row["lastError"] == "rejected"));
}

#[test]
fn secretary_and_administrator_read_only() {
    let mut conn = database();
    identity(&mut conn, "secretary", "secretary", None, "clinic");
    assert!(local::cache_prescription(&mut conn, &prescription("draft", "clinic", None, "draft")).is_err());
    cache(&mut conn, "signed", Some("a"), "signed");
    cache(&mut conn, "voided", Some("a"), "voided");
    cache(&mut conn, "superseded", Some("b"), "superseded");
    assert_eq!(local::list_prescriptions(&mut conn, "").unwrap().len(), 3);
    assert!(local::offline_void(&mut conn, "signed", "reason", "secretary").is_err());
    identity(&mut conn, "admin", "administrator", None, "clinic");
    cache(&mut conn, "draft", Some("b"), "draft");
    cache(&mut conn, "signed", Some("a"), "signed");
    assert_eq!(local::list_prescriptions(&mut conn, "").unwrap().len(), 2);
    assert!(local::update_local_prescription(&mut conn, "draft", "bad", "", "", None, "", "", "admin").is_err());
    assert!(local::create_local_prescription(&mut conn, "patient", "template", "clinic", None, "", "", "", "admin").is_err());
}

#[test]
fn draft_signing_and_transitions_require_owner_doctor() {
    let mut conn = database();
    local::cache_rx_block(&mut conn, &json!({
        "id":"block", "clinic_id":"clinic", "device_id":"device",
        "start_number":10, "end_number":20, "consumed":0, "status":"active",
        "allocated_at":"2026", "created_at":"2026", "updated_at":"2026"
    })).unwrap();
    identity(&mut conn, "nurse", "nursing", None, "clinic");
    cache(&mut conn, "handoff", None, "draft");
    let nursing_update = local::update_local_prescription(&mut conn, "handoff", "nursing", "", "", None, "", "", "nurse").unwrap();
    assert!(local::offline_sign(&mut conn, "handooff", "device", "a", "nurse").is_err());
    assert_eq!(local::active_block_remaining(&mut conn, "device").unwrap(), 11);
    identity(&mut conn, "doctor-a", "doctor", Some("a"), "clinic");
    local::mark_outbox_synced(&mut conn, &nursing_update).unwrap();
    cache(&mut conn, "handoff", None, "draft");
    assert!(local::offline_sign(&mut conn, "handoff", "device", "b", "doctor-a").is_err());
    assert_eq!(local::active_block_remaining(&mut conn, "device").unwrap(), 11);
    local::issued_snapshot_tests::seed_render_inputs(&mut conn, "doctor-a", "a");
    let signed = local::offline_sign(&mut conn, "handoff", "device", "a", "doctor-a").unwrap();
    assert_eq!(signed.rx_number, 10);
    let row = local::get_prescription(&mut conn, "handoff").unwrap().unwrap();
    assert_eq!(row["doctorId"], "a");
    assert_eq!(row["status"], "signed");
    assert!(local::update_local_prescription(&mut conn, "handoff", "bad", "", "", None, "", "", "doctor-a").is_err());
    assert!(local::offline_void(&mut conn, "handoff", "reason", "doctor-a").is_ok());
    assert_eq!(local::get_prescription(&mut conn, "handoff").unwrap().unwrap()["status"], "voided");
    assert!(local::offline_supersede(&mut conn, "handoff", "device", "a", "doctor-a").is_err());
    assert_eq!(local::active_block_remaining(&mut conn, "device").unwrap(), 10);
}

#[test]
fn pending_and_failed_item_operations_protect_prescription_from_refresh_and_eviction() {
    let mut conn = database();
    identity(&mut conn, "doctor-a", "doctor", Some("a"), "clinic");
    cache(&mut conn, "draft", Some("a"), "draft");
    let item = local::upsert_local_prescription_item(&mut conn, "draft", None, "medicine", "", "", "", "", "", None, "", "doctor-a").unwrap();
    assert!(local::has_pending_outbox(&mut conn, "draft").unwrap());
    let rows = local::outbox_rows(&mut conn).unwrap();
    let operation = rows.iter().find(|row| row["entityId"] == item).unwrap();
    let outbox_id = operation["id"].as_str().unwrap();
    local::mark_outbox_failed(&mut conn, outbox_id, "rejected").unwrap();
    assert!(local::has_pending_outbox(&mut conn, "draft").unwrap());
    diesel::sql_query("UPDATE prescriptions SET synced_at = '2026-01-01T00:00:00Z' WHERE id = 'draft'").execute(&mut conn).unwrap();
    assert!(!local::evict_local_cache_row(&mut conn, "prescriptions", "draft").unwrap());
    assert_eq!(local::local_prescription_items(&mut conn, "draft").unwrap().len(), 1);
    local::mark_outbox_synced(&mut conn, outbox_id).unwrap();
    assert!(!local::has_pending_outbox(&mut conn, "draft").unwrap());
}

#[test]
fn legacy_rows_require_reauthorization_without_deleting_work() {
    let mut conn = database();
    identity(&mut conn, "doctor-a", "doctor", Some("a"), "clinic");
    cache(&mut conn, "legacy", Some("a"), "draft");
    diesel::sql_query("DELETE FROM sync_meta WHERE key = 'prescription_scope:legacy'")
        .execute(&mut conn).unwrap();
    let outbox = local::enqueue_outbox(&mut conn, "update_prescription", "prescription", "legacy", &json!({"prescriptionId":"legacy"})).unwrap();
    assert!(local::get_prescription(&mut conn, "legacy").unwrap().is_none());
    assert!(local::has_pending_outbox(&mut conn, "legacy").unwrap());
    assert!(local::outbox_rows(&mut conn).unwrap().iter().any(|row| row["id"] == outbox));
    local::mark_outbox_synced(&mut conn, &outbox).unwrap();
    cache(&mut conn, "legacy", Some("a"), "draft");
    assert_eq!(local::get_prescription(&mut conn, "legacy").unwrap().unwrap()["diagnosis"], "diagnosis-legacy");
    identity(&mut conn, "doctor-b", "doctor", Some("b"), "clinic");
    assert!(local::get_prescription(&mut conn, "legacy").unwrap().is_none());
}

#[test]
fn refresh_never_overwrites_pending_or_failed_local_changes() {
    let mut conn = database();
    identity(&mut conn, "doctor-a", "doctor", Some("a"), "clinic");
    cache(&mut conn, "draft", Some("a"), "draft");
    let item = local::upsert_local_prescription_item(&mut conn, "draft", None, "local medicine", "", "", "", "", "", None, "", "doctor-a").unwrap();
    let operation = local::outbox_rows(&mut conn).unwrap().into_iter()
        .find(|row| row["entityId"] == item).unwrap();
    let outbox_id = operation["id"].as_str().unwrap();
    let mut remote = prescription("draft", "clinic", Some("a"), "draft");
    remote["diagnosis"] = json!("remote");
    assert_eq!(local::list_prescriptions(&mut conn, "pending_sync").unwrap().len(), 1);
    assert!(local::cache_prescription(&mut conn, &remote).is_err());
    assert_eq!(local::local_prescription_items(&mut conn, "draft").unwrap()[0]["medName"], "local medicine");
    local::mark_outbox_failed(&mut conn, outbox_id, "server rejected").unwrap();
    assert_eq!(local::list_prescriptions(&mut conn, "failed").unwrap().len(), 1);
    assert!(local::cache_prescription(&mut conn, &remote).is_err());
    assert_eq!(local::get_prescription(&mut conn, "draft").unwrap().unwrap()["diagnosis"], "diagnosis-draft");
    identity(&mut conn, "nurse", "nursing", None, "clinic");
    assert!(local::cache_prescription(&mut conn, &remote).is_err());
    assert!(local::get_prescription(&mut conn, "draft").unwrap().is_none());
    local::mark_outbox_synced(&mut conn, outbox_id).unwrap();
    local::cache_prescription(&mut conn, &remote).unwrap();
    assert_eq!(local::get_prescription(&mut conn, "draft").unwrap().unwrap()["diagnosis"], "remote");
}

#[test]
fn create_replay_cannot_reassign_local_work_after_identity_or_role_change() {
    let mut conn = database();
    identity(&mut conn, "doctor-a", "doctor", Some("a"), "clinic");
    let created = local::create_local_prescription(&mut conn, "patient", "template", "clinic", Some("a"), "private", "", "", "doctor-a").unwrap();
    let row = local::outbox_rows(&mut conn).unwrap().into_iter().find(|row| row["id"] == created.outbox_id).unwrap();
    let payload: Value = serde_json::from_str(row["payloadJson"].as_str().unwrap()).unwrap();
    assert!(local::authorize_create_replay(&mut conn, &payload).is_ok());
    local::set_meta(&mut conn, "session_user", &json!({"id": "different-user"}).to_string()).unwrap();
    assert!(local::authorize_create_replay(&mut conn, &payload).is_err());
    identity(&mut conn, "doctor-a", "doctor", Some("b"), "clinic");
    assert!(local::authorize_create_replay(&mut conn, &payload).is_err());
    identity(&mut conn, "doctor-a", "nursing", None, "clinic");
    assert!(local::authorize_create_replay(&mut conn, &payload).is_err());
    identity(&mut conn, "doctor-a", "doctor", Some("a"), "other-clinic");
    assert!(local::authorize_create_replay(&mut conn, &payload).is_err());
    identity(&mut conn, "doctor-b", "doctor", Some("a"), "clinic");
    assert!(local::authorize_create_replay(&mut conn, &payload).is_err());
    identity(&mut conn, "doctor-a", "doctor", Some("a"), "clinic");
    assert_eq!(local::get_prescription(&mut conn, &created.id).unwrap().unwrap()["diagnosis"], "private");
    assert_eq!(local::outbox_rows(&mut conn).unwrap()[0]["status"], "pending");
}

#[tokio::test]
async fn server_refusal_cannot_be_mistaken_for_network_failure() {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = [0; 2048];
        let _ = stream.read(&mut request).unwrap();
        let body = r#"{"message":"No se pudo contactar con el servidor: denied"}"#;
        write!(stream, "HTTP/1.1 403 Forbidden\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body).unwrap();
    });
    let request = reqwest::Client::new().get(format!("http://{address}/"));
    let error = crate::http::send_typed(request).await.unwrap_err();
    server.join().unwrap();
    assert!(matches!(error, crate::http::RequestError::Response(_)));
    let mut conn = database();
    identity(&mut conn, "doctor-a", "doctor", Some("a"), "clinic");
    let created = local::create_local_prescription(&mut conn, "patient", "template", "clinic", Some("a"), "private", "", "", "doctor-a").unwrap();
    assert!(!crate::finish_prescription_replay(&mut conn, &created.outbox_id, 0, Err(error.clone())));
    let row = local::outbox_rows(&mut conn).unwrap().into_iter().find(|row| row["id"] == created.outbox_id).unwrap();
    assert_eq!(row["status"], "failed");
    assert!(row["lastError"].as_str().unwrap().contains("denied"));
    assert!(crate::open_cache_for_network_error(&error).await.is_err());
}

#[test]
fn local_pdf_requires_current_authorized_identity() {
    let mut conn = database();
    identity(&mut conn, "doctor-a", "doctor", Some("a"), "clinic");
    local::issued_snapshot_tests::seed_render_inputs(&mut conn, "doctor-a", "a");
    cache(&mut conn, "issued", Some("a"), "draft");
    local::cache_rx_block(&mut conn, &json!({
        "id":"pdf-block", "clinic_id":"clinic", "device_id":"device",
        "start_number":1, "end_number":2, "consumed":0, "status":"active",
        "allocated_at":"2026", "created_at":"2026", "updated_at":"2026"
    })).unwrap();
    local::offline_sign(&mut conn, "issued", "device", "a", "doctor-a").unwrap();
    let input = crate::pdf::PdfInput {
        doc_json: crate::forms::default_document_json(),
        values: vec![("patient.name".into(), "Example".into())],
        medications: Vec::new(),
        assets: Vec::new(),
        margin_mm: 12,
        rx_number: Some(1),
        issued_at: None,
        mode: crate::pdf::RenderMode::Provisional,
        title: "local.pdf".into(),
    };
    assert!(local::render_prescription_pdf(&mut conn, "issued", &input).unwrap().starts_with(b"%PDF-"));
    identity(&mut conn, "doctor-b", "doctor", Some("b"), "clinic");
    assert!(local::render_prescription_pdf(&mut conn, "issued", &input).is_err());
    local::set_meta(&mut conn, "session_profile", "null").unwrap();
    assert!(local::render_prescription_pdf(&mut conn, "issued", &input).is_err());
}

#[test]
fn local_preview_obeys_identity_changes() {
    let mut conn = database();
    identity(&mut conn, "doctor-a", "doctor", Some("a"), "clinic");
    cache(&mut conn, "draft", Some("a"), "draft");
    let profile = crate::api::Me {
        profile: crate::api::ProfileSummary {
            id: "doctor-a".into(),
            display_name: "Doctor A".into(),
            status: "active".into(),
        },
        clinic: None,
        role: Some("doctor".into()),
        doctor: None,
    };
    let preview = crate::build_local_preview(&mut conn, "draft", &profile).unwrap().unwrap();
    assert_eq!(preview.id, "draft");
    assert!(preview.provisional);
    assert!(preview.variables.iter().any(|variable| variable.value.contains("diagnosis-draft")));
    identity(&mut conn, "doctor-b", "doctor", Some("b"), "clinic");
    assert!(crate::build_local_preview(&mut conn, "draft", &profile).unwrap().is_none());
    identity(&mut conn, "doctor-a", "nursing", None, "clinic");
    assert!(crate::build_local_preview(&mut conn, "draft", &profile).unwrap().is_none());
    local::set_meta(&mut conn, "session_profile", "null").unwrap();
    assert!(crate::build_local_preview(&mut conn, "draft", &profile).unwrap().is_none());
}
