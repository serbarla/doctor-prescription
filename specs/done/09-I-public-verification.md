# Spec 09 — Phase I: Public verification

- Source: `context/PRD.md` §5.6 (FR-050), FR-033/034; `context/sitemap.md` §1.1 (`/verify/:code`), §5 (deep links); `context/database-design.md` §6 (verification code rules).
- Depends on: 07-G, 08-H.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Result carried forward: anyone scanning a prescription QR or re-typing its code reaches `/verify/:code` and sees authenticity status plus doctor, date, and number — and nothing else.

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

- [x] 1. Implement the public verification read — requirements: `GET /api/verify/:code` without authentication looks up `verification_codes.code_hash` and returns status (`valid` / `voided` / `superseded` / `not found`) plus doctor name, issue date, and prescription number only; no patient name, cédula, diagnosis, or medications ever leave the endpoint (FR-050); the same lookup serves the re-typed printed code; commands: `cargo check --manifest-path persona-sanus/Cargo.toml`
- [x] 2. Build the verification page — requirements: route `/verify/:code` outside the authenticated shell, matching the QR target `{PUBLIC_BASE_URL}/verify/{code}` (architecture §14, same base as spec 08-H) and the typed code form; `es-EC` copy ("Receta auténtica" / status variants), minimal design-token styling; "not found" renders the same shape without leaking whether a code ever existed; commands: `cd persona-sanus && dx build --release --fullstack`
- [x] 3. Rate-limit and log verification — requirements: per-IP rate limit at the HTTP layer on `/api/verify/:code` and `/verify/:code` returning `429` past the limit (`VERIFY_RATE_LIMIT_PER_MINUTE`, architecture §14; in-memory keyed by IP, resets on restart); rate-limited (429) requests are not audited as verification attempts (bounded public write amplification); every other verification attempt writes an `audit_log` row with `actor_id` NULL (the spec 02-B audit helper takes `Option<actor>`) and without storing raw IPs beyond what the law allows; commands: `cargo check --manifest-path persona-sanus/Cargo.toml`
- [x] 4. Extend `.blocks/persona-probe.py` with `--flow verify` — requirements: composing the `rx` flow fixtures (signed, voided, and superseded prescriptions) by running them internally or accepting their ids as params (a single invocation must be self-contained), it asserts each status response, the `not found` response, the absence of PHI keys in the JSON, and the `429` under repetition — run last or with its own key/limit so saturated limits never poison later flows; block header and indexes updated; commands: `python3 .blocks/persona-probe.py --base http://127.0.0.1:8080 --flow verify`

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No PHI in any response, URL, page, log, or audit metadata.
- No auth-endpoint rate limiting or security headers (spec 12-L); this spec limits only the verification surface.
- No changes to prescription or template code (specs 05, 07) beyond what the lookup reads.
- No billing or entitlement code (v1.1 only).
- No unit tests; no comments in any code.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [x] `python3 .blocks/persona-smoke.py --dir persona-sanus --port 8091` passes (fmt, check, clippy, release build, auth probe)
- [x] objective 4: `python3 .blocks/persona-probe.py --base http://127.0.0.1:8080 --flow verify` → all checks true (statuses, not-found, no PHI keys, rate limit)
- [x] objective 2: `http://127.0.0.1:8080/verify/:code` in a browser signed out → "Receta auténtica — … Dr. …, date …, No. …" with no patient data; an unknown code shows the same layout
- [x] objective 3: hammering the endpoint returns `429` and `audit_log` shows verification rows without raw IPs
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
file to `specs/done/` (e.g. `specs/done/09-I-public-verification.md`).
Specs with open divergences stay in `specs/` next to `TEMPLATE.md` until the
tracking file is cleared.*
