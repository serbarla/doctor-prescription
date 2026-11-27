# Spec 44-AR — Issued-document snapshots

- Source: `context/PRD.md` §4 flows 1, 2, 5 and FR-013, FR-023–024, FR-030–034, FR-040–043, FR-085; `context/database-design.md` §§6–7; `persona-sanus/src/service.rs` (`prescription_document`, `sign_prescription`), `persona-sanus/src/pdf_server.rs`, `persona-sanus/src/main.rs` (`sign_prescription_offline`, `offline_preview_variables`), `persona-sanus/src/desktop_local.rs` (`offline_sign`).
- Depends on: 43-AQ.
- Project root: `/Users/sergiobarba/projects/doctor-prescriptions`. All paths and commands below are relative to it.
- Owned files: `specs/44-AR-issued-document-snapshots.md`, `persona-sanus/migrations/*`, `persona-sanus/src/schema.rs`, `persona-sanus/src/db.rs`, `persona-sanus/src/service.rs`, `persona-sanus/src/api_server.rs`, `persona-sanus/src/api.rs`, `persona-sanus/src/pdf_server.rs`, `persona-sanus/src/pdf.rs`, `persona-sanus/src/desktop_local.rs`, `persona-sanus/src/main.rs`, `persona-sanus/src/forms.rs`, `persona-sanus/src/prescription_cache_authorization_tests.rs` and `persona-sanus/src/prescription_authorization_tests.rs` (fixture inputs only, approved during execution), `persona-sanus/README.md`
- Gate: `cd persona-sanus && cargo test --no-default-features --features server issued_snapshot && cargo test --no-default-features --features desktop issued_snapshot`
- Desktop-verify: no
- User-interface verification: not applicable — the invariant is tested at signing, persistence, API and render-input boundaries; this spec adds no visual workflow.
- Result carried forward: new issued prescriptions can be regenerated from their frozen issuance inputs online and offline; old records without recoverable inputs are explicitly identified rather than presented as faithful originals.

## 1. Objectives and scope

- [x] 1. Freeze the actual issued document — requirements: introduce a versioned, private snapshot of every content-bearing render input, including pinned document/layout/theme and asset identities, resolved patient (including age at issue), doctor and clinic values, clinical fields and ordered medications, issue time and number. Capture in the same transaction as successful online signing, number/code allocation and audit; failure rolls back all. Draft previews remain live. Preserve the existing no-stored-PDF model and immutable signed content; status/verification lifecycle remains separate so void/supersede still works. Use existing asset pinning and v1 retention rules; do not duplicate image bytes in outbox or weaken deletion protection.
- [x] 2. Support offline issue and reconciliation — requirements: freeze the same complete inputs and actual local signed timestamp atomically with local number consumption, signed state and outbox enqueue; if required data or pinned assets are missing, fail visibly before signing without consuming a number. Persist snapshot across restart and upload with the sign operation; server validates clinic, actor, ownership, device/block and snapshot structure, binds it to the signed prescription atomically, and does not replace it with current server profile/patient values. Exact replay returns the original snapshot/number/code without changing any; mismatch is rejected visibly and retained for resolution. A provisional render uses that snapshot and inactive QR; the acknowledged final adds only server verification information, never changes clinical content. Do not equate a local timestamp with server receipt time.
- [x] 3. Regenerate safely — requirements: all server PDF/preview and desktop provisional/final reprint paths for signed, voided or superseded records use the frozen inputs rather than live mutable entities; preserve authorization, audit, PHI-free QR, and status semantics. Historical server-backed records still fetch on demand after eviction. For legacy signed records with no reliable issuance snapshot, migration marks them unavailable; never backfill from today's mutable rows as if original. Return an actionable localized unavailable-history message instead of an allegedly faithful PDF; retain access to existing record data and verification. Migrate existing local databases without overwriting pending work; do not silently reconstitute an already signed pending record from current values.
- [x] 4. Implement focused automated behavioral regression tests named with `issued_snapshot` in the existing Rust source test modules (server and desktop feature builds), and document legacy limitation/recovery and snapshot privacy in `persona-sanus/README.md`. Tests must exercise real persistence/transaction and render-input assembly rather than only serialization or mocks. Use isolated temporary SQLite fixtures and no external network or PHI.

### Out of scope / no-go

- No digital signature, cryptographic non-repudiation claim, PDF-byte identity guarantee, invented historical data or retroactive legal assurance; the doctor signs paper (PRD §4).
- No public snapshot/PHI endpoint, PHI in QR, logs, filenames beyond the existing documented PDF filename policy, or new image bytes in the outbox. Follow current authorization and local-storage protections. No billing/v1.1 behavior changes; PRD FR-085 explicitly permits future cancellation placeholders.
- Do not edit context or other specs. Do not change image lifecycle beyond preserving existing pinned-asset behavior. If reliable pre-migration recovery or an unowned file is needed, record a divergence and ask the project owner.

## 2. Problem and decision

- Problem: issuing binds a template version, but server document generation reads current patient, doctor and clinic rows, and desktop computes preview values live. Later changes can make a reprint differ from what was issued.
- Decision: PRD requires equivalent historical content, not byte-identical PDFs. Recommended implementation is a versioned issuance snapshot with a nullable legacy marker, made durable with the signing transaction and carried by offline reconciliation. Lifecycle-only verification fields may change independently. Exact schema is an implementation choice, not an established product decision.
- Open questions: None; legacy unavailability and missing-input failure are conservative recommendations authorized for this spec, not a legal conclusion.

## 3. Acceptance and verification

- [x] Gate: `cd persona-sanus && cargo test --no-default-features --features server issued_snapshot && cargo test --no-default-features --features desktop issued_snapshot` exits 0. Each feature run must execute its nonzero named tests; inspect test counts.
- [x] Server `issued_snapshot` tests: sign with representative patient, doctor, clinic, template, age/date, asset and medication values, mutate each live source, and assert actual PDF/preview render inputs remain issued values; unauthorized/cross-clinic reads fail and verification payload has no PHI. Forced snapshot failure rolls back status, number, code and audit. Voiding/superseding does not mutate the old snapshot. Repeated sign has one immutable snapshot and one code.
- [x] Desktop `issued_snapshot` tests: offline sign transaction, restart/reopen, change live cache, compare provisional inputs, reconcile and compare final content excluding explicit QR/status change; failed missing data/asset consumes no number, and offline path requires no network. Repeated reconciliation, altered payload, and failed transport preserve pending data and never replace issued content. Server-side matching reconciliation and idempotency are covered by server tests.
- [x] Migration tests in the same gate start from pre-migration server and local schemas with signed, draft and pending records; signed legacy records report unavailable honestly, drafts remain editable and pending records survive. Verify no silent live-data backfill and no success response claiming a faithful reprint.
- [x] User-interface verification: not applicable as above. Every objective in section 1 is checked only after evidence passes.

## 4. Execution and divergences

Read applicable `AGENTS.md` files before execution. If implementation requires unowned files, conflicts with a dependency, or cannot preserve old pending data, stop affected work and record evidence, impact, owner and requested decision here; ask the project owner for scope changes. Open divergences and incomplete dependencies block completion. Archive only after all criteria pass.

### Divergences

### Existing desktop authorization fixture requires additional ownership

- Current state and evidence: `persona-sanus/src/prescription_cache_authorization_tests.rs:102-129` expects successful offline signing without cached patient or template data and supplies only identity IDs. Objective 2 requires signing to fail when required issuance inputs are missing.
- Attempts: inspected the existing fixture and signing callers before changing the signing contract. No fixture changes or production validation bypass have been applied.
- Impact: the authorization regression fixture must supply complete issuance inputs to retain its intended successful-signing assertion. That file is not listed in this spec's owned files.
- Resolution: the project owner approved the requested ownership extension during execution. The fixture may gain representative cached issuance inputs; its authorization assertions must remain intact.
- Owner: project owner, as scope-change approver.
- Status: scope divergence resolved. Verification passed; see evidence below.

### Existing server authorization fixture requires additional ownership

- Current state and evidence: `persona-sanus/src/prescription_authorization_tests.rs:144-147` expects successful PDF regeneration for a directly seeded signed record without an issuance snapshot.
- Attempts: inspected the existing assertion and identified the conflict with objective 3. No bypass or fixture changes have been applied.
- Impact: preserve the authorized owner/secretary PDF checks by issuing their fixture through real signing with complete inputs. A legacy record must remain unavailable for faithful rendering.
- Resolution: the project owner approved the additional fixture ownership during execution. Retain the successful authorized owner/secretary PDF assertions using reliable issuance inputs.
- Owner: project owner, as scope-change approver.
- Status: scope divergence resolved. Verification passed; see evidence below.

### Verification evidence

- Final verification after the last code edit: both specified `issued_snapshot` feature commands exited 0; server executed 6 tests, desktop executed 5 tests.
- Affected caller regressions also passed: `cd persona-sanus && cargo test --no-default-features --features server prescription_authorization_tests` (12 tests) and `cd persona-sanus && cargo test --no-default-features --features desktop prescription_cache_authorization_tests` (11 tests).
- Coverage includes frozen render-input and actual PDF assembly, authorization/private delivery, atomic rollback, lifecycle transitions, offline restart and reconciliation, mismatch/transport preservation, missing inputs/assets and legacy migrations.
- The initial desktop gate exposed use of the server-only optional `image` dependency. Desktop validation now reuses the renderer's `printpdf::RawImage` decoder and its PNG-generation helper for the fixture; no dependency or manifest expansion was required.
- No live desktop visual workflow was run; the spec's nonvisual persistence/API/render-input verification applies. Compiler warnings remain; no unrelated warning cleanup was performed.
