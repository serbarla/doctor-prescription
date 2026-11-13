use std::env;
use std::path::Path;
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, CustomizeConnection, Pool};
use diesel::sql_types::{BigInt, Text};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use crate::schema::{
    assets, audit_log, clinic_members, clinics, devices, doctors, patients, prescription_items,
    prescriptions, profiles, rx_blocks, rx_counters, sessions, template_version_assets,
    template_versions, templates, users, verification_codes,
};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

const PRAGMAS: &str = "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA busy_timeout = 5000; PRAGMA synchronous = NORMAL;";
#[allow(dead_code)]
const LOCK_ATTEMPTS: u32 = 6;
#[allow(dead_code)]
const LOCK_RETRY_DELAY: Duration = Duration::from_millis(25);

#[derive(Debug)]
struct ConnectionPragmas;

impl CustomizeConnection<SqliteConnection, diesel::r2d2::Error> for ConnectionPragmas {
    fn on_acquire(
        &self,
        conn: &mut SqliteConnection,
    ) -> std::result::Result<(), diesel::r2d2::Error> {
        conn.batch_execute(PRAGMAS)
            .map_err(diesel::r2d2::Error::QueryError)
    }
}

#[allow(dead_code)]
fn is_locked(error: &diesel::result::Error) -> bool {
    match error {
        diesel::result::Error::DatabaseError(_, info) => {
            let message = info.message().to_lowercase();
            message.contains("database is locked") || message.contains("database table is locked")
        }
        _ => false,
    }
}

#[allow(dead_code)]
fn backoff(attempt: u32) {
    thread::sleep(LOCK_RETRY_DELAY * attempt);
}

#[allow(dead_code)]
pub fn write_transaction<T, F>(
    connection: &mut SqliteConnection,
    mut operation: F,
) -> QueryResult<T>
where
    F: FnMut(&mut SqliteConnection) -> QueryResult<T>,
{
    let mut attempt = 0;
    loop {
        let result = connection
            .transaction::<T, diesel::result::Error, _>(|connection| operation(connection));
        match result {
            Ok(value) => return Ok(value),
            Err(error) if is_locked(&error) && attempt < LOCK_ATTEMPTS => {
                attempt += 1;
                backoff(attempt);
            }
            Err(error) => return Err(error),
        }
    }
}

pub fn immediate_transaction<T, E, F>(
    connection: &mut SqliteConnection,
    mut operation: F,
) -> Result<T, E>
where
    E: From<diesel::result::Error>,
    F: FnMut(&mut SqliteConnection) -> Result<T, E>,
{
    let mut attempt = 0;
    loop {
        if let Err(error) = connection.batch_execute("BEGIN IMMEDIATE") {
            if is_locked(&error) && attempt < LOCK_ATTEMPTS {
                attempt += 1;
                backoff(attempt);
                continue;
            }
            return Err(E::from(error));
        }
        match operation(connection) {
            Ok(value) => match connection.batch_execute("COMMIT") {
                Ok(()) => return Ok(value),
                Err(error) => {
                    let _ = connection.batch_execute("ROLLBACK");
                    if is_locked(&error) && attempt < LOCK_ATTEMPTS {
                        attempt += 1;
                        backoff(attempt);
                        continue;
                    }
                    return Err(E::from(error));
                }
            },
            Err(error) => {
                let _ = connection.batch_execute("ROLLBACK");
                return Err(error);
            }
        }
    }
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub display_name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub id: &'a str,
    pub email: &'a str,
    pub password_hash: &'a str,
    pub display_name: &'a str,
    pub created_at: &'a str,
    pub updated_at: &'a str,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = sessions)]
pub struct NewSession<'a> {
    pub id: &'a str,
    pub token_hash: &'a str,
    pub user_id: &'a str,
    pub expires_at: &'a str,
    pub created_at: &'a str,
    pub device_id: Option<&'a str>,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = sessions)]
pub struct SessionRow {
    pub id: String,
    pub user_id: String,
    pub expires_at: String,
    pub device_id: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = devices)]
pub struct Device {
    pub id: String,
    pub profile_id: String,
    pub name: Option<String>,
    pub platform: String,
    pub app_version: Option<String>,
    pub last_seen_at: Option<String>,
    pub revoked_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = devices)]
pub struct NewDevice<'a> {
    pub id: &'a str,
    pub profile_id: &'a str,
    pub name: Option<&'a str>,
    pub platform: &'a str,
    pub app_version: Option<&'a str>,
    pub last_seen_at: Option<&'a str>,
    pub revoked_at: Option<&'a str>,
    pub created_at: &'a str,
    pub updated_at: &'a str,
    pub created_by: Option<&'a str>,
}

#[allow(dead_code)]
#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = profiles)]
pub struct Profile {
    pub id: String,
    pub auth_user_id: String,
    pub display_name: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = profiles)]
pub struct NewProfile<'a> {
    pub id: &'a str,
    pub auth_user_id: &'a str,
    pub display_name: &'a str,
    pub status: &'a str,
    pub created_at: &'a str,
    pub updated_at: &'a str,
    pub created_by: Option<&'a str>,
}

#[allow(dead_code)]
#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = clinics)]
pub struct Clinic {
    pub id: String,
    pub name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub logo_asset_id: Option<String>,
    pub timezone: String,
    pub llm_url_template: Option<String>,
    pub retention_days: i32,
    pub rx_block_size: i32,
    pub server_retention_until: String,
    pub data_grace_ends_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = clinics)]
pub struct NewClinic<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub address: Option<&'a str>,
    pub phone: Option<&'a str>,
    pub logo_asset_id: Option<&'a str>,
    pub timezone: &'a str,
    pub llm_url_template: Option<&'a str>,
    pub retention_days: i32,
    pub rx_block_size: i32,
    pub server_retention_until: &'a str,
    pub data_grace_ends_at: Option<&'a str>,
    pub created_at: &'a str,
    pub updated_at: &'a str,
    pub created_by: Option<&'a str>,
}

#[allow(dead_code)]
#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = clinic_members)]
pub struct ClinicMember {
    pub clinic_id: String,
    pub profile_id: String,
    pub role: String,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = clinic_members)]
pub struct NewClinicMember<'a> {
    pub clinic_id: &'a str,
    pub profile_id: &'a str,
    pub role: &'a str,
    pub created_at: &'a str,
    pub updated_at: &'a str,
    pub created_by: Option<&'a str>,
}

#[allow(dead_code)]
#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = doctors)]
pub struct Doctor {
    pub id: String,
    pub profile_id: String,
    pub specialty: Option<String>,
    pub license_no: Option<String>,
    pub national_id: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = doctors)]
pub struct NewDoctor<'a> {
    pub id: &'a str,
    pub profile_id: &'a str,
    pub specialty: Option<&'a str>,
    pub license_no: Option<&'a str>,
    pub national_id: Option<&'a str>,
    pub phone: Option<&'a str>,
    pub email: Option<&'a str>,
    pub created_at: &'a str,
    pub updated_at: &'a str,
    pub created_by: Option<&'a str>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = patients)]
pub struct Patient {
    pub id: String,
    pub clinic_id: String,
    pub full_name: String,
    pub national_id: Option<String>,
    pub birth_date: Option<String>,
    pub sex: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
    pub client_id: Option<String>,
    pub deleted_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = patients)]
pub struct NewPatient<'a> {
    pub id: &'a str,
    pub clinic_id: &'a str,
    pub full_name: &'a str,
    pub national_id: Option<&'a str>,
    pub birth_date: Option<&'a str>,
    pub sex: Option<&'a str>,
    pub phone: Option<&'a str>,
    pub notes: Option<&'a str>,
    pub client_id: Option<&'a str>,
    pub deleted_at: Option<&'a str>,
    pub created_at: &'a str,
    pub updated_at: &'a str,
    pub created_by: Option<&'a str>,
}

pub struct PatientFields<'a> {
    pub full_name: &'a str,
    pub national_id: Option<&'a str>,
    pub birth_date: Option<&'a str>,
    pub sex: Option<&'a str>,
    pub phone: Option<&'a str>,
    pub notes: Option<&'a str>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = audit_log)]
pub struct NewAuditLog<'a> {
    pub id: &'a str,
    pub actor_id: Option<&'a str>,
    pub action: &'a str,
    pub entity: &'a str,
    pub entity_id: &'a str,
    pub metadata_json: Option<&'a str>,
    pub at: &'a str,
}

pub fn connect() -> Result<DbPool> {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://app.db".to_owned());
    ensure_parent_dir(&database_url)?;
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = Pool::builder()
        .max_size(8)
        .connection_customizer(Box::new(ConnectionPragmas))
        .build(manager)
        .context("could not open the SQLite database")?;
    run_migrations(&pool)?;
    Ok(pool)
}

fn ensure_parent_dir(database_url: &str) -> Result<()> {
    let path = database_url
        .strip_prefix("sqlite://")
        .or_else(|| database_url.strip_prefix("sqlite:"))
        .unwrap_or(database_url);
    let path = path.split('?').next().unwrap_or(path);
    if path.is_empty() || path == ":memory:" {
        return Ok(());
    }
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!(
                    "could not create the database directory {}",
                    parent.display()
                )
            })?;
        }
    }
    Ok(())
}

fn run_migrations(pool: &DbPool) -> Result<()> {
    let mut pooled = pool.get()?;
    let connection: &mut SqliteConnection = &mut pooled;
    connection
        .run_pending_migrations(MIGRATIONS)
        .map(|_| ())
        .map_err(|error| anyhow!("database migrations failed: {error}"))
}

pub fn find_user_by_email(
    connection: &mut SqliteConnection,
    email: &str,
) -> QueryResult<Option<User>> {
    users::table
        .filter(users::email.eq(email))
        .select(User::as_select())
        .first(connection)
        .optional()
}

pub fn find_user_by_id(connection: &mut SqliteConnection, id: &str) -> QueryResult<Option<User>> {
    users::table
        .filter(users::id.eq(id))
        .select(User::as_select())
        .first(connection)
        .optional()
}

pub fn insert_user(connection: &mut SqliteConnection, user: &NewUser<'_>) -> QueryResult<usize> {
    diesel::insert_into(users::table)
        .values(user)
        .execute(connection)
}

pub fn insert_session(
    connection: &mut SqliteConnection,
    session: &NewSession<'_>,
) -> QueryResult<usize> {
    diesel::insert_into(sessions::table)
        .values(session)
        .execute(connection)
}

pub fn find_session_by_token_hash(
    connection: &mut SqliteConnection,
    token_hash: &str,
) -> QueryResult<Option<SessionRow>> {
    sessions::table
        .filter(sessions::token_hash.eq(token_hash))
        .select(SessionRow::as_select())
        .first(connection)
        .optional()
}

pub fn link_session_device(
    connection: &mut SqliteConnection,
    session_id: &str,
    device_id: &str,
) -> QueryResult<usize> {
    diesel::update(sessions::table.filter(sessions::id.eq(session_id)))
        .set(sessions::device_id.eq(Some(device_id)))
        .execute(connection)
}

pub fn delete_sessions_by_device(
    connection: &mut SqliteConnection,
    device_id: &str,
) -> QueryResult<usize> {
    diesel::delete(sessions::table.filter(sessions::device_id.eq(Some(device_id))))
        .execute(connection)
}

pub fn find_device_by_id(
    connection: &mut SqliteConnection,
    id: &str,
) -> QueryResult<Option<Device>> {
    devices::table
        .filter(devices::id.eq(id))
        .select(Device::as_select())
        .first(connection)
        .optional()
}

pub fn insert_device(
    connection: &mut SqliteConnection,
    device: &NewDevice<'_>,
) -> QueryResult<usize> {
    diesel::insert_into(devices::table)
        .values(device)
        .execute(connection)
}

pub fn update_device_seen(
    connection: &mut SqliteConnection,
    id: &str,
    name: Option<&str>,
    app_version: Option<&str>,
    seen_at: &str,
) -> QueryResult<usize> {
    diesel::update(devices::table.filter(devices::id.eq(id)))
        .set((
            devices::name.eq(name),
            devices::app_version.eq(app_version),
            devices::last_seen_at.eq(Some(seen_at)),
            devices::updated_at.eq(seen_at),
        ))
        .execute(connection)
}

pub fn revoke_device(
    connection: &mut SqliteConnection,
    id: &str,
    revoked_at: &str,
) -> QueryResult<usize> {
    diesel::update(devices::table.filter(devices::id.eq(id)))
        .set((
            devices::revoked_at.eq(Some(revoked_at)),
            devices::updated_at.eq(revoked_at),
        ))
        .execute(connection)
}

pub fn list_devices_by_profile(
    connection: &mut SqliteConnection,
    profile_id: &str,
) -> QueryResult<Vec<Device>> {
    devices::table
        .filter(devices::profile_id.eq(profile_id))
        .select(Device::as_select())
        .order(devices::created_at.asc())
        .load(connection)
}

pub fn list_devices_by_clinic(
    connection: &mut SqliteConnection,
    clinic_id: &str,
) -> QueryResult<Vec<(Device, String)>> {
    let clinic_id = clinic_id.to_owned();
    devices::table
        .inner_join(profiles::table.on(profiles::id.eq(devices::profile_id)))
        .inner_join(clinic_members::table.on(clinic_members::profile_id.eq(profiles::id)))
        .filter(clinic_members::clinic_id.eq(clinic_id))
        .select((Device::as_select(), profiles::display_name))
        .order((profiles::display_name.asc(), devices::created_at.asc()))
        .load(connection)
}

pub fn delete_session_by_token_hash(
    connection: &mut SqliteConnection,
    token_hash: &str,
) -> QueryResult<usize> {
    diesel::delete(sessions::table.filter(sessions::token_hash.eq(token_hash))).execute(connection)
}

pub fn delete_expired_sessions(connection: &mut SqliteConnection, now: &str) -> QueryResult<usize> {
    diesel::delete(sessions::table.filter(sessions::expires_at.lt(now))).execute(connection)
}

pub fn count_clinics(connection: &mut SqliteConnection) -> QueryResult<i64> {
    clinics::table.count().get_result(connection)
}

pub fn find_clinic_by_id(
    connection: &mut SqliteConnection,
    id: &str,
) -> QueryResult<Option<Clinic>> {
    clinics::table
        .filter(clinics::id.eq(id))
        .select(Clinic::as_select())
        .first(connection)
        .optional()
}

pub fn insert_clinic(
    connection: &mut SqliteConnection,
    clinic: &NewClinic<'_>,
) -> QueryResult<usize> {
    diesel::insert_into(clinics::table)
        .values(clinic)
        .execute(connection)
}

pub fn find_profile_by_id(
    connection: &mut SqliteConnection,
    id: &str,
) -> QueryResult<Option<Profile>> {
    profiles::table
        .filter(profiles::id.eq(id))
        .select(Profile::as_select())
        .first(connection)
        .optional()
}

pub fn find_profile_by_auth_user_id(
    connection: &mut SqliteConnection,
    auth_user_id: &str,
) -> QueryResult<Option<Profile>> {
    profiles::table
        .filter(profiles::auth_user_id.eq(auth_user_id))
        .select(Profile::as_select())
        .first(connection)
        .optional()
}

pub fn insert_profile(
    connection: &mut SqliteConnection,
    profile: &NewProfile<'_>,
) -> QueryResult<usize> {
    diesel::insert_into(profiles::table)
        .values(profile)
        .execute(connection)
}

pub fn update_profile_display_name(
    connection: &mut SqliteConnection,
    id: &str,
    display_name: &str,
    now: &str,
) -> QueryResult<usize> {
    diesel::update(profiles::table.filter(profiles::id.eq(id)))
        .set((
            profiles::display_name.eq(display_name),
            profiles::updated_at.eq(now),
        ))
        .execute(connection)
}

pub fn update_profile_status(
    connection: &mut SqliteConnection,
    id: &str,
    status: &str,
    now: &str,
) -> QueryResult<usize> {
    diesel::update(profiles::table.filter(profiles::id.eq(id)))
        .set((profiles::status.eq(status), profiles::updated_at.eq(now)))
        .execute(connection)
}

pub fn update_user_display_name(
    connection: &mut SqliteConnection,
    id: &str,
    display_name: &str,
    now: &str,
) -> QueryResult<usize> {
    diesel::update(users::table.filter(users::id.eq(id)))
        .set((
            users::display_name.eq(display_name),
            users::updated_at.eq(now),
        ))
        .execute(connection)
}

pub fn find_membership_by_profile(
    connection: &mut SqliteConnection,
    profile_id: &str,
) -> QueryResult<Option<ClinicMember>> {
    clinic_members::table
        .filter(clinic_members::profile_id.eq(profile_id))
        .select(ClinicMember::as_select())
        .first(connection)
        .optional()
}

pub fn insert_clinic_member(
    connection: &mut SqliteConnection,
    member: &NewClinicMember<'_>,
) -> QueryResult<usize> {
    diesel::insert_into(clinic_members::table)
        .values(member)
        .execute(connection)
}

pub fn update_clinic_member_role(
    connection: &mut SqliteConnection,
    clinic_id: &str,
    profile_id: &str,
    role: &str,
    now: &str,
) -> QueryResult<usize> {
    diesel::update(
        clinic_members::table
            .filter(clinic_members::clinic_id.eq(clinic_id))
            .filter(clinic_members::profile_id.eq(profile_id)),
    )
    .set((
        clinic_members::role.eq(role),
        clinic_members::updated_at.eq(now),
    ))
    .execute(connection)
}

pub fn delete_clinic_member(
    connection: &mut SqliteConnection,
    clinic_id: &str,
    profile_id: &str,
) -> QueryResult<usize> {
    diesel::delete(
        clinic_members::table
            .filter(clinic_members::clinic_id.eq(clinic_id))
            .filter(clinic_members::profile_id.eq(profile_id)),
    )
    .execute(connection)
}

pub fn count_administrators(
    connection: &mut SqliteConnection,
    clinic_id: &str,
) -> QueryResult<i64> {
    clinic_members::table
        .filter(clinic_members::clinic_id.eq(clinic_id))
        .filter(clinic_members::role.eq("administrator"))
        .count()
        .get_result(connection)
}

pub fn find_doctor_by_profile_id(
    connection: &mut SqliteConnection,
    profile_id: &str,
) -> QueryResult<Option<Doctor>> {
    doctors::table
        .filter(doctors::profile_id.eq(profile_id))
        .select(Doctor::as_select())
        .first(connection)
        .optional()
}

pub fn insert_doctor(
    connection: &mut SqliteConnection,
    doctor: &NewDoctor<'_>,
) -> QueryResult<usize> {
    diesel::insert_into(doctors::table)
        .values(doctor)
        .execute(connection)
}

pub struct DoctorFields<'a> {
    pub specialty: Option<&'a str>,
    pub license_no: Option<&'a str>,
    pub national_id: Option<&'a str>,
    pub phone: Option<&'a str>,
    pub email: Option<&'a str>,
}

pub fn update_doctor(
    connection: &mut SqliteConnection,
    id: &str,
    fields: &DoctorFields<'_>,
    now: &str,
) -> QueryResult<usize> {
    diesel::update(doctors::table.filter(doctors::id.eq(id)))
        .set((
            doctors::specialty.eq(fields.specialty),
            doctors::license_no.eq(fields.license_no),
            doctors::national_id.eq(fields.national_id),
            doctors::phone.eq(fields.phone),
            doctors::email.eq(fields.email),
            doctors::updated_at.eq(now),
        ))
        .execute(connection)
}

pub fn insert_audit(
    connection: &mut SqliteConnection,
    entry: &NewAuditLog<'_>,
) -> QueryResult<usize> {
    diesel::insert_into(audit_log::table)
        .values(entry)
        .execute(connection)
}

pub fn find_patient_by_id(
    connection: &mut SqliteConnection,
    clinic_id: &str,
    id: &str,
) -> QueryResult<Option<Patient>> {
    patients::table
        .filter(patients::clinic_id.eq(clinic_id))
        .filter(patients::id.eq(id))
        .filter(patients::deleted_at.is_null())
        .select(Patient::as_select())
        .first(connection)
        .optional()
}

pub fn find_patient_id_by_national_id(
    connection: &mut SqliteConnection,
    clinic_id: &str,
    national_id: &str,
) -> QueryResult<Option<String>> {
    patients::table
        .filter(patients::clinic_id.eq(clinic_id))
        .filter(patients::national_id.eq(Some(national_id)))
        .filter(patients::deleted_at.is_null())
        .select(patients::id)
        .first(connection)
        .optional()
}

pub fn list_patient_names(
    connection: &mut SqliteConnection,
    clinic_id: &str,
) -> QueryResult<Vec<(String, String)>> {
    patients::table
        .filter(patients::clinic_id.eq(clinic_id))
        .filter(patients::deleted_at.is_null())
        .select((patients::id, patients::full_name))
        .load(connection)
}

pub fn search_patients(
    connection: &mut SqliteConnection,
    clinic_id: &str,
    query: Option<&str>,
    limit: i64,
) -> QueryResult<Vec<Patient>> {
    let mut statement = patients::table
        .filter(patients::clinic_id.eq(clinic_id))
        .filter(patients::deleted_at.is_null())
        .into_boxed();
    if let Some(query) = query {
        let pattern = format!("%{query}%");
        statement = statement.filter(
            patients::full_name
                .like(pattern)
                .or(patients::national_id.eq(Some(query))),
        );
    }
    statement
        .select(Patient::as_select())
        .order(patients::full_name.asc())
        .limit(limit)
        .load(connection)
}

pub fn insert_patient(
    connection: &mut SqliteConnection,
    patient: &NewPatient<'_>,
) -> QueryResult<usize> {
    diesel::insert_into(patients::table)
        .values(patient)
        .execute(connection)
}

pub fn update_patient(
    connection: &mut SqliteConnection,
    clinic_id: &str,
    id: &str,
    fields: &PatientFields<'_>,
    now: &str,
) -> QueryResult<usize> {
    diesel::update(
        patients::table
            .filter(patients::clinic_id.eq(clinic_id))
            .filter(patients::id.eq(id))
            .filter(patients::deleted_at.is_null()),
    )
    .set((
        patients::full_name.eq(fields.full_name),
        patients::national_id.eq(fields.national_id),
        patients::birth_date.eq(fields.birth_date),
        patients::sex.eq(fields.sex),
        patients::phone.eq(fields.phone),
        patients::notes.eq(fields.notes),
        patients::updated_at.eq(now),
    ))
    .execute(connection)
}

pub fn soft_delete_patient(
    connection: &mut SqliteConnection,
    clinic_id: &str,
    id: &str,
    now: &str,
) -> QueryResult<usize> {
    diesel::update(
        patients::table
            .filter(patients::clinic_id.eq(clinic_id))
            .filter(patients::id.eq(id))
            .filter(patients::deleted_at.is_null()),
    )
    .set((
        patients::deleted_at.eq(Some(now)),
        patients::updated_at.eq(now),
    ))
    .execute(connection)
}

pub fn list_profiles_with_membership(
    connection: &mut SqliteConnection,
    clinic_id: &str,
) -> QueryResult<Vec<(Profile, Option<String>, String)>> {
    let clinic_id = clinic_id.to_owned();
    profiles::table
        .left_join(
            clinic_members::table.on(clinic_members::profile_id
                .eq(profiles::id)
                .and(clinic_members::clinic_id.eq(clinic_id))),
        )
        .inner_join(users::table.on(users::id.eq(profiles::auth_user_id)))
        .select((
            Profile::as_select(),
            clinic_members::role.nullable(),
            users::email,
        ))
        .order(profiles::display_name.asc())
        .load(connection)
}

#[allow(dead_code)]
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = templates)]
pub struct Template {
    pub id: String,
    pub owner_type: String,
    pub owner_id: String,
    pub name: String,
    pub current_version_id: Option<String>,
    pub status: String,
    pub client_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = templates)]
pub struct NewTemplate<'a> {
    pub id: &'a str,
    pub owner_type: &'a str,
    pub owner_id: &'a str,
    pub name: &'a str,
    pub current_version_id: Option<&'a str>,
    pub status: &'a str,
    pub client_id: Option<&'a str>,
    pub created_at: &'a str,
    pub updated_at: &'a str,
    pub created_by: Option<&'a str>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = template_versions)]
pub struct TemplateVersion {
    pub id: String,
    pub template_id: String,
    pub version_no: i32,
    pub doc_json: String,
    pub variables_spec: String,
    pub theme: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = template_versions)]
pub struct NewTemplateVersion<'a> {
    pub id: &'a str,
    pub template_id: &'a str,
    pub version_no: i32,
    pub doc_json: &'a str,
    pub variables_spec: &'a str,
    pub theme: Option<&'a str>,
    pub status: &'a str,
    pub created_at: &'a str,
    pub updated_at: &'a str,
    pub created_by: Option<&'a str>,
}

pub fn list_templates_by_owner(
    connection: &mut SqliteConnection,
    owner_type: &str,
    owner_id: &str,
) -> QueryResult<Vec<Template>> {
    templates::table
        .filter(templates::owner_type.eq(owner_type))
        .filter(templates::owner_id.eq(owner_id))
        .select(Template::as_select())
        .order(templates::created_at.desc())
        .load(connection)
}

pub fn find_template_by_id(
    connection: &mut SqliteConnection,
    id: &str,
) -> QueryResult<Option<Template>> {
    templates::table
        .filter(templates::id.eq(id))
        .select(Template::as_select())
        .first(connection)
        .optional()
}

pub fn insert_template(
    connection: &mut SqliteConnection,
    template: &NewTemplate<'_>,
) -> QueryResult<usize> {
    diesel::insert_into(templates::table)
        .values(template)
        .execute(connection)
}

pub fn find_template_version(
    connection: &mut SqliteConnection,
    template_id: &str,
    id: &str,
) -> QueryResult<Option<TemplateVersion>> {
    template_versions::table
        .filter(template_versions::template_id.eq(template_id))
        .filter(template_versions::id.eq(id))
        .select(TemplateVersion::as_select())
        .first(connection)
        .optional()
}

pub fn find_template_version_by_id(
    connection: &mut SqliteConnection,
    id: &str,
) -> QueryResult<Option<TemplateVersion>> {
    template_versions::table
        .filter(template_versions::id.eq(id))
        .select(TemplateVersion::as_select())
        .first(connection)
        .optional()
}

pub fn list_template_versions(
    connection: &mut SqliteConnection,
    template_id: &str,
) -> QueryResult<Vec<TemplateVersion>> {
    template_versions::table
        .filter(template_versions::template_id.eq(template_id))
        .select(TemplateVersion::as_select())
        .order(template_versions::version_no.desc())
        .load(connection)
}

pub fn latest_template_version_no(
    connection: &mut SqliteConnection,
    template_id: &str,
) -> QueryResult<Option<i32>> {
    template_versions::table
        .filter(template_versions::template_id.eq(template_id))
        .select(diesel::dsl::max(template_versions::version_no))
        .first(connection)
}

pub fn insert_template_version(
    connection: &mut SqliteConnection,
    version: &NewTemplateVersion<'_>,
) -> QueryResult<usize> {
    diesel::insert_into(template_versions::table)
        .values(version)
        .execute(connection)
}

pub fn update_template_version_draft(
    connection: &mut SqliteConnection,
    id: &str,
    doc_json: &str,
    variables_spec: &str,
    theme: Option<&str>,
    now: &str,
) -> QueryResult<usize> {
    diesel::update(
        template_versions::table
            .filter(template_versions::id.eq(id))
            .filter(template_versions::status.eq("draft")),
    )
    .set((
        template_versions::doc_json.eq(doc_json),
        template_versions::variables_spec.eq(variables_spec),
        template_versions::theme.eq(theme),
        template_versions::updated_at.eq(now),
    ))
    .execute(connection)
}

pub fn publish_template_version_row(
    connection: &mut SqliteConnection,
    id: &str,
    now: &str,
) -> QueryResult<usize> {
    diesel::update(
        template_versions::table
            .filter(template_versions::id.eq(id))
            .filter(template_versions::status.eq("draft")),
    )
    .set((
        template_versions::status.eq("published"),
        template_versions::updated_at.eq(now),
    ))
    .execute(connection)
}

pub fn set_template_published(
    connection: &mut SqliteConnection,
    id: &str,
    current_version_id: &str,
    now: &str,
) -> QueryResult<usize> {
    diesel::update(templates::table.filter(templates::id.eq(id)))
        .set((
            templates::current_version_id.eq(Some(current_version_id)),
            templates::status.eq("published"),
            templates::updated_at.eq(now),
        ))
        .execute(connection)
}

pub fn archive_template_row(
    connection: &mut SqliteConnection,
    id: &str,
    now: &str,
) -> QueryResult<usize> {
    diesel::update(templates::table.filter(templates::id.eq(id)))
        .set((
            templates::status.eq("archived"),
            templates::updated_at.eq(now),
        ))
        .execute(connection)
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = assets)]
pub struct Asset {
    pub id: String,
    pub clinic_id: String,
    pub object_key: String,
    pub owner_type: String,
    pub owner_id: String,
    pub kind: String,
    pub original_name: Option<String>,
    pub content_type: String,
    pub size_bytes: i32,
    pub sha256: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub status: String,
    pub deleted_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = assets)]
pub struct NewAsset<'a> {
    pub id: &'a str,
    pub clinic_id: &'a str,
    pub object_key: &'a str,
    pub owner_type: &'a str,
    pub owner_id: &'a str,
    pub kind: &'a str,
    pub original_name: Option<&'a str>,
    pub content_type: &'a str,
    pub size_bytes: i32,
    pub sha256: &'a str,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub status: &'a str,
    pub deleted_at: Option<&'a str>,
    pub created_at: &'a str,
    pub updated_at: &'a str,
    pub created_by: Option<&'a str>,
}

#[derive(QueryableByName)]
struct AssetReferenceCount {
    #[diesel(sql_type = BigInt)]
    ref_count: i64,
}

pub fn insert_asset(connection: &mut SqliteConnection, asset: &NewAsset<'_>) -> QueryResult<usize> {
    diesel::insert_into(assets::table)
        .values(asset)
        .execute(connection)
}

pub fn find_asset_by_id(connection: &mut SqliteConnection, id: &str) -> QueryResult<Option<Asset>> {
    assets::table
        .filter(assets::id.eq(id))
        .filter(assets::deleted_at.is_null())
        .select(Asset::as_select())
        .first(connection)
        .optional()
}

pub fn list_assets_by_clinic(
    connection: &mut SqliteConnection,
    clinic_id: &str,
) -> QueryResult<Vec<Asset>> {
    assets::table
        .filter(assets::clinic_id.eq(clinic_id))
        .filter(assets::deleted_at.is_null())
        .select(Asset::as_select())
        .order(assets::created_at.desc())
        .load(connection)
}

pub fn list_assets_by_kind(
    connection: &mut SqliteConnection,
    clinic_id: &str,
    kind: &str,
) -> QueryResult<Vec<Asset>> {
    assets::table
        .filter(assets::clinic_id.eq(clinic_id))
        .filter(assets::kind.eq(kind))
        .filter(assets::deleted_at.is_null())
        .select(Asset::as_select())
        .order(assets::created_at.desc())
        .load(connection)
}

pub fn count_assets_by_kind(
    connection: &mut SqliteConnection,
    clinic_id: &str,
    kind: &str,
) -> QueryResult<i64> {
    assets::table
        .filter(assets::clinic_id.eq(clinic_id))
        .filter(assets::kind.eq(kind))
        .filter(assets::deleted_at.is_null())
        .count()
        .get_result(connection)
}

pub fn mark_asset_ready(
    connection: &mut SqliteConnection,
    id: &str,
    now: &str,
) -> QueryResult<usize> {
    diesel::update(assets::table.filter(assets::id.eq(id)))
        .set((assets::status.eq("ready"), assets::updated_at.eq(now)))
        .execute(connection)
}

pub fn soft_delete_asset(
    connection: &mut SqliteConnection,
    id: &str,
    now: &str,
) -> QueryResult<usize> {
    diesel::update(assets::table.filter(assets::id.eq(id)))
        .set((assets::deleted_at.eq(Some(now)), assets::updated_at.eq(now)))
        .execute(connection)
}

pub fn hard_delete_asset(connection: &mut SqliteConnection, id: &str) -> QueryResult<usize> {
    diesel::delete(assets::table.filter(assets::id.eq(id))).execute(connection)
}

pub fn count_asset_references(
    connection: &mut SqliteConnection,
    asset_id: &str,
) -> QueryResult<i64> {
    diesel::sql_query(
        "SELECT ((SELECT COUNT(*) FROM template_version_assets tva JOIN template_versions tv ON tv.id = tva.template_version_id WHERE tva.asset_id = ? AND tv.status = 'published') + (SELECT COUNT(*) FROM prescriptions p JOIN template_version_assets tva ON tva.template_version_id = p.template_version_id WHERE tva.asset_id = ?)) AS ref_count",
    )
    .bind::<Text, _>(asset_id)
    .bind::<Text, _>(asset_id)
    .get_result::<AssetReferenceCount>(connection)
    .map(|row| row.ref_count)
}

pub fn list_template_version_asset_ids(
    connection: &mut SqliteConnection,
    template_version_id: &str,
) -> QueryResult<Vec<String>> {
    template_version_assets::table
        .filter(template_version_assets::template_version_id.eq(template_version_id))
        .select(template_version_assets::asset_id)
        .load(connection)
}

pub fn insert_template_version_asset(
    connection: &mut SqliteConnection,
    template_version_id: &str,
    asset_id: &str,
) -> QueryResult<usize> {
    diesel::insert_into(template_version_assets::table)
        .values((
            template_version_assets::template_version_id.eq(template_version_id),
            template_version_assets::asset_id.eq(asset_id),
        ))
        .on_conflict_do_nothing()
        .execute(connection)
}

pub fn set_clinic_branding(
    connection: &mut SqliteConnection,
    clinic_id: &str,
    name: &str,
    address: Option<&str>,
    phone: Option<&str>,
    now: &str,
) -> QueryResult<usize> {
    diesel::update(clinics::table.filter(clinics::id.eq(clinic_id)))
        .set((
            clinics::name.eq(name),
            clinics::address.eq(address),
            clinics::phone.eq(phone),
            clinics::updated_at.eq(now),
        ))
        .execute(connection)
}

pub fn set_clinic_logo(
    connection: &mut SqliteConnection,
    clinic_id: &str,
    asset_id: Option<&str>,
    now: &str,
) -> QueryResult<usize> {
    diesel::update(clinics::table.filter(clinics::id.eq(clinic_id)))
        .set((
            clinics::logo_asset_id.eq(asset_id),
            clinics::updated_at.eq(now),
        ))
        .execute(connection)
}

#[allow(dead_code)]
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = prescriptions)]
pub struct Prescription {
    pub id: String,
    pub patient_id: String,
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
    pub synced_at: Option<String>,
    pub voided_at: Option<String>,
    pub void_reason: Option<String>,
    pub client_id: String,
    pub supersedes_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = prescriptions)]
pub struct NewPrescription<'a> {
    pub id: &'a str,
    pub patient_id: &'a str,
    pub doctor_id: Option<&'a str>,
    pub clinic_id: &'a str,
    pub template_version_id: Option<&'a str>,
    pub status: &'a str,
    pub rx_number: Option<i32>,
    pub rx_block_id: Option<&'a str>,
    pub diagnosis: Option<&'a str>,
    pub validity: Option<&'a str>,
    pub instructions: Option<&'a str>,
    pub signed_at: Option<&'a str>,
    pub synced_at: Option<&'a str>,
    pub voided_at: Option<&'a str>,
    pub void_reason: Option<&'a str>,
    pub client_id: &'a str,
    pub supersedes_id: Option<&'a str>,
    pub created_at: &'a str,
    pub updated_at: &'a str,
    pub created_by: Option<&'a str>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = prescription_items)]
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
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = prescription_items)]
pub struct NewPrescriptionItem<'a> {
    pub id: &'a str,
    pub prescription_id: &'a str,
    pub position: i32,
    pub med_name: &'a str,
    pub dose: Option<&'a str>,
    pub route: Option<&'a str>,
    pub frequency: Option<&'a str>,
    pub duration: Option<&'a str>,
    pub quantity: Option<&'a str>,
    pub notes: Option<&'a str>,
    pub created_at: &'a str,
    pub updated_at: &'a str,
    pub created_by: Option<&'a str>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = verification_codes)]
pub struct VerificationCode {
    pub id: String,
    pub prescription_id: String,
    pub code: String,
    pub code_hash: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = verification_codes)]
pub struct NewVerificationCode<'a> {
    pub id: &'a str,
    pub prescription_id: &'a str,
    pub code: &'a str,
    pub code_hash: &'a str,
    pub status: &'a str,
    pub created_at: &'a str,
    pub updated_at: &'a str,
    pub created_by: Option<&'a str>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = rx_counters)]
pub struct RxCounter {
    pub clinic_id: String,
    pub last_rx_number: i32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = rx_blocks)]
pub struct RxBlock {
    pub id: String,
    pub clinic_id: String,
    pub device_id: String,
    pub start_number: i32,
    pub end_number: i32,
    pub consumed: i32,
    pub status: String,
    pub allocated_at: String,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = rx_blocks)]
pub struct NewRxBlock<'a> {
    pub id: &'a str,
    pub clinic_id: &'a str,
    pub device_id: &'a str,
    pub start_number: i32,
    pub end_number: i32,
    pub consumed: i32,
    pub status: &'a str,
    pub allocated_at: &'a str,
    pub created_at: &'a str,
    pub updated_at: &'a str,
    pub created_by: Option<&'a str>,
}

pub fn find_prescription_by_id(
    connection: &mut SqliteConnection,
    clinic_id: &str,
    id: &str,
) -> QueryResult<Option<Prescription>> {
    prescriptions::table
        .filter(prescriptions::clinic_id.eq(clinic_id))
        .filter(prescriptions::id.eq(id))
        .select(Prescription::as_select())
        .first(connection)
        .optional()
}

pub fn find_prescription_by_client_id(
    connection: &mut SqliteConnection,
    clinic_id: &str,
    client_id: &str,
) -> QueryResult<Option<Prescription>> {
    prescriptions::table
        .filter(prescriptions::clinic_id.eq(clinic_id))
        .filter(prescriptions::client_id.eq(client_id))
        .select(Prescription::as_select())
        .first(connection)
        .optional()
}

pub fn list_prescriptions(
    connection: &mut SqliteConnection,
    clinic_id: &str,
    doctor_id: Option<&str>,
    patient_id: Option<&str>,
    status: Option<&str>,
    date: Option<&str>,
    limit: i64,
) -> QueryResult<Vec<Prescription>> {
    let mut statement = prescriptions::table
        .filter(prescriptions::clinic_id.eq(clinic_id))
        .into_boxed();
    if let Some(doctor_id) = doctor_id {
        statement = statement.filter(prescriptions::doctor_id.eq(Some(doctor_id)));
    }
    if let Some(patient_id) = patient_id {
        statement = statement.filter(prescriptions::patient_id.eq(patient_id));
    }
    if let Some(status) = status {
        statement = statement.filter(prescriptions::status.eq(status));
    }
    if let Some(date) = date {
        statement = statement.filter(prescriptions::created_at.like(format!("{date}%")));
    }
    statement
        .select(Prescription::as_select())
        .order(prescriptions::created_at.desc())
        .limit(limit)
        .load(connection)
}

pub fn insert_prescription(
    connection: &mut SqliteConnection,
    prescription: &NewPrescription<'_>,
) -> QueryResult<usize> {
    diesel::insert_into(prescriptions::table)
        .values(prescription)
        .execute(connection)
}

pub fn update_prescription_draft(
    connection: &mut SqliteConnection,
    clinic_id: &str,
    id: &str,
    diagnosis: Option<&str>,
    validity: Option<&str>,
    instructions: Option<&str>,
    now: &str,
) -> QueryResult<usize> {
    diesel::update(
        prescriptions::table
            .filter(prescriptions::clinic_id.eq(clinic_id))
            .filter(prescriptions::id.eq(id))
            .filter(prescriptions::status.eq("draft")),
    )
    .set((
        prescriptions::diagnosis.eq(diagnosis),
        prescriptions::validity.eq(validity),
        prescriptions::instructions.eq(instructions),
        prescriptions::updated_at.eq(now),
    ))
    .execute(connection)
}

pub fn mark_prescription_signed(
    connection: &mut SqliteConnection,
    clinic_id: &str,
    id: &str,
    doctor_id: &str,
    rx_number: i32,
    rx_block_id: &str,
    signed_at: &str,
) -> QueryResult<usize> {
    diesel::update(
        prescriptions::table
            .filter(prescriptions::clinic_id.eq(clinic_id))
            .filter(prescriptions::id.eq(id))
            .filter(prescriptions::status.eq("draft")),
    )
    .set((
        prescriptions::status.eq("signed"),
        prescriptions::doctor_id.eq(Some(doctor_id)),
        prescriptions::rx_number.eq(Some(rx_number)),
        prescriptions::rx_block_id.eq(Some(rx_block_id)),
        prescriptions::signed_at.eq(Some(signed_at)),
        prescriptions::updated_at.eq(signed_at),
    ))
    .execute(connection)
}

pub fn mark_prescription_voided(
    connection: &mut SqliteConnection,
    clinic_id: &str,
    id: &str,
    reason: &str,
    at: &str,
) -> QueryResult<usize> {
    diesel::update(
        prescriptions::table
            .filter(prescriptions::clinic_id.eq(clinic_id))
            .filter(prescriptions::id.eq(id))
            .filter(prescriptions::status.eq("signed")),
    )
    .set((
        prescriptions::status.eq("voided"),
        prescriptions::voided_at.eq(Some(at)),
        prescriptions::void_reason.eq(Some(reason)),
        prescriptions::updated_at.eq(at),
    ))
    .execute(connection)
}

pub fn mark_prescription_superseded(
    connection: &mut SqliteConnection,
    clinic_id: &str,
    id: &str,
    at: &str,
) -> QueryResult<usize> {
    diesel::update(
        prescriptions::table
            .filter(prescriptions::clinic_id.eq(clinic_id))
            .filter(prescriptions::id.eq(id))
            .filter(prescriptions::status.eq("signed")),
    )
    .set((
        prescriptions::status.eq("superseded"),
        prescriptions::updated_at.eq(at),
    ))
    .execute(connection)
}

pub fn list_prescription_items(
    connection: &mut SqliteConnection,
    prescription_id: &str,
) -> QueryResult<Vec<PrescriptionItem>> {
    prescription_items::table
        .filter(prescription_items::prescription_id.eq(prescription_id))
        .select(PrescriptionItem::as_select())
        .order(prescription_items::position.asc())
        .load(connection)
}

pub fn find_prescription_item(
    connection: &mut SqliteConnection,
    prescription_id: &str,
    id: &str,
) -> QueryResult<Option<PrescriptionItem>> {
    prescription_items::table
        .filter(prescription_items::prescription_id.eq(prescription_id))
        .filter(prescription_items::id.eq(id))
        .select(PrescriptionItem::as_select())
        .first(connection)
        .optional()
}

pub fn insert_prescription_item(
    connection: &mut SqliteConnection,
    item: &NewPrescriptionItem<'_>,
) -> QueryResult<usize> {
    diesel::insert_into(prescription_items::table)
        .values(item)
        .execute(connection)
}

pub struct PrescriptionItemFields<'a> {
    pub position: i32,
    pub med_name: &'a str,
    pub dose: Option<&'a str>,
    pub route: Option<&'a str>,
    pub frequency: Option<&'a str>,
    pub duration: Option<&'a str>,
    pub quantity: Option<&'a str>,
    pub notes: Option<&'a str>,
}

pub fn update_prescription_item(
    connection: &mut SqliteConnection,
    prescription_id: &str,
    id: &str,
    fields: &PrescriptionItemFields<'_>,
    now: &str,
) -> QueryResult<usize> {
    diesel::update(
        prescription_items::table
            .filter(prescription_items::prescription_id.eq(prescription_id))
            .filter(prescription_items::id.eq(id)),
    )
    .set((
        prescription_items::position.eq(fields.position),
        prescription_items::med_name.eq(fields.med_name),
        prescription_items::dose.eq(fields.dose),
        prescription_items::route.eq(fields.route),
        prescription_items::frequency.eq(fields.frequency),
        prescription_items::duration.eq(fields.duration),
        prescription_items::quantity.eq(fields.quantity),
        prescription_items::notes.eq(fields.notes),
        prescription_items::updated_at.eq(now),
    ))
    .execute(connection)
}

pub fn update_prescription_item_position(
    connection: &mut SqliteConnection,
    prescription_id: &str,
    id: &str,
    position: i32,
    now: &str,
) -> QueryResult<usize> {
    diesel::update(
        prescription_items::table
            .filter(prescription_items::prescription_id.eq(prescription_id))
            .filter(prescription_items::id.eq(id)),
    )
    .set((
        prescription_items::position.eq(position),
        prescription_items::updated_at.eq(now),
    ))
    .execute(connection)
}

pub fn delete_prescription_item(
    connection: &mut SqliteConnection,
    prescription_id: &str,
    id: &str,
) -> QueryResult<usize> {
    diesel::delete(
        prescription_items::table
            .filter(prescription_items::prescription_id.eq(prescription_id))
            .filter(prescription_items::id.eq(id)),
    )
    .execute(connection)
}

pub fn find_verification_code_by_prescription(
    connection: &mut SqliteConnection,
    prescription_id: &str,
) -> QueryResult<Option<VerificationCode>> {
    verification_codes::table
        .filter(verification_codes::prescription_id.eq(prescription_id))
        .select(VerificationCode::as_select())
        .first(connection)
        .optional()
}

pub fn insert_verification_code(
    connection: &mut SqliteConnection,
    code: &NewVerificationCode<'_>,
) -> QueryResult<usize> {
    diesel::insert_into(verification_codes::table)
        .values(code)
        .execute(connection)
}

pub fn set_verification_code_status(
    connection: &mut SqliteConnection,
    prescription_id: &str,
    status: &str,
    now: &str,
) -> QueryResult<usize> {
    diesel::update(
        verification_codes::table.filter(verification_codes::prescription_id.eq(prescription_id)),
    )
    .set((
        verification_codes::status.eq(status),
        verification_codes::updated_at.eq(now),
    ))
    .execute(connection)
}

pub fn find_rx_counter(
    connection: &mut SqliteConnection,
    clinic_id: &str,
) -> QueryResult<Option<RxCounter>> {
    rx_counters::table
        .filter(rx_counters::clinic_id.eq(clinic_id))
        .select(RxCounter::as_select())
        .first(connection)
        .optional()
}

pub fn insert_rx_counter(
    connection: &mut SqliteConnection,
    clinic_id: &str,
    last_rx_number: i32,
) -> QueryResult<usize> {
    diesel::insert_into(rx_counters::table)
        .values((
            rx_counters::clinic_id.eq(clinic_id),
            rx_counters::last_rx_number.eq(last_rx_number),
        ))
        .execute(connection)
}

pub fn set_rx_counter(
    connection: &mut SqliteConnection,
    clinic_id: &str,
    last_rx_number: i32,
) -> QueryResult<usize> {
    diesel::update(rx_counters::table.filter(rx_counters::clinic_id.eq(clinic_id)))
        .set(rx_counters::last_rx_number.eq(last_rx_number))
        .execute(connection)
}

pub fn find_active_rx_block(
    connection: &mut SqliteConnection,
    device_id: &str,
) -> QueryResult<Option<RxBlock>> {
    rx_blocks::table
        .filter(rx_blocks::device_id.eq(device_id))
        .filter(rx_blocks::status.eq("active"))
        .select(RxBlock::as_select())
        .first(connection)
        .optional()
}

pub fn find_rx_block_by_id(
    connection: &mut SqliteConnection,
    id: &str,
) -> QueryResult<Option<RxBlock>> {
    rx_blocks::table
        .filter(rx_blocks::id.eq(id))
        .select(RxBlock::as_select())
        .first(connection)
        .optional()
}

pub fn insert_rx_block(
    connection: &mut SqliteConnection,
    block: &NewRxBlock<'_>,
) -> QueryResult<usize> {
    diesel::insert_into(rx_blocks::table)
        .values(block)
        .execute(connection)
}

pub fn set_rx_block_consumed(
    connection: &mut SqliteConnection,
    id: &str,
    consumed: i32,
    now: &str,
) -> QueryResult<usize> {
    diesel::update(rx_blocks::table.filter(rx_blocks::id.eq(id)))
        .set((
            rx_blocks::consumed.eq(consumed),
            rx_blocks::updated_at.eq(now),
        ))
        .execute(connection)
}

pub fn set_rx_block_status(
    connection: &mut SqliteConnection,
    id: &str,
    status: &str,
    now: &str,
) -> QueryResult<usize> {
    diesel::update(rx_blocks::table.filter(rx_blocks::id.eq(id)))
        .set((rx_blocks::status.eq(status), rx_blocks::updated_at.eq(now)))
        .execute(connection)
}

pub fn find_doctor_by_id(
    connection: &mut SqliteConnection,
    id: &str,
) -> QueryResult<Option<Doctor>> {
    doctors::table
        .filter(doctors::id.eq(id))
        .select(Doctor::as_select())
        .first(connection)
        .optional()
}

pub fn find_patient_any(
    connection: &mut SqliteConnection,
    clinic_id: &str,
    id: &str,
) -> QueryResult<Option<Patient>> {
    patients::table
        .filter(patients::clinic_id.eq(clinic_id))
        .filter(patients::id.eq(id))
        .select(Patient::as_select())
        .first(connection)
        .optional()
}

pub fn set_clinic_llm_url(
    connection: &mut SqliteConnection,
    clinic_id: &str,
    llm_url_template: Option<&str>,
    now: &str,
) -> QueryResult<usize> {
    diesel::update(clinics::table.filter(clinics::id.eq(clinic_id)))
        .set((
            clinics::llm_url_template.eq(llm_url_template),
            clinics::updated_at.eq(now),
        ))
        .execute(connection)
}
