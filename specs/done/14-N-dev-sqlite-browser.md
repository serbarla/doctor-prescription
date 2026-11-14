# Spec 14 — Phase N: Dev SQLite browser

- Source: developer request — mount a SQLite browser when running `dx serve --fullstack`.
- Depends on: `13-M-ops-and-release`.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Result carried forward: debug fullstack builds serve a read-only SQLite browser at `/dev/db`; release builds contain no such route.

## Before you start

Unit testing is not required; avoid it. Prioritize end-to-end and integration
testing only.

Read the nearest `AGENTS.md` (target workspace, then parents up to repo root)
before executing.

If reality diverges from this spec: STOP, append a divergence (section 4), and
NEVER improvise outside the objectives.

## 1. Objectives

- [x] 1. Add a debug-only dev database module — requirements: `src/dev_db.rs` opens `DATABASE_URL` with `SQLITE_OPEN_READONLY`, implements one-statement and `sqlite3_stmt_readonly` guards, returns JSON table lists, column metadata, paginated rows worth at most 500 rows per request, and query results; commands: `cd persona-sanus && cargo check --features server`.
- [x] 2. Mount the browser only in debug builds — requirements: `main.rs` declares `dev_db` behind `feature = "server"`, non-wasm, and `debug_assertions`; `server.rs` merges its router and logs the mount path; commands: `cd persona-sanus && cargo check --features server`.
- [x] 3. Ship the browser UI and document it — requirements: inline HTML at `/dev/db` with table list, paginated data grid, schema panel, and read-only SQL console; README documents the debug-only route; commands: `cd persona-sanus && cargo check --features server`.
- [x] 4. Prove the live surface — requirements: `dx serve --fullstack` exposes `/dev/db`, `/dev/db/api/tables`, and `/dev/db/api/table?name=users`; a write statement is rejected; commands: `curl -sf http://127.0.0.1:8080/dev/db >/dev/null`, `curl -sf http://127.0.0.1:8080/dev/db/api/tables`, `curl -sG --data-urlencode 'sql=DELETE FROM users' http://127.0.0.1:8080/dev/db/api/query`.

## 2. No-go

- No new crates and no write endpoints.
- No change to release routing, authentication, CSP for other routes, or Docker images.
- No dependency on the `sqlite3` CLI being installed.

## 3. Testing criteria

- [x] `cd persona-sanus && cargo check --features server` passes.
- [x] `curl -sf http://127.0.0.1:8080/dev/db >/dev/null` returns the browser page while `dx serve --fullstack` runs.
- [x] `curl -sf http://127.0.0.1:8080/dev/db/api/tables` lists the live schema.
- [x] `curl -sf 'http://127.0.0.1:8080/dev/db/api/table?name=users&limit=5'` returns columns and rows.
- [x] `curl -sG --data-urlencode 'sql=DELETE FROM users' http://127.0.0.1:8080/dev/db/api/query` returns a read-only rejection.
- [x] objective 4: the three live route checks above pass.
- [x] every objective in section 1 is checked

## 4. Divergences

- [x] every divergence found is appended below (or the section reads "None.")

None.
