use axum::extract::{DefaultBodyLimit, Multipart, Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::auth_server::{self, AppState, AuthError};
use crate::db;
use crate::service::{
    self, ActorContext, AssetKind, ClinicSettings, DeviceRegistration, DoctorUpdate,
    PatientCreateOutcome, PatientInput, PatientUpdate, PrescriptionInput, PrescriptionItemInput,
    PrescriptionSignInput, PrescriptionSupersedeInput, PrescriptionUpdate, ProfileUpdate, Role,
    TemplateVersionInput, UploadFile,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleRequest {
    profile_id: String,
    role: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRequest {
    profile_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisableRequest {
    profile_id: String,
    disabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct PatientSearchRequest {
    #[serde(
        default,
        alias = "query",
        alias = "name",
        alias = "nationalId",
        alias = "national_id",
        alias = "cedula"
    )]
    q: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatientCreateRequest {
    #[serde(alias = "full_name")]
    full_name: String,
    #[serde(default, alias = "national_id")]
    national_id: Option<String>,
    #[serde(default, alias = "birth_date")]
    birth_date: Option<String>,
    #[serde(default)]
    sex: Option<String>,
    #[serde(default)]
    phone: Option<String>,
    #[serde(default)]
    notes: Option<String>,
    #[serde(default, alias = "confirm_duplicate")]
    confirm_duplicate: bool,
    #[serde(default, alias = "client_id")]
    client_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatientUpdateRequest {
    #[serde(default, alias = "full_name")]
    full_name: Option<String>,
    #[serde(default, alias = "national_id")]
    national_id: Option<String>,
    #[serde(default, alias = "birth_date")]
    birth_date: Option<String>,
    #[serde(default)]
    sex: Option<String>,
    #[serde(default)]
    phone: Option<String>,
    #[serde(default)]
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterDeviceRequest {
    device_id: String,
    platform: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    app_version: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevokeDeviceRequest {
    device_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateCreateRequest {
    name: String,
    #[serde(default, alias = "owner_type")]
    owner_type: Option<String>,
    #[serde(default, alias = "client_id")]
    client_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateVersionRequest {
    #[serde(alias = "doc_json")]
    doc_json: String,
    #[serde(default)]
    theme: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishTemplateRequest {
    #[serde(alias = "version_id")]
    version_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClinicUpdateRequest {
    name: String,
    #[serde(default)]
    address: Option<String>,
    #[serde(default)]
    phone: Option<String>,
    #[serde(default, alias = "llm_url_template")]
    llm_url_template: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileUpdateRequest {
    display_name: String,
    #[serde(default)]
    specialty: Option<String>,
    #[serde(default)]
    license_no: Option<String>,
    #[serde(default)]
    national_id: Option<String>,
    #[serde(default)]
    phone: Option<String>,
    #[serde(default)]
    email: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrescriptionCreateRequest {
    #[serde(alias = "patient_id")]
    patient_id: String,
    #[serde(alias = "template_version_id")]
    template_version_id: String,
    #[serde(default, alias = "client_id")]
    client_id: Option<String>,
    #[serde(default)]
    diagnosis: Option<String>,
    #[serde(default)]
    validity: Option<String>,
    #[serde(default)]
    instructions: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrescriptionUpdateRequest {
    #[serde(default)]
    diagnosis: Option<String>,
    #[serde(default)]
    validity: Option<String>,
    #[serde(default)]
    instructions: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrescriptionItemRequest {
    #[serde(default, alias = "item_id")]
    item_id: Option<String>,
    #[serde(alias = "med_name")]
    med_name: String,
    #[serde(default)]
    dose: Option<String>,
    #[serde(default)]
    route: Option<String>,
    #[serde(default)]
    frequency: Option<String>,
    #[serde(default)]
    duration: Option<String>,
    #[serde(default)]
    quantity: Option<String>,
    #[serde(default)]
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrescriptionReorderRequest {
    #[serde(alias = "item_ids")]
    item_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrescriptionSignRequest {
    #[serde(default, alias = "acknowledge_warnings")]
    acknowledge_warnings: bool,
    #[serde(default, alias = "rx_number")]
    rx_number: Option<i32>,
    #[serde(default, alias = "rx_block_id")]
    rx_block_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrescriptionVoidRequest {
    #[serde(alias = "void_reason")]
    void_reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrescriptionSupersedeRequest {
    #[serde(default, alias = "client_id")]
    client_id: Option<String>,
    #[serde(default)]
    diagnosis: Option<String>,
    #[serde(default)]
    validity: Option<String>,
    #[serde(default)]
    instructions: Option<String>,
    #[serde(default, alias = "acknowledge_warnings")]
    acknowledge_warnings: bool,
    #[serde(default, alias = "rx_number")]
    rx_number: Option<i32>,
    #[serde(default, alias = "rx_block_id")]
    rx_block_id: Option<String>,
    #[serde(default, alias = "new_prescription_id")]
    new_prescription_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PrescriptionListQuery {
    #[serde(default, alias = "patientId")]
    patient_id: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    date: Option<String>,
}

fn clean(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
}

fn clinic_json(actor: &ActorContext) -> Value {
    match actor.clinic.as_ref() {
        Some(clinic) => json!({
            "id": clinic.id,
            "name": clinic.name,
            "timezone": clinic.timezone,
            "llmUrlTemplate": clinic.llm_url_template,
            "rxBlockSize": clinic.rx_block_size,
            "retentionDays": clinic.retention_days,
        }),
        None => Value::Null,
    }
}

fn doctor_json(actor: &ActorContext) -> Value {
    match actor.doctor.as_ref() {
        Some(doctor) => json!({
            "id": doctor.id,
            "specialty": doctor.specialty,
            "licenseNo": doctor.license_no,
            "nationalId": doctor.national_id,
            "phone": doctor.phone,
            "email": doctor.email,
        }),
        None => Value::Null,
    }
}

fn patient_json(patient: &db::Patient) -> Value {
    json!({
        "id": patient.id,
        "fullName": patient.full_name,
        "nationalId": patient.national_id,
        "birthDate": patient.birth_date,
        "sex": patient.sex,
        "phone": patient.phone,
        "notes": patient.notes,
        "createdAt": patient.created_at,
        "updatedAt": patient.updated_at,
    })
}

fn prescription_json(prescription: &db::Prescription, patient_name: &str) -> Value {
    json!({
        "id": prescription.id,
        "patientId": prescription.patient_id,
        "patientName": patient_name,
        "doctorId": prescription.doctor_id,
        "clinicId": prescription.clinic_id,
        "templateVersionId": prescription.template_version_id,
        "status": prescription.status,
        "rxNumber": prescription.rx_number,
        "rxBlockId": prescription.rx_block_id,
        "diagnosis": prescription.diagnosis,
        "validity": prescription.validity,
        "instructions": prescription.instructions,
        "signedAt": prescription.signed_at,
        "voidedAt": prescription.voided_at,
        "voidReason": prescription.void_reason,
        "clientId": prescription.client_id,
        "supersedesId": prescription.supersedes_id,
        "createdAt": prescription.created_at,
        "updatedAt": prescription.updated_at,
        "createdBy": prescription.created_by,
    })
}

fn prescription_item_json(item: &db::PrescriptionItem) -> Value {
    json!({
        "id": item.id,
        "prescriptionId": item.prescription_id,
        "position": item.position,
        "medName": item.med_name,
        "dose": item.dose,
        "route": item.route,
        "frequency": item.frequency,
        "duration": item.duration,
        "quantity": item.quantity,
        "notes": item.notes,
    })
}

fn verification_json(code: &db::VerificationCode) -> Value {
    json!({
        "code": code.code,
        "status": code.status,
        "createdAt": code.created_at,
        "updatedAt": code.updated_at,
    })
}

fn prescription_detail_json(detail: &service::PrescriptionDetail) -> Value {
    let mut value = prescription_json(&detail.prescription, &detail.patient_name);
    if let Some(object) = value.as_object_mut() {
        object.insert(
            "items".to_owned(),
            Value::Array(detail.items.iter().map(prescription_item_json).collect()),
        );
        object.insert(
            "verification".to_owned(),
            detail
                .verification
                .as_ref()
                .map(verification_json)
                .unwrap_or(Value::Null),
        );
    }
    value
}

fn rx_block_json(block: &db::RxBlock) -> Value {
    json!({
        "id": block.id,
        "deviceId": block.device_id,
        "startNumber": block.start_number,
        "endNumber": block.end_number,
        "consumed": block.consumed,
        "remaining": (block.end_number - block.start_number + 1 - block.consumed).max(0),
        "status": block.status,
        "allocatedAt": block.allocated_at,
    })
}

fn rx_status_json(status: &service::RxBlockStatus) -> Value {
    json!({
        "active": status.active,
        "blockId": status.block_id,
        "startNumber": status.start_number,
        "endNumber": status.end_number,
        "consumed": status.consumed,
        "remaining": status.remaining,
        "limit": status.limit,
        "exhausted": status.exhausted,
        "deviceId": status.device_id,
    })
}

fn device_json(device: &db::Device, profile_name: &str) -> Value {
    json!({
        "id": device.id,
        "profileId": device.profile_id,
        "profileName": profile_name,
        "name": device.name,
        "platform": device.platform,
        "appVersion": device.app_version,
        "lastSeenAt": device.last_seen_at,
        "revokedAt": device.revoked_at,
        "createdAt": device.created_at,
        "updatedAt": device.updated_at,
    })
}

fn template_version_json(version: &db::TemplateVersion) -> Value {
    json!({
        "id": version.id,
        "templateId": version.template_id,
        "versionNo": version.version_no,
        "docJson": version.doc_json,
        "variablesSpec": serde_json::from_str::<Value>(&version.variables_spec)
            .unwrap_or_else(|_| json!([])),
        "theme": version.theme,
        "status": version.status,
        "createdAt": version.created_at,
        "updatedAt": version.updated_at,
    })
}

fn template_json(template: &db::Template, versions: &[db::TemplateVersion]) -> Value {
    let current = template
        .current_version_id
        .as_ref()
        .and_then(|id| versions.iter().find(|version| &version.id == id))
        .map(template_version_json);
    json!({
        "id": template.id,
        "ownerType": template.owner_type,
        "ownerId": template.owner_id,
        "name": template.name,
        "status": template.status,
        "currentVersionId": template.current_version_id,
        "currentVersion": current,
        "versions": versions.iter().map(template_version_json).collect::<Vec<Value>>(),
        "createdAt": template.created_at,
        "updatedAt": template.updated_at,
    })
}

fn asset_json(asset: &db::Asset, url: Option<String>) -> Value {
    json!({
        "id": asset.id,
        "clinicId": asset.clinic_id,
        "kind": asset.kind,
        "originalName": asset.original_name,
        "contentType": asset.content_type,
        "sizeBytes": asset.size_bytes,
        "sha256": asset.sha256,
        "width": asset.width,
        "height": asset.height,
        "status": asset.status,
        "url": url,
        "createdAt": asset.created_at,
        "updatedAt": asset.updated_at,
        "createdBy": asset.created_by,
    })
}

fn clinic_settings_json(settings: &ClinicSettings, state: &AppState) -> Value {
    let logo = settings.logo.as_ref().map(|asset| {
        let url = state
            .assets
            .as_ref()
            .and_then(|storage| storage.presigned_get(&asset.object_key).ok());
        asset_json(asset, url)
    });
    json!({
        "id": settings.clinic.id,
        "name": settings.clinic.name,
        "address": settings.clinic.address,
        "phone": settings.clinic.phone,
        "timezone": settings.clinic.timezone,
        "llmUrlTemplate": settings.clinic.llm_url_template,
        "serverRetentionUntil": settings.clinic.server_retention_until,
        "dataGraceEndsAt": settings.clinic.data_grace_ends_at,
        "logo": logo,
    })
}

fn me_json(actor: &ActorContext) -> Value {
    json!({
        "profile": {
            "id": actor.profile.id,
            "displayName": actor.profile.display_name,
            "status": actor.profile.status,
        },
        "clinic": clinic_json(actor),
        "role": actor.role().map(Role::as_str),
        "doctor": doctor_json(actor),
    })
}

pub(crate) async fn require_actor(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<ActorContext, AuthError> {
    service::resolve_actor(&state.pool, headers)
        .await?
        .ok_or_else(|| AuthError::Unauthorized("Sign in required.".to_owned()))
}

pub async fn me(State(state): State<AppState>, headers: HeaderMap) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    Ok(Json(me_json(&actor)).into_response())
}

#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    #[serde(default)]
    action: Option<String>,
    #[serde(default)]
    entity: Option<String>,
    #[serde(default)]
    q: Option<String>,
    #[serde(default)]
    limit: Option<i64>,
}

pub async fn list_audit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AuditQuery>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    actor.require_administrator()?;
    let clinic_id = actor.clinic_id()?.to_owned();
    let action = query.action.filter(|value| !value.trim().is_empty());
    let entity = query.entity.filter(|value| !value.trim().is_empty());
    let q = query.q.filter(|value| !value.trim().is_empty());
    let limit = query.limit.unwrap_or(100).clamp(1, 500);
    let rows = auth_server::with_conn(&state.pool, move |connection| {
        Ok(db::list_audit(
            connection,
            &clinic_id,
            action.as_deref(),
            entity.as_deref(),
            q.as_deref(),
            limit,
        )?)
    })
    .await?;
    let audit = rows
        .iter()
        .map(|row| {
            json!({
                "id": row.id,
                "actorId": row.actor_id,
                "actorName": row.actor_name,
                "action": row.action,
                "entity": row.entity,
                "entityId": row.entity_id,
                "metadata": serde_json::from_str::<Value>(row.metadata_json.as_deref().unwrap_or("null"))
                    .unwrap_or(Value::Null),
                "at": row.at,
            })
        })
        .collect::<Vec<_>>();
    Ok(Json(json!({ "audit": audit })).into_response())
}

pub async fn register_device(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<RegisterDeviceRequest>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let device = service::register_device(
        &state.pool,
        &actor,
        DeviceRegistration {
            device_id: request.device_id,
            platform: request.platform,
            name: request.name,
            app_version: request.app_version,
        },
    )
    .await?;
    Ok(Json(device_json(&device, &actor.profile.display_name)).into_response())
}

pub async fn list_devices(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let list = service::list_devices(&state.pool, &actor).await?;
    let devices: Vec<Value> = list
        .devices
        .iter()
        .map(|row| device_json(&row.device, &row.profile_name))
        .collect();
    Ok(Json(json!({
        "devices": devices,
        "currentDeviceId": list.current_device_id,
    }))
    .into_response())
}

pub async fn revoke_device(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<RevokeDeviceRequest>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    service::revoke_device(&state.pool, &actor, request.device_id).await?;
    Ok(Json(json!({ "ok": true })).into_response())
}

pub async fn list_users(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let rows = service::list_users(&state.pool, &actor).await?;
    let users: Vec<Value> = rows
        .into_iter()
        .map(|row| {
            json!({
                "profileId": row.profile.id,
                "displayName": row.profile.display_name,
                "email": row.email,
                "role": row.role,
                "status": row.profile.status,
                "createdAt": row.profile.created_at,
            })
        })
        .collect();
    Ok(Json(json!({ "users": users })).into_response())
}

pub async fn set_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<RoleRequest>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    service::set_member_role(&state.pool, &actor, request.profile_id, request.role).await?;
    Ok(Json(json!({ "ok": true })).into_response())
}

pub async fn remove_member(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ProfileRequest>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    service::remove_member(&state.pool, &actor, request.profile_id).await?;
    Ok(Json(json!({ "ok": true })).into_response())
}

pub async fn disable_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<DisableRequest>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    service::disable_profile(&state.pool, &actor, request.profile_id, request.disabled).await?;
    Ok(Json(json!({ "ok": true })).into_response())
}

pub async fn get_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    Ok(Json(me_json(&actor)).into_response())
}

pub async fn update_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ProfileUpdateRequest>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    service::update_profile(
        &state.pool,
        &actor,
        ProfileUpdate {
            display_name: request.display_name,
        },
    )
    .await?;
    let doctor_fields = (
        clean(request.specialty),
        clean(request.license_no),
        clean(request.national_id),
        clean(request.phone),
        clean(request.email),
    );
    if actor.role() == Some(Role::Doctor)
        && doctor_fields
            .0
            .as_ref()
            .or(doctor_fields.1.as_ref())
            .or(doctor_fields.2.as_ref())
            .or(doctor_fields.3.as_ref())
            .or(doctor_fields.4.as_ref())
            .is_some()
    {
        service::update_doctor(
            &state.pool,
            &actor,
            DoctorUpdate {
                specialty: doctor_fields.0,
                license_no: doctor_fields.1,
                national_id: doctor_fields.2,
                phone: doctor_fields.3,
                email: doctor_fields.4,
            },
        )
        .await?;
    }
    let actor = require_actor(&state, &headers).await?;
    Ok(Json(me_json(&actor)).into_response())
}

pub async fn list_prescriptions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PrescriptionListQuery>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let rows = service::list_prescriptions(
        &state.pool,
        &actor,
        query.patient_id,
        query.status,
        query.date,
    )
    .await?;
    let prescriptions: Vec<Value> = rows
        .iter()
        .map(|row| prescription_json(&row.prescription, &row.patient_name))
        .collect();
    Ok(Json(json!({ "prescriptions": prescriptions })).into_response())
}

pub async fn create_prescription(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<PrescriptionCreateRequest>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let prescription = service::create_prescription(
        &state.pool,
        &actor,
        PrescriptionInput {
            patient_id: request.patient_id,
            template_version_id: request.template_version_id,
            client_id: request.client_id,
            diagnosis: request.diagnosis,
            validity: request.validity,
            instructions: request.instructions,
        },
    )
    .await?;
    let detail = service::get_prescription(&state.pool, &actor, prescription.id).await?;
    Ok(Json(prescription_detail_json(&detail)).into_response())
}

pub async fn get_prescription(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let detail = service::get_prescription(&state.pool, &actor, id).await?;
    Ok(Json(prescription_detail_json(&detail)).into_response())
}

pub async fn update_prescription(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<PrescriptionUpdateRequest>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let prescription = service::update_prescription(
        &state.pool,
        &actor,
        id,
        PrescriptionUpdate {
            diagnosis: request.diagnosis,
            validity: request.validity,
            instructions: request.instructions,
        },
    )
    .await?;
    let detail = service::get_prescription(&state.pool, &actor, prescription.id).await?;
    Ok(Json(prescription_detail_json(&detail)).into_response())
}

pub async fn upsert_prescription_item(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<PrescriptionItemRequest>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let item = service::upsert_prescription_item(
        &state.pool,
        &actor,
        id,
        PrescriptionItemInput {
            item_id: request.item_id,
            med_name: request.med_name,
            dose: request.dose,
            route: request.route,
            frequency: request.frequency,
            duration: request.duration,
            quantity: request.quantity,
            notes: request.notes,
        },
    )
    .await?;
    Ok(Json(prescription_item_json(&item)).into_response())
}

pub async fn delete_prescription_item(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((prescription_id, item_id)): Path<(String, String)>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    service::delete_prescription_item(&state.pool, &actor, prescription_id, item_id).await?;
    Ok(Json(json!({ "ok": true })).into_response())
}

pub async fn reorder_prescription_items(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<PrescriptionReorderRequest>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let items =
        service::reorder_prescription_items(&state.pool, &actor, id, request.item_ids).await?;
    Ok(Json(json!({
        "items": items.iter().map(prescription_item_json).collect::<Vec<Value>>(),
    }))
    .into_response())
}

pub async fn sign_prescription(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<PrescriptionSignRequest>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let detail = service::sign_prescription(
        &state.pool,
        &actor,
        id,
        PrescriptionSignInput {
            acknowledge_warnings: request.acknowledge_warnings,
            rx_number: request.rx_number,
            rx_block_id: request.rx_block_id,
        },
    )
    .await?;
    Ok(Json(prescription_detail_json(&detail)).into_response())
}

pub async fn void_prescription(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<PrescriptionVoidRequest>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let detail = service::void_prescription(&state.pool, &actor, id, request.void_reason).await?;
    Ok(Json(prescription_detail_json(&detail)).into_response())
}

pub async fn supersede_prescription(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<PrescriptionSupersedeRequest>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let detail = service::supersede_prescription(
        &state.pool,
        &actor,
        id,
        PrescriptionSupersedeInput {
            client_id: request.client_id,
            diagnosis: request.diagnosis,
            validity: request.validity,
            instructions: request.instructions,
            acknowledge_warnings: request.acknowledge_warnings,
            rx_number: request.rx_number,
            rx_block_id: request.rx_block_id,
            new_prescription_id: request.new_prescription_id,
        },
    )
    .await?;
    Ok(Json(prescription_detail_json(&detail)).into_response())
}

pub async fn allocate_rx_block(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let block = service::allocate_rx_block(&state.pool, &actor).await?;
    Ok(Json(rx_block_json(&block)).into_response())
}

pub async fn rx_block_status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let status = service::rx_block_status(&state.pool, &actor).await?;
    Ok(Json(rx_status_json(&status)).into_response())
}

pub async fn list_patients(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(request): Query<PatientSearchRequest>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let patients = service::search_patients(&state.pool, &actor, request.q).await?;
    let rows: Vec<Value> = patients.iter().map(patient_json).collect();
    Ok(Json(json!({ "patients": rows })).into_response())
}

pub async fn create_patient(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<PatientCreateRequest>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let outcome = service::create_patient(
        &state.pool,
        &actor,
        PatientInput {
            full_name: request.full_name,
            national_id: request.national_id,
            birth_date: request.birth_date,
            sex: request.sex,
            phone: request.phone,
            notes: request.notes,
            client_id: request.client_id,
        },
        request.confirm_duplicate,
    )
    .await?;
    match outcome {
        PatientCreateOutcome::Created(patient) => Ok(Json(patient_json(&patient)).into_response()),
        PatientCreateOutcome::LikelyDuplicate(patient_ids) => Ok(Json(json!({
            "warning": "likely_duplicate",
            "patientIds": patient_ids,
        }))
        .into_response()),
    }
}

pub async fn get_patient(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let patient = service::get_patient(&state.pool, &actor, id).await?;
    Ok(Json(patient_json(&patient)).into_response())
}

pub async fn update_patient(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<PatientUpdateRequest>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let patient = service::update_patient(
        &state.pool,
        &actor,
        id,
        PatientUpdate {
            full_name: request.full_name,
            national_id: request.national_id,
            birth_date: request.birth_date,
            sex: request.sex,
            phone: request.phone,
            notes: request.notes,
        },
    )
    .await?;
    Ok(Json(patient_json(&patient)).into_response())
}

pub async fn delete_patient(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    service::soft_delete_patient(&state.pool, &actor, id).await?;
    Ok(Json(json!({ "ok": true })).into_response())
}

pub async fn list_templates(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let rows = service::list_templates(&state.pool, &actor).await?;
    let templates: Vec<Value> = rows
        .into_iter()
        .map(|(template, version)| {
            let versions: Vec<db::TemplateVersion> = version.into_iter().collect();
            template_json(&template, &versions)
        })
        .collect();
    Ok(Json(json!({ "templates": templates })).into_response())
}

pub async fn create_template(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<TemplateCreateRequest>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let template = service::create_template(
        &state.pool,
        &actor,
        request.name,
        request.owner_type,
        request.client_id,
    )
    .await?;
    Ok(Json(template_json(&template, &[])).into_response())
}

pub async fn get_template(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let (template, versions) = service::get_template(&state.pool, &actor, id).await?;
    Ok(Json(template_json(&template, &versions)).into_response())
}

pub async fn create_template_version(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<TemplateVersionRequest>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let version = service::create_template_version(
        &state.pool,
        &actor,
        id,
        TemplateVersionInput {
            doc_json: request.doc_json,
            theme: request.theme,
        },
    )
    .await?;
    Ok(Json(template_version_json(&version)).into_response())
}

pub async fn update_template_version(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((template_id, version_id)): Path<(String, String)>,
    Json(request): Json<TemplateVersionRequest>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let version = service::update_template_version(
        &state.pool,
        &actor,
        template_id,
        version_id,
        TemplateVersionInput {
            doc_json: request.doc_json,
            theme: request.theme,
        },
    )
    .await?;
    Ok(Json(template_version_json(&version)).into_response())
}

pub async fn publish_template_version(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<PublishTemplateRequest>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let template =
        service::publish_template_version(&state.pool, &actor, id, request.version_id).await?;
    let (template, versions) = service::get_template(&state.pool, &actor, template.id).await?;
    Ok(Json(template_json(&template, &versions)).into_response())
}

pub async fn archive_template(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let template = service::archive_template(&state.pool, &actor, id).await?;
    let (template, versions) = service::get_template(&state.pool, &actor, template.id).await?;
    Ok(Json(template_json(&template, &versions)).into_response())
}

async fn object_storage(state: &AppState) -> Result<&crate::assets::AssetStorage, AuthError> {
    let storage = state.assets.as_deref().ok_or_else(|| {
        AuthError::StorageUnavailable(
            "El almacenamiento de imágenes no está configurado.".to_owned(),
        )
    })?;
    storage.ensure_ready().await?;
    Ok(storage)
}

async fn multipart_file(mut multipart: Multipart) -> Result<UploadFile, AuthError> {
    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|_| AuthError::BadRequest("No se pudo leer el archivo.".to_owned()))?
    {
        if field.file_name().is_none() && field.name() != Some("file") {
            continue;
        }
        let file_name = field.file_name().map(str::to_owned);
        let mut bytes = Vec::new();
        while let Some(chunk) = field
            .chunk()
            .await
            .map_err(|_| AuthError::BadRequest("No se pudo leer el archivo.".to_owned()))?
        {
            if bytes.len() + chunk.len() > crate::assets::MAX_IMAGE_BYTES {
                return Err(AuthError::TooLarge(
                    "La imagen supera el límite de 10 MB.".to_owned(),
                ));
            }
            bytes.extend_from_slice(&chunk);
        }
        return Ok(UploadFile { file_name, bytes });
    }
    Err(AuthError::BadRequest(
        "Adjunta una imagen JPEG o PNG.".to_owned(),
    ))
}

pub async fn list_assets(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let storage = object_storage(&state).await?;
    let assets = service::list_assets(&state.pool, &actor).await?;
    let used = assets
        .iter()
        .filter(|asset| asset.kind == "image" && asset.deleted_at.is_none())
        .count();
    let payload: Vec<Value> = assets
        .iter()
        .map(|asset| asset_json(asset, storage.presigned_get(&asset.object_key).ok()))
        .collect();
    Ok(Json(json!({
        "assets": payload,
        "quota": { "limit": 1, "used": used },
        "storage": { "ready": true },
    }))
    .into_response())
}

pub async fn upload_asset(
    State(state): State<AppState>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let storage = object_storage(&state).await?;
    let file = multipart_file(multipart).await?;
    let asset = service::upload_asset(&state.pool, storage, &actor, AssetKind::Image, file).await?;
    let url = storage.presigned_get(&asset.object_key).ok();
    Ok((StatusCode::CREATED, Json(asset_json(&asset, url))).into_response())
}

pub async fn upload_logo(
    State(state): State<AppState>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let storage = object_storage(&state).await?;
    let file = multipart_file(multipart).await?;
    let asset = service::upload_asset(&state.pool, storage, &actor, AssetKind::Logo, file).await?;
    let url = storage.presigned_get(&asset.object_key).ok();
    Ok((StatusCode::CREATED, Json(asset_json(&asset, url))).into_response())
}

pub async fn delete_asset(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let storage = object_storage(&state).await?;
    service::delete_asset(&state.pool, storage, &actor, id).await?;
    Ok(Json(json!({ "ok": true })).into_response())
}

pub async fn asset_url(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let storage = object_storage(&state).await?;
    let asset = service::asset_for_read(&state.pool, &actor, id).await?;
    let url = storage.presigned_get(&asset.object_key)?;
    Ok(Json(json!({
        "url": url,
        "expiresIn": storage.ttl_seconds(),
        "asset": asset_json(&asset, None),
    }))
    .into_response())
}

pub async fn get_clinic(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let settings = service::clinic_settings(&state.pool, &actor).await?;
    Ok(Json(clinic_settings_json(&settings, &state)).into_response())
}

pub async fn export_clinic(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let data = service::export_clinic_data(&state.pool, &actor).await?;
    Ok(Json(data).into_response())
}

pub async fn update_clinic(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ClinicUpdateRequest>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let settings = service::update_clinic_settings(
        &state.pool,
        &actor,
        request.name,
        request.address,
        request.phone,
        request.llm_url_template,
    )
    .await?;
    Ok(Json(clinic_settings_json(&settings, &state)).into_response())
}

pub fn router(state: AppState) -> Router {
    let asset_router = Router::new()
        .route("/api/assets", get(list_assets).post(upload_asset))
        .route("/api/assets/{id}", delete(delete_asset))
        .route("/api/assets/{id}/url", get(asset_url))
        .route("/api/clinic", get(get_clinic).put(update_clinic))
        .route("/api/clinic/export", get(export_clinic))
        .route("/api/clinic/logo", post(upload_logo))
        .layer(DefaultBodyLimit::max(crate::assets::body_limit()));
    Router::new()
        .route("/api/me", get(me))
        .route("/api/audit", get(list_audit))
        .route("/api/devices", get(list_devices))
        .route("/api/devices/register", post(register_device))
        .route("/api/devices/revoke", post(revoke_device))
        .route("/api/settings/users", get(list_users))
        .route("/api/settings/users/role", post(set_role))
        .route("/api/settings/users/remove", post(remove_member))
        .route("/api/settings/users/disable", post(disable_profile))
        .route(
            "/api/settings/profile",
            get(get_profile).post(update_profile),
        )
        .route("/api/patients", get(list_patients).post(create_patient))
        .route(
            "/api/patients/{id}",
            get(get_patient).put(update_patient).delete(delete_patient),
        )
        .route("/api/templates", get(list_templates).post(create_template))
        .route("/api/templates/{id}", get(get_template))
        .route(
            "/api/templates/{id}/versions",
            post(create_template_version),
        )
        .route(
            "/api/templates/{id}/versions/{version_id}",
            put(update_template_version),
        )
        .route(
            "/api/templates/{id}/publish",
            post(publish_template_version),
        )
        .route("/api/templates/{id}/archive", post(archive_template))
        .route(
            "/api/prescriptions",
            get(list_prescriptions).post(create_prescription),
        )
        .route(
            "/api/prescriptions/{id}",
            get(get_prescription).put(update_prescription),
        )
        .route(
            "/api/prescriptions/{id}/items",
            post(upsert_prescription_item),
        )
        .route(
            "/api/prescriptions/{id}/items/reorder",
            post(reorder_prescription_items),
        )
        .route(
            "/api/prescriptions/{id}/items/{item_id}",
            delete(delete_prescription_item),
        )
        .route("/api/prescriptions/{id}/sign", post(sign_prescription))
        .route("/api/prescriptions/{id}/void", post(void_prescription))
        .route(
            "/api/prescriptions/{id}/supersede",
            post(supersede_prescription),
        )
        .route("/api/rx/blocks", post(allocate_rx_block))
        .route("/api/rx/block-status", get(rx_block_status))
        .merge(asset_router)
        .with_state(state)
}
