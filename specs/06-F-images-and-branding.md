# Spec 06 — Phase F: Images and branding

- Source: `context/PRD.md` §5.8 (FR-080–085), D-05, D-06; `context/architecture.md` §10, §14 (ASSETS_* config), §15, §18; `context/database-design.md` §5 (`assets`, `template_version_assets`), §6 (image rules).
- Depends on: 01-A, 02-B, 05-E.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Result carried forward: a clinic uploads its logo and its single 10 MiB library image through a validated, quota-checked, EXIF-stripping flow into a private MinIO bucket served by 15-minute signed URLs, and published template versions pin the assets they reference.

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

- [ ] 1. Add MinIO to the stack and configuration — requirements: a private-bucket MinIO service in `docker-compose.yml` (named volume, healthcheck, and the app `depends_on` its healthy state), bucket creation at server startup with bounded retry/backoff (MinIO down = server stays up and `/api/assets` returns `503` until reachable), and the `ASSETS_*` variables of `context/architecture.md` §14 (`ASSETS_BUCKET`, `ASSETS_ENDPOINT`, `ASSETS_REGION`, `ASSETS_ACCESS_KEY_ID`, `ASSETS_SECRET_ACCESS_KEY`, `ASSETS_SIGNED_URL_TTL_SECONDS=900`) plus `ASSETS_PUBLIC_ENDPOINT` (browser-reachable base for signed URLs — Compose maps MinIO's port to the host so the URL the browser opens is not `http://minio:9000`); no public object URLs; commands: `python3 .blocks/persona-docker-verify.py --dir . --port 8080`
- [ ] 2. Implement the upload and delete service — requirements: `POST /api/assets` (multipart — `axum` gains the `multipart` feature — server-proxied) and `DELETE /api/assets/:id` with role gates (Administrator clinic-wide, Doctor for own templates; Nursing/Secretary read-only); v1 quota is one non-deleted `kind='image'` per clinic enforced inside the `db::immediate_transaction` helper (spec 01-A — Diesel's `transaction()` is deferred and would race): the transaction only counts and inserts the `pending` row, commits before the bucket PUT (never held across network I/O), and returns `409 image_quota_exceeded` when full; the route raises axum's default 2 MiB body limit (`DefaultBodyLimit::max` ≈ 11 MiB) and enforces the 10 MiB (10,485,760 bytes) cap while streaming; JPEG and PNG only — WebP and SVG rejected by decoder allowlist (`image` crate; SVG has no magic bytes) — decoded with `image::Limits` (dimension/allocation caps, decompression-bomb guard) for `width`/`height`, EXIF/GPS metadata stripped, `sha256` recorded; object I/O and presigning use an S3/SigV4 client (`rust-s3`, named in implementation); a failed upload deletes row and object; `audit_log` rows `asset_upload`/`asset_delete` carry metadata only; commands: `cargo check --manifest-path persona-sanus/Cargo.toml`
- [ ] 3. Serve assets via signed URLs — requirements: objects live under `clinics/{clinic_id}/images/{asset_id}` in the private bucket; `GET /api/assets/:id/url` returns a 15-minute signed GET URL to authorized roles only (signed against `ASSETS_PUBLIC_ENDPOINT`); object keys and URLs carry no PHI; commands: `cargo check --manifest-path persona-sanus/Cargo.toml`
- [ ] 4. Add branding and pinning rules — requirements: `POST /api/clinic/logo` stores `kind='logo'` (same validation, excluded from the quota; at most one non-deleted logo per clinic — replacing the logo soft-deletes the previous row and object; no signature or stamp assets exist — D-06) and sets `clinics.logo_asset_id` following the `context/database-design.md` §6 insertion order; build `/settings/clinic` (Administrator: name/address/phone, logo upload, library usage and delete — also hosts the spec 13-M export action); template publish writes `template_version_assets` pins; deletion of an image referenced by a published version or retained prescription is refused; uploads are online-only (the desktop never queues image bytes); the template editor's image block now inserts library images; commands: `cargo check --manifest-path persona-sanus/Cargo.toml`
- [ ] 5. Create `.blocks/asset-probe` — requirements: new block (header per `.blocks` rules, indexed in `AGENTS.md` and `.blocks/INDEX.md`) that signs in, uploads a valid JPEG under the quota (fixture bytes synthesized in stdlib/base64 — no third-party imaging deps), rejects a second upload with `409 image_quota_exceeded`, rejects an oversized file and a WebP/SVG with `4xx`, checks `width`/`height`/`sha256` are stored, asserts the signed URL carries `X-Amz-Expires=900` and a tampered short-expiry variant stops working (never waits 900 s), and proves a pinned image cannot be deleted; commands: `python3 .blocks/asset-probe.py --base http://127.0.0.1:8080`

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No state-based quota tiers (5/1/0), `402 image_upload_requires_plan`, placeholders, or `canceled` purge — v1.1 only (PRD D-04, D-05).
- No billing or entitlement code (v1.1 only).
- No signature or stamp image kinds (D-06); no patient photos.
- No orphan-cleanup or retention jobs (spec 13-M).
- No unit tests; no comments in any code.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [ ] `python3 .blocks/persona-smoke.py --dir persona-sanus --port 8091` passes (fmt, check, clippy, release build, auth probe)
- [ ] `python3 .blocks/persona-docker-verify.py --dir . --port 8080` passes (compose build incl. MinIO, boot, health, auth persistence)
- [ ] objective 5: `python3 .blocks/asset-probe.py --base http://127.0.0.1:8080` → all checks true (quota, formats, size cap, metadata, signed URL TTL, pin protection)
- [ ] objective 2: uploading the same file twice leaves exactly one non-deleted `kind='image'` row (`python3 .blocks/sqlite-schema-check.py --db <booted-db> --query "select count(*) from assets where kind='image' and deleted_at is null"` → `1`)
- [ ] objective 4: `http://127.0.0.1:8080/settings/clinic` in a browser → logo upload does not consume the library slot; `/templates/:id/edit` inserts the library image and publish pins it
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
file to `specs/done/` (e.g. `specs/done/06-F-images-and-branding.md`).
Specs with open divergences stay in `specs/` next to `TEMPLATE.md` until the
tracking file is cleared.*
