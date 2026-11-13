# Spec 07 — Phase G: Prescriptions and numbering

- Source: `context/PRD.md` §5.2 (FR-010–014), D-02, D-20; `context/sitemap.md` §1.2 (`/prescriptions*`), §1.3 (LLM link); `context/database-design.md` §5, §6 (block-allocated numbers, state machine, immutability).
- Depends on: 02-B, 03-C, 04-D, 05-E.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Result carried forward: doctors issue prescriptions through `draft → signed → superseded/voided` with ordered medication items and block-allocated unique numbers, and role immutability is enforced server-side.

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

- [x] 1. Implement the prescription service — requirements: `create_prescription(draft)` from patient + pinned `template_version_id`, `update_prescription` (draft only), `upsert_prescription_item` (ordered `position`, draggable), `get_prescription`, `list_prescriptions` (filters: patient, date, status) under `/api/prescriptions`; fields per FR-010 (date/time, diagnosis, general instructions as text, validity); role rules server-side (Doctor signs own, Nursing drafts and edits medication fields but cannot sign or publish, Secretary reads and reprints only, Administrator reads clinic-wide); `client_id` idempotency on create; every mutation and sensitive read writes audit rows; commands: `cargo check --manifest-path persona-sanus/Cargo.toml`
- [x] 2. Implement number-block allocation — requirements: `allocate_rx_block` runs in the `db::immediate_transaction` helper (spec 01-A — Diesel's `transaction()` is deferred) — insert `rx_counters` if absent, advance `last_rx_number`, insert `rx_blocks(start_number, end_number, consumed)`; block size comes from `clinics.rx_block_size` (default 50, schema spec 01-A); the allocating device comes from `ActorContext.device_id` (`sessions.device_id`, spec 04-D) and a web client registers its `devices` row (`platform='web'`) before its first allocation; `consume_rx_number` takes the next number inside the device's active block and increments `consumed`; `GET /api/rx/block-status` returns the active block's remaining count and bounds; the block-exhaustion state is returned to the client; commands: `cargo check --manifest-path persona-sanus/Cargo.toml`
- [x] 3. Implement signing and transitions — requirements: `sign_prescription` in one transaction consumes a block number (allocating a block when the active one is exhausted or absent), sets `doctor_id`/`signed_at`/`rx_number`/`rx_block_id`, creates the `verification_codes` row (`code` printable + `code_hash` unique lookup, `create_verification_code`), and writes audit rows; signing is idempotent on the prescription's `client_id` (a replay returns the existing signed row — no second number or code); unknown template variables raise a pre-signing warning: the client acknowledges with `acknowledge_warnings: true`, otherwise signing returns `409 unacknowledged_warnings` listing the unknown keys (FR-023); `void_prescription` requires `void_reason` and sets `verification_codes.status='voided'` in the same transaction; `supersede_prescription` creates a new row with its own number and code and marks the old row `superseded` with `verification_codes.status='superseded'` (its code stays resolvable and reports `superseded`); signed rows have no update path other than void/supersede; commands: `cargo check --manifest-path persona-sanus/Cargo.toml`
- [x] 4. Build the prescriptions UI — requirements: routes `/prescriptions` (history with search/filter), `/prescriptions/new` (`?patient_id=&template_id=` optional; pickers when absent), `/prescriptions/:id` (form/detail; `draft` editable, `signed` locked with *Void*/*Supersede* actions); per-medication *Explain with LLM* link opening `clinics.llm_url_template` in a new tab with the medication name only plus disclaimer (FR-012); remaining-number display (from `GET /api/rx/block-status`) and a visible block-exhaustion warning that blocks signing offline-capable flows; `es-EC` labels; commands: `cd persona-sanus && dx build --release --fullstack`
- [x] 5. Extend `.blocks/persona-probe.py` with `--flow rx` — requirements: sign in, create patient and template, publish the template, create a draft with two items, reorder items, sign it (asserts `rx_number` allocated from a device block and a verification code created; a replayed sign with the same `client_id` returns the same row and changes no counts), supersede it (asserts the new number/code and `superseded` status on both rows), and void a third (asserts `verification_codes.status='voided'`); the JSON verdict carries created `patient_id`, `template_id`, `prescription_id`, `rx_number`, and `code` so later blocks compose it; block header and indexes updated; commands: `python3 .blocks/persona-probe.py --base http://127.0.0.1:8080 --flow rx`

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No PDF rendering, QR embedding, or `printpdf` dependency (spec 08-H).
- No sync, outbox, or offline code (specs 10-J, 11-K); `reconcile_rx_block` belongs to spec 11-K.
- No `/verify/:code` page (spec 09-I).
- No billing or entitlement code (v1.1 only).
- No unit tests; no comments in any code.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [x] `python3 .blocks/persona-smoke.py --dir persona-sanus --port 8091` passes (fmt, check, clippy, release build, auth probe)
- [x] objective 5: `python3 .blocks/persona-probe.py --base http://127.0.0.1:8080 --flow rx` → all checks true (draft, items, sign, block number, code, supersede, void)
- [x] objective 2: `python3 .blocks/sqlite-schema-check.py --db <booted-db> --query "select start_number, end_number, consumed, status from rx_blocks"` → one active block per allocating device, `consumed` matching signed count; a second allocation for the same device is refused while a block is active
- [x] objective 3: signing the same `client_id` twice returns the same prescription (no duplicate rows: `select count(*) from prescriptions` unchanged after replay)
- [x] objective 4: `http://127.0.0.1:8080/prescriptions/:id` in a browser → nursing cannot see *Sign*, secretary cannot edit a signed row, the LLM link carries only the medication name, and the remaining-number indicator matches the API
- [x] every objective in section 1 is checked

## 4. Divergences

*Process and tracking-file rules: see `AGENTS.md` (§ Spec-Driven Development).
Summary: when reality diverges from the spec, append a numbered entry below;
leave the matching testing-criteria `- [x]` unchecked and note the divergence
inline next to it. Open divergences owned by a later spec also get a tracking
file in `specs/divergences/`.*

- [x] every divergence found is appended below (or the section reads "None.")

*Entry format (one numbered item per divergence):*

1. <what diverged and its current state> — <why it could not be done as written, citing the No-go rule or spec that blocks it> (Verified <date>: <N> fix attempts — <attempt 1>, <attempt 2>, <attempt 3> — all fenced by <constraint>; <impact> is deferred to <later spec> by design, not a defect.)

### Divergences

None.

---

*Housekeeping: once a spec is fully done with no open divergences, move its
file to `specs/done/` (e.g. `specs/done/07-G-prescriptions-and-numbering.md`).
Specs with open divergences stay in `specs/` next to `TEMPLATE.md` until the
tracking file is cleared.*
