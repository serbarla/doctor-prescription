use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::auth_server::{AppState, AuthError};
use crate::db;
use crate::service::{
    self, ActorContext, DoctorUpdate, PatientCreateOutcome, PatientInput, PatientUpdate,
    ProfileUpdate, Role,
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

async fn require_actor(state: &AppState, headers: &HeaderMap) -> Result<ActorContext, AuthError> {
    service::resolve_actor(&state.pool, headers)
        .await?
        .ok_or_else(|| AuthError::Unauthorized("Sign in required.".to_owned()))
}

pub async fn me(State(state): State<AppState>, headers: HeaderMap) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    Ok(Json(me_json(&actor)).into_response())
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

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/me", get(me))
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
        .with_state(state)
}
