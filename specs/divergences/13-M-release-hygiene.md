# Divergence tracking: 13-M release hygiene

Source spec: `specs/13-M-ops-and-release.md`, objective 5 (runbook and release evidence) and its result carried forward ("ships encrypted daily backups and a runbook"). Blocks: the release-facing claim that operators can discover and run the shipped build; the repository's entry README and the `context/` status tables no longer describe it. None of these items changes runtime behavior.
Resolving spec: none — fold into the remediation spec that closes the P1 backup/update/deploy packets; do not close 13-M independently. 13-M stays in `specs/`.

Severity: P3 — documentation and release metadata drift. It does not expose PHI or weaken auth, but it sends reviewers and operators to the wrong surfaces and understates what must be backed up and configured.

## What's broken

- `persona-sanus/README.md` is 67 lines and describes the auth-demo slice: a six-module tree, `app.db`, auth routes only (`:5-19`), and "no CORS setup is needed" (`:21`). The runtime has 20 modules including `api_server`, `service`, `assets`, `jobs`, `sync_server`, `verify_server`, `pdf_server`, `desktop_local`, and `update`; auth, assets, jobs, and sync are the relevant server surfaces. The "no CORS setup" line is accurate only for same-origin web use; it omits `PERSONA_WEB_ORIGINS`, which enables credentialed CORS for another web origin (`context/architecture.md:434`).
- The README environment table (`:50-55`) lists only `DATABASE_URL` and `PERSONA_COOKIE_SECURE`. It omits `PUBLIC_BASE_URL`, `PERSONA_WEB_ORIGINS`, all `ASSETS_*`, `BACKUP_DIR`, `BACKUP_AGE_RECIPIENT`, `JOBS_INTERVAL_SECONDS`, `RETENTION_TRIM_ENABLED`, `AUTH_RATE_LIMIT_PER_MINUTE`, `VERIFY_RATE_LIMIT_PER_MINUTE`, `UPDATE_*`, and the desktop `PERSONA_SERVER_ORIGIN`/local data dir. It also never says object storage is required for image features or that the backup recipient is required for backups.
- The README acknowledges only the auth and dev-SQLite surfaces; MFA, devices, patients, templates, prescriptions, export, retention, backups, sync, updates, and public verification are undocumented entry points. `context/runbook.md` covers operations but is not linked.
- `context/architecture.md` still marks shipped features as planned: line 38 (`Audit, MFA, rate limiting, backups` ⬜), line 223 (`/audit`, `/verify/:code` ⬜), lines 251-255 (images, verification, audit, sync, billing), and lines 402-409 (asset cleanup, scheduler/backup) — while the code and specs 12-L/13-M/14-N claim them implemented. `context/sitemap.md:74`, `:95`, `:100` likewise mark `/verify/:code`, `/settings`, and `/audit` planned.
- `persona-sanus/Dioxus.toml:6` sets the browser title to `"persona-sanus"` instead of the product name.
- `persona-sanus/Cargo.toml:4` has `"Segio Barba"`; likely `"Sergio Barba"`, but this is a personal-name field and must be confirmed by the owner.
- `persona-sanus/Cargo.toml:9`, `:39`, `:42-43`, `:59-60` declare and feature-enable `dioxus-motion`; `rg -ni 'dioxus_motion|dioxus-motion|motion' persona-sanus/src` returns no use. It is dead build weight.
- `persona-sanus/Dockerfile` does not set `ENV IP=0.0.0.0` or `EXPOSE 8080`. Compose supplies `IP`, so the documented compose path works, but the image carries no standalone bind/port contract. The `mc` checksum gap is recorded in `13-M-backup-minio-mirror.md`.

## Why each fix was fenced

- Docs are the source of truth and need owner sign-off on the public/ops wiring. Updating them during the report-only pass, before the P1 fixes choose the final backup paths and proxy policy, would produce a second stale description.
- The English copy sweep is tracked separately in `13-M-es-ec-copy.md`; rewriting README examples now would overlap that diff.
- Removing `dioxus-motion` changes the feature graph and needs a three-target `cargo check`; `cargo`/`dx` are not on this shell's PATH in this pass.
- The author field is personal data; a spelling change must come from the owner, not be inferred from the email.
- The Dockerfile `IP`/`EXPOSE` change is behavioral for standalone users; it belongs with the deployment-config remediation and its container E2E, not a docs-only edit.

Verified 2026-11-15: 0 fix attempts — static inspection of README, `context/architecture.md`, `context/sitemap.md`, `Dioxus.toml`, `Cargo.toml`, Dockerfile, compose, and the existing specs. Fenced by doc ownership, diff overlap with the P1 packets, and the absent Rust/Docker build path.

## How to clear it

- [ ] After the P1 backup/update/deploy packets choose their final paths and env names, replace the README's auth-only tree with the real module inventory and link `context/runbook.md`; remove the "no CORS setup" sentence or qualify it with `PERSONA_WEB_ORIGINS`.
- [ ] Replace the two-row env table with a complete table or a single link to `context/architecture.md` §14 plus the runbook; at minimum name `PUBLIC_BASE_URL`, `PERSONA_WEB_ORIGINS`, `ASSETS_*`, `BACKUP_DIR`, `BACKUP_AGE_RECIPIENT`, `JOBS_*`, rate limits, `UPDATE_*`, and `PERSONA_SERVER_ORIGIN`.
- [ ] Document the operational surfaces in one place: MFA, devices, `/dev/db` debug-only, backups/failure injection, retention, sync, public verification, and auto-update. Do not duplicate the runbook; link it.
- [ ] Reconcile the status marks in `context/architecture.md:38,223,251-255,402-409` and `context/sitemap.md:74,95,100` with the code after the P1 fixes; use ✅ only when the linked criterion/evidence exists.
- [ ] Set `Dioxus.toml` title to the product name (for example `Persona Sanus`).
- [ ] Confirm the `Cargo.toml` author spelling with the owner, then correct it.
- [ ] Remove `dioxus-motion` from `Cargo.toml` dependencies and from the `web`, `desktop`, and `server` feature lists; verify `cargo check` for default, `--no-default-features --features desktop`, and `--no-default-features --features web` still pass. Confirm `rg -ni motion src` was already empty.
- [ ] Optionally add `ENV IP=0.0.0.0` and `EXPOSE 8080` to the Dockerfile; confirm a standalone run without compose env reaches `/health`.
- [ ] Re-run `python3 .blocks/persona-smoke.py --dir persona-sanus --port 8091`, `python3 .blocks/persona-docker-verify.py --dir . --port 8080`, and `rg -n 'src/|PUBLIC_BASE_URL|PERSONA_WEB_ORIGINS|ASSETS_|BACKUP_|JOBS_|RATE_LIMIT|UPDATE_|PERSONA_SERVER_ORIGIN' persona-sanus/README.md` to confirm every named path and env still exists; then re-check 13-M objective 5 and delete this file.
