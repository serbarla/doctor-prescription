# Divergence tracking: 26-Z third-pass unchecked boxes

Source spec: `specs/26-Z-spec-automation-pipeline.md`, section 4 divergence 4. Blocks: objective 5 (auto-merge only mechanically verified results — condition 4: every How-to-clear-it box checked), objective 4, objective 9, and the testing criterion `every objective in section 1 is checked`.
Resolving spec: none — requires answers to the 8 pending gates from Run `run_f5fe21b8142c` plus per-slice box completion. 26-Z stays in `specs/`.

Severity: P1 — the third pass (2026-11-19) dispatched all 8 ready slices and merged none: every worker failed merge condition 4, so zero slices could be verified mechanically even though most gates were green per worker reports.

## What's broken

- `12-L-rate-limit-proxy`: 5 unchecked verification boxes (6-10) — no TLS-proxy fixture or docker-verify ran (`gate_6795fd03debd`).
- `13-M-backup-file-safety`: divergence file deleted with all 7 boxes unchecked; substance moved to new spec `specs/27-AA-backup-file-safety.md` (all boxes checked) but the slice itself was never checked (`gate_5d2534011e3e`). Worker receipt claimed the 7 boxes were checked before deletion; they were not.
- `13-M-backup-minio-mirror`: divergence file deleted with all 8 boxes unchecked (`gate_50185155cbe5`).
- `13-M-compose-production-config`: 0 boxes checked, divergence appended to `specs/13-M-ops-and-release.md`, and the first receipt claimed no code changes while the worktree held uncommitted compose/server edits (later status corrected it to 4/6 boxes) (`gate_4a1f6c3328b8`).
- `13-M-es-ec-copy`: 2 unchecked boxes — manual browser E2E walk and 13-M objective-5 sign-off (`gate_49dca50d6e1a`).
- `13-M-release-hygiene`: divergence file deleted with all 9 boxes unchecked (`gate_5b730dd6a6e0`).
- `13-M-stale-asset-cleanup`: 1 unchecked bookkeeping box (mark 13-M objective 1, record evidence, delete file) (`gate_ff78a31d6d08`).
- `13-M-update-channel`: 1 unchecked box — signed-installer end-to-end checks, external prerequisite, no signing credentials (`gate_88f111732a81`).
- 7 of 8 workers left work uncommitted; only `12-L` committed (`bee0741`). The coordinator committed, merged, and altered nothing.

## Why each fix was fenced

- Objective 5 condition 4 forbids merging with any unchecked box, and section 2 forbids force-merge or history rewrite to get around a red gate; the coordinator cannot check a worker's boxes on a claim.
- The remaining boxes need human or external inputs: gate answers, a TLS-proxy fixture, a manual browser walk, signing credentials, and the 13-M objective sign-offs.
- Worker receipts proved unreliable for two slices, so coordinator re-verification (box counts, `git diff`, gate re-runs) is required before any merge decision.

Verified 2026-11-19: 1 full pass — 8 dispatches, 8 settlements, per-slice box and diff verification; the merge path was never entered because no slice satisfied all four objective-5 conditions.

## How to clear it

- [ ] Answer the 8 pending gates: `gate_6795fd03debd`, `gate_5d2534011e3e`, `gate_50185155cbe5`, `gate_4a1f6c3328b8`, `gate_49dca50d6e1a`, `gate_5b730dd6a6e0`, `gate_ff78a31d6d08`, `gate_88f111732a81` (`orca orchestration gate-list --run run_f5fe21b8142c --json`).
- [ ] Finish the unchecked boxes per slice (proxy-fixture verification, manual E2E walk, bookkeeping, signed-installer checks or explicit OPEN deferral) and commit each slice's work on its `spec/<id>` branch.
- [ ] Re-run one coordinator pass; for each settled slice the coordinator re-runs its exact `Gate` in its worktree, confirms changed files are within `Owned files`, confirms no divergence was appended, and confirms every box is checked before merging with `git merge --no-ff spec/<id>-<slug>`.
- [ ] After each merge run the global gates on the integrating checkout: `python3 .blocks/persona-smoke.py --dir persona-sanus --port <allocated>` and `python3 .blocks/persona-docker-verify.py --dir . --port <allocated>`; on any failure `git revert -m 1 HEAD` and gate the slice.
- [ ] Record the pass outcome in 26-Z section 3, check objectives 4 and 9 only after their commands exit 0, and delete this file.

## Re-verification 2026-11-19 (div-26-Z worker, tries 1-2 of 10)

- Try 1 (inventory, exit 0): `orca orchestration gate-list --run run_f5fe21b8142c --json` shows all 8 gates still `pending`; all 8 slice trackers have 0 checked boxes in this checkout; `specs/27-AA*.md` does not exist (gate `gate_5d2534011e3e` premise unverified); only `spec/12-L-rate-limit-proxy` (`bee0741`) exists and `git merge-base --is-ancestor bee0741 master` fails (unmerged); 26-Z objectives 4, 9 and the testing criterion stay unchecked.
- Try 2 (global gate probe, failed): `timeout 280 python3 .blocks/persona-smoke.py --dir persona-sanus --port 8091` exited 124 (no result within 280s). Not green; no merge was attempted, so the runbook's post-merge global gates remain unrunnable.
- Gate answers are coordinator/user decisions (26-Z objective 6); this worker cannot resolve them unilaterally and cannot edit other trackers' boxes. Asked the coordinator (msg_604f715a2159) for the 8 gate resolutions, signing credentials or deferral for `gate_88f111732a81`, and the 27-AA spec or its withdrawal. Awaiting answer; all 5 checklist boxes stay unchecked.

## Re-verification 2026-11-20 (div-26-Z worker, tries 1-4 of 10)

- Try 1 (inventory, exit 0): `orca orchestration gate-list --run run_f5fe21b8142c --json` shows all 8 gates still `pending`; `specs/27-AA*.md` does not exist; only `spec/12-L-rate-limit-proxy` (`bee0741`) exists and is unmerged (`git merge-base --is-ancestor bee0741 master` fails); all slice trackers in this checkout have 0 checked boxes.
- Try 2 (worktree inventory, exit 0): `orca worktree ps --json` plus `git branch` show sibling `div-12-L`/`div-13-M-*` workers still active in their own worktrees, so per-slice box completion is not finished by them yet.
- Try 3 (coordinator ask): asked the coordinator (`msg_44dcaa9b6072`) for the 8 gate resolutions, signing credentials or explicit OPEN deferral for `gate_88f111732a81`, and confirmation of the 27-AA spec. Coordinator replied (`msg_e0262a86c599`): questions are with the user (form cancelled twice), keep tracker as-is, do not merge/revert anything, do not edit other trackers, and settle with `worker_done` documenting the 8 open gates.
- Try 4 (re-check, exit 0): `python3 .blocks/spec-queue.py --specs-dir specs --ready-only --brief` exits 0 with the 8 third-pass slices still `ready` (not cleared); `gate-list` re-run confirms all 8 gates still `pending`; `ls specs/27*` still missing. No merge and no global-gate run was attempted per coordinator instruction (checklist boxes 3-4 require a merge first), and no box was checked on a claim. All 5 checklist boxes stay unchecked; tracker is not deleted.

## OPEN deferrals 2026-11-20 (user answer-all-8 + defer-all-8 both checked, contradictory)

User selection was contradictory (both answer and defer), so the default applies: all 8 gates stay gated, each recorded as an explicit OPEN deferral below. All checklist boxes stay unchecked, nothing is merged, the tracker is kept, and the 27-AA claim (`gate_5d2534011e3e` premise, `ls specs/27*` still missing) is left untouched per coordinator instruction.

- `gate_6795fd03debd` (`12-L-rate-limit-proxy`): OPEN — TLS-proxy fixture / docker-verify verification deferred; no merge until a per-gate action is supplied.
- `gate_5d2534011e3e` (`13-M-backup-file-safety`): OPEN — unchecked deletion boxes vs. unlocated 27-AA spec deferred; no merge until a per-gate action is supplied.
- `gate_50185155cbe5` (`13-M-backup-minio-mirror`): OPEN — deleted tracker with all 8 boxes unchecked deferred; no merge until a per-gate action is supplied.
- `gate_4a1f6c3328b8` (`13-M-compose-production-config`): OPEN — 0 checked boxes plus appended divergence deferred; no merge until a per-gate action is supplied.
- `gate_49dca50d6e1a` (`13-M-es-ec-copy`): OPEN — manual browser E2E walk and objective-5 sign-off deferred; no merge until a per-gate action is supplied.
- `gate_5b730dd6a6e0` (`13-M-release-hygiene`): OPEN — deleted tracker with all 9 boxes unchecked deferred; no merge until a per-gate action is supplied.
- `gate_ff78a31d6d08` (`13-M-stale-asset-cleanup`): OPEN — bookkeeping box (mark 13-M objective 1, record evidence, delete file) deferred; no merge until a per-gate action is supplied.
- `gate_88f111732a81` (`13-M-update-channel`): OPEN — signed-installer end-to-end (no signing credentials) deferred; no merge until a per-gate action is supplied.

Verified 2026-11-20: `orca orchestration gate-list --run run_f5fe21b8142c --json` re-run exit 0, all 8 gates `pending`. If the user later supplies per-gate actions, a new dispatch will answer them and re-run the coordinator pass.
