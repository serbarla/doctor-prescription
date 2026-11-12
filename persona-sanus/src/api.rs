use serde::Deserialize;
use serde_json::{json, Value};

use crate::http::{delete_json, encode_query, get_json, post_json, put_json};

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileSummary {
    pub id: String,
    pub display_name: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClinicSummary {
    pub id: String,
    pub name: String,
    pub timezone: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctorSummary {
    pub id: String,
    pub specialty: Option<String>,
    pub license_no: Option<String>,
    pub national_id: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Me {
    pub profile: ProfileSummary,
    pub clinic: Option<ClinicSummary>,
    pub role: Option<String>,
    pub doctor: Option<DoctorSummary>,
}

impl Me {
    pub fn is_administrator(&self) -> bool {
        self.role.as_deref() == Some("administrator")
    }

    pub fn is_doctor(&self) -> bool {
        self.role.as_deref() == Some("doctor")
    }

    pub fn can_read_patients(&self) -> bool {
        self.role.is_some()
    }

    pub fn can_write_patients(&self) -> bool {
        matches!(
            self.role.as_deref(),
            Some("doctor" | "nursing" | "secretary")
        )
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Patient {
    pub id: String,
    pub full_name: String,
    pub national_id: Option<String>,
    pub birth_date: Option<String>,
    pub sex: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PatientForm {
    pub full_name: String,
    pub national_id: String,
    pub birth_date: String,
    pub sex: String,
    pub phone: String,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatientCreateOutcome {
    Created(Patient),
    LikelyDuplicate(Vec<String>),
}

#[derive(Debug, Deserialize)]
struct PatientsResponse {
    patients: Vec<Patient>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRow {
    pub profile_id: String,
    pub display_name: String,
    pub email: String,
    pub role: Option<String>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
struct UsersResponse {
    users: Vec<UserRow>,
}

pub async fn fetch_me() -> Result<Me, String> {
    let value = get_json("/api/me").await?;
    serde_json::from_value(value).map_err(|_| "Could not read the profile.".to_owned())
}

pub async fn fetch_users() -> Result<Vec<UserRow>, String> {
    let value = get_json("/api/settings/users").await?;
    let response: UsersResponse =
        serde_json::from_value(value).map_err(|_| "Could not read the user list.".to_owned())?;
    Ok(response.users)
}

pub async fn set_role(profile_id: &str, role: &str) -> Result<(), String> {
    post_json(
        "/api/settings/users/role",
        json!({ "profileId": profile_id, "role": role }),
    )
    .await
    .map(|_| ())
}

pub async fn remove_member(profile_id: &str) -> Result<(), String> {
    post_json(
        "/api/settings/users/remove",
        json!({ "profileId": profile_id }),
    )
    .await
    .map(|_| ())
}

pub async fn disable_profile(profile_id: &str, disabled: bool) -> Result<(), String> {
    post_json(
        "/api/settings/users/disable",
        json!({ "profileId": profile_id, "disabled": disabled }),
    )
    .await
    .map(|_| ())
}

pub async fn update_profile(
    display_name: &str,
    specialty: &str,
    license_no: &str,
    national_id: &str,
    phone: &str,
    email: &str,
) -> Result<Me, String> {
    let value = post_json(
        "/api/settings/profile",
        json!({
            "displayName": display_name,
            "specialty": specialty,
            "licenseNo": license_no,
            "nationalId": national_id,
            "phone": phone,
            "email": email
        }),
    )
    .await?;
    serde_json::from_value(value).map_err(|_| "Could not read the profile.".to_owned())
}

fn patient_body(form: &PatientForm) -> Value {
    json!({
        "fullName": form.full_name,
        "nationalId": form.national_id,
        "birthDate": form.birth_date,
        "sex": form.sex,
        "phone": form.phone,
        "notes": form.notes,
    })
}

pub async fn fetch_patients(query: &str) -> Result<Vec<Patient>, String> {
    let query = query.trim();
    let path = if query.is_empty() {
        "/api/patients".to_owned()
    } else {
        format!("/api/patients?q={}", encode_query(query))
    };
    let value = get_json(&path).await?;
    let response: PatientsResponse =
        serde_json::from_value(value).map_err(|_| "Could not read the patient list.".to_owned())?;
    Ok(response.patients)
}

pub async fn fetch_patient(id: &str) -> Result<Patient, String> {
    let value = get_json(&format!("/api/patients/{id}")).await?;
    serde_json::from_value(value).map_err(|_| "Could not read the patient.".to_owned())
}

pub async fn create_patient(
    form: &PatientForm,
    confirm_duplicate: bool,
) -> Result<PatientCreateOutcome, String> {
    let mut body = patient_body(form);
    if confirm_duplicate {
        body["confirmDuplicate"] = json!(true);
    }
    let value = post_json("/api/patients", body).await?;
    if value.get("warning").and_then(|warning| warning.as_str()) == Some("likely_duplicate") {
        let ids = value
            .get("patientIds")
            .and_then(|ids| ids.as_array())
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| id.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default();
        return Ok(PatientCreateOutcome::LikelyDuplicate(ids));
    }
    serde_json::from_value(value)
        .map(PatientCreateOutcome::Created)
        .map_err(|_| "Could not read the patient.".to_owned())
}

pub async fn update_patient(id: &str, form: &PatientForm) -> Result<Patient, String> {
    let value = put_json(&format!("/api/patients/{id}"), patient_body(form)).await?;
    serde_json::from_value(value).map_err(|_| "Could not read the patient.".to_owned())
}

pub async fn delete_patient(id: &str) -> Result<(), String> {
    delete_json(&format!("/api/patients/{id}"))
        .await
        .map(|_| ())
}
