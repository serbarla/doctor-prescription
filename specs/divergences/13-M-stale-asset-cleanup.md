# Divergence tracking: 13-M stale asset cleanup

Source spec: `specs/13-M-ops-and-release.md`, objective 1 (in-process scheduler) and its criterion `objective 1: jobs are idempotent … stale pending assets are removed after one hour`. Blocks: objective 1 requirement "stale `pending` asset cleanup (rows older than one hour, row and object deleted); each completed run writes one `audit_log` row", and the objective-1 testing criterion.
Resolving spec: none — requires a jobs-remediation spec. 13-M stays in `specs/`.

Severity: P2 — object-storage failures are treated as successful cleanup, leaving the database and bucket inconsistent; the audit trail can claim nothing was deleted while rows were deleted.

## What's broken

`persona-sanus/src/jobs.rs:84-150`:

- Line 104: `let _ = storage.delete_object(object_key).await;` — every S3 error is discarded. The loop then unconditionally proceeds to `hard_delete_asset` at line 115. A storage outage removes the row and leaves the object; the upload quota no longer sees it, so a later upload can fill the bucket with orphaned objects.
- Lines 115-124: `hard_delete_asset(...).is_ok() && audit(...).is_ok()` — these are two autocommit statements, not one transaction. If the row delete succeeds and the audit insert fails, `outcome` is false and `removed` is not incremented; when no other asset succeeds, the run then writes the aggregate `{"deleted": 0}` at line 145, which is false.
- Lines 132-149: the aggregate audit row is written only when `removed == 0`. A run that deletes some rows and silently fails others has no run-level failure record; per-asset success audits alone make it look complete.
- When storage is `None`, line 103 is skipped and the row is still hard-deleted. No object delete was attempted, yet the code behaves as if the cleanup was complete.
- The same ignored-result pattern makes a missing bucket, suspended MinIO, invalid credentials, or a network flap indistinguishable from a successful cleanup.

## Why each fix was fenced

- Fail-closed behavior is a data-retention decision: should a row whose object cannot be deleted stay pending forever, be retried with backoff, or be marked failed? The asset schema's `status` only allows `pending|ready`, and there is no retry/error column; adding one here would be schema drift outside 13-M.
- Testing requires MinIO to be stopped or credentials corrupted mid-job. That failure environment was not built in the report-only pass.
- Audit atomicity needs the row delete and audit insert inside `db::immediate_transaction`; doing it ad hoc would duplicate spec 01-A's transaction contract and needs a remediation spec to name it.

Verified 2026-11-15: 0 fix attempts — static inspection of `jobs.rs:84-150` and the `AssetStorage::delete_object` error contract in `assets.rs:198-209`. Fenced by the unresolved failure/retention policy and the spec process.

## How to clear it

- [ ] Storage failure: do not hard-delete the asset. Record an `asset_cleanup_failed` audit row with asset id, object key, and a bounded error string, then continue with the next asset. The next scheduler tick retries.
- [ ] Storage success: perform `hard_delete_asset` and the `asset_cleanup` audit insert inside one `db::immediate_transaction`; if either fails, roll back and leave the row for the next run.
- [ ] Storage not configured: do not claim object deletion. Skip cleanup with an `asset_cleanup_skipped` audit (`"reason":"storage_not_configured"`) or fail closed; pick one and document it. No silent row delete.
- [ ] Run-level audit: write one summary per run even when `removed > 0`, with `deleted`, `failed`, and `skipped` counts, so a partial run is visible. Keep per-asset error rows for forensics.
- [ ] Verify success: seed a `pending` asset older than one hour with its object; run the scheduler; assert both gone and one `asset_cleanup` row with `deleted > 0`.
- [ ] Verify failure: seed a stale pending asset, make MinIO unavailable (stop `minio` or set a bad `ASSETS_SECRET_ACCESS_KEY`), run with `JOBS_INTERVAL_SECONDS=5`; assert the row remains pending and an `asset_cleanup_failed` row exists. Restore MinIO; assert the next run deletes both.
- [ ] Confirm re-running each completed cleanup changes nothing beyond its own audit row (idempotency under the corrected semantics).
- [ ] Mark 13-M objective 1 and its testing criterion `[x]`, record the failure-injection evidence, and delete this file.
