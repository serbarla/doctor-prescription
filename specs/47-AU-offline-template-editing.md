# Spec 47-AU — Offline template draft editing

- Source: `context/PRD.md` FR-021, FR-040–043, D-06, D-07, D-21; `context/screens/TemplateEdit.md` §§2.1–2.4; `context/screens/TemplatePreview.md`; `persona-sanus/src/main.rs` (`TemplateEdit`, `TemplatePreview`, `publish_template_dispatch`), `src/forms.rs` (`TemplateDocumentForm`), `src/desktop_local.rs`.
- Depends on: 46-AT.
- Project root: `/Users/sergiobarba/projects/doctor-prescriptions`. All paths and commands below are relative to it.
- Owned files: `specs/47-AU-offline-template-editing.md`, `persona-sanus/src/main.rs`, `persona-sanus/src/forms.rs`, `persona-sanus/src/desktop_local.rs`
- Gate: `cargo check --manifest-path persona-sanus/Cargo.toml --no-default-features --features desktop && cargo check --manifest-path persona-sanus/Cargo.toml --no-default-features --features web,server`
- Desktop-verify: yes
- User-interface verification: `required` — desktop editor and preview, disconnect/restart/reconnect workflow in §3; record screenshots of draft and preview and non-PHI steps/outcomes.
- Result carried forward: An authorized desktop user can save and reopen a local template draft and preview it offline without mistaking it for a server-published version.

## 1. Objectives and scope

- [ ] 1. Persist editable desktop drafts — requirements: use the existing structured document validation and plain-text controls; save document JSON, theme/margin, template and version identity, author/clinic, and a base server version/revision in durable local storage atomically. Support both a previously cached template and the locally created template with no version. Save repeatedly without duplicating or modifying published versions. A failed local write leaves the form and prior durable draft intact and shows an error. Restrict offline read/write to the restored authenticated actor's clinic and the owner doctor or administrator for a clinic master; no cached copy is authority for another user. Local drafts survive navigation, restart, cache refresh and eviction.
- [ ] 2. Make preview use the saved local draft — requirements: reopen the draft in the editor and show that same saved document and margin in the A4 sample preview, before published versions; retain explicit version selection. Use available cached referenced image bytes through the existing local asset cache; show a visible placeholder when missing, never claim remote assets were fetched. Missing template, malformed document or cache failure has a clear error, not a blank/default document that could overwrite saved data. Mark the view local/unsynced, not published or final.
- [ ] 3. Safe reconciliation — requirements: recommended bounded choice: keep draft saves local without queuing a new server version mutation; on reconnect retain the local draft and offer explicit server save through the existing authorized version API. Before replacing or updating any server draft, compare its identity and baseline against the captured base; on changed, removed, archived, unauthorized or unavailable server state preserve local content and show a conflict/actionable error, never overwrite silently. A successful authorized save is the only point at which its local unsynced marker may clear. Disable offline publish of a local-only or changed draft; never enqueue publish for a version the server has not saved. Existing queued publish operations and published immutable versions must not be discarded. Migration must preserve existing local caches and outbox rows, including pending/failed entries; no reset.

### Out of scope / no-go

- No new automated tests, image upload/offline upload, rich text, signature/stamp imagery, billing, changes to server authorization or published-version immutability. Web stays online-only. Do not put clinical or other sensitive text in URLs/logs. OS disk encryption remains the deployment prerequisite (D-07).
- Do not silently publish a stale draft or conflate a local save with audit/server acceptance. If a safe reconciliation requires changes outside owned files, record a divergence and request scope approval.

## 2. Problem and decision

- Problem: The editor can read cached templates but `forms.rs` saves versions only through the API, and `TemplatePreview` loads only from the API. An offline edit cannot reliably survive and be previewed.
- Decision: Local, actor-scoped durable drafts with explicit conflict-aware server save on reconnect. This reconciliation policy is a recommended implementation choice, not an established product decision; FR-040–043 require honest offline state and preservation, not a claim of successful publication.
- Open questions: None for this bounded choice.

## 3. Acceptance and verification

Use a disposable local clinic/account and synthetic, non-PHI content. Capture steps and results; reconnect only to that test environment.

- [ ] Desktop workflow: prime a doctor-owned template and cached image online, disconnect, change distinct header/body, margin and medication, save, leave and restart; reopen editor and preview → all saved values and cached image appear, local/unsynced state is clear; missing cached image gives a placeholder. Repeat for a newly created offline template with no server version.
- [ ] Desktop workflow: edit after an earlier save, force a local write failure, reopen → error is visible, unsaved form remains until exit and the last successful durable draft remains unchanged. A different actor/clinic, nursing or secretary cannot read/edit the draft through the route.
- [ ] Desktop workflow: with local changes, reconnect after another authorized client changes/archives the server draft; attempt server save → conflict/error, both copies remain intact and publish is unavailable until resolved. With an unchanged base, explicit server save succeeds, then publish uses the actual server-saved version. Repeat with network failure and an existing pending/failed outbox row → no lost draft or queue row.
- [ ] Gate: `cargo check --manifest-path persona-sanus/Cargo.toml --no-default-features --features desktop && cargo check --manifest-path persona-sanus/Cargo.toml --no-default-features --features web,server` exits 0.
- [ ] User-interface verification: perform the desktop workflow above and retain screenshots with no real PHI.
- [ ] Every objective in section 1 is checked.

## 4. Execution and divergences

Read project instructions first. If a dependency is incomplete or required work is outside owned files, record evidence, impact and affected criteria and ask the project owner for scope approval; do not improvise. Unresolved divergences block completion.

### Divergences

None.
