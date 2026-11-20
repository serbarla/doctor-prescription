# PersonaSanus — Runbook

| Field | Value |
|---|---|
| Source | `context/PRD.md` §9, §11 (M3/M4), §12; `context/architecture.md` §13–§14, §18 |
| Spec | `specs/13-M-ops-and-release.md` |

## 1. Deploy

```sh
docker compose up -d --build
curl -sf http://127.0.0.1:8080/health
```

Expected health body: `{"ok":true,"foreign_keys":true,"journal_mode":"wal"}`. The `persona-sanus` service has `restart: unless-stopped` and a 5 s healthcheck; `minio` gates startup on its own healthcheck. Embedded Diesel migrations run at boot before any request is served; a migration failure aborts startup. No manual migration step.

External prerequisites (not provided by this repo): TLS termination in front of the app (Caddy/Nginx) with `PERSONA_COOKIE_SECURE=1` once HTTPS is live, and host-level disk encryption for the Docker volumes.

## 2. Backups

The in-process scheduler runs the daily backup job (`BACKUP_INTERVAL_SECONDS`, default 86400):

- `VACUUM INTO` writes a WAL-consistent snapshot to `$BACKUP_DIR/app-YYYYMMDD.db`. The compose file bind-mounts a host directory outside the `persona-data` volume: `${PERSONA_BACKUP_DIR:-./backups}:/backups`.
- `$BACKUP_DIR` is PHI space: the MinIO mirror holds raw clinical images and a failed encryption step can leave a plaintext SQLite snapshot. Create the host directory with mode `0700` before the first backup, keep it outside any synced or backed-up-as-code directory (git checkout, dotfiles repo, cloud-sync folders), and never commit it. The default `./backups` is excluded by the root `.gitignore` (`/backups/`, `*.db.age`). If you set `PERSONA_BACKUP_DIR` to another path, it carries the same rules: inside the checkout it must satisfy `git check-ignore -v "$PERSONA_BACKUP_DIR"`; outside the repo it still needs mode `0700` and exclusion from host backup/sync tooling.
- The snapshot is encrypted with `age` to `$BACKUP_AGE_RECIPIENT` (an `age1…` recipient) and the plaintext file is deleted on success. An unset recipient aborts before the snapshot is taken, so no plaintext is written, and the failure is recorded in `audit_log` (`backup_daily`); an `age` execution failure can leave the plaintext file on disk (see the verification note below — tracked in `specs/divergences/13-M-backup-file-safety.md`).
- The same cadence mirrors the MinIO bucket with `mc mirror` into `$BACKUP_DIR/minio-YYYYMMDD`. A DB-only backup breaks reprints because `template_version_assets` pins objects; the mirror is required, not optional.
- RPO ≤ 24 h, RTO ≤ 4 h. Restore: stop the app, decrypt with the age key, place the file at the `DATABASE_URL` path, restart, verify `/health` and a sign-in.

Verify backups: `ls backups/` shows `app-*.db.age` daily; `SELECT action, metadata_json, at FROM audit_log WHERE action = 'backup_daily' ORDER BY at DESC LIMIT 5;` shows `{"path": …, "encrypted": true}` per run. Treat that audit row as unconfirmed until you test-restore one file: an interrupted `age` run can leave the plaintext snapshot on disk, and the `mc mirror` result is not checked before success is recorded — known gaps tracked in `specs/divergences/13-M-backup-file-safety.md` and `specs/divergences/13-M-backup-minio-mirror.md`. Keep `backups/` out of git and mode-private (tracked in `13-M-backup-repo-exclusion.md`).

## 3. Key custody

| Key | Where | Notes |
|---|---|---|
| age recipient/key | Host env (`BACKUP_AGE_RECIPIENT`) + offline key copy | The server holds only the recipient. Decryption key stays off the server. |
| MinIO credentials | Compose env (`ASSETS_*`) | Rotate by changing env and recreating the stack. |
| Update signing key | `UPDATE_SIGNING_KEY` (base64 ed25519 seed), server-side only | Never shipped to clients. The matching public key is `UPDATE_PUBLIC_KEY` on desktop installs. Rotate by publishing a new public key before signing with the new key. |
| Apple Developer ID / Windows code-signing certificate | External, per release | Required for signed DMG/MSI; see spec 13-M divergence 1. |

## 4. Scheduled jobs

One scheduler runs in the server process. Jobs are idempotent and restart-safe; eligibility derives from row state, there is no job-state table, and every completed run writes one `audit_log` row.

| Job | Cadence | What it does |
|---|---|---|
| Session purge | `JOBS_INTERVAL_SECONDS` (default 3600) | Deletes `sessions` rows with `expires_at < now`. `get_session` still rejects expired sessions on every read. |
| Stale asset cleanup | same | Deletes `assets` rows with `status='pending'` created more than one hour ago, plus their objects. Object-delete failures are currently swallowed (tracked in `specs/divergences/13-M-stale-asset-cleanup.md`): if MinIO was down, verify the object is gone after the run. |
| Retention trim | same | Trims clinics past `data_grace_ends_at` (enabled by default, `RETENTION_TRIM_ENABLED=true`). Deletes only rows created after `server_retention_until`, children before parents, never a row referenced by a retained record. See spec 13-M objective 2 for the table list. |
| Daily backup | `BACKUP_INTERVAL_SECONDS` (default 86400) | Section 2 above. |

## 5. Retention and legal prerequisites

The trim job runs in v1 by default (D-12): a clinic loses server records created after `server_retention_until` once `now >= data_grace_ends_at`, unless the administrator exports first. The export lives at `/settings/clinic` → "Exportar datos (JSON)" (patients, prescriptions, items; no assets) and is audited (`clinic_export`).

Production prerequisite: counsel sign-off on prescription retention under MSP/ACESS rules and the Ley Orgánica de Protección de Datos Personales. The 5-year audit window is never trimmed. If counsel requires an encrypted archive of trimmed rows, that is a deliberate product decision, not an ops workaround; nothing is archived today.

## 6. Updates

Desktop installs check `UPDATE_MANIFEST_URL` at launch. The manifest (`version`, `rolloutPercent`, `url`, `signature`) is served by `GET /api/updates/manifest` and signed with the server-side ed25519 key over `version|rolloutPercent|url`. The client verifies signature and version before staging, honors the staged rollout bucket, and keeps the previous binary (`.prev`) for rollback. A bad signature is refused and the running version is untouched. Updates never carry clinical data.

Current caveats (tracked in `specs/divergences/13-M-update-channel.md` and `13-M-signed-installers.md`): rollout evaluation hashes an empty device id when `PERSONA_DEVICE_ID` is unset, so intermediate percentages act as 0% or 100%; the artifact download enforces no HTTPS and the manifest carries no artifact digest; installers are not yet signed/notarized. Until those close, publish only at `UPDATE_MANIFEST_ROLLOUT=0` (kill switch) or 100 to a controlled host over HTTPS.

## 7. Incidents

- **App down:** `docker compose ps`; `docker compose logs persona-sanus`. Recreate: `docker compose up -d --build`.
- **DB locked errors:** WAL + busy timeout are already configured; sustained lock contention means jobs plus traffic exceed a single SQLite file. Reduce `JOBS_INTERVAL_SECONDS` contention by raising the interval, then restore database from the latest backup if corruption is suspected.
- **MinIO unreachable:** the app still boots; `/api/assets` answers 503. Check `docker compose ps minio` and its healthcheck.
- **Bad update in rollout:** the manifest is the kill switch — set `UPDATE_MANIFEST_ROLLOUT=0` (or sign a manifest pointing at the previous version) and restart the app container. Clients that already staged keep `.prev` for rollback.
- **Trim ran before export:** restore from the latest encrypted backup to a scratch `DATABASE_URL`, export from there, and re-run. The trim itself writes per-clinic `retention_trim` audit rows with counts.

## 8. PRD §12 acceptance evidence

| Criterion | Evidence (2026-11-20, all commands run against the release server binary built by `python3 .blocks/persona-smoke.py --dir persona-sanus --port 8091`, exit 0) |
|---|---|
| Doctor creates patient, template, form, signs, prints A4 PDF with logo and active QR | `persona-probe --flow rx` (sign, `rx_number_in_block`), `--flow templates` (publish/immutable), `pdf-probe` (`signed_pdf`, `a4_media_box`, `qr_payload`) — all `ok: true` |
| Offline desktop flow, sync reconcile, final PDF with active QR | `reconnect-probe --port 8125` (`ok: true`, oldest-first drain, reconciled a/b, final `pdf_qr/a4`) + `persona-probe --flow sync` (reconcile idempotency); two earlier runs flaked on `drain` alone under extreme daemon load and were retried |
| Block exhaustion blocks signing with warning and remaining count | `persona-probe --flow rx` (`duplicate_allocation_refused`); `desktop_local.rs` `BlockExhausted` remaining count + `main.rs` warning |
| Unique monotonic numbers, burned tails recorded | `persona-probe --flow sync` (`burned_tail_recorded`, `reused_number_rejected`); `sqlite-schema-check --query "select … from rx_blocks"` |
| 30-day local cache eviction, on-demand regeneration | `sqlite-schema-check` (`retention_days = 30`); `desktop_local.rs` eviction cutoff + regeneration on miss |
| QR verification without PHI; provisional PDF has no scannable QR | `persona-probe --flow verify` (`no_phi`); `pdf-probe` (`qr_payload`); `pdf.rs` provisional branch (no QR bitmap) |
| LLM link carries drug name only | `main.rs` `llm_explain_url` (trimmed medication name only, into `clinics.llm_url_template`) |
| Print title `{Patient}-receta-{dd-mm-aaaa}-{Doctor}.pdf` | `pdf-probe` (`rfc5987_filename`) |
| Secretary cannot edit signed; nursing cannot sign; all audited | `persona-probe --flow roles` + `--flow security` (`member_forbidden`, `audit_listed`, `audit_admin_only`); `service.rs` Doctor-only sign + signed-edit guard |
| One 10 MB image; second and oversize rejected; logo free; offline refused | `asset-probe` (`quota_exceeded`, `oversized_rejected`, `logo_excluded_from_quota`) against native server + MinIO |
| Pinned image cannot be deleted | `asset-probe` (`template_publish_pins`, `pinned_delete_refused`) |
| 3-month retention: export offered, trim after 30-day grace | `sqlite-schema-check` (window = creation + 3 months, grace + 30 days); export `/settings/clinic`; trim per `specs/13-M-ops-and-release.md` §3 objective 2 |
| Auto-update only with valid signature, rollback on failure | BLOCKED — installer signing OPEN per `specs/divergences/13-M-signed-installers.md` (no Apple/Windows credentials); unchecked in `context/PRD.md` §12 |
