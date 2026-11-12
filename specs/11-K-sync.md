# Spec 11 — Phase K: Sync

- Source: `context/PRD.md` §5.5 (FR-041–043), §4 flow 2, D-11; `context/architecture.md` §9 (offline sync contract), §17 risk 5; `context/database-design.md` §6 (idempotency, block validation), §7.
- Depends on: 03-C, 04-D, 05-E, 07-G, 10-J.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Result carried forward: on reconnect the desktop outbox drains oldest-first into idempotent server mutations, number blocks reconcile with burned tails recorded, and final numbered PDFs with active QR replace provisional ones — over SSE with a 30-second polling fallback.

## Before you start

Unit testing is not required; avoid it. Prioritize end-to-end and integration
testing only.

Read the nearest `AGENTS.md` (target workspace, then parents up to repo root)
before executing.

If reality diverges from the spec: STOP, append a divergence (section 4), and
NEVER improvise outside the objectives.

## 1. Objectives

*The ordered work to implement — one item per step. Check `[x]` only after the
step is implemented and verified.*

*Item format:* `<verb phrase>` — requirements: `<what must change / constraints>`; commands: `<exact command>`.

- [ ] 1. Make every outbox-replayed mutation idempotent — requirements: each mutating endpoint accepts the client-generated `client_id` and a replay returns the existing record instead of duplicating it — add `client_id` TEXT UNIQUE columns to `patients` and `templates` (`context/architecture.md` §17 risk 5) and cover every outbox operation: `template_versions` rows carry a client-generated `id` (publish replay = no-op), `upsert_prescription_item` keys on the client-generated item `id`, `void`/`supersede` are no-ops when the transition already happened, and `sign_prescription` replay is idempotent per spec 07-G; updates carry full state so they are naturally idempotent; commands: `cargo check --manifest-path persona-sanus/Cargo.toml`
- [ ] 2. Implement `reconcile_rx_block` — requirements: on sync the server accepts a signed prescription's `rx_number` only when it lies inside a block issued to that device and is unused — the device resolves from the session's `sessions.device_id` (spec 04-D; the outbox payload carries `rx_block_id`, which must belong to the same device and clinic) — validating and writing in one transaction (prescription, items, verification code, `rx_blocks.consumed`, audit); duplicate or foreign numbers are rejected with the outbox row marked `failed` plus a visible reason and the user notified; unused block tails stay burned and recorded in `rx_blocks` (D-02); commands: `cargo check --manifest-path persona-sanus/Cargo.toml`
- [ ] 3. Add the push and fallback channels — requirements: an SSE endpoint on the existing axum router (`GET /api/sync/events`) notifies clients of server-side changes, with a 30-second polling fallback (`GET /api/sync/state`) when SSE is unavailable; events are filtered to the requesting session's clinic and carry entity ids/types only (no PHI in payloads); no second socket server; refresh of cached blocks and metadata on both channels; commands: `python3 .blocks/persona-docker-verify.py --dir . --port 8080`
- [ ] 4. Drain the outbox on the desktop — requirements: oldest-first drain with retry/backoff per `attempts`/`next_retry_at`, visible per-row failure state, and never-discard policy; after a signed row reconciles, the final PDF regenerates with the assigned number and active QR (`src/pdf.rs` in-process); server wins for signed data; only an in-progress draft prompts for a conflict choice; commands: `cd persona-sanus && cargo check --no-default-features --features desktop`
- [ ] 5. Extend `.blocks/persona-probe.py` with `--flow sync` — requirements: uploads a signed prescription twice with the same `client_id` (second call is a no-op returning the same id), rejects a `rx_number` outside the device's block, rejects a reused number, and confirms the burned tail is recorded in `rx_blocks`; block header and indexes updated; commands: `python3 .blocks/persona-probe.py --base http://127.0.0.1:8080 --flow sync`

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No billing or entitlement gate (`can_sync`, `402 sync_locked`) — v1.1 only; v1 sync is ungated.
- No local-cache eviction changes (spec 10-J owns eviction; this spec only reconciles uploads).
- No second socket server, no WebSocket; SSE + polling only.
- No unit tests; no comments in any code.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [ ] `python3 .blocks/persona-smoke.py --dir persona-sanus --port 8091` passes (fmt, check, clippy, release build, auth probe)
- [ ] `python3 .blocks/persona-docker-verify.py --dir . --port 8080` passes (compose build, boot, health, auth persistence)
- [ ] objective 5: `python3 .blocks/persona-probe.py --base http://127.0.0.1:8080 --flow sync` → all checks true (idempotent replay, foreign/reused number rejected, burned tail recorded)
- [ ] objective 3: `curl -N http://127.0.0.1:8080/api/sync/events` (with a session cookie) streams change events, and stopping SSE falls back to `GET /api/sync/state` polling within 30 seconds
- [ ] objective 4: desktop reconnect run (manual, report limits as a divergence) → outbox drains oldest-first, the final PDF shows the reconciled number and a scannable QR pointing at `/verify/{code}`
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
file to `specs/done/` (e.g. `specs/done/11-K-sync.md`).
Specs with open divergences stay in `specs/` next to `TEMPLATE.md` until the
tracking file is cleared.*
