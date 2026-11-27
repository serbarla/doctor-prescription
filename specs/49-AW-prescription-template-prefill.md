# Spec 49-AW — Prescription template selection and prefill

- Source: `context/screens/PrescriptionNew.md` §§1, 2.2–2.8; `context/screens/Templates.md`; `context/PRD.md` FR-010, FR-020–021, FR-040 and D-21; `persona-sanus/src/main.rs` (`PrescriptionNew`, `Templates`); `src/service.rs` (`template_version_for_prescription`, `create_prescription`); `src/desktop_local.rs` (`create_local_prescription`).
- Depends on: 48-AV.
- Project root: `/Users/sergiobarba/projects/doctor-prescriptions`. All paths and commands below are relative to it.
- Owned files: `specs/49-AW-prescription-template-prefill.md`, `persona-sanus/src/main.rs`, `persona-sanus/src/forms.rs`, `persona-sanus/src/api.rs`, `persona-sanus/src/api_server.rs`, `persona-sanus/src/service.rs`, `persona-sanus/src/desktop_local.rs`
- Gate: `cargo check --manifest-path persona-sanus/Cargo.toml --no-default-features --features desktop && cargo check --manifest-path persona-sanus/Cargo.toml --no-default-features --features web,server`
- Desktop-verify: yes
- User-interface verification: `required` — prescription creation picker and deep-link workflows in §3 on desktop with synthetic records; capture screenshots and non-PHI outcome evidence.
- Result carried forward: The selected current published version, its previewed prefill and the resulting draft agree even after switching templates or following a deep link.

## 1. Objectives and scope

- [ ] 1. Correct selection lookup and stale state in `PrescriptionNew`: match the selected version ID across all eligible templates, not `find_map` of the first current version followed by a filter. Only current published versions of non-archived templates available to the actor are selectable. On every actual selection change derive diagnosis, medication summary and variable summary together from that exact version; clear old derived values before handling empty, missing, malformed or unavailable data, and show an actionable error instead of creating from stale prefill. Preserve unrelated user-entered fields. A delayed result for A cannot replace a later selection B; no stale error or summary survives clearing.
- [ ] 2. Implement the screen's diagnosis contract: prefill from the selected document until the user edits diagnosis, then never overwrite the user's value on subsequent selection/loading, including when the user intentionally clears it. Ensure the created draft persists the displayed value, distinguishing an explicitly cleared diagnosis from an absent legacy field where necessary; retain server-side validation and legacy default behavior for callers that genuinely omit it. Template medications are represented once in the created draft, not copied both by client and server. Variables remain placeholders resolved from authorized context at preview, not PHI in query strings.
- [ ] 3. Normalize deep links once per route/list load: `patient_id` must match an available patient; `template_id` may match an eligible template ID or its current published version ID, both mapping to the current version ID. Blank, invalid, archived, historical or inaccessible IDs leave the relevant picker empty; a user choice or intentional clear made while lists load must win over late query normalization. A changed route query on an already mounted page starts a new normalization for that route without reapplying the old one. No auto-submit or URL rewrite. Validate the selected patient/version again at submit and let the server reauthorize rather than trusting the picker.
- [ ] 4. Keep online and offline draft behavior consistent: use cached eligible versions only for a restored authorized desktop actor; make the local draft contain the selected version's diagnosis and medication prefill once, and queue the same intended version and operator values for replay. Local transaction failure must not leave a partial draft/outbox. On replay, do not duplicate medications or overwrite later offline edits; preserve the pending record and expose failures if the server rejects a stale/inaccessible version. Existing saved drafts, published versions and outbox rows require no destructive migration.

### Out of scope / no-go

- No automated tests, new routes, signing/number allocation, publishing template versions, new image upload or billing. No HTML/rich text, no patient name or clinical content in URL, logs or error metadata. Do not weaken clinic/role authorization or use an offline local draft version as a published template.

## 2. Problem and decision

- Problem: The existing effect selects the first `current_version` before checking its ID, so any later template may leave stale medication/variable summaries; it also never seeds diagnosis. Reapplying query values to empty selections can undo a deliberate clear, and local creation does not copy template medications.
- Decision: Recommended single derived selection state with an explicit user-edited diagnosis flag and one-time, route-scoped deep-link normalization; keep the version ID as the create contract. Established behavior is the screen spec's current-version selection and user-edit protection; the explicit-clear and late-response rules make that behavior deterministic.
- Open questions: None.

## 3. Acceptance and verification

Use synthetic patients and two authorized published templates A and B with different diagnoses, medication rows and variable keys, plus one archived template and one older version; have both doctor and nursing accounts in a disposable clinic. Record exact steps and resulting non-PHI screenshots.

- [ ] UI workflow: choose A then B then A, with B not first in the returned list → each selection shows only its own diagnosis, medications and variables. Select empty, or select a version whose JSON/variables are malformed → no previous summary remains, error is visible for malformed data, and draft creation cannot use stale data. Simulate delayed list completion and rapid changes → only the final selection wins.
- [ ] UI workflow: choose A, type a custom diagnosis, choose B → custom text survives. Clear it intentionally, switch again, create → persisted diagnosis remains empty rather than being silently reseeded; unrelated typed fields remain unchanged. Without editing, switch A to B and create → B's diagnosis and medication rows appear exactly once in the draft.
- [ ] Deep-link workflow: open `/prescriptions/new?patient_id=<synthetic-id>&template_id=<template-id>` then repeat with its current version ID → same patient and version selected, no automatic create; use historical, archived, foreign, invalid and blank IDs → empty picker, no stale summary, no unauthorized fetch. Choose or clear while list loading → late results never override that action. Change the route query while mounted → new route is applied once; an intentional subsequent clear stays clear.
- [ ] Desktop workflow: cache A/B, disconnect, choose B and create a draft → B's diagnosis and medications match online semantics, one local draft/outbox entry, no duplicates. Edit that local draft before reconnect, then sync → later edits remain and medication rows are not doubled. If a version is no longer acceptable at the server, failure remains visible and local draft/outbox survive. On a forced local failure no partial creation is reported.
- [ ] Gate: `cargo check --manifest-path persona-sanus/Cargo.toml --no-default-features --features desktop && cargo check --manifest-path persona-sanus/Cargo.toml --no-default-features --features web,server` exits 0.
- [ ] User-interface verification: perform the desktop picker, diagnosis, deep-link and offline workflows above; retain screenshots without real PHI.
- [ ] Every objective in section 1 is checked.

## 4. Execution and divergences

Read project instructions first. If a dependency is incomplete or required work is outside owned files, record evidence, impact and affected criteria and ask the project owner for scope approval; do not improvise. Unresolved divergences block completion.

### Divergences

None.
