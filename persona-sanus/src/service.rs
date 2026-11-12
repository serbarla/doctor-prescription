use axum::http::HeaderMap;
use chrono::{Duration as ChronoDuration, Months, NaiveDate, SecondsFormat, Utc};
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

pub struct PatientInput {
    pub full_name: String,
    pub national_id: Option<String>,
    pub birth_date: Option<String>,
    pub sex: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
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
        let Some((_, user_id, _)) = db::find_session_by_token_hash(connection, &token_hash)? else {
            return Ok(None);
        };
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
            profile,
            clinic,
            member,
            doctor,
        }))
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
    auth_server::with_conn(pool, move |connection| {
        let at = auth_server::now();
        db::immediate_transaction(connection, |connection| {
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
                    client_id: None,
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
