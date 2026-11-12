# Spec 05 — Phase E: Templates and forms

- Source: `context/PRD.md` §5.3 (FR-020–024), §7 (editor row, D-21); `context/sitemap.md` §1.2 (`/templates*`); `context/database-design.md` §5 (template tables), §6 (insertion order, immutability).
- Depends on: 01-A, 02-B.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Result carried forward: doctors and administrators edit form-built structured-JSON templates with the FR-023 variable set and publish immutable versions that later prescriptions can pin.

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

- [ ] 1. Implement template service and REST — requirements: `create_template`, `create_template_version` (draft `doc_json` edit), `publish_template_version` (immutable: `version_no` increments, status `published`, `templates.current_version_id` set following the `context/database-design.md` §6 insertion order), `archive_template`; ownership rules (doctor: own templates; administrator: clinic masters via `owner_type`); each mutation writes an audit row; endpoints under `/api/templates`; commands: `cargo check --manifest-path persona-sanus/Cargo.toml`
- [ ] 2. Build the shared template form component — requirements: one shared Dioxus component `src/forms.rs` (plain forms on every target, no gate — PRD D-21) with fields for letterhead (logo `asset_id`, doctor/clinic details, colors), body text, footer, image slots, and medication blocks; it persists the document as schema-checked structured JSON (`doc_json`, carrying a fixed `schema_version` field); no rich text, no WYSIWYG, no editor dependency, and raw HTML is never produced, accepted, or persisted; the same component compiles on web and desktop; commands: `cd persona-sanus && dx build --release --fullstack`
- [ ] 3. Support the v1 variable set — requirements: `variables_spec` stores exactly the FR-023 variables (`patient.{name,national_id,age}`, `doctor.{name,specialty,national_id,contact}`, `clinic.{name,address,phone}`, `prescription.{number,date,diagnosis}`, `medications[]`); the form's variable picker inserts them; unknown variables render as warnings in preview, never as literal `{{key}}`; commands: `cd persona-sanus && dx build --release --fullstack`
- [ ] 4. Build the templates UI — requirements: routes `/templates` (per-doctor + clinic master, status and current version), `/templates/new`, `/templates/:id/edit`, `/templates/:id/preview` (true-scale 210×297 mm browser projection of the same `doc_json` with sample values, configurable margins, repeated header/footer); the projection is one shared Dioxus component that spec 08-H reuses for `/prescriptions/:id/preview` — one layout projection, no second engine; `es-EC` labels; commands: `cd persona-sanus && dx build --release --fullstack`
- [ ] 5. Extend `.blocks/persona-probe.py` with `--flow templates` — requirements: create a template, save a draft version, publish it, and prove the published version is immutable (update refused) and `current_version_id` points at it; block header and indexes updated; commands: `python3 .blocks/persona-probe.py --base http://127.0.0.1:8080 --flow templates`

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No image upload, storage, or asset bytes (spec 06-F supplies the library; this spec only emits `asset_id`-bearing placeholders).
- No rich text or WYSIWYG editing anywhere (PRD D-21); no editor dependency of any kind.
- No PDF rendering, QR, or `printpdf` dependency (spec 08-H); no `from_html` anywhere.
- No prescriptions or numbering code (spec 07-G).
- No billing or entitlement code (v1.1 only).
- No unit tests; no comments in any code.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [ ] `python3 .blocks/persona-smoke.py --dir persona-sanus --port 8091` passes (fmt, check, clippy, release build, auth probe)
- [ ] objective 5: `python3 .blocks/persona-probe.py --base http://127.0.0.1:8080 --flow templates` → all checks true (create, draft version, publish, immutability)
- [ ] objective 2: `http://127.0.0.1:8080/templates/new` in a browser → form values and variable insertion persist after reload as structured JSON (`doc_json` via the API contains form data, never HTML)
- [ ] objective 4: `http://127.0.0.1:8080/templates/:id/preview` → true-scale A4 projection with sample values; unknown variables show warnings
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
file to `specs/done/` (e.g. `specs/done/05-E-templates-and-forms.md`).
Specs with open divergences stay in `specs/` next to `TEMPLATE.md` until the
tracking file is cleared.*
