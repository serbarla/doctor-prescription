# Spec 03 — Phase C: Patient CRUD

- Source: `context/PRD.md` §5.1 (FR-001–003), §3 roles, §9 (ARCO); `context/sitemap.md` §1.2 (`/patients*`); `context/database-design.md` §5 (`patients`), §6 (soft delete).
- Depends on: 02-B.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Result carried forward: authorized roles create, search, edit, and soft-delete clinic-scoped patients with cédula deduplication and an audited sensitive-read trail.

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

- [x] 1. Implement patient service functions and REST — requirements: `create_patient`, `update_patient`, `search_patients` (by name and cédula), `get_patient`, `soft_delete_patient` behind `/api/patients` and `/api/patients/:id`; clinic scoping on every query and role filters (Doctor, Nursing, Secretary write; Administrator read-only); every patient read writes a `sensitive_read` audit row (who/when/what); fields: names, cédula, birth date, sex, phone, notes; commands: `cargo check --manifest-path persona-sanus/Cargo.toml`
- [x] 2. Enforce cédula deduplication — requirements: an exact `(clinic_id, national_id)` duplicate is refused by the unique index and surfaced as `409` with `code: "duplicate_national_id"` (FR-002); a likely duplicate (same normalized `full_name` — Unicode casefold, accent-stripped, whitespace-collapsed) returns `200` with `warning: "likely_duplicate"` plus the matching `patient_id`s and creates nothing until the client resends with `confirm_duplicate: true`; commands: `cargo check --manifest-path persona-sanus/Cargo.toml`
- [x] 3. Add ARCO soft delete — requirements: `soft_delete_patient` sets `deleted_at` through an audited action; soft-deleted patients are excluded from search and dedupe; no hard delete path exists; commands: `cargo check --manifest-path persona-sanus/Cargo.toml`
- [x] 4. Build the patients UI — requirements: routes `/patients` (search by name/cédula), `/patients/new`, `/patients/:id` (demographics + actions), `/patients/:id/edit`; `es-EC` labels, `DD-MM-AAAA` dates (the client gains `chrono` on the `web` and `desktop` features — it is `server`-only today); the *New prescription* deep link stays inert until spec 07; commands: `cd persona-sanus && dx build --release --fullstack`
- [x] 5. Extend `.blocks/persona-probe.py` with `--flow patients` — requirements: create, search, update, soft-delete inside one clinic; a second clinic's patient is never readable; Administrator gets read-only (update refused server-side); duplicate cédula returns the `409 duplicate_national_id` conflict, and same-name creation returns the warning first and succeeds only after `confirm_duplicate: true`; block header and indexes updated; commands: `python3 .blocks/persona-probe.py --base http://127.0.0.1:8080 --flow patients`

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No prescriptions, templates, or assets code or routes (specs 05, 06, 07).
- No hard delete, no patient export (export belongs to spec 13-M).
- No billing or entitlement code (v1.1 only).
- No unit tests; no comments in any code.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [x] `python3 .blocks/persona-smoke.py --dir persona-sanus --port 8091` passes (fmt, check, clippy, release build, auth probe)
- [x] objective 5: `python3 .blocks/persona-probe.py --base http://127.0.0.1:8080 --flow patients` → all checks true (CRUD, scoping, role filters, dedupe)
- [x] objective 4: `http://127.0.0.1:8080/patients` in a browser → search by name and cédula returns the created patient; edit reloads saved values; deleted patients disappear from search
- [x] objective 1: sensitive reads appear in `audit_log` (visible later at `/audit`; verified for now via `python3 .blocks/sqlite-schema-check.py --db <booted-db> --query "select action, entity from audit_log order by at desc limit 5"` → `sensitive_read`, `patient`)
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
file to `specs/done/` (e.g. `specs/done/03-C-patient-crud.md`).
Specs with open divergences stay in `specs/` next to `TEMPLATE.md` until the
tracking file is cleared.*
