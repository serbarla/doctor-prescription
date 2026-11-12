CREATE TABLE profiles (
  id           TEXT PRIMARY KEY,
  auth_user_id TEXT NOT NULL UNIQUE,
  display_name TEXT NOT NULL,
  status       TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','disabled')),
  created_at   TEXT NOT NULL,
  updated_at   TEXT NOT NULL,
  created_by   TEXT REFERENCES profiles(id)
);

CREATE TABLE clinics (
  id               TEXT PRIMARY KEY,
  name             TEXT NOT NULL,
  address          TEXT,
  phone            TEXT,
  logo_asset_id    TEXT REFERENCES assets(id),
  timezone         TEXT NOT NULL DEFAULT 'America/Guayaquil',
  llm_url_template TEXT,
  retention_days   INTEGER NOT NULL DEFAULT 30,
  rx_block_size    INTEGER NOT NULL DEFAULT 50,
  server_retention_until TEXT NOT NULL,
  data_grace_ends_at     TEXT,
  created_at       TEXT NOT NULL,
  updated_at       TEXT NOT NULL,
  created_by       TEXT REFERENCES profiles(id)
);

CREATE TABLE assets (
  id            TEXT PRIMARY KEY,
  clinic_id     TEXT NOT NULL REFERENCES clinics(id) ON DELETE CASCADE,
  object_key    TEXT NOT NULL UNIQUE,
  owner_type    TEXT NOT NULL CHECK (owner_type IN ('doctor','clinic')),
  owner_id      TEXT NOT NULL,
  kind          TEXT NOT NULL DEFAULT 'image' CHECK (kind IN ('logo','image')),
  original_name TEXT,
  content_type  TEXT NOT NULL CHECK (content_type IN ('image/jpeg','image/png')),
  size_bytes    INTEGER NOT NULL CHECK (size_bytes > 0 AND size_bytes <= 10485760),
  sha256        TEXT NOT NULL,
  width         INTEGER CHECK (width IS NULL OR width > 0),
  height        INTEGER CHECK (height IS NULL OR height > 0),
  status        TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','ready')),
  deleted_at    TEXT,
  created_at    TEXT NOT NULL,
  updated_at    TEXT NOT NULL,
  created_by    TEXT REFERENCES profiles(id)
);
CREATE INDEX idx_assets_clinic_kind ON assets (clinic_id, kind, deleted_at);
CREATE INDEX idx_assets_owner ON assets (owner_type, owner_id);

CREATE TABLE clinic_members (
  clinic_id  TEXT NOT NULL REFERENCES clinics(id) ON DELETE CASCADE,
  profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  role       TEXT NOT NULL CHECK (role IN ('administrator','doctor','nursing','secretary')),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  created_by TEXT REFERENCES profiles(id),
  PRIMARY KEY (clinic_id, profile_id)
);
CREATE UNIQUE INDEX uq_clinic_members_profile ON clinic_members (profile_id);

CREATE TABLE devices (
  id           TEXT PRIMARY KEY,
  profile_id   TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  name         TEXT,
  platform     TEXT NOT NULL CHECK (platform IN ('macos','windows','web')),
  app_version  TEXT,
  last_seen_at TEXT,
  revoked_at   TEXT,
  created_at   TEXT NOT NULL,
  updated_at   TEXT NOT NULL,
  created_by   TEXT REFERENCES profiles(id)
);
CREATE INDEX idx_devices_profile ON devices (profile_id, revoked_at);

CREATE TABLE doctors (
  id                 TEXT PRIMARY KEY,
  profile_id         TEXT NOT NULL UNIQUE REFERENCES profiles(id) ON DELETE CASCADE,
  specialty          TEXT,
  license_no         TEXT,
  national_id        TEXT,
  phone              TEXT,
  email              TEXT,
  created_at         TEXT NOT NULL,
  updated_at         TEXT NOT NULL,
  created_by         TEXT REFERENCES profiles(id)
);

CREATE TABLE patients (
  id          TEXT PRIMARY KEY,
  clinic_id   TEXT NOT NULL REFERENCES clinics(id),
  full_name   TEXT NOT NULL,
  national_id TEXT,
  birth_date  TEXT,
  sex         TEXT CHECK (sex IN ('male','female','other','unspecified')),
  phone       TEXT,
  notes       TEXT,
  client_id   TEXT UNIQUE,
  deleted_at  TEXT,
  created_at  TEXT NOT NULL,
  updated_at  TEXT NOT NULL,
  created_by  TEXT REFERENCES profiles(id)
);
CREATE UNIQUE INDEX uq_patients_national_id
  ON patients (clinic_id, national_id)
  WHERE national_id IS NOT NULL AND deleted_at IS NULL;
CREATE INDEX idx_patients_clinic_name ON patients (clinic_id, full_name);

CREATE TABLE templates (
  id                TEXT PRIMARY KEY,
  owner_type        TEXT NOT NULL CHECK (owner_type IN ('doctor','clinic')),
  owner_id          TEXT NOT NULL,
  name              TEXT NOT NULL,
  current_version_id TEXT REFERENCES template_versions(id),
  status            TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft','published','archived')),
  client_id         TEXT UNIQUE,
  created_at        TEXT NOT NULL,
  updated_at        TEXT NOT NULL,
  created_by        TEXT REFERENCES profiles(id)
);
CREATE INDEX idx_templates_owner ON templates (owner_type, owner_id);

CREATE TABLE template_versions (
  id             TEXT PRIMARY KEY,
  template_id    TEXT NOT NULL REFERENCES templates(id) ON DELETE CASCADE,
  version_no     INTEGER NOT NULL,
  doc_json       TEXT NOT NULL,
  variables_spec TEXT NOT NULL,
  theme          TEXT,
  status         TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft','published','archived')),
  created_at     TEXT NOT NULL,
  updated_at     TEXT NOT NULL,
  created_by     TEXT REFERENCES profiles(id),
  UNIQUE (template_id, version_no)
);

CREATE TABLE template_version_assets (
  template_version_id TEXT NOT NULL REFERENCES template_versions(id) ON DELETE CASCADE,
  asset_id            TEXT NOT NULL REFERENCES assets(id),
  PRIMARY KEY (template_version_id, asset_id)
);
CREATE INDEX idx_template_version_assets_asset ON template_version_assets (asset_id);

CREATE TABLE rx_counters (
  clinic_id       TEXT PRIMARY KEY REFERENCES clinics(id) ON DELETE CASCADE,
  last_rx_number  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE rx_blocks (
  id           TEXT PRIMARY KEY,
  clinic_id    TEXT NOT NULL REFERENCES clinics(id) ON DELETE CASCADE,
  device_id    TEXT NOT NULL REFERENCES devices(id),
  start_number INTEGER NOT NULL,
  end_number   INTEGER NOT NULL,
  consumed     INTEGER NOT NULL DEFAULT 0 CHECK (consumed >= 0),
  status       TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','exhausted','revoked')),
  allocated_at TEXT NOT NULL,
  created_at   TEXT NOT NULL,
  updated_at   TEXT NOT NULL,
  created_by   TEXT REFERENCES profiles(id),
  CHECK (start_number > 0 AND end_number >= start_number),
  CHECK (consumed <= end_number - start_number + 1),
  UNIQUE (clinic_id, start_number)
);
CREATE UNIQUE INDEX uq_rx_blocks_active ON rx_blocks (device_id) WHERE status = 'active';

CREATE TABLE prescriptions (
  id                  TEXT PRIMARY KEY,
  patient_id          TEXT NOT NULL REFERENCES patients(id),
  doctor_id           TEXT REFERENCES doctors(id),
  clinic_id           TEXT NOT NULL REFERENCES clinics(id),
  template_version_id TEXT REFERENCES template_versions(id),
  status              TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft','signed','superseded','voided')),
  rx_number           INTEGER,
  rx_block_id         TEXT REFERENCES rx_blocks(id),
  diagnosis           TEXT,
  validity            TEXT,
  instructions        TEXT,
  signed_at           TEXT,
  synced_at           TEXT,
  voided_at           TEXT,
  void_reason         TEXT,
  client_id           TEXT NOT NULL UNIQUE,
  supersedes_id       TEXT REFERENCES prescriptions(id),
  created_at          TEXT NOT NULL,
  updated_at          TEXT NOT NULL,
  created_by          TEXT REFERENCES profiles(id),
  CHECK (status = 'draft' OR doctor_id IS NOT NULL)
);
CREATE UNIQUE INDEX uq_prescriptions_rx
  ON prescriptions (clinic_id, rx_number)
  WHERE rx_number IS NOT NULL;
CREATE INDEX idx_prescriptions_patient ON prescriptions (patient_id, created_at);
CREATE INDEX idx_prescriptions_doctor  ON prescriptions (doctor_id, created_at);
CREATE INDEX idx_prescriptions_status  ON prescriptions (clinic_id, status, created_at);

CREATE TABLE prescription_items (
  id              TEXT PRIMARY KEY,
  prescription_id TEXT NOT NULL REFERENCES prescriptions(id) ON DELETE CASCADE,
  position        INTEGER NOT NULL,
  med_name        TEXT NOT NULL,
  dose            TEXT,
  route           TEXT,
  frequency       TEXT,
  duration        TEXT,
  quantity        TEXT,
  notes           TEXT,
  created_at      TEXT NOT NULL,
  updated_at      TEXT NOT NULL,
  created_by      TEXT REFERENCES profiles(id),
  UNIQUE (prescription_id, position)
);

CREATE TABLE verification_codes (
  id              TEXT PRIMARY KEY,
  prescription_id TEXT NOT NULL UNIQUE REFERENCES prescriptions(id) ON DELETE CASCADE,
  code            TEXT NOT NULL,
  code_hash       TEXT NOT NULL UNIQUE,
  status          TEXT NOT NULL DEFAULT 'valid' CHECK (status IN ('valid','voided','superseded')),
  created_at      TEXT NOT NULL,
  updated_at      TEXT NOT NULL,
  created_by      TEXT REFERENCES profiles(id)
);

CREATE TABLE audit_log (
  id            TEXT PRIMARY KEY,
  actor_id      TEXT REFERENCES profiles(id),
  action        TEXT NOT NULL,
  entity        TEXT NOT NULL,
  entity_id     TEXT NOT NULL,
  metadata_json TEXT,
  at            TEXT NOT NULL
);
CREATE INDEX idx_audit_entity ON audit_log (entity, entity_id);
CREATE INDEX idx_audit_actor  ON audit_log (actor_id, at);
CREATE INDEX idx_audit_at     ON audit_log (at);
