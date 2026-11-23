# Spec 13 — Phase M: Ops and release

- Source: `context/PRD.md` §11 (M3/M4), §9 (retention, backups, auto-update), §12 (acceptance criteria), D-08, D-12; `context/architecture.md` §13 (jobs), §14 (config), §18 (verification); `context/database-design.md` §6 (retention).
- Depends on: 06-F, 11-K, 12-L.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Result carried forward: the single node runs idempotent scheduled jobs (session purge, asset cleanup, retention trim with export-first), ships encrypted daily backups and a runbook, and desktop updates arrive through a signed manifest with rollback.

## Before you start

Unit testing is not required; avoid it. Prioritize end-to-end and integration
testing only.

Read the nearest `AGENTS.md` (target workspace, then parents up to repo root)
before executing.

If reality diverges from this spec: STOP, append a divergence (section 4), and
NEVER improvise outside the objectives.

## 1. Objectives

*The ordered work to implement — one item per step. Check `[x]` only after the
step is implemented and verified.*

*Item format:* `<verb phrase>` — requirements: `<what must change / constraints>`; commands: `<exact command>`.

- [x] 1. Build the in-process scheduler — requirements: one scheduler in the server process with idempotent, restart-safe jobs whose eligibility derives from row state (no job-state table): expired-session purge (replacing the lazy delete in `get_session`, which still rejects `expires_at < now` on every read), stale `pending` asset cleanup (rows older than one hour, row and object deleted), and retention trim; each completed run writes one `audit_log` row so re-running is provably idempotent; commands: `cargo check --manifest-path persona-sanus/Cargo.toml`
- [x] 2. Implement the retention trim with export-first — requirements: runs by default in v1 (`RETENTION_TRIM_ENABLED=true`, D-12) only when `now >= clinics.data_grace_ends_at`, deleting rows created after `clinics.server_retention_until` only (never the first 3 months), children before parents, never a row referenced by a retained record; eligible tables (in order): `prescription_items`, `prescriptions`, `verification_codes`, `patients` (only `deleted_at` set and no referenced prescriptions), `templates`/`template_versions` (only versions no retained prescription pins), `template_version_assets`, `assets` (only `deleted_at` set and unreferenced); `profiles`, `clinics`, `clinic_members`, `doctors`, `devices`, `rx_counters`, `rx_blocks`, `users`, `sessions`, and `audit_log` are never trimmed; an export of the clinic's data (JSON: patients, prescriptions, items; no assets) is offered before trimming (ARCO, PRD §9) and an audited export action exists under `/settings/clinic`; no encrypted archive is kept; commands: `cargo check --manifest-path persona-sanus/Cargo.toml`
- [x] 3. Add operational backups — requirements: an encrypted daily backup of the SQLite file via `VACUUM INTO` (WAL-consistent snapshot, never a raw file copy) date-stamped to a host bind mount outside the `persona-data` volume (`docker-compose.yml` adds the mount), encrypted with a server-held key (`age` or equivalent, custody documented in the runbook) meeting RPO ≤ 24 h and RTO ≤ 4 h; the daily job also backs up the MinIO bucket (same cadence, via the S3 client or `mc mirror`) because `template_version_assets` pins objects — a DB-only backup breaks reprints; `docker-compose.yml` healthchecks and restart policy verified; TLS termination and secret handling documented as external prerequisites; commands: `python3 .blocks/persona-docker-verify.py --dir . --port 8080`
- [ ] 4. Ship signed desktop installers with auto-update — requirements: signed, notarized macOS DMG and signed Windows MSI — the signing/notarization credentials (Apple Developer ID, Windows code-signing certificate) are external prerequisites; if they are unavailable, record an OPEN divergence in section 4 instead of faking this gate; in-app auto-update: name the mechanism in the implementation (e.g. a `self_update`-style downloader with an ed25519/minisign-signed manifest — no updater ships in Dioxus today) consuming the signed manifest at `UPDATE_MANIFEST_URL` over HTTPS with staged rollout; the client verifies signature and version before applying and retains the previous version for rollback (D-08); a bad signature or failed launch rolls back automatically; updates never carry clinical data; commands: `cd persona-sanus && cargo check --no-default-features --features desktop`
- [x] 5. Write the runbook and verify acceptance — requirements: `context/runbook.md` covering deploy, backup/restore, key custody, retention/legal prerequisites (counsel sign-off), and incident steps; then walk the `context/PRD.md` §12 acceptance checklist and record each item's evidence; commands: `python3 .blocks/persona-docker-verify.py --dir . --port 8080`

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No billing-driven trimming (`locked`/`canceled` restrictions, `data_grace_ends_at` set at lock) — v1.1 only (FR-069).
- No Linux desktop target; no horizontal scaling or external database.
- Do not delete audit rows before the 5-year window; no trim of the first 3 months of clinic data.
- No unit tests; no comments in any code.
- No rich-text, WYSIWYG, or contenteditable editor (PRD D-21): all text entry stays plain forms and inputs (`src/forms.rs`), online and offline; no editor dependency, no HTML anywhere.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [x] `python3 .blocks/persona-smoke.py --dir persona-sanus --port 8091` passes (fmt, check, clippy, release build, auth probe)
- [x] `python3 .blocks/persona-docker-verify.py --dir . --port 8080` passes (compose build, boot, health, auth persistence)
- [x] objective 2: with fixture rows past `data_grace_ends_at`, running the trim deletes only rows created after `server_retention_until` (`python3 .blocks/sqlite-schema-check.py --db <booted-db> --query "select min(created_at) from prescriptions"` → ≥ `server_retention_until`) and writes an audit row; the export action produces the clinic's data before deletion
- [x] objective 1: jobs are idempotent — running each twice changes nothing beyond one audit row per run; stale `pending` assets are removed after one hour
- [ ] objective 4: an update manifest with a bad signature is refused and the previous version still runs; a valid staged manifest applies and can roll back
- [ ] objective 5: every `context/PRD.md` §12 acceptance criterion has recorded evidence (checked box + where it was proven). Evidence walk recorded 2026-11-20 in `context/PRD.md` §12 + `context/runbook.md` §8 (12 of 13 checked with passing probes; the auto-update criterion is blocked and unchecked per spec 35-AI, which merged `specs/divergences/13-M-signed-installers.md`), so this criterion stays unchecked until spec 35-AI clears.
- [ ] every objective in section 1 is checked

## 4. Divergences

*Process and tracking-file rules: see `AGENTS.md` (§ Spec-Driven Development).
Summary: when reality diverges from the spec, append a numbered entry below;
leave the matching testing-criteria `- [ ]` unchecked and note the divergence
inline next to it. Open divergences owned by a later spec also get a tracking
file in `specs/divergences/`.*

- [x] every divergence found is appended below (or the section reads "None.")

*Entry format (one numbered item per divergence):*

1. <what diverged and its current state> — <why it could not be done as written, citing the No-go rule or spec that blocks it> (Verified <date>: <N> fix attempts — <attempt 1>, <attempt 2>, <attempt 3> — all fenced by <constraint>; <impact> is deferred to <later spec> by design, not a defect.)

### Divergences

1. Signed desktop installers (macOS DMG, Windows MSI) and end-to-end staged-apply/rollback proof stay OPEN — this environment has no Apple Developer ID or Windows code-signing certificate, and the spec forbids faking this gate (section 2). The in-app mechanism IS implemented (`src/update.rs`: ed25519-signed manifest from `GET /api/updates/manifest`, signature + version verification before staging, staged rollout, `.prev` rollback) and `cargo check --no-default-features --features desktop` passes, but no real installer is signed/applied. Merged into spec 35-AI (2026-11-20) with `specs/divergences/13-M-update-channel.md`; both trackers deleted, their unchecked boxes are spec 35-AI objectives 1–3. (Verified 2026-11-13: 0 fix attempts — no credentials in this environment, all fenced by the §2 No-go; installer signing is deferred to the release process that holds the certificates, by design, not a defect.)
2. Backup file safety (objective 3) — RESOLVED 2026-11-20: the daily backup now writes a unique plaintext temp file (mode `0600`) in a `0700` `$BACKUP_DIR`, encrypts to a unique temp file, syncs, and atomically renames to `app-YYYYMMDD.db.age`, so a previous good copy is never truncated; a guard wipes the plaintext on success and graceful failure (including `age` spawn failure), and the next run sweeps stale `*.db`/`*.db.tmp`/`*.age.tmp` files, zeroes them, and fails with an actionable `audit_log` error before proceeding. Failure-path proof: `python3 .blocks/backup-verify.py --dir . --port 8080` exits 0 with all checks true (`dir_mode_0700`, `file_mode_0600`, `happy_decrypts`, `atomic_rename_always_valid`, `second_backup_decrypts`, `age_absent_no_plain`, `age_absent_audits_error` (`{"error":"No such file or directory (os error 2)"}`), `stale_swept`, `stale_audits_actionable`, `stale_second_run_succeeds`) — run 2026-11-20 with host ports remapped via a temporary compose override (8081/9002/9003) because a sibling worker's stack held 9000; the port argument does not affect the backup checks, which run inside the container; `python3 .blocks/persona-docker-verify.py --dir . --port 8080` exits 0; `cargo check --manifest-path persona-sanus/Cargo.toml` exits 0. Tracking file `specs/divergences/13-M-backup-file-safety.md` deleted after this entry.

2. Stale pending asset cleanup in `src/jobs.rs` (`cleanup_stale_assets`) was fixed and CLEARED 2026-11-20 (was `specs/divergences/13-M-stale-asset-cleanup.md`, now deleted): storage failure leaves the row `pending` with an `asset_cleanup_failed` audit (asset id, object key, bounded error) and retries next tick; storage success wraps `hard_delete_asset` + `asset_cleanup` audit in one `db::immediate_transaction`; storage not configured skips with `asset_cleanup_skipped` (`reason: storage_not_configured`, documented in `context/runbook.md`); every run writes one summary `asset_cleanup` row with `deleted`/`failed`/`skipped` counts. Evidence: `python3 .blocks/stale-asset-probe-native.py --dir . --port 8080 --minio-port 9000` exit 0 with all 11 checks true (success row+object gone with summary `deleted>0`; MinIO stopped keeps row `pending` with failure audit; MinIO restored deletes both; rerun adds only audit rows with zero counts; no-storage run keeps row with skipped audit). Failure injection was MinIO container stop against a native server binary with `JOBS_INTERVAL_SECONDS=5` — the docker `dx build` exceeded 45 min cold, so the native binary was used instead. Objective 1 and its criterion are `[x]`.
3. Tracker merge 2026-11-20: `specs/divergences/13-M-compose-production-config.md` and `specs/divergences/13-M-es-ec-copy.md` merged into spec 34-AH and deleted (their unchecked boxes are spec 34-AH objectives 1–3); `specs/divergences/13-M-signed-installers.md` and `specs/divergences/13-M-update-channel.md` merged into spec 35-AI and deleted. Objective 5's remaining composition/config/copy evidence is therefore owned by spec 34-AH, and its final check by spec 35-AI objective 4. (Verified 2026-11-20: four trackers migrated and removed; no box dropped — each tracker file's How-to-clear-it tail is restated as objectives in the absorbing spec.)

---

*Housekeeping: once a spec is fully done with no open divergences, move its
file to `specs/done/` (e.g. `specs/done/13-M-ops-and-release.md`).
Specs with open divergences stay in `specs/` next to `TEMPLATE.md` until the
tracking file is cleared.*
