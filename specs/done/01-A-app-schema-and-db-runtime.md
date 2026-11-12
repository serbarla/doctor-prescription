# Spec 01 — Phase A: App schema and DB runtime

- Source: `context/database-design.md` §5 (SQLite DDL), §6, §8 (migrations); `context/architecture.md` §2, §6, §16, §18.
- Depends on: nothing.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Result carried forward: the live schema contains every non-billing table of the database design with its constraints and indexes, `src/schema.rs` matches it, and every pooled connection enforces foreign keys, WAL, and a busy timeout.

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

- [ ] 1. Extend `.blocks/sqlite-schema-check.py` to plain SQL and live databases — requirements: `--path` also accepts a `.sql` file and executes it verbatim (Markdown fenced blocks keep working), and a new `--db <path>` mode opens an existing SQLite file and runs `--query` against it without executing any script; when both `--path` and `--query` are given, the query runs in the same connection after the script executes; block header and the block index in `AGENTS.md` and `.blocks/INDEX.md` updated; commands: `python3 .blocks/sqlite-schema-check.py --path persona-sanus/migrations/20261111000000_create_auth/up.sql --query "select name from sqlite_master where type='table' order by name"`
- [ ] 2. Add the app-schema migration — requirements: `persona-sanus/migrations/<timestamp>_create_app_schema/{up,down}.sql` creating exactly the non-billing tables of `context/database-design.md` §5 in FK-safe order (`profiles`, `clinics` with `logo_asset_id` NULL and `rx_block_size INTEGER NOT NULL DEFAULT 50`, `assets`, `clinic_members`, `devices`, `doctors`, `patients`, `templates`, `template_versions`, `template_version_assets`, `rx_counters`, `rx_blocks`, `prescriptions`, `prescription_items`, `verification_codes`, `audit_log`) with the documented CHECK constraints, indexes, and partial unique indexes (`uq_clinic_members_profile`, `uq_patients_national_id`, `uq_prescriptions_rx`, `uq_rx_blocks_active`); `src/schema.rs` updated in the same change; `down.sql` drops exactly what `up.sql` creates in reverse FK order (children before parents); no comments in SQL; commands: `python3 .blocks/sqlite-schema-check.py --path persona-sanus/migrations/<timestamp>_create_app_schema/up.sql --query "select name from sqlite_master where type='table' order by name"`
- [ ] 3. Centralize connection pragmas — requirements: an r2d2 `CustomizeConnection` in `persona-sanus/src/db.rs` sets `PRAGMA foreign_keys = ON`, `journal_mode = WAL`, `busy_timeout = 5000`, `synchronous = NORMAL` on every connection; `auth_server::with_conn` no longer issues its own pragma so exactly one enforcement path exists; `/health` gains `foreign_keys` (bool) and `journal_mode` (string) fields read from a pooled connection so the settings are observable through the running app (`PRAGMA foreign_keys` is per-connection and not observable from a second connection); commands: `cargo check --manifest-path persona-sanus/Cargo.toml`
- [ ] 4. Add write-transaction helpers — requirements: `db` exposes `write_transaction` (deferred begin) and `immediate_transaction` (raw `BEGIN IMMEDIATE`) helpers that run a write transaction and retry on SQLite `database is locked` with bounded attempts; `immediate_transaction` is the only path specs 06-F (quota reservation) and 07-G (block allocation) may use for `BEGIN IMMEDIATE` (Diesel's `transaction()` emits a deferred `BEGIN` and would race); no module builds its own retry loop; no comments in code; commands: `cargo check --manifest-path persona-sanus/Cargo.toml`
- [ ] 5. Extend `.blocks/persona-smoke.py` to keep its database — requirements: a `--db-path <path>` param fixes `DATABASE_URL` to that file and leaves it in place after the run (default behavior unchanged: throwaway tempdir); persona-smoke additionally asserts `/health` reports `foreign_keys: true` and `journal_mode: "wal"`; block header and index updated; commands: `python3 .blocks/persona-smoke.py --dir persona-sanus --port 8091 --db-path /tmp/persona-verify.db`

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No billing tables (`tax_rates`, `plans`, `billing_accounts`, `payment_methods`, `subscriptions`, `payment_transactions`, `payment_events`) — v1.1 only (PRD D-04).
- Do not edit `persona-sanus/migrations/20261111000000_create_auth/` or the auth-owned `users`/`sessions` shape.
- No clinical service, HTTP handler, or UI code (the `/health` field additions and the `auth_server` pragma removal required by objective 3 are in scope); no schema drift from `context/database-design.md` §5.
- No unit tests; no comments in any code.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [ ] `python3 .blocks/persona-smoke.py --dir persona-sanus --port 8091` passes (fmt, check, clippy, release build, auth probe)
- [ ] `python3 .blocks/persona-docker-verify.py --dir . --port 8080` passes (compose build, boot applies embedded migrations, health, auth persistence)
- [ ] objective 1: `python3 .blocks/sqlite-schema-check.py --path persona-sanus/migrations/20261111000000_create_auth/up.sql --query "select name from sqlite_master where type='table' order by name"` → `sessions`, `users`
- [ ] objective 2: `python3 .blocks/sqlite-schema-check.py --path persona-sanus/migrations/<timestamp>_create_app_schema/up.sql --query "select name from sqlite_master where type='table' order by name"` → exactly the 16 non-billing tables of `context/database-design.md` §5
- [ ] objective 3: `python3 .blocks/persona-smoke.py --dir persona-sanus --port 8091 --db-path /tmp/persona-verify.db` → health assertions pass (`foreign_keys: true`, `journal_mode: "wal"`)
- [ ] objective 4: `cargo check --manifest-path persona-sanus/Cargo.toml` passes and both helpers exist in `db`; `immediate_transaction` is exercised by specs 06-F/07-G, and the FK setting is proven through the app's own pool (health assertion above), never a second connection
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
file to `specs/done/` (e.g. `specs/done/01-A-app-schema-and-db-runtime.md`).
Specs with open divergences stay in `specs/` next to `TEMPLATE.md` until the
tracking file is cleared.*
