# Divergence tracking: 13-M backup MinIO mirror

Source spec: `specs/13-M-ops-and-release.md`, objective 3 (operational backups). Blocks: objective 3 — "the daily job also backs up the MinIO bucket (same cadence, via the S3 client or `mc mirror`) because `template_version_assets` pins objects — a DB-only backup breaks reprints"; and `context/runbook.md:25` ("the mirror is required, not optional").
Resolving spec: none — requires a backup-remediation spec. 13-M stays in `specs/`.

Severity: P1 — without the object-store half, the SQLite half restores but pinned template images are gone. The job can report success while this is true.

## What's broken

- `persona-sanus/Dockerfile:9` downloads `mc.linux-amd64.RELEASE.2025-08-13T08-35-41Z`. The only architecture selector is the hardcoded URL.
- This workstation is `arm64`; `docker-compose.yml` has no `platform:` override, so a compose build defaults to `linux/arm64`. On an arm64 runtime without x86-64 `binfmt_misc` emulation, `mc` cannot execute. Emulation is environment luck, not an architecture contract. The matching `mc.linux-arm64.RELEASE.2025-08-13T08-35-41Z` asset exists.
- `persona-sanus/src/jobs.rs:251-267` discards both `std::process::Command::status()` results: the `mc alias set` status and the `mc mirror` status. A missing `mc`, `exec format error`, bad credentials, DNS failure, or partial mirror all continue.
- The closure still returns `Ok(encrypted)` at line 270, and the audit at lines 283-290 records `{"path": …, "encrypted": true}`. The runbook's own verification query (`runbook.md:28`) therefore shows a healthy backup when the bucket archive is absent or stale.
- The download is tag-pinned but has no checksum against the MinIO release artifact.
- `state.assets.is_some()` only means the asset environment variables parsed. It is not a reachability check; an unreachable bucket still claims backup success.

## Why each fix was fenced

- The 2026-11-15 pass was report-only; no container build or failure injection was run, and `cargo`/`dx` are not on this shell's PATH.
- The fix has an ops-policy decision: whether a failed bucket mirror fails the whole `backup_daily` run (the runbook says required, so yes) or is recorded as "SQLite valid, object backup failed" with RPO explicitly unmet. The code currently pretends the first interpretation while doing neither.
- Multi-arch support changes the image build contract. Forcing `--platform linux/amd64` preserves the current `mc` binary but makes the server image emulated; selecting per-`TARGETARCH` assets needs a checked mapping.

Verified 2026-11-15: 0 fix attempts — static inspection of `Dockerfile`, `jobs.rs`, `docker-compose.yml`, and `context/runbook.md`; `uname -m` reports `arm64`; the arm64 release asset resolves by URL. Repair is fenced by the spec process; no credential blocks it.

## How to clear it

- [ ] Make the Dockerfile architecture-aware: `ARG TARGETARCH`, map `amd64`/`arm64` to the matching MinIO `mc` asset, fail the build for unsupported values, and verify the download against MinIO's published SHA-256.
- [ ] Replace both ignored `status()` calls with checked execution. On any non-zero exit, return an error naming the stage (`alias` or `mirror`) and capture a bounded stderr tail for the audit metadata.
- [ ] Only return `Ok` after the SQLite snapshot is encrypted and, when object storage is enabled, `mc mirror` completed successfully. On mirror failure, record a `backup_daily` error that includes the SQLite path and the mirror failure; do not record a bare `encrypted: true` success.
- [ ] Keep the mirror destination unique per day, and document whether stale objects are retained or deleted (`mc mirror` does not remove by default). The backup must not mutate the live bucket.
- [ ] Verify architecture: build/run on this arm64 host, then `docker compose run --rm --entrypoint /usr/local/bin/mc persona-sanus --version`; assert no `exec format error` and that the binary reports the target architecture.
- [ ] Verify failure is visible: point `ASSETS_ENDPOINT`/credentials at a non-existent endpoint in a scratch run, trigger one backup, and assert an error audit row with no `"encrypted": true`; restore credentials, rerun, and assert `backups/minio-YYYYMMDD` contains the objects pinned by `template_version_assets`.
- [ ] Run `python3 .blocks/persona-docker-verify.py --dir . --port 8080` and `cd persona-sanus && cargo check --manifest-path Cargo.toml`.
- [ ] Re-check 13-M objective 3 evidence, then delete this file.
