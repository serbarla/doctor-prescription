# Divergence tracking: 26-Z pilot dispatch fences

Source spec: `specs/26-Z-spec-automation-pipeline.md`, section 4 divergence 1 (and the remaining fence of divergence 2). Blocks: objective 4 (dispatch one worktree per spec or divergence, all file-disjoint ready slices in one pass), objective 9 (pilot pass end to end), and the testing criterion `every objective in section 1 is checked`.
Resolving spec: none — requires a human-authored `Owned files` split for the 8 ready slices and a user-confirmed per-day cost ceiling recorded in `AGENTS.md`. 26-Z stays in `specs/`.

Severity: P1 — DAG-width dispatch cannot run as written: the 8 ready slices are template-era `whole_spec` rows (`Owned files: *`), four touch `persona-sanus/src/jobs.rs`, two touch `docker-compose.yml`, and the rest overlap `context/runbook.md`, `persona-sanus/README.md`, `persona-sanus/Dockerfile`, and `persona-sanus/src/auth_server.rs`. Dispatching them together breaks the section 2 No-go on silently reassigning shared files.

## What's broken

- The scanner reports 8 `ready` slices but none are file-disjoint: every row carries `owned_files: ["*"]` and `whole_spec: true`, so overlap demotion never fires and wide dispatch would be a coin flip over shared files.
- The `$5/day` ceiling set under objective 1 covers one worker, not eight; the runbook serializes `whole_spec` rows one alone per pass.
- Divergence 2's dirty-checkout fence cleared on 2026-11-19 (coordinator checkout clean, base branch `serbarla/merge-screens-specs` resolved at runtime, third pass dispatched all eight), leaving only the split and ceiling fences from divergence 1.

## Why each fix was fenced

- A file split is an ownership decision the coordinator cannot invent: reassigning `jobs.rs`, `docker-compose.yml`, or the runbook to one of two overlapping slices violates the section 2 No-go, so the split must be human-authored per slice.
- The cost ceiling is a user-set number; objective 1 forbids inventing it, and the hourly automation stays disabled until the user reviews it.
- Divergence 1's 2026-11-17 verification already recorded the same fence; the third pass confirmed it again by gating all eight slices instead of merging overlapping work.

Verified 2026-11-19: 1 full dispatch pass (8 worktrees, 8 workers) plus box-by-box verification — dispatch works mechanically, merging is fenced by the missing split and ceiling, not by tooling.

## How to clear it

- [ ] Author `Owned files` for each of the 8 ready slices (split `persona-sanus/src/jobs.rs`, `docker-compose.yml`, `context/runbook.md`, `persona-sanus/README.md`, `persona-sanus/Dockerfile`, `persona-sanus/src/auth_server.rs` between slices) and add the header fields to each slice file, or to new remediation specs.
- [ ] Confirm or raise the per-day cost ceiling in `AGENTS.md` § Subagent Delegation Fleet, then enable the automation only after that review.
- [ ] Re-run the scan and confirm the split: `python3 .blocks/spec-queue.py --specs-dir specs` shows no `conflict` rows among the formerly overlapping slices.
- [ ] Run one manual coordinator pass: `orca automations run --id 75f49b64-df62-4a76-a714-a651c4cd7e31` (or the manual pass), then `orca worktree ps --json` shows one worktree per dispatched slice on branch `spec/<id>-<slug>`.
- [ ] Record the pass outcome in 26-Z section 3, check objectives 4 and 9 only after their commands exit 0, and delete this file.

Fourth pass 2026-11-20 (try 2/10, unfenced prerequisites only): `python3 .blocks/spec-queue.py --specs-dir specs` exits 0 with 15 `ready`, 0 `conflict`, 14 of 15 with `owned_files: ["*"]` and `whole_spec: true` (only `25-Y-screens-index-refs` and `26-Z-repo-root-path` declare explicit files), so zero `conflict` rows do not prove disjointness. Automation `75f49b64-df62-4a76-a714-a651c4cd7e31` is `enabled: false` and `AGENTS.md` carries no numeric ceiling, only the disabled-until-reviewed rule. Receipt check: commit `bee0741` exists on unmerged `spec/12-L-rate-limit-proxy` and no `specs/27-AA*` file exists; neither supplies the split or the ceiling. Items 1–2 await the human-authored split and the user-set ceiling; items 3–5 stay unchecked until those land. Coordinator relaying both questions to the user on 2026-11-20.
Fifth pass 2026-11-20 (try 3/10, unfenced prerequisites only): `python3 .blocks/spec-queue.py --specs-dir specs` exits 0 with 15 `ready`, 0 `conflict`, 3 `blocked`; 13 of 15 ready rows still carry `owned_files: ["*"]` and `whole_spec: true` (only `25-Y-screens-index-refs` and `26-Z-repo-root-path` declare explicit files), so zero `conflict` rows do not prove disjointness and item 3 stays unchecked. Automation `75f49b64-df62-4a76-a714-a651c4cd7e31` is `enabled: false` and `AGENTS.md` carries no numeric per-day ceiling, only the disabled-until-reviewed rule. Receipt check: commit `bee0741` remains on unmerged `spec/12-L-rate-limit-proxy` and no `specs/27-AA*` file exists; neither supplies the split or the ceiling. Item 4 manual pass was not run: dispatching 13 wildcard slices in one pass would breach the ceiling fence and the 26-Z No-go on silently reassigning shared files. Items 1–2 remain fenced on the human-authored `Owned files` split and the user-confirmed ceiling; items 3–5 stay unchecked until those land.
