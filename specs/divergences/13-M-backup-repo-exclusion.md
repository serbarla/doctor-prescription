# Divergence tracking: 13-M backup repo exclusion

Source spec: `specs/13-M-ops-and-release.md`, objective 3 (operational backups) and objective 5 (runbook/acceptance). Blocks: the host bind mount introduced by objective 3 (`docker-compose.yml:24`) is inside the tracked workspace and no repository rule excludes it; objective 5 cannot claim a clean PHI-handling story while `git add -A` can stage backups.
Resolving spec: none — requires a backup-remediation spec. 13-M stays in `specs/`.

Severity: P1 — the directory holds the encrypted SQLite database and a raw MinIO mirror of clinical images. A transient plaintext SQLite file from the failure in `13-M-backup-file-safety.md` is additional readable PHI. One accidental `git add .` / `git add -A` stages all of it.

## What's broken

- `docker-compose.yml:24` bind-mounts `./backups` to `/backups`, so backup artifacts are written under the repository root.
- There is no root `.gitignore`. `persona-sanus/.gitignore` ignores `/target`, `*.db`, `/public`, and `.env`, but those patterns are relative to `persona-sanus/`, not the parent checkout. The root `backups/` directory is outside that scope.
- `git check-ignore -v backups` returns nothing; a generated `backups/app-YYYYMMDD.db.age` is an untracked file ready for `git add`.
- The MinIO mirror at `$BACKUP_DIR/minio-YYYYMMDD` is raw object data (images referenced by published templates and retained prescriptions), not an encrypted archive. `age` protects only the SQLite file.
- `context/runbook.md:23` tells operators to use `./backups` as a host directory but does not state that it must stay out of git or that its mode must be private.
- The directory is absent today, so nothing is currently tracked. The first `docker compose up` creates it; the risk starts then.
- `BACKUP_DIR` is runtime-configurable, so a non-default path also needs exclusion/review. The documented default is the path that will be used.

## Why each fix was fenced

- The fix is repository policy outside any one package: adding a root `.gitignore` or moving the backup mount out of the checkout needs owner approval. The 2026-11-15 pass was report-only.
- No backups exist yet, so there is nothing to remove from the index. Running `git rm --cached` would be an unnecessary, destructive-looking action.
- A proper fix also decides the canonical host backup path for the runbook and compose; inventing an absolute path for this workstation would not generalize.

Verified 2026-11-15: 0 fix attempts — `git check-ignore -v backups` empty; `git ls-files` contains no backup files; root tree has no `.gitignore`. Fenced by repo-policy and spec-process constraints, not by unavailable tooling.

## How to clear it

- [x] Add a root `.gitignore` with at least `/backups/` and `*.db.age`; do not broaden it to `*.db` unless fixtures and tests are reviewed. — Added root `.gitignore` with exactly `/backups/` and `*.db.age`; `*.db` untouched.
- [x] In `docker-compose.yml`, make the host path explicit and parameterized, e.g. `${PERSONA_BACKUP_DIR:-./backups}:/backups`, and document in `context/runbook.md` §2 that the path is PHI-handling space: mode `0700`, outside any synced or backed-up-as-code directory, never committed. — Mount is `- ${PERSONA_BACKUP_DIR:-./backups}:/backups` (`docker compose config` renders the default bind); runbook §2 now states the PHI rules, mode `0700`, never committed, and the same rules for an explicit `PERSONA_BACKUP_DIR` (incl. `git check-ignore -v "$PERSONA_BACKUP_DIR"`).
- [x] If backups already exist under the checkout, move them to a private host directory or leave them ignored; do not delete them without the operator's confirmation. — None existed (`git ls-files backups` empty, directory absent pre-run); the compose run created an empty `./backups`, left in place, ignored.
- [x] Verify: `git check-ignore -v backups/app-test.db.age` → matches `/.gitignore:/backups/`; `git status --short` shows no backup files after a compose run; `git ls-files backups` is empty. — `git check-ignore -v` → `.gitignore:1:/backups/	backups/app-test.db.age`; after the compose gate run `git status --short` shows only the three owned edits, no backup files; `git ls-files backups` empty.
- [x] Run `python3 .blocks/persona-docker-verify.py --dir . --port 8080` to confirm the compose change did not break boot, health, or auth persistence. — Ran 2026-11-17 on `spec/13-M-backup-repo-exclusion`: exit 0, `{"ok": true}`, all 28 checks true (deployed, health, auth persistence across recreate, asset probes).
- [ ] Re-check 13-M objective 3 and objective 5 evidence, then delete this file. — Re-checked 2026-11-17: 13-M objectives 3 and 5 stay `[x]`; the compose/runbook change preserves objective 3's bind mount and objective 5's runbook coverage, and the gate command that verifies both passed post-change. File deletion not performed: out of this slice's owned-file scope (`specs/`), left to the coordinator/integration owner.
