# Spec 04 — Phase D: Devices and auth transport

- Source: `context/architecture.md` §7 (auth transport defect and decision), §15, §17 risk 1 (P0); `context/PRD.md` §9 (devices), D-20; `context/database-design.md` §2, §6, §8 (`sessions.device_id`).
- Depends on: 01-A, 02-B.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Result carried forward: auth works on web and desktop (relative base on wasm, configurable origin + cookie jar + credentialed CORS on native), and registered devices are linked to sessions and revocable.

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

- [ ] 1. Fix the auth transport — requirements: `persona-sanus/src/auth.rs` uses a relative `/api/auth` base on wasm and `PERSONA_SERVER_ORIGIN` (default `http://127.0.0.1:8080`) on native; the native client carries a `reqwest` cookie store (`reqwest` gains the `cookies` feature in `Cargo.toml`); the server allows credentialed CORS for the web origins listed in `PERSONA_WEB_ORIGINS` (comma-separated, empty = same-origin only — the desktop's native `reqwest` is not a browser and needs no CORS, and the session cookie stays `SameSite=Lax`); the hardcoded loopback constant is gone; `/api/auth/*` shapes unchanged; commands: `cargo check --manifest-path persona-sanus/Cargo.toml && python3 .blocks/persona-probe.py --base http://127.0.0.1:8080 --flow full`
- [ ] 2. Implement the device lifecycle — requirements: an auth migration adds nullable `sessions.device_id`; `register_device` (platform `macos`/`windows`/`web`, name, `app_version`, `last_seen_at`), `list_devices`, `revoke_device` (sets `revoked_at`, ends the profile's sessions on that device, audited) under `/api/devices`; the client persists its `device_id` (localStorage on web, app-data file on desktop), calls `register_device` idempotently with the same id every app start, and the server links the current session (`sessions.device_id`) to that device — a fresh sign-in re-links the new session to the stored device so specs 07-G/11-K can attribute number blocks; session validation rejects a session whose device is revoked or missing, while sessions with NULL `device_id` stay valid (web before the first block allocation, D-20); commands: `cargo check --manifest-path persona-sanus/Cargo.toml`
- [ ] 3. Register web clients as devices — requirements: before its first prescription-number block allocation a web client creates a `devices` row with `platform='web'` (PRD D-20); the registration call is shared with the desktop path; the allocation hook itself lands in spec 07; commands: `cargo check --manifest-path persona-sanus/Cargo.toml`
- [ ] 4. Manage devices in `/settings/users` — requirements: the Administrator sees each profile's devices (name, platform, last seen, revoked) and can revoke one, with immediate session rejection; `es-EC` labels; commands: `cd persona-sanus && dx build --release --fullstack`
- [ ] 5. Extend `.blocks/persona-probe.py` with `--flow devices` — requirements: register a device, link it to the session, list it, revoke it, prove the revoked device's session is rejected, and prove a fresh sign-in re-links the session to the same stored device id; block header and indexes updated; commands: `python3 .blocks/persona-probe.py --base http://127.0.0.1:8080 --flow devices`

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No MFA, rate limiting, or security headers (spec 12-L).
- No prescription-number blocks (spec 07-G), no sync or outbox code (specs 10-J, 11-K).
- Do not change `/api/auth/*` request/response shapes (persona-probe `--flow full` must stay green).
- No billing or entitlement code (v1.1 only).
- No unit tests; no comments in any code.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [ ] `python3 .blocks/persona-smoke.py --dir persona-sanus --port 8091` passes (fmt, check, clippy, release build, auth probe)
- [ ] `python3 .blocks/persona-docker-verify.py --dir . --port 8080` passes (compose build, boot, health, auth persistence)
- [ ] objective 1: `python3 .blocks/persona-probe.py --base http://127.0.0.1:8080 --flow full` → all checks true on the server whose origin is not loopback-bound (compose `IP=0.0.0.0`)
- [ ] objective 5: `python3 .blocks/persona-probe.py --base http://127.0.0.1:8080 --flow devices` → all checks true (register, link, list, revoke, session rejected)
- [ ] objective 4: `http://127.0.0.1:8080/settings/users` in a browser → devices listed with last-seen; revoking one signs that client out on its next request
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
file to `specs/done/` (e.g. `specs/done/04-D-devices-and-auth-transport.md`).
Specs with open divergences stay in `specs/` next to `TEMPLATE.md` until the
tracking file is cleared.*
