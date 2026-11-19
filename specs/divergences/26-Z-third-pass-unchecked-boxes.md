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
