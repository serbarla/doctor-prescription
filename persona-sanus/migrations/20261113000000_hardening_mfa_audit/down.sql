DROP TRIGGER IF EXISTS audit_log_no_delete;
DROP TRIGGER IF EXISTS audit_log_no_update;
CREATE TABLE IF NOT EXISTS sessions_new (
  id         TEXT NOT NULL PRIMARY KEY,
  token_hash TEXT NOT NULL UNIQUE,
  user_id    TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  expires_at TEXT NOT NULL,
  created_at TEXT NOT NULL,
  device_id  TEXT
);
INSERT INTO sessions_new (id, token_hash, user_id, expires_at, created_at, device_id)
  SELECT id, token_hash, user_id, expires_at, created_at, device_id FROM sessions;
DROP TABLE sessions;
ALTER TABLE sessions_new RENAME TO sessions;
CREATE INDEX idx_sessions_user ON sessions (user_id);
CREATE INDEX idx_sessions_expiry ON sessions (expires_at);
DROP TABLE IF EXISTS user_mfa;
