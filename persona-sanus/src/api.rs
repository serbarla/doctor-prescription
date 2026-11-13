use crate::device;
use crate::http::{delete_json, encode_query, get_json, post_file, post_json, put_json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileSummary {
    pub id: String,
    pub display_name: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClinicSummary {
    pub id: String,
    pub name: String,
    pub timezone: String,
    #[serde(default)]
    pub llm_url_template: Option<String>,
    #[serde(default)]
    pub rx_block_size: Option<i64>,
    #[serde(default)]
    pub retention_days: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctorSummary {
    pub id: String,
    pub specialty: Option<String>,
    pub license_no: Option<String>,
    pub national_id: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

    pub fn can_manage_templates(&self) -> bool {
        matches!(self.role.as_deref(), Some("administrator" | "doctor"))
    }

    pub fn can_create_prescriptions(&self) -> bool {
        matches!(self.role.as_deref(), Some("doctor" | "nursing"))
    }

    pub fn can_sign_prescriptions(&self) -> bool {
        self.role.as_deref() == Some("doctor")
    }

    pub fn can_read_prescriptions(&self) -> bool {
        self.role.is_some()
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
pub struct PrescriptionItem {
    pub id: String,
    pub prescription_id: String,
    pub position: i32,
    pub med_name: String,
    pub dose: Option<String>,
    pub route: Option<String>,
    pub frequency: Option<String>,
    pub duration: Option<String>,
    pub quantity: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationCode {
    pub code: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Prescription {
    pub id: String,
    pub patient_id: String,
    pub patient_name: String,
    pub doctor_id: Option<String>,
    pub clinic_id: String,
    pub template_version_id: Option<String>,
    pub status: String,
    pub rx_number: Option<i32>,
    pub rx_block_id: Option<String>,
    pub diagnosis: Option<String>,
    pub validity: Option<String>,
    pub instructions: Option<String>,
    pub signed_at: Option<String>,
    pub voided_at: Option<String>,
    pub void_reason: Option<String>,
    pub client_id: String,
    pub supersedes_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
    #[serde(default)]
    pub items: Vec<PrescriptionItem>,
    #[serde(default)]
    pub verification: Option<VerificationCode>,
}

#[derive(Debug, Deserialize)]
struct PrescriptionsResponse {
    prescriptions: Vec<Prescription>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RxBlockStatus {
    pub active: bool,
    pub block_id: Option<String>,
    pub start_number: Option<i32>,
    pub end_number: Option<i32>,
    pub consumed: Option<i32>,
    pub remaining: i32,
    pub limit: i32,
    pub exhausted: bool,
    pub device_id: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PrescriptionItemForm {
    pub id: Option<String>,
    pub med_name: String,
    pub dose: String,
    pub route: String,
    pub frequency: String,
    pub duration: String,
    pub quantity: String,
    pub notes: String,
}

pub async fn fetch_prescriptions(
    patient_id: &str,
    status: &str,
    date: &str,
) -> Result<Vec<Prescription>, String> {
    let mut params = Vec::new();
    if !patient_id.trim().is_empty() {
        params.push(format!("patientId={}", encode_query(patient_id)));
    }
    if !status.trim().is_empty() {
        params.push(format!("status={}", encode_query(status)));
    }
    if !date.trim().is_empty() {
        params.push(format!("date={}", encode_query(date)));
    }
    let path = if params.is_empty() {
        "/api/prescriptions".to_owned()
    } else {
        format!("/api/prescriptions?{}", params.join("&"))
    };
    let value = get_json(&path).await?;
    let response: PrescriptionsResponse = serde_json::from_value(value)
        .map_err(|_| "No se pudo leer el historial de recetas.".to_owned())?;
    Ok(response.prescriptions)
}

pub async fn fetch_prescription(id: &str) -> Result<Prescription, String> {
    let value = get_json(&format!("/api/prescriptions/{id}")).await?;
    serde_json::from_value(value).map_err(|_| "No se pudo leer la receta.".to_owned())
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PreviewVariable {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewImageSource {
    pub asset_id: String,
    pub data_url: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrescriptionPreview {
    pub id: String,
    pub status: String,
    pub rx_number: Option<i32>,
    pub verification_code: Option<String>,
    pub issued_at: String,
    pub provisional: bool,
    pub doc_json: String,
    pub margin_mm: u32,
    pub variables: Vec<PreviewVariable>,
    pub medications: Vec<crate::forms::MedicationBlock>,
    pub qr_data_url: Option<String>,
    pub image_sources: Vec<PreviewImageSource>,
}

pub async fn fetch_prescription_preview(id: &str) -> Result<PrescriptionPreview, String> {
    let value = get_json(&format!("/api/prescriptions/{id}/preview")).await?;
    serde_json::from_value(value).map_err(|_| "No se pudo leer la vista previa.".to_owned())
}

pub async fn create_prescription(
    patient_id: &str,
    template_version_id: &str,
    client_id: &str,
    diagnosis: &str,
    validity: &str,
    instructions: &str,
) -> Result<Prescription, String> {
    let value = post_json(
        "/api/prescriptions",
        json!({
            "patientId": patient_id,
            "templateVersionId": template_version_id,
            "clientId": client_id,
            "diagnosis": diagnosis,
            "validity": validity,
            "instructions": instructions,
        }),
    )
    .await?;
    serde_json::from_value(value).map_err(|_| "No se pudo leer la receta.".to_owned())
}

pub async fn update_prescription(
    id: &str,
    diagnosis: &str,
    validity: &str,
    instructions: &str,
) -> Result<Prescription, String> {
    let value = put_json(
        &format!("/api/prescriptions/{id}"),
        json!({
            "diagnosis": diagnosis,
            "validity": validity,
            "instructions": instructions,
        }),
    )
    .await?;
    serde_json::from_value(value).map_err(|_| "No se pudo leer la receta.".to_owned())
}

fn item_body(form: &PrescriptionItemForm) -> Value {
    json!({
        "itemId": form.id,
        "medName": form.med_name,
        "dose": form.dose,
        "route": form.route,
        "frequency": form.frequency,
        "duration": form.duration,
        "quantity": form.quantity,
        "notes": form.notes,
    })
}

pub async fn upsert_prescription_item(
    prescription_id: &str,
    form: &PrescriptionItemForm,
) -> Result<PrescriptionItem, String> {
    let value = post_json(
        &format!("/api/prescriptions/{prescription_id}/items"),
        item_body(form),
    )
    .await?;
    serde_json::from_value(value).map_err(|_| "No se pudo leer el medicamento.".to_owned())
}

pub async fn delete_prescription_item(prescription_id: &str, item_id: &str) -> Result<(), String> {
    delete_json(&format!(
        "/api/prescriptions/{prescription_id}/items/{item_id}"
    ))
    .await
    .map(|_| ())
}

pub async fn reorder_prescription_items(
    prescription_id: &str,
    item_ids: &[String],
) -> Result<Vec<PrescriptionItem>, String> {
    let value = post_json(
        &format!("/api/prescriptions/{prescription_id}/items/reorder"),
        json!({ "itemIds": item_ids }),
    )
    .await?;
    let items = value.get("items").cloned().unwrap_or_else(|| json!([]));
    serde_json::from_value(items)
        .map_err(|_| "No se pudo leer el orden de medicamentos.".to_owned())
}

pub async fn sign_prescription(
    id: &str,
    acknowledge_warnings: bool,
) -> Result<Prescription, String> {
    let value = post_json(
        &format!("/api/prescriptions/{id}/sign"),
        json!({ "acknowledgeWarnings": acknowledge_warnings }),
    )
    .await?;
    serde_json::from_value(value).map_err(|_| "No se pudo leer la receta firmada.".to_owned())
}

#[cfg(feature = "desktop")]
pub async fn reconcile_sign_prescription(
    id: &str,
    rx_number: i64,
    rx_block_id: &str,
) -> Result<Prescription, String> {
    let value = post_json(
        &format!("/api/prescriptions/{id}/sign"),
        json!({ "acknowledgeWarnings": true, "rxNumber": rx_number, "rxBlockId": rx_block_id }),
    )
    .await?;
    serde_json::from_value(value).map_err(|_| "No se pudo leer la receta firmada.".to_owned())
}

pub async fn void_prescription(id: &str, reason: &str) -> Result<Prescription, String> {
    let value = post_json(
        &format!("/api/prescriptions/{id}/void"),
        json!({ "voidReason": reason }),
    )
    .await?;
    serde_json::from_value(value).map_err(|_| "No se pudo leer la receta.".to_owned())
}

pub async fn supersede_prescription(id: &str) -> Result<Prescription, String> {
    let value = post_json(
        &format!("/api/prescriptions/{id}/supersede"),
        json!({ "clientId": uuid_like() }),
    )
    .await?;
    serde_json::from_value(value).map_err(|_| "No se pudo leer la corrección.".to_owned())
}

#[cfg(feature = "desktop")]
pub async fn reconcile_supersede_prescription(
    id: &str,
    client_id: &str,
    new_prescription_id: &str,
    rx_number: i64,
    rx_block_id: &str,
) -> Result<Prescription, String> {
    let value = post_json(
        &format!("/api/prescriptions/{id}/supersede"),
        json!({
            "clientId": client_id,
            "newPrescriptionId": new_prescription_id,
            "acknowledgeWarnings": true,
            "rxNumber": rx_number,
            "rxBlockId": rx_block_id,
        }),
    )
    .await?;
    serde_json::from_value(value).map_err(|_| "No se pudo leer la corrección.".to_owned())
}

fn uuid_like() -> String {
    format!(
        "{}-{}-{}-{}",
        timestamp_part(),
        random_part(),
        random_part(),
        random_part()
    )
}

fn timestamp_part() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    format!("{millis:016x}")
}

fn random_part() -> String {
    use std::cell::Cell;
    thread_local! {
        static SEED: Cell<u64> = const { Cell::new(0x9e3779b97f4a7c15) };
    }
    SEED.with(|seed| {
        let mut value = seed.get();
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        seed.set(value);
        format!("{:08x}", value as u32)
    })
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RxBlockInfo {
    pub id: String,
    pub device_id: String,
    pub start_number: i64,
    pub end_number: i64,
    pub consumed: i64,
    pub status: String,
    pub allocated_at: String,
}

pub async fn allocate_rx_block() -> Result<RxBlockInfo, String> {
    let value = post_json("/api/rx/blocks", json!({})).await?;
    serde_json::from_value(value).map_err(|_| "No se pudo leer el bloque de números.".to_owned())
}

pub async fn fetch_rx_block_status() -> Result<RxBlockStatus, String> {
    let value = get_json("/api/rx/block-status").await?;
    serde_json::from_value(value).map_err(|_| "No se pudo leer el bloque de números.".to_owned())
}
#[cfg(feature = "desktop")]
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncEvent {
    pub cursor: i64,
    pub action: String,
    pub entity: String,
    pub entity_id: String,
    #[serde(default)]
    pub at: String,
}

#[cfg(feature = "desktop")]
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncState {
    pub cursor: i64,
    #[serde(default)]
    pub events: Vec<SyncEvent>,
    #[serde(default)]
    pub rx_block: Option<RxBlockStatus>,
}

#[cfg(feature = "desktop")]
pub async fn fetch_sync_state(since: i64) -> Result<SyncState, String> {
    let value = get_json(&format!("/api/sync/state?since={since}")).await?;
    serde_json::from_value(value)
        .map_err(|_| "No se pudo leer el estado de sincronización.".to_owned())
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

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceRow {
    pub id: String,
    pub profile_id: String,
    pub profile_name: String,
    pub name: Option<String>,
    pub platform: String,
    pub app_version: Option<String>,
    pub last_seen_at: Option<String>,
    pub revoked_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
struct DevicesResponse {
    devices: Vec<DeviceRow>,
}

pub async fn register_current_device() -> Result<(), String> {
    post_json(
        "/api/devices/register",
        json!({
            "deviceId": device::device_id(),
            "platform": device::platform(),
            "name": device::name(),
            "appVersion": env!("CARGO_PKG_VERSION"),
        }),
    )
    .await
    .map(|_| ())
}

pub async fn fetch_devices() -> Result<Vec<DeviceRow>, String> {
    let value = get_json("/api/devices").await?;
    let response: DevicesResponse =
        serde_json::from_value(value).map_err(|_| "Could not read the device list.".to_owned())?;
    Ok(response.devices)
}

pub async fn revoke_device(device_id: &str) -> Result<(), String> {
    post_json("/api/devices/revoke", json!({ "deviceId": device_id }))
        .await
        .map(|_| ())
}

pub async fn fetch_me() -> Result<Me, String> {
    let value = get_json("/api/me").await?;
    serde_json::from_value(value).map_err(|_| "Could not read the profile.".to_owned())
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditRow {
    pub id: String,
    pub actor_id: Option<String>,
    pub actor_name: Option<String>,
    pub action: String,
    pub entity: String,
    pub entity_id: String,
    pub metadata: Value,
    pub at: String,
}

#[derive(Debug, Deserialize)]
struct AuditResponse {
    audit: Vec<AuditRow>,
}

pub async fn fetch_audit(action: &str, entity: &str, q: &str) -> Result<Vec<AuditRow>, String> {
    let mut params = Vec::new();
    if !action.trim().is_empty() {
        params.push(format!("action={}", encode_query(action)));
    }
    if !entity.trim().is_empty() {
        params.push(format!("entity={}", encode_query(entity)));
    }
    if !q.trim().is_empty() {
        params.push(format!("q={}", encode_query(q)));
    }
    let path = if params.is_empty() {
        "/api/audit".to_owned()
    } else {
        format!("/api/audit?{}", params.join("&"))
    };
    let value = get_json(&path).await?;
    let response: AuditResponse = serde_json::from_value(value)
        .map_err(|_| "No se pudo leer el registro de auditoría.".to_owned())?;
    Ok(response.audit)
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

#[cfg(feature = "desktop")]
pub async fn create_patient_with_client_id(
    form: &PatientForm,
    client_id: &str,
) -> Result<PatientCreateOutcome, String> {
    let mut body = patient_body(form);
    body["clientId"] = json!(client_id);
    body["confirmDuplicate"] = json!(true);
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

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateVersion {
    pub id: String,
    pub template_id: String,
    pub version_no: i32,
    pub doc_json: String,
    pub variables_spec: Value,
    pub theme: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Template {
    pub id: String,
    pub owner_type: String,
    pub owner_id: String,
    pub name: String,
    pub status: String,
    pub current_version_id: Option<String>,
    pub current_version: Option<TemplateVersion>,
    #[serde(default)]
    pub versions: Vec<TemplateVersion>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
struct TemplatesResponse {
    templates: Vec<Template>,
}

pub async fn fetch_templates() -> Result<Vec<Template>, String> {
    let value = get_json("/api/templates").await?;
    let response: TemplatesResponse = serde_json::from_value(value)
        .map_err(|_| "No se pudo leer la lista de plantillas.".to_owned())?;
    Ok(response.templates)
}

pub async fn fetch_template(id: &str) -> Result<Template, String> {
    let value = get_json(&format!("/api/templates/{id}")).await?;
    serde_json::from_value(value).map_err(|_| "No se pudo leer la plantilla.".to_owned())
}

pub async fn create_template(name: &str, owner_type: &str) -> Result<Template, String> {
    let value = post_json(
        "/api/templates",
        json!({ "name": name, "ownerType": owner_type }),
    )
    .await?;
    serde_json::from_value(value).map_err(|_| "No se pudo leer la plantilla.".to_owned())
}
#[cfg(feature = "desktop")]
pub async fn create_template_with_client_id(
    name: &str,
    owner_type: &str,
    client_id: &str,
) -> Result<Template, String> {
    let value = post_json(
        "/api/templates",
        json!({ "name": name, "ownerType": owner_type, "clientId": client_id }),
    )
    .await?;
    serde_json::from_value(value).map_err(|_| "No se pudo leer la plantilla.".to_owned())
}

pub async fn create_template_version(
    template_id: &str,
    doc_json: &str,
    theme: &str,
) -> Result<TemplateVersion, String> {
    let value = post_json(
        &format!("/api/templates/{template_id}/versions"),
        json!({ "docJson": doc_json, "theme": theme }),
    )
    .await?;
    serde_json::from_value(value).map_err(|_| "No se pudo leer la versión.".to_owned())
}

pub async fn update_template_version(
    template_id: &str,
    version_id: &str,
    doc_json: &str,
    theme: &str,
) -> Result<TemplateVersion, String> {
    let value = put_json(
        &format!("/api/templates/{template_id}/versions/{version_id}"),
        json!({ "docJson": doc_json, "theme": theme }),
    )
    .await?;
    serde_json::from_value(value).map_err(|_| "No se pudo leer la versión.".to_owned())
}

pub async fn publish_template(template_id: &str, version_id: &str) -> Result<Template, String> {
    let value = post_json(
        &format!("/api/templates/{template_id}/publish"),
        json!({ "versionId": version_id }),
    )
    .await?;
    serde_json::from_value(value).map_err(|_| "No se pudo leer la plantilla.".to_owned())
}

pub async fn archive_template(template_id: &str) -> Result<Template, String> {
    let value = post_json(&format!("/api/templates/{template_id}/archive"), json!({})).await?;
    serde_json::from_value(value).map_err(|_| "No se pudo leer la plantilla.".to_owned())
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetRow {
    pub id: String,
    pub kind: String,
    pub original_name: Option<String>,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub status: String,
    pub url: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetQuota {
    pub limit: i64,
    pub used: i64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetLibrary {
    pub assets: Vec<AssetRow>,
    pub quota: AssetQuota,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClinicSettings {
    pub id: String,
    pub name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub timezone: String,
    #[serde(default)]
    pub llm_url_template: Option<String>,
    #[serde(default)]
    pub server_retention_until: Option<String>,
    #[serde(default)]
    pub data_grace_ends_at: Option<String>,
    pub logo: Option<AssetRow>,
}

pub async fn fetch_assets() -> Result<AssetLibrary, String> {
    let value = get_json("/api/assets").await?;
    serde_json::from_value(value)
        .map_err(|_| "No se pudo leer la biblioteca de imágenes.".to_owned())
}

pub async fn upload_asset(file_name: String, bytes: Vec<u8>) -> Result<AssetRow, String> {
    let value = post_file("/api/assets", file_name, bytes).await?;
    serde_json::from_value(value).map_err(|_| "No se pudo leer la imagen subida.".to_owned())
}

pub async fn upload_logo(file_name: String, bytes: Vec<u8>) -> Result<AssetRow, String> {
    let value = post_file("/api/clinic/logo", file_name, bytes).await?;
    serde_json::from_value(value).map_err(|_| "No se pudo leer el logo subido.".to_owned())
}

pub async fn delete_asset(asset_id: &str) -> Result<(), String> {
    delete_json(&format!("/api/assets/{asset_id}"))
        .await
        .map(|_| ())
}

pub async fn fetch_clinic_settings() -> Result<ClinicSettings, String> {
    let value = get_json("/api/clinic").await?;
    serde_json::from_value(value)
        .map_err(|_| "No se pudo leer la configuración de la clínica.".to_owned())
}

pub async fn update_clinic_settings(
    name: &str,
    address: &str,
    phone: &str,
    llm_url_template: &str,
) -> Result<ClinicSettings, String> {
    let value = put_json(
        "/api/clinic",
        json!({
            "name": name,
            "address": address,
            "phone": phone,
            "llmUrlTemplate": llm_url_template,
        }),
    )
    .await?;
    serde_json::from_value(value)
        .map_err(|_| "No se pudo leer la configuración de la clínica.".to_owned())
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationResult {
    pub status: String,
    pub doctor_name: Option<String>,
    pub issued_at: Option<String>,
    pub rx_number: Option<i32>,
}

pub async fn fetch_verification(code: &str) -> Result<VerificationResult, String> {
    let path = format!("/api/verify/{}", encode_query(code));
    let value = get_json(&path).await?;
    serde_json::from_value(value).map_err(|_| "No se pudo verificar la receta.".to_owned())
}
