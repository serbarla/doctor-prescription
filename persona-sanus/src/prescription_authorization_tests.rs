use diesel::connection::SimpleConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::RunQueryDsl;
use diesel::SqliteConnection;
use diesel_migrations::MigrationHarness;

use crate::db;
use crate::service::{self, ActorContext, PrescriptionInput, PrescriptionItemInput, PrescriptionUpdate};

async fn fixture() -> db::DbPool {
    let manager = ConnectionManager::<SqliteConnection>::new(":memory:");
    let pool = Pool::builder().max_size(1).build(manager).unwrap();
    {
        let mut conn = pool.get().unwrap();
        conn.run_pending_migrations(db::MIGRATIONS).unwrap();
        conn.batch_execute("\
            INSERT INTO clinics(id,name,server_retention_until,created_at,updated_at) VALUES ('c','Clinic','2099','2026','2026'),('other','Other','2099','2026','2026');\
            INSERT INTO users(id,email,password_hash,display_name,created_at,updated_at) VALUES ('a','a','x','A','2026','2026'),('b','b','x','B','2026','2026'),('n','n','x','N','2026','2026'),('s','s','x','S','2026','2026'),('admin','admin','x','Admin','2026','2026'),('missing','missing','x','Missing','2026','2026'),('x','x','x','X','2026','2026');\
            INSERT INTO profiles(id,auth_user_id,display_name,created_at,updated_at) SELECT id,id,display_name,'2026','2026' FROM users;\
            INSERT INTO clinic_members(clinic_id,profile_id,role,created_at,updated_at) VALUES ('c','a','doctor','2026','2026'),('c','b','doctor','2026','2026'),('c','n','nursing','2026','2026'),('c','s','secretary','2026','2026'),('c','admin','administrator','2026','2026'),('c','missing','doctor','2026','2026'),('other','x','doctor','2026','2026');\
            INSERT INTO doctors(id,profile_id,created_at,updated_at) VALUES ('da','a','2026','2026'),('db','b','2026','2026'),('dx','x','2026','2026');\
            INSERT INTO patients(id,clinic_id,full_name,created_at,updated_at) VALUES ('p','c','Patient','2026','2026'),('px','other','Other','2026','2026');\
            INSERT INTO prescriptions(id,patient_id,clinic_id,doctor_id,status,diagnosis,client_id,created_at,updated_at) VALUES ('draft-a','p','c','da','draft','private A','client-a','2026-01','2026'),('draft-b','p','c','db','draft','private B','client-b','2026-02','2026'),('unassigned','p','c',NULL,'draft','handoff','client-n','2026-03','2026'),('signed-a','p','c','da','signed','issued A','client-sa','2026-04','2026'),('void-b','p','c','db','voided','issued B','client-vb','2026-05','2026'),('foreign','px','other','dx','signed','foreign','client-x','2026-06','2026');\
            INSERT INTO prescription_items(id,prescription_id,position,med_name,created_at,updated_at) VALUES ('item-b','draft-b',1,'medicine','2026','2026');\
        ").unwrap();
        conn.batch_execute("INSERT INTO templates(id,owner_type,owner_id,name,status,created_at,updated_at) VALUES ('t','clinic','c','Template','published','2026','2026'); INSERT INTO devices(id,profile_id,platform,created_at,updated_at) VALUES ('device-a','a','web','2026','2026'),('device-b','b','web','2026','2026');").unwrap();
        diesel::sql_query("INSERT INTO template_versions(id,template_id,version_no,doc_json,variables_spec,status,created_at,updated_at) VALUES ('v','t',1,?,'{}','published','2026','2026')")
            .bind::<diesel::sql_types::Text, _>(crate::forms::default_document_json())
            .execute(&mut *conn).unwrap();
        conn.batch_execute("UPDATE prescriptions SET template_version_id='v' WHERE clinic_id='c'; UPDATE prescriptions SET status='draft' WHERE id='signed-a';").unwrap();
        for id in ["a", "b", "n", "s", "admin", "missing", "x"] {
            diesel::sql_query("INSERT INTO sessions(id,token_hash,user_id,expires_at,created_at,device_id,mfa_pending) VALUES (?,?,?,?,?,?,0)")
                .bind::<diesel::sql_types::Text, _>(id)
                .bind::<diesel::sql_types::Text, _>(crate::auth_server::hash_token(id))
                .bind::<diesel::sql_types::Text, _>(id)
                .bind::<diesel::sql_types::Text, _>("2099-01-01T00:00:00Z")
                .bind::<diesel::sql_types::Text, _>("2026")
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(match id { "a" => Some("device-a"), "b" => Some("device-b"), _ => None })
                .execute(&mut *conn).unwrap();
        }
    }
    let mut author = actor(&pool, "a");
    author.device_id = Some("device-a".to_owned());
    service::sign_prescription(&pool, &author, "signed-a".to_owned(), service::PrescriptionSignInput {
        acknowledge_warnings: true, rx_number: None, rx_block_id: None, issued_snapshot: None,
    }).await.unwrap();
    pool
}

fn actor(pool: &db::DbPool, id: &str) -> ActorContext {
    let mut conn = pool.get().unwrap();
    let user = db::find_user_by_email(&mut conn, id).unwrap().unwrap();
    let profile = db::find_profile_by_auth_user_id(&mut conn, id).unwrap().unwrap();
    let member = db::find_membership_by_profile(&mut conn, id).unwrap();
    let clinic = db::find_clinic_by_id(&mut conn, &member.as_ref().unwrap().clinic_id).unwrap();
    let doctor = db::find_doctor_by_profile_id(&mut conn, id).unwrap();
    ActorContext { user, session_id: String::new(), device_id: None, profile, clinic, member, doctor }
}

fn update() -> PrescriptionUpdate {
    PrescriptionUpdate { diagnosis: Some("changed".into()), validity: None, recetario_type: None, validity_days: None, warning_signs: None, instructions: None }
}

fn item() -> PrescriptionItemInput {
    PrescriptionItemInput { item_id: Some("item-b".into()), med_name: "changed".into(), dose: None, route: None, frequency: None, duration: None, quantity: None, quantity_number: None, notes: None }
}

#[tokio::test]
async fn lists_and_direct_reads_obey_roles_and_identity() {
    let pool = fixture().await;
    for (id, expected) in [
        ("a", vec!["signed-a", "unassigned", "draft-a"]),
        ("b", vec!["void-b", "unassigned", "draft-b"]),
        ("s", vec!["void-b", "signed-a"]),
        ("n", vec!["void-b", "signed-a", "unassigned", "draft-b", "draft-a"]),
        ("admin", vec!["void-b", "signed-a", "unassigned", "draft-b", "draft-a"]),
    ] {
        let actor = actor(&pool, id);
        let rows = service::list_prescriptions(&pool, &actor, Some("p".into()), None, Some("2026".into())).await.unwrap();
        assert_eq!(rows.iter().map(|r| r.prescription.id.as_str()).collect::<Vec<_>>(), expected);
        for rx in ["draft-a", "draft-b", "unassigned", "signed-a", "void-b", "foreign"] {
            assert_eq!(service::get_prescription(&pool, &actor, rx.into()).await.is_ok(), expected.contains(&rx));
        }
    }
    let missing = actor(&pool, "missing");
    assert!(service::list_prescriptions(&pool, &missing, None, None, None).await.is_err());
    assert!(service::get_prescription(&pool, &missing, "unassigned".into()).await.is_err());
}

#[tokio::test]
async fn mutations_replay_and_documents_do_not_cross_doctor_boundary() {
    let pool = fixture().await;
    let a = actor(&pool, "a");
    assert!(service::update_prescription(&pool, &a, "draft-b".into(), update()).await.is_err());
    assert!(service::upsert_prescription_item(&pool, &a, "draft-b".into(), item()).await.is_err());
    assert!(service::delete_prescription_item(&pool, &a, "draft-b".into(), "item-b".into()).await.is_err());
    assert!(service::reorder_prescription_items(&pool, &a, "draft-b".into(), vec!["item-b".into()]).await.is_err());
    assert!(service::prescription_document(&pool, &a, "draft-b".into(), true).await.is_err());
    let replay = PrescriptionInput { patient_id: "p".into(), template_version_id: "unused".into(), client_id: Some("client-b".into()), diagnosis: None, validity: None, recetario_type: None, validity_days: None, warning_signs: None, instructions: None };
    assert!(service::create_prescription(&pool, &a, replay).await.is_err());
    assert!(service::update_prescription(&pool, &a, "unassigned".into(), update()).await.is_ok());
    let n = actor(&pool, "n");
    assert!(service::update_prescription(&pool, &n, "draft-b".into(), update()).await.is_ok());
    let s = actor(&pool, "s");
    assert!(service::update_prescription(&pool, &s, "draft-b".into(), update()).await.is_err());
    let mut conn = pool.get().unwrap();
    let saved = db::find_prescription_by_id(&mut conn, "c", "draft-b").unwrap().unwrap();
    assert_eq!(saved.diagnosis.as_deref(), Some("changed"));
    let saved_item = db::find_prescription_item(&mut conn, "draft-b", "item-b").unwrap().unwrap();
    assert_eq!(saved_item.med_name, "medicine");
}

fn headers(id: &str) -> axum::http::HeaderMap {
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(axum::http::header::COOKIE, format!("persona_session={id}").parse().unwrap());
    headers
}

fn state(pool: &db::DbPool) -> crate::auth_server::AppState {
    crate::auth_server::AppState { pool: pool.clone(), cookie_secure: false, assets: None }
}

async fn body(response: axum::response::Response) -> Vec<u8> {
    axum::body::to_bytes(response.into_body(), 2_000_000).await.unwrap().to_vec()
}

#[tokio::test]
async fn http_detail_preview_pdf_and_mutations_enforce_authorization() {
    use axum::extract::{Path, Query, State};
    use axum::Json;
    use serde_json::json;
    let pool = fixture().await;
    let state = state(&pool);
    let a = headers("a");
    let query = serde_json::from_value(json!({"patientId":"p","status":"draft","date":"2026"})).unwrap();
    let response = crate::api_server::list_prescriptions(State(state.clone()), a.clone(), Query(query)).await.unwrap();
    let list = String::from_utf8(body(response).await).unwrap();
    assert!(list.contains("draft-a") && list.contains("unassigned") && !list.contains("draft-b"));
    for id in ["draft-b", "foreign", "not-present"] {
        assert!(matches!(crate::api_server::get_prescription(State(state.clone()), a.clone(), Path(id.to_owned())).await, Err(crate::auth_server::AuthError::NotFound(_))));
        assert!(matches!(crate::pdf_server::prescription_preview(State(state.clone()), a.clone(), Path(id.to_owned())).await, Err(crate::auth_server::AuthError::NotFound(_))));
        assert!(matches!(crate::pdf_server::prescription_pdf(State(state.clone()), a.clone(), Path(id.to_owned())).await, Err(crate::auth_server::AuthError::NotFound(_))));
    }
    let preview = crate::pdf_server::prescription_preview(State(state.clone()), a.clone(), Path("unassigned".into())).await.unwrap();
    assert!(String::from_utf8(body(preview).await).unwrap().contains("handoff"));
    for who in ["a", "s"] {
        let pdf = crate::pdf_server::prescription_pdf(State(state.clone()), headers(who), Path("signed-a".into())).await.unwrap();
        assert_eq!(pdf.headers().get(axum::http::header::CONTENT_TYPE).unwrap(), "application/pdf");
        assert!(body(pdf).await.starts_with(b"%PDF-"));
    }
    let req = serde_json::from_value(json!({"diagnosis":"forbidden"})).unwrap();
    assert!(crate::api_server::update_prescription(State(state.clone()), a.clone(), Path("draft-b".into()), Json(req)).await.is_err());
    let req = serde_json::from_value(json!({"medName":"forbidden"})).unwrap();
    assert!(crate::api_server::upsert_prescription_item(State(state.clone()), a.clone(), Path("draft-b".into()), Json(req)).await.is_err());
    assert!(crate::api_server::delete_prescription_item(State(state.clone()), a.clone(), Path(("draft-b".into(), "item-b".into()))).await.is_err());
    let req = serde_json::from_value(json!({"itemIds":["item-b"]})).unwrap();
    assert!(crate::api_server::reorder_prescription_items(State(state.clone()), a.clone(), Path("draft-b".into()), Json(req)).await.is_err());
    let req = serde_json::from_value(json!({"patientId":"p","templateVersionId":"v","clientId":"client-b"})).unwrap();
    assert!(crate::api_server::create_prescription(State(state.clone()), a.clone(), Json(req)).await.is_err());
    let req = serde_json::from_value(json!({"acknowledgeWarnings":true})).unwrap();
    assert!(crate::api_server::sign_prescription(State(state.clone()), a.clone(), Path("draft-b".into()), Json(req)).await.is_err());
    let req = serde_json::from_value(json!({"voidReason":"reason"})).unwrap();
    assert!(crate::api_server::void_prescription(State(state.clone()), a.clone(), Path("void-b".into()), Json(req)).await.is_err());
    let req = serde_json::from_value(json!({"acknowledgeWarnings":true})).unwrap();
    assert!(crate::api_server::supersede_prescription(State(state.clone()), a, Path("void-b".into()), Json(req)).await.is_err());
}

#[tokio::test]
async fn handoff_sign_and_transitions_are_owner_only() {
    use axum::extract::{Path, State};
    use axum::Json;
    use serde_json::json;
    let pool = fixture().await;
    let state = state(&pool);
    let req = serde_json::from_value(json!({"acknowledgeWarnings":true})).unwrap();
    assert!(crate::api_server::sign_prescription(State(state.clone()), headers("n"), Path("unassigned".into()), Json(req)).await.is_err());
    let req = serde_json::from_value(json!({"acknowledgeWarnings":true})).unwrap();
    let signed = crate::api_server::sign_prescription(State(state.clone()), headers("a"), Path("unassigned".into()), Json(req)).await.unwrap();
    let signed = String::from_utf8(body(signed).await).unwrap();
    assert!(signed.contains("signed"));
    assert!(crate::api_server::get_prescription(State(state.clone()), headers("b"), Path("unassigned".into())).await.is_err());
    let req = serde_json::from_value(json!({"voidReason":"reason"})).unwrap();
    assert!(crate::api_server::void_prescription(State(state.clone()), headers("n"), Path("unassigned".into()), Json(req)).await.is_err());
    let req = serde_json::from_value(json!({"clientId":"replacement","acknowledgeWarnings":true})).unwrap();
    assert!(crate::api_server::supersede_prescription(State(state.clone()), headers("b"), Path("unassigned".into()), Json(req)).await.is_err());
    let req = serde_json::from_value(json!({"clientId":"replacement","acknowledgeWarnings":true})).unwrap();
    assert!(crate::api_server::supersede_prescription(State(state.clone()), headers("a"), Path("unassigned".into()), Json(req)).await.is_ok());
    let mut conn = pool.get().unwrap();
    let original = db::find_prescription_by_id(&mut conn, "c", "unassigned").unwrap().unwrap();
    assert_eq!(original.status, "superseded");
    assert_eq!(original.doctor_id.as_deref(), Some("da"));
    let replacement = db::find_prescription_by_client_id(&mut conn, "c", "replacement").unwrap().unwrap();
    assert_eq!(replacement.doctor_id.as_deref(), Some("da"));
}

#[tokio::test]
async fn sync_payload_and_stream_do_not_disclose_prescription_identifiers() {
    use axum::extract::{Query, State};
    use tokio_stream::StreamExt;
    let pool = fixture().await;
    let state = state(&pool);
    let b = actor(&pool, "b");
    service::get_prescription(&pool, &b, "draft-b".into()).await.unwrap();
    service::get_prescription(&pool, &b, "draft-b".into()).await.unwrap();
    let response = crate::sync_server::sync_state(State(state.clone()), headers("a"), Query(crate::sync_server::SyncQuery { since: Some(0) })).await.unwrap();
    let text = String::from_utf8(body(response).await).unwrap();
    assert!(!text.contains("draft-b") && !text.contains("private B"));
    let response = crate::sync_server::sync_events(State(state), headers("a"), Query(crate::sync_server::SyncQuery { since: Some(1) })).await.unwrap();
    let mut stream = response.into_body().into_data_stream();
    let frame = tokio::time::timeout(std::time::Duration::from_secs(5), stream.next()).await.unwrap().unwrap().unwrap();
    let text = String::from_utf8(frame.to_vec()).unwrap();
    assert!(text.contains("event: change"));
    assert!(!text.contains("draft-b") && !text.contains("private B"));
}

#[tokio::test]
async fn authorization_is_applied_before_the_history_limit() {
    let pool = fixture().await;
    {
        let mut conn = pool.get().unwrap();
        conn.batch_execute("WITH RECURSIVE seq(n) AS (VALUES(1) UNION ALL SELECT n+1 FROM seq WHERE n<205) INSERT INTO prescriptions(id,patient_id,clinic_id,doctor_id,status,client_id,created_at,updated_at) SELECT 'extra-'||n,'p','c','db','draft','extra-client-'||n,'2027','2027' FROM seq;").unwrap();
    }
    let a = actor(&pool, "a");
    let rows = service::list_prescriptions(&pool, &a, Some("p".into()), Some("draft".into()), None).await.unwrap();
    let ids = rows.iter().map(|row| row.prescription.id.as_str()).collect::<Vec<_>>();
    assert_eq!(ids, vec!["unassigned", "draft-a"]);
}

#[tokio::test]
async fn void_replay_is_not_available_to_other_roles() {
    use axum::extract::{Path, State};
    use axum::Json;
    use serde_json::json;
    let pool = fixture().await;
    let state = state(&pool);
    let req = serde_json::from_value(json!({"voidReason":"reason"})).unwrap();
    assert!(crate::api_server::void_prescription(State(state.clone()), headers("a"), Path("signed-a".into()), Json(req)).await.is_ok());
    for id in ["b", "n", "s", "admin"] {
        let req = serde_json::from_value(json!({"voidReason":"reason"})).unwrap();
        assert!(crate::api_server::void_prescription(State(state.clone()), headers(id), Path("signed-a".into()), Json(req)).await.is_err());
    }
    let mut conn = pool.get().unwrap();
    assert_eq!(db::find_prescription_by_id(&mut conn, "c", "signed-a").unwrap().unwrap().status, "voided");
    assert_eq!(db::find_verification_code_by_prescription(&mut conn, "signed-a").unwrap().unwrap().status, "voided");
}

#[tokio::test]
async fn public_verification_remains_minimal() {
    use axum::extract::{Path, State};
    let pool = fixture().await;
    let code = {
        let mut connection = pool.get().unwrap();
        db::find_verification_code_by_prescription(&mut connection, "signed-a").unwrap().unwrap().code
    };
    let response = crate::verify_server::verify(State(state(&pool)), Path(code)).await.unwrap();
    let text = String::from_utf8(body(response).await).unwrap();
    assert!(!text.contains("issued A"));
    assert!(!text.contains("Patient"));
    assert!(!text.contains("signed-a"));
}

async fn router_fixture(pool: &db::DbPool) -> (String, tokio::task::JoinHandle<()>) {
    let state = state(pool);
    let router = crate::api_server::router(state.clone())
        .merge(crate::pdf_server::router(state.clone()))
        .merge(crate::sync_server::router(state));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    (format!("http://{address}"), task)
}

async fn request(base: &str, who: &str, method: reqwest::Method, path: &str, data: Option<serde_json::Value>) -> (reqwest::StatusCode, Vec<u8>) {
    let client = reqwest::Client::new();
    let mut req = client.request(method, format!("{base}{path}"))
        .header(reqwest::header::COOKIE, format!("persona_session={who}"));
    if let Some(data) = data {
        req = req.json(&data);
    }
    let response = req.send().await.unwrap();
    let status = response.status();
    (status, response.bytes().await.unwrap().to_vec())
}

#[tokio::test]
async fn router_dispatch_enforces_reads_filters_and_identity_changes() {
    use reqwest::{Method, StatusCode};
    let pool = fixture().await;
    let (base, server) = router_fixture(&pool).await;
    for (who, visible, hidden) in [
        ("a", "draft-a", "draft-b"),
        ("b", "draft-b", "draft-a"),
        ("n", "draft-b", "foreign"),
        ("admin", "draft-b", "foreign"),
        ("s", "signed-a", "draft-a"),
        ("x", "foreign", "signed-a"),
    ] {
        let (status, text) = request(&base, who, Method::GET, "/api/prescriptions", None).await;
        assert_eq!(status, StatusCode::OK);
        let text = String::from_utf8(text).unwrap();
        assert!(text.contains(visible) && !text.contains(hidden), "{who}: {text}");
        let (status, _) = request(&base, who, Method::GET, &format!("/api/prescriptions/{visible}"), None).await;
        assert_eq!(status, StatusCode::OK);
        let (status, denied) = request(&base, who, Method::GET, &format!("/api/prescriptions/{hidden}"), None).await;
        let (missing_status, missing) = request(&base, who, Method::GET, "/api/prescriptions/not-present", None).await;
        assert_eq!((status, denied), (missing_status, missing));
        assert_eq!(status, StatusCode::NOT_FOUND);
    }
    for path in [
        "/api/prescriptions?patientId=p&status=draft&date=2026-02",
        "/api/prescriptions?patientId=px",
        "/api/prescriptions?status=voided",
    ] {
        let (status, text) = request(&base, "a", Method::GET, path, None).await;
        assert_eq!(status, StatusCode::OK);
        let text = String::from_utf8(text).unwrap();
        assert!(!text.contains("draft-b") && !text.contains("void-b") && !text.contains("foreign"));
    }
    for suffix in ["", "/preview", "/pdf"] {
        let path = format!("/api/prescriptions/draft-b{suffix}");
        let absent = format!("/api/prescriptions/not-present{suffix}");
        let denied = request(&base, "a", Method::GET, &path, None).await;
        let missing = request(&base, "a", Method::GET, &absent, None).await;
        assert_eq!(denied, missing);
        assert_eq!(denied.0, StatusCode::NOT_FOUND);
        assert!(!denied.1.starts_with(b"%PDF-"));
    }
    for who in ["a", "s"] {
        let (status, pdf) = request(&base, who, Method::GET, "/api/prescriptions/signed-a/pdf", None).await;
        assert_eq!(status, StatusCode::OK);
        assert!(pdf.starts_with(b"%PDF-"));
    }
    let (status, _) = request(&base, "missing", Method::GET, "/api/prescriptions", None).await;
    assert!(!status.is_success());
    let (status, _) = request(&base, "missing", Method::GET, "/api/prescriptions/unassigned", None).await;
    assert!(!status.is_success());
    {
        let mut conn = pool.get().unwrap();
        conn.batch_execute("UPDATE clinic_members SET role='secretary' WHERE profile_id='a';").unwrap();
    }
    let (status, _) = request(&base, "a", Method::GET, "/api/prescriptions/draft-a", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = request(&base, "a", Method::GET, "/api/prescriptions/void-b", None).await;
    assert_eq!(status, StatusCode::OK);
    server.abort();
}

#[tokio::test]
async fn router_dispatch_rejects_mutations_and_replay_without_changes() {
    use reqwest::Method;
    use serde_json::json;
    let pool = fixture().await;
    let (base, server) = router_fixture(&pool).await;
    for (method, path, data) in [
        (Method::PUT, "/api/prescriptions/draft-b", Some(json!({"diagnosis":"forbidden"}))),
        (Method::POST, "/api/prescriptions/draft-b/items", Some(json!({"itemId":"item-b","medName":"forbidden"}))),
        (Method::DELETE, "/api/prescriptions/draft-b/items/item-b", None),
        (Method::POST, "/api/prescriptions/draft-b/items/reorder", Some(json!({"itemIds":["item-b"]}))),
        (Method::POST, "/api/prescriptions/draft-b/sign", Some(json!({"acknowledgeWarnings":true}))),
        (Method::POST, "/api/prescriptions/void-b/void", Some(json!({"voidReason":"forbidden"}))),
        (Method::POST, "/api/prescriptions/void-b/supersede", Some(json!({"clientId":"bad-replacement","acknowledgeWarnings":true}))),
        (Method::POST, "/api/prescriptions", Some(json!({"patientId":"p","templateVersionId":"v","clientId":"client-b","diagnosis":"forbidden"}))),
    ] {
        let (status, response) = request(&base, "a", method, path, data).await;
        assert!(!status.is_success(), "{path}");
        let text = String::from_utf8_lossy(&response);
        assert!(!text.contains("private B") && !text.contains("medicine") && !text.contains("issued B"));
    }
    for who in ["s", "admin", "missing", "x"] {
        for (method, path, data) in [
            (Method::PUT, "/api/prescriptions/draft-a", json!({"diagnosis":"forbidden"})),
            (Method::POST, "/api/prescriptions/draft-a/items", json!({"medName":"forbidden"})),
            (Method::POST, "/api/prescriptions/draft-a/sign", json!({"acknowledgeWarnings":true})),
            (Method::POST, "/api/prescriptions/signed-a/void", json!({"voidReason":"forbidden"})),
            (Method::POST, "/api/prescriptions/signed-a/supersede", json!({"clientId":"bad-replacement","acknowledgeWarnings":true})),
            (Method::POST, "/api/prescriptions", json!({"patientId":"p","templateVersionId":"v","clientId":"client-a"})),
        ] {
            let (status, _) = request(&base, who, method, path, Some(data)).await;
            assert!(!status.is_success(), "{who}: {path}");
        }
    }
    {
        let mut conn = pool.get().unwrap();
        let draft = db::find_prescription_by_id(&mut conn, "c", "draft-b").unwrap().unwrap();
        assert_eq!(draft.diagnosis.as_deref(), Some("private B"));
        assert_eq!(draft.status, "draft");
        assert_eq!(draft.rx_number, None);
        let item = db::find_prescription_item(&mut conn, "draft-b", "item-b").unwrap().unwrap();
        assert_eq!(item.med_name, "medicine");
        assert_eq!(item.position, 1);
        assert_eq!(db::find_prescription_by_id(&mut conn, "c", "void-b").unwrap().unwrap().status, "voided");
        assert!(db::find_prescription_by_client_id(&mut conn, "c", "bad-replacement").unwrap().is_none());
    }
    server.abort();
}

#[tokio::test]
async fn router_dispatch_supports_nursing_draft_handoff_and_owner_replay() {
    use reqwest::{Method, StatusCode};
    use serde_json::json;
    let pool = fixture().await;
    let (base, server) = router_fixture(&pool).await;
    let create = json!({"patientId":"p","templateVersionId":"v","clientId":"nursing-new","diagnosis":"nursing draft"});
    let (status, created) = request(&base, "n", Method::POST, "/api/prescriptions", Some(create.clone())).await;
    assert!(status.is_success());
    let created: serde_json::Value = serde_json::from_slice(&created).unwrap();
    let id = created["id"].as_str().unwrap();
    let path = format!("/api/prescriptions/{id}");
    let (status, _) = request(&base, "n", Method::PUT, &path, Some(json!({"diagnosis":"handoff updated"}))).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = request(&base, "n", Method::POST, &format!("{path}/items"), Some(json!({"itemId":"handoff-item","medName":"medicine"}))).await;
    assert!(status.is_success());
    let (status, _) = request(&base, "n", Method::POST, &format!("{path}/items/reorder"), Some(json!({"itemIds":["handoff-item"]}))).await;
    assert!(status.is_success());
    let (status, _) = request(&base, "n", Method::DELETE, &format!("{path}/items/handoff-item"), None).await;
    assert!(status.is_success());
    let (status, _) = request(&base, "n", Method::POST, &format!("{path}/items"), Some(json!({"itemId":"handoff-item","medName":"medicine"}))).await;
    assert!(status.is_success());
    let (status, _) = request(&base, "n", Method::POST, &format!("{path}/sign"), Some(json!({"acknowledgeWarnings":true}))).await;
    assert!(!status.is_success());
    let (status, _) = request(&base, "a", Method::POST, &format!("{path}/sign"), Some(json!({"acknowledgeWarnings":true}))).await;
    assert!(status.is_success());
    let (status, _) = request(&base, "b", Method::POST, "/api/prescriptions", Some(create.clone())).await;
    assert!(!status.is_success());
    let (status, _) = request(&base, "a", Method::POST, "/api/prescriptions", Some(create)).await;
    assert!(status.is_success());
    let (status, _) = request(&base, "b", Method::GET, &path, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = request(&base, "s", Method::GET, &format!("{path}/pdf"), None).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = request(&base, "n", Method::PUT, &path, Some(json!({"diagnosis":"forbidden"}))).await;
    assert!(!status.is_success());
    let supersede = json!({"clientId":"handoff-replacement","acknowledgeWarnings":true});
    let (status, _) = request(&base, "b", Method::POST, &format!("{path}/supersede"), Some(supersede.clone())).await;
    assert!(!status.is_success());
    let (status, replacement) = request(&base, "a", Method::POST, &format!("{path}/supersede"), Some(supersede.clone())).await;
    assert!(status.is_success());
    let (status, replay) = request(&base, "a", Method::POST, &format!("{path}/supersede"), Some(supersede)).await;
    assert!(status.is_success());
    assert_eq!(serde_json::from_slice::<serde_json::Value>(&replacement).unwrap()["id"], serde_json::from_slice::<serde_json::Value>(&replay).unwrap()["id"]);
    {
        let mut conn = pool.get().unwrap();
        let saved = db::find_prescription_by_id(&mut conn, "c", id).unwrap().unwrap();
        assert_eq!(saved.doctor_id.as_deref(), Some("da"));
        assert_eq!(saved.status, "superseded");
        assert_eq!(saved.diagnosis.as_deref(), Some("handoff updated"));
    }
    server.abort();
}

#[tokio::test]
async fn sensitive_read_audit_records_only_authorized_access_without_clinical_data() {
    let pool = fixture().await;
    let doctor = actor(&pool, "a");
    service::list_prescriptions(&pool, &doctor, None, None, None).await.unwrap();
    service::get_prescription(&pool, &doctor, "draft-a".into()).await.unwrap();
    service::prescription_document(&pool, &doctor, "signed-a".into(), true).await.unwrap();
    assert!(service::get_prescription(&pool, &doctor, "draft-b".into()).await.is_err());
    let mut conn = pool.get().unwrap();
    let rows = db::list_audit(&mut conn, "c", Some("sensitive_read"), Some("prescription"), None, 100).unwrap();
    for id in ["list", "draft-a", "signed-a"] {
        assert!(rows.iter().any(|row| row.entity_id == id && row.actor_id.as_deref() == Some("a")));
    }
    assert!(!rows.iter().any(|row| row.entity_id == "draft-b"));
    for row in rows {
        let metadata = row.metadata_json.unwrap_or_default();
        assert!(!metadata.contains("private") && !metadata.contains("medicine") && !metadata.contains("Patient"));
    }
}
