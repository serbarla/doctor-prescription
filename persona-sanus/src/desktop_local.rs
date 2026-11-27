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
  medical_record_no TEXT,
  allergies TEXT,
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
  recetario_type TEXT NOT NULL DEFAULT 'ambulatorio',
  validity_days INTEGER,
  warning_signs TEXT,
  instructions TEXT,
  signed_at TEXT,
  issued_snapshot_json TEXT,
  verification_json TEXT,
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
  quantity_number INTEGER,
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

#[cfg(test)]
pub(crate) fn initialize_test_db(conn: &mut SqliteConnection) {
    use diesel::connection::SimpleConnection;
    conn.batch_execute(SCHEMA).unwrap();
}

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

fn migrate_issued_snapshot(conn: &mut SqliteConnection) -> Result<(), String> {
    immediate(conn, |conn| {
        for column in ["issued_snapshot_json", "verification_json"] {
            let present = diesel::sql_query(format!(
                "SELECT COUNT(*) AS value FROM pragma_table_info('prescriptions') WHERE name = '{column}'"
            ))
            .load::<IntRow>(conn)
            .map_err(|error| error.to_string())?
            .iter()
            .any(|row| row.value > 0);
            if !present {
                exec(conn, &format!("ALTER TABLE prescriptions ADD COLUMN {column} TEXT"))?;
            }
        }
        Ok(())
    })
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
    let _ = conn.batch_execute(
        "ALTER TABLE patients ADD COLUMN medical_record_no TEXT; \
         ALTER TABLE patients ADD COLUMN allergies TEXT; \
         ALTER TABLE prescriptions ADD COLUMN recetario_type TEXT NOT NULL DEFAULT 'ambulatorio'; \
         ALTER TABLE prescriptions ADD COLUMN validity_days INTEGER; \
         ALTER TABLE prescriptions ADD COLUMN warning_signs TEXT; \
         ALTER TABLE prescription_items ADD COLUMN quantity_number INTEGER;",
    );
    if outbox_needs_migration(&mut conn)? {
        migrate_outbox(&mut conn)?;
    }
    migrate_issued_snapshot(&mut conn)?;
    exec(&mut conn, "PRAGMA user_version=2")?;
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
    } else {
        delete_meta(&mut conn, "session_profile")?;
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
            "medical_record_no",
            "allergies",
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
    let identity = local_identity(conn)?.ok_or_else(|| "Sesión local no disponible.".to_owned())?;
    let row = json!({"clinicId": prescription.get("clinic_id"), "doctorId": prescription.get("doctor_id"), "status": prescription.get("status")});
    if !permitted(&identity, &row, "read") {
        return Err("Receta no encontrada en la caché local.".to_owned());
    }
    let id = prescription.get("id").and_then(Value::as_str).ok_or("Receta inválida.")?;
    if has_pending_outbox(conn, id)? {
        return Err("La receta tiene cambios locales pendientes.".to_owned());
    }
    let mut prescription = prescription.clone();
    let existing = diesel::sql_query("SELECT issued_snapshot_json AS value FROM prescriptions WHERE id = ?")
        .bind::<Text, _>(id).load::<OptRow>(conn)
        .map_err(|error| error.to_string())?
        .into_iter().next().and_then(|row| row.value);
    let incoming = prescription.get("issued_snapshot").filter(|value| !value.is_null()).cloned();
    let parsed = incoming.as_ref().map(|snapshot| serde_json::from_value::<crate::api::IssuedSnapshot>(snapshot.clone()))
        .transpose().map_err(|_| "La instantánea de emisión es inválida.".to_owned())?;
    if let Some(snapshot) = &parsed {
        snapshot.validate_structure()?;
        if snapshot.prescription_id != id
            || snapshot.clinic_id != prescription.get("clinic_id").and_then(Value::as_str).unwrap_or_default()
            || snapshot.patient_id != prescription.get("patient_id").and_then(Value::as_str).unwrap_or_default()
        {
            return Err("La instantánea no corresponde a la receta.".to_owned());
        }
    }
    if let Some(existing) = existing {
        let original: crate::api::IssuedSnapshot = serde_json::from_str(&existing)
            .map_err(|_| "La instantánea de emisión es inválida.".to_owned())?;
        if parsed.as_ref() != Some(&original) {
            return Err("La instantánea emitida no coincide; conserva la receta pendiente para resolver el conflicto.".to_owned());
        }
    }
    prescription["issued_snapshot_json"] = incoming.unwrap_or(Value::Null);
    prescription["verification_json"] = prescription.get("verification").cloned().unwrap_or(Value::Null);
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
            "recetario_type",
            "validity_days",
            "warning_signs",
            "instructions",
            "signed_at",
            "issued_snapshot_json",
            "verification_json",
            "synced_at",
            "voided_at",
            "void_reason",
            "client_id",
            "supersedes_id",
            "created_at",
            "updated_at",
            "created_by",
        ],
        &prescription,
    )?;
    let scope = local_scope(conn)?.ok_or("Sesión local no disponible.")?;
    set_meta(conn, &format!("prescription_scope:{id}"), &scope)
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
            "quantity_number",
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
    let rows = diesel::sql_query(
        "SELECT COUNT(*) AS value FROM outbox WHERE status IN ('pending', 'failed') AND \
         (entity_id = ? OR json_extract(payload_json, '$.prescriptionId') = ?)"
    )
    .bind::<Text, _>(entity_id)
    .bind::<Text, _>(entity_id)
    .load::<IntRow>(conn)
    .map_err(|error| error.to_string())?;
    Ok(rows.iter().any(|row| row.value > 0))
}

pub fn list_patients(conn: &mut SqliteConnection) -> Result<Vec<Value>, String> {
    json_rows(
        conn,
        "SELECT json_object('id', id, 'clinicId', clinic_id, 'fullName', full_name, \
         'nationalId', national_id, 'birthDate', birth_date, 'sex', sex, 'phone', phone, \
         'notes', notes, 'medicalRecordNo', medical_record_no, 'allergies', allergies, \
         'createdAt', created_at, 'updatedAt', updated_at, 'createdBy', created_by) AS row_json \
         FROM patients WHERE deleted_at IS NULL ORDER BY full_name",
    )
}

pub fn get_clinic(conn: &mut SqliteConnection, id: &str) -> Result<Option<Value>, String> {
    let rows = json_rows(
        conn,
        &format!(
            "SELECT json_object('name', name, 'address', address, 'phone', phone) AS row_json \
             FROM clinics WHERE id = '{}'",
            id.replace('\'', "''")
        ),
    )?;
    Ok(rows.into_iter().next())
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

fn local_identity(conn: &mut SqliteConnection) -> Result<Option<Value>, String> {
    if get_meta(conn, "session_token")?.filter(|s| !s.is_empty()).is_none()
        || get_meta(conn, "session_user")?.and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .and_then(|u| u.get("id").and_then(Value::as_str).map(str::to_owned))
            .filter(|s| !s.is_empty()).is_none()
    {
        return Ok(None);
    }
    let expires = get_meta(conn, "session_expires_at")?.unwrap_or_default();
    if expires.is_empty() || DateTime::parse_from_rfc3339(&expires)
        .map(|v| v.timestamp() <= Utc::now().timestamp()).unwrap_or(true)
    {
        return Ok(None);
    }
    Ok(get_meta(conn, "session_profile")?.and_then(|s| serde_json::from_str(&s).ok()))
}

fn local_scope(conn: &mut SqliteConnection) -> Result<Option<String>, String> {
    let Some(identity) = local_identity(conn)? else { return Ok(None); };
    let Some(user) = get_meta(conn, "session_user")?.and_then(|s| serde_json::from_str::<Value>(&s).ok()) else { return Ok(None); };
    Ok(Some(json!([user.get("id"), identity.pointer("/profile/id"), identity.pointer("/clinic/id"), identity.get("role"), identity.pointer("/doctor/id")]).to_string()))
}

pub(crate) fn authorize_create_replay(conn: &mut SqliteConnection, payload: &Value) -> Result<(), String> {
    let identity = local_identity(conn)?.ok_or("Sesión local no disponible.")?;
    let actor = payload.get("createdBy").and_then(Value::as_str);
    let doctor = payload.get("doctorId").and_then(Value::as_str);
    let clinic = payload.get("clinicId").and_then(Value::as_str);
    let recorded_scope = payload.get("clientId").and_then(Value::as_str)
        .map(|id| get_meta(conn, &format!("prescription_scope:{id}")))
        .transpose()?.flatten();
    if recorded_scope.is_some() && recorded_scope != local_scope(conn)? {
        return Err("La operación pertenece a otra identidad o función.".to_owned());
    }
    let valid = actor.is_some()
        && actor == identity.pointer("/profile/id").and_then(Value::as_str)
        && clinic.is_some()
        && clinic == identity.pointer("/clinic/id").and_then(Value::as_str)
        && match identity.get("role").and_then(Value::as_str) {
            Some("doctor") => doctor.is_some() && doctor == identity.pointer("/doctor/id").and_then(Value::as_str),
            Some("nursing") => doctor.is_none(),
            _ => false,
        };
    if valid { Ok(()) } else { Err("La operación pertenece a otra identidad o función.".to_owned()) }
}

fn permitted(identity: &Value, row: &Value, action: &str) -> bool {
    let field = |v: &Value, name: &str| v.get(name).and_then(Value::as_str).unwrap_or_default().to_owned();
    let clinic = identity.pointer("/clinic/id").and_then(Value::as_str).unwrap_or_default();
    let actor = identity.pointer("/profile/id").and_then(Value::as_str).unwrap_or_default();
    let role = identity.get("role").and_then(Value::as_str).unwrap_or_default();
    let doctor = identity.pointer("/doctor/id").and_then(Value::as_str).unwrap_or_default();
    let status = field(row, "status");
    let owner = row.get("doctorId").and_then(Value::as_str);
    if clinic.is_empty() || actor.is_empty() || field(row, "clinicId") != clinic {
        return false;
    }
    let read = match role {
        "doctor" => !doctor.is_empty() && (owner == Some(doctor) || (owner.is_none() && status == "draft")),
        "nursing" | "administrator" => true,
        "secretary" => matches!(status.as_str(), "signed" | "voided" | "superseded"),
        _ => false,
    };
    read && match action {
        "read" => true,
        "edit" => status == "draft" && matches!(role, "doctor" | "nursing"),
        "sign" => status == "draft" && role == "doctor",
        "transition" => status == "signed" && role == "doctor",
        _ => false,
    }
}

fn authorize_action(conn: &mut SqliteConnection, row: &Value, action: &str, actor_id: &str, doctor_id: Option<&str>) -> Result<(), String> {
    let identity = local_identity(conn)?.ok_or_else(|| "Receta no encontrada en la caché local.".to_owned())?;
    if identity.pointer("/profile/id").and_then(Value::as_str) != Some(actor_id)
        || doctor_id.is_some_and(|id| identity.pointer("/doctor/id").and_then(Value::as_str) != Some(id))
        || !permitted(&identity, row, action)
    {
        return Err("Receta no encontrada en la caché local.".to_owned());
    }
    Ok(())
}

pub fn list_prescriptions(conn: &mut SqliteConnection, status: &str) -> Result<Vec<Value>, String> {
    let filter = if status.is_empty() {
        String::new()
    } else if status == "pending_sync" {
        " WHERE EXISTS (SELECT 1 FROM outbox WHERE status = 'pending' AND \
         (entity_id = prescriptions.id OR json_extract(payload_json, '$.prescriptionId') = prescriptions.id))"
            .to_owned()
    } else if status == "failed" {
        " WHERE EXISTS (SELECT 1 FROM outbox WHERE status = 'failed' AND \
         (entity_id = prescriptions.id OR json_extract(payload_json, '$.prescriptionId') = prescriptions.id))"
            .to_owned()
    } else {
        format!(" WHERE status = '{}'", status.replace('\'', "''"))
    };
    let Some(identity) = local_identity(conn)? else { return Ok(Vec::new()); };
    let rows = json_rows(
        conn,
        &format!(
            "SELECT json_object('id', id, 'patientId', patient_id, 'patientName', \
             COALESCE((SELECT full_name FROM patients WHERE patients.id = prescriptions.patient_id), 'Paciente'), \
             'doctorId', doctor_id, 'clinicId', clinic_id, 'templateVersionId', template_version_id, \
             'status', status, 'rxNumber', rx_number, 'rxBlockId', rx_block_id, \
             'diagnosis', diagnosis, 'validity', validity, 'recetarioType', recetario_type, 'validityDays', validity_days, 'warningSigns', warning_signs, 'instructions', instructions, \
             'signedAt', signed_at, 'issuedSnapshot', json(issued_snapshot_json), 'verification', json(verification_json), 'syncedAt', synced_at, 'voidedAt', voided_at, 'voidReason', void_reason, \
             'clientId', client_id, 'supersedesId', supersedes_id, 'createdAt', created_at, \
             'updatedAt', updated_at, 'createdBy', created_by, 'items', json_array()) AS row_json \
             FROM prescriptions{filter} ORDER BY created_at DESC"
        ),
    )?;
    let Some(scope) = local_scope(conn)? else { return Ok(Vec::new()); };
    let mut allowed = Vec::new();
    for row in rows {
        let id = row.get("id").and_then(Value::as_str).unwrap_or_default();
        if permitted(&identity, &row, "read")
            && get_meta(conn, &format!("prescription_scope:{id}"))?.as_deref() == Some(scope.as_str())
        {
            allowed.push(row);
        }
    }
    Ok(allowed)
}

pub fn get_prescription(conn: &mut SqliteConnection, id: &str) -> Result<Option<Value>, String> {
    let rows = list_prescriptions(conn, "")?;
    Ok(rows
        .into_iter()
        .find(|row| row.get("id").and_then(Value::as_str) == Some(id)))
}

pub fn render_prescription_pdf(conn: &mut SqliteConnection, id: &str, input: &crate::pdf::PdfInput) -> Result<Vec<u8>, String> {
    let found = get_prescription(conn, id)?
        .ok_or_else(|| "Receta no encontrada en la caché local.".to_owned())?;
    if found.get("status").and_then(Value::as_str) == Some("draft") {
        return Err("Firma la receta antes de generar el PDF.".to_owned());
    }
    let frozen = issued_snapshot_pdf_input(conn, &found, &input.title)?;
    crate::pdf::render(&frozen)
}

const HISTORY_UNAVAILABLE: &str = "El documento original no está disponible: esta receta se emitió antes del registro de instantáneas. Consulta el ejemplar impreso o solicita una nueva receta; los datos y la verificación siguen disponibles.";

fn issued_snapshot_from_row(found: &Value) -> Result<crate::api::IssuedSnapshot, String> {
    let snapshot = found.get("issuedSnapshot").filter(|value| !value.is_null())
        .ok_or_else(|| HISTORY_UNAVAILABLE.to_owned())?;
    let snapshot: crate::api::IssuedSnapshot = serde_json::from_value(snapshot.clone())
        .map_err(|_| "La instantánea de emisión es inválida.".to_owned())?;
    snapshot.validate_structure()?;
    Ok(snapshot)
}

fn issued_snapshot_assets(conn: &mut SqliteConnection, snapshot: &crate::api::IssuedSnapshot) -> Result<Vec<(String, Vec<u8>)>, String> {
    let mut assets = Vec::with_capacity(snapshot.assets.len());
    for asset in &snapshot.assets {
        let path = diesel::sql_query("SELECT local_path AS value FROM asset_cache WHERE asset_id = ? AND sha256 = ?")
            .bind::<Text, _>(&asset.id).bind::<Text, _>(&asset.sha256)
            .load::<TextRow>(conn).map_err(|error| error.to_string())?
            .into_iter().next().map(|row| row.value)
            .ok_or("Falta una imagen fijada. Conéctate para descargarla antes de emitir o reimprimir.")?;
        let bytes = std::fs::read(path).map_err(|_| "Falta una imagen fijada. Conéctate para descargarla antes de emitir o reimprimir.")?;
        if hex::encode(Sha256::digest(&bytes)) != asset.sha256 {
            return Err("La imagen fijada no coincide con la emisión. Descárgala de nuevo.".to_owned());
        }
        assets.push((asset.id.clone(), bytes));
    }
    Ok(assets)
}

fn issued_snapshot_pdf_input(conn: &mut SqliteConnection, found: &Value, title: &str) -> Result<crate::pdf::PdfInput, String> {
    let snapshot = issued_snapshot_from_row(found)?;
    let code = found.pointer("/verification/code").and_then(Value::as_str);
    let mode = match code {
        Some(code) => crate::pdf::RenderMode::Final {
            code: code.to_owned(),
            verify_url: format!("{}/verify/{code}", crate::http::api_base()),
        },
        None => crate::pdf::RenderMode::Provisional,
    };
    Ok(crate::pdf::PdfInput {
        assets: issued_snapshot_assets(conn, &snapshot)?,
        margin_mm: crate::forms::template_theme_margin(snapshot.theme.as_deref()),
        doc_json: snapshot.doc_json,
        values: snapshot.variables,
        medications: snapshot.medications,
        rx_number: Some(snapshot.rx_number),
        issued_at: Some(snapshot.issued_at),
        mode,
        title: title.to_owned(),
    })
}

pub(crate) fn issued_snapshot_preview(conn: &mut SqliteConnection, found: &Value) -> Result<crate::api::PrescriptionPreview, String> {
    use base64::Engine;
    let snapshot = issued_snapshot_from_row(found)?;
    let input = issued_snapshot_pdf_input(conn, found, "Receta")?;
    let data_url = |kind: &str, bytes: &[u8]| format!("data:{kind};base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes));
    let (verification_code, qr_data_url) = match &input.mode {
        crate::pdf::RenderMode::Final { code, verify_url } => (Some(code.clone()), crate::pdf::qr_png(verify_url).map(|png| data_url("image/png", &png))),
        crate::pdf::RenderMode::Provisional => (None, None),
    };
    let image_sources = input.assets.iter().map(|(id, bytes)| crate::api::PreviewImageSource {
        asset_id: id.clone(),
        data_url: data_url(if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) { "image/jpeg" } else { "image/png" }, bytes),
    }).collect();
    Ok(crate::api::PrescriptionPreview {
        id: snapshot.prescription_id,
        status: found.get("status").and_then(Value::as_str).unwrap_or("signed").to_owned(),
        rx_number: input.rx_number,
        provisional: verification_code.is_none(),
        verification_code,
        issued_at: snapshot.issued_at,
        doc_json: input.doc_json,
        margin_mm: input.margin_mm,
        variables: input.values.into_iter().map(|(key, value)| crate::api::PreviewVariable { key, value }).collect(),
        medications: input.medications,
        qr_data_url,
        image_sources,
    })
}

pub(crate) fn apply_issued_snapshot_reconciliation(conn: &mut SqliteConnection, payload: &Value, response: &Value) -> Result<(), String> {
    immediate(conn, |conn| {
        serde_json::from_value::<crate::api::Prescription>(response.clone())
            .map_err(|_| "La respuesta de emisión está incompleta.".to_owned())?;
        let snapshot: crate::api::IssuedSnapshot = serde_json::from_value(
            payload.get("issuedSnapshot").cloned().ok_or(HISTORY_UNAVAILABLE)?
        ).map_err(|_| HISTORY_UNAVAILABLE.to_owned())?;
        snapshot.validate_structure()?;
        let found = get_prescription(conn, &snapshot.prescription_id)?.ok_or("Receta no encontrada en la caché local.")?;
        if issued_snapshot_from_row(&found)? != snapshot
            || response.get("issuedSnapshot") != payload.get("issuedSnapshot")
            || response.get("id").and_then(Value::as_str) != Some(snapshot.prescription_id.as_str())
            || response.get("rxNumber").and_then(Value::as_i64) != Some(i64::from(snapshot.rx_number))
            || response.get("rxBlockId").and_then(Value::as_str) != Some(snapshot.rx_block_id.as_str())
            || response.get("signedAt").and_then(Value::as_str) != Some(snapshot.issued_at.as_str())
            || response.get("clinicId").and_then(Value::as_str) != Some(snapshot.clinic_id.as_str())
            || response.get("patientId").and_then(Value::as_str) != Some(snapshot.patient_id.as_str())
            || response.get("doctorId").and_then(Value::as_str) != Some(snapshot.doctor_id.as_str())
            || response.get("clientId").and_then(Value::as_str) != Some(snapshot.client_id.as_str())
            || response.get("templateVersionId").and_then(Value::as_str) != Some(snapshot.template_version_id.as_str())
            || !matches!(response.get("status").and_then(Value::as_str), Some("signed" | "voided" | "superseded"))
        {
            return Err("La respuesta no coincide con la emisión pendiente. Conserva la receta para resolver el conflicto.".to_owned());
        }
        let verification = response.get("verification").filter(|value| value.get("code").and_then(Value::as_str).is_some_and(|code| !code.is_empty()))
            .ok_or("La respuesta de emisión no contiene un código de verificación.")?;
        if let Some(original) = found.pointer("/verification/code").and_then(Value::as_str) {
            if verification.get("code").and_then(Value::as_str) != Some(original) {
                return Err("El código de verificación no coincide con la emisión reconciliada.".to_owned());
            }
        }
        diesel::sql_query("UPDATE prescriptions SET verification_json = ?, synced_at = ? WHERE id = ?")
            .bind::<Text, _>(verification.to_string())
            .bind::<Text, _>(now())
            .bind::<Text, _>(&snapshot.prescription_id)
            .execute(conn).map_err(|error| error.to_string())?;
        Ok(())
    })
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
    if has_pending_outbox(conn, row_id)? {
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
    let rows = diesel::sql_query(
        "SELECT (SELECT COUNT(*) FROM template_version_assets WHERE asset_id = ?) + \
         (SELECT COUNT(*) FROM prescriptions, json_each(issued_snapshot_json, '$.assets') WHERE json_extract(json_each.value, '$.id') = ?) AS value"
    )
    .bind::<Text, _>(asset_id)
    .bind::<Text, _>(asset_id)
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


#[derive(Debug, Clone, PartialEq)]
pub struct OfflineSignResult {
    pub rx_number: i64,
    pub rx_block_id: String,
    pub outbox_id: String,
    pub remaining: i64,
}

fn capture_issued_snapshot(
    conn: &mut SqliteConnection,
    prescription: &Value,
    source_id: &str,
    device_id: &str,
    doctor_id: &str,
    actor_id: &str,
    at: &str,
) -> Result<crate::api::IssuedSnapshot, String> {
    let required = |key: &str| prescription.get(key).and_then(Value::as_str)
        .filter(|value| !value.is_empty()).map(str::to_owned)
        .ok_or_else(|| "Faltan datos de la receta. Conéctate antes de emitir.".to_owned());
    let id = required("id")?;
    let clinic_id = required("clinicId")?;
    let patient_id = required("patientId")?;
    let version_id = required("templateVersionId")?;
    let identity = local_identity(conn)?.ok_or("Sesión local no disponible.")?;
    let profile: crate::api::Me = serde_json::from_value(identity)
        .map_err(|_| "El perfil médico local está incompleto. Conéctate antes de emitir.")?;
    if profile.profile.id != actor_id
        || profile.profile.display_name.trim().is_empty()
        || profile.doctor.as_ref().map(|doctor| doctor.id.as_str()) != Some(doctor_id)
        || profile.clinic.as_ref().map(|clinic| clinic.id.as_str()) != Some(clinic_id.as_str())
    {
        return Err("El perfil médico local no corresponde a la emisión.".to_owned());
    }
    let patient = list_patients(conn)?.into_iter()
        .find(|patient| patient.get("id").and_then(Value::as_str) == Some(patient_id.as_str()))
        .filter(|patient| patient.get("clinicId").and_then(Value::as_str) == Some(clinic_id.as_str()))
        .ok_or("Faltan los datos del paciente. Conéctate antes de emitir.")?;
    if patient.get("fullName").and_then(Value::as_str).is_none_or(|name| name.trim().is_empty()) {
        return Err("Falta el nombre del paciente.".to_owned());
    }
    let clinic = get_clinic(conn, &clinic_id)?
        .ok_or("Faltan los datos de la clínica. Conéctate antes de emitir.")?;
    let version = get_template_version(conn, &version_id)?
        .ok_or("Falta la plantilla fijada. Conéctate antes de emitir.")?;
    if version.get("status").and_then(Value::as_str) != Some("published") {
        return Err("La plantilla debe estar publicada antes de emitir.".to_owned());
    }
    let doc_json = version.get("docJson").and_then(Value::as_str)
        .ok_or("La plantilla local no contiene un documento válido.")?.to_owned();
    crate::forms::parse_document(&doc_json)?;
    let theme = version.get("theme").and_then(Value::as_str).map(str::to_owned);
    let mut assets = Vec::new();
    let asset_ids = doc_asset_ids(&doc_json);
    for asset_id in &asset_ids {
        let rows = diesel::sql_query(
            "SELECT sha256 AS value FROM assets WHERE id = ? AND clinic_id = ? AND status = 'ready' AND deleted_at IS NULL AND EXISTS (SELECT 1 FROM template_version_assets WHERE template_version_id = ? AND asset_id = assets.id)"
        ).bind::<Text, _>(asset_id).bind::<Text, _>(&clinic_id).bind::<Text, _>(&version_id)
            .load::<OptRow>(conn).map_err(|error| error.to_string())?;
        let sha256 = rows.into_iter().next().and_then(|row| row.value)
            .filter(|hash| hash.len() == 64).ok_or("Faltan imágenes fijadas de la plantilla. Conéctate antes de emitir.")?;
        assets.push(crate::api::IssuedSnapshotAsset { id: asset_id.clone(), sha256 });
    }
    let medications = local_prescription_items(conn, source_id)?.into_iter().map(|item| {
        let field = |key: &str| item.get(key).and_then(Value::as_str).unwrap_or_default().to_owned();
        crate::forms::MedicationBlock {
            name: field("medName"), dose: field("dose"), route: field("route"),
            frequency: field("frequency"), duration: field("duration"), quantity: field("quantity"),
            quantity_number: item.get("quantityNumber").and_then(Value::as_i64).map(|number| number as i32),
            notes: field("notes"),
        }
    }).collect::<Vec<_>>();
    let mut issued = prescription.clone();
    issued["doctorId"] = doctor_id.into();
    let variables = crate::offline_preview_variables(&issued, Some(&patient), Some(&clinic), &profile, at, &medications)
        .into_iter().map(|variable| (variable.key, variable.value)).collect();
    let snapshot = crate::api::IssuedSnapshot {
        version: 1,
        prescription_id: id,
        client_id: required("clientId")?,
        clinic_id, patient_id, doctor_id: doctor_id.to_owned(), actor_id: actor_id.to_owned(),
        device_id: device_id.to_owned(), template_version_id: version_id, rx_block_id: String::new(),
        rx_number: 0, issued_at: at.to_owned(),
        doc_sha256: hex::encode(Sha256::digest(doc_json.as_bytes())),
        theme_sha256: hex::encode(Sha256::digest(theme.as_deref().unwrap_or("").as_bytes())),
        doc_json, theme, assets, variables, medications,
        patient_birth_date: patient.get("birthDate").and_then(Value::as_str).map(str::to_owned),
    };
    for (_, bytes) in issued_snapshot_assets(conn, &snapshot)? {
        printpdf::RawImage::decode_from_bytes(&bytes, &mut Vec::new())
            .map_err(|_| "Una imagen fijada no es válida.")?;
    }
    let block = diesel::sql_query("SELECT COUNT(*) AS value FROM (SELECT clinic_id FROM rx_blocks WHERE device_id = ? AND status = 'active' AND consumed <= end_number - start_number ORDER BY allocated_at DESC LIMIT 1) WHERE clinic_id = ?")
        .bind::<Text, _>(device_id).bind::<Text, _>(&snapshot.clinic_id)
        .load::<IntRow>(conn).map_err(|error| error.to_string())?;
    if !block.iter().any(|row| row.value > 0) {
        return Err("No hay un bloque de números disponible para esta clínica y dispositivo.".to_owned());
    }
    Ok(snapshot)
}

fn bind_issued_snapshot_number(snapshot: &mut crate::api::IssuedSnapshot, consumed: &ConsumedNumber) -> Result<(), String> {
    snapshot.rx_number = i32::try_from(consumed.number).map_err(|_| "Número de receta fuera de rango.")?;
    snapshot.rx_block_id = consumed.block_id.clone();
    let number = snapshot.variables.iter_mut().find(|(key, _)| key == "prescription.number")
        .ok_or("Falta el número en las variables de emisión.")?;
    number.1 = format!("{:06}", snapshot.rx_number);
    snapshot.validate_structure()
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
        authorize_action(conn, &prescription, "sign", actor_id, Some(doctor_id))?;
        if prescription.get("status").and_then(Value::as_str) != Some("draft") {
            return Err("Solo un borrador puede firmarse.".to_owned());
        }
        let at = now();
        let mut snapshot = capture_issued_snapshot(conn, &prescription, prescription_id, device_id, doctor_id, actor_id, &at)?;
        let consumed = consume_local_rx_number(conn, device_id).map_err(|exhausted| {
            format!(
                "Bloque de números agotado. Quedan {} números.",
                exhausted.remaining
            )
        })?;
        bind_issued_snapshot_number(&mut snapshot, &consumed)?;
        let client_id = prescription
            .get("clientId")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        diesel::sql_query(
            "UPDATE prescriptions SET status = 'signed', doctor_id = :doctor_id, rx_number = :rx_number, rx_block_id = :rx_block_id, signed_at = :signed_at, synced_at = NULL, updated_at = :updated_at, issued_snapshot_json = :snapshot WHERE id = :id",
        )
        .bind::<Text, _>(doctor_id)
        .bind::<Integer, _>(&(consumed.number as i32))
        .bind::<Text, _>(consumed.block_id.as_str())
        .bind::<Text, _>(at.as_str())
        .bind::<Text, _>(at.as_str())
        .bind::<Text, _>(serde_json::to_string(&snapshot).map_err(|error| error.to_string())?)
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
                "issuedSnapshot": snapshot,
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
        "medicalRecordNo": form.get("medicalRecordNo").and_then(Value::as_str).unwrap_or_default(),
        "allergies": form.get("allergies").and_then(Value::as_str).unwrap_or_default(),
        "clientId": id,
        "createdBy": actor_id,
    });
    diesel::sql_query(
        "INSERT INTO patients (id, clinic_id, full_name, national_id, birth_date, sex, phone, notes, medical_record_no, allergies, client_id, deleted_at, created_at, updated_at, created_by) \
         VALUES (:id, :clinic_id, :full_name, :national_id, :birth_date, :sex, :phone, :notes, :medical_record_no, :allergies, :client_id, NULL, :created_at, :updated_at, :created_by)",
    )
    .bind::<Text, _>(id.as_str())
    .bind::<Text, _>(clinic_id)
    .bind::<Text, _>(form.get("fullName").and_then(Value::as_str).unwrap_or_default())
    .bind::<Text, _>(form.get("nationalId").and_then(Value::as_str).unwrap_or_default())
    .bind::<Text, _>(form.get("birthDate").and_then(Value::as_str).unwrap_or_default())
    .bind::<Text, _>(form.get("sex").and_then(Value::as_str).unwrap_or_default())
    .bind::<Text, _>(form.get("phone").and_then(Value::as_str).unwrap_or_default())
    .bind::<Text, _>(form.get("notes").and_then(Value::as_str).unwrap_or_default())
    .bind::<Text, _>(form.get("medicalRecordNo").and_then(Value::as_str).unwrap_or_default())
    .bind::<Text, _>(form.get("allergies").and_then(Value::as_str).unwrap_or_default())
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
    let identity = local_identity(conn)?.ok_or_else(|| "Receta no encontrada en la caché local.".to_owned())?;
    let row = json!({"clinicId": clinic_id, "doctorId": doctor_id, "status": "draft"});
    authorize_action(conn, &row, "edit", actor_id, None)?;
    let role = identity.get("role").and_then(Value::as_str);
    let current_doctor = identity.pointer("/doctor/id").and_then(Value::as_str);
    if (role == Some("doctor") && (current_doctor.is_none() || doctor_id != current_doctor))
        || (role == Some("nursing") && doctor_id.is_some())
    {
        return Err("Receta no encontrada en la caché local.".to_owned());
    }
    let id = Uuid::now_v7().to_string();
    let at = now();
    let diagnosis = clean_local_text(diagnosis, 2000);
    let validity = clean_local_text(validity, 200);
    let recetario_type = "ambulatorio";
    let instructions = clean_local_text(instructions, 8000);
    diesel::sql_query(
        "INSERT INTO prescriptions (id, patient_id, doctor_id, clinic_id, template_version_id, diagnosis, validity, recetario_type, validity_days, warning_signs, instructions, status, rx_number, rx_block_id, signed_at, synced_at, client_id, created_at, updated_at, created_by) \
         VALUES (:id, :patient_id, :doctor_id, :clinic_id, :template_version_id, :diagnosis, :validity, :recetario_type, :validity_days, :warning_signs, :instructions, 'draft', NULL, NULL, NULL, NULL, :client_id, :created_at, :updated_at, :created_by)",
    )
    .bind::<Text, _>(id.as_str())
    .bind::<Text, _>(patient_id)
    .bind::<Nullable<Text>, _>(doctor_id)
    .bind::<Text, _>(clinic_id)
    .bind::<Text, _>(template_version_id)
    .bind::<Nullable<Text>, _>(diagnosis.as_deref())
    .bind::<Nullable<Text>, _>(validity.as_deref())
    .bind::<Text, _>(recetario_type)
    .bind::<Nullable<Integer>, _>(None::<i32>)
    .bind::<Nullable<Text>, _>(None::<&str>)
    .bind::<Nullable<Text>, _>(instructions.as_deref())
    .bind::<Text, _>(id.as_str())
    .bind::<Text, _>(at.as_str())
    .bind::<Text, _>(at.as_str())
    .bind::<Text, _>(actor_id)
    .execute(conn)
    .map_err(|error| error.to_string())?;
    let scope = local_scope(conn)?.ok_or("Sesión local no disponible.")?;
    set_meta(conn, &format!("prescription_scope:{id}"), &scope)?;
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
            "recetarioType": recetario_type,
            "validityDays": Value::Null,
            "warningSigns": Value::Null,
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
    recetario_type: &str,
    validity_days: Option<i32>,
    warning_signs: &str,
    instructions: &str,
    actor_id: &str,
) -> Result<String, String> {
    let prescription = get_prescription(conn, prescription_id)?
        .ok_or_else(|| "Receta no encontrada en la caché local.".to_owned())?;
    authorize_action(conn, &prescription, "edit", actor_id, None)?;
    if prescription.get("status").and_then(Value::as_str) != Some("draft") {
        return Err("La receta no se puede editar.".to_owned());
    }
    let diagnosis = clean_local_text(diagnosis, 2000);
    let validity = clean_local_text(validity, 200);
    let recetario_type = recetario_type.trim();
    let recetario_type = if recetario_type.is_empty() {
        "ambulatorio"
    } else {
        recetario_type
    };
    let warning_signs = clean_local_text(warning_signs, 2000);
    let instructions = clean_local_text(instructions, 8000);
    let at = now();
    diesel::sql_query(
        "UPDATE prescriptions SET diagnosis = :diagnosis, validity = :validity, recetario_type = :recetario_type, validity_days = :validity_days, warning_signs = :warning_signs, instructions = :instructions, synced_at = NULL, updated_at = :updated_at WHERE id = :id",
    )
    .bind::<Nullable<Text>, _>(diagnosis.as_deref())
    .bind::<Nullable<Text>, _>(validity.as_deref())
    .bind::<Text, _>(recetario_type)
    .bind::<Nullable<Integer>, _>(validity_days)
    .bind::<Nullable<Text>, _>(warning_signs.as_deref())
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
            "recetarioType": recetario_type,
            "validityDays": validity_days,
            "warningSigns": warning_signs,
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
    quantity_number: Option<i32>,
    notes: &str,
    actor_id: &str,
) -> Result<String, String> {
    let prescription = get_prescription(conn, prescription_id)?
        .ok_or_else(|| "Receta no encontrada en la caché local.".to_owned())?;
    authorize_action(conn, &prescription, "edit", actor_id, None)?;
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
                "UPDATE prescription_items SET med_name = :med_name, dose = :dose, route = :route, frequency = :frequency, duration = :duration, quantity = :quantity, quantity_number = :quantity_number, notes = :notes, updated_at = :updated_at WHERE id = :id",
            )
            .bind::<Text, _>(med_name.as_str())
            .bind::<Nullable<Text>, _>(dose.as_deref())
            .bind::<Nullable<Text>, _>(route.as_deref())
            .bind::<Nullable<Text>, _>(frequency.as_deref())
            .bind::<Nullable<Text>, _>(duration.as_deref())
            .bind::<Nullable<Text>, _>(quantity.as_deref())
            .bind::<Nullable<Integer>, _>(quantity_number)
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
                "INSERT INTO prescription_items (id, prescription_id, position, med_name, dose, route, frequency, duration, quantity, quantity_number, notes, created_at, updated_at, created_by) VALUES (:id, :prescription_id, :position, :med_name, :dose, :route, :frequency, :duration, :quantity, :quantity_number, :notes, :created_at, :updated_at, :created_by)",
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
            .bind::<Nullable<Integer>, _>(quantity_number)
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
            "quantityNumber": quantity_number,
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
    if get_prescription(conn, prescription_id)?.is_none() {
        return Err("Receta no encontrada en la caché local.".to_owned());
    }
    json_rows(
        conn,
        &format!(
            "SELECT json_object('id', id, 'prescriptionId', prescription_id, 'position', position, \
             'medName', med_name, 'dose', dose, 'route', route, 'frequency', frequency, \
             'duration', duration, 'quantity', quantity, 'quantityNumber', quantity_number, 'notes', notes) AS row_json \
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
        "UPDATE patients SET full_name = :full_name, national_id = :national_id, birth_date = :birth_date, sex = :sex, phone = :phone, notes = :notes, medical_record_no = :medical_record_no, allergies = :allergies, updated_at = :updated_at WHERE id = :id",
    )
    .bind::<Text, _>(form.get("fullName").and_then(Value::as_str).unwrap_or_default())
    .bind::<Text, _>(national_id)
    .bind::<Text, _>(form.get("birthDate").and_then(Value::as_str).unwrap_or_default())
    .bind::<Text, _>(form.get("sex").and_then(Value::as_str).unwrap_or_default())
    .bind::<Text, _>(form.get("phone").and_then(Value::as_str).unwrap_or_default())
    .bind::<Text, _>(form.get("notes").and_then(Value::as_str).unwrap_or_default())
    .bind::<Text, _>(form.get("medicalRecordNo").and_then(Value::as_str).unwrap_or_default())
    .bind::<Text, _>(form.get("allergies").and_then(Value::as_str).unwrap_or_default())
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
            "medicalRecordNo": form.get("medicalRecordNo").and_then(Value::as_str).unwrap_or_default(),
            "allergies": form.get("allergies").and_then(Value::as_str).unwrap_or_default(),
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
    authorize_action(conn, &prescription, "transition", actor_id, None)?;
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
        authorize_action(conn, &original, "transition", actor_id, Some(doctor_id))?;
        let at = now();
        let id = Uuid::now_v7().to_string();
        let mut replacement = original.clone();
        replacement["id"] = id.clone().into();
        replacement["clientId"] = id.clone().into();
        let mut snapshot = capture_issued_snapshot(conn, &replacement, prescription_id, device_id, doctor_id, actor_id, &at)?;
        let consumed = consume_local_rx_number(conn, device_id)
            .map_err(|_| "Bloque de números agotado.".to_owned())?;
        bind_issued_snapshot_number(&mut snapshot, &consumed)?;
        cache_prescription(conn, &json!({
            "id": id, "patient_id": original["patientId"], "doctor_id": doctor_id,
            "clinic_id": original["clinicId"], "template_version_id": original["templateVersionId"],
            "status": "signed", "rx_number": consumed.number, "rx_block_id": consumed.block_id,
            "diagnosis": original["diagnosis"], "validity": original["validity"],
            "recetario_type": original["recetarioType"], "validity_days": original["validityDays"],
            "warning_signs": original["warningSigns"], "instructions": original["instructions"],
            "signed_at": at, "issued_snapshot": snapshot, "client_id": id, "supersedes_id": prescription_id,
            "created_at": at, "updated_at": at, "created_by": actor_id
        }))?;
        for item in local_prescription_items(conn, prescription_id)? {
            cache_prescription_item(conn, &json!({
                "id": Uuid::now_v7().to_string(), "prescription_id": id,
                "position": item["position"], "med_name": item["medName"], "dose": item["dose"],
                "route": item["route"], "frequency": item["frequency"], "duration": item["duration"],
                "quantity": item["quantity"], "quantity_number": item["quantityNumber"], "notes": item["notes"],
                "created_at": at, "updated_at": at, "created_by": actor_id
            }))?;
        }
        diesel::sql_query("UPDATE prescriptions SET status = 'superseded', synced_at = NULL, updated_at = ? WHERE id = ?")
            .bind::<Text, _>(&at).bind::<Text, _>(prescription_id)
            .execute(conn).map_err(|error| error.to_string())?;
        let outbox_id = enqueue_outbox(conn, "supersede_prescription", "prescription", &id, &json!({
            "prescriptionId": prescription_id, "newPrescriptionId": id, "rxNumber": consumed.number,
            "rxBlockId": consumed.block_id, "clientId": id, "doctorId": doctor_id,
            "createdBy": actor_id, "signedAt": at, "issuedSnapshot": snapshot
        }))?;
        Ok(OfflineSupersedeResult { id, rx_number: consumed.number, rx_block_id: consumed.block_id, outbox_id })
    })
}

pub fn date_now() -> NaiveDate {
    Utc::now().date_naive()
}

#[cfg(test)]
pub(crate) mod issued_snapshot_tests {
    use super::*;

    pub(crate) fn seed_render_inputs(conn: &mut SqliteConnection, actor: &str, doctor: &str) {
        set_meta(conn, "session_token", "fixture-token").unwrap();
        set_meta(conn, "session_expires_at", "2099-01-01T00:00:00Z").unwrap();
        set_meta(conn, "session_user", &json!({"id": actor}).to_string()).unwrap();
        set_meta(conn, "session_profile", &json!({
            "profile": {"id": actor, "displayName": "Fixture Doctor", "status": "active"},
            "role": "doctor", "doctor": {"id": doctor, "specialty": "Fixture specialty",
                "licenseNo": "FIXTURE", "nationalId": "fixture-doctor", "phone": "fixture-phone", "email": "fixture@example.invalid"},
            "clinic": {"id": "clinic", "name": "Fixture Clinic", "timezone": "America/Guayaquil"}
        }).to_string()).unwrap();
        cache_clinic(conn, &json!({
            "id": "clinic", "name": "Fixture Clinic", "address": "Fixture Address", "phone": "Fixture Phone",
            "timezone": "America/Guayaquil", "retention_days": 30, "rx_block_size": 50,
            "created_at": "2026-01-01T00:00:00Z", "updated_at": "2026-01-01T00:00:00Z"
        })).unwrap();
        cache_patient(conn, &json!({
            "id": "patient", "clinic_id": "clinic", "full_name": "Fixture Patient", "birth_date": "2000-01-01",
            "national_id": "fixture-patient", "sex": "fixture-sex", "medical_record_no": "fixture-record",
            "allergies": "fixture-allergy", "created_at": "2026-01-01T00:00:00Z", "updated_at": "2026-01-01T00:00:00Z"
        })).unwrap();
        cache_template_version(conn, &json!({
            "id": "template", "template_id": "parent-template", "version_no": 1,
            "doc_json": crate::forms::default_document_json(), "variables_spec": "[]",
            "theme": "{\"marginMm\":14}", "status": "published",
            "created_at": "2026-01-01T00:00:00Z", "updated_at": "2026-01-01T00:00:00Z"
        })).unwrap();
    }

    fn fixture() -> (PathBuf, SqliteConnection) {
        let path = std::env::temp_dir().join(format!("issued-snapshot-{}.sqlite", Uuid::now_v7()));
        let mut conn = SqliteConnection::establish(&path.to_string_lossy()).unwrap();
        initialize_test_db(&mut conn);
        seed_render_inputs(&mut conn, "actor", "doctor");
        cache_rx_block(&mut conn, &json!({
            "id": "block", "clinic_id": "clinic", "device_id": "device", "start_number": 100, "end_number": 109,
            "consumed": 0, "status": "active", "allocated_at": "2026-01-01T00:00:00Z",
            "created_at": "2026-01-01T00:00:00Z", "updated_at": "2026-01-01T00:00:00Z"
        })).unwrap();
        cache_prescription(&mut conn, &json!({
            "id": "issued", "patient_id": "patient", "doctor_id": "doctor", "clinic_id": "clinic",
            "template_version_id": "template", "status": "draft", "client_id": "issued",
            "diagnosis": "Fixture diagnosis", "recetario_type": "ambulatorio", "validity_days": 3,
            "warning_signs": "Fixture warning", "instructions": "Fixture instructions",
            "created_at": "2026-01-01T00:00:00Z", "updated_at": "2026-01-01T00:00:00Z", "created_by": "actor"
        })).unwrap();
        for (position, name) in [(2, "Fixture B"), (1, "Fixture A")] {
            cache_prescription_item(&mut conn, &json!({
                "id": format!("item-{position}"), "prescription_id": "issued", "position": position,
                "med_name": name, "dose": "Fixture dose", "route": "Fixture route", "frequency": "Fixture frequency",
                "duration": "Fixture duration", "quantity": "Fixture quantity", "quantity_number": position,
                "notes": "Fixture notes", "created_at": "2026-01-01T00:00:00Z", "updated_at": "2026-01-01T00:00:00Z"
            })).unwrap();
        }
        (path, conn)
    }

    fn preview(conn: &mut SqliteConnection) -> crate::api::PrescriptionPreview {
        let profile = serde_json::from_str::<crate::api::Me>(&get_meta(conn, "session_profile").unwrap().unwrap()).unwrap();
        crate::build_local_preview(conn, "issued", &profile).unwrap().unwrap()
    }

    #[test]
    fn issued_snapshot_restart_cache_mutation_transport_and_final_replay() {
        let (path, mut conn) = fixture();
        let before = Utc::now().timestamp();
        let sign = offline_sign(&mut conn, "issued", "device", "doctor", "actor").unwrap();
        let row = get_prescription(&mut conn, "issued").unwrap().unwrap();
        let snapshot = issued_snapshot_from_row(&row).unwrap();
        assert!(DateTime::parse_from_rfc3339(&snapshot.issued_at).unwrap().timestamp() >= before);
        assert_eq!(row["signedAt"], snapshot.issued_at);
        assert_eq!(snapshot.medications[0].name, "Fixture A");
        assert_eq!(snapshot.medications[1].quantity_number, Some(2));
        let provisional = preview(&mut conn);
        assert!(provisional.provisional);
        assert!(provisional.qr_data_url.is_none());
        let original_outbox = outbox_rows(&mut conn).unwrap();
        let payload: Value = serde_json::from_str(original_outbox[0]["payloadJson"].as_str().unwrap()).unwrap();
        assert_eq!(payload["issuedSnapshot"], row["issuedSnapshot"]);
        assert!(offline_sign(&mut conn, "issued", "device", "doctor", "actor").is_err());
        assert_eq!(active_block_remaining(&mut conn, "device").unwrap(), 9);
        drop(conn);
        let mut conn = SqliteConnection::establish(&path.to_string_lossy()).unwrap();
        exec(&mut conn, "UPDATE patients SET full_name = 'Changed', birth_date = '2010-01-01', national_id = 'changed', sex = 'changed', medical_record_no = 'changed', allergies = 'changed'").unwrap();
        exec(&mut conn, "UPDATE clinics SET name = 'Changed', address = 'Changed', phone = 'Changed'").unwrap();
        exec(&mut conn, "UPDATE template_versions SET doc_json = '{}', theme = '{}'").unwrap();
        exec(&mut conn, "UPDATE prescriptions SET diagnosis = 'Changed', validity_days = 99, warning_signs = 'Changed', instructions = 'Changed'").unwrap();
        exec(&mut conn, "DELETE FROM prescription_items").unwrap();
        let mut profile: Value = serde_json::from_str(&get_meta(&mut conn, "session_profile").unwrap().unwrap()).unwrap();
        profile["profile"]["displayName"] = "Changed".into();
        profile["doctor"]["specialty"] = "Changed".into();
        profile["doctor"]["nationalId"] = "Changed".into();
        profile["doctor"]["licenseNo"] = "Changed".into();
        profile["doctor"]["phone"] = "Changed".into();
        profile["doctor"]["email"] = "Changed".into();
        set_meta(&mut conn, "session_profile", &profile.to_string()).unwrap();
        assert_eq!(preview(&mut conn), provisional);
        assert!(crate::finish_prescription_replay(&mut conn, &sign.outbox_id, 0, Err(crate::http::RequestError::Network("fixture transport unavailable".into()))));
        assert!(has_pending_outbox(&mut conn, "issued").unwrap());
        assert_eq!(outbox_rows(&mut conn).unwrap()[0]["payloadJson"], original_outbox[0]["payloadJson"]);
        assert_eq!(preview(&mut conn), provisional);
        assert!(cache_prescription(&mut conn, &json!({"id":"issued", "clinic_id":"clinic", "doctor_id":"doctor", "status":"signed"})).is_err());
        assert!(!evict_local_cache_row(&mut conn, "prescriptions", "issued").unwrap());
        assert!(!crate::finish_prescription_replay(&mut conn, &sign.outbox_id, 1, Err(crate::http::RequestError::Response("fixture mismatch".into()))));
        assert!(has_pending_outbox(&mut conn, "issued").unwrap());
        assert_eq!(outbox_rows(&mut conn).unwrap()[0]["payloadJson"], original_outbox[0]["payloadJson"]);
        assert_eq!(preview(&mut conn), provisional);
        let mut response = row.clone();
        response["verification"] = json!({"code": "fixture-opaque-code", "status": "valid", "createdAt": snapshot.issued_at, "updatedAt": snapshot.issued_at});
        let mut altered = payload.clone();
        altered["issuedSnapshot"]["variables"][0][1] = "Altered".into();
        assert!(apply_issued_snapshot_reconciliation(&mut conn, &altered, &response).is_err());
        assert_eq!(preview(&mut conn), provisional);
        apply_issued_snapshot_reconciliation(&mut conn, &payload, &response).unwrap();
        apply_issued_snapshot_reconciliation(&mut conn, &payload, &response).unwrap();
        assert!(!crate::finish_prescription_replay(&mut conn, &sign.outbox_id, 1, Ok(())));
        let final_preview = preview(&mut conn);
        assert!(!final_preview.provisional);
        assert!(final_preview.qr_data_url.is_some());
        assert_eq!(final_preview.doc_json, provisional.doc_json);
        assert_eq!(final_preview.variables, provisional.variables);
        assert_eq!(final_preview.medications, provisional.medications);
        assert_eq!(final_preview.image_sources, provisional.image_sources);
        assert_eq!(final_preview.margin_mm, provisional.margin_mm);
        assert_eq!(final_preview.issued_at, provisional.issued_at);
        let final_row = get_prescription(&mut conn, "issued").unwrap().unwrap();
        let input = issued_snapshot_pdf_input(&mut conn, &final_row, "fixture.pdf").unwrap();
        assert_eq!(input.values, snapshot.variables);
        assert_eq!(input.medications, snapshot.medications);
        assert!(matches!(input.mode, crate::pdf::RenderMode::Final { .. }));
        assert!(render_prescription_pdf(&mut conn, "issued", &input).unwrap().starts_with(b"%PDF-"));
        assert_eq!(issued_snapshot_from_row(&final_row).unwrap(), snapshot);
        offline_void(&mut conn, "issued", "Fixture void reason", "actor").unwrap();
        assert_eq!(issued_snapshot_from_row(&get_prescription(&mut conn, "issued").unwrap().unwrap()).unwrap(), snapshot);
        assert_eq!(preview(&mut conn).variables, provisional.variables);
        drop(conn);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn issued_snapshot_missing_inputs_and_assets_never_consume_number() {
        let (path, mut conn) = fixture();
        for statement in [
            "DELETE FROM patients",
            "DELETE FROM clinics",
            "DELETE FROM template_versions",
            "UPDATE template_versions SET doc_json = '{}'",
            "UPDATE template_versions SET status = 'draft'",
        ] {
            exec(&mut conn, statement).unwrap();
            let error = offline_sign(&mut conn, "issued", "device", "doctor", "actor").unwrap_err();
            assert!(!error.contains("transaction"), "{error}");
            assert_eq!(active_block_remaining(&mut conn, "device").unwrap(), 10);
            assert!(outbox_rows(&mut conn).unwrap().is_empty());
            seed_render_inputs(&mut conn, "actor", "doctor");
        }
        let mut document = crate::forms::parse_document(&crate::forms::default_document_json()).unwrap();
        document.letterhead.logo_asset_id = Some("asset".into());
        diesel::sql_query("UPDATE template_versions SET doc_json = ?")
            .bind::<Text, _>(serde_json::to_string(&document).unwrap()).execute(&mut conn).unwrap();
        assert!(offline_sign(&mut conn, "issued", "device", "doctor", "actor").is_err());
        let bytes = crate::pdf::qr_png("issued-snapshot-asset-fixture").unwrap();
        let hash = hex::encode(Sha256::digest(&bytes));
        cache_asset_meta(&mut conn, &json!({
            "id": "asset", "clinic_id": "clinic", "kind": "logo", "status": "ready", "sha256": hash,
            "created_at": "2026-01-01T00:00:00Z", "updated_at": "2026-01-01T00:00:00Z"
        })).unwrap();
        upsert_pinned_assets(&mut conn, "template", &["asset".into()]).unwrap();
        assert!(offline_sign(&mut conn, "issued", "device", "doctor", "actor").is_err());
        assert_eq!(active_block_remaining(&mut conn, "device").unwrap(), 10);
        let asset_path = path.with_extension("png");
        std::fs::write(&asset_path, b"corrupt").unwrap();
        diesel::sql_query("INSERT INTO asset_cache VALUES ('asset', ?, ?, ?, ?)")
            .bind::<Text, _>(&hash).bind::<Integer, _>(bytes.len() as i32)
            .bind::<Text, _>(asset_path.to_string_lossy().as_ref()).bind::<Text, _>(now()).execute(&mut conn).unwrap();
        assert!(offline_sign(&mut conn, "issued", "device", "doctor", "actor").is_err());
        assert_eq!(active_block_remaining(&mut conn, "device").unwrap(), 10);
        assert!(outbox_rows(&mut conn).unwrap().is_empty());
        std::fs::write(&asset_path, &bytes).unwrap();
        offline_sign(&mut conn, "issued", "device", "doctor", "actor").unwrap();
        assert_eq!(preview(&mut conn).image_sources.len(), 1);
        let provisional = preview(&mut conn);
        let row = get_prescription(&mut conn, "issued").unwrap().unwrap();
        let pending = outbox_rows(&mut conn).unwrap();
        let payload: Value = serde_json::from_str(pending[0]["payloadJson"].as_str().unwrap()).unwrap();
        let mut response = row.clone();
        response["verification"] = json!({"code":"fixture-asset-code", "status":"valid", "createdAt":row["signedAt"], "updatedAt":row["signedAt"]});
        apply_issued_snapshot_reconciliation(&mut conn, &payload, &response).unwrap();
        exec(&mut conn, "UPDATE assets SET sha256 = 'changed', status = 'deleted'").unwrap();
        let final_preview = preview(&mut conn);
        assert_eq!(final_preview.image_sources, provisional.image_sources);
        assert_eq!(final_preview.doc_json, provisional.doc_json);
        assert_eq!(final_preview.variables, provisional.variables);
        assert!(!final_preview.provisional);
        let final_row = get_prescription(&mut conn, "issued").unwrap().unwrap();
        let input = issued_snapshot_pdf_input(&mut conn, &final_row, "asset-fixture.pdf").unwrap();
        assert_eq!(input.assets, vec![("asset".to_owned(), bytes.clone())]);
        assert!(render_prescription_pdf(&mut conn, "issued", &input).unwrap().starts_with(b"%PDF-"));
        exec(&mut conn, "DELETE FROM template_version_assets").unwrap();
        assert!(!evict_asset_cache(&mut conn, "asset").unwrap());
        std::fs::write(&asset_path, b"changed").unwrap();
        let row = get_prescription(&mut conn, "issued").unwrap().unwrap();
        assert!(issued_snapshot_preview(&mut conn, &row).is_err());
        drop(conn);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_file(asset_path).unwrap();
    }

    #[test]
    fn issued_snapshot_outbox_failure_rolls_back_entire_issue() {
        let (path, mut conn) = fixture();
        exec(&mut conn, "CREATE TRIGGER reject_issue BEFORE INSERT ON outbox WHEN NEW.operation = 'sign_prescription' BEGIN SELECT RAISE(ABORT, 'fixture failure'); END").unwrap();
        assert!(offline_sign(&mut conn, "issued", "device", "doctor", "actor").is_err());
        let row = get_prescription(&mut conn, "issued").unwrap().unwrap();
        assert_eq!(row["status"], "draft");
        assert!(row["signedAt"].is_null());
        assert!(row["issuedSnapshot"].is_null());
        assert_eq!(active_block_remaining(&mut conn, "device").unwrap(), 10);
        assert!(outbox_rows(&mut conn).unwrap().is_empty());
        drop(conn);
        std::fs::remove_file(path).unwrap();
    }

    #[tokio::test]
    async fn issued_snapshot_migration_preserves_legacy_pending_without_backfill() {
        use diesel::connection::SimpleConnection;
        let path = std::env::temp_dir().join(format!("issued-snapshot-legacy-{}.sqlite", Uuid::now_v7()));
        let mut conn = SqliteConnection::establish(&path.to_string_lossy()).unwrap();
        conn.batch_execute(&SCHEMA.replace("  issued_snapshot_json TEXT,\n", "").replace("  verification_json TEXT,\n", "")).unwrap();
        seed_render_inputs(&mut conn, "actor", "doctor");
        exec(&mut conn, "INSERT INTO prescriptions (id, patient_id, doctor_id, clinic_id, status, client_id, created_at, updated_at) VALUES ('legacy', 'patient', 'doctor', 'clinic', 'signed', 'legacy', '2026', '2026'), ('draft', 'patient', 'doctor', 'clinic', 'draft', 'draft', '2026', '2026'), ('pending', 'patient', 'doctor', 'clinic', 'signed', 'pending', '2026', '2026')").unwrap();
        let pending = json!({"prescriptionId": "pending", "rxNumber": 9, "rxBlockId": "old-block", "signedAt": "2026-01-01T00:00:00Z", "createdBy": "actor"});
        enqueue_outbox(&mut conn, "sign_prescription", "prescription", "pending", &pending).unwrap();
        let original = outbox_rows(&mut conn).unwrap();
        migrate_issued_snapshot(&mut conn).unwrap();
        migrate_issued_snapshot(&mut conn).unwrap();
        let scope = local_scope(&mut conn).unwrap().unwrap();
        for id in ["legacy", "draft", "pending"] {
            set_meta(&mut conn, &format!("prescription_scope:{id}"), &scope).unwrap();
        }
        drop(conn);
        let mut conn = SqliteConnection::establish(&path.to_string_lossy()).unwrap();
        assert_eq!(outbox_rows(&mut conn).unwrap(), original);
        for id in ["legacy", "pending"] {
            let row = get_prescription(&mut conn, id).unwrap().unwrap();
            assert!(row["issuedSnapshot"].is_null());
            assert_eq!(issued_snapshot_preview(&mut conn, &row).unwrap_err(), HISTORY_UNAVAILABLE);
        }
        assert!(matches!(crate::replay_prescription_outbox("sign_prescription", &pending).await, Err(crate::http::RequestError::Response(_))));
        assert!(update_local_prescription(&mut conn, "draft", "Editable", "", "ambulatorio", None, "", "", "actor").is_ok());
        assert_eq!(outbox_rows(&mut conn).unwrap()[0], original[0]);
        drop(conn);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn issued_snapshot_supersede_preserves_original_and_freezes_replacement() {
        let (path, mut conn) = fixture();
        offline_sign(&mut conn, "issued", "device", "doctor", "actor").unwrap();
        let original = issued_snapshot_from_row(&get_prescription(&mut conn, "issued").unwrap().unwrap()).unwrap();
        let corrected = offline_supersede(&mut conn, "issued", "device", "doctor", "actor").unwrap();
        let old = get_prescription(&mut conn, "issued").unwrap().unwrap();
        assert_eq!(old["status"], "superseded");
        assert_eq!(issued_snapshot_from_row(&old).unwrap(), original);
        let replacement = get_prescription(&mut conn, &corrected.id).unwrap().unwrap();
        let replacement_snapshot = issued_snapshot_from_row(&replacement).unwrap();
        assert_eq!(replacement_snapshot.rx_number, original.rx_number + 1);
        assert_eq!(replacement_snapshot.medications, original.medications);
        assert_eq!(replacement_snapshot.variables.iter().find(|(key, _)| key == "prescription.warning_signs"), original.variables.iter().find(|(key, _)| key == "prescription.warning_signs"));
        let actual_items = local_prescription_items(&mut conn, &corrected.id).unwrap();
        assert_eq!(actual_items[1]["quantityNumber"], 2);
        assert_eq!(issued_snapshot_preview(&mut conn, &replacement).unwrap().medications, original.medications);
        drop(conn);
        std::fs::remove_file(path).unwrap();
    }
}
