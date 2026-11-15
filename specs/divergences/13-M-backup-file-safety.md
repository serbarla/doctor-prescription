# Divergence tracking: 13-M backup file safety

Source spec: `specs/13-M-ops-and-release.md`, objective 3 (operational backups). Blocks: objective 3 — "an encrypted daily backup of the SQLite file via `VACUUM INTO` … encrypted with a server-held key (`age` or equivalent, custody documented in the runbook)"; the backup/restore evidence claimed by objective 5; and `context/runbook.md:24` ("no plaintext is left on disk").
Resolving spec: none — requires a backup-remediation spec. 13-M stays in `specs/`.

Severity: P1 — a failed or interrupted encryption can leave the complete SQLite database, with clinical data, readable on the host. A leftover plaintext file also blocks later `VACUUM INTO` runs at the same path.

## What's broken

`persona-sanus/src/jobs.rs:220-241`:

- `VACUUM INTO` writes a plaintext SQLite file at `$BACKUP_DIR/app-YYYYMMDD.db` (line 231) before `age` starts (lines 234-237).
- `age` spawn errors use `?` (line 238), which returns before `remove_file` (line 239). A missing `age`, broken `PATH`, or execute-permission failure leaves the plaintext DB on disk.
- A `SIGKILL`, crash, container stop, or host reboot between lines 231 and 239 also leaves the plaintext DB.
- `remove_file` is ignored (line 239). If removal fails, the job still reports success and the plaintext remains.
- `backup_paths` uses a date-only target (lines 208-217), so every run or restart on the same day targets `app-YYYYMMDD.db.age`. `age` documents that an existing `-o` output is overwritten, and the write is direct, not temp-then-rename. A crash during the second run can truncate the only good encrypted copy.
- If a plaintext file is left, the next run's `VACUUM INTO` fails with SQLite's `output file already exists`; the job audits an error and never encrypts that database. Recovery is manual.
- No explicit permissions are set. `create_dir_all` uses the default umask; under 022 the backup directory is `0755`, and the SQLite file created by `VACUUM INTO` is `0644`. Local users can read the plaintext during the window.
- `context/runbook.md:24` promises "no plaintext is left on disk" on an unset recipient or `age` failure. The spawn-error case contradicts it; the unset-recipient path returns before `VACUUM INTO`.

## Why each fix was fenced

- The 2026-11-15 pass was report-only and made no code changes; `AGENTS.md` requires a numbered spec before implementation, and no backup-remediation spec exists.
- The fix needs an explicit backup retention decision: unique snapshot per run versus one overwritten per day, how many successful copies to keep, and whether to sweep plaintext files at startup. None is specified in 13-M.
- Failure injection needs a container rebuild and a way to run the backup with `age` hidden or killed. `cargo` and `dx` are not on this shell's PATH, and no build or container run was part of the discovery pass.

Verified 2026-11-15: 0 fix attempts — code inspection plus a throwaway `sqlite3` reproduction (`VACUUM INTO` twice to the same path → `output file already exists`, exit 1; created file mode `0644` under a 022 umask); repair is fenced by the spec process, not by a missing credential.

## How to clear it

- [ ] Create the backup directory with mode `0700` (or tighten it with `set_permissions` when it already exists); create each plaintext and encrypted temp file with mode `0600`.
- [ ] Write to a unique temp path per run, e.g. `$BACKUP_DIR/.app-YYYYMMDD-HHMMSS-<pid>.db.tmp`, then `age -o <encrypted>.tmp`, `sync`, and atomically rename to the final `.db.age`. Never truncate a previous successful `.age` before the replacement is complete.
- [ ] Make cleanup guard-based so spawn failure, encryption failure, and normal completion all remove the plaintext temp file; keep a startup sweep of stale `*.db`/`*.db.tmp` paths behind the single-process assumption.
- [ ] If a plaintext temp file survives a hard kill, fail the next backup with an actionable error rather than silently continuing, and remove or zero it during startup recovery.
- [ ] Correct `context/runbook.md:24` to state the actual guarantee: plaintext is removed on success and on graceful failure, but a process kill between snapshot and encryption can leave the file until restart; describe directory permissions and the recovery step.
- [ ] Verify: set a valid `BACKUP_AGE_RECIPIENT` and run a backup with `age` absent from `PATH`; no plaintext may remain after recovery. Run two backups on the same day; both encrypted files must decrypt to valid SQLite, and the first must remain untouched until the second rename. `ls -l backups` must show `drwx------`/`-rw-------`. `python3 .blocks/persona-docker-verify.py --dir . --port 8080` must remain green.
- [ ] Re-check 13-M objective 3 evidence, record the failure-path proof next to it, and delete this file only after that box is verified again.
