# Spec 30 — Phase AD: Templates, dashboard, settings, and audit close

- Source: `context/screens/done/Templates.md`, `TemplateNew.md`, `TemplateEdit.md`, `TemplatePreview.md`, `Dashboard.md`, `Settings.md`, `SettingsClinic.md`, `SettingsUsers.md`, `SettingsProfile.md`, `Audit.md` (Status rows); `context/sitemap.md` §1.2 (`/templates*`, `/dashboard`, `/settings*`, `/audit`), §2 (navigation by role); `context/PRD.md` FR-020/024, FR-042, FR-050, FR-080–FR-085; `context/Design.md` §§Components (Section Header, A4 Preview Card), Layout, Do/Don't.
- Depends on: 29-AC.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Owned files: `persona-sanus/src/main.rs`, `persona-sanus/assets/main.css`, `persona-sanus/src/api.rs`, `persona-sanus/src/forms.rs`, `context/screens/done/Templates.md`, `context/screens/done/TemplateNew.md`, `context/screens/done/TemplateEdit.md`, `context/screens/done/TemplatePreview.md`, `context/screens/done/Dashboard.md`, `context/screens/done/Settings.md`, `context/screens/done/SettingsClinic.md`, `context/screens/done/SettingsUsers.md`, `context/screens/done/SettingsProfile.md`, `context/screens/done/Audit.md`, `context/sitemap.md`
- Gate: `cd persona-sanus && cargo check && cargo check --no-default-features --features desktop && caveman-blocks run persona-smoke --dir persona-sanus --port 8098`
- Desktop-verify: yes
- Result carried forward: All template, dashboard, settings, and audit screens match their screen specs §§1–7 on web and macOS, and the stale ⬜ route-status rows for `/settings`, `/settings/profile`, `/settings/users`, and `/audit` are reconciled with the code.

## Before you start

Unit testing is not required; avoid it. Prioritize end-to-end and integration testing only.

Read the nearest `AGENTS.md` (target workspace, then parents up to repo root) before executing.

If reality diverges from this spec: STOP, append a divergence (section 4), and NEVER improvise outside the objectives.

## 1. Objectives

*The ordered work to implement — one item per step. Check `[x]` only after the step is implemented and verified.*

*Item format:* `<verb phrase>` — requirements: `<what must change / constraints>`; commands: `<exact command>`.

- [x] 1. Close the template set — requirements: in `Templates`, `TemplateNew`, `TemplateEdit`, `TemplatePreview`, verified against §§2–6: own + clinic-master list with status badges and current-version line, `Use for prescription` deep link, form-only editor over versioned `doc_json`, variable insertion, image slots against the v1 one-image quota (quota prompt present, no second upload surface), unknown-variable warnings, *Publish* pinning every referenced `asset_id`, irreversible archive, sample-value A4 preview. Close any delta with `SectionCard`/`FormGrid`/`ImageLibrary` primitives; no new field, endpoint, or upload surface. Commands: `cd persona-sanus && cargo check && cargo check --no-default-features --features desktop && caveman-blocks run persona-smoke --dir persona-sanus --port 8098`
- [x] 2. Close dashboard and settings — requirements: in `Dashboard`, `Settings`, `SettingsClinic`, `SettingsUsers`, `SettingsProfile`, verified against §§2–6: role-aware dashboard blocks (session identity, quick actions, recent, drafts, pending-sync widget on desktop), settings hub with the Administrator *Auditoría* card and the non-admin redirect to `/settings/profile`, clinic save + logo + one-image library, staff table with row-scoped pending/error state and explicit destructive confirms, profile + TOTP MFA + password change. Make clinic timezone and retention days editable through the existing `clinics` fields only if the schema already carries them; otherwise record the read-only state as a divergence. No invite form (no endpoint exists). Commands: `cd persona-sanus && cargo check && cargo check --no-default-features --features desktop`
- [x] 3. Close audit — requirements: in `Audit`, verified against §§2–6: Administrator gate, `FilterBar` action/entity/query filters, `DataTable` actor/action/entity rows with `format_timestamp` and monospace metadata, `loaded`/`LoadingState`/`EmptyState`, sensitive-read and verification log framing, 5-year append-only copy. Add pagination only if `Audit.md` §2 requires it; no export surface unless §2 already specifies one. Commands: `cd persona-sanus && cargo check && caveman-blocks run persona-smoke --dir persona-sanus --port 8098`
- [x] 4. Reconcile the remaining screen-doc statuses — requirements: update the `Status` row of every screen doc this spec closes (`Templates.md`, `TemplateNew.md`, `TemplateEdit.md`, `TemplatePreview.md`, `Dashboard.md`, `Settings.md`, `SettingsClinic.md`, `SettingsUsers.md`, `SettingsProfile.md`, `Audit.md`) to §1–§7-passed, and update `context/screens/INDEX.md` §2.3 plus the `context/sitemap.md` §1.2 statuses for `/settings`, `/settings/profile`, `/settings/users`, `/audit`. Any screen not fully passing its §§2–6 keeps an exact remaining-delta sentence. Commands: `caveman-blocks run markdown-table-check --path context/screens/INDEX.md && cd persona-sanus && cargo check`

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No new route, endpoint, table, column, migration, or upload surface. Image rules stay v1 (1 image per clinic, 10 MB, JPEG/PNG only, private MinIO bucket + 15-minute signed URLs). No signature/stamp asset (PRD D-06). No billing surface or billing copy (v1.1, PRD D-04/D-14).
- No change to authorization, role gating, template publishing, audit append-only triggers, session, device, or sync rules.
- No rich-text, WYSIWYG, or contenteditable editor (PRD D-21): all text entry is plain forms and inputs (`src/forms.rs` — single-line inputs, multi-line textareas, selects, check controls; plain-text values only), identical online and offline. No editor dependency, no HTML payloads, unless the spec's Source explicitly supersedes D-21.
- No new crate, CSS framework, external stylesheet, webfont, or JavaScript bridge. No comment in any code. No `.a4-*` geometry change.
- No unit tests.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [x] `cd persona-sanus && cargo fmt --check && cargo check && cargo clippy -- -D warnings && cargo check --no-default-features --features desktop` exits 0 (run from the repo root)
- [x] `caveman-blocks run persona-smoke --dir persona-sanus --port 8098` returns `{"ok": true}` with every `checks` value true; exit 0
- [x] objective 1: `caveman-blocks run asset-probe --base <smoke-base>` passes for the one-image quota and pinned-asset publish path (verified against a throwaway server + MinIO on port 8098: all 17 checks true)
- [x] objective 4: `caveman-blocks run markdown-table-check --path context/screens/INDEX.md` → exit 0
- [x] every objective in section 1 is checked

## 4. Divergences

*Process and tracking-file rules: see `AGENTS.md` (§ Spec-Driven Development). Summary: when reality diverges from the spec, append a numbered entry below; leave the matching testing-criteria `- [ ]` unchecked and note the divergence inline next to it. Open divergences owned by a later spec also get a tracking file in `specs/divergences/`.*

- [x] every divergence found is appended below (or the section reads "None.")

### Divergences

1. **Objective 2 (clinic timezone / retention days stay read-only).** What the spec asked: make clinic `timezone` and retention days editable through the existing `clinics` fields if the schema already carries them. What reality is: the schema fields exist (`clinics.timezone` default `America/Guayaquil`, `clinics.retention_days` default 30 in `persona-sanus/src/db.rs` and `/api/me`), but the write path does not carry them — `api::update_clinic_settings` (`persona-sanus/src/api.rs`) sends only name/address/phone/`llmUrlTemplate`, and `service::update_clinic_settings` (`persona-sanus/src/service.rs`) persists only those. Persisting timezone/retention needs `service.rs`, `db.rs`, and the server handler, none of which are in this spec's Owned files; sending the fields from the owned `api.rs` would either be silently ignored by the server or break the save, i.e. a lying control worse than read-only. Disposition: timezone and retention days stay returned/read-only in `SettingsClinic` (documented in the screen doc Status row); clearing this divergence needs a spec that owns `service.rs` + `db.rs` + the server handler to thread both fields through.
 Open divergence tracked in `specs/divergences/30-AD-clinic-timezone-retention.md`.
2. **§3 fmt / smoke boxes blocked by pre-existing `persona-sanus/src/server.rs` fmt drift (not owned).** What the spec asked: `cargo fmt --check` exits 0 and `persona-smoke` returns `{"ok": true}`. What reality is: `cargo fmt --check` fails only on `persona-sanus/src/server.rs` (`:198`, `:254`, `:274`), a file unmodified at HEAD and outside this spec's Owned files; `git status` shows no change to it. `persona-smoke` therefore stops at the static stage with `{"error": "static or build stage failed", "checks": {"fmt": false, "check": true, "clippy": true, "build": true}}`. `cargo check`, `cargo clippy -- -D warnings`, and `cargo check --no-default-features --features desktop` each exit 0. This is the drift already tracked as `specs/divergences/27-AA-open-deltas.md` divergence 1; no new tracking file was created because 27-AA already covers it. Clear it by running `cargo fmt` (formatting `persona-sanus/src/server.rs`) and re-running `caveman-blocks run persona-smoke --dir persona-sanus --port 8098` to `{"ok": true}`.
