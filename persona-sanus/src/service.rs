use axum::http::HeaderMap;
use chrono::{
    Datelike, Duration as ChronoDuration, FixedOffset, Months, NaiveDate, SecondsFormat, Utc,
};
use diesel::prelude::*;
use serde_json::{json, Value};
use unicode_casefold::UnicodeCaseFold;
use unicode_normalization::char::is_combining_mark;
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;

use crate::auth_server::{self, AuthError, SESSION_COOKIE};
use crate::db::{
    self, DbPool, NewAuditLog, NewClinic, NewClinicMember, NewDoctor, NewProfile, NewSession,
    NewUser,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Administrator,
    Doctor,
    Nursing,
    Secretary,
}

impl Role {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "administrator" => Some(Self::Administrator),
            "doctor" => Some(Self::Doctor),
            "nursing" => Some(Self::Nursing),
            "secretary" => Some(Self::Secretary),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Administrator => "administrator",
            Self::Doctor => "doctor",
            Self::Nursing => "nursing",
            Self::Secretary => "secretary",
        }
    }
}

#[derive(Debug)]
pub struct ActorContext {
    pub user: db::User,
    pub session_id: String,
    pub device_id: Option<String>,
    pub profile: db::Profile,
    pub clinic: Option<db::Clinic>,
    pub member: Option<db::ClinicMember>,
    pub doctor: Option<db::Doctor>,
}

impl ActorContext {
    pub fn role(&self) -> Option<Role> {
        self.member
            .as_ref()
            .and_then(|member| Role::parse(&member.role))
    }

    pub fn clinic_id(&self) -> Result<&str, AuthError> {
        self.clinic
            .as_ref()
            .map(|clinic| clinic.id.as_str())
            .ok_or_else(|| AuthError::Forbidden("No clinic membership.".to_owned()))
    }

    pub fn require_administrator(&self) -> Result<(), AuthError> {
        match self.role() {
            Some(Role::Administrator) => Ok(()),
            _ => Err(AuthError::Forbidden(
                "Administrator role required.".to_owned(),
            )),
        }
    }

    pub fn require_patient_read(&self) -> Result<(), AuthError> {
        match self.role() {
            Some(_) => Ok(()),
            None => Err(AuthError::Forbidden(
                "A clinic role is required.".to_owned(),
            )),
        }
    }

    pub fn require_patient_write(&self) -> Result<(), AuthError> {
        match self.role() {
            Some(Role::Doctor | Role::Nursing | Role::Secretary) => Ok(()),
            Some(Role::Administrator) => Err(AuthError::Forbidden(
                "The administrator can only read patients.".to_owned(),
            )),
            None => Err(AuthError::Forbidden(
                "A clinic role is required.".to_owned(),
            )),
        }
    }
}

pub struct RegisterUser {
    pub user_id: String,
    pub profile_id: String,
    pub session_id: String,
    pub email: String,
    pub display_name: String,
    pub password_hash: String,
    pub token_hash: String,
    pub created_at: String,
    pub expires_at: String,
}

pub struct ProfileUpdate {
    pub display_name: String,
}

pub struct DoctorUpdate {
    pub specialty: Option<String>,
    pub license_no: Option<String>,
    pub national_id: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}

pub struct UserRow {
    pub profile: db::Profile,
    pub role: Option<String>,
    pub email: String,
}

pub struct DeviceRegistration {
    pub device_id: String,
    pub platform: String,
    pub name: Option<String>,
    pub app_version: Option<String>,
}

pub struct DeviceRow {
    pub device: db::Device,
    pub profile_name: String,
}

pub struct DeviceList {
    pub devices: Vec<DeviceRow>,
    pub current_device_id: Option<String>,
}

pub struct PatientInput {
    pub full_name: String,
    pub national_id: Option<String>,
    pub birth_date: Option<String>,
    pub sex: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
    pub client_id: Option<String>,
}

pub struct PatientUpdate {
    pub full_name: Option<String>,
    pub national_id: Option<String>,
    pub birth_date: Option<String>,
    pub sex: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
}

#[allow(clippy::large_enum_variant)]
pub enum PatientCreateOutcome {
    Created(db::Patient),
    LikelyDuplicate(Vec<String>),
}

pub fn record_audit(
    connection: &mut SqliteConnection,
    actor_id: Option<&str>,
    action: &str,
    entity: &str,
    entity_id: &str,
    metadata: Option<Value>,
    at: &str,
) -> Result<(), AuthError> {
    let id = Uuid::now_v7().to_string();
    let metadata_json = metadata.and_then(|value| serde_json::to_string(&value).ok());
    db::insert_audit(
        connection,
        &NewAuditLog {
            id: &id,
            actor_id,
            action,
            entity,
            entity_id,
            metadata_json: metadata_json.as_deref(),
            at,
        },
    )?;
    Ok(())
}

pub fn create_clinic(
    connection: &mut SqliteConnection,
    creator: Option<&str>,
    name: &str,
    created_at: &str,
) -> Result<db::Clinic, AuthError> {
    let created = Utc::now();
    let server_retention = created
        .checked_add_months(Months::new(3))
        .unwrap_or(created);
    let grace_ends = server_retention + ChronoDuration::days(30);
    let server_retention_until = server_retention.to_rfc3339_opts(SecondsFormat::Secs, true);
    let data_grace_ends_at = grace_ends.to_rfc3339_opts(SecondsFormat::Secs, true);
    let id = Uuid::now_v7().to_string();
    db::insert_clinic(
        connection,
        &NewClinic {
            id: &id,
            name,
            address: None,
            phone: None,
            logo_asset_id: None,
            timezone: "America/Guayaquil",
            llm_url_template: None,
            retention_days: 30,
            rx_block_size: 50,
            server_retention_until: &server_retention_until,
            data_grace_ends_at: Some(&data_grace_ends_at),
            created_at,
            updated_at: created_at,
            created_by: creator,
        },
    )?;
    Ok(db::Clinic {
        id,
        name: name.to_owned(),
        address: None,
        phone: None,
        logo_asset_id: None,
        timezone: "America/Guayaquil".to_owned(),
        llm_url_template: None,
        retention_days: 30,
        rx_block_size: 50,
        server_retention_until,
        data_grace_ends_at: Some(data_grace_ends_at),
        created_at: created_at.to_owned(),
        updated_at: created_at.to_owned(),
        created_by: creator.map(str::to_owned),
    })
}

pub async fn register_user(pool: &DbPool, input: RegisterUser) -> Result<db::User, AuthError> {
    auth_server::with_conn(pool, move |connection| {
        db::immediate_transaction(connection, |connection| {
            if db::find_user_by_email(connection, &input.email)?.is_some() {
                return Err(AuthError::Conflict(
                    "An account with this email already exists.".to_owned(),
                ));
            }
            db::insert_user(
                connection,
                &NewUser {
                    id: &input.user_id,
                    email: &input.email,
                    password_hash: &input.password_hash,
                    display_name: &input.display_name,
                    created_at: &input.created_at,
                    updated_at: &input.created_at,
                },
            )?;
            db::insert_profile(
                connection,
                &NewProfile {
                    id: &input.profile_id,
                    auth_user_id: &input.user_id,
                    display_name: &input.display_name,
                    status: "active",
                    created_at: &input.created_at,
                    updated_at: &input.created_at,
                    created_by: None,
                },
            )?;
            if db::count_clinics(connection)? == 0 {
                let clinic = create_clinic(
                    connection,
                    Some(&input.profile_id),
                    &format!("Clínica de {}", input.display_name),
                    &input.created_at,
                )?;
                db::insert_clinic_member(
                    connection,
                    &NewClinicMember {
                        clinic_id: &clinic.id,
                        profile_id: &input.profile_id,
                        role: Role::Administrator.as_str(),
                        created_at: &input.created_at,
                        updated_at: &input.created_at,
                        created_by: None,
                    },
                )?;
                record_audit(
                    connection,
                    Some(&input.profile_id),
                    "create_clinic",
                    "clinic",
                    &clinic.id,
                    Some(json!({ "bootstrap": true })),
                    &input.created_at,
                )?;
            }
            db::insert_session(
                connection,
                &NewSession {
                    id: &input.session_id,
                    token_hash: &input.token_hash,
                    user_id: &input.user_id,
                    expires_at: &input.expires_at,
                    created_at: &input.created_at,
                    device_id: None,
                    mfa_pending: 0,
                },
            )?;
            record_audit(
                connection,
                None,
                "register_user",
                "profile",
                &input.profile_id,
                Some(json!({ "email": input.email.clone() })),
                &input.created_at,
            )?;
            Ok(db::User {
                id: input.user_id.clone(),
                email: input.email.clone(),
                password_hash: input.password_hash.clone(),
                display_name: input.display_name.clone(),
                created_at: input.created_at.clone(),
                updated_at: input.created_at.clone(),
            })
        })
    })
    .await
}

pub async fn resolve_actor(
    pool: &DbPool,
    headers: &HeaderMap,
) -> Result<Option<ActorContext>, AuthError> {
    let Some(token) = auth_server::cookie_value(headers, SESSION_COOKIE) else {
        return Ok(None);
    };
    let token_hash = auth_server::hash_token(&token);
    auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::delete_expired_sessions(connection, &at)?;
        let Some(session) = db::find_session_by_token_hash(connection, &token_hash)? else {
            return Ok(None);
        };
        if session.mfa_pending != 0 {
            return Ok(None);
        }
        if let Some(device_id) = session.device_id.as_deref() {
            let valid = db::find_device_by_id(connection, device_id)?
                .is_some_and(|device| device.revoked_at.is_none());
            if !valid {
                db::delete_session_by_token_hash(connection, &token_hash)?;
                return Ok(None);
            }
        }
        let user_id = session.user_id;
        let Some(user) = db::find_user_by_id(connection, &user_id)? else {
            return Ok(None);
        };
        let profile = match db::find_profile_by_auth_user_id(connection, &user.id)? {
            Some(profile) => profile,
            None => {
                let profile = db::Profile {
                    id: Uuid::now_v7().to_string(),
                    auth_user_id: user.id.clone(),
                    display_name: user.display_name.clone(),
                    status: "active".to_owned(),
                    created_at: at.clone(),
                    updated_at: at.clone(),
                    created_by: None,
                };
                db::insert_profile(
                    connection,
                    &NewProfile {
                        id: &profile.id,
                        auth_user_id: &profile.auth_user_id,
                        display_name: &profile.display_name,
                        status: &profile.status,
                        created_at: &profile.created_at,
                        updated_at: &profile.updated_at,
                        created_by: None,
                    },
                )?;
                profile
            }
        };
        if profile.status == "disabled" {
            return Err(AuthError::Forbidden("Profile disabled.".to_owned()));
        }
        let member = db::find_membership_by_profile(connection, &profile.id)?;
        let clinic = match member.as_ref() {
            Some(member) => db::find_clinic_by_id(connection, &member.clinic_id)?,
            None => None,
        };
        let doctor = db::find_doctor_by_profile_id(connection, &profile.id)?;
        Ok(Some(ActorContext {
            user,
            session_id: session.id,
            device_id: session.device_id,
            profile,
            clinic,
            member,
            doctor,
        }))
    })
    .await
}

pub async fn register_device(
    pool: &DbPool,
    actor: &ActorContext,
    input: DeviceRegistration,
) -> Result<db::Device, AuthError> {
    let device_id = input.device_id.trim().to_owned();
    if device_id.is_empty() || device_id.chars().count() > 200 {
        return Err(AuthError::BadRequest("Invalid device id.".to_owned()));
    }
    let platform = input.platform.trim().to_owned();
    if !matches!(platform.as_str(), "macos" | "windows" | "web") {
        return Err(AuthError::BadRequest("Unknown platform.".to_owned()));
    }
    let name = clean_optional(input.name);
    if name.as_ref().is_some_and(|name| name.chars().count() > 120) {
        return Err(AuthError::BadRequest("Device name is too long.".to_owned()));
    }
    let app_version = clean_optional(input.app_version);
    if app_version
        .as_ref()
        .is_some_and(|version| version.chars().count() > 64)
    {
        return Err(AuthError::BadRequest("App version is too long.".to_owned()));
    }
    let profile_id = actor.profile.id.clone();
    let session_id = actor.session_id.clone();
    auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
            let existing = db::find_device_by_id(connection, &device_id)?;
            let device = match existing {
                Some(device) => {
                    if device.profile_id != profile_id {
                        return Err(AuthError::Conflict("Device already registered.".to_owned()));
                    }
                    if device.revoked_at.is_some() {
                        return Err(AuthError::Forbidden(
                            "This device has been revoked.".to_owned(),
                        ));
                    }
                    db::update_device_seen(
                        connection,
                        &device_id,
                        name.as_deref(),
                        app_version.as_deref(),
                        &at,
                    )?;
                    db::find_device_by_id(connection, &device_id)?.ok_or(AuthError::Internal)?
                }
                None => {
                    db::insert_device(
                        connection,
                        &db::NewDevice {
                            id: &device_id,
                            profile_id: &profile_id,
                            name: name.as_deref(),
                            platform: &platform,
                            app_version: app_version.as_deref(),
                            last_seen_at: Some(&at),
                            revoked_at: None,
                            created_at: &at,
                            updated_at: &at,
                            created_by: Some(&profile_id),
                        },
                    )?;
                    db::find_device_by_id(connection, &device_id)?.ok_or(AuthError::Internal)?
                }
            };
            db::link_session_device(connection, &session_id, &device_id)?;
            Ok(device)
        })
    })
    .await
}

pub async fn list_devices(pool: &DbPool, actor: &ActorContext) -> Result<DeviceList, AuthError> {
    let profile_id = actor.profile.id.clone();
    let profile_name = actor.profile.display_name.clone();
    let clinic_id = actor.clinic.as_ref().map(|clinic| clinic.id.clone());
    let is_administrator = actor.role() == Some(Role::Administrator);
    let current_device_id = actor.device_id.clone();
    auth_server::with_conn(pool, move |connection| {
        let rows = if is_administrator {
            match clinic_id {
                Some(clinic_id) => db::list_devices_by_clinic(connection, &clinic_id)?,
                None => db::list_devices_by_profile(connection, &profile_id)?
                    .into_iter()
                    .map(|device| (device, profile_name.clone()))
                    .collect(),
            }
        } else {
            db::list_devices_by_profile(connection, &profile_id)?
                .into_iter()
                .map(|device| (device, profile_name.clone()))
                .collect()
        };
        Ok(DeviceList {
            devices: rows
                .into_iter()
                .map(|(device, profile_name)| DeviceRow {
                    device,
                    profile_name,
                })
                .collect(),
            current_device_id,
        })
    })
    .await
}

pub async fn revoke_device(
    pool: &DbPool,
    actor: &ActorContext,
    device_id: String,
) -> Result<(), AuthError> {
    let device_id = device_id.trim().to_owned();
    if device_id.is_empty() {
        return Err(AuthError::BadRequest("Invalid device id.".to_owned()));
    }
    let actor_profile_id = actor.profile.id.clone();
    let is_administrator = actor.role() == Some(Role::Administrator);
    let clinic_id = actor.clinic.as_ref().map(|clinic| clinic.id.clone());
    auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
            let device = db::find_device_by_id(connection, &device_id)?
                .ok_or_else(|| AuthError::NotFound("Device not found.".to_owned()))?;
            let is_owner = device.profile_id == actor_profile_id;
            let in_clinic = if is_administrator {
                match clinic_id.as_deref() {
                    Some(clinic_id) => {
                        db::find_membership_by_profile(connection, &device.profile_id)?
                            .is_some_and(|member| member.clinic_id == clinic_id)
                    }
                    None => false,
                }
            } else {
                false
            };
            if !is_owner && !in_clinic {
                return Err(AuthError::NotFound("Device not found.".to_owned()));
            }
            if device.revoked_at.is_none() {
                db::revoke_device(connection, &device_id, &at)?;
                record_audit(
                    connection,
                    Some(&actor_profile_id),
                    "revoke_device",
                    "device",
                    &device_id,
                    Some(json!({ "profileId": device.profile_id })),
                    &at,
                )?;
            }
            db::delete_sessions_by_device(connection, &device_id)?;
            Ok(())
        })
    })
    .await
}

pub async fn list_users(pool: &DbPool, actor: &ActorContext) -> Result<Vec<UserRow>, AuthError> {
    actor.require_administrator()?;
    let clinic_id = actor.clinic_id()?.to_owned();
    auth_server::with_conn(pool, move |connection| {
        let rows = db::list_profiles_with_membership(connection, &clinic_id)?;
        Ok(rows
            .into_iter()
            .map(|(profile, role, email)| UserRow {
                profile,
                role,
                email,
            })
            .collect())
    })
    .await
}

pub async fn set_member_role(
    pool: &DbPool,
    actor: &ActorContext,
    profile_id: String,
    role: String,
) -> Result<(), AuthError> {
    actor.require_administrator()?;
    let clinic_id = actor.clinic_id()?.to_owned();
    let actor_id = actor.profile.id.clone();
    let Some(role) = Role::parse(&role) else {
        return Err(AuthError::BadRequest("Unknown role.".to_owned()));
    };
    auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
            if db::find_profile_by_id(connection, &profile_id)?.is_none() {
                return Err(AuthError::NotFound("Profile not found.".to_owned()));
            }
            let member = db::find_membership_by_profile(connection, &profile_id)?;
            if let Some(member) = member.as_ref() {
                if member.clinic_id != clinic_id {
                    return Err(AuthError::NotFound("Profile not found.".to_owned()));
                }
                if member.role == Role::Administrator.as_str()
                    && role != Role::Administrator
                    && db::count_administrators(connection, &clinic_id)? <= 1
                {
                    return Err(AuthError::BadRequest(
                        "The clinic needs an administrator.".to_owned(),
                    ));
                }
                db::update_clinic_member_role(
                    connection,
                    &clinic_id,
                    &profile_id,
                    role.as_str(),
                    &at,
                )?;
            } else {
                db::insert_clinic_member(
                    connection,
                    &NewClinicMember {
                        clinic_id: &clinic_id,
                        profile_id: &profile_id,
                        role: role.as_str(),
                        created_at: &at,
                        updated_at: &at,
                        created_by: Some(&actor_id),
                    },
                )?;
            }
            if role == Role::Doctor
                && db::find_doctor_by_profile_id(connection, &profile_id)?.is_none()
            {
                let doctor_id = Uuid::now_v7().to_string();
                db::insert_doctor(
                    connection,
                    &NewDoctor {
                        id: &doctor_id,
                        profile_id: &profile_id,
                        specialty: None,
                        license_no: None,
                        national_id: None,
                        phone: None,
                        email: None,
                        created_at: &at,
                        updated_at: &at,
                        created_by: Some(&actor_id),
                    },
                )?;
            }
            record_audit(
                connection,
                Some(&actor_id),
                "set_member_role",
                "clinic_member",
                &profile_id,
                Some(json!({ "role": role.as_str(), "clinic_id": clinic_id })),
                &at,
            )
        })
    })
    .await
}

pub async fn remove_member(
    pool: &DbPool,
    actor: &ActorContext,
    profile_id: String,
) -> Result<(), AuthError> {
    actor.require_administrator()?;
    let clinic_id = actor.clinic_id()?.to_owned();
    let actor_id = actor.profile.id.clone();
    auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
            let member = db::find_membership_by_profile(connection, &profile_id)?
                .ok_or_else(|| AuthError::NotFound("Member not found.".to_owned()))?;
            if member.clinic_id != clinic_id {
                return Err(AuthError::NotFound("Member not found.".to_owned()));
            }
            if member.role == Role::Administrator.as_str()
                && db::count_administrators(connection, &clinic_id)? <= 1
            {
                return Err(AuthError::BadRequest(
                    "The clinic needs an administrator.".to_owned(),
                ));
            }
            db::delete_clinic_member(connection, &clinic_id, &profile_id)?;
            record_audit(
                connection,
                Some(&actor_id),
                "remove_member",
                "clinic_member",
                &profile_id,
                Some(json!({ "clinic_id": clinic_id })),
                &at,
            )
        })
    })
    .await
}

pub async fn disable_profile(
    pool: &DbPool,
    actor: &ActorContext,
    profile_id: String,
    disabled: bool,
) -> Result<(), AuthError> {
    actor.require_administrator()?;
    let clinic_id = actor.clinic_id()?.to_owned();
    let actor_id = actor.profile.id.clone();
    auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
            if db::find_profile_by_id(connection, &profile_id)?.is_none() {
                return Err(AuthError::NotFound("Profile not found.".to_owned()));
            }
            let member = db::find_membership_by_profile(connection, &profile_id)?;
            if disabled
                && member
                    .as_ref()
                    .is_some_and(|member| member.clinic_id == clinic_id)
                && member
                    .as_ref()
                    .is_some_and(|member| member.role == Role::Administrator.as_str())
                && db::count_administrators(connection, &clinic_id)? <= 1
            {
                return Err(AuthError::BadRequest(
                    "The clinic needs an administrator.".to_owned(),
                ));
            }
            let status = if disabled { "disabled" } else { "active" };
            db::update_profile_status(connection, &profile_id, status, &at)?;
            record_audit(
                connection,
                Some(&actor_id),
                "disable_profile",
                "profile",
                &profile_id,
                Some(json!({ "status": status })),
                &at,
            )
        })
    })
    .await
}

pub async fn update_profile(
    pool: &DbPool,
    actor: &ActorContext,
    input: ProfileUpdate,
) -> Result<db::Profile, AuthError> {
    let display_name = input.display_name.trim().to_owned();
    if display_name.is_empty() {
        return Err(AuthError::BadRequest("Enter a display name.".to_owned()));
    }
    if display_name.chars().count() > 120 {
        return Err(AuthError::BadRequest(
            "Display name is too long.".to_owned(),
        ));
    }
    let profile_id = actor.profile.id.clone();
    let user_id = actor.user.id.clone();
    auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
            db::update_profile_display_name(connection, &profile_id, &display_name, &at)?;
            db::update_user_display_name(connection, &user_id, &display_name, &at)?;
            record_audit(
                connection,
                Some(&profile_id),
                "update_profile",
                "profile",
                &profile_id,
                Some(json!({ "display_name": display_name })),
                &at,
            )?;
            db::find_profile_by_id(connection, &profile_id)?
                .ok_or_else(|| AuthError::NotFound("Profile not found.".to_owned()))
        })
    })
    .await
}

pub async fn update_doctor(
    pool: &DbPool,
    actor: &ActorContext,
    input: DoctorUpdate,
) -> Result<db::Doctor, AuthError> {
    if actor.role() != Some(Role::Doctor) {
        return Err(AuthError::Forbidden("Doctor role required.".to_owned()));
    }
    let profile_id = actor.profile.id.clone();
    let actor_id = actor.profile.id.clone();
    auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
            let fields = db::DoctorFields {
                specialty: input.specialty.as_deref(),
                license_no: input.license_no.as_deref(),
                national_id: input.national_id.as_deref(),
                phone: input.phone.as_deref(),
                email: input.email.as_deref(),
            };
            let existing = db::find_doctor_by_profile_id(connection, &profile_id)?;
            let doctor_id = match existing {
                Some(doctor) => {
                    db::update_doctor(connection, &doctor.id, &fields, &at)?;
                    doctor.id
                }
                None => {
                    let doctor_id = Uuid::now_v7().to_string();
                    db::insert_doctor(
                        connection,
                        &NewDoctor {
                            id: &doctor_id,
                            profile_id: &profile_id,
                            specialty: fields.specialty,
                            license_no: fields.license_no,
                            national_id: fields.national_id,
                            phone: fields.phone,
                            email: fields.email,
                            created_at: &at,
                            updated_at: &at,
                            created_by: Some(&actor_id),
                        },
                    )?;
                    doctor_id
                }
            };
            record_audit(
                connection,
                Some(&actor_id),
                "update_doctor",
                "doctor",
                &doctor_id,
                None,
                &at,
            )?;
            db::find_doctor_by_profile_id(connection, &profile_id)?
                .ok_or_else(|| AuthError::NotFound("Doctor not found.".to_owned()))
        })
    })
    .await
}

fn clean_name(value: &str) -> Result<String, AuthError> {
    let name = value.trim().to_owned();
    if name.is_empty() {
        return Err(AuthError::BadRequest("Enter the patient name.".to_owned()));
    }
    if name.chars().count() > 200 {
        return Err(AuthError::BadRequest(
            "Patient name is too long.".to_owned(),
        ));
    }
    Ok(name)
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
}

fn clean_birth_date(value: Option<String>) -> Result<Option<String>, AuthError> {
    let Some(date) = clean_optional(value) else {
        return Ok(None);
    };
    NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|_| AuthError::BadRequest("Enter a valid birth date.".to_owned()))?;
    Ok(Some(date))
}

fn clean_sex(value: Option<String>) -> Result<Option<String>, AuthError> {
    let Some(sex) = clean_optional(value) else {
        return Ok(None);
    };
    match sex.as_str() {
        "male" | "female" | "other" | "unspecified" => Ok(Some(sex)),
        _ => Err(AuthError::BadRequest("Select a valid sex.".to_owned())),
    }
}

fn normalize_name(value: &str) -> String {
    value
        .case_fold()
        .collect::<String>()
        .nfd()
        .filter(|character| !is_combining_mark(*character))
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub async fn create_patient(
    pool: &DbPool,
    actor: &ActorContext,
    input: PatientInput,
    confirm_duplicate: bool,
) -> Result<PatientCreateOutcome, AuthError> {
    actor.require_patient_write()?;
    let clinic_id = actor.clinic_id()?.to_owned();
    let actor_id = actor.profile.id.clone();
    let full_name = clean_name(&input.full_name)?;
    let national_id = clean_optional(input.national_id);
    let birth_date = clean_birth_date(input.birth_date)?;
    let sex = clean_sex(input.sex)?;
    let phone = clean_optional(input.phone);
    let notes = clean_optional(input.notes);
    let normalized_name = normalize_name(&full_name);
    let client_id = input
        .client_id
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());
    auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
            if let Some(client_id) = client_id.as_deref() {
                if let Some(existing) =
                    db::find_patient_by_client_id(connection, &clinic_id, client_id)?
                {
                    return Ok(PatientCreateOutcome::Created(existing));
                }
            }
            if let Some(national_id) = national_id.as_deref() {
                if db::find_patient_id_by_national_id(connection, &clinic_id, national_id)?
                    .is_some()
                {
                    return Err(AuthError::DuplicateNationalId);
                }
            }
            if !confirm_duplicate {
                let duplicates: Vec<String> = db::list_patient_names(connection, &clinic_id)?
                    .into_iter()
                    .filter_map(|(id, name)| {
                        (normalize_name(&name) == normalized_name).then_some(id)
                    })
                    .collect();
                if !duplicates.is_empty() {
                    return Ok(PatientCreateOutcome::LikelyDuplicate(duplicates));
                }
            }
            let id = Uuid::now_v7().to_string();
            let inserted = db::insert_patient(
                connection,
                &db::NewPatient {
                    id: &id,
                    clinic_id: &clinic_id,
                    full_name: &full_name,
                    national_id: national_id.as_deref(),
                    birth_date: birth_date.as_deref(),
                    sex: sex.as_deref(),
                    phone: phone.as_deref(),
                    notes: notes.as_deref(),
                    client_id: client_id.as_deref(),
                    deleted_at: None,
                    created_at: &at,
                    updated_at: &at,
                    created_by: Some(&actor_id),
                },
            );
            match inserted {
                Err(diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                )) => return Err(AuthError::DuplicateNationalId),
                Err(error) => return Err(error.into()),
                Ok(_) => {}
            }
            record_audit(
                connection,
                Some(&actor_id),
                "create_patient",
                "patient",
                &id,
                None,
                &at,
            )?;
            db::find_patient_by_id(connection, &clinic_id, &id)?
                .map(PatientCreateOutcome::Created)
                .ok_or_else(|| AuthError::NotFound("Patient not found.".to_owned()))
        })
    })
    .await
}

pub async fn update_patient(
    pool: &DbPool,
    actor: &ActorContext,
    patient_id: String,
    input: PatientUpdate,
) -> Result<db::Patient, AuthError> {
    actor.require_patient_write()?;
    let clinic_id = actor.clinic_id()?.to_owned();
    let actor_id = actor.profile.id.clone();
    auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
            let existing = db::find_patient_by_id(connection, &clinic_id, &patient_id)?
                .ok_or_else(|| AuthError::NotFound("Patient not found.".to_owned()))?;
            let full_name = match input.full_name.as_deref() {
                Some(value) => clean_name(value)?,
                None => existing.full_name.clone(),
            };
            let national_id = match input.national_id.as_deref() {
                Some(value) => clean_optional(Some(value.to_owned())),
                None => existing.national_id.clone(),
            };
            let birth_date = match input.birth_date.as_deref() {
                Some(value) => clean_birth_date(Some(value.to_owned()))?,
                None => existing.birth_date.clone(),
            };
            let sex = match input.sex.as_deref() {
                Some(value) => clean_sex(Some(value.to_owned()))?,
                None => existing.sex.clone(),
            };
            let phone = match input.phone.as_deref() {
                Some(value) => clean_optional(Some(value.to_owned())),
                None => existing.phone.clone(),
            };
            let notes = match input.notes.as_deref() {
                Some(value) => clean_optional(Some(value.to_owned())),
                None => existing.notes.clone(),
            };
            if let Some(national_id) = national_id.as_deref() {
                if let Some(other_id) =
                    db::find_patient_id_by_national_id(connection, &clinic_id, national_id)?
                {
                    if other_id != patient_id {
                        return Err(AuthError::DuplicateNationalId);
                    }
                }
            }
            let updated = db::update_patient(
                connection,
                &clinic_id,
                &patient_id,
                &db::PatientFields {
                    full_name: &full_name,
                    national_id: national_id.as_deref(),
                    birth_date: birth_date.as_deref(),
                    sex: sex.as_deref(),
                    phone: phone.as_deref(),
                    notes: notes.as_deref(),
                },
                &at,
            );
            match updated {
                Err(diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                )) => return Err(AuthError::DuplicateNationalId),
                Err(error) => return Err(error.into()),
                Ok(_) => {}
            }
            record_audit(
                connection,
                Some(&actor_id),
                "update_patient",
                "patient",
                &patient_id,
                None,
                &at,
            )?;
            db::find_patient_by_id(connection, &clinic_id, &patient_id)?
                .ok_or_else(|| AuthError::NotFound("Patient not found.".to_owned()))
        })
    })
    .await
}

pub async fn search_patients(
    pool: &DbPool,
    actor: &ActorContext,
    query: Option<String>,
) -> Result<Vec<db::Patient>, AuthError> {
    actor.require_patient_read()?;
    let clinic_id = actor.clinic_id()?.to_owned();
    let actor_id = actor.profile.id.clone();
    let query = query
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());
    auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
            let found = db::search_patients(connection, &clinic_id, query.as_deref(), 50)?;
            if found.is_empty() {
                record_audit(
                    connection,
                    Some(&actor_id),
                    "sensitive_read",
                    "patient",
                    &clinic_id,
                    Some(json!({ "via": "search", "results": 0 })),
                    &at,
                )?;
            } else {
                for patient in &found {
                    record_audit(
                        connection,
                        Some(&actor_id),
                        "sensitive_read",
                        "patient",
                        &patient.id,
                        Some(json!({ "via": "search" })),
                        &at,
                    )?;
                }
            }
            Ok(found)
        })
    })
    .await
}

pub async fn get_patient(
    pool: &DbPool,
    actor: &ActorContext,
    patient_id: String,
) -> Result<db::Patient, AuthError> {
    actor.require_patient_read()?;
    let clinic_id = actor.clinic_id()?.to_owned();
    let actor_id = actor.profile.id.clone();
    auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
            let patient = db::find_patient_by_id(connection, &clinic_id, &patient_id)?;
            record_audit(
                connection,
                Some(&actor_id),
                "sensitive_read",
                "patient",
                &patient_id,
                Some(json!({ "via": "detail", "found": patient.is_some() })),
                &at,
            )?;
            patient.ok_or_else(|| AuthError::NotFound("Patient not found.".to_owned()))
        })
    })
    .await
}

pub async fn soft_delete_patient(
    pool: &DbPool,
    actor: &ActorContext,
    patient_id: String,
) -> Result<(), AuthError> {
    actor.require_patient_write()?;
    let clinic_id = actor.clinic_id()?.to_owned();
    let actor_id = actor.profile.id.clone();
    auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
            db::find_patient_by_id(connection, &clinic_id, &patient_id)?
                .ok_or_else(|| AuthError::NotFound("Patient not found.".to_owned()))?;
            db::soft_delete_patient(connection, &clinic_id, &patient_id, &at)?;
            record_audit(
                connection,
                Some(&actor_id),
                "delete_patient",
                "patient",
                &patient_id,
                None,
                &at,
            )
        })
    })
    .await
}

pub struct TemplateVersionInput {
    pub doc_json: String,
    pub theme: Option<String>,
}

fn template_name(value: &str) -> Result<String, AuthError> {
    let name = value.trim().to_owned();
    if name.is_empty() {
        return Err(AuthError::BadRequest(
            "Escribe el nombre de la plantilla.".to_owned(),
        ));
    }
    if name.chars().count() > 120 {
        return Err(AuthError::BadRequest(
            "El nombre de la plantilla es demasiado largo.".to_owned(),
        ));
    }
    Ok(name)
}

fn template_document(value: &str) -> Result<String, AuthError> {
    let document = crate::forms::parse_document(value).map_err(AuthError::BadRequest)?;
    serde_json::to_string(&document).map_err(|_| AuthError::Internal)
}

fn template_theme(value: Option<String>) -> Result<String, AuthError> {
    let raw = value.unwrap_or_else(|| "{\"margin_mm\":15}".to_owned());
    let parsed: Value = serde_json::from_str(&raw)
        .map_err(|_| AuthError::BadRequest("El tema de la plantilla no es válido.".to_owned()))?;
    let Some(object) = parsed.as_object() else {
        return Err(AuthError::BadRequest(
            "El tema de la plantilla debe ser un objeto.".to_owned(),
        ));
    };
    if let Some(margin) = object.get("margin_mm") {
        let Some(margin) = margin.as_u64() else {
            return Err(AuthError::BadRequest(
                "El margen debe ser un número entero en milímetros.".to_owned(),
            ));
        };
        if !(5..=40).contains(&margin) {
            return Err(AuthError::BadRequest(
                "El margen debe estar entre 5 y 40 mm.".to_owned(),
            ));
        }
    }
    serde_json::to_string(&parsed).map_err(|_| AuthError::Internal)
}

fn canonical_variables() -> Result<String, AuthError> {
    serde_json::to_string(&crate::forms::TEMPLATE_VARIABLES).map_err(|_| AuthError::Internal)
}

fn template_access(
    is_administrator: bool,
    is_nursing: bool,
    doctor_id: Option<&str>,
    clinic_id: Option<&str>,
    template: &db::Template,
    write: bool,
) -> Result<(), AuthError> {
    if is_administrator || (is_nursing && !write) {
        if template.owner_type == "clinic" && Some(template.owner_id.as_str()) == clinic_id {
            return Ok(());
        }
        return Err(AuthError::NotFound("Plantilla no encontrada.".to_owned()));
    }
    let Some(doctor_id) = doctor_id else {
        return Err(AuthError::Forbidden(
            "El rol de doctor es necesario para gestionar plantillas.".to_owned(),
        ));
    };
    let own = template.owner_type == "doctor" && template.owner_id == doctor_id;
    let clinic_master =
        template.owner_type == "clinic" && Some(template.owner_id.as_str()) == clinic_id;
    if own || (!write && clinic_master) {
        return Ok(());
    }
    Err(AuthError::NotFound("Plantilla no encontrada.".to_owned()))
}

pub async fn list_templates(
    pool: &DbPool,
    actor: &ActorContext,
) -> Result<Vec<(db::Template, Option<db::TemplateVersion>)>, AuthError> {
    let is_administrator = actor.role() == Some(Role::Administrator);
    let is_nursing = actor.role() == Some(Role::Nursing);
    let doctor_id = actor.doctor.as_ref().map(|doctor| doctor.id.clone());
    let clinic_id = actor.clinic.as_ref().map(|clinic| clinic.id.clone());
    if !is_administrator && !is_nursing && doctor_id.is_none() {
        return Err(AuthError::Forbidden(
            "El rol de doctor es necesario para ver plantillas.".to_owned(),
        ));
    }
    auth_server::with_conn(pool, move |connection| {
        let mut rows = Vec::new();
        if is_administrator || is_nursing {
            if let Some(clinic_id) = clinic_id.as_deref() {
                rows.extend(db::list_templates_by_owner(
                    connection, "clinic", clinic_id,
                )?);
            }
        } else if let Some(doctor_id) = doctor_id.as_deref() {
            rows.extend(db::list_templates_by_owner(
                connection, "doctor", doctor_id,
            )?);
            if let Some(clinic_id) = clinic_id.as_deref() {
                rows.extend(db::list_templates_by_owner(
                    connection, "clinic", clinic_id,
                )?);
            }
        }
        let mut result = Vec::new();
        for template in rows {
            let version = match template.current_version_id.as_deref() {
                Some(id) => db::find_template_version_by_id(connection, id)?,
                None => None,
            };
            result.push((template, version));
        }
        Ok(result)
    })
    .await
}

pub async fn get_template(
    pool: &DbPool,
    actor: &ActorContext,
    template_id: String,
) -> Result<(db::Template, Vec<db::TemplateVersion>), AuthError> {
    let is_administrator = actor.role() == Some(Role::Administrator);
    let is_nursing = actor.role() == Some(Role::Nursing);
    let doctor_id = actor.doctor.as_ref().map(|doctor| doctor.id.clone());
    let clinic_id = actor.clinic.as_ref().map(|clinic| clinic.id.clone());
    auth_server::with_conn(pool, move |connection| {
        let template = db::find_template_by_id(connection, &template_id)?
            .ok_or_else(|| AuthError::NotFound("Plantilla no encontrada.".to_owned()))?;
        template_access(
            is_administrator,
            is_nursing,
            doctor_id.as_deref(),
            clinic_id.as_deref(),
            &template,
            false,
        )?;
        let versions = db::list_template_versions(connection, &template.id)?;
        Ok((template, versions))
    })
    .await
}

pub async fn create_template(
    pool: &DbPool,
    actor: &ActorContext,
    name: String,
    owner_type: Option<String>,
    client_id: Option<String>,
) -> Result<db::Template, AuthError> {
    let name = template_name(&name)?;
    let is_administrator = actor.role() == Some(Role::Administrator);
    let doctor_id = actor.doctor.as_ref().map(|doctor| doctor.id.clone());
    let clinic_id = actor.clinic.as_ref().map(|clinic| clinic.id.clone());
    let requested = owner_type.unwrap_or_else(|| {
        if is_administrator {
            "clinic".to_owned()
        } else {
            "doctor".to_owned()
        }
    });
    let (owner_type, owner_id) = match (is_administrator, requested.as_str()) {
        (true, "clinic") => (
            "clinic".to_owned(),
            clinic_id
                .clone()
                .ok_or_else(|| AuthError::Forbidden("Sin clínica asignada.".to_owned()))?,
        ),
        (false, "doctor") => (
            "doctor".to_owned(),
            doctor_id
                .clone()
                .ok_or_else(|| AuthError::Forbidden("El rol de doctor es necesario.".to_owned()))?,
        ),
        (true, _) => {
            return Err(AuthError::Forbidden(
                "El administrador solo crea plantillas maestras de la clínica.".to_owned(),
            ))
        }
        (false, _) => {
            return Err(AuthError::Forbidden(
                "El doctor solo crea sus propias plantillas.".to_owned(),
            ))
        }
    };
    let actor_id = actor.profile.id.clone();
    auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
            let client_id = client_id
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty());
            if let Some(client_id) = client_id {
                if let Some(existing) = db::find_template_by_client_id(connection, client_id)? {
                    return Ok(existing);
                }
            }
            let id = Uuid::now_v7().to_string();
            db::insert_template(
                connection,
                &db::NewTemplate {
                    id: &id,
                    owner_type: &owner_type,
                    owner_id: &owner_id,
                    name: &name,
                    current_version_id: None,
                    status: "draft",
                    client_id,
                    created_at: &at,
                    updated_at: &at,
                    created_by: Some(&actor_id),
                },
            )?;
            record_audit(
                connection,
                Some(&actor_id),
                "create_template",
                "template",
                &id,
                Some(json!({ "owner_type": owner_type, "name": name })),
                &at,
            )?;
            db::find_template_by_id(connection, &id)?
                .ok_or_else(|| AuthError::NotFound("Plantilla no encontrada.".to_owned()))
        })
    })
    .await
}

pub async fn create_template_version(
    pool: &DbPool,
    actor: &ActorContext,
    template_id: String,
    input: TemplateVersionInput,
) -> Result<db::TemplateVersion, AuthError> {
    let doc_json = template_document(&input.doc_json)?;
    let theme = template_theme(input.theme)?;
    let variables_spec = canonical_variables()?;
    let is_administrator = actor.role() == Some(Role::Administrator);
    let is_nursing = false;
    let doctor_id = actor.doctor.as_ref().map(|doctor| doctor.id.clone());
    let clinic_id = actor.clinic.as_ref().map(|clinic| clinic.id.clone());
    let actor_id = actor.profile.id.clone();
    auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
            let template = db::find_template_by_id(connection, &template_id)?
                .ok_or_else(|| AuthError::NotFound("Plantilla no encontrada.".to_owned()))?;
            template_access(
                is_administrator,
                is_nursing,
                doctor_id.as_deref(),
                clinic_id.as_deref(),
                &template,
                true,
            )?;
            if template.status == "archived" {
                return Err(AuthError::Conflict(
                    "La plantilla está archivada.".to_owned(),
                ));
            }
            let version_no =
                db::latest_template_version_no(connection, &template_id)?.unwrap_or(0) + 1;
            let id = Uuid::now_v7().to_string();
            db::insert_template_version(
                connection,
                &db::NewTemplateVersion {
                    id: &id,
                    template_id: &template.id,
                    version_no,
                    doc_json: &doc_json,
                    variables_spec: &variables_spec,
                    theme: Some(&theme),
                    status: "draft",
                    created_at: &at,
                    updated_at: &at,
                    created_by: Some(&actor_id),
                },
            )?;
            record_audit(
                connection,
                Some(&actor_id),
                "create_template_version",
                "template_version",
                &id,
                Some(json!({ "template_id": template.id, "version_no": version_no })),
                &at,
            )?;
            db::find_template_version(connection, &template.id, &id)?
                .ok_or_else(|| AuthError::NotFound("Versión no encontrada.".to_owned()))
        })
    })
    .await
}

pub async fn update_template_version(
    pool: &DbPool,
    actor: &ActorContext,
    template_id: String,
    version_id: String,
    input: TemplateVersionInput,
) -> Result<db::TemplateVersion, AuthError> {
    let doc_json = template_document(&input.doc_json)?;
    let theme = template_theme(input.theme)?;
    let variables_spec = canonical_variables()?;
    let is_administrator = actor.role() == Some(Role::Administrator);
    let is_nursing = false;
    let doctor_id = actor.doctor.as_ref().map(|doctor| doctor.id.clone());
    let clinic_id = actor.clinic.as_ref().map(|clinic| clinic.id.clone());
    let actor_id = actor.profile.id.clone();
    auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
            let template = db::find_template_by_id(connection, &template_id)?
                .ok_or_else(|| AuthError::NotFound("Plantilla no encontrada.".to_owned()))?;
            template_access(
                is_administrator,
                is_nursing,
                doctor_id.as_deref(),
                clinic_id.as_deref(),
                &template,
                true,
            )?;
            if template.status == "archived" {
                return Err(AuthError::Conflict(
                    "La plantilla está archivada.".to_owned(),
                ));
            }
            let version = db::find_template_version(connection, &template.id, &version_id)?
                .ok_or_else(|| AuthError::NotFound("Versión no encontrada.".to_owned()))?;
            let updated = db::update_template_version_draft(
                connection,
                &version.id,
                &doc_json,
                &variables_spec,
                Some(&theme),
                &at,
            )?;
            if updated == 0 {
                return Err(AuthError::Conflict(
                    "Una versión publicada es inmutable.".to_owned(),
                ));
            }
            record_audit(
                connection,
                Some(&actor_id),
                "update_template_version",
                "template_version",
                &version.id,
                Some(json!({ "template_id": template.id, "version_no": version.version_no })),
                &at,
            )?;
            db::find_template_version(connection, &template.id, &version.id)?
                .ok_or_else(|| AuthError::NotFound("Versión no encontrada.".to_owned()))
        })
    })
    .await
}

pub async fn publish_template_version(
    pool: &DbPool,
    actor: &ActorContext,
    template_id: String,
    version_id: String,
) -> Result<db::Template, AuthError> {
    let is_administrator = actor.role() == Some(Role::Administrator);
    let is_nursing = actor.role() == Some(Role::Nursing);
    let doctor_id = actor.doctor.as_ref().map(|doctor| doctor.id.clone());
    let clinic_id = actor.clinic.as_ref().map(|clinic| clinic.id.clone());
    let actor_id = actor.profile.id.clone();
    auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
            let template = db::find_template_by_id(connection, &template_id)?
                .ok_or_else(|| AuthError::NotFound("Plantilla no encontrada.".to_owned()))?;
            template_access(
                is_administrator,
                is_nursing,
                doctor_id.as_deref(),
                clinic_id.as_deref(),
                &template,
                true,
            )?;
            if template.status == "archived" {
                return Err(AuthError::Conflict(
                    "La plantilla está archivada.".to_owned(),
                ));
            }
            let version = db::find_template_version(connection, &template.id, &version_id)?
                .ok_or_else(|| AuthError::NotFound("Versión no encontrada.".to_owned()))?;
            if version.status == "published" {
                db::set_template_published(connection, &template.id, &version.id, &at)?;
                return db::find_template_by_id(connection, &template.id)?
                    .ok_or_else(|| AuthError::NotFound("Plantilla no encontrada.".to_owned()));
            }
            let published = db::publish_template_version_row(connection, &version.id, &at)?;
            if published == 0 {
                return Err(AuthError::Conflict(
                    "La versión ya está publicada y es inmutable.".to_owned(),
                ));
            }
            db::set_template_published(connection, &template.id, &version.id, &at)?;
            let document =
                crate::forms::parse_document(&version.doc_json).map_err(AuthError::BadRequest)?;
            let mut asset_ids: Vec<String> = document
                .images
                .into_iter()
                .filter_map(|image| image.asset_id)
                .collect();
            if let Some(logo) = document.letterhead.logo_asset_id {
                asset_ids.push(logo);
            }
            asset_ids.sort();
            asset_ids.dedup();
            for asset_id in asset_ids {
                let asset = db::find_asset_by_id(connection, &asset_id)?
                    .filter(|asset| {
                        clinic_id.as_deref() == Some(asset.clinic_id.as_str())
                            && asset.status == "ready"
                    })
                    .ok_or_else(|| {
                        AuthError::BadRequest(
                            "La plantilla referencia una imagen que no está disponible.".to_owned(),
                        )
                    })?;
                db::insert_template_version_asset(connection, &version.id, &asset.id)?;
            }
            record_audit(
                connection,
                Some(&actor_id),
                "publish_template_version",
                "template_version",
                &version.id,
                Some(json!({ "template_id": template.id, "version_no": version.version_no })),
                &at,
            )?;
            db::find_template_by_id(connection, &template.id)?
                .ok_or_else(|| AuthError::NotFound("Plantilla no encontrada.".to_owned()))
        })
    })
    .await
}

pub async fn archive_template(
    pool: &DbPool,
    actor: &ActorContext,
    template_id: String,
) -> Result<db::Template, AuthError> {
    let is_administrator = actor.role() == Some(Role::Administrator);
    let is_nursing = actor.role() == Some(Role::Nursing);
    let doctor_id = actor.doctor.as_ref().map(|doctor| doctor.id.clone());
    let clinic_id = actor.clinic.as_ref().map(|clinic| clinic.id.clone());
    let actor_id = actor.profile.id.clone();
    auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
            let template = db::find_template_by_id(connection, &template_id)?
                .ok_or_else(|| AuthError::NotFound("Plantilla no encontrada.".to_owned()))?;
            template_access(
                is_administrator,
                is_nursing,
                doctor_id.as_deref(),
                clinic_id.as_deref(),
                &template,
                true,
            )?;
            if template.status != "archived" {
                db::archive_template_row(connection, &template.id, &at)?;
                record_audit(
                    connection,
                    Some(&actor_id),
                    "archive_template",
                    "template",
                    &template.id,
                    None,
                    &at,
                )?;
            }
            db::find_template_by_id(connection, &template.id)?
                .ok_or_else(|| AuthError::NotFound("Plantilla no encontrada.".to_owned()))
        })
    })
    .await
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetKind {
    Image,
    Logo,
}

impl AssetKind {
    fn as_db(self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Logo => "logo",
        }
    }
}

#[derive(Debug)]
pub struct UploadFile {
    pub file_name: Option<String>,
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
pub struct ClinicSettings {
    pub clinic: db::Clinic,
    pub logo: Option<db::Asset>,
}

fn upload_owner(
    actor: &ActorContext,
    kind: AssetKind,
) -> Result<(&'static str, String, String), AuthError> {
    let clinic_id = actor.clinic_id()?.to_owned();
    match kind {
        AssetKind::Logo => {
            actor.require_administrator()?;
            Ok(("clinic", clinic_id.clone(), clinic_id))
        }
        AssetKind::Image => match actor.role() {
            Some(Role::Administrator) => Ok(("clinic", clinic_id.clone(), clinic_id)),
            Some(Role::Doctor) => {
                let doctor_id = actor
                    .doctor
                    .as_ref()
                    .map(|doctor| doctor.id.clone())
                    .ok_or_else(|| {
                        AuthError::Forbidden(
                            "El perfil de doctor es necesario para subir imágenes.".to_owned(),
                        )
                    })?;
                Ok(("doctor", doctor_id, clinic_id))
            }
            _ => Err(AuthError::Forbidden(
                "Solo el administrador y el doctor pueden subir imágenes.".to_owned(),
            )),
        },
    }
}

pub async fn upload_asset(
    pool: &DbPool,
    storage: &crate::assets::AssetStorage,
    actor: &ActorContext,
    kind: AssetKind,
    file: UploadFile,
) -> Result<db::Asset, AuthError> {
    let (owner_type, owner_id, clinic_id) = upload_owner(actor, kind)?;
    if file.bytes.is_empty() {
        return Err(AuthError::BadRequest("Adjunta una imagen.".to_owned()));
    }
    let bytes = file.bytes;
    let processed = tokio::task::spawn_blocking(move || crate::assets::process_image(&bytes))
        .await
        .map_err(|_| AuthError::Internal)??;
    let actor_id = actor.profile.id.clone();
    let asset_id = Uuid::now_v7().to_string();
    let object_key = crate::assets::object_key(&clinic_id, &asset_id);
    let original_name = file
        .file_name
        .map(|name| name.trim().chars().take(200).collect::<String>())
        .filter(|name| !name.is_empty());
    let content_type = processed.content_type.to_owned();
    let put_content_type = content_type.clone();
    let sha256 = processed.sha256.clone();
    let size_bytes = processed.bytes.len() as i32;
    let width = processed.width as i32;
    let height = processed.height as i32;
    let kind_db = kind.as_db();
    let owner_type = owner_type.to_owned();
    let staged_id = asset_id.clone();
    let staged_clinic = clinic_id.clone();
    let staged_key = object_key.clone();
    let staged_actor = actor_id.clone();
    let staged_content_type = content_type.clone();
    let staged_sha256 = sha256.clone();
    let staged_pending = auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
            if kind == AssetKind::Image
                && db::count_assets_by_kind(connection, &staged_clinic, "image")? >= 1
            {
                return Err(AuthError::ImageQuotaExceeded);
            }
            db::insert_asset(
                connection,
                &db::NewAsset {
                    id: &staged_id,
                    clinic_id: &staged_clinic,
                    object_key: &staged_key,
                    owner_type: &owner_type,
                    owner_id: &owner_id,
                    kind: kind_db,
                    original_name: original_name.as_deref(),
                    content_type: &staged_content_type,
                    size_bytes,
                    sha256: &staged_sha256,
                    width: Some(width),
                    height: Some(height),
                    status: "pending",
                    deleted_at: None,
                    created_at: &at,
                    updated_at: &at,
                    created_by: Some(&staged_actor),
                },
            )?;
            Ok(())
        })
    })
    .await;
    staged_pending?;
    if let Err(error) = storage
        .put_object(&object_key, &put_content_type, &processed.bytes)
        .await
    {
        let cleanup_id = asset_id.clone();
        let _ = auth_server::with_conn(pool, move |connection| {
            db::hard_delete_asset(connection, &cleanup_id)?;
            Ok(())
        })
        .await;
        let _ = storage.delete_object(&object_key).await;
        return Err(error);
    }
    let ready_id = asset_id.clone();
    let ready_clinic = clinic_id.clone();
    let ready_actor = actor_id.clone();
    let ready = auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
            db::mark_asset_ready(connection, &ready_id, &at)?;
            record_audit(
                connection,
                Some(&ready_actor),
                "asset_upload",
                "asset",
                &ready_id,
                Some(json!({
                    "kind": kind_db,
                    "size_bytes": size_bytes,
                    "content_type": content_type,
                    "width": width,
                    "height": height,
                    "sha256": sha256,
                })),
                &at,
            )?;
            let mut replaced = Vec::new();
            if kind == AssetKind::Logo {
                for previous in db::list_assets_by_kind(connection, &ready_clinic, "logo")? {
                    if previous.id == ready_id {
                        continue;
                    }
                    db::soft_delete_asset(connection, &previous.id, &at)?;
                    record_audit(
                        connection,
                        Some(&ready_actor),
                        "asset_delete",
                        "asset",
                        &previous.id,
                        Some(json!({ "kind": "logo", "replaced": true })),
                        &at,
                    )?;
                    replaced.push(previous.object_key);
                }
                db::set_clinic_logo(connection, &ready_clinic, Some(&ready_id), &at)?;
            }
            let asset = db::find_asset_by_id(connection, &ready_id)?
                .ok_or_else(|| AuthError::NotFound("Imagen no encontrada.".to_owned()))?;
            Ok((asset, replaced))
        })
    })
    .await?;
    for key in ready.1 {
        let _ = storage.delete_object(&key).await;
    }
    Ok(ready.0)
}

pub async fn delete_asset(
    pool: &DbPool,
    storage: &crate::assets::AssetStorage,
    actor: &ActorContext,
    asset_id: String,
) -> Result<(), AuthError> {
    let clinic_id = actor.clinic_id()?.to_owned();
    let is_administrator = actor.role() == Some(Role::Administrator);
    let doctor_id = actor.doctor.as_ref().map(|doctor| doctor.id.clone());
    if !is_administrator && doctor_id.is_none() {
        return Err(AuthError::Forbidden(
            "Solo el administrador y el doctor pueden eliminar imágenes.".to_owned(),
        ));
    }
    let actor_id = actor.profile.id.clone();
    let object_key = auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
            let asset = db::find_asset_by_id(connection, &asset_id)?
                .filter(|asset| asset.clinic_id == clinic_id)
                .ok_or_else(|| AuthError::NotFound("Imagen no encontrada.".to_owned()))?;
            let owns = is_administrator
                || (asset.owner_type == "doctor"
                    && doctor_id.as_deref() == Some(asset.owner_id.as_str()));
            if !owns {
                return Err(AuthError::Forbidden(
                    "No puedes eliminar esta imagen.".to_owned(),
                ));
            }
            if db::count_asset_references(connection, &asset.id)? > 0 {
                return Err(AuthError::AssetInUse);
            }
            if asset.kind == "logo" {
                db::set_clinic_logo(connection, &clinic_id, None, &at)?;
            }
            db::soft_delete_asset(connection, &asset.id, &at)?;
            record_audit(
                connection,
                Some(&actor_id),
                "asset_delete",
                "asset",
                &asset.id,
                Some(json!({ "kind": asset.kind, "clinic_id": asset.clinic_id })),
                &at,
            )?;
            Ok(asset.object_key)
        })
    })
    .await?;
    storage.delete_object(&object_key).await
}

pub async fn list_assets(pool: &DbPool, actor: &ActorContext) -> Result<Vec<db::Asset>, AuthError> {
    actor.require_patient_read()?;
    let clinic_id = actor.clinic_id()?.to_owned();
    auth_server::with_conn(pool, move |connection| {
        db::list_assets_by_clinic(connection, &clinic_id).map_err(AuthError::from)
    })
    .await
}

pub async fn asset_for_read(
    pool: &DbPool,
    actor: &ActorContext,
    asset_id: String,
) -> Result<db::Asset, AuthError> {
    actor.require_patient_read()?;
    let clinic_id = actor.clinic_id()?.to_owned();
    auth_server::with_conn(pool, move |connection| {
        db::find_asset_by_id(connection, &asset_id)?
            .filter(|asset| asset.clinic_id == clinic_id && asset.status == "ready")
            .ok_or_else(|| AuthError::NotFound("Imagen no encontrada.".to_owned()))
    })
    .await
}

fn clinic_name(value: &str) -> Result<String, AuthError> {
    let name = value.trim().to_owned();
    if name.is_empty() {
        return Err(AuthError::BadRequest(
            "Escribe el nombre de la clínica.".to_owned(),
        ));
    }
    if name.chars().count() > 150 {
        return Err(AuthError::BadRequest(
            "El nombre de la clínica es demasiado largo.".to_owned(),
        ));
    }
    Ok(name)
}

fn clinic_llm_url(value: Option<String>) -> Result<Option<String>, AuthError> {
    let value = value
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty());
    if let Some(text) = value.as_deref() {
        if text.chars().count() > 500 {
            return Err(AuthError::BadRequest(
                "La dirección del LLM es demasiado larga.".to_owned(),
            ));
        }
        if !text.starts_with("http://") && !text.starts_with("https://") {
            return Err(AuthError::BadRequest(
                "La dirección del LLM debe iniciar con http:// o https://.".to_owned(),
            ));
        }
    }
    Ok(value)
}

fn clinic_field(value: Option<String>, label: &str) -> Result<Option<String>, AuthError> {
    let value = value
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty());
    if let Some(text) = value.as_deref() {
        if text.chars().count() > 200 {
            return Err(AuthError::BadRequest(format!(
                "El campo {label} es demasiado largo."
            )));
        }
    }
    Ok(value)
}

pub async fn clinic_settings(
    pool: &DbPool,
    actor: &ActorContext,
) -> Result<ClinicSettings, AuthError> {
    actor.require_patient_read()?;
    let clinic_id = actor.clinic_id()?.to_owned();
    auth_server::with_conn(pool, move |connection| {
        let clinic = db::find_clinic_by_id(connection, &clinic_id)?
            .ok_or_else(|| AuthError::NotFound("Clínica no encontrada.".to_owned()))?;
        let logo = match clinic.logo_asset_id.as_deref() {
            Some(id) => db::find_asset_by_id(connection, id)?,
            None => None,
        };
        Ok(ClinicSettings { clinic, logo })
    })
    .await
}

pub async fn update_clinic_settings(
    pool: &DbPool,
    actor: &ActorContext,
    name: String,
    address: Option<String>,
    phone: Option<String>,
    llm_url_template: Option<String>,
) -> Result<ClinicSettings, AuthError> {
    actor.require_administrator()?;
    let name = clinic_name(&name)?;
    let address = clinic_field(address, "dirección")?;
    let phone = clinic_field(phone, "teléfono")?;
    let llm_url_template = clinic_llm_url(llm_url_template)?;
    let clinic_id = actor.clinic_id()?.to_owned();
    let actor_id = actor.profile.id.clone();
    auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
            db::set_clinic_branding(
                connection,
                &clinic_id,
                &name,
                address.as_deref(),
                phone.as_deref(),
                &at,
            )?;
            db::set_clinic_llm_url(connection, &clinic_id, llm_url_template.as_deref(), &at)?;
            record_audit(
                connection,
                Some(&actor_id),
                "update_clinic",
                "clinic",
                &clinic_id,
                None,
                &at,
            )?;
            let clinic = db::find_clinic_by_id(connection, &clinic_id)?
                .ok_or_else(|| AuthError::NotFound("Clínica no encontrada.".to_owned()))?;
            let logo = match clinic.logo_asset_id.as_deref() {
                Some(id) => db::find_asset_by_id(connection, id)?,
                None => None,
            };
            Ok(ClinicSettings { clinic, logo })
        })
    })
    .await
}

pub struct PrescriptionInput {
    pub patient_id: String,
    pub template_version_id: String,
    pub client_id: Option<String>,
    pub diagnosis: Option<String>,
    pub validity: Option<String>,
    pub instructions: Option<String>,
}

pub struct PrescriptionUpdate {
    pub diagnosis: Option<String>,
    pub validity: Option<String>,
    pub instructions: Option<String>,
}

pub struct PrescriptionItemInput {
    pub item_id: Option<String>,
    pub med_name: String,
    pub dose: Option<String>,
    pub route: Option<String>,
    pub frequency: Option<String>,
    pub duration: Option<String>,
    pub quantity: Option<String>,
    pub notes: Option<String>,
}

pub struct PrescriptionSignInput {
    pub acknowledge_warnings: bool,
    pub rx_number: Option<i32>,
    pub rx_block_id: Option<String>,
}

pub struct PrescriptionSupersedeInput {
    pub client_id: Option<String>,
    pub diagnosis: Option<String>,
    pub validity: Option<String>,
    pub instructions: Option<String>,
    pub acknowledge_warnings: bool,
    pub rx_number: Option<i32>,
    pub rx_block_id: Option<String>,
    pub new_prescription_id: Option<String>,
}

pub struct PrescriptionDetail {
    pub prescription: db::Prescription,
    pub items: Vec<db::PrescriptionItem>,
    pub verification: Option<db::VerificationCode>,
    pub patient_name: String,
}

pub struct PrescriptionListRow {
    pub prescription: db::Prescription,
    pub patient_name: String,
}

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

fn clean_rx_text(value: Option<String>, limit: usize) -> Option<String> {
    value
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
        .map(|text| text.chars().take(limit).collect())
}

fn prescription_edit_allowed(role: Option<Role>, prescription: &db::Prescription) -> bool {
    prescription.status == "draft" && matches!(role, Some(Role::Doctor | Role::Nursing))
}

fn prescription_sign_allowed(
    role: Option<Role>,
    doctor_id: Option<&str>,
    prescription: &db::Prescription,
) -> Result<(), AuthError> {
    if role != Some(Role::Doctor) {
        return Err(AuthError::Forbidden(
            "Solo el médico puede firmar recetas.".to_owned(),
        ));
    }
    let doctor_id = doctor_id
        .ok_or_else(|| AuthError::Forbidden("El perfil no tiene ficha de médico.".to_owned()))?;
    if prescription.status != "draft" {
        return Err(AuthError::Conflict(
            "Solo un borrador se puede firmar.".to_owned(),
        ));
    }
    if prescription
        .doctor_id
        .as_deref()
        .is_some_and(|owner| owner != doctor_id)
    {
        return Err(AuthError::Forbidden(
            "Solo el médico autor puede firmar esta receta.".to_owned(),
        ));
    }
    Ok(())
}

fn prescription_transition_allowed(
    role: Option<Role>,
    doctor_id: Option<&str>,
    prescription: &db::Prescription,
) -> Result<(), AuthError> {
    if role != Some(Role::Doctor) {
        return Err(AuthError::Forbidden(
            "Solo el médico autor puede modificar una receta firmada.".to_owned(),
        ));
    }
    let doctor_id = doctor_id
        .ok_or_else(|| AuthError::Forbidden("El perfil no tiene ficha de médico.".to_owned()))?;
    if prescription.status != "signed" {
        return Err(AuthError::Conflict("La receta no está firmada.".to_owned()));
    }
    if prescription.doctor_id.as_deref() != Some(doctor_id) {
        return Err(AuthError::Forbidden(
            "Solo el médico autor puede modificar esta receta.".to_owned(),
        ));
    }
    Ok(())
}

fn template_version_for_prescription(
    connection: &mut SqliteConnection,
    clinic_id: &str,
    template_version_id: &str,
) -> Result<db::TemplateVersion, AuthError> {
    let version = db::find_template_version_by_id(connection, template_version_id)?
        .ok_or_else(|| AuthError::NotFound("Versión de plantilla no encontrada.".to_owned()))?;
    if version.status != "published" {
        return Err(AuthError::BadRequest(
            "La plantilla debe estar publicada.".to_owned(),
        ));
    }
    let template = db::find_template_by_id(connection, &version.template_id)?
        .ok_or_else(|| AuthError::NotFound("Plantilla no encontrada.".to_owned()))?;
    let in_clinic = match template.owner_type.as_str() {
        "clinic" => template.owner_id == clinic_id,
        "doctor" => match db::find_doctor_by_id(connection, &template.owner_id)? {
            Some(doctor) => db::find_membership_by_profile(connection, &doctor.profile_id)?
                .is_some_and(|member| member.clinic_id == clinic_id),
            None => false,
        },
        _ => false,
    };
    if !in_clinic {
        return Err(AuthError::NotFound("Plantilla no encontrada.".to_owned()));
    }
    Ok(version)
}

fn prescription_warnings(
    connection: &mut SqliteConnection,
    prescription: &db::Prescription,
) -> Result<Vec<String>, AuthError> {
    let Some(version_id) = prescription.template_version_id.as_deref() else {
        return Ok(Vec::new());
    };
    let Some(version) = db::find_template_version_by_id(connection, version_id)? else {
        return Ok(Vec::new());
    };
    let document =
        crate::forms::parse_document(&version.doc_json).map_err(AuthError::BadRequest)?;
    let mut warnings = crate::forms::document_warnings(&document);
    warnings.sort();
    warnings.dedup();
    Ok(warnings)
}

fn new_verification_code() -> (String, String) {
    let raw = Uuid::new_v4().simple().to_string();
    let code = raw[..16].to_uppercase();
    let printable = format!(
        "{}-{}-{}-{}",
        &code[..4],
        &code[4..8],
        &code[8..12],
        &code[12..16]
    );
    let hash = auth_server::hash_token(&printable);
    (printable, hash)
}

fn allocate_block_locked(
    connection: &mut SqliteConnection,
    actor_id: &str,
    clinic_id: &str,
    device_id: &str,
    block_size: i32,
    at: &str,
) -> Result<db::RxBlock, AuthError> {
    let size = block_size.max(1);
    let last = db::find_rx_counter(connection, clinic_id)?
        .map(|counter| counter.last_rx_number)
        .unwrap_or(0);
    let start_number = last + 1;
    let end_number = last + size;
    if db::find_rx_counter(connection, clinic_id)?.is_some() {
        db::set_rx_counter(connection, clinic_id, end_number)?;
    } else {
        db::insert_rx_counter(connection, clinic_id, end_number)?;
    }
    let id = Uuid::now_v7().to_string();
    db::insert_rx_block(
        connection,
        &db::NewRxBlock {
            id: &id,
            clinic_id,
            device_id,
            start_number,
            end_number,
            consumed: 0,
            status: "active",
            allocated_at: at,
            created_at: at,
            updated_at: at,
            created_by: Some(actor_id),
        },
    )?;
    db::find_rx_block_by_id(connection, &id)?
        .ok_or_else(|| AuthError::NotFound("Bloque no encontrado.".to_owned()))
}

fn active_or_new_block(
    connection: &mut SqliteConnection,
    actor_id: &str,
    clinic_id: &str,
    device_id: &str,
    block_size: i32,
    at: &str,
) -> Result<db::RxBlock, AuthError> {
    if let Some(block) = db::find_active_rx_block(connection, device_id)? {
        let capacity = block.end_number - block.start_number + 1;
        if block.consumed < capacity {
            return Ok(block);
        }
        db::set_rx_block_status(connection, &block.id, "exhausted", at)?;
    }
    allocate_block_locked(connection, actor_id, clinic_id, device_id, block_size, at)
}

fn consume_block_number(
    connection: &mut SqliteConnection,
    actor_id: &str,
    clinic_id: &str,
    device_id: &str,
    block_size: i32,
    at: &str,
) -> Result<(db::RxBlock, i32), AuthError> {
    let block = active_or_new_block(connection, actor_id, clinic_id, device_id, block_size, at)?;
    let number = block.start_number + block.consumed;
    let consumed = block.consumed + 1;
    db::set_rx_block_consumed(connection, &block.id, consumed, at)?;
    if consumed > block.end_number - block.start_number {
        db::set_rx_block_status(connection, &block.id, "exhausted", at)?;
    }
    db::find_rx_block_by_id(connection, &block.id)?
        .map(|block| (block, number))
        .ok_or_else(|| AuthError::NotFound("Bloque no encontrado.".to_owned()))
}

fn reconcile_block_number(
    connection: &mut SqliteConnection,
    clinic_id: &str,
    device_id: &str,
    block_id: &str,
    number: i32,
    at: &str,
) -> Result<db::RxBlock, AuthError> {
    let block = db::find_rx_block_by_id(connection, block_id)?
        .ok_or_else(|| AuthError::NotFound("Bloque no encontrado.".to_owned()))?;
    if block.device_id != device_id || block.clinic_id != clinic_id {
        return Err(AuthError::BadRequest(
            "El bloque de números no pertenece a este dispositivo.".to_owned(),
        ));
    }
    if number < block.start_number || number > block.end_number {
        return Err(AuthError::Conflict(
            "El número está fuera del bloque asignado.".to_owned(),
        ));
    }
    if db::find_prescription_by_rx_number(connection, number)?.is_some() {
        return Err(AuthError::Conflict(
            "El número ya está asignado a otra receta.".to_owned(),
        ));
    }
    let consumed = (number - block.start_number + 1).max(block.consumed);
    db::set_rx_block_consumed(connection, &block.id, consumed, at)?;
    if consumed > block.end_number - block.start_number {
        db::set_rx_block_status(connection, &block.id, "exhausted", at)?;
    }
    db::find_rx_block_by_id(connection, &block.id)?
        .ok_or_else(|| AuthError::NotFound("Bloque no encontrado.".to_owned()))
}

pub async fn allocate_rx_block(
    pool: &DbPool,
    actor: &ActorContext,
) -> Result<db::RxBlock, AuthError> {
    let clinic_id = actor.clinic_id()?.to_owned();
    let device_id = actor.device_id.clone().ok_or_else(|| {
        AuthError::BadRequest("Registra este dispositivo antes de reservar números.".to_owned())
    })?;
    let block_size = actor
        .clinic
        .as_ref()
        .map(|clinic| clinic.rx_block_size)
        .unwrap_or(50);
    let actor_id = actor.profile.id.clone();
    auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
            if db::find_active_rx_block(connection, &device_id)?.is_some() {
                return Err(AuthError::Conflict(
                    "Este dispositivo ya tiene un bloque activo.".to_owned(),
                ));
            }
            let block = allocate_block_locked(
                connection, &actor_id, &clinic_id, &device_id, block_size, &at,
            )?;
            record_audit(
                connection,
                Some(&actor_id),
                "allocate_rx_block",
                "rx_block",
                &block.id,
                Some(json!({
                    "start_number": block.start_number,
                    "end_number": block.end_number,
                })),
                &at,
            )?;
            Ok(block)
        })
    })
    .await
}

pub async fn rx_block_status(
    pool: &DbPool,
    actor: &ActorContext,
) -> Result<RxBlockStatus, AuthError> {
    let limit = actor
        .clinic
        .as_ref()
        .map(|clinic| clinic.rx_block_size)
        .unwrap_or(50);
    let Some(device_id) = actor.device_id.clone() else {
        return Ok(RxBlockStatus {
            active: false,
            block_id: None,
            start_number: None,
            end_number: None,
            consumed: None,
            remaining: 0,
            limit,
            exhausted: false,
            device_id: None,
        });
    };
    auth_server::with_conn(pool, move |connection| {
        let block = db::find_active_rx_block(connection, &device_id)?;
        Ok(match block {
            Some(block) => {
                let capacity = block.end_number - block.start_number + 1;
                let remaining = (capacity - block.consumed).max(0);
                RxBlockStatus {
                    active: true,
                    block_id: Some(block.id),
                    start_number: Some(block.start_number),
                    end_number: Some(block.end_number),
                    consumed: Some(block.consumed),
                    remaining,
                    limit,
                    exhausted: remaining == 0,
                    device_id: Some(device_id),
                }
            }
            None => RxBlockStatus {
                active: false,
                block_id: None,
                start_number: None,
                end_number: None,
                consumed: None,
                remaining: 0,
                limit,
                exhausted: false,
                device_id: Some(device_id),
            },
        })
    })
    .await
}

pub async fn create_prescription(
    pool: &DbPool,
    actor: &ActorContext,
    input: PrescriptionInput,
) -> Result<db::Prescription, AuthError> {
    let role = actor
        .role()
        .ok_or_else(|| AuthError::Forbidden("Se necesita un rol en la clínica.".to_owned()))?;
    if !matches!(role, Role::Doctor | Role::Nursing) {
        return Err(AuthError::Forbidden(
            "Solo doctor o enfermería pueden crear recetas.".to_owned(),
        ));
    }
    let clinic_id = actor.clinic_id()?.to_owned();
    let patient_id = input.patient_id.trim().to_owned();
    if patient_id.is_empty() {
        return Err(AuthError::BadRequest("Selecciona un paciente.".to_owned()));
    }
    let template_version_id = input.template_version_id.trim().to_owned();
    if template_version_id.is_empty() {
        return Err(AuthError::BadRequest(
            "Selecciona una plantilla.".to_owned(),
        ));
    }
    let client_id = input
        .client_id
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| Uuid::now_v7().to_string());
    let diagnosis = clean_rx_text(input.diagnosis, 2000);
    let validity = clean_rx_text(input.validity, 200);
    let instructions = clean_rx_text(input.instructions, 8000);
    let doctor_id = actor.doctor.as_ref().map(|doctor| doctor.id.clone());
    let actor_id = actor.profile.id.clone();
    auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
            if let Some(existing) =
                db::find_prescription_by_client_id(connection, &clinic_id, &client_id)?
            {
                return Ok(existing);
            }
            let patient = db::find_patient_by_id(connection, &clinic_id, &patient_id)?
                .ok_or_else(|| AuthError::NotFound("Paciente no encontrado.".to_owned()))?;
            let version =
                template_version_for_prescription(connection, &clinic_id, &template_version_id)?;
            let id = Uuid::now_v7().to_string();
            db::insert_prescription(
                connection,
                &db::NewPrescription {
                    id: &id,
                    patient_id: &patient.id,
                    doctor_id: doctor_id.as_deref(),
                    clinic_id: &clinic_id,
                    template_version_id: Some(&version.id),
                    status: "draft",
                    rx_number: None,
                    rx_block_id: None,
                    diagnosis: diagnosis.as_deref(),
                    validity: validity.as_deref(),
                    instructions: instructions.as_deref(),
                    signed_at: None,
                    synced_at: None,
                    voided_at: None,
                    void_reason: None,
                    client_id: &client_id,
                    supersedes_id: None,
                    created_at: &at,
                    updated_at: &at,
                    created_by: Some(&actor_id),
                },
            )?;
            record_audit(
                connection,
                Some(&actor_id),
                "create_prescription",
                "prescription",
                &id,
                Some(json!({ "patient_id": patient.id, "template_version_id": version.id })),
                &at,
            )?;
            db::find_prescription_by_id(connection, &clinic_id, &id)?
                .ok_or_else(|| AuthError::NotFound("Receta no encontrada.".to_owned()))
        })
    })
    .await
}

pub async fn update_prescription(
    pool: &DbPool,
    actor: &ActorContext,
    prescription_id: String,
    input: PrescriptionUpdate,
) -> Result<db::Prescription, AuthError> {
    let clinic_id = actor.clinic_id()?.to_owned();
    let actor_id = actor.profile.id.clone();
    let role = actor.role();
    let diagnosis = clean_rx_text(input.diagnosis, 2000);
    let validity = clean_rx_text(input.validity, 200);
    let instructions = clean_rx_text(input.instructions, 8000);
    auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
            let prescription =
                db::find_prescription_by_id(connection, &clinic_id, &prescription_id)?
                    .ok_or_else(|| AuthError::NotFound("Receta no encontrada.".to_owned()))?;
            if !prescription_edit_allowed(role, &prescription) {
                return Err(AuthError::Forbidden(
                    "La receta no se puede editar.".to_owned(),
                ));
            }
            db::update_prescription_draft(
                connection,
                &clinic_id,
                &prescription.id,
                diagnosis.as_deref(),
                validity.as_deref(),
                instructions.as_deref(),
                &at,
            )?;
            record_audit(
                connection,
                Some(&actor_id),
                "update_prescription",
                "prescription",
                &prescription.id,
                None,
                &at,
            )?;
            db::find_prescription_by_id(connection, &clinic_id, &prescription.id)?
                .ok_or_else(|| AuthError::NotFound("Receta no encontrada.".to_owned()))
        })
    })
    .await
}

pub async fn upsert_prescription_item(
    pool: &DbPool,
    actor: &ActorContext,
    prescription_id: String,
    input: PrescriptionItemInput,
) -> Result<db::PrescriptionItem, AuthError> {
    let clinic_id = actor.clinic_id()?.to_owned();
    let actor_id = actor.profile.id.clone();
    let role = actor.role();
    let med_name = input.med_name.trim().to_owned();
    if med_name.is_empty() {
        return Err(AuthError::BadRequest(
            "Escribe el nombre del medicamento.".to_owned(),
        ));
    }
    if med_name.chars().count() > 500 {
        return Err(AuthError::BadRequest(
            "El nombre del medicamento es demasiado largo.".to_owned(),
        ));
    }
    let dose = clean_rx_text(input.dose, 500);
    let route = clean_rx_text(input.route, 500);
    let frequency = clean_rx_text(input.frequency, 500);
    let duration = clean_rx_text(input.duration, 500);
    let quantity = clean_rx_text(input.quantity, 500);
    let notes = clean_rx_text(input.notes, 1000);
    let item_id = input
        .item_id
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());
    auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
            let prescription =
                db::find_prescription_by_id(connection, &clinic_id, &prescription_id)?
                    .ok_or_else(|| AuthError::NotFound("Receta no encontrada.".to_owned()))?;
            if !prescription_edit_allowed(role, &prescription) {
                return Err(AuthError::Forbidden(
                    "La receta no se puede editar.".to_owned(),
                ));
            }
            let item = match item_id.clone() {
                Some(item_id) => {
                    if let Some(existing) =
                        db::find_prescription_item(connection, &prescription.id, &item_id)?
                    {
                        db::update_prescription_item(
                            connection,
                            &prescription.id,
                            &existing.id,
                            &db::PrescriptionItemFields {
                                position: existing.position,
                                med_name: &med_name,
                                dose: dose.as_deref(),
                                route: route.as_deref(),
                                frequency: frequency.as_deref(),
                                duration: duration.as_deref(),
                                quantity: quantity.as_deref(),
                                notes: notes.as_deref(),
                            },
                            &at,
                        )?;
                        existing.id
                    } else {
                        let items = db::list_prescription_items(connection, &prescription.id)?;
                        let position =
                            items.iter().map(|item| item.position).max().unwrap_or(0) + 1;
                        db::insert_prescription_item(
                            connection,
                            &db::NewPrescriptionItem {
                                id: &item_id,
                                prescription_id: &prescription.id,
                                position,
                                med_name: &med_name,
                                dose: dose.as_deref(),
                                route: route.as_deref(),
                                frequency: frequency.as_deref(),
                                duration: duration.as_deref(),
                                quantity: quantity.as_deref(),
                                notes: notes.as_deref(),
                                created_at: &at,
                                updated_at: &at,
                                created_by: Some(&actor_id),
                            },
                        )?;
                        item_id
                    }
                }
                None => {
                    let items = db::list_prescription_items(connection, &prescription.id)?;
                    let position = items.iter().map(|item| item.position).max().unwrap_or(0) + 1;
                    let id = Uuid::now_v7().to_string();
                    db::insert_prescription_item(
                        connection,
                        &db::NewPrescriptionItem {
                            id: &id,
                            prescription_id: &prescription.id,
                            position,
                            med_name: &med_name,
                            dose: dose.as_deref(),
                            route: route.as_deref(),
                            frequency: frequency.as_deref(),
                            duration: duration.as_deref(),
                            quantity: quantity.as_deref(),
                            notes: notes.as_deref(),
                            created_at: &at,
                            updated_at: &at,
                            created_by: Some(&actor_id),
                        },
                    )?;
                    id
                }
            };
            record_audit(
                connection,
                Some(&actor_id),
                "upsert_prescription_item",
                "prescription_item",
                &item,
                Some(json!({ "prescription_id": prescription.id })),
                &at,
            )?;
            db::find_prescription_item(connection, &prescription.id, &item)?
                .ok_or_else(|| AuthError::NotFound("Medicamento no encontrado.".to_owned()))
        })
    })
    .await
}

pub async fn delete_prescription_item(
    pool: &DbPool,
    actor: &ActorContext,
    prescription_id: String,
    item_id: String,
) -> Result<(), AuthError> {
    let clinic_id = actor.clinic_id()?.to_owned();
    let actor_id = actor.profile.id.clone();
    let role = actor.role();
    auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
            let prescription =
                db::find_prescription_by_id(connection, &clinic_id, &prescription_id)?
                    .ok_or_else(|| AuthError::NotFound("Receta no encontrada.".to_owned()))?;
            if !prescription_edit_allowed(role, &prescription) {
                return Err(AuthError::Forbidden(
                    "La receta no se puede editar.".to_owned(),
                ));
            }
            db::delete_prescription_item(connection, &prescription.id, &item_id)?;
            record_audit(
                connection,
                Some(&actor_id),
                "delete_prescription_item",
                "prescription_item",
                &item_id,
                Some(json!({ "prescription_id": prescription.id })),
                &at,
            )
        })
    })
    .await
}

pub async fn reorder_prescription_items(
    pool: &DbPool,
    actor: &ActorContext,
    prescription_id: String,
    ordered_ids: Vec<String>,
) -> Result<Vec<db::PrescriptionItem>, AuthError> {
    let clinic_id = actor.clinic_id()?.to_owned();
    let actor_id = actor.profile.id.clone();
    let role = actor.role();
    auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
            let prescription =
                db::find_prescription_by_id(connection, &clinic_id, &prescription_id)?
                    .ok_or_else(|| AuthError::NotFound("Receta no encontrada.".to_owned()))?;
            if !prescription_edit_allowed(role, &prescription) {
                return Err(AuthError::Forbidden(
                    "La receta no se puede editar.".to_owned(),
                ));
            }
            let items = db::list_prescription_items(connection, &prescription.id)?;
            if items.len() != ordered_ids.len() {
                return Err(AuthError::BadRequest(
                    "El orden debe incluir todos los medicamentos.".to_owned(),
                ));
            }
            for item in &items {
                db::update_prescription_item_position(
                    connection,
                    &prescription.id,
                    &item.id,
                    -(item.position + 1),
                    &at,
                )?;
            }
            for (index, id) in ordered_ids.iter().enumerate() {
                if !items.iter().any(|item| &item.id == id) {
                    return Err(AuthError::BadRequest(
                        "El orden incluye un medicamento desconocido.".to_owned(),
                    ));
                }
                db::update_prescription_item_position(
                    connection,
                    &prescription.id,
                    id,
                    index as i32 + 1,
                    &at,
                )?;
            }
            record_audit(
                connection,
                Some(&actor_id),
                "reorder_prescription_items",
                "prescription",
                &prescription.id,
                Some(json!({ "order": ordered_ids })),
                &at,
            )?;
            Ok(db::list_prescription_items(connection, &prescription.id)?)
        })
    })
    .await
}

pub async fn get_prescription(
    pool: &DbPool,
    actor: &ActorContext,
    prescription_id: String,
) -> Result<PrescriptionDetail, AuthError> {
    actor
        .role()
        .ok_or_else(|| AuthError::Forbidden("Se necesita un rol en la clínica.".to_owned()))?;
    let clinic_id = actor.clinic_id()?.to_owned();
    let actor_id = actor.profile.id.clone();
    auth_server::with_conn(pool, move |connection| {
        let prescription =
            db::find_prescription_by_id(connection, &clinic_id, &prescription_id)?
                .ok_or_else(|| AuthError::NotFound("Receta no encontrada.".to_owned()))?;
        let items = db::list_prescription_items(connection, &prescription.id)?;
        let verification =
            db::find_verification_code_by_prescription(connection, &prescription.id)?;
        let patient_name = db::find_patient_any(connection, &clinic_id, &prescription.patient_id)?
            .map(|patient| patient.full_name)
            .unwrap_or_else(|| "Paciente eliminado".to_owned());
        record_audit(
            connection,
            Some(&actor_id),
            "sensitive_read",
            "prescription",
            &prescription.id,
            Some(json!({ "scope": "detail" })),
            &auth_server::now(),
        )?;
        Ok(PrescriptionDetail {
            prescription,
            items,
            verification,
            patient_name,
        })
    })
    .await
}

#[derive(Debug, Clone)]
pub struct PrescriptionDocument {
    pub prescription_id: String,
    pub clinic_id: String,
    pub status: String,
    pub rx_number: Option<i32>,
    pub verification_code: Option<String>,
    pub doc_json: String,
    pub margin_mm: u32,
    pub asset_ids: Vec<String>,
    pub items: Vec<db::PrescriptionItem>,
    pub patient_name: String,
    pub patient_national_id: Option<String>,
    pub patient_birth_date: Option<String>,
    pub doctor_name: String,
    pub doctor_specialty: Option<String>,
    pub doctor_national_id: Option<String>,
    pub doctor_phone: Option<String>,
    pub doctor_email: Option<String>,
    pub clinic_name: String,
    pub clinic_address: Option<String>,
    pub clinic_phone: Option<String>,
    pub diagnosis: Option<String>,
    pub issued_at: String,
}

impl PrescriptionDocument {
    pub fn file_date(&self) -> String {
        display_date(&self.issued_at)
    }

    pub fn issued_display(&self) -> String {
        display_datetime(&self.issued_at)
    }

    pub fn patient_age(&self) -> Option<i32> {
        let birth = self
            .patient_birth_date
            .as_deref()
            .and_then(|value| NaiveDate::parse_from_str(value, "%Y-%m-%d").ok())?;
        let issued = NaiveDate::parse_from_str(&self.file_date(), "%d-%m-%Y").ok()?;
        let mut age = issued.year() - birth.year();
        if (issued.month(), issued.day()) < (birth.month(), birth.day()) {
            age -= 1;
        }
        (age >= 0).then_some(age)
    }

    pub fn variables(&self) -> Vec<(String, String)> {
        let mut values = vec![
            ("patient.name".to_owned(), self.patient_name.clone()),
            (
                "patient.national_id".to_owned(),
                self.patient_national_id
                    .clone()
                    .unwrap_or_else(|| "—".to_owned()),
            ),
            (
                "patient.age".to_owned(),
                self.patient_age()
                    .map(|age| format!("{age} años"))
                    .unwrap_or_else(|| "—".to_owned()),
            ),
            ("doctor.name".to_owned(), self.doctor_name.clone()),
            (
                "doctor.specialty".to_owned(),
                self.doctor_specialty
                    .clone()
                    .unwrap_or_else(|| "—".to_owned()),
            ),
            (
                "doctor.national_id".to_owned(),
                self.doctor_national_id
                    .clone()
                    .unwrap_or_else(|| "—".to_owned()),
            ),
            (
                "doctor.contact".to_owned(),
                [self.doctor_phone.clone(), self.doctor_email.clone()]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<String>>()
                    .join(" · "),
            ),
            ("clinic.name".to_owned(), self.clinic_name.clone()),
            (
                "clinic.address".to_owned(),
                self.clinic_address
                    .clone()
                    .unwrap_or_else(|| "—".to_owned()),
            ),
            (
                "clinic.phone".to_owned(),
                self.clinic_phone.clone().unwrap_or_else(|| "—".to_owned()),
            ),
            (
                "prescription.number".to_owned(),
                self.rx_number
                    .map(|number| format!("{number:06}"))
                    .unwrap_or_else(|| "PENDIENTE".to_owned()),
            ),
            (
                "prescription.date".to_owned(),
                display_date(&self.issued_at),
            ),
            (
                "prescription.diagnosis".to_owned(),
                self.diagnosis.clone().unwrap_or_else(|| "—".to_owned()),
            ),
            (
                "medications[]".to_owned(),
                self.items
                    .iter()
                    .map(|item| item.med_name.clone())
                    .collect::<Vec<String>>()
                    .join(", "),
            ),
        ];
        if values.last().is_some_and(|(_, value)| value.is_empty()) {
            if let Some(last) = values.last_mut() {
                last.1 = "—".to_owned();
            }
        }
        values
    }

    pub fn medications(&self) -> Vec<crate::forms::MedicationBlock> {
        self.items
            .iter()
            .map(|item| crate::forms::MedicationBlock {
                name: item.med_name.clone(),
                dose: item.dose.clone().unwrap_or_default(),
                route: item.route.clone().unwrap_or_default(),
                frequency: item.frequency.clone().unwrap_or_default(),
                duration: item.duration.clone().unwrap_or_default(),
                quantity: item.quantity.clone().unwrap_or_default(),
                notes: item.notes.clone().unwrap_or_default(),
            })
            .collect()
    }
}

fn ecuador_offset() -> FixedOffset {
    FixedOffset::west_opt(5 * 3600).unwrap_or_else(|| FixedOffset::east_opt(0).unwrap())
}

fn display_date(value: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|moment| {
            moment
                .with_timezone(&ecuador_offset())
                .date_naive()
                .format("%d-%m-%Y")
                .to_string()
        })
        .unwrap_or_else(|_| value.to_owned())
}

fn display_datetime(value: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|moment| {
            moment
                .with_timezone(&ecuador_offset())
                .format("%d-%m-%Y %H:%M")
                .to_string()
        })
        .unwrap_or_else(|_| value.to_owned())
}

pub struct VerificationResult {
    pub status: String,
    pub doctor_name: Option<String>,
    pub issued_at: Option<String>,
    pub rx_number: Option<i32>,
}

fn canonical_verification_code(value: &str) -> String {
    let compact: String = value
        .trim()
        .chars()
        .filter(|character| !character.is_whitespace() && *character != '-')
        .collect::<String>()
        .to_uppercase();
    if compact.len() == 16
        && compact
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
    {
        format!(
            "{}-{}-{}-{}",
            &compact[..4],
            &compact[4..8],
            &compact[8..12],
            &compact[12..16]
        )
    } else {
        compact
    }
}

pub async fn verify_code(pool: &DbPool, code: &str) -> Result<VerificationResult, AuthError> {
    let canonical = canonical_verification_code(code);
    let code_hash = auth_server::hash_token(&canonical);
    auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        let found = db::find_verification_code_by_hash(connection, &code_hash)?;
        let Some(verification) = found else {
            record_audit(
                connection,
                None,
                "verification_attempt",
                "verification",
                "not_found",
                Some(json!({ "status": "not found" })),
                &at,
            )?;
            return Ok(VerificationResult {
                status: "not found".to_owned(),
                doctor_name: None,
                issued_at: None,
                rx_number: None,
            });
        };
        let prescription = db::find_prescription_any(connection, &verification.prescription_id)?;
        let (doctor_name, issued_at, rx_number) = match prescription.as_ref() {
            Some(prescription) => {
                let doctor_name = prescription
                    .doctor_id
                    .as_deref()
                    .and_then(|doctor_id| {
                        db::find_doctor_by_id(connection, doctor_id).ok().flatten()
                    })
                    .and_then(|doctor| {
                        db::find_profile_by_id(connection, &doctor.profile_id)
                            .ok()
                            .flatten()
                    })
                    .map(|profile| profile.display_name)
                    .or_else(|| {
                        prescription
                            .created_by
                            .as_deref()
                            .and_then(|profile_id| {
                                db::find_profile_by_id(connection, profile_id)
                                    .ok()
                                    .flatten()
                            })
                            .map(|profile| profile.display_name)
                    });
                let issued_at = prescription
                    .signed_at
                    .clone()
                    .unwrap_or_else(|| prescription.created_at.clone());
                (doctor_name, Some(issued_at), prescription.rx_number)
            }
            None => (None, None, None),
        };
        record_audit(
            connection,
            None,
            "verification_attempt",
            "verification",
            &verification.id,
            Some(json!({ "status": verification.status })),
            &at,
        )?;
        Ok(VerificationResult {
            status: verification.status.clone(),
            doctor_name,
            issued_at,
            rx_number,
        })
    })
    .await
}

pub async fn prescription_document(
    pool: &DbPool,
    actor: &ActorContext,
    prescription_id: String,
    allow_draft: bool,
) -> Result<PrescriptionDocument, AuthError> {
    actor
        .role()
        .ok_or_else(|| AuthError::Forbidden("Se necesita un rol en la clínica.".to_owned()))?;
    let clinic_id = actor.clinic_id()?.to_owned();
    let actor_id = actor.profile.id.clone();
    let actor_name = actor.profile.display_name.clone();
    auth_server::with_conn(pool, move |connection| {
        let prescription =
            db::find_prescription_by_id(connection, &clinic_id, &prescription_id)?
                .ok_or_else(|| AuthError::NotFound("Receta no encontrada.".to_owned()))?;
        if prescription.status == "draft" && !allow_draft {
            return Err(AuthError::Unprocessable(
                "La receta todavía no está firmada.".to_owned(),
            ));
        }
        let version_id = prescription.template_version_id.clone().ok_or_else(|| {
            AuthError::Unprocessable("La receta no tiene una versión fijada.".to_owned())
        })?;
        let version = db::find_template_version_by_id(connection, &version_id)?
            .ok_or_else(|| AuthError::NotFound("Versión de plantilla no encontrada.".to_owned()))?;
        let margin_mm = crate::forms::template_theme_margin(version.theme.as_deref());
        let asset_ids = db::list_template_version_asset_ids(connection, &version.id)?;
        let patient = db::find_patient_any(connection, &clinic_id, &prescription.patient_id)?
            .ok_or_else(|| AuthError::NotFound("Paciente no encontrado.".to_owned()))?;
        let doctor = match prescription.doctor_id.as_deref() {
            Some(doctor_id) => db::find_doctor_by_id(connection, doctor_id)?,
            None => None,
        };
        let doctor_profile = match doctor.as_ref() {
            Some(doctor) => db::find_profile_by_id(connection, &doctor.profile_id)?,
            None => None,
        };
        let clinic = db::find_clinic_by_id(connection, &clinic_id)?
            .ok_or_else(|| AuthError::NotFound("Clínica no encontrada.".to_owned()))?;
        let verification =
            db::find_verification_code_by_prescription(connection, &prescription.id)?;
        let items = db::list_prescription_items(connection, &prescription.id)?;
        let issued_at = prescription
            .signed_at
            .clone()
            .unwrap_or_else(|| prescription.created_at.clone());
        record_audit(
            connection,
            Some(&actor_id),
            "sensitive_read",
            "prescription",
            &prescription.id,
            Some(json!({ "scope": "pdf" })),
            &auth_server::now(),
        )?;
        let doctor_name = doctor_profile
            .map(|profile| profile.display_name)
            .unwrap_or(actor_name);
        Ok(PrescriptionDocument {
            prescription_id: prescription.id,
            clinic_id,
            status: prescription.status,
            rx_number: prescription.rx_number,
            verification_code: verification.map(|code| code.code),
            doc_json: version.doc_json,
            margin_mm,
            asset_ids,
            items,
            patient_name: patient.full_name,
            patient_national_id: patient.national_id,
            patient_birth_date: patient.birth_date,
            doctor_name,
            doctor_specialty: doctor.as_ref().and_then(|doctor| doctor.specialty.clone()),
            doctor_national_id: doctor
                .as_ref()
                .and_then(|doctor| doctor.national_id.clone()),
            doctor_phone: doctor.as_ref().and_then(|doctor| doctor.phone.clone()),
            doctor_email: doctor.as_ref().and_then(|doctor| doctor.email.clone()),
            clinic_name: clinic.name,
            clinic_address: clinic.address,
            clinic_phone: clinic.phone,
            diagnosis: prescription.diagnosis,
            issued_at,
        })
    })
    .await
}

pub async fn list_prescriptions(
    pool: &DbPool,
    actor: &ActorContext,
    patient_id: Option<String>,
    status: Option<String>,
    date: Option<String>,
) -> Result<Vec<PrescriptionListRow>, AuthError> {
    actor
        .role()
        .ok_or_else(|| AuthError::Forbidden("Se necesita un rol en la clínica.".to_owned()))?;
    let clinic_id = actor.clinic_id()?.to_owned();
    let actor_id = actor.profile.id.clone();
    let patient_id = patient_id
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());
    let status = status
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());
    let date = date
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());
    auth_server::with_conn(pool, move |connection| {
        let rows = db::list_prescriptions(
            connection,
            &clinic_id,
            None,
            patient_id.as_deref(),
            status.as_deref(),
            date.as_deref(),
            200,
        )?;
        let mut result = Vec::with_capacity(rows.len());
        for prescription in rows {
            let patient_name =
                db::find_patient_any(connection, &clinic_id, &prescription.patient_id)?
                    .map(|patient| patient.full_name)
                    .unwrap_or_else(|| "Paciente eliminado".to_owned());
            result.push(PrescriptionListRow {
                prescription,
                patient_name,
            });
        }
        record_audit(
            connection,
            Some(&actor_id),
            "sensitive_read",
            "prescription",
            "list",
            Some(json!({
                "patient_id": patient_id,
                "status": status,
                "date": date,
            })),
            &auth_server::now(),
        )?;
        Ok(result)
    })
    .await
}

pub async fn sign_prescription(
    pool: &DbPool,
    actor: &ActorContext,
    prescription_id: String,
    input: PrescriptionSignInput,
) -> Result<PrescriptionDetail, AuthError> {
    let clinic_id = actor.clinic_id()?.to_owned();
    let actor_id = actor.profile.id.clone();
    let role = actor.role();
    if role != Some(Role::Doctor) {
        return Err(AuthError::Forbidden(
            "Solo el médico puede firmar recetas.".to_owned(),
        ));
    }
    let doctor_id = actor.doctor.as_ref().map(|doctor| doctor.id.clone());
    let block_size = actor
        .clinic
        .as_ref()
        .map(|clinic| clinic.rx_block_size)
        .unwrap_or(50);
    let device_id = actor.device_id.clone().ok_or_else(|| {
        AuthError::BadRequest("Registra este dispositivo antes de firmar.".to_owned())
    })?;
    auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
            let prescription =
                db::find_prescription_by_id(connection, &clinic_id, &prescription_id)?
                    .ok_or_else(|| AuthError::NotFound("Receta no encontrada.".to_owned()))?;
            let doctor_id = doctor_id.as_deref().ok_or_else(|| {
                AuthError::Forbidden("El perfil no tiene ficha de médico.".to_owned())
            })?;
            if prescription.status != "draft" {
                if let Some(number) = input.rx_number {
                    if prescription.rx_number != Some(number) {
                        return Err(AuthError::Conflict(
                            "El número ya reconciliado no coincide.".to_owned(),
                        ));
                    }
                }
            }
            if prescription.status != "draft" {
                if prescription.doctor_id.as_deref() != Some(doctor_id) {
                    return Err(AuthError::Forbidden(
                        "Solo el médico autor puede firmar esta receta.".to_owned(),
                    ));
                }
                let items = db::list_prescription_items(connection, &prescription.id)?;
                let verification =
                    db::find_verification_code_by_prescription(connection, &prescription.id)?;
                let patient_name =
                    db::find_patient_any(connection, &clinic_id, &prescription.patient_id)?
                        .map(|patient| patient.full_name)
                        .unwrap_or_else(|| "Paciente eliminado".to_owned());
                return Ok(PrescriptionDetail {
                    prescription,
                    items,
                    verification,
                    patient_name,
                });
            }
            prescription_sign_allowed(role, Some(doctor_id), &prescription)?;
            let warnings = prescription_warnings(connection, &prescription)?;
            if !warnings.is_empty() && !input.acknowledge_warnings {
                return Err(AuthError::UnacknowledgedWarnings(warnings));
            }
            let (block, number) = match (input.rx_number, input.rx_block_id.as_deref()) {
                (Some(number), Some(block_id)) => (
                    reconcile_block_number(
                        connection, &clinic_id, &device_id, block_id, number, &at,
                    )?,
                    number,
                ),
                (Some(_), None) => {
                    return Err(AuthError::BadRequest(
                        "Falta el bloque de números.".to_owned(),
                    ));
                }
                _ => consume_block_number(
                    connection, &actor_id, &clinic_id, &device_id, block_size, &at,
                )?,
            };
            db::mark_prescription_signed(
                connection,
                &clinic_id,
                &prescription.id,
                doctor_id,
                number,
                &block.id,
                &at,
            )?;
            let (code, code_hash) = new_verification_code();
            let code_id = Uuid::now_v7().to_string();
            db::insert_verification_code(
                connection,
                &db::NewVerificationCode {
                    id: &code_id,
                    prescription_id: &prescription.id,
                    code: &code,
                    code_hash: &code_hash,
                    status: "valid",
                    created_at: &at,
                    updated_at: &at,
                    created_by: Some(&actor_id),
                },
            )?;
            record_audit(
                connection,
                Some(&actor_id),
                "sign_prescription",
                "prescription",
                &prescription.id,
                Some(json!({
                    "rx_number": number,
                    "rx_block_id": block.id,
                    "warnings": warnings,
                })),
                &at,
            )?;
            let signed = db::find_prescription_by_id(connection, &clinic_id, &prescription.id)?
                .ok_or_else(|| AuthError::NotFound("Receta no encontrada.".to_owned()))?;
            let items = db::list_prescription_items(connection, &signed.id)?;
            let verification = db::find_verification_code_by_prescription(connection, &signed.id)?;
            let patient_name = db::find_patient_any(connection, &clinic_id, &signed.patient_id)?
                .map(|patient| patient.full_name)
                .unwrap_or_else(|| "Paciente eliminado".to_owned());
            Ok(PrescriptionDetail {
                prescription: signed,
                items,
                verification,
                patient_name,
            })
        })
    })
    .await
}

pub async fn void_prescription(
    pool: &DbPool,
    actor: &ActorContext,
    prescription_id: String,
    reason: String,
) -> Result<PrescriptionDetail, AuthError> {
    let clinic_id = actor.clinic_id()?.to_owned();
    let actor_id = actor.profile.id.clone();
    let role = actor.role();
    let doctor_id = actor.doctor.as_ref().map(|doctor| doctor.id.clone());
    let reason = reason.trim().to_owned();
    if reason.is_empty() {
        return Err(AuthError::BadRequest(
            "Escribe el motivo de anulación.".to_owned(),
        ));
    }
    if reason.chars().count() > 1000 {
        return Err(AuthError::BadRequest(
            "El motivo de anulación es demasiado largo.".to_owned(),
        ));
    }
    auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
            let prescription =
                db::find_prescription_by_id(connection, &clinic_id, &prescription_id)?
                    .ok_or_else(|| AuthError::NotFound("Receta no encontrada.".to_owned()))?;
            if prescription.status == "voided" {
                let items = db::list_prescription_items(connection, &prescription.id)?;
                let verification =
                    db::find_verification_code_by_prescription(connection, &prescription.id)?;
                let patient_name =
                    db::find_patient_any(connection, &clinic_id, &prescription.patient_id)?
                        .map(|patient| patient.full_name)
                        .unwrap_or_else(|| "Paciente eliminado".to_owned());
                return Ok(PrescriptionDetail {
                    prescription,
                    items,
                    verification,
                    patient_name,
                });
            }
            prescription_transition_allowed(role, doctor_id.as_deref(), &prescription)?;
            db::mark_prescription_voided(connection, &clinic_id, &prescription.id, &reason, &at)?;
            db::set_verification_code_status(connection, &prescription.id, "voided", &at)?;
            record_audit(
                connection,
                Some(&actor_id),
                "void_prescription",
                "prescription",
                &prescription.id,
                Some(json!({ "reason": reason })),
                &at,
            )?;
            let voided = db::find_prescription_by_id(connection, &clinic_id, &prescription.id)?
                .ok_or_else(|| AuthError::NotFound("Receta no encontrada.".to_owned()))?;
            let items = db::list_prescription_items(connection, &voided.id)?;
            let verification = db::find_verification_code_by_prescription(connection, &voided.id)?;
            let patient_name = db::find_patient_any(connection, &clinic_id, &voided.patient_id)?
                .map(|patient| patient.full_name)
                .unwrap_or_else(|| "Paciente eliminado".to_owned());
            Ok(PrescriptionDetail {
                prescription: voided,
                items,
                verification,
                patient_name,
            })
        })
    })
    .await
}

pub async fn supersede_prescription(
    pool: &DbPool,
    actor: &ActorContext,
    prescription_id: String,
    input: PrescriptionSupersedeInput,
) -> Result<PrescriptionDetail, AuthError> {
    let clinic_id = actor.clinic_id()?.to_owned();
    let actor_id = actor.profile.id.clone();
    let role = actor.role();
    let doctor_id = actor.doctor.as_ref().map(|doctor| doctor.id.clone());
    let block_size = actor
        .clinic
        .as_ref()
        .map(|clinic| clinic.rx_block_size)
        .unwrap_or(50);
    let device_id = actor.device_id.clone().ok_or_else(|| {
        AuthError::BadRequest("Registra este dispositivo antes de corregir.".to_owned())
    })?;
    let client_id = input
        .client_id
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| Uuid::now_v7().to_string());
    let diagnosis_override = clean_rx_text(input.diagnosis, 2000);
    let validity_override = clean_rx_text(input.validity, 200);
    let instructions_override = clean_rx_text(input.instructions, 8000);
    auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
            if let Some(existing) =
                db::find_prescription_by_client_id(connection, &clinic_id, &client_id)?
            {
                if existing.supersedes_id.as_deref() != Some(prescription_id.as_str()) {
                    return Err(AuthError::Conflict(
                        "El identificador de cliente ya está en uso.".to_owned(),
                    ));
                }
                let items = db::list_prescription_items(connection, &existing.id)?;
                let verification =
                    db::find_verification_code_by_prescription(connection, &existing.id)?;
                let patient_name =
                    db::find_patient_any(connection, &clinic_id, &existing.patient_id)?
                        .map(|patient| patient.full_name)
                        .unwrap_or_else(|| "Paciente eliminado".to_owned());
                return Ok(PrescriptionDetail {
                    prescription: existing,
                    items,
                    verification,
                    patient_name,
                });
            }
            let original =
                db::find_prescription_by_id(connection, &clinic_id, &prescription_id)?
                    .ok_or_else(|| AuthError::NotFound("Receta no encontrada.".to_owned()))?;
            prescription_transition_allowed(role, doctor_id.as_deref(), &original)?;
            let warnings = prescription_warnings(connection, &original)?;
            if !warnings.is_empty() && !input.acknowledge_warnings {
                return Err(AuthError::UnacknowledgedWarnings(warnings));
            }
            let (block, number) = match (input.rx_number, input.rx_block_id.as_deref()) {
                (Some(number), Some(block_id)) => (
                    reconcile_block_number(
                        connection, &clinic_id, &device_id, block_id, number, &at,
                    )?,
                    number,
                ),
                (Some(_), None) => {
                    return Err(AuthError::BadRequest(
                        "Falta el bloque de números.".to_owned(),
                    ));
                }
                _ => consume_block_number(
                    connection, &actor_id, &clinic_id, &device_id, block_size, &at,
                )?,
            };
            let new_id = input
                .new_prescription_id
                .clone()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| Uuid::now_v7().to_string());
            db::insert_prescription(
                connection,
                &db::NewPrescription {
                    id: &new_id,
                    patient_id: &original.patient_id,
                    doctor_id: Some(doctor_id.as_deref().unwrap_or_default()),
                    clinic_id: &clinic_id,
                    template_version_id: original.template_version_id.as_deref(),
                    status: "signed",
                    rx_number: Some(number),
                    rx_block_id: Some(&block.id),
                    diagnosis: diagnosis_override
                        .as_deref()
                        .or(original.diagnosis.as_deref()),
                    validity: validity_override
                        .as_deref()
                        .or(original.validity.as_deref()),
                    instructions: instructions_override
                        .as_deref()
                        .or(original.instructions.as_deref()),
                    signed_at: Some(&at),
                    synced_at: None,
                    voided_at: None,
                    void_reason: None,
                    client_id: &client_id,
                    supersedes_id: Some(&original.id),
                    created_at: &at,
                    updated_at: &at,
                    created_by: Some(&actor_id),
                },
            )?;
            for item in db::list_prescription_items(connection, &original.id)? {
                let item_id = Uuid::now_v7().to_string();
                db::insert_prescription_item(
                    connection,
                    &db::NewPrescriptionItem {
                        id: &item_id,
                        prescription_id: &new_id,
                        position: item.position,
                        med_name: &item.med_name,
                        dose: item.dose.as_deref(),
                        route: item.route.as_deref(),
                        frequency: item.frequency.as_deref(),
                        duration: item.duration.as_deref(),
                        quantity: item.quantity.as_deref(),
                        notes: item.notes.as_deref(),
                        created_at: &at,
                        updated_at: &at,
                        created_by: Some(&actor_id),
                    },
                )?;
            }
            let (code, code_hash) = new_verification_code();
            let code_id = Uuid::now_v7().to_string();
            db::insert_verification_code(
                connection,
                &db::NewVerificationCode {
                    id: &code_id,
                    prescription_id: &new_id,
                    code: &code,
                    code_hash: &code_hash,
                    status: "valid",
                    created_at: &at,
                    updated_at: &at,
                    created_by: Some(&actor_id),
                },
            )?;
            db::mark_prescription_superseded(connection, &clinic_id, &original.id, &at)?;
            db::set_verification_code_status(connection, &original.id, "superseded", &at)?;
            record_audit(
                connection,
                Some(&actor_id),
                "supersede_prescription",
                "prescription",
                &new_id,
                Some(json!({
                    "supersedes_id": original.id,
                    "rx_number": number,
                    "warnings": warnings,
                })),
                &at,
            )?;
            let created = db::find_prescription_by_id(connection, &clinic_id, &new_id)?
                .ok_or_else(|| AuthError::NotFound("Receta no encontrada.".to_owned()))?;
            let items = db::list_prescription_items(connection, &created.id)?;
            let verification = db::find_verification_code_by_prescription(connection, &created.id)?;
            let patient_name = db::find_patient_any(connection, &clinic_id, &created.patient_id)?
                .map(|patient| patient.full_name)
                .unwrap_or_else(|| "Paciente eliminado".to_owned());
            Ok(PrescriptionDetail {
                prescription: created,
                items,
                verification,
                patient_name,
            })
        })
    })
    .await
}
