# Spec 15 — Phase O: Spec automation pipeline

- Source: user decision 2026-11-16 — automate the current one-agent-per-spec workflow. Locked choices: auto-merge on green gate, parallelism as wide as the dependency DAG allows, scheduled Orca automation as trigger, Computer Use for desktop visual smoke only, one worktree per spec or divergence file.
- Depends on: 13-M (active spec whose `specs/divergences/` slices are the pilot queue input).
- Repo root: `/Users/sergio/projects/doctor-prescription`. Every path and command below is relative to it.
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

- [ ] 1. Amend the worker-pool and spec-template contracts — requirements: `AGENTS.md` (§ Subagent Delegation) gains an Orca-worker row stating DAG-width parallelism supersedes the single-subagent caps for Orca orchestration workers only, with a per-day cost ceiling the user sets before objective 6 is enabled (record the number in `AGENTS.md`, do not invent it); `specs/TEMPLATE.md` header gains three fields after `Repo root`: `Owned files: <comma-separated paths/globs the executing worker may modify>`, `Gate: <exact .blocks or cargo command that must exit 0>`, `Desktop-verify: <yes|no>`; specs written before this change and lacking the fields default to whole-spec ownership, their global smoke as gate, and `Desktop-verify: no`; commands: `python3 .blocks/markdown-table-check.py --path AGENTS.md` and `python3 .blocks/markdown-table-check.py --path specs/TEMPLATE.md`
- [ ] 2. Build the queue scanner block — requirements: new `.blocks/spec-queue.py` following the block rules in `.blocks/INDEX.md` (header first, params only, one small JSON object, exit 0 on success and non-zero with `{"error": ...}` on failure, idempotent, registered in `AGENTS.md` and `.blocks/INDEX.md`); it scans `specs/*.md` (excluding `TEMPLATE.md`, `done/`) plus `specs/divergences/*.md`, parses for each file its unchecked §1 objectives, `Depends on`, `Owned files`, `Gate`, and `Desktop-verify`, and emits `{queue: [{id, path, kind: spec|divergence, ready|blocked|conflict, blocked_by, owned_files, gate, desktop_verify}], counts}`; `ready` = no unchecked dependency and no `Owned files` overlap with another ready file, overlap demotes both to `conflict` (never picks a winner silently), divergence files are blocked while their source spec lists them as OPEN with an external prerequisite (e.g. signing credentials); `--fail-if-empty` exits non-zero when no file is `ready` (used as the automation precheck); commands: `python3 .blocks/spec-queue.py --specs-dir specs` → JSON listing `13-M-ops-and-release` and all files in `specs/divergences/`, and `python3 .blocks/spec-queue.py --specs-dir specs --ready-only --fail-if-empty; echo $?` → `0` only if at least one slice is dispatchable
- [ ] 3. Write the coordinator runbook and prompt — requirements: new `context/spec-automation.md` documenting the loop end to end and containing the verbatim coordinator prompt: read this spec, `AGENTS.md`, and `context/spec-automation.md`; run one pass only (scan → dispatch ready set → collect settled workers → merge or gate → stop); a worker prompt per slice states the spec path, owned-files confinement, its exact `Gate` command, "check spec boxes only after the gate exits 0", "on any mismatch STOP, append a divergence to the spec, check nothing, and report", and the AGENTS.md rule of 15 fix attempts then a Linear issue in Specs-Issue format; no credentials, secrets, ports, or dates hardcoded — ports derive as `8091 + <worktree index>` and each worker gets a unique `--db-path` under its worktree; commands: `python3 .blocks/markdown-table-check.py --path context/spec-automation.md`
- [ ] 4. Dispatch one worktree per spec or divergence — requirements: coordinator pass runs `orca orchestration run-create --objective "spec automation pass" --json`, then per ready file `orca worktree create --name spec-<id>-<slug> --branch spec/<id>-<slug> --base-branch <branch resolved at runtime from the coordinator checkout> --json` (never a hardcoded base branch), records the returned worktree path in the orchestration task, and starts exactly one worker with `orca orchestration worker-start --spec-file <spec path> --worktree path:<worktree path> --agent <agent id> --json`; tasks mirror the scanner DAG via `orca orchestration task-create --spec-file <spec path> --deps <json_array of task ids> --json`; file-disjoint ready slices all start in the same pass, `conflict`/`blocked` slices are not dispatched and stay queued; commands: `orca worktree ps --json` shows one worktree per dispatched slice on branch `spec/<id>-<slug>`
- [ ] 5. Auto-merge only mechanically verified results — requirements: a settled worker is mergeable only when all four hold: its `Gate` command re-run by the coordinator in that worktree exits 0, `git -C <worktree> diff --name-only <base>...HEAD` is a subset of the slice's `Owned files` (template-era files without the field: subset check skipped, slice merges alone), no divergence was appended to the spec during the run, and every §1 box of the slice is checked; merge in topological (dependency) order with `git merge --no-ff spec/<id>-<slug>`, then run the parent spec's global gates on the integrating checkout (`python3 .blocks/persona-smoke.py --dir persona-sanus --port <allocated>` and `python3 .blocks/persona-docker-verify.py --dir . --port <allocated>`); on any global-gate failure `git revert -m 1 HEAD` the merge, reopen the slice as `blocked`, and gate it (objective 6) — a red main is never left standing; on success move a fully-checked spec to `specs/done/` (divergence files are deleted per their own "How to clear it" section) and remove the worktree with `orca worktree rm --worktree path:<worktree path> --delete-branch --json`, never removing a worktree with unmerged or unverified work; commands: `git log --oneline -5` shows the merge and `python3 .blocks/persona-smoke.py --dir persona-sanus --port 8091` passes on the integrating checkout
- [ ] 6. Escalate through decision gates, never around them — requirements: any non-mergeable settled worker, any divergence appended by a worker, any external prerequisite (signing certificates, counsel sign-off, credentials), and any 15-attempt failure (which also files the Linear Specs-Issue per `AGENTS.md`) blocks its task with `orca orchestration gate-create --task <task_id> --question <text> --options <json_array> --json` and the coordinator pass continues with independent slices; gated slices keep their worktree and branch untouched for inspection; commands: `orca orchestration gate-list --json` shows the gate and `orca worktree ps --json` still lists the gated slice's worktree
- [ ] 7. Restrict Computer Use to desktop visual smoke — requirements: only slices with `Desktop-verify: yes` or whose `Owned files` include the desktop feature surface (`persona-sanus/src/update.rs`, desktop installer config) get a GUI pass, executed after objective 5's gate re-run and before merge: launch the desktop build from the worker's worktree, inspect it with `orca computer list-windows --json` and `orca computer get-app-state --json` (resolve exact flags via `orca computer get-app-state --help` first — do not guess flags), and record the observed state as text evidence in the spec's §3; a GUI observation alone never merges anything — the slice's `Gate` must still exit 0, and if the GUI state contradicts the gate the slice is gated (objective 6), not merged; Computer Use is never used for editing code, git, tests, terminal work, or Linear; commands: `orca computer capabilities --json` succeeds and the spec §3 entry names the window/state observed
- [ ] 8. Enable the scheduled automation — requirements: resolve the provider/agent id from the local Orca install at execution time (do not hardcode an unverified id), then create a disabled-by-default automation `orca automations create --name "Spec queue coordinator" --trigger hourly --repo path:/Users/sergio/projects/doctor-prescription --precheck "python3 .blocks/spec-queue.py --specs-dir specs --ready-only --fail-if-empty" --prompt "Read specs/15-O-spec-automation-pipeline.md and context/spec-automation.md, then execute exactly one coordinator pass." --provider <resolved agent id> --json`; the user reviews the created automation and the `AGENTS.md` cost ceiling from objective 1 before it is enabled (`orca automations edit --id <id>` / enable path from `orca automations edit --help`); manual triggering stays available via `orca automations run --id <id>`; commands: `orca automations list --json` shows the automation with its precheck and hourly trigger
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

- [ ] `python3 .blocks/spec-queue.py --specs-dir specs` exits 0 and its JSON queue contains `13-M-ops-and-release` plus every file in `specs/divergences/`, each classified `ready`, `blocked`, or `conflict` with `blocked_by`/`owned_files` populated
- [ ] objective 2: two fixture specs declaring the same owned file both classify `conflict` and neither is dispatched (fixture dir under a temp path via `--specs-dir`, removed afterwards)
- [ ] objective 4: the pilot pass leaves `orca worktree ps --json` showing exactly one worktree per dispatched slice, each on branch `spec/<id>-<slug>`
- [ ] objective 5: a merged pilot slice shows its `Gate` re-run exit 0 in the worktree, a merge commit in `git log --oneline -5`, and `python3 .blocks/persona-smoke.py --dir persona-sanus --port 8091` + `python3 .blocks/persona-docker-verify.py --dir . --port 8080` passing on the integrating checkout after the merge
- [ ] objective 5: a deliberately reddened gate in a throwaway worktree is not merged and its merge attempt, if any, is reverted (recorded in the pass evidence)
- [ ] objective 6: the credential-blocked signed-installers slice produces an `orca orchestration gate-list --json` entry, no signed artifact, and its worktree preserved
- [ ] objective 7: any `Desktop-verify: yes` pilot slice has a §3 entry naming the `orca computer` window/state observed, and no merge rests on GUI evidence alone
- [ ] objective 8: `orca automations list --json` shows the hourly automation with the `spec-queue --fail-if-empty` precheck, created disabled; `orca automations run --id <id>` executes one pass on demand
- [ ] every objective in section 1 is checked

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

None.

---

*Housekeeping: once a spec is fully done with no open divergences, move its
file to `specs/done/` (same file name, e.g.
`specs/done/15-O-spec-automation-pipeline.md`).
Specs with open divergences stay in `specs/` next to `TEMPLATE.md` until the
tracking file is cleared.*
