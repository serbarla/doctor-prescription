# Divergence tracking: 13-M / 12-L tracker evidence stale after UI specs and third pass

Source specs: `specs/divergences/13-M-release-hygiene.md`, `13-M-es-ec-copy.md`, `12-L-rate-limit-proxy.md`, `13-M-backup-file-safety.md`, `26-Z-third-pass-unchecked-boxes.md`; specs 15-O–25-Y (`specs/done/`) and the 2026-11-19 third pass changed the code and docs those trackers cite.
Resolving spec: none — requires a tracker-refresh pass before any remediation dispatch. Parent specs stay where they are.

Severity: P2 — dispatching workers from these trackers as written sends them to wrong files/lines and already-fixed claims, and the queue scanner still counts every tracker as fully open (78 unchecked boxes across 11 files, verified 2026-11-19).

## What's broken

- `13-M-release-hygiene.md` claims `context/architecture.md` marks audit/MFA/backups and `/audit`, `/verify/:code` as planned and `context/sitemap.md:74,95,100` marks them planned. Today `architecture.md:38` reads ✅ for audit/MFA/rate-limiting/backups, and `sitemap.md:74` (`/verify/:code`) and `:101` (`/audit`) read ✅. That portion of the tracker is already fixed; its remaining items (README env table, `Segio Barba` at `persona-sanus/Cargo.toml:4` — confirmed still present, `title = "persona-sanus"` at `Dioxus.toml:6`, dead `dioxus-motion` at `Cargo.toml:9,39,42-43,59-60`) are still live but mixed with cleared ones, all 9 boxes unchecked.
- `13-M-es-ec-copy.md` cites pre-refactor line numbers in a monolithic `main.rs` (e.g. Home/Navbar/Auth at `1287-1754`). Specs 15-O–25-Y split UI into `persona-sanus/src/ui/*` and translated the shell: sampled literals `"Sign in"`, `"Dashboard"`, `"Welcome back"`, `"Fill in every field"` return 0 hits in `persona-sanus/src/` on 2026-11-19; nav labels now read `Panel`, `Pacientes`, `Plantillas`, `Configuración` (`main.rs:1509-1531`). The tracker's 92-literal inventory and per-line fix list no longer map to the code; a fresh scan is required to know what, if anything, remains.
- `26-Z-third-pass-unchecked-boxes.md` and `specs/26-Z-spec-automation-pipeline.md:111` state the substance of backup-file-safety "moved to new spec `specs/27-AA-backup-file-safety.md` (all boxes checked)". No `27-*` file exists in `specs/` or `specs/done/` on this checkout (verified 2026-11-19); the claim points at a worktree-only artifact that was never merged.
- The same third-pass record says three workers "deleted their divergence files" (`backup-file-safety`, `backup-minio-mirror`, `release-hygiene`). On `master` all 11 tracker files are still present with all boxes unchecked (7, 8, 9 respectively) — the deletions, like 7 of 8 workers' work, were never committed/merged. Tracker state on `master` and worker-receipt state describe different realities.
- `12-L-rate-limit-proxy.md` describes the unfixed limiter on `master` (accurate here), but a fix already exists as commit `bee0741` on unmerged branch `spec/12-L-rate-limit-proxy`. The tracker does not mention it, so a fresh dispatch would redo finished work instead of reviewing/merging that branch.

## Why each fix was fenced

- Refreshing a tracker means re-verifying its claims against current code (fresh es-EC literal scan, README/architecture re-read, branch review of `bee0741`), which is remediation work needing its own spec pass, not an audit edit.
- Merging or discarding the third-pass worktree work is fenced by 26-Z objective 5 (all four merge conditions) and its 8 pending gates (`gate_6795fd03debd` … `gate_88f111732a81`).

Verified 2026-11-19: 1 fix attempt — re-scan (rg exit 1), arch/sitemap grep, `git branch`/`git log --all --diff-filter=A`, `bee0741` review (`client_ip.rs` right-to-left trusted-CIDR + 10k bounded map matches tracker items 1–5), `gate-list` (all 8 pending). Fenced from here by the pending 26-Z gates, not by tooling.

## How to clear it

- [x] Re-scan es-EC copy against current `persona-sanus/src/` (including `src/ui/`); rewrite `13-M-es-ec-copy.md` with current files/lines or delete it if the scan is clean, attaching the scan command. (Done 2026-11-19: rewritten — shell translated, service/auth/http/api errors still English with current lines; scan `rg -n '"Sign in"|"Dashboard"|"Welcome back"|"Fill in every field"' persona-sanus/src/` exit 1.)
- [x] Strike the already-fixed architecture/sitemap items from `13-M-release-hygiene.md`; keep only verified-live items (README, `Segio Barba` pending owner confirmation, Dioxus title, `dioxus-motion`). (Done 2026-11-19: struck; `architecture.md:38` ✅, `sitemap.md:74/:101` ✅; README/Dioxus/`Segio Barba`/motion re-verified live and kept unchecked.)
- [x] Locate the `27-AA-backup-file-safety` work: recover it from the gated worktree into a real `specs/27-AA-*.md`, or correct both 26-Z records to say it does not exist on any merged branch. (Done 2026-11-19: no `27-AA*` on any branch — `git log --all --diff-filter=A -- 'specs/27-AA*'` empty, `git branch -a` has no `27*`; both 26-Z records corrected.)
- [ ] Review commit `bee0741` (`spec/12-L-rate-limit-proxy`) as the 12-L remediation candidate instead of redispatching from scratch; merge or reject it through the 26-Z gate, then update the tracker. (Review done 2026-11-19: static review matches items 1–5; `cargo check` scratch-build + merge/reject FENCED by pending `gate_6795fd03debd` — leave unchecked.)
- [ ] After the 8 third-pass gates are answered, reconcile each tracker file's existence/boxes with `master` (merged = delete, rejected = restore accurate unchecked state). (FENCED 2026-11-19: all 8 gates pending per `gate-list --run run_f5fe21b8142c`.)
- [ ] Re-run `python3 .blocks/spec-queue.py --specs-dir specs` and confirm counts match the refreshed trackers; delete this file then. (Scanner ran 2026-11-19 exit 0 — 18 total, 15 ready, 3 blocked; counts match refreshed trackers (es-ec 7, release-hygiene 8). Deletion blocked on the two boxes above.)

## Refresh note 2026-11-19 (1 try used of 10)

- Items 1–3 are done and verified above. Item 4 is reviewed but unmerged: merging `bee0741` requires `gate_6795fd03debd` and 26-Z objective 5's four conditions; the `12-L` gate explicitly asks whether to run proxy-fixture verification or keep gated. Item 5 is fenced by all 8 pending gates (`gate_6795fd03debd` … `gate_88f111732a81`). This file therefore stays until the gates answer; no box was checked on a claim and no tracker was deleted while boxes remain unchecked.

## Refresh note 2026-11-20 (3 tries used of 10)

- Try 1 re-verified items 1-3 in this checkout: `rg -n '"Sign in"|"Dashboard"|"Welcome back"|"Fill in every field"' persona-sanus/src/` → exit 1; `13-M-es-ec-copy.md` carries the rewritten inventory with the shell box `[x]`; `architecture.md:38` ✅ and `sitemap.md:74/:101` ✅ with the corresponding `13-M-release-hygiene.md` box `[x]` and live items kept unchecked; `ls specs/27* specs/done/27*` → no matches and `git log --all --diff-filter=A -- 'specs/27-AA*'` empty, with both 26-Z records carrying the correction. Boxes below reflect this.
- Try 2 confirmed the fence still holds: `orca orchestration gate-list --run run_f5fe21b8142c --json` → exit 0, all 8 gates `pending` (`gate_6795fd03debd` … `gate_88f111732a81`); `git merge-base --is-ancestor bee0741 master` → NOT merged (branch `spec/12-L-rate-limit-proxy` only, no `client_ip.rs` in this checkout). Item 5 therefore stays fenced.
- Try 3 re-reviewed `bee0741` statically (`git show bee0741` exit 0): `client_ip.rs` default-ignores XFF with `PERSONA_TRUSTED_PROXY_CIDRS` CIDR opt-in, right-to-left untrusted parse, malformed → peer, `MAX_WINDOWS = 10_000` with pruning and oldest eviction, wired into auth and verify via `admit()`; `architecture.md`/`runbook.md` document the topology — matching `12-L` items 1-5. Cargo check / proxy-fixture verification and merge-reject are part of the gated `gate_6795fd03debd` decision, so item 4 stays unchecked.
- Try 2 also ran `python3 .blocks/spec-queue.py --specs-dir specs` → exit 0: 18 total, 15 ready, 3 blocked; `13-M-es-ec-copy` 7 unchecked, `13-M-release-hygiene` 8 unchecked, `13-M-stale-tracker-evidence` 3 unchecked — matching the refreshed trackers. Deletion stays blocked on items 4-5; this file stays.
