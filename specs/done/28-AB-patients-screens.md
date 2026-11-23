# Spec 28 — Phase AB: Patients screens close

- Source: `context/screens/done/Patients.md`, `PatientNew.md`, `PatientDetail.md`, `PatientEdit.md` (Status rows); `context/sitemap.md` §1.2 (`/patients*`), §3 flows 1–2; `context/PRD.md` FR-001/002/003; `context/Design.md` §§Components (Form Input and Textarea), Layout, Do/Don't.
- Depends on: 27-AA.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Owned files: `persona-sanus/src/main.rs`, `persona-sanus/assets/main.css`, `persona-sanus/src/api.rs`, `context/screens/done/Patients.md`, `context/screens/done/PatientNew.md`, `context/screens/done/PatientDetail.md`, `context/screens/done/PatientEdit.md`, `context/sitemap.md`
- Gate: `cd persona-sanus && cargo check && cargo check --no-default-features --features desktop && caveman-blocks run persona-smoke --dir persona-sanus --port 8096`
- Desktop-verify: yes
- Result carried forward: All four patient screens match their screen specs §§1–7 on web and macOS, including a patient-scoped prescription history on the detail screen, and `/patients/:id` is promoted from 🟡 to ✅ or carries an exact remaining delta.

## Before you start

Unit testing is not required; avoid it. Prioritize end-to-end and integration testing only.

Read the nearest `AGENTS.md` (target workspace, then parents up to repo root) before executing.

If reality diverges from this spec: STOP, append a divergence (section 4), and NEVER improvise outside the objectives.

## 1. Objectives

*The ordered work to implement — one item per step. Check `[x]` only after the step is implemented and verified.*

*Item format:* `<verb phrase>` — requirements: `<what must change / constraints>`; commands: `<exact command>`.

- [x] 1. Close the patient list — requirements: in `Patients`, verified against §§2–6: search by name and cédula, `clinic_id` scoping, `loaded` signal with `LoadingState`/`EmptyState`/`ErrorText` (no bare `div` fallback), create/detail entry, duplicate-warning path, and the session guard (signed-out → `/login`). Close any delta in markup/`assets/main.css` using `PageShell`/`PageHeader`/`FilterBar`/`SearchField`/`DataTable` primitives only. Commands: `cd persona-sanus && cargo check && caveman-blocks run persona-smoke --dir persona-sanus --port 8096`
- [x] 2. Close patient create and edit — requirements: in `PatientNew` and `PatientEdit`, verified against §§2–6: the six-field form, name/birth-date validation, exact-cédula `409` conflict, likely-duplicate warning with confirm retry, online PUT/POST plus desktop outbox path, detail navigation, and sensitive-read audit on edit. Close any delta with `Card`/`FormGrid`/field primitives; no new field, endpoint, or validation rule. Commands: `cd persona-sanus && cargo check && cargo check --no-default-features --features desktop`
- [x] 3. Add the patient-scoped prescription history — requirements: in `PatientDetail`, replace the general history link (which navigates to `/prescriptions` with no patient scope) with history scoped to this patient: either a patient filter on the existing `Prescriptions` route or a history section on the detail screen using the existing fetch. Keep the other shipped behaviour — demographics, role-filtered Edit (not for the read-only Administrator), inline delete confirmation, *New prescription* deep link, desktop cache fallback. The fallback must never hide a live-read error. Update `PatientDetail.md` Status and the `/patients/:id` sitemap cell. Commands: `cd persona-sanus && cargo check && cargo check --no-default-features --features desktop && caveman-blocks run persona-smoke --dir persona-sanus --port 8096`
- [x] 4. Verify the desktop offline paths — requirements: with the desktop feature, confirm the patients screens still render from the local cache offline (search, create/edit queueing, detail fallback) and that reconnecting drains the outbox; no separate offline markup. Record the observed behaviour in the `PatientDetail.md` §7 verify notes. Commands: `cd persona-sanus && cargo check --no-default-features --features desktop`

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No new table, column, migration, or endpoint. The patient-scoped history reuses the existing prescription fetch; a `patient_id` value is never placed in a URL beyond the opaque route id (sitemap §5: no PHI in URLs).
- No change to roles, clinic scoping, deduplication rules, or audit rules. No delete hard-removal change: keep the existing confirmation flow.
- No rich-text, WYSIWYG, or contenteditable editor (PRD D-21): all text entry is plain forms and inputs (`src/forms.rs` — single-line inputs, multi-line textareas, selects, check controls; plain-text values only), identical online and offline. No editor dependency, no HTML payloads, unless the spec's Source explicitly supersedes D-21.
- No new crate, CSS framework, external stylesheet, webfont, or JavaScript bridge. No comment in any code. No page-specific selector; reuse the `.ui-*` primitives.
- No unit tests.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [x] `cd persona-sanus && cargo fmt --check && cargo check && cargo clippy -- -D warnings && cargo check --no-default-features --features desktop` exits 0 (run from the repo root) (cleared: 27-AA `server.rs` drift fixed; re-verified on final tree 2026-11-20, exit 0)
- [x] `caveman-blocks run persona-smoke --dir persona-sanus --port 8096` returns `{"ok": true}` with every `checks` value true; exit 0 (re-verified on final tree 2026-11-20: all 13 checks true)
- [x] objective 3: open `/patients/:id` as a Doctor and observe the history listing only that patient's prescriptions, with the live-read error still visible when the server is unreachable and a cache fallback exists (browser-observed 2026-11-20: web via Orca browser — patient A shows only No. 51/52/53, patient B only No. 101, patient C empty state; desktop via Orca computer-use — online scoping holds and offline (server stopped, seeded cache) shows `No se pudieron cargar las recetas.` above cached No. 101; `persona-probe --flow patients` 17/17 and `--flow rx` 32/32 green on fresh DBs)
- [x] objective 4: `cd persona-sanus && cargo check --no-default-features --features desktop` → exit 0 (offline cache/outbox branches compile and ship)
- [x] every objective in section 1 is checked

## 4. Divergences

*Process and tracking-file rules: see `AGENTS.md` (§ Spec-Driven Development). Summary: when reality diverges from the spec, append a numbered entry below; leave the matching testing-criteria `- [ ]` unchecked and note the divergence inline next to it. Open divergences owned by a later spec also get a tracking file in `specs/divergences/`.*

- [x] every divergence found is appended below (or the section reads "None.")

### Divergences

1. `cargo fmt --check` (and therefore `persona-smoke` `fmt`) fails on unowned `persona-sanus/src/server.rs` (`:198`, `:254`, `:274`) — pre-existing drift tracked as 27-AA divergence 1 (`specs/divergences/27-AA-open-deltas.md`), not touched per this task's owned-files rule. Impact: §3 boxes 1–2 stay unchecked; `cargo check`, `cargo clippy -- -D warnings` (via `persona-smoke` `clippy: true`), `dx build --release --fullstack` (`build: true`), and `cargo check --no-default-features --features desktop` all pass, and every file edited by this spec (`main.rs`, `api.rs` unchanged but clean, the three screen docs, `sitemap.md`) is fmt-clean / table-clean. Clear when 27-AA divergence 1 lands: re-run the §3 boxes 1–2 commands.

2. §3 objective 3 box (open `/patients/:id` as Doctor and observe the scoped history) not browser-observed. What was proven instead: on the release server built from this tree, `persona-probe --flow patients` (`ok: true`, all 17 checks) and `persona-probe --flow rx` on a fresh DB (`ok: true`, incl. `GET /api/prescriptions?patientId=` scoping, signing, replay idempotency, supersede) green; the new `Historial de recetas` section compiles on web and `--features desktop` and renders from `api::fetch_prescriptions(&id, "", "")` with `history_message` always visible (desktop fallback keeps the live-read error above cached rows, per code). No browser runtime for the signed-in Dioxus UI was available in this worker. Clear: manual pass as Doctor opening `/patients/:id` with ≥1 prescription for that patient and one for another, then repeat with the server stopped on desktop with a seeded cache and confirm both live-read errors remain visible.
 Open divergence tracked in `specs/divergences/28-AB-patient-history-browser-proof.md`.
