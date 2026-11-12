# Spec 10 — Phase J: Desktop offline

- Source: `context/PRD.md` §5.5 (FR-040–043), §4 flow 2, D-07; `context/architecture.md` §7 (desktop), §9 (sync contract); `context/database-design.md` §7 (desktop local schema).
- Depends on: 04-D, 05-E, 06-F, 07-G, 08-H.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Result carried forward: the desktop works fully offline after first login — cached patients/templates/prescriptions, an outbox that queues every mutation, offline signing with preallocated numbers, provisional PDFs, and honest sync indicators.

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

- [ ] 1. Build the desktop local database — requirements: a separate local SQLite per `context/database-design.md` §7 (`outbox`, `sync_meta`, `asset_cache`, plus cache mirrors of the server clinical tables including the `clinics` row for `retention_days` and `assets` metadata); the `desktop` feature gains `diesel` (`sqlite`, `r2d2`) and `libsqlite3-sys` (`bundled`); the local DB lives at the OS app-data dir and its schema is created idempotently at startup (`CREATE TABLE IF NOT EXISTS`, separate from the server's embedded migrations); first login populates the cache; `evict_local_cache_row` (client-only) removes server-backed rows older than `clinics.retention_days` (default 30) and never touches rows with no server copy; `sync_meta` caches `last_synced_at`, the active `rx_block`, and `image_quota=1`; OS disk encryption (FileVault/BitLocker) is a documented deployment prerequisite (D-07), no app-level encryption; commands: `cd persona-sanus && cargo check --no-default-features --features desktop`
- [ ] 2. Implement the outbox — requirements: every offline mutation queues an `outbox` row (operations per `context/database-design.md` §4) with the client-generated `client_id` (= row id), `payload_json`, `attempts`, `next_retry_at`, `last_error`, and visible `pending`/`failed` state; the queue is drained only by spec 11-K; `pending_sync` is derived from the outbox, never a server column; the outbox is never discarded; commands: `cd persona-sanus && cargo check --no-default-features --features desktop`
- [ ] 3. Sign offline — requirements: signing consumes the next number in the device's active `rx_blocks` row via `consume_rx_number`; when the block is exhausted further signing is blocked with a visible warning and remaining count; the signed row queues in the outbox; the provisional PDF (rendered in-process by `src/pdf.rs` from spec 08-H) carries `PENDIENTE DE SINCRONIZACIÓN`, a placeholder QR, and no final verification code; commands: `cd persona-sanus && cargo check --no-default-features --features desktop`
- [ ] 4. Cache images and show sync state — requirements: `asset_cache` stores images pinned by cached published templates (SHA-256 verified, per-asset 10 MiB cap — the upload cap, so logo + library ≤ 20 MiB in v1 — never evict a referenced image, filenames use the asset id only); the Navbar shows online / offline / syncing / pending uploads (count) / provisional markers per FR-043 and links the pending count to filtered `/prescriptions?status=pending_sync`; new image uploads are refused offline; commands: `cd persona-sanus && cargo check --no-default-features --features desktop`
- [ ] 5. Prove the offline flow on a desktop build — requirements: end-to-end run of the desktop binary: login once, go offline (disconnect network), create patient + draft + sign (number consumed, provisional PDF, outbox row), verify the cache renders history without network, then reconnect and observe the pending count; commands: `cd persona-sanus && cargo check --no-default-features --features desktop`

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No server-side sync endpoints, SSE, polling, or outbox drain (spec 11-K); no `reconcile_rx_block` (spec 11-K).
- No app-level database encryption (D-07); no Linux desktop target.
- No billing-driven states (`sync_locked`, `SINCRONIZACIÓN BLOQUEADA`) — v1.1 only (FR-043).
- No image upload queueing offline (images are online-only).
- No unit tests; no comments in any code.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [ ] `python3 .blocks/persona-smoke.py --dir persona-sanus --port 8091` passes (fmt, check, clippy, release build, auth probe)
- [ ] `cd persona-sanus && cargo check --no-default-features --features desktop` passes
- [ ] objective 5: desktop run described in objective 5 completes with: number consumed from the active block, provisional PDF marked `PENDIENTE DE SINCRONIZACIÓN`, one `outbox` row, pending count visible; report any step that could not be observed on this machine as a divergence
- [ ] objective 1: a server-backed row older than `retention_days` disappears from the local cache while an unsynced row survives (`outbox` row still present)
- [ ] objective 3: exhausting the block offline shows the warning and blocks signing (no number consumed past `end_number`)
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
file to `specs/done/` (e.g. `specs/done/10-J-desktop-offline.md`).
Specs with open divergences stay in `specs/` next to `TEMPLATE.md` until the
tracking file is cleared.*
