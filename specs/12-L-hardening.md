# Spec 12 — Phase L: Hardening

- Source: `context/PRD.md` §9 (security, privacy, compliance), §11 M3; `context/architecture.md` §5 (security findings), §15 (security boundaries), §17 risks 9/10; `context/sitemap.md` §1.2 (`/audit`, `/login`).
- Depends on: 02-B, 04-D, 09-I.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Result carried forward: the known auth findings are closed, doctors and administrators sign in with MFA, abuse controls cover auth and verification, and the Administrator has the 5-year append-only audit view.

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

- [ ] 1. Close the auth security findings — requirements: the session token is no longer returned in the JSON body for cookie transport (`context/architecture.md` §5 table); sign-in returns uniform errors with a dummy Argon2 hash for unknown emails (no enumeration or timing signal; sign-up keeps its `409` duplicate — enumeration is inherent in account creation); `Origin` is validated on state-changing routes **when present** — requests without an `Origin` header (native `reqwest`, spec 04-D) are allowed, and allowlisted origins come from `PERSONA_WEB_ORIGINS`; `tower-http` adds security headers (HSTS only behind TLS, `PERSONA_COOKIE_SECURE=1`) and CSP with `script-src 'self' 'wasm-unsafe-eval'` plus the inline-script policy Dioxus `document::eval` needs (verified against a signed-in page with the spec 05-E forms mounted — plain Dioxus forms add no scripts), and CORS consistent with the spec 04-D origin model; commands: `cargo check --manifest-path persona-sanus/Cargo.toml && python3 .blocks/persona-probe.py --base http://127.0.0.1:8080 --flow full`
- [ ] 2. Add MFA and session hygiene — requirements: MFA for Doctor and Administrator roles on `/login` (TOTP, RFC 6238, per user in a new auth migration; enrollment on `/settings/profile`; PRD §9 M3), a new `POST /api/auth/change-password` endpoint (session-authed) that rotates the session token on success, and expiring sessions already enforced at 30 days; `persona-probe` enrolls MFA with a fixed secret and computes codes with stdlib `hmac`/`hashlib`, and every existing flow is updated to complete MFA for Administrator/Doctor accounts — otherwise `--flow full`, `--flow signin`, and `persona-docker-verify` break the moment MFA lands; commands: `cargo check --manifest-path persona-sanus/Cargo.toml`
- [ ] 3. Rate-limit the abuse surfaces — requirements: per-IP limits on `/api/auth/*` (`AUTH_RATE_LIMIT_PER_MINUTE`, `governor`/`tower_governor`, in addition to the spec 09-I verification limit) returning `429`; auth attempts are audited; commands: `cargo check --manifest-path persona-sanus/Cargo.toml`
- [ ] 4. Build `/audit` — requirements: Administrator-only route listing `audit_log` (actor, action, entity, metadata, `at`) with filters and the sensitive-read trail (who/when/what on patients and prescriptions) and verification log; `audit_log` is append-only with a 5-year retention policy (no purge job before the window); `es-EC` labels; commands: `cd persona-sanus && dx build --release --fullstack`
- [ ] 5. Extend `.blocks/persona-probe.py` with `--flow security` — requirements: asserts the token is absent from the sign-in body (and the earlier flows now assert a cookie-backed session instead), sign-in errors are uniform for unknown vs known emails, a foreign `Origin` is rejected on a state change while a missing `Origin` succeeds, security headers are present, and the rate limit engages (`AUTH_RATE_LIMIT_PER_MINUTE`, tuned so other flows stay below it and the security flow trips it last); block header and indexes updated; commands: `python3 .blocks/persona-probe.py --base http://127.0.0.1:8080 --flow security`

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No billing or entitlement code (v1.1 only).
- No auto-update, installers, backups, or jobs (spec 13-M).
- Do not weaken `/api/auth/*` behavior relied on by existing probe flows (adjust flows only when the contract intentionally changes, and record a divergence entry).
- No unit tests; no comments in any code.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [ ] `python3 .blocks/persona-smoke.py --dir persona-sanus --port 8091` passes (fmt, check, clippy, release build, auth probe)
- [ ] `python3 .blocks/persona-docker-verify.py --dir . --port 8080` passes (compose build, boot, health, auth persistence)
- [ ] objective 5: `python3 .blocks/persona-probe.py --base http://127.0.0.1:8080 --flow security` → all checks true (token hidden, uniform errors, origin check, headers, rate limit)
- [ ] objective 2: a Doctor account signs in with the second factor; a Nursing account does not need it; changing a password invalidates the old session
- [ ] objective 4: `http://127.0.0.1:8080/audit` in a browser as Administrator → mutation, sensitive-read, and verification rows visible with filters; as Doctor → refused
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
file to `specs/done/` (e.g. `specs/done/12-L-hardening.md`).
Specs with open divergences stay in `specs/` next to `TEMPLATE.md` until the
tracking file is cleared.*
