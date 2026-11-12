use std::env;
use std::path::Path;
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, CustomizeConnection, Pool};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use crate::schema::{
    audit_log, clinic_members, clinics, doctors, patients, profiles, sessions, users,
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
) -> QueryResult<Option<(String, String, String)>> {
    sessions::table
        .filter(sessions::token_hash.eq(token_hash))
        .select((sessions::id, sessions::user_id, sessions::expires_at))
        .first(connection)
        .optional()
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
