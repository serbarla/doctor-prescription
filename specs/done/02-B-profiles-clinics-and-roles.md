# Spec 02 — Phase B: Profiles, clinics, and role authorization

- Source: `context/PRD.md` §3 (users and roles), §6 (`profiles`, `clinics`, `clinic_members`, `doctors`); `context/architecture.md` §8, §16; `context/sitemap.md` §1.2 (`/settings*`); `context/database-design.md` §2, §6.
- Depends on: 01-A.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Result carried forward: every app read and write passes through a server-side authorized service layer keyed on `clinic_members.role`, the first sign-up bootstraps the clinic administrator, and `/settings/users` + `/settings/profile` manage roles and doctor details.

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

- [x] 1. Create the authorized service layer — requirements: new server-gated module(s) following the `auth_server` seam (`server::router` wires state, handlers speak HTTP, `db` holds SQL): an `ActorContext` resolved from the session (profile, clinic, `clinic_members.role`), role checks inside every function, and one audit helper appending `audit_log` rows (`actor_id` takes `Option<actor>` — NULL for system/public writes like the spec 09-I verification attempts, plus `action`, `entity`, `entity_id`, `metadata_json`, `at`); `ActorContext` resolution rejects disabled profiles (`profiles.status='disabled'`, 403) before any service call; app data is exposed as REST under `/api/*` with the session cookie (configurable origin + cookie jar + credentialed CORS per `context/architecture.md` §7), never Diesel or axum types in shared client structs; no comments in code; commands: `cargo check --manifest-path persona-sanus/Cargo.toml`
- [x] 2. Implement clinic, membership, and profile mutations — requirements: `register_user`, `create_clinic` (sets `server_retention_until` = creation + 3 calendar months and `data_grace_ends_at` = that + 30 days per PRD D-09), `set_member_role`, `remove_member`, `update_profile`, `update_doctor` (specialty, `license_no`, cédula, phone, email); `clinic_members.role` is the only role authority (D-03) and one clinic per profile (`uq_clinic_members_profile`); assigning role `doctor` upserts the `doctors` row (specs 05-E/07-G need `doctors.id` as owner and signer); `update_profile` writes `profiles.display_name` and mirrors it into auth `users.display_name` so exactly one visible name exists; each mutation writes an audit row; commands: `cargo check --manifest-path persona-sanus/Cargo.toml`
- [x] 3. Wire the bootstrap flow — requirements: the first sign-up on an install with no `clinics` row creates the clinic and an `administrator` membership (`profiles.created_by` NULL) in a `BEGIN IMMEDIATE` transaction with a re-check (two concurrent first sign-ups cannot create two clinics); the bootstrap clinic is named `Clínica de {display_name}` with the default `America/Guayaquil` timezone (the Administrator renames it in `/settings/clinic`, spec 06-F); every later sign-up creates `users` + `profiles` with no membership until an administrator assigns a role; `POST /api/auth/sign-up/email` request/response shapes stay unchanged; commands: `python3 .blocks/persona-probe.py --base http://127.0.0.1:8080 --flow full`
- [x] 4. Build the settings surface — requirements: `GET /api/me` (profile + clinic + role), `/settings` hub that redirects non-administrators to `/settings/profile`, `/settings/users` (Administrator only: list profiles, assign/revoke the four roles, disable profiles), `/settings/profile` (display name and `doctors` fields); Navbar renders only role-permitted items; `es-EC` labels and `DD-MM-AAAA` dates; commands: `cd persona-sanus && dx build --release --fullstack`
- [x] 5. Extend `.blocks/persona-probe.py` with `--flow roles` — requirements: exercises bootstrap sign-up, second sign-up stays unaffiliated, administrator assigns a role, a non-administrator calling the same API is refused server-side (HTTP 403-class, not merely hidden UI), and the unaffiliated user sees the no-clinic state; this flow establishes the probe convention every later flow follows: deterministic unique emails derived from the flow name per run and idempotent re-runs (tolerate or clean up existing fixtures); block header and indexes updated; commands: `python3 .blocks/persona-probe.py --base http://127.0.0.1:8080 --flow roles`

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No patients, templates, prescriptions, or asset code or routes (specs 03, 05, 06, 07).
- No MFA, rate limiting, security headers, or `/audit` UI (spec 12-L).
- No billing, subscription, or entitlement code (v1.1 only, PRD D-04).
- Do not change `/api/auth/*` request/response shapes (persona-probe `--flow full` must stay green).
- No unit tests; no comments in any code.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [x] `python3 .blocks/persona-smoke.py --dir persona-sanus --port 8091` passes (fmt, check, clippy, release build, auth probe)
- [x] `python3 .blocks/persona-docker-verify.py --dir . --port 8080` passes (compose build, boot, health, auth persistence)
- [x] objective 3: `python3 .blocks/persona-probe.py --base http://127.0.0.1:8080 --flow full` → all checks true (auth contract intact)
- [x] objective 5: `python3 .blocks/persona-probe.py --base http://127.0.0.1:8080 --flow roles` → all checks true (bootstrap, membership, server-side refusal, no-clinic state)
- [x] objective 4: `http://127.0.0.1:8080/settings/users` in a browser as administrator → role management works; as secretary → refused state; `http://127.0.0.1:8080/settings/profile` → doctor fields save and reload
- [x] every objective in section 1 is checked

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

None.

---

*Housekeeping: once a spec is fully done with no open divergences, move its
file to `specs/done/` (e.g. `specs/done/02-B-profiles-clinics-and-roles.md`).
Specs with open divergences stay in `specs/` next to `TEMPLATE.md` until the
tracking file is cleared.*
