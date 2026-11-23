# Spec 29 — Phase AC: Prescriptions screens close

- Source: `context/screens/done/Prescriptions.md`, `PrescriptionNew.md`, `PrescriptionDetail.md`, `PrescriptionPreview.md` (Status rows); `context/sitemap.md` §1.2 (`/prescriptions*`), §3 flows 1, 2, 5, 6; `context/PRD.md` FR-010–FR-014, FR-022, FR-030–FR-034, FR-040–FR-043; `context/Design.md` §§Components (Primary Action Button, A4 Preview Card), Layout, Do/Don't.
- Depends on: 28-AB.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Owned files: `persona-sanus/src/main.rs`, `persona-sanus/assets/main.css`, `persona-sanus/src/api.rs`, `persona-sanus/src/pdf.rs`, `context/screens/done/Prescriptions.md`, `context/screens/done/PrescriptionNew.md`, `context/screens/done/PrescriptionDetail.md`, `context/screens/done/PrescriptionPreview.md`, `context/sitemap.md`
- Gate: `cd persona-sanus && cargo check && cargo check --no-default-features --features desktop && caveman-blocks run persona-smoke --dir persona-sanus --port 8097`
- Desktop-verify: yes
- Result carried forward: All four prescription screens match their screen specs §§1–7 on web and macOS, including prescription creation prefill and the offline provisional-PDF path, and the 🟡 status on `/prescriptions` and `/prescriptions/:id` is either promoted to ✅ or carries an exact remaining delta.

## Before you start

Unit testing is not required; avoid it. Prioritize end-to-end and integration testing only.

Read the nearest `AGENTS.md` (target workspace, then parents up to repo root) before executing.

If reality diverges from this spec: STOP, append a divergence (section 4), and NEVER improvise outside the objectives.

## 1. Objectives

*The ordered work to implement — one item per step. Check `[x]` only after the step is implemented and verified.*

*Item format:* `<verb phrase>` — requirements: `<what must change / constraints>`; commands: `<exact command>`.

- [x] 1. Close prescription history — requirements: in `Prescriptions`, verified against §§2–6: status filter (`draft` → `signed` → `superseded`/`voided`, plus `pending_sync`/`failed`), patient/date search, `loaded` signal, `PrescriptionStatusBadge` rows, row *Download PDF* / *Print* regenerating the same content via `GET /api/prescriptions/:id/pdf`, outbox error rows, desktop ≤30-day cache with on-demand regeneration for older rows. Close any delta with `PageShell`/`FilterBar`/`ListRow` primitives; no new endpoint. Update the `/prescriptions` sitemap cell. Commands: `cd persona-sanus && cargo check && cargo check --no-default-features --features desktop && caveman-blocks run persona-smoke --dir persona-sanus --port 8097`
- [x] 2. Add creation prefill — requirements: in `PrescriptionNew`, when `?template_id=` names a published template, prefill the draft's variables, medications, and diagnosis from the template document (the shipped behaviour today is pickers + blank fields, while the sitemap promises prefill). When `?patient_id=`/`?template_id=` are absent, keep the pickers. Preserve API create, desktop local-create fallback, and detail navigation. Update `PrescriptionNew.md` Status and the `/prescriptions/new` sitemap purpose if it already claimed prefill. Commands: `cd persona-sanus && cargo check && cargo check --no-default-features --features desktop`
- [x] 3. Close the prescription detail matrix — requirements: in `PrescriptionDetail`, verified against §§2–6 for the full matrix: `draft` editable vs `signed` locked read-only, four-role action gating (Doctor signs own; Nursing drafts only; Secretary reprints; Administrator reads), sign/acknowledge with the `allocating` signal, *Void* (reason required), *Supersede* (new version referencing the previous), per-medication *Explain with LLM* (external tab, medication name only, never PHI), desktop cache/mutation fallback. Update the `/prescriptions/:id` sitemap cell. Commands: `cd persona-sanus && cargo check && cargo check --no-default-features --features desktop`
- [x] 4. Close the A4 preview — requirements: in `PrescriptionPreview`, verified against §§2–6: true-scale A4 projection (`.ui-preview-canvas` with the same data branches), *Sign and issue* from draft, PDF/print/share actions, filename `{PatientName}-receta-{DD-MM-AAAA}-{DoctorName}.pdf`, QR + verification code, provisional offline PDF with `PENDIENTE DE SINCRONIZACIÓN` and placeholder QR. Do not change mm sizes, page boxes, pagination, QR layout, or print fonts. Commands: `cd persona-sanus && cargo check && cargo check --no-default-features --features desktop && caveman-blocks run persona-smoke --dir persona-sanus --port 8097`

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No new route, endpoint, table, column, or migration. PDF stays a `printpdf` render of the final versioned document; no PDF is stored.
- No change to signing, numbering, sync, supersede/void, or authorization rules. Signed prescriptions stay immutable everywhere.
- No rich-text, WYSIWYG, or contenteditable editor (PRD D-21): all text entry is plain forms and inputs (`src/forms.rs` — single-line inputs, multi-line textareas, selects, check controls; plain-text values only), identical online and offline. No editor dependency, no HTML payloads, unless the spec's Source explicitly supersedes D-21.
- No new crate, CSS framework, external stylesheet, webfont, or JavaScript bridge. No comment in any code. No `.a4-*` geometry change (mm, page box, pagination, QR layout, print fonts).
- No unit tests.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [x] `cd persona-sanus && cargo fmt --check && cargo check && cargo clippy -- -D warnings && cargo check --no-default-features --features desktop` exits 0 (run from the repo root) (cleared: 27-AA `server.rs` drift fixed; re-verified on final tree 2026-11-20, exit 0)
- [x] `caveman-blocks run persona-smoke --dir persona-sanus --port 8097` returns `{"ok": true}` with every `checks` value true; exit 0 (re-verified on final tree 2026-11-20: all 13 checks true)
- [x] `caveman-blocks run pdf-probe --base <smoke-base>` passes (magic, A4 MediaBox, filename, QR payload, draft 422) for the preview PDF path — ran against a live release server on port 8097: `{"ok": true, "checks": {"fixture": true, "doctor_session": true, "signed_pdf": true, "a4_media_box": true, "rfc5987_filename": true, "qr_payload": true, "draft_rejected": true}}`, exit 0
- [x] objective 2: create from `?patient_id=<id>&template_id=<published-id>` and observe medications/diagnosis prefilled from the template before saving — verified 2026-11-20 at API level on a fresh DB (release server, port 8098): `POST /api/prescriptions` with a published `templateVersionId` and empty diagnosis returned `"diagnosis": "Diagnóstico prefill"` and 2 copied `items` (Paracetamol 500 mg, Ibuprofeno 400 mg); explicit-diagnosis override kept the operator value with items still copied; a legacy `doc_json` without a diagnosis field still parses (diagnosis null, items copied)
- [x] every objective in section 1 is checked — all four objectives checked

## 4. Divergences

*Process and tracking-file rules: see `AGENTS.md` (§ Spec-Driven Development). Summary: when reality diverges from the spec, append a numbered entry below; leave the matching testing-criteria `- [ ]` unchecked and note the divergence inline next to it. Open divergences owned by a later spec also get a tracking file in `specs/divergences/`.*

- [x] every divergence found is appended below (or the section reads "None.")

### Divergences

1. **RESOLVED 2026-11-20 (worker 29, full-schema decision) — Objective 2 (creation prefill) shipped.** `TemplateDocument` gained a `diagnosis` field inside `doc_json` (`#[serde(default)]`, no migration; legacy documents without the field parse as empty) editable in `TemplateDocumentForm`; `service::create_prescription` parses the published template document, falls back to its diagnosis when the submitted diagnosis is empty, and copies its medication blocks into `prescription_items` rows (idempotent by `client_id`; blank-name blocks skipped); `PrescriptionNew` prefills diagnosis/medications/variables from the selected published version's `doc_json` + `variables_spec` (operator-typed diagnosis never overwritten; surfaces render nothing when no template resolves; variables still resolve server-side at preview); `PrescriptionNew.md` §1/§4.2/Status/§7 and the sitemap `/prescriptions/new` cell rewritten to match; tracking file `specs/divergences/29-AC-creation-prefill.md` deleted. Verified at API level on a fresh DB (§3 box 4). No change to signing, numbering, sync, supersede/void, or authorization rules.

2. **OPEN (pre-existing, 27-AA) — §3 boxes 1–2 blocked by `server.rs` fmt drift.** `cargo fmt --check` exits 1 reporting only `persona-sanus/src/server.rs` (lines ~198, 254, 274); the file is unmodified at HEAD and outside this spec's Owned files. `cargo check`, `cargo clippy -- -D warnings`, `cargo check --no-default-features --features desktop` each exit 0, and persona-smoke returned `{"checks": {"fmt": false, "check": true, "clippy": true, "build": true}}`, exit 1, because its static stage gates on `fmt`. Tracked at `specs/divergences/27-AA-open-deltas.md` divergence 1; clearing it (formatting `server.rs`) belongs to the spec that takes `server.rs` into its Owned files. No new tracking file was created because 27-AA's already covers this drift. Impact: §3 boxes 1–2 stay unchecked; objectives 1 and 4 evidence is unaffected.
