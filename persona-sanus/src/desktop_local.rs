use chrono::{DateTime, NaiveDate, SecondsFormat, Utc};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Integer, Nullable, Text};
use diesel::sqlite::SqliteConnection;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use uuid::Uuid;

const SCHEMA: &str = r"
CREATE TABLE IF NOT EXISTS clinics (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  address TEXT,
  phone TEXT,
  logo_asset_id TEXT,
  timezone TEXT NOT NULL DEFAULT 'America/Guayaquil',
  llm_url_template TEXT,
  retention_days INTEGER NOT NULL DEFAULT 30,
  rx_block_size INTEGER NOT NULL DEFAULT 50,
  server_retention_until TEXT,
  data_grace_ends_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  created_by TEXT
);
CREATE TABLE IF NOT EXISTS patients (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL,
  full_name TEXT NOT NULL,
  national_id TEXT,
  birth_date TEXT,
  sex TEXT,
  phone TEXT,
  notes TEXT,
  client_id TEXT,
  deleted_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  created_by TEXT
);
CREATE TABLE IF NOT EXISTS templates (
  id TEXT PRIMARY KEY,
  owner_type TEXT NOT NULL,
  owner_id TEXT NOT NULL,
  name TEXT NOT NULL,
  current_version_id TEXT,
  status TEXT NOT NULL DEFAULT 'draft',
  client_id TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  created_by TEXT
);
CREATE TABLE IF NOT EXISTS template_versions (
  id TEXT PRIMARY KEY,
  template_id TEXT NOT NULL,
  version_no INTEGER NOT NULL,
  doc_json TEXT NOT NULL,
  variables_spec TEXT NOT NULL,
  theme TEXT,
  status TEXT NOT NULL DEFAULT 'draft',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  created_by TEXT
);
CREATE TABLE IF NOT EXISTS template_version_assets (
  template_version_id TEXT NOT NULL,
  asset_id TEXT NOT NULL,
  PRIMARY KEY (template_version_id, asset_id)
);
CREATE TABLE IF NOT EXISTS assets (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL,
  object_key TEXT,
  owner_type TEXT,
  owner_id TEXT,
  kind TEXT NOT NULL DEFAULT 'image',
  original_name TEXT,
  content_type TEXT,
  size_bytes INTEGER,
  sha256 TEXT,
  width INTEGER,
  height INTEGER,
  status TEXT NOT NULL DEFAULT 'ready',
  deleted_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  created_by TEXT
);
CREATE TABLE IF NOT EXISTS rx_blocks (
  id TEXT PRIMARY KEY,
  clinic_id TEXT NOT NULL,
  device_id TEXT NOT NULL,
  start_number INTEGER NOT NULL,
  end_number INTEGER NOT NULL,
  consumed INTEGER NOT NULL DEFAULT 0,
  status TEXT NOT NULL DEFAULT 'active',
  allocated_at TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  created_by TEXT
);
CREATE TABLE IF NOT EXISTS prescriptions (
  id TEXT PRIMARY KEY,
  patient_id TEXT NOT NULL,
  doctor_id TEXT,
  clinic_id TEXT NOT NULL,
  template_version_id TEXT,
  status TEXT NOT NULL DEFAULT 'draft',
  rx_number INTEGER,
  rx_block_id TEXT,
  diagnosis TEXT,
  validity TEXT,
  instructions TEXT,
  signed_at TEXT,
  synced_at TEXT,
  voided_at TEXT,
  void_reason TEXT,
  client_id TEXT NOT NULL,
  supersedes_id TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  created_by TEXT
);
CREATE TABLE IF NOT EXISTS prescription_items (
  id TEXT PRIMARY KEY,
  prescription_id TEXT NOT NULL,
  position INTEGER NOT NULL,
  med_name TEXT NOT NULL,
  dose TEXT,
  route TEXT,
  frequency TEXT,
  duration TEXT,
  quantity TEXT,
  notes TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  created_by TEXT
);
CREATE TABLE IF NOT EXISTS verification_codes (
  id TEXT PRIMARY KEY,
  prescription_id TEXT NOT NULL,
  code TEXT NOT NULL,
  code_hash TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'valid',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  created_by TEXT
);
CREATE TABLE IF NOT EXISTS outbox (
  id TEXT PRIMARY KEY,
  operation TEXT NOT NULL CHECK (operation IN (
    'create_patient','update_patient','create_template',
    'publish_template_version','create_prescription',
    'update_prescription','upsert_prescription_item','sign_prescription',
    'void_prescription','supersede_prescription')),
  entity_type TEXT NOT NULL,
  entity_id TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','failed','synced')),
  attempts INTEGER NOT NULL DEFAULT 0,
  next_retry_at TEXT,
  last_error TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_outbox_pending ON outbox (status, created_at);
CREATE TABLE IF NOT EXISTS sync_meta (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS asset_cache (
  asset_id TEXT PRIMARY KEY,
  sha256 TEXT NOT NULL,
  bytes INTEGER NOT NULL,
  local_path TEXT NOT NULL,
  fetched_at TEXT NOT NULL
);
";

#[derive(Debug, Clone, QueryableByName)]
struct JsonRow {
    #[diesel(sql_type = Text)]
    row_json: String,
}

#[derive(Debug, Clone, QueryableByName)]
struct IntRow {
    #[diesel(sql_type = BigInt)]
    value: i64,
}

#[derive(Debug, Clone, QueryableByName)]
struct TextRow {
    #[diesel(sql_type = Text)]
    value: String,
}

#[derive(Debug, Clone, QueryableByName)]
struct OptRow {
    #[diesel(sql_type = Nullable<Text>)]
    value: Option<String>,
}

fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub fn app_data_dir() -> PathBuf {
    if let Ok(value) = std::env::var("PERSONA_LOCAL_DATA_DIR") {
        if !value.trim().is_empty() {
            return PathBuf::from(value);
        }
    }
    #[cfg(target_os = "macos")]
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join("Library/Application Support/PersonaSanus");
    }
    #[cfg(target_os = "windows")]
    if let Ok(roaming) = std::env::var("APPDATA") {
        return PathBuf::from(roaming).join("PersonaSanus");
    }
    PathBuf::new()
}

fn exec(conn: &mut SqliteConnection, sql: &str) -> Result<(), String> {
    diesel::sql_query(sql)
        .execute(conn)
        .map(|_| ())
        .map_err(|error| error.to_string())
}
fn immediate<T, F>(conn: &mut SqliteConnection, mut operation: F) -> Result<T, String>
where
    F: FnMut(&mut SqliteConnection) -> Result<T, String>,
{
    use diesel::connection::SimpleConnection;
    let mut attempt = 0;
    loop {
        if let Err(error) = conn.batch_execute("BEGIN IMMEDIATE") {
            let message = error.to_string().to_lowercase();
            let locked = message.contains("database is locked")
                || message.contains("database table is locked")
                || message.contains("busy");
            if locked && attempt < 12 {
                attempt += 1;
                std::thread::sleep(std::time::Duration::from_millis(25 * attempt));
                continue;
            }
            return Err(message);
        }
        match operation(conn) {
            Ok(value) => match conn.batch_execute("COMMIT") {
                Ok(()) => return Ok(value),
                Err(error) => {
                    let _ = conn.batch_execute("ROLLBACK");
                    return Err(error.to_string());
                }
            },
            Err(error) => {
                let _ = conn.batch_execute("ROLLBACK");
                return Err(error);
            }
        }
    }
}

pub fn cache_dir() -> PathBuf {
    app_data_dir().join("asset-cache")
}

fn outbox_needs_migration(conn: &mut SqliteConnection) -> Result<bool, String> {
    let rows = diesel::sql_query(
        "SELECT sql AS value FROM sqlite_master WHERE type = 'table' AND name = 'outbox'",
    )
    .load::<OptRow>(conn)
    .map_err(|error| error.to_string())?;
    Ok(rows
        .into_iter()
        .next()
        .and_then(|row| row.value)
        .is_some_and(|sql| !sql.contains("'synced'")))
}

fn migrate_outbox(conn: &mut SqliteConnection) -> Result<(), String> {
    exec(conn, "ALTER TABLE outbox RENAME TO outbox_old")?;
    exec(
        conn,
        "CREATE TABLE outbox (id TEXT PRIMARY KEY, operation TEXT NOT NULL CHECK (operation IN ('create_patient','update_patient','create_template','publish_template_version','create_prescription','update_prescription','upsert_prescription_item','sign_prescription','void_prescription','supersede_prescription')), entity_type TEXT NOT NULL, entity_id TEXT NOT NULL, payload_json TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','failed','synced')), attempts INTEGER NOT NULL DEFAULT 0, next_retry_at TEXT, last_error TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL)",
    )?;
    exec(
        conn,
        "INSERT INTO outbox (id, operation, entity_type, entity_id, payload_json, status, attempts, next_retry_at, last_error, created_at, updated_at) SELECT id, operation, entity_type, entity_id, payload_json, status, attempts, next_retry_at, last_error, created_at, updated_at FROM outbox_old",
    )?;
    exec(conn, "DROP TABLE outbox_old")?;
    exec(
        conn,
        "CREATE INDEX IF NOT EXISTS idx_outbox_pending ON outbox (status, created_at)",
    )
}

pub fn open_local_db() -> Result<SqliteConnection, String> {
    let dir = app_data_dir();
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))
            .map_err(|error| error.to_string())?;
    }
    let path = dir.join("desktop-local.sqlite");
    let mut conn =
        SqliteConnection::establish(&path.to_string_lossy()).map_err(|error| error.to_string())?;
    exec(&mut conn, "PRAGMA foreign_keys=ON")?;
    exec(&mut conn, "PRAGMA journal_mode=WAL")?;
    exec(&mut conn, "PRAGMA busy_timeout=5000")?;
    use diesel::connection::SimpleConnection;
    conn.batch_execute(SCHEMA)
        .map_err(|error| error.to_string())?;
    if outbox_needs_migration(&mut conn)? {
        migrate_outbox(&mut conn)?;
    }
    exec(&mut conn, "PRAGMA user_version=1")?;
    Ok(conn)
}

fn json_rows(conn: &mut SqliteConnection, sql: &str) -> Result<Vec<Value>, String> {
    let rows: Vec<JsonRow> = diesel::sql_query(sql)
        .load(conn)
        .map_err(|error| error.to_string())?;
    rows.into_iter()
        .map(|row| serde_json::from_str(&row.row_json).map_err(|error| error.to_string()))
        .collect()
}

fn sql_scalar(value: &Value) -> String {
    match value {
        Value::Null => "NULL".to_owned(),
        Value::Bool(flag) => {
            if *flag {
                "1".to_owned()
            } else {
                "0".to_owned()
            }
        }
        Value::Number(number) => number.to_string(),
        Value::String(text) => format!("'{}'", text.replace('\'', "''")),
        Value::Array(_) | Value::Object(_) => format!(
            "'{}'",
            serde_json::to_string(value)
                .unwrap_or_default()
                .replace('\'', "''")
        ),
    }
}

fn upsert_json(
    conn: &mut SqliteConnection,
    table: &str,
    columns: &[&str],
    value: &Value,
) -> Result<(), String> {
    let mut insert_columns = String::new();
    let mut insert_values = String::new();
    for (index, column) in columns.iter().enumerate() {
        if index > 0 {
            insert_columns.push_str(", ");
            insert_values.push_str(", ");
        }
        insert_columns.push_str(column);
        let field = value.get(*column).cloned().unwrap_or(Value::Null);
        insert_values.push_str(&sql_scalar(&field));
    }
    let mut updates = String::new();
    for column in columns.iter().filter(|column| **column != "id") {
        if !updates.is_empty() {
            updates.push_str(", ");
        }
        updates.push_str(&format!("{column} = excluded.{column}"));
    }
    let sql = format!(
        "INSERT INTO {table} ({insert_columns}) VALUES ({insert_values}) ON CONFLICT(id) DO UPDATE SET {updates}"
    );
    exec(conn, &sql)
}

pub fn set_meta(conn: &mut SqliteConnection, key: &str, value: &str) -> Result<(), String> {
    diesel::sql_query(
        "INSERT INTO sync_meta (key, value, updated_at) VALUES (:key, :value, :updated_at) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
    )
    .bind::<Text, _>(key)
    .bind::<Text, _>(value)
    .bind::<Text, _>(now().as_str())
    .execute(conn)
    .map(|_| ())
    .map_err(|error| error.to_string())
}

pub fn get_meta(conn: &mut SqliteConnection, key: &str) -> Result<Option<String>, String> {
    let rows = diesel::sql_query("SELECT value AS value FROM sync_meta WHERE key = :key")
        .bind::<Text, _>(key)
        .load::<OptRow>(conn)
        .map_err(|error| error.to_string())?;
    Ok(rows.into_iter().next().and_then(|row| row.value))
}
pub struct LocalSession {
    pub token: String,
    pub user: Value,
    pub profile: Option<Value>,
    pub expires_at: String,
}
pub fn save_local_session(
    token: &str,
    user: &Value,
    profile: Option<&Value>,
    expires_at: &str,
) -> Result<(), String> {
    let mut conn = open_local_db()?;
    set_meta(&mut conn, "session_token", token)?;
    set_meta(&mut conn, "session_user", &user.to_string())?;
    if let Some(profile) = profile {
        set_meta(&mut conn, "session_profile", &profile.to_string())?;
    } else {
        delete_meta(&mut conn, "session_profile")?;
    }
    set_meta(&mut conn, "session_expires_at", expires_at)?;
    Ok(())
}
pub fn refresh_local_session(user: &Value, profile: Option<&Value>) -> Result<(), String> {
    let mut conn = open_local_db()?;
    if get_meta(&mut conn, "session_token")?.is_none() {
        return Ok(());
    }
    set_meta(&mut conn, "session_user", &user.to_string())?;
    if let Some(profile) = profile {
        set_meta(&mut conn, "session_profile", &profile.to_string())?;
    }
    Ok(())
}
pub fn load_local_session() -> Option<LocalSession> {
    let mut conn = open_local_db().ok()?;
    let token = get_meta(&mut conn, "session_token").ok().flatten()?;
    let user_raw = get_meta(&mut conn, "session_user").ok().flatten()?;
    let user: Value = serde_json::from_str(&user_raw).ok()?;
    let profile = get_meta(&mut conn, "session_profile")
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str(&raw).ok());
    let expires_at = get_meta(&mut conn, "session_expires_at")
        .ok()
        .flatten()
        .unwrap_or_default();
    if !expires_at.is_empty() {
        let expired = chrono::DateTime::parse_from_rfc3339(&expires_at)
            .map(|when| when.timestamp() <= Utc::now().timestamp())
            .unwrap_or(true);
        if expired {
            return None;
        }
    }
    Some(LocalSession {
        token,
        user,
        profile,
        expires_at,
    })
}
pub fn clear_local_session() {
    if let Ok(mut conn) = open_local_db() {
        let _ = delete_meta(&mut conn, "session_token");
        let _ = delete_meta(&mut conn, "session_user");
        let _ = delete_meta(&mut conn, "session_profile");
        let _ = delete_meta(&mut conn, "session_expires_at");
    }
}
fn delete_meta(conn: &mut SqliteConnection, key: &str) -> Result<(), String> {
    diesel::sql_query("DELETE FROM sync_meta WHERE key = :key")
        .bind::<Text, _>(key)
        .execute(conn)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

pub fn cache_clinic(conn: &mut SqliteConnection, clinic: &Value) -> Result<(), String> {
    upsert_json(
        conn,
        "clinics",
        &[
            "id",
            "name",
            "address",
            "phone",
            "logo_asset_id",
            "timezone",
            "llm_url_template",
            "retention_days",
            "rx_block_size",
            "server_retention_until",
            "data_grace_ends_at",
            "created_at",
            "updated_at",
            "created_by",
        ],
        clinic,
    )
}

pub fn cache_patient(conn: &mut SqliteConnection, patient: &Value) -> Result<(), String> {
    upsert_json(
        conn,
        "patients",
        &[
            "id",
            "clinic_id",
            "full_name",
            "national_id",
            "birth_date",
            "sex",
            "phone",
            "notes",
            "client_id",
            "deleted_at",
            "created_at",
            "updated_at",
            "created_by",
        ],
        patient,
    )
}

pub fn cache_template(conn: &mut SqliteConnection, template: &Value) -> Result<(), String> {
    upsert_json(
        conn,
        "templates",
        &[
            "id",
            "owner_type",
            "owner_id",
            "name",
            "current_version_id",
            "status",
            "client_id",
            "created_at",
            "updated_at",
            "created_by",
        ],
        template,
    )
}

pub fn cache_template_version(conn: &mut SqliteConnection, version: &Value) -> Result<(), String> {
    upsert_json(
        conn,
        "template_versions",
        &[
            "id",
            "template_id",
            "version_no",
            "doc_json",
            "variables_spec",
            "theme",
            "status",
            "created_at",
            "updated_at",
            "created_by",
        ],
        version,
    )
}

pub fn cache_prescription(conn: &mut SqliteConnection, prescription: &Value) -> Result<(), String> {
    upsert_json(
        conn,
        "prescriptions",
        &[
            "id",
            "patient_id",
            "doctor_id",
            "clinic_id",
            "template_version_id",
            "status",
            "rx_number",
            "rx_block_id",
            "diagnosis",
            "validity",
            "instructions",
            "signed_at",
            "synced_at",
            "voided_at",
            "void_reason",
            "client_id",
            "supersedes_id",
            "created_at",
            "updated_at",
            "created_by",
        ],
        prescription,
    )
}

pub fn cache_asset_meta(conn: &mut SqliteConnection, asset: &Value) -> Result<(), String> {
    upsert_json(
        conn,
        "assets",
        &[
            "id",
            "clinic_id",
            "object_key",
            "owner_type",
            "owner_id",
            "kind",
            "original_name",
            "content_type",
            "size_bytes",
            "sha256",
            "width",
            "height",
            "status",
            "deleted_at",
            "created_at",
            "updated_at",
            "created_by",
        ],
        asset,
    )
}

pub fn cache_rx_block(conn: &mut SqliteConnection, block: &Value) -> Result<(), String> {
    upsert_json(
        conn,
        "rx_blocks",
        &[
            "id",
            "clinic_id",
            "device_id",
            "start_number",
            "end_number",
            "consumed",
            "status",
            "allocated_at",
            "created_at",
            "updated_at",
            "created_by",
        ],
        block,
    )
}
pub fn cache_prescription_item(conn: &mut SqliteConnection, item: &Value) -> Result<(), String> {
    upsert_json(
        conn,
        "prescription_items",
        &[
            "id",
            "prescription_id",
            "position",
            "med_name",
            "dose",
            "route",
            "frequency",
            "duration",
            "quantity",
            "notes",
            "created_at",
            "updated_at",
            "created_by",
        ],
        item,
    )
}

pub fn cache_rx_block_status(conn: &mut SqliteConnection, block: &Value) -> Result<(), String> {
    let mut value = block.clone();
    let id = value.get("id").and_then(Value::as_str).unwrap_or_default();
    let existing = json_rows(
        conn,
        &format!(
            "SELECT json_object('allocatedAt', allocated_at, 'createdAt', created_at) AS row_json FROM rx_blocks WHERE id = '{}'",
            id.replace('\'', "''")
        ),
    )?;
    let at = now();
    if let Some(row) = existing.into_iter().next() {
        value["allocated_at"] = row.get("allocatedAt").cloned().unwrap_or(Value::Null);
        value["created_at"] = row.get("createdAt").cloned().unwrap_or(Value::Null);
    } else {
        value["allocated_at"] = json!(at);
        value["created_at"] = json!(at);
    }
    if value.get("updated_at").is_none() || value.get("updated_at") == Some(&Value::Null) {
        value["updated_at"] = json!(at);
    }
    cache_rx_block(conn, &value)
}
pub fn rx_block_clinic_id(conn: &mut SqliteConnection, id: &str) -> Result<Option<String>, String> {
    let rows = json_rows(
        conn,
        &format!(
            "SELECT json_object('clinicId', clinic_id) AS row_json FROM rx_blocks WHERE id = '{}'",
            id.replace('\'', "''")
        ),
    )?;
    Ok(rows.into_iter().next().and_then(|row| {
        row.get("clinicId")
            .and_then(Value::as_str)
            .map(str::to_owned)
    }))
}

pub fn has_pending_outbox(conn: &mut SqliteConnection, entity_id: &str) -> Result<bool, String> {
    let rows = diesel::sql_query(&format!(
        "SELECT COUNT(*) AS value FROM outbox WHERE entity_id = '{}' AND status = 'pending'",
        entity_id.replace('\'', "''")
    ))
    .load::<IntRow>(conn)
    .map_err(|error| error.to_string())?;
    Ok(rows.iter().any(|row| row.value > 0))
}

pub fn list_patients(conn: &mut SqliteConnection) -> Result<Vec<Value>, String> {
    json_rows(
        conn,
        "SELECT json_object('id', id, 'clinicId', clinic_id, 'fullName', full_name, \
         'nationalId', national_id, 'birthDate', birth_date, 'sex', sex, 'phone', phone, \
         'notes', notes, 'createdAt', created_at, 'updatedAt', updated_at, 'createdBy', created_by) AS row_json \
         FROM patients WHERE deleted_at IS NULL ORDER BY full_name",
    )
}

pub fn list_templates(conn: &mut SqliteConnection) -> Result<Vec<Value>, String> {
    json_rows(
        conn,
        "SELECT json_object('id', templates.id, 'ownerType', templates.owner_type, 'ownerId', templates.owner_id, \
         'name', templates.name, 'currentVersionId', templates.current_version_id, 'status', templates.status, \
         'createdAt', templates.created_at, 'updatedAt', templates.updated_at, \
         'currentVersion', CASE WHEN template_versions.id IS NULL THEN NULL ELSE json_object('id', template_versions.id, 'templateId', template_versions.template_id, 'versionNo', template_versions.version_no, 'docJson', template_versions.doc_json, 'variablesSpec', json_extract(template_versions.variables_spec, '$'), 'theme', template_versions.theme, 'status', template_versions.status, 'createdAt', template_versions.created_at, 'updatedAt', template_versions.updated_at) END, \
         'versions', json_array()) AS row_json \
         FROM templates LEFT JOIN template_versions ON template_versions.id = templates.current_version_id ORDER BY templates.name",
    )
}

pub fn list_template_versions(
    conn: &mut SqliteConnection,
    template_id: &str,
) -> Result<Vec<Value>, String> {
    json_rows(
        conn,
        &format!(
            "SELECT json_object('id', id, 'templateId', template_id, 'versionNo', version_no, \
             'status', status, 'createdAt', created_at) AS row_json \
             FROM template_versions WHERE template_id = '{}' ORDER BY version_no",
            template_id.replace('\'', "''")
        ),
    )
}

pub fn list_prescriptions(conn: &mut SqliteConnection, status: &str) -> Result<Vec<Value>, String> {
    let filter = if status.is_empty() {
        String::new()
    } else if status == "pending_sync" {
        " WHERE status <> 'draft' AND id IN (SELECT entity_id FROM outbox WHERE entity_type = 'prescription' AND status = 'pending')"
            .to_owned()
    } else if status == "failed" {
        " WHERE status <> 'draft' AND id IN (SELECT entity_id FROM outbox WHERE entity_type = 'prescription' AND status = 'failed')"
            .to_owned()
    } else {
        format!(" WHERE status = '{}'", status.replace('\'', "''"))
    };
    json_rows(
        conn,
        &format!(
            "SELECT json_object('id', id, 'patientId', patient_id, 'patientName', \
             COALESCE((SELECT full_name FROM patients WHERE patients.id = prescriptions.patient_id), 'Paciente'), \
             'doctorId', doctor_id, 'clinicId', clinic_id, 'templateVersionId', template_version_id, \
             'status', status, 'rxNumber', rx_number, 'rxBlockId', rx_block_id, \
             'diagnosis', diagnosis, 'validity', validity, 'instructions', instructions, \
             'signedAt', signed_at, 'voidedAt', voided_at, 'voidReason', void_reason, \
             'clientId', client_id, 'supersedesId', supersedes_id, 'createdAt', created_at, \
             'updatedAt', updated_at, 'createdBy', created_by, 'items', json_array()) AS row_json \
             FROM prescriptions{filter} ORDER BY created_at DESC"
        ),
    )
}

pub fn get_prescription(conn: &mut SqliteConnection, id: &str) -> Result<Option<Value>, String> {
    let rows = list_prescriptions(conn, "")?;
    Ok(rows
        .into_iter()
        .find(|row| row.get("id").and_then(Value::as_str) == Some(id)))
}

pub fn get_template_version(
    conn: &mut SqliteConnection,
    id: &str,
) -> Result<Option<Value>, String> {
    let rows = json_rows(
        conn,
        &format!(
            "SELECT json_object('id', id, 'templateId', template_id, 'versionNo', version_no, \
             'docJson', doc_json, 'variablesSpec', variables_spec, 'theme', theme, \
             'status', status, 'createdAt', created_at) AS row_json \
             FROM template_versions WHERE id = '{}'",
            id.replace('\'', "''")
        ),
    )?;
    Ok(rows.into_iter().next())
}

pub fn pending_count(conn: &mut SqliteConnection) -> Result<i64, String> {
    let rows = diesel::sql_query(
        "SELECT COUNT(*) AS value FROM prescriptions WHERE status <> 'draft' AND id IN (SELECT entity_id FROM outbox WHERE entity_type = 'prescription' AND status = 'pending')",
    )
    .load::<IntRow>(conn)
    .map_err(|error| error.to_string())?;
    Ok(rows.into_iter().next().map(|row| row.value).unwrap_or(0))
}

pub fn failed_count(conn: &mut SqliteConnection) -> Result<i64, String> {
    let rows = diesel::sql_query(
        "SELECT COUNT(*) AS value FROM prescriptions WHERE status <> 'draft' AND id IN (SELECT entity_id FROM outbox WHERE entity_type = 'prescription' AND status = 'failed')",
    )
    .load::<IntRow>(conn)
    .map_err(|error| error.to_string())?;
    Ok(rows.into_iter().next().map(|row| row.value).unwrap_or(0))
}

pub fn enqueue_outbox(
    conn: &mut SqliteConnection,
    operation: &str,
    entity_type: &str,
    entity_id: &str,
    payload_json: &Value,
) -> Result<String, String> {
    let id = Uuid::now_v7().to_string();
    let payload = serde_json::to_string(payload_json).map_err(|error| error.to_string())?;
    let at = now();
    diesel::sql_query(
        "INSERT INTO outbox (id, operation, entity_type, entity_id, payload_json, status, attempts, next_retry_at, last_error, created_at, updated_at) \
         VALUES (:id, :operation, :entity_type, :entity_id, :payload_json, 'pending', 0, NULL, NULL, :created_at, :updated_at)",
    )
    .bind::<Text, _>(id.as_str())
    .bind::<Text, _>(operation)
    .bind::<Text, _>(entity_type)
    .bind::<Text, _>(entity_id)
    .bind::<Text, _>(payload.as_str())
    .bind::<Text, _>(at.as_str())
    .bind::<Text, _>(at.as_str())
    .execute(conn)
    .map(|_| ())
    .map_err(|error| error.to_string())?;
    Ok(id)
}

pub fn mark_outbox_failed(
    conn: &mut SqliteConnection,
    id: &str,
    error_message: &str,
) -> Result<(), String> {
    let at = now();
    diesel::sql_query(
        "UPDATE outbox SET status = 'failed', attempts = attempts + 1, last_error = :last_error, updated_at = :updated_at WHERE id = :id",
    )
    .bind::<Text, _>(error_message)
    .bind::<Text, _>(at.as_str())
    .bind::<Text, _>(id)
    .execute(conn)
    .map(|_| ())
    .map_err(|error| error.to_string())
}

pub fn outbox_rows(conn: &mut SqliteConnection) -> Result<Vec<Value>, String> {
    json_rows(
        conn,
        "SELECT json_object('id', id, 'operation', operation, 'entityType', entity_type, \
         'entityId', entity_id, 'payloadJson', payload_json, 'status', status, 'attempts', attempts, \
         'nextRetryAt', next_retry_at, 'lastError', last_error, 'createdAt', created_at) AS row_json \
         FROM outbox ORDER BY created_at",
    )
}

pub fn pending_outbox_rows(conn: &mut SqliteConnection) -> Result<Vec<Value>, String> {
    let at = now();
    json_rows(
        conn,
        &format!(
            "SELECT json_object('id', id, 'operation', operation, 'entityType', entity_type, \
             'entityId', entity_id, 'payloadJson', payload_json, 'status', status, 'attempts', attempts, \
             'nextRetryAt', next_retry_at, 'lastError', last_error, 'createdAt', created_at) AS row_json \
             FROM outbox WHERE status = 'pending' AND (next_retry_at IS NULL OR next_retry_at <= '{}') ORDER BY created_at, id",
            at.replace('\'', "''")
        ),
    )
}

pub fn mark_outbox_synced(conn: &mut SqliteConnection, id: &str) -> Result<(), String> {
    let at = now();
    diesel::sql_query(
        "UPDATE outbox SET status = 'synced', last_error = NULL, next_retry_at = NULL, updated_at = :updated_at WHERE id = :id",
    )
    .bind::<Text, _>(at.as_str())
    .bind::<Text, _>(id)
    .execute(conn)
    .map(|_| ())
    .map_err(|error| error.to_string())
}

pub fn mark_outbox_retry(
    conn: &mut SqliteConnection,
    id: &str,
    attempts: i64,
    error_message: &str,
) -> Result<(), String> {
    let backoff_minutes = attempts.clamp(1, 8);
    let retry_at = Utc::now() + chrono::Duration::minutes(backoff_minutes);
    let at = now();
    diesel::sql_query(
        "UPDATE outbox SET status = 'pending', attempts = attempts + 1, next_retry_at = :next_retry_at, last_error = :last_error, updated_at = :updated_at WHERE id = :id",
    )
    .bind::<Text, _>(retry_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true).as_str())
    .bind::<Text, _>(error_message)
    .bind::<Text, _>(at.as_str())
    .bind::<Text, _>(id)
    .execute(conn)
    .map(|_| ())
    .map_err(|error| error.to_string())
}

pub fn clear_prescription_items(
    conn: &mut SqliteConnection,
    prescription_id: &str,
) -> Result<(), String> {
    exec(
        conn,
        &format!(
            "DELETE FROM prescription_items WHERE prescription_id = '{}'",
            prescription_id.replace('\'', "''")
        ),
    )
}

pub fn evict_local_cache_row(
    conn: &mut SqliteConnection,
    table: &str,
    row_id: &str,
) -> Result<bool, String> {
    let allowed = [
        "patients",
        "prescriptions",
        "templates",
        "template_versions",
        "assets",
    ];
    if !allowed.contains(&table) {
        return Err(format!("Tabla no evictible: {table}"));
    }
    let rows = diesel::sql_query(&format!(
        "SELECT created_at AS value FROM {table} WHERE id = '{}'",
        row_id.replace('\'', "''")
    ))
    .load::<OptRow>(conn)
    .map_err(|error| error.to_string())?;
    let Some(created_raw) = rows.into_iter().next().and_then(|row| row.value) else {
        return Ok(false);
    };
    let retention = retention_days(conn)?;
    let created = created_raw
        .parse::<DateTime<Utc>>()
        .map_err(|error| error.to_string())?;
    let cutoff = Utc::now() - chrono::Duration::days(retention);
    if created >= cutoff {
        return Ok(false);
    }
    let queued = diesel::sql_query(&format!(
        "SELECT COUNT(*) AS value FROM outbox WHERE entity_id = '{}' AND status = 'pending'",
        row_id.replace('\'', "''")
    ))
    .load::<IntRow>(conn)
    .map_err(|error| error.to_string())?;
    if queued.iter().any(|row| row.value > 0) {
        return Ok(false);
    }
    if table == "prescriptions" {
        let unsynced = diesel::sql_query(&format!(
            "SELECT COUNT(*) AS value FROM prescriptions WHERE id = '{}' AND synced_at IS NULL",
            row_id.replace('\'', "''")
        ))
        .load::<IntRow>(conn)
        .map_err(|error| error.to_string())?;
        if unsynced.iter().any(|row| row.value > 0) {
            return Ok(false);
        }
    }
    exec(
        conn,
        &format!(
            "DELETE FROM {table} WHERE id = '{}'",
            row_id.replace('\'', "''")
        ),
    )?;
    Ok(true)
}

fn retention_days(conn: &mut SqliteConnection) -> Result<i64, String> {
    let rows = diesel::sql_query("SELECT retention_days AS value FROM clinics LIMIT 1")
        .load::<IntRow>(conn)
        .map_err(|error| error.to_string())?;
    Ok(rows.into_iter().next().map(|row| row.value).unwrap_or(30))
}

pub fn evict_expired(conn: &mut SqliteConnection) -> Result<i64, String> {
    let rows = diesel::sql_query("SELECT id AS value FROM prescriptions")
        .load::<TextRow>(conn)
        .map_err(|error| error.to_string())?;
    let mut evicted = 0;
    for row in rows {
        if evict_local_cache_row(conn, "prescriptions", &row.value)? {
            evicted += 1;
        }
    }
    Ok(evicted)
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConsumedNumber {
    pub block_id: String,
    pub number: i64,
    pub remaining: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockExhausted {
    pub remaining: i64,
}

pub fn consume_local_rx_number(
    conn: &mut SqliteConnection,
    device_id: &str,
) -> Result<ConsumedNumber, BlockExhausted> {
    let blocks = json_rows(
        conn,
        &format!(
            "SELECT json_object('id', id, 'startNumber', start_number, 'endNumber', end_number, \
             'consumed', consumed) AS row_json FROM rx_blocks WHERE device_id = '{}' AND status = 'active' AND consumed <= end_number - start_number ORDER BY allocated_at DESC LIMIT 1",
            device_id.replace('\'', "''")
        ),
    )
    .map_err(|_| BlockExhausted { remaining: 0 })?;
    let Some(block) = blocks.into_iter().next() else {
        return Err(BlockExhausted { remaining: 0 });
    };
    let start = block
        .get("startNumber")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let end = block.get("endNumber").and_then(Value::as_i64).unwrap_or(0);
    let consumed = block.get("consumed").and_then(Value::as_i64).unwrap_or(0);
    let capacity = end - start + 1;
    let remaining = (capacity - consumed).max(0);
    if remaining <= 0 {
        let _ = exec(
            conn,
            &format!(
                "UPDATE rx_blocks SET status = 'exhausted', updated_at = '{}' WHERE id = '{}'",
                now(),
                block.get("id").and_then(Value::as_str).unwrap_or_default()
            ),
        );
        return Err(BlockExhausted { remaining: 0 });
    }
    let number = start + consumed;
    let new_consumed = consumed + 1;
    let status = if new_consumed >= capacity {
        "exhausted"
    } else {
        "active"
    };
    let at = now();
    diesel::sql_query(
        "UPDATE rx_blocks SET consumed = :consumed, status = :status, updated_at = :updated_at WHERE id = :id",
    )
    .bind::<Integer, _>(&(new_consumed as i32))
    .bind::<Text, _>(status)
    .bind::<Text, _>(at.as_str())
    .bind::<Text, _>(block.get("id").and_then(Value::as_str).unwrap_or_default())
    .execute(conn)
    .map_err(|_| BlockExhausted { remaining: (capacity - new_consumed).max(0) })?;
    Ok(ConsumedNumber {
        block_id: block
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        number,
        remaining: (capacity - new_consumed).max(0),
    })
}

pub fn active_block_remaining(conn: &mut SqliteConnection, device_id: &str) -> Result<i64, String> {
    let rows = json_rows(
        conn,
        &format!(
            "SELECT json_object('startNumber', start_number, 'endNumber', end_number, 'consumed', consumed, 'status', status) AS row_json \
             FROM rx_blocks WHERE device_id = '{}' ORDER BY allocated_at DESC LIMIT 1",
            device_id.replace('\'', "''")
        ),
    )?;
    let Some(block) = rows.into_iter().next() else {
        return Ok(0);
    };
    if block.get("status").and_then(Value::as_str) != Some("active") {
        return Ok(0);
    }
    let start = block
        .get("startNumber")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let end = block.get("endNumber").and_then(Value::as_i64).unwrap_or(0);
    let consumed = block.get("consumed").and_then(Value::as_i64).unwrap_or(0);
    Ok((end - start + 1 - consumed).max(0))
}

pub fn asset_referenced(conn: &mut SqliteConnection, asset_id: &str) -> Result<bool, String> {
    let rows = diesel::sql_query(&format!(
        "SELECT COUNT(*) AS value FROM template_version_assets WHERE asset_id = '{}'",
        asset_id.replace('\'', "''")
    ))
    .load::<IntRow>(conn)
    .map_err(|error| error.to_string())?;
    Ok(rows.iter().any(|row| row.value > 0))
}

pub const MAX_ASSET_BYTES: usize = 10 * 1024 * 1024;

pub fn cache_image(
    conn: &mut SqliteConnection,
    asset_id: &str,
    bytes: &[u8],
    expected_sha256: &str,
) -> Result<PathBuf, String> {
    if bytes.len() > MAX_ASSET_BYTES {
        return Err("La imagen supera el límite de 10 MB.".to_owned());
    }
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hex::encode(hasher.finalize());
    if !expected_sha256.is_empty() && digest != expected_sha256 {
        return Err("SHA-256 de la imagen no coincide.".to_owned());
    }
    let dir = cache_dir();
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    let path = dir.join(format!("{asset_id}.bin"));
    std::fs::write(&path, bytes).map_err(|error| error.to_string())?;
    let at = now();
    diesel::sql_query(
        "INSERT INTO asset_cache (asset_id, sha256, bytes, local_path, fetched_at) \
         VALUES (:asset_id, :sha256, :bytes, :local_path, :fetched_at) \
         ON CONFLICT(asset_id) DO UPDATE SET sha256 = excluded.sha256, bytes = excluded.bytes, local_path = excluded.local_path, fetched_at = excluded.fetched_at",
    )
    .bind::<Text, _>(asset_id)
    .bind::<Text, _>(digest.as_str())
    .bind::<Integer, _>(&(bytes.len() as i32))
    .bind::<Text, _>(path.to_string_lossy().as_ref())
    .bind::<Text, _>(at.as_str())
    .execute(conn)
    .map(|_| ())
    .map_err(|error| error.to_string())?;
    Ok(path)
}

pub fn evict_asset_cache(conn: &mut SqliteConnection, asset_id: &str) -> Result<bool, String> {
    if asset_referenced(conn, asset_id)? {
        return Ok(false);
    }
    let rows = diesel::sql_query(&format!(
        "SELECT local_path AS value FROM asset_cache WHERE asset_id = '{}'",
        asset_id.replace('\'', "''")
    ))
    .load::<OptRow>(conn)
    .map_err(|error| error.to_string())?;
    if let Some(path) = rows.into_iter().next().and_then(|row| row.value) {
        let _ = std::fs::remove_file(path);
    }
    diesel::sql_query(&format!(
        "DELETE FROM asset_cache WHERE asset_id = '{}'",
        asset_id.replace('\'', "''")
    ))
    .execute(conn)
    .map_err(|error| error.to_string())?;
    Ok(true)
}

pub fn cached_asset_path(asset_id: &str) -> Option<PathBuf> {
    let path = cache_dir().join(format!("{asset_id}.bin"));
    if path.exists() {
        Some(path)
    } else {
        None
    }
}
pub fn cached_asset_ids(conn: &mut SqliteConnection) -> Result<Vec<String>, String> {
    let rows = diesel::sql_query("SELECT asset_id AS value FROM asset_cache")
        .load::<TextRow>(conn)
        .map_err(|error| error.to_string())?;
    Ok(rows.into_iter().map(|row| row.value).collect())
}

pub fn doc_asset_ids(doc_json: &str) -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();
    if let Ok(document) = crate::forms::parse_document(doc_json) {
        for image in document.images {
            if let Some(asset_id) = image.asset_id {
                ids.push(asset_id);
            }
        }
        if let Some(logo) = document.letterhead.logo_asset_id {
            ids.push(logo);
        }
    }
    ids.sort();
    ids.dedup();
    ids
}

pub fn cached_doc_assets(doc_json: &str) -> Vec<(String, Vec<u8>)> {
    let mut assets = Vec::new();
    for asset_id in doc_asset_ids(doc_json) {
        if let Some(path) = cached_asset_path(&asset_id) {
            if let Ok(bytes) = std::fs::read(path) {
                assets.push((asset_id, bytes));
            }
        }
    }
    assets
}

#[derive(Debug, Clone, PartialEq)]
pub struct OfflineSignResult {
    pub rx_number: i64,
    pub rx_block_id: String,
    pub outbox_id: String,
    pub remaining: i64,
}

pub fn offline_sign(
    conn: &mut SqliteConnection,
    prescription_id: &str,
    device_id: &str,
    doctor_id: &str,
    actor_id: &str,
) -> Result<OfflineSignResult, String> {
    immediate(conn, |conn| {
        let prescription = get_prescription(conn, prescription_id)?
            .ok_or_else(|| "Receta no encontrada en la caché local.".to_owned())?;
        if prescription.get("status").and_then(Value::as_str) != Some("draft") {
            return Err("Solo un borrador puede firmarse.".to_owned());
        }
        let consumed = consume_local_rx_number(conn, device_id).map_err(|exhausted| {
            format!(
                "Bloque de números agotado. Quedan {} números.",
                exhausted.remaining
            )
        })?;
        let at = now();
        let client_id = prescription
            .get("clientId")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        diesel::sql_query(
            "UPDATE prescriptions SET status = 'signed', doctor_id = :doctor_id, rx_number = :rx_number, rx_block_id = :rx_block_id, signed_at = :signed_at, synced_at = NULL, updated_at = :updated_at WHERE id = :id",
        )
        .bind::<Text, _>(doctor_id)
        .bind::<Integer, _>(&(consumed.number as i32))
        .bind::<Text, _>(consumed.block_id.as_str())
        .bind::<Text, _>(at.as_str())
        .bind::<Text, _>(at.as_str())
        .bind::<Text, _>(prescription_id)
        .execute(conn)
        .map_err(|error| error.to_string())?;
        let outbox_id = enqueue_outbox(
            conn,
            "sign_prescription",
            "prescription",
            prescription_id,
            &json!({
                "prescriptionId": prescription_id,
                "rxNumber": consumed.number,
                "rxBlockId": consumed.block_id,
                "clientId": client_id,
                "doctorId": doctor_id,
                "createdBy": actor_id,
                "signedAt": at,
            }),
        )?;
        Ok(OfflineSignResult {
            rx_number: consumed.number,
            rx_block_id: consumed.block_id.clone(),
            outbox_id,
            remaining: consumed.remaining,
        })
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCreatePatient {
    pub id: String,
    pub outbox_id: String,
}

pub fn create_local_patient(
    conn: &mut SqliteConnection,
    form: &Value,
    actor_id: &str,
) -> Result<LocalCreatePatient, String> {
    let id = Uuid::now_v7().to_string();
    let at = now();
    let clinic_id = form
        .get("clinicId")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let payload = json!({
        "clinicId": clinic_id,
        "fullName": form.get("fullName").and_then(Value::as_str).unwrap_or_default(),
        "nationalId": form.get("nationalId").and_then(Value::as_str).unwrap_or_default(),
        "birthDate": form.get("birthDate").and_then(Value::as_str).unwrap_or_default(),
        "sex": form.get("sex").and_then(Value::as_str).unwrap_or_default(),
        "phone": form.get("phone").and_then(Value::as_str).unwrap_or_default(),
        "notes": form.get("notes").and_then(Value::as_str).unwrap_or_default(),
        "clientId": id,
        "createdBy": actor_id,
    });
    diesel::sql_query(
        "INSERT INTO patients (id, clinic_id, full_name, national_id, birth_date, sex, phone, notes, client_id, deleted_at, created_at, updated_at, created_by) \
         VALUES (:id, :clinic_id, :full_name, :national_id, :birth_date, :sex, :phone, :notes, :client_id, NULL, :created_at, :updated_at, :created_by)",
    )
    .bind::<Text, _>(id.as_str())
    .bind::<Text, _>(clinic_id)
    .bind::<Text, _>(form.get("fullName").and_then(Value::as_str).unwrap_or_default())
    .bind::<Text, _>(form.get("nationalId").and_then(Value::as_str).unwrap_or_default())
    .bind::<Text, _>(form.get("birthDate").and_then(Value::as_str).unwrap_or_default())
    .bind::<Text, _>(form.get("sex").and_then(Value::as_str).unwrap_or_default())
    .bind::<Text, _>(form.get("phone").and_then(Value::as_str).unwrap_or_default())
    .bind::<Text, _>(form.get("notes").and_then(Value::as_str).unwrap_or_default())
    .bind::<Text, _>(id.as_str())
    .bind::<Text, _>(at.as_str())
    .bind::<Text, _>(at.as_str())
    .bind::<Text, _>(actor_id)
    .execute(conn)
    .map_err(|error| error.to_string())?;
    let outbox_id = enqueue_outbox(conn, "create_patient", "patient", &id, &payload)?;
    Ok(LocalCreatePatient { id, outbox_id })
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCreatePrescription {
    pub id: String,
    pub outbox_id: String,
}

pub fn create_local_prescription(
    conn: &mut SqliteConnection,
    patient_id: &str,
    template_version_id: &str,
    clinic_id: &str,
    doctor_id: Option<&str>,
    diagnosis: &str,
    validity: &str,
    instructions: &str,
    actor_id: &str,
) -> Result<LocalCreatePrescription, String> {
    let id = Uuid::now_v7().to_string();
    let at = now();
    let diagnosis = clean_local_text(diagnosis, 2000);
    let validity = clean_local_text(validity, 200);
    let instructions = clean_local_text(instructions, 8000);
    diesel::sql_query(
        "INSERT INTO prescriptions (id, patient_id, doctor_id, clinic_id, template_version_id, diagnosis, validity, instructions, status, rx_number, rx_block_id, signed_at, synced_at, client_id, created_at, updated_at, created_by) \
         VALUES (:id, :patient_id, :doctor_id, :clinic_id, :template_version_id, :diagnosis, :validity, :instructions, 'draft', NULL, NULL, NULL, NULL, :client_id, :created_at, :updated_at, :created_by)",
    )
    .bind::<Text, _>(id.as_str())
    .bind::<Text, _>(patient_id)
    .bind::<Nullable<Text>, _>(doctor_id)
    .bind::<Text, _>(clinic_id)
    .bind::<Text, _>(template_version_id)
    .bind::<Nullable<Text>, _>(diagnosis.as_deref())
    .bind::<Nullable<Text>, _>(validity.as_deref())
    .bind::<Nullable<Text>, _>(instructions.as_deref())
    .bind::<Text, _>(id.as_str())
    .bind::<Text, _>(at.as_str())
    .bind::<Text, _>(at.as_str())
    .bind::<Text, _>(actor_id)
    .execute(conn)
    .map_err(|error| error.to_string())?;
    let outbox_id = enqueue_outbox(
        conn,
        "create_prescription",
        "prescription",
        &id,
        &json!({
            "patientId": patient_id,
            "templateVersionId": template_version_id,
            "clinicId": clinic_id,
            "doctorId": doctor_id,
            "diagnosis": diagnosis,
            "validity": validity,
            "instructions": instructions,
            "clientId": id,
            "createdBy": actor_id,
        }),
    )?;
    Ok(LocalCreatePrescription { id, outbox_id })
}

fn clean_local_text(value: &str, limit: usize) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.chars().take(limit).collect())
    }
}

pub fn update_local_prescription(
    conn: &mut SqliteConnection,
    prescription_id: &str,
    diagnosis: &str,
    validity: &str,
    instructions: &str,
    actor_id: &str,
) -> Result<String, String> {
    let prescription = get_prescription(conn, prescription_id)?
        .ok_or_else(|| "Receta no encontrada en la caché local.".to_owned())?;
    if prescription.get("status").and_then(Value::as_str) != Some("draft") {
        return Err("La receta no se puede editar.".to_owned());
    }
    let diagnosis = clean_local_text(diagnosis, 2000);
    let validity = clean_local_text(validity, 200);
    let instructions = clean_local_text(instructions, 8000);
    let at = now();
    diesel::sql_query(
        "UPDATE prescriptions SET diagnosis = :diagnosis, validity = :validity, instructions = :instructions, synced_at = NULL, updated_at = :updated_at WHERE id = :id",
    )
    .bind::<Nullable<Text>, _>(diagnosis.as_deref())
    .bind::<Nullable<Text>, _>(validity.as_deref())
    .bind::<Nullable<Text>, _>(instructions.as_deref())
    .bind::<Text, _>(at.as_str())
    .bind::<Text, _>(prescription_id)
    .execute(conn)
    .map_err(|error| error.to_string())?;
    let client_id = prescription
        .get("clientId")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    enqueue_outbox(
        conn,
        "update_prescription",
        "prescription",
        prescription_id,
        &json!({
            "prescriptionId": prescription_id,
            "diagnosis": diagnosis,
            "validity": validity,
            "instructions": instructions,
            "clientId": client_id,
            "createdBy": actor_id,
        }),
    )
}

pub fn upsert_local_prescription_item(
    conn: &mut SqliteConnection,
    prescription_id: &str,
    item_id: Option<&str>,
    med_name: &str,
    dose: &str,
    route: &str,
    frequency: &str,
    duration: &str,
    quantity: &str,
    notes: &str,
    actor_id: &str,
) -> Result<String, String> {
    let prescription = get_prescription(conn, prescription_id)?
        .ok_or_else(|| "Receta no encontrada en la caché local.".to_owned())?;
    if prescription.get("status").and_then(Value::as_str) != Some("draft") {
        return Err("La receta no se puede editar.".to_owned());
    }
    let med_name = med_name.trim().to_owned();
    if med_name.is_empty() {
        return Err("Escribe el nombre del medicamento.".to_owned());
    }
    let dose = clean_local_text(dose, 500);
    let route = clean_local_text(route, 500);
    let frequency = clean_local_text(frequency, 500);
    let duration = clean_local_text(duration, 500);
    let quantity = clean_local_text(quantity, 500);
    let notes = clean_local_text(notes, 1000);
    let at = now();
    let existing = item_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    let id = match existing.clone() {
        Some(item_id) => {
            let count = diesel::sql_query(&format!(
                "SELECT COUNT(*) AS value FROM prescription_items WHERE id = '{}' AND prescription_id = '{}'",
                item_id.replace('\'', "''"),
                prescription_id.replace('\'', "''")
            ))
            .load::<IntRow>(conn)
            .map_err(|error| error.to_string())?;
            if count.iter().all(|row| row.value == 0) {
                return Err("Medicamento no encontrado.".to_owned());
            }
            diesel::sql_query(
                "UPDATE prescription_items SET med_name = :med_name, dose = :dose, route = :route, frequency = :frequency, duration = :duration, quantity = :quantity, notes = :notes, updated_at = :updated_at WHERE id = :id",
            )
            .bind::<Text, _>(med_name.as_str())
            .bind::<Nullable<Text>, _>(dose.as_deref())
            .bind::<Nullable<Text>, _>(route.as_deref())
            .bind::<Nullable<Text>, _>(frequency.as_deref())
            .bind::<Nullable<Text>, _>(duration.as_deref())
            .bind::<Nullable<Text>, _>(quantity.as_deref())
            .bind::<Nullable<Text>, _>(notes.as_deref())
            .bind::<Text, _>(at.as_str())
            .bind::<Text, _>(item_id.as_str())
            .execute(conn)
            .map_err(|error| error.to_string())?;
            item_id
        }
        None => {
            let generated = Uuid::now_v7().to_string();
            let position = diesel::sql_query(&format!(
                "SELECT COALESCE(MAX(position), 0) + 1 AS value FROM prescription_items WHERE prescription_id = '{}'",
                prescription_id.replace('\'', "''")
            ))
            .load::<IntRow>(conn)
            .map_err(|error| error.to_string())?;
            let next = position
                .into_iter()
                .next()
                .map(|row| row.value)
                .unwrap_or(1);
            diesel::sql_query(
                "INSERT INTO prescription_items (id, prescription_id, position, med_name, dose, route, frequency, duration, quantity, notes, created_at, updated_at, created_by) VALUES (:id, :prescription_id, :position, :med_name, :dose, :route, :frequency, :duration, :quantity, :notes, :created_at, :updated_at, :created_by)",
            )
            .bind::<Text, _>(generated.as_str())
            .bind::<Text, _>(prescription_id)
            .bind::<Integer, _>(&(next as i32))
            .bind::<Text, _>(med_name.as_str())
            .bind::<Nullable<Text>, _>(dose.as_deref())
            .bind::<Nullable<Text>, _>(route.as_deref())
            .bind::<Nullable<Text>, _>(frequency.as_deref())
            .bind::<Nullable<Text>, _>(duration.as_deref())
            .bind::<Nullable<Text>, _>(quantity.as_deref())
            .bind::<Nullable<Text>, _>(notes.as_deref())
            .bind::<Text, _>(at.as_str())
            .bind::<Text, _>(at.as_str())
            .bind::<Text, _>(actor_id)
            .execute(conn)
            .map_err(|error| error.to_string())?;
            generated
        }
    };
    enqueue_outbox(
        conn,
        "upsert_prescription_item",
        "prescription_item",
        &id,
        &json!({
            "prescriptionId": prescription_id,
            "itemId": id,
            "medName": med_name,
            "dose": dose,
            "route": route,
            "frequency": frequency,
            "duration": duration,
            "quantity": quantity,
            "notes": notes,
            "createdBy": actor_id,
        }),
    )?;
    Ok(id)
}

pub fn local_prescription_items(
    conn: &mut SqliteConnection,
    prescription_id: &str,
) -> Result<Vec<Value>, String> {
    json_rows(
        conn,
        &format!(
            "SELECT json_object('id', id, 'prescriptionId', prescription_id, 'position', position, \
             'medName', med_name, 'dose', dose, 'route', route, 'frequency', frequency, \
             'duration', duration, 'quantity', quantity, 'notes', notes) AS row_json \
             FROM prescription_items WHERE prescription_id = '{}' ORDER BY position",
            prescription_id.replace('\'', "''")
        ),
    )
}

pub fn cached_pinned_asset_ids(conn: &mut SqliteConnection) -> Result<Vec<String>, String> {
    let rows = diesel::sql_query("SELECT asset_id AS value FROM template_version_assets")
        .load::<TextRow>(conn)
        .map_err(|error| error.to_string())?;
    Ok(rows.into_iter().map(|row| row.value).collect())
}

pub fn upsert_pinned_assets(
    conn: &mut SqliteConnection,
    version_id: &str,
    asset_ids: &[String],
) -> Result<(), String> {
    for asset_id in asset_ids {
        diesel::sql_query(
            "INSERT OR IGNORE INTO template_version_assets (template_version_id, asset_id) VALUES (:version_id, :asset_id)",
        )
        .bind::<Text, _>(version_id)
        .bind::<Text, _>(asset_id.as_str())
        .execute(conn)
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}
#[derive(Debug, Clone, PartialEq)]
pub struct LocalCreateTemplate {
    pub id: String,
    pub outbox_id: String,
}

pub fn create_local_template(
    conn: &mut SqliteConnection,
    name: &str,
    owner_type: &str,
    owner_id: &str,
    actor_id: &str,
) -> Result<LocalCreateTemplate, String> {
    let name = name.trim().to_owned();
    if name.is_empty() {
        return Err("Escribe el nombre de la plantilla.".to_owned());
    }
    let id = Uuid::now_v7().to_string();
    let at = now();
    diesel::sql_query(
        "INSERT INTO templates (id, owner_type, owner_id, name, current_version_id, status, client_id, created_at, updated_at, created_by) \
         VALUES (:id, :owner_type, :owner_id, :name, NULL, 'draft', :client_id, :created_at, :updated_at, :created_by)",
    )
    .bind::<Text, _>(id.as_str())
    .bind::<Text, _>(owner_type)
    .bind::<Text, _>(owner_id)
    .bind::<Text, _>(name.as_str())
    .bind::<Text, _>(id.as_str())
    .bind::<Text, _>(at.as_str())
    .bind::<Text, _>(at.as_str())
    .bind::<Text, _>(actor_id)
    .execute(conn)
    .map_err(|error| error.to_string())?;
    let outbox_id = enqueue_outbox(
        conn,
        "create_template",
        "template",
        &id,
        &json!({
            "name": name,
            "ownerType": owner_type,
            "ownerId": owner_id,
            "clientId": id,
            "createdBy": actor_id,
        }),
    )?;
    Ok(LocalCreateTemplate { id, outbox_id })
}

pub fn publish_local_template(
    conn: &mut SqliteConnection,
    template_id: &str,
    version_id: &str,
    actor_id: &str,
) -> Result<String, String> {
    let version = get_template_version(conn, version_id)?
        .ok_or_else(|| "Versión no encontrada en la caché local.".to_owned())?;
    if version.get("templateId").and_then(Value::as_str) != Some(template_id) {
        return Err("La versión no pertenece a esta plantilla.".to_owned());
    }
    if version.get("status").and_then(Value::as_str) != Some("draft") {
        return Err("Solo un borrador puede publicarse.".to_owned());
    }
    let at = now();
    diesel::sql_query(
        "UPDATE template_versions SET status = 'published', updated_at = :updated_at WHERE id = :id",
    )
    .bind::<Text, _>(at.as_str())
    .bind::<Text, _>(version_id)
    .execute(conn)
    .map_err(|error| error.to_string())?;
    diesel::sql_query(
        "UPDATE templates SET current_version_id = :version_id, status = 'published', updated_at = :updated_at WHERE id = :id",
    )
    .bind::<Text, _>(version_id)
    .bind::<Text, _>(at.as_str())
    .bind::<Text, _>(template_id)
    .execute(conn)
    .map_err(|error| error.to_string())?;
    enqueue_outbox(
        conn,
        "publish_template_version",
        "template_version",
        version_id,
        &json!({
            "templateId": template_id,
            "versionId": version_id,
            "createdBy": actor_id,
        }),
    )
}

pub fn update_local_patient(
    conn: &mut SqliteConnection,
    patient_id: &str,
    form: &Value,
    actor_id: &str,
) -> Result<String, String> {
    let patient = list_patients(conn)?
        .into_iter()
        .find(|row| row.get("id").and_then(Value::as_str) == Some(patient_id))
        .ok_or_else(|| "Paciente no encontrado en la caché local.".to_owned())?;
    let clinic_id = patient
        .get("clinicId")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let national_id = form
        .get("nationalId")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let at = now();
    diesel::sql_query(
        "UPDATE patients SET full_name = :full_name, national_id = :national_id, birth_date = :birth_date, sex = :sex, phone = :phone, notes = :notes, updated_at = :updated_at WHERE id = :id",
    )
    .bind::<Text, _>(form.get("fullName").and_then(Value::as_str).unwrap_or_default())
    .bind::<Text, _>(national_id)
    .bind::<Text, _>(form.get("birthDate").and_then(Value::as_str).unwrap_or_default())
    .bind::<Text, _>(form.get("sex").and_then(Value::as_str).unwrap_or_default())
    .bind::<Text, _>(form.get("phone").and_then(Value::as_str).unwrap_or_default())
    .bind::<Text, _>(form.get("notes").and_then(Value::as_str).unwrap_or_default())
    .bind::<Text, _>(at.as_str())
    .bind::<Text, _>(patient_id)
    .execute(conn)
    .map_err(|error| error.to_string())?;
    enqueue_outbox(
        conn,
        "update_patient",
        "patient",
        patient_id,
        &json!({
            "patientId": patient_id,
            "clinicId": clinic_id,
            "fullName": form.get("fullName").and_then(Value::as_str).unwrap_or_default(),
            "nationalId": national_id,
            "birthDate": form.get("birthDate").and_then(Value::as_str).unwrap_or_default(),
            "sex": form.get("sex").and_then(Value::as_str).unwrap_or_default(),
            "phone": form.get("phone").and_then(Value::as_str).unwrap_or_default(),
            "notes": form.get("notes").and_then(Value::as_str).unwrap_or_default(),
            "createdBy": actor_id,
        }),
    )
}

pub fn offline_void(
    conn: &mut SqliteConnection,
    prescription_id: &str,
    reason: &str,
    actor_id: &str,
) -> Result<String, String> {
    let reason = reason.trim().to_owned();
    if reason.is_empty() {
        return Err("Escribe el motivo de anulación.".to_owned());
    }
    let prescription = get_prescription(conn, prescription_id)?
        .ok_or_else(|| "Receta no encontrada en la caché local.".to_owned())?;
    if prescription.get("status").and_then(Value::as_str) != Some("signed") {
        return Err("Solo una receta firmada puede anularse.".to_owned());
    }
    let at = now();
    diesel::sql_query(
        "UPDATE prescriptions SET status = 'voided', voided_at = :voided_at, void_reason = :void_reason, synced_at = NULL, updated_at = :updated_at WHERE id = :id",
    )
    .bind::<Text, _>(at.as_str())
    .bind::<Text, _>(reason.as_str())
    .bind::<Text, _>(at.as_str())
    .bind::<Text, _>(prescription_id)
    .execute(conn)
    .map_err(|error| error.to_string())?;
    enqueue_outbox(
        conn,
        "void_prescription",
        "prescription",
        prescription_id,
        &json!({
            "prescriptionId": prescription_id,
            "voidReason": reason,
            "createdBy": actor_id,
        }),
    )
}

#[derive(Debug, Clone, PartialEq)]
pub struct OfflineSupersedeResult {
    pub id: String,
    pub rx_number: i64,
    pub rx_block_id: String,
    pub outbox_id: String,
}

pub fn offline_supersede(
    conn: &mut SqliteConnection,
    prescription_id: &str,
    device_id: &str,
    doctor_id: &str,
    actor_id: &str,
) -> Result<OfflineSupersedeResult, String> {
    immediate(conn, |conn| {
        let original = get_prescription(conn, prescription_id)?
            .ok_or_else(|| "Receta no encontrada en la caché local.".to_owned())?;
        if original.get("status").and_then(Value::as_str) != Some("signed") {
            return Err("Solo una receta firmada puede reemplazarse.".to_owned());
        }
        let consumed = consume_local_rx_number(conn, device_id).map_err(|exhausted| {
            format!(
                "Bloque de números agotado. Quedan {} números.",
                exhausted.remaining
            )
        })?;
        let items = local_prescription_items(conn, prescription_id)?;
        let at = now();
        let id = Uuid::now_v7().to_string();
        diesel::sql_query(
        "INSERT INTO prescriptions (id, patient_id, doctor_id, clinic_id, template_version_id, diagnosis, validity, instructions, status, rx_number, rx_block_id, signed_at, synced_at, client_id, supersedes_id, created_at, updated_at, created_by) \
         VALUES (:id, :patient_id, :doctor_id, :clinic_id, :template_version_id, :diagnosis, :validity, :instructions, 'signed', :rx_number, :rx_block_id, :signed_at, NULL, :client_id, :supersedes_id, :created_at, :updated_at, :created_by)",
    )
    .bind::<Text, _>(id.as_str())
    .bind::<Text, _>(
        original
            .get("patientId")
            .and_then(Value::as_str)
            .unwrap_or_default(),
    )
    .bind::<Text, _>(doctor_id)
    .bind::<Text, _>(
        original
            .get("clinicId")
            .and_then(Value::as_str)
            .unwrap_or_default(),
    )
    .bind::<Text, _>(
        original
            .get("templateVersionId")
            .and_then(Value::as_str)
            .unwrap_or_default(),
    )
    .bind::<Text, _>(
        original
            .get("diagnosis")
            .and_then(Value::as_str)
            .unwrap_or_default(),
    )
    .bind::<Text, _>(
        original
            .get("validity")
            .and_then(Value::as_str)
            .unwrap_or_default(),
    )
    .bind::<Text, _>(
        original
            .get("instructions")
            .and_then(Value::as_str)
            .unwrap_or_default(),
    )
    .bind::<Integer, _>(&(consumed.number as i32))
    .bind::<Text, _>(consumed.block_id.as_str())
    .bind::<Text, _>(at.as_str())
    .bind::<Text, _>(id.as_str())
    .bind::<Text, _>(prescription_id)
    .bind::<Text, _>(at.as_str())
    .bind::<Text, _>(at.as_str())
    .bind::<Text, _>(actor_id)
    .execute(conn)
    .map_err(|error| error.to_string())?;
        for item in items {
            let item_id = Uuid::now_v7().to_string();
            diesel::sql_query(
            "INSERT INTO prescription_items (id, prescription_id, position, med_name, dose, route, frequency, duration, quantity, notes, created_at, updated_at, created_by) VALUES (:id, :prescription_id, :position, :med_name, :dose, :route, :frequency, :duration, :quantity, :notes, :created_at, :updated_at, :created_by)",
        )
        .bind::<Text, _>(item_id.as_str())
        .bind::<Text, _>(id.as_str())
        .bind::<Integer, _>(
            &(item.get("position").and_then(Value::as_i64).unwrap_or(1) as i32),
        )
        .bind::<Text, _>(
            item.get("medName")
                .and_then(Value::as_str)
                .unwrap_or_default(),
        )
        .bind::<Text, _>(item.get("dose").and_then(Value::as_str).unwrap_or_default())
        .bind::<Text, _>(item.get("route").and_then(Value::as_str).unwrap_or_default())
        .bind::<Text, _>(
            item.get("frequency")
                .and_then(Value::as_str)
                .unwrap_or_default(),
        )
        .bind::<Text, _>(
            item.get("duration")
                .and_then(Value::as_str)
                .unwrap_or_default(),
        )
        .bind::<Text, _>(
            item.get("quantity")
                .and_then(Value::as_str)
                .unwrap_or_default(),
        )
        .bind::<Text, _>(item.get("notes").and_then(Value::as_str).unwrap_or_default())
        .bind::<Text, _>(at.as_str())
        .bind::<Text, _>(at.as_str())
        .bind::<Text, _>(actor_id)
        .execute(conn)
        .map_err(|error| error.to_string())?;
        }
        diesel::sql_query(
        "UPDATE prescriptions SET status = 'superseded', synced_at = NULL, updated_at = :updated_at WHERE id = :id",
    )
    .bind::<Text, _>(at.as_str())
    .bind::<Text, _>(prescription_id)
    .execute(conn)
    .map_err(|error| error.to_string())?;
        let outbox_id = enqueue_outbox(
            conn,
            "supersede_prescription",
            "prescription",
            &id,
            &json!({
                "prescriptionId": prescription_id,
                "newPrescriptionId": id,
                "rxNumber": consumed.number,
                "rxBlockId": consumed.block_id,
                "clientId": id,
                "doctorId": doctor_id,
                "createdBy": actor_id,
                "signedAt": at,
            }),
        )?;
        Ok(OfflineSupersedeResult {
            id,
            rx_number: consumed.number,
            rx_block_id: consumed.block_id,
            outbox_id,
        })
    })
}

pub fn date_now() -> NaiveDate {
    Utc::now().date_naive()
}
