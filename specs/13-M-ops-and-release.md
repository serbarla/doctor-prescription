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

- [ ] 1. Build the in-process scheduler — requirements: one scheduler in the server process with idempotent, restart-safe jobs whose eligibility derives from row state (no job-state table): expired-session purge (replacing the lazy delete in `get_session`, which still rejects `expires_at < now` on every read), stale `pending` asset cleanup (rows older than one hour, row and object deleted), and retention trim; each completed run writes one `audit_log` row so re-running is provably idempotent; commands: `cargo check --manifest-path persona-sanus/Cargo.toml`
- [ ] 2. Implement the retention trim with export-first — requirements: runs by default in v1 (`RETENTION_TRIM_ENABLED=true`, D-12) only when `now >= clinics.data_grace_ends_at`, deleting rows created after `clinics.server_retention_until` only (never the first 3 months), children before parents, never a row referenced by a retained record; eligible tables (in order): `prescription_items`, `prescriptions`, `verification_codes`, `patients` (only `deleted_at` set and no referenced prescriptions), `templates`/`template_versions` (only versions no retained prescription pins), `template_version_assets`, `assets` (only `deleted_at` set and unreferenced); `profiles`, `clinics`, `clinic_members`, `doctors`, `devices`, `rx_counters`, `rx_blocks`, `users`, `sessions`, and `audit_log` are never trimmed; an export of the clinic's data (JSON: patients, prescriptions, items; no assets) is offered before trimming (ARCO, PRD §9) and an audited export action exists under `/settings/clinic`; no encrypted archive is kept; commands: `cargo check --manifest-path persona-sanus/Cargo.toml`
- [ ] 3. Add operational backups — requirements: an encrypted daily backup of the SQLite file via `VACUUM INTO` (WAL-consistent snapshot, never a raw file copy) date-stamped to a host bind mount outside the `persona-data` volume (`docker-compose.yml` adds the mount), encrypted with a server-held key (`age` or equivalent, custody documented in the runbook) meeting RPO ≤ 24 h and RTO ≤ 4 h; the daily job also backs up the MinIO bucket (same cadence, via the S3 client or `mc mirror`) because `template_version_assets` pins objects — a DB-only backup breaks reprints; `docker-compose.yml` healthchecks and restart policy verified; TLS termination and secret handling documented as external prerequisites; commands: `python3 .blocks/persona-docker-verify.py --dir . --port 8080`
- [ ] 4. Ship signed desktop installers with auto-update — requirements: signed, notarized macOS DMG and signed Windows MSI — the signing/notarization credentials (Apple Developer ID, Windows code-signing certificate) are external prerequisites; if they are unavailable, record an OPEN divergence in section 4 instead of faking this gate; in-app auto-update: name the mechanism in the implementation (e.g. a `self_update`-style downloader with an ed25519/minisign-signed manifest — no updater ships in Dioxus today) consuming the signed manifest at `UPDATE_MANIFEST_URL` over HTTPS with staged rollout; the client verifies signature and version before applying and retains the previous version for rollback (D-08); a bad signature or failed launch rolls back automatically; updates never carry clinical data; commands: `cd persona-sanus && cargo check --no-default-features --features desktop`
- [ ] 5. Write the runbook and verify acceptance — requirements: `context/runbook.md` covering deploy, backup/restore, key custody, retention/legal prerequisites (counsel sign-off), and incident steps; then walk the `context/PRD.md` §12 acceptance checklist and record each item's evidence; commands: `python3 .blocks/persona-docker-verify.py --dir . --port 8080`

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No billing-driven trimming (`locked`/`canceled` restrictions, `data_grace_ends_at` set at lock) — v1.1 only (FR-069).
- No Linux desktop target; no horizontal scaling or external database.
- Do not delete audit rows before the 5-year window; no trim of the first 3 months of clinic data.
- No unit tests; no comments in any code.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [ ] `python3 .blocks/persona-smoke.py --dir persona-sanus --port 8091` passes (fmt, check, clippy, release build, auth probe)
- [ ] `python3 .blocks/persona-docker-verify.py --dir . --port 8080` passes (compose build, boot, health, auth persistence)
- [ ] objective 2: with fixture rows past `data_grace_ends_at`, running the trim deletes only rows created after `server_retention_until` (`python3 .blocks/sqlite-schema-check.py --db <booted-db> --query "select min(created_at) from prescriptions"` → ≥ `server_retention_until`) and writes an audit row; the export action produces the clinic's data before deletion
- [ ] objective 1: jobs are idempotent — running each twice changes nothing beyond one audit row per run; stale `pending` assets are removed after one hour
- [ ] objective 4: an update manifest with a bad signature is refused and the previous version still runs; a valid staged manifest applies and can roll back
- [ ] objective 5: every `context/PRD.md` §12 acceptance criterion has recorded evidence (checked box + where it was proven)
- [ ] every objective in section 1 is checked

## 4. Divergences

*Process and tracking-file rules: see `AGENTS.md` (§ Spec-Driven Development).
Summary: when reality diverges from the spec, append a numbered entry below;
leave the matching testing-criteria `- [ ]` unchecked and note the divergence
inline next to it. Open divergences owned by a later spec also get a tracking
file in `specs/divergences/`.*

- [ ] every divergence found is appended below (or the section reads "None.")

*Entry format (one numbered item per divergence):*

1. <what diverged and its current state> — <why it could not be done as written, citing the No-go rule or spec that blocks it> (Verified <date>: <N> fix attempts — <attempt 1>, <attempt 2>, <attempt 3> — all fenced by <constraint>; <impact> is deferred to <later spec> by design, not a defect.)

### Divergences

None.

---

*Housekeeping: once a spec is fully done with no open divergences, move its
file to `specs/done/` (e.g. `specs/done/13-M-ops-and-release.md`).
Specs with open divergences stay in `specs/` next to `TEMPLATE.md` until the
tracking file is cleared.*
