# Spec 43-AQ — Prescription authorization consistency

- Source: `context/PRD.md` §3, §5.5 and §9; `context/architecture.md` §6 (prescriptions); `context/screens/PatientDetail.md` (history), `context/screens/PrescriptionDetail.md` and `context/screens/Prescriptions.md`; `persona-sanus/src/service.rs` (`prescription_edit_allowed`, `get_prescription`, `prescription_document`, `list_prescriptions`) and `src/pdf_server.rs`.
- Depends on: nothing.
- Project root: `/Users/sergiobarba/projects/doctor-prescriptions`. All paths and commands below are relative to it.
- Owned files: `specs/43-AQ-prescription-authorization.md`, `persona-sanus/src/service.rs`, `persona-sanus/src/db.rs`, `persona-sanus/src/api_server.rs`, `persona-sanus/src/pdf_server.rs`, `persona-sanus/src/sync_server.rs`, `persona-sanus/src/desktop_local.rs`, `persona-sanus/src/main.rs`, `persona-sanus/src/http.rs`, `persona-sanus/src/api.rs`, affected callers of their transport APIs, `persona-sanus/src/prescription_authorization_tests.rs`, `persona-sanus/src/prescription_cache_authorization_tests.rs`, `context/architecture.md`, `context/screens/PrescriptionDetail.md`, `context/screens/PrescriptionPreview.md`, `context/screens/Prescriptions.md`, `context/screens/PatientDetail.md`
- Gate: `cd persona-sanus && cargo test --no-default-features --features server prescription_authorization_tests && cargo test --no-default-features --features desktop prescription_cache_authorization_tests`
- Desktop-verify: no
- User-interface verification: `not applicable` — this is an authorization boundary; deterministic HTTP/service and cache tests prove decisions, rather than visual appearance.
- Result carried forward: every prescription access path enforces the same actor- and status-aware policy, including document generation and offline cache reads.

## 1. Objectives and scope

- [x] 1. Establish one server-side policy using current `clinic_members.role`, clinic, and actual doctor identity, not request-provided ownership. PRD §3 establishes doctors' own history and edits, nursing drafts, secretary reprints, and administrator clinic-wide read-only access. **Recommended operational choice:** an unassigned nursing-authored draft (`doctor_id IS NULL`, status draft) is visible and editable by clinic doctors and nursing so a doctor can review and sign it; once signed/attributed, only its owner doctor may read or act as doctor. Nursing may read clinic prescriptions and edit drafts, but never sign/void/supersede; secretary may read and reprint issued rows (signed, voided, superseded), not drafts or mutate; administrator may read clinic-wide but not mutate. An assigned draft belongs only to its doctor for doctor access; signed rows remain immutable and only their doctor may perform existing transitions. Missing doctor profile fails closed. Cross-clinic access fails closed for every role. Apply this policy before assembling sensitive detail, preview or PDF and before *every* mutation, including item upsert/delete/reorder, signing, void, supersede, and idempotent create replay; replay must not return another doctor's data or mutate it. Preserve existing validation, audit and public verification behavior.
- [x] 2. Make history/list and patient-filtered history reflect exactly the permitted rows, including unassigned nursing drafts, applying authorization before the 200-row limit and before filters can expose other doctors' rows. Cover direct detail, preview and PDF using the same policy. Ensure no error response reveals prescription content or whether a foreign ID exists; preserve permitted secretary reprint of issued documents. Audit actual allowed sensitive reads without PHI in metadata.
- [x] 3. Apply equivalent role/identity filtering to desktop cached list, detail, local preview/PDF and local draft operations; do not trust cached UI controls. On identity/role changes or missing identity, do not surface a previous user's cached clinical data. Never delete unsynced local-only work to accomplish filtering; preserve outbox and allow server to reject unauthorized replays visibly. Account for clinic sync/event payloads and any alternate prescription-returning paths so they cannot deliver unauthorized clinical content; do not weaken public minimal QR verification. Preserve offline use for already-authorized cached data; server checks are authoritative upon reconnect. No silent reassignment of existing records.
- [x] 4. Implement deterministic, isolated behavioral regression tests in `persona-sanus/src/prescription_authorization_tests.rs` and `persona-sanus/src/prescription_cache_authorization_tests.rs`, registered from `main.rs` for the appropriate features, using a temporary SQLite database, real service/HTTP and desktop cache entry points and no external server or personal data. Seed two doctors in one clinic, a second clinic, nursing, secretary and administrator, attributed and unassigned drafts, signed and transitioned records. Exercise every route including `/api/prescriptions`, `/{id}`, `/{id}/preview`, `/{id}/pdf`, item routes and transitions, client-id replay, filters, and desktop cache paths; assert responses and persisted state, not just helper decisions. Update the owned context files to reflect the implemented policy and its explicitly recommended unassigned-draft choice.
### Out of scope / no-go


- No new roles, delegation, reassignment workflow, schema migration that rewrites ownership, new legal claim, or deletion of offline work. Public verification remains minimal and unauthenticated as in PRD §5.6.
- Do not rely on hidden buttons, opaque IDs, or client-side filtering as the server authorization boundary. Do not alter unrelated patient, template or PDF rendering semantics.

## 2. Problem and decision

- Problem: history currently filters by doctor while direct detail/document and draft edits are clinic-scoped; a same-clinic doctor can reach another doctor's data or draft through a direct API call. Unassigned nursing drafts also need a coherent route to signing.
- Decision: enforce the matrix in objective 1 at each data boundary, including list and offline paths. The unassigned-draft handoff is a recommendation resolving a gap, not an established explicit PRD decision; it does not confer access to another doctor's assigned records.
- Open questions: None for this bounded implementation; any conflicting behavior or necessary unowned change is a divergence for the project owner.

## 3. Acceptance and verification

- [x] Run the gate: doctor A sees own and unassigned nursing drafts but not doctor B's rows, including patient/status/date filters and direct detail, preview and final PDF; B has the symmetric access. Denied requests expose no clinical body and cannot change fields, items, order, state or number, including replayed `client_id`. Cross-clinic IDs and a doctor without a doctor record fail closed.
- [x] Same tests prove nursing draft create/edit and handoff/sign by a doctor, no nursing signing or transitions, secretary issued reprint but no draft or edits, and administrator read-only; signed/superseded/voided invariants and public verification remain intact. Permitted owner PDF returns PDF bytes; denied PDF never does.
- [x] User-interface verification: not applicable; deterministic boundary tests above are the evidence.
- [x] Same tests prove local list/detail/preview/action filtering across identity/role changes, preservation of unsynced work, and rejection surfaced on reconnect; exercise sync payload/event alternatives without leaking clinical content.
- [x] Gate: `cd persona-sanus && cargo test --no-default-features --features server prescription_authorization_tests && cargo test --no-default-features --features desktop prescription_cache_authorization_tests` exits 0. Observed: 12 server tests and 11 desktop tests passed. The desktop tests verify that a server refusal is recorded as a failed outbox entry even when its message resembles a network error.
- [x] Every objective in section 1 is checked.

## 4. Execution and divergences

Read project instructions and referenced sources before implementation. If a required change lies outside owned files or a dependency/implementation contradicts this matrix, record the current state, evidence, impact, attempted resolution and owner below; stop affected work and request the project owner's scope decision. Unresolved divergences block completion; leave affected criteria unchecked. Archive only per project convention after completion.

### Divergences

Resolved by project owner: legacy desktop prescription rows without authorization provenance remain hidden offline, while the rows and their outbox entries are preserved. Server-backed rows can become visible after a successful authorized refresh; pending local changes are not overwritten by refresh. Local-only work requires successful replay before a server-backed refresh can authorize it. Newly cached and locally created rows record the current identity and role scope. Hashing cannot reconstruct missing provenance.

Resolved: the project owner approved adding `http.rs`, `api.rs`, and affected callers. Prescription reads now carry typed network and response errors, and only a network error permits the desktop cache fallback. A regression test sends a real HTTP refusal containing the former network-error prefix and verifies that it cannot open the cache.

Resolved: the desktop preview now uses a testable local assembly entry point. A regression test verifies the assembled preview and denies it after identity, role, and missing-profile changes.
