# Spec 26 — Phase Z: Spec automation pipeline

- Source: user decision 2026-11-16 — automate the current one-agent-per-spec workflow. Locked choices: auto-merge on green gate, parallelism as wide as the dependency DAG allows, scheduled Orca automation as trigger, Computer Use for desktop visual smoke only, one worktree per spec or divergence file.
- Depends on: 13-M (active spec whose `specs/divergences/` slices are the pilot queue input).
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Result carried forward: a scheduled coordinator turns unchecked specs and divergence files into worktree-isolated worker runs, merges only mechanically-verified results in dependency order, and escalates everything else to a decision gate — no human launches, verifies, or merges individual specs by hand.

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

- [x] 1. Amend the worker-pool and spec-template contracts — requirements: `AGENTS.md` (§ Subagent Delegation) gains an Orca-worker row stating DAG-width parallelism supersedes the single-subagent caps for Orca orchestration workers only, with a cap of max 5 active agents before objective 6 is enabled (record it in `AGENTS.md`); `specs/TEMPLATE.md` header gains three fields after `Repo root`: `Owned files: <comma-separated paths/globs the executing worker may modify>`, `Gate: <exact .blocks or cargo command that must exit 0>`, `Desktop-verify: <yes|no>`; specs written before this change and lacking the fields default to whole-spec ownership, their global smoke as gate, and `Desktop-verify: no`; commands: `python3 .blocks/markdown-table-check.py --path AGENTS.md` and `python3 .blocks/markdown-table-check.py --path specs/TEMPLATE.md`
- [x] 2. Build the queue scanner block — requirements: new `.blocks/spec-queue.py` following the block rules in `.blocks/INDEX.md` (header first, params only, one small JSON object, exit 0 on success and non-zero with `{"error": ...}` on failure, idempotent, registered in `AGENTS.md` and `.blocks/INDEX.md`); it scans `specs/*.md` (excluding `TEMPLATE.md`, `done/`) plus `specs/divergences/*.md`, parses for each file its unchecked §1 objectives, `Depends on`, `Owned files`, `Gate`, and `Desktop-verify`, and emits `{queue: [{id, path, kind: spec|divergence, ready|blocked|conflict, blocked_by, owned_files, gate, desktop_verify}], counts}`; `ready` = no unchecked dependency and no `Owned files` overlap with another ready file, overlap demotes both to `conflict` (never picks a winner silently), divergence files are blocked while their source spec lists them as OPEN with an external prerequisite (e.g. signing credentials); `--fail-if-empty` exits non-zero when no file is `ready` (used as the automation precheck); commands: `python3 .blocks/spec-queue.py --specs-dir specs` → JSON listing `13-M-ops-and-release` and all files in `specs/divergences/`, and `python3 .blocks/spec-queue.py --specs-dir specs --ready-only --fail-if-empty; echo $?` → `0` only if at least one slice is dispatchable
- [x] 3. Write the coordinator runbook and prompt — requirements: new `context/spec-automation.md` documenting the loop end to end and containing the verbatim coordinator prompt: read this spec, `AGENTS.md`, and `context/spec-automation.md`; run one pass only (scan → dispatch ready set → collect settled workers → merge or gate → stop); a worker prompt per slice states the spec path, owned-files confinement, its exact `Gate` command, "check spec boxes only after the gate exits 0", "on any mismatch STOP, append a divergence to the spec, check nothing, and report", and the AGENTS.md rule of 15 fix attempts then a Linear issue in Specs-Issue format; no credentials, secrets, ports, or dates hardcoded — ports derive as `8091 + <worktree index>` and each worker gets a unique `--db-path` under its worktree; commands: `python3 .blocks/markdown-table-check.py --path context/spec-automation.md`
- [ ] 4. Dispatch one worktree per spec or divergence — requirements: coordinator pass runs `orca orchestration run-create --objective "spec automation pass" --json`, then per ready file `orca worktree create --name spec-<id>-<slug> --branch spec/<id>-<slug> --base-branch <branch resolved at runtime from the coordinator checkout> --json` (never a hardcoded base branch), records the returned worktree path in the orchestration task, and starts exactly one worker with `orca orchestration worker-start --spec-file <spec path> --worktree path:<worktree path> --agent <agent id> --json`; tasks mirror the scanner DAG via `orca orchestration task-create --spec-file <spec path> --deps <json_array of task ids> --json`; file-disjoint ready slices all start in the same pass, `conflict`/`blocked` slices are not dispatched and stay queued; commands: `orca worktree ps --json` shows one worktree per dispatched slice on branch `spec/<id>-<slug>`
- [x] 5. Auto-merge only mechanically verified results — requirements: a settled worker is mergeable only when all four hold: its `Gate` command re-run by the coordinator in that worktree exits 0, `git -C <worktree> diff --name-only <base>...HEAD` is a subset of the slice's `Owned files` (template-era files without the field: subset check skipped, slice merges alone), no divergence was appended to the spec during the run, and every §1 box of the slice is checked; merge in topological (dependency) order with `git merge --no-ff spec/<id>-<slug>`, then run the parent spec's global gates on the integrating checkout (`python3 .blocks/persona-smoke.py --dir persona-sanus --port <allocated>` and `python3 .blocks/persona-docker-verify.py --dir . --port <allocated>`); on any global-gate failure `git revert -m 1 HEAD` the merge, reopen the slice as `blocked`, and gate it (objective 6) — a red main is never left standing; on success move a fully-checked spec to `specs/done/` (divergence files are deleted per their own "How to clear it" section) and remove the worktree with `orca worktree rm --worktree path:<worktree path> --delete-branch --json`, never removing a worktree with unmerged or unverified work; commands: `git log --oneline -5` shows the merge and `python3 .blocks/persona-smoke.py --dir persona-sanus --port 8091` passes on the integrating checkout
- [x] 6. Escalate through decision gates, never around them — requirements: any non-mergeable settled worker, any divergence appended by a worker, any external prerequisite (signing certificates, counsel sign-off, credentials), and any 15-attempt failure (which also files the Linear Specs-Issue per `AGENTS.md`) blocks its task with `orca orchestration gate-create --task <task_id> --question <text> --options <json_array> --json` and the coordinator pass continues with independent slices; gated slices keep their worktree and branch untouched for inspection; commands: `orca orchestration gate-list --json` shows the gate and `orca worktree ps --json` still lists the gated slice's worktree
- [x] 7. Restrict Computer Use to desktop visual smoke — requirements: only slices with `Desktop-verify: yes` or whose `Owned files` include the desktop feature surface (`persona-sanus/src/update.rs`, desktop installer config) get a GUI pass, executed after objective 5's gate re-run and before merge: launch the desktop build from the worker's worktree, inspect it with `orca computer list-windows --json` and `orca computer get-app-state --json` (resolve exact flags via `orca computer get-app-state --help` first — do not guess flags), and record the observed state as text evidence in the spec's §3; a GUI observation alone never merges anything — the slice's `Gate` must still exit 0, and if the GUI state contradicts the gate the slice is gated (objective 6), not merged; Computer Use is never used for editing code, git, tests, terminal work, or Linear; commands: `orca computer capabilities --json` succeeds and the spec §3 entry names the window/state observed
- [x] 8. Enable the scheduled automation — requirements: resolve the provider/agent id from the local Orca install at execution time (do not hardcode an unverified id), then create a disabled-by-default automation `orca automations create --name "Spec queue coordinator" --trigger hourly --repo path:/Users/sergiobarba/projects/doctor-prescriptions --precheck "python3 .blocks/spec-queue.py --specs-dir specs --ready-only --fail-if-empty" --prompt "Read specs/26-Z-spec-automation-pipeline.md and context/spec-automation.md, then execute exactly one coordinator pass." --provider <resolved agent id> --json`; the user reviews the created automation and the `AGENTS.md` active-agent cap from objective 1 before it is enabled (`orca automations edit --id <id>` / enable path from `orca automations edit --help`); manual triggering stays available via `orca automations run --id <id>`; commands: `orca automations list --json` shows the automation with its precheck and hourly trigger
- [ ] 9. Run the pilot pass end to end — requirements: with the automation still disabled, run one manual coordinator pass over the current queue (`13-M` + its divergence files): the scanner classifies, every dispatchable slice gets its own worktree and worker, credential-blocked slices (signed installers) gate without any fake signing attempt, and each settled slice ends exactly one of merged-with-green-global-gates, gated with its worktree preserved, or re-queued as `conflict`; record the pass outcome (dispatched, merged, gated, re-queued counts plus worktree/branch names) in §3 evidence; commands: `orca automations run --id <id>` (or the manual pass) followed by `orca worktree ps --json` and `python3 .blocks/spec-queue.py --specs-dir specs` showing the post-pass queue state

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No auto-merge when any of objective 5's four conditions is unmet, and no force-merge, force-push, or history rewrite to get around a red gate.
- No dispatching `conflict` or `blocked` slices, and no silently reassigning shared files (e.g. `persona-sanus/src/main.rs`, migrations) to one of two overlapping slices — overlap means both wait for a human-authored split.
- No Computer Use for editing, git, tests, builds, or Linear; no GUI-only merge evidence.
- No faking external prerequisites: signing/notarization credentials, counsel sign-off, or store accounts stay OPEN gates (13-M objective 4 rule applies unchanged).
- No new daemon, service, database, or dependency: the pipeline is one `.blocks` scanner, Orca automations/orchestration, and git.
- No touching `specs/done/`, `context/PRD.md` scope, or other specs' objectives; this spec changes process, not product behaviour.
- No unit tests; no comments in any code.
- No rich-text, WYSIWYG, or contenteditable editor (PRD D-21): all text entry is plain forms and inputs (`src/forms.rs` — single-line inputs, multi-line textareas, selects, check controls; plain-text values only), identical online and offline. No editor dependency, no HTML payloads, unless the spec's Source explicitly supersedes D-21.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [x] `python3 .blocks/spec-queue.py --specs-dir specs` exits 0 and its JSON queue contains `13-M-ops-and-release` plus every file in `specs/divergences/`, each classified `ready`, `blocked`, or `conflict` with `blocked_by`/`owned_files` populated
- [x] objective 2: two fixture specs declaring the same owned file both classify `conflict` and neither is dispatched (fixture dir under a temp path via `--specs-dir`, removed afterwards)
- [x] objective 4: the pilot pass leaves `orca worktree ps --json` showing exactly one worktree per dispatched slice, each on branch `spec/<id>-<slug>`
- [x] objective 5: a merged pilot slice shows its `Gate` re-run exit 0 in the worktree, a merge commit in `git log --oneline -5`, and `python3 .blocks/persona-smoke.py --dir persona-sanus --port 8091` + `python3 .blocks/persona-docker-verify.py --dir . --port 8080` passing on the integrating checkout after the merge
- [x] objective 5: a deliberately reddened gate in a throwaway worktree is not merged and its merge attempt, if any, is reverted (recorded in the pass evidence)
- [x] objective 6: the credential-blocked signed-installers slice produces an `orca orchestration gate-list --json` entry, no signed artifact, and its worktree preserved
- [x] objective 7: any `Desktop-verify: yes` pilot slice has a §3 entry naming the `orca computer` window/state observed, and no merge rests on GUI evidence alone
- [x] objective 8: `orca automations list --json` shows the hourly automation with the `spec-queue --fail-if-empty` precheck, created disabled; `orca automations run --id <id>` executes one pass on demand
- [ ] every objective in section 1 is checked (blocked by: objective 4 and objective 9 stay open — see §4 divergence 1)

### Pilot evidence

Pass 2026-11-17, Run `run_0a5cb3406016`, automation `75f49b64-df62-4a76-a714-a651c4cd7e31` still disabled.

| Outcome | Count |
| --- | --- |
| dispatched | 1 |
| merged | 1 |
| gated | 11 |
| re-queued `conflict` | 0 |
| scanner ready before pass | 10 |
| scanner ready after pass | 9 |

- Dispatched: `13-M-backup-repo-exclusion` — branch `spec/13-M-backup-repo-exclusion`, worktree `/Users/sergiobarba/projects/doctor-prescriptions/spec-13-M-backup-repo-exclusion`, dispatch `ctx_7a5c7bb2914b`, gate `python3 .blocks/persona-docker-verify.py --dir . --port 8080`.
- Merged: gate re-ran in the worktree, exit 0, 28/28 checks. Merge `05f96bc`; its first global gate run failed `fmt` at the pre-existing `persona-sanus/src/main.rs:2383`, so it was reverted (`94a4d0c`) and gated (`gate_2a7e1cfa00b5`). Baseline formatting then passed (`43b8a51`, 14/14 smoke checks), and the slice was reapplied (`a1aeb27`, 28/28 compose checks). Divergence cleared and deleted in `455eb7bd4c`.
- Reddened gate: `spec-15-O-red-gate-probe` / `spec/15-O-red-gate-probe`, invalid compose; gate exit 1 (`compose build or up failed`); never merged; worktree removed.
- Preserved gate worktree: `spec-13-M-signed-installers` / `spec/13-M-signed-installers`; gate `gate_0de125de84ac`; no `.dmg`, `.msi`, or `.pkg` in the checkout. Objective 4 gate: `gate_991970b05208`.
- Gated for the 5-active-agent cap and shared legacy files: `12-L`, `backup-file-safety`, `backup-minio-mirror`, `compose-production-config`, `es-ec-copy`, `release-hygiene`, `stale-asset-cleanup`, `update-channel`. Shared `persona-sanus/src/jobs.rs`, `docker-compose.yml`, and `context/runbook.md` ownership has no human-authored split, so none was dispatched.
- Desktop: no pilot slice declared `Desktop-verify: yes` and no pilot slice owned `persona-sanus/src/update.rs` or installer config; Computer Use was not invoked.
- Manual trigger: `orca automations run --id 75f49b64-df62-4a76-a714-a651c4cd7e31 --json` created run `e9f7ca49-6354-4e46-899b-7f4d8d7a8d6d`, `trigger: manual`, `precheckResult: null`, workspace `auto-spec-queue-coordinator-run-1-20261117T1647`. It was stopped and the workspace removed before it could dispatch, because the queue would exceed the 5-active-agent cap.

Second pass 2026-11-17, gated scan only, no Run created, no worker dispatched, automation `75f49b64-df62-4a76-a714-a651c4cd7e31` still disabled (`enabled: false`).

| Outcome | Count |
| --- | --- |
| dispatched | 0 |
| merged | 0 |
| gated | 8 ready slices remain queued |
| scanner ready at scan | 8 |

- Scanner: `python3 .blocks/spec-queue.py --specs-dir specs --ready-only --brief` exits 0 with 8 `ready` (`12-L-rate-limit-proxy`, `13-M-backup-file-safety`, `13-M-backup-minio-mirror`, `13-M-compose-production-config`, `13-M-es-ec-copy`, `13-M-release-hygiene`, `13-M-stale-asset-cleanup`, `13-M-update-channel`), all `whole_spec: true`, so the runbook serializes them one alone per pass and the 5-active-agent cap covers five workers, not eight.
- No clean coordinator checkout: `encantado` worktree on `serbarla/merge-screens-specs` has uncommitted user changes (`persona-sanus/assets/main.css`, `persona-sanus/src/forms.rs`, `persona-sanus/src/main.rs`, `persona-sanus/src/ui/format.rs`, `specs/22-V-ui-templates.md`); the main checkout on `master` is also dirty (`encantado`, deleted `spec-13-M-backup-repo-exclusion`, untracked `spec-13-M-signed-installers/`). Dispatching from either HEAD would fork without that in-progress work and merge back into conflict.
- Prior gates unchanged: `orca orchestration gate-list --run run_0a5cb3406016 --json` still shows `gate_991970b05208` (objective 4, signing credentials) and `gate_0de125de84ac` (signed-installers divergence) pending; `spec-13-M-signed-installers` worktree preserved, no `.dmg`/`.msi`/`.pkg` produced.
- `specs/26-Z-spec-automation-pipeline.md` itself scans `blocked` by `13-M`, which scans `blocked` by `external:signing-and-notarization-credentials` (OPEN per 13-M objective 4).

Third pass 2026-11-19, Run `run_f5fe21b8142c`, automation `75f49b64-df62-4a76-a714-a651c4cd7e31` still disabled (`enabled: false`). User chose dispatch-all-eight over the gated/one-slice options after the fence was named (whole_spec overlap, 5-active-agent cap, no human-authored split).

| Outcome | Count |
| --- | --- |
| dispatched | 8 |
| merged | 0 |
| gated | 8 |
| re-queued `conflict` | 0 |
| scanner ready before pass | 8 |
| scanner ready after pass | 8 |

- Dispatched (worktree / branch / dispatch / port): `12-L-rate-limit-proxy` (`spec-12-L-rate-limit-proxy` / `spec/12-L-rate-limit-proxy` / `ctx_ccd3a527d54e` / 8091), `13-M-backup-file-safety` (`ctx_290e948aa163` / 8092), `13-M-backup-minio-mirror` (`ctx_16bcbcd58af` / 8093), `13-M-compose-production-config` (`ctx_30e9953d5a7d` / 8094), `13-M-es-ec-copy` (`ctx_c0c8d0d202c7` / 8095), `13-M-release-hygiene` (`ctx_aac6edf97878` / 8096), `13-M-stale-asset-cleanup` (`ctx_3dd46ab9ebca` / 8097), `13-M-update-channel` (`ctx_03ebad9ff52a` / 8098). Base branch resolved at runtime: `serbarla/merge-screens-specs`. Agent `omp` (only installed agent, default).
- Merged: none. Every slice failed objective 5 condition 4 (every How-to-clear-it box checked): `12-L` 5 unchecked (verification boxes 6-10, no proxy fixture), `compose-production-config` 0 checked + divergence appended, `es-ec-copy` 2 unchecked (manual E2E walk, objective-5 sign-off), `stale-asset-cleanup` 1 unchecked (bookkeeping box), `update-channel` 1 unchecked (signed-installer E2E, external prerequisite), and `backup-file-safety` / `backup-minio-mirror` / `release-hygiene` deleted their divergence files with all boxes unchecked (7, 8, 9 respectively). No merge was attempted, so no global gate ran and no revert was needed.
- Worker receipt inaccuracies found on verification: `backup-file-safety` claimed its 7 boxes were checked before deletion (they were not; substance moved to new spec `27-AA-backup-file-safety`, all boxes checked there); `compose-production-config` claimed no code changes but its worktree holds uncommitted compose/server/README changes. Seven of eight workers left work uncommitted; only `12-L` committed (`bee0741`). Coordinator did not commit, merge, or alter worker worktrees.
- Gates (all pending, worktrees/branches preserved): `gate_6795fd03debd` (12-L), `gate_5d2534011e3e` (backup-file-safety), `gate_50185155cbe5` (backup-minio-mirror), `gate_4a1f6c3328b8` (compose-production-config), `gate_49dca50d6e1a` (es-ec-copy), `gate_5b730dd6a6e0` (release-hygiene), `gate_ff78a31d6d08` (stale-asset-cleanup), `gate_88f111732a81` (update-channel).
- Coordinator decisions relayed: release-hygiene author spelling — user confirmed `Sergio Barba`, worker corrected `Cargo.toml`; minio-mirror port bind — coordinator tore down the settled compose worker's stack (`docker compose -p spec-13-m-compose-production-config down`, ports 9000/9001 free) instead of having a worker operate cross-worktree.
- Desktop: no slice declared `Desktop-verify: yes`; Computer Use not invoked.

## 4. Divergences

*Process and tracking-file rules: see `AGENTS.md` (§ Spec-Driven Development).
Summary: when reality diverges from the spec, append a numbered entry below;
leave the matching testing-criteria `- [ ]` unchecked and note the divergence
inline next to it. Open divergences owned by a later spec also get a tracking
file in `specs/divergences/`.*

- [ ] every divergence found is appended below (or the section reads "None.")

*Entry format (one numbered item per divergence):*

1. <what diverged and its current state> — <why it could not be done as
   written, citing the No-go rule or spec that blocks it> (Verified <date>:
   <N> fix attempts — <attempt 1>, <attempt 2>, <attempt 3> — all fenced by
   <constraint>; <impact> is deferred to <later spec> by design, not a defect.)

### Divergences

1. Pilot dispatch is partial: 1 of 9 in-scope ready slices (`13-M-backup-repo-exclusion`) got a worktree and worker; the other 8 stay gated, not `conflict`. The 5-active-agent cap set under objective 1 covers five workers, not nine, and the template-era slices carry no `Owned files`: three share `persona-sanus/src/jobs.rs`, two share `docker-compose.yml`, and the rest overlap `context/runbook.md`, `README.md`, `Dockerfile`, and `persona-sanus/src/auth_server.rs`. No-go forbids dispatching overlapping or silently reassigning shared files, so wide dispatch waits for a human-authored split and a raised cap. Objectives 4 and 9 stay open. Tracking: `specs/divergences/26-Z-pilot-dispatch-fences.md`. (Verified 2026-11-17: 0 fix attempts — the nine-worker pass was not launched; the one non-overlapping slice was dispatched, its gate passed, its merge was reverted on a pre-existing `fmt` failure, the baseline was formatted, and the merge was reapplied green. The remaining slices are fenced by the 5-active-agent cap and the missing file split, not by tooling; wide dispatch is deferred to a later pass by design, not a defect.)
2. Second pass dispatched nothing: both coordinator checkouts are dirty with in-progress user work, so there is no clean base to fork `spec/<id>-<slug>` worktrees from without dropping that work or merging back into conflict; the No-go on touching unfinished work outside owned files and the runbook's clean-checkout / merge-alone rules fence dispatch. Combined with divergence 1 (5-active-agent cap, no human-authored `Owned files` split for the 8 `whole_spec` ready slices, each serialized alone per pass) and 26-Z (renumbered from 15-O) scanning `blocked` by 13-M's OPEN signing-credentials prerequisite, objectives 4 and 9 stay open. (Verified 2026-11-17: 1 scan plus gate/worktree/automation inspection — scanner exits 0 with 8 `ready`, gates `gate_991970b05208`/`gate_0de125de84ac` still pending, automation still disabled, both checkouts dirty — 0 dispatch fix attempts, because launching a worker from a dirty or cap-exceeding base would violate the No-go; cleared when the user commits or stashes the in-progress work, authors the shared-file split, and raises or confirms the cap, by design not a defect.)
3. This spec was renumbered from `15-O` to `26-Z` on 2026-11-18 — the original number collided with `specs/done/15-O-ui-tokens-actions-and-formatting.md`, and `spec-queue.py` prefix-matches `Depends on: 15-O` against this spec, so `23-W`, `24-X`, and `25-Y` (which depend on the UI-tokens spec) scanned `blocked` behind this spec's open objectives 4 and 9, deadlocking the whole UI queue. User-approved fix: rename this file to `specs/26-Z-spec-automation-pipeline.md` and update every path reference (`.blocks/spec-queue.py` doc, `context/spec-automation.md`, the disabled automation's prompt, this spec's title and housekeeping example). Objectives 4 and 9 stay open under the new id. (Verified 2026-11-18: 1 rename plus 8 reference edits; the scanner then classifies `23-W` and `24-X` `ready`.)
4. Third pass dispatched all eight ready slices at user direction but merged none: every worker failed condition 4 — five slices left How-to-clear-it boxes unchecked, three deleted their divergence files with every box unchecked, and one appended a divergence while its receipt misstated its own worktree. Objective 5 forbids merging with unchecked boxes or a divergence appended, so all eight slices are gated with worktrees preserved and objectives 4 and 9 stay open. Tracking: `specs/divergences/26-Z-third-pass-unchecked-boxes.md`. (Verified 2026-11-19: 1 full pass — 8 dispatches, 8 settlements, box-by-box and diff verification per slice; the merge path was never entered because no slice satisfied all four conditions, not because of tooling; box completion and gate resolution are deferred to gate answers by design, not a defect.)

---

*Housekeeping: once a spec is fully done with no open divergences, move its
file to `specs/done/` (same file name, e.g.
`specs/done/26-Z-spec-automation-pipeline.md`).
Specs with open divergences stay in `specs/` next to `TEMPLATE.md` until the
tracking file is cleared.*
