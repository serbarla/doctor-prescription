# Spec automation pipeline

Spec 15-O turns the one-agent-per-spec workflow into one Orca coordinator pass.
A pass scans the queue, dispatches one supervised worker per file-disjoint ready
slice, collects settled workers, verifies them mechanically, and either merges it or
opens a decision gate. No human launches, verifies, or merges an individual slice
by hand; humans resolve gates, split ownership conflicts, and set policy.

## Artifacts

| Artifact | Role |
| --- | --- |
| `AGENTS.md` § Subagent Delegation Fleet | Orca-worker row, per-day cost ceiling, DAG-width policy. |
| `.blocks/spec-queue.py` | Queue scanner; classifies every spec and divergence slice. |
| `context/spec-automation.md` | This runbook plus the verbatim coordinator and worker prompts. |
| `specs/<NN-X>.md` | A spec slice; §1 boxes are the completion contract. |
| `specs/divergences/<id>.md` | A remediation slice; "How to clear it" boxes are the contract. |
| Orca Run + tasks | Dispatch, dependency DAG, worker terminals, decision gates. |
| Git worktrees + branches | One `spec/<id>-<slug>` branch per slice. |

## Queue contract

`python3 .blocks/spec-queue.py --specs-dir specs` prints one JSON object:

```json
{"ok": true, "queue": [], "counts": {"total": 0, "ready": 0, "blocked": 0, "conflict": 0, "specs": 0, "divergences": 0}}
```

Each queue row carries `id`, `path`, `kind` (`spec|divergence`), `status`
(`ready|blocked|conflict`), `blocked_by`, `owned_files`, `gate`,
`desktop_verify`, `whole_spec`, and `unchecked`.

- `ready` means every dependency has no unchecked §1 objective cell and no
  declared `Owned files` pattern overlaps another ready row.
- `blocked` means a dependency still has unchecked objectives, the slice has no
  unchecked work (`blocked_by: ["complete"]`), or a divergence's source spec
  lists it as `OPEN` behind an external prerequisite.
- `conflict` means two ready rows declare overlapping ownership. Both are
  demoted; the scanner never chooses a winner. A human must split the files or
  the specs.
- Files written before the three header fields default to `whole_spec: true`,
  `owned_files: ["*"]`, the global `persona-smoke` gate, and
  `Desktop-verify: no`. Their ownership subset check is skipped, they merge
  alone, and the coordinator serializes them.
- `--ready-only` prints just dispatchable rows. `--fail-if-empty` exits non-zero
  when no row is dispatchable, which is the automation precheck.
- `--brief` prints `id`/`status` only and fits the 2 KB block run cap.

### Spec header fields

```text
- Owned files: persona-sanus/src/ui/button.rs, assets/main.css
- Gate: python3 .blocks/persona-smoke.py --dir persona-sanus --port {port} --db-path {db_path}
- Desktop-verify: no
```

`Owned files` is comma-separated paths or globs. `Gate` is the exact command the
worker runs and the coordinator re-runs. `{port}` and `{db_path}` are rendered by
the coordinator; a gate only names the worktree and its index. `Desktop-verify`
is `yes` only for a desktop surface that needs a GUI observation.

### Ports and databases

For worktree index `i` (zero-based, sorted by slice id):

- port: `8091 + i`
- smoke database: `<worktree>/.spec-data/app-i.db`
- compose host port: `8080 + i` only when the compose stack's own port is not in
  use; otherwise gate rather than stealing a port.

Ports are derived, never written into a spec or a prompt. The coordinator frees a
port only after that slice's gate passes and its worktree is removed.

## One pass

The coordinator runs exactly one pass and then stops. A pass is:

1. Scan.
2. Dispatch the ready set.
3. Collect settled workers.
4. Verify and merge or gate.
5. Clean up and record evidence.

### 1. Scan

```bash
python3 .blocks/spec-queue.py --specs-dir specs
```

Read `counts`, then dispatch every `status: ready` row in the same pass. Do not
dispatch `blocked` or `conflict`. Do not reorder the DAG to create work. If a
ready row depends on a row dispatched in this pass, that cannot happen: an
unchecked dependency is `blocked` by definition.

Resolve the base branch at runtime from the coordinator checkout, never from a
constant in a prompt or spec:

```bash
git -C . symbolic-ref --short HEAD
```

### 2. Dispatch the ready set

Create one Run for the pass:

```bash
orca orchestration run-create --objective "spec automation pass" --json
```

For each ready row, create one task with its dependency task ids as `--deps`
(`[]` when the scanner reports none). Keep the mapping `slice id -> task id`.

```bash
orca orchestration task-create --spec-file <spec path> --task-title "<id>" --deps '<json_array>' --json
```

For each ready row, create exactly one worktree from the runtime base branch.
Worktree index `i` and port `8091 + i` come from the sorted ready-id order.

```bash
orca worktree create \
  --name spec-<id>-<slug> \
  --branch spec/<id>-<slug> \
  --base-branch <runtime base branch> \
  --json
```

Record the returned worktree path, branch, index, port, and database path. Never
remove or recreate an existing worktree for the same slice; resume from its path
if it is present and clean.

Start exactly one supervised worker per ready row. The worker prompt is section
"Worker prompt" below with its placeholders rendered. `--agent` is an id resolved
from the local Orca install at run time (`orca account list --json`,
`orca agent launch --help`, the app's enabled-agent list); never invent one.

```bash
orca orchestration worker-start \
  --spec-file <spec path> \
  --worktree path:<worktree path> \
  --agent <resolved agent id> \
  --json
```

All file-disjoint ready rows start in the same pass. A `conflict` or `blocked`
row stays queued and gets no worktree or worker.

Apply the cost ceiling in `AGENTS.md` § Subagent Delegation Fleet before each
launch. If the projected spend for the day would exceed it, create a gate
instead of a worker and continue with independent slices.

### 3. Collect settled workers

A coordinator pass ends after it has dispatched and then handled every settled
worker. `worker_done` settles a Task. An escalation with `stopReason` does not:
steer it with `orca orchestration send`, stop and retry it, or gate it.

```bash
orca orchestration inbox --terminal run:<run_id> --limit 50
orca orchestration worker-list --json
orca orchestration gate-list --json
```

For each settled worker, collect: dispatch id, task id, worktree path, branch,
base commit, final commit, the gate command and its exit code, the §1 cell
state, and whether §4 gained extra lines.

### 4. Verify mechanically

A settled worker is mergeable only when all four hold. Re-run them yourself; a
worker's claim is not evidence.

1. **Gate green.** Re-run the slice's exact `Gate` in its worktree, rendered with
   that worktree's port and database path. Exit code 0.
   ```bash
   cd <worktree> && <gate command with {port}/{db_path} rendered>
   ```
2. **Ownership.** The changed files are a subset of `Owned files`.
   ```bash
   git -C <worktree> diff --name-only <base_commit>...HEAD
   ```
   For a legacy row with `whole_spec: true`, skip the subset check: merge it
   alone, never in the same batch as another row.
3. **No divergence appended.** The spec did not gain a §4 entry during the run.
   ```bash
   git -C <worktree> diff <base_commit>...HEAD -- <spec path>
   ```
   Any added numbered entry under `### Divergences` is a gate, not a merge.
4. **All boxes checked.** Every §1 box of the slice is `[x]`; for a divergence
   slice, every "How to clear it" box is `[x]`. Unchecked rows in the slice's own
   section are not complete.
   ```bash
   python3 .blocks/spec-queue.py --specs-dir <worktree>/specs
   ```

If any condition fails, do not merge. Go to step 5 and gate it. A red gate is
never force-merged, force-pushed, or history-rewritten.

Merge in topological order (dependency first), with `--no-ff`:

```bash
git merge --no-ff spec/<id>-<slug>
```

Then run the parent spec's global gates on the integrating checkout with a port
that no live worker holds:

```bash
python3 .blocks/persona-smoke.py --dir persona-sanus --port <allocated>
python3 .blocks/persona-docker-verify.py --dir . --port <allocated>
```

If a global gate fails, revert the merge immediately
(`git revert -m 1 HEAD`), reopen the slice as `blocked`, and gate it. A red
integrating checkout is never left standing. If the gates pass

- move a fully-checked spec to `specs/done/` with its file name unchanged;
- delete a cleared divergence file per its "How to clear it" section;
- remove the worktree only when its work is merged and verified:
  ```bash
  orca worktree rm --worktree path:<worktree path> --delete-branch --json
  ```

### 5. Gate everything that is not mechanically mergeable

Create one decision gate per non-mergeable slice:

```bash
orca orchestration gate-create \
  --task <task_id> \
  --question "<what needs a decision>" \
  --options '["<option>", "<option>"]' \
  --json
```

Gate when:

- the gate command is red after 15 fix attempts;
- changed files exceed `Owned files`;
- a divergence was appended during the run;
- any §1 or "How to clear it" box is unchecked;
- an external prerequisite is missing (signing certificates, counsel sign-off,
  credentials, store accounts);
- two slices overlap on owned files (a human-authored split);
- the daily cost ceiling would be exceeded.

A 15-attempt failure also files a Linear issue in Specs-Issue format. Faking
signing, notarization, credentials, or a passing gate is forbidden; an OPEN
prerequisite stays OPEN.

Gated slices keep their worktree and branch untouched for inspection. Do not
delete, reset, or reuse them. Record the gate id in the pass evidence.

### 6. Desktop visual smoke

Only a slice with `Desktop-verify: yes` or `Owned files` naming the desktop
surface (`persona-sanus/src/update.rs`, desktop installer config) gets a GUI
pass. Run it after the gate re-run in step 4 and before merge:

1. Launch the desktop build from the worker's worktree.
2. Resolve flags first: `orca computer get-app-state --help`.
3. Inspect: `orca computer capabilities --json`, `orca computer list-windows --json`,
   `orca computer get-app-state ... --json`.
4. Record the observed window/state as text in the spec's §3 evidence.

A GUI observation never merges anything. The `Gate` must still exit 0. If the
GUI state contradicts the gate, gate the slice (step 5), do not merge it.
Computer Use is never used for editing code, git, tests, builds, terminal work,
or Linear.

### 7. Record and stop

Record the pass outcome in the executed spec's §3 evidence: dispatched, merged,
gated, and re-queued counts; worktree and branch names; gate ids; port and
database assignments. Update §1 boxes only for work actually verified. Append
a §4 divergence for anything the pass could not do as written.

Stop after one pass. Do not loop, poll, or launch a second pass; the hourly
automation is the only trigger.

## Verbatim coordinator prompt

```text
Read specs/15-O-spec-automation-pipeline.md, AGENTS.md, and context/spec-automation.md.
Run exactly one coordinator pass: scan -> dispatch the ready set -> collect settled workers -> merge or gate -> stop.
Never run a second pass in the same turn.

Scan:
  python3 .blocks/spec-queue.py --specs-dir specs
Dispatch every row with status ready, and only those. Do not dispatch blocked or conflict rows.
Resolve the base branch at runtime with `git symbolic-ref --short HEAD`; never use a hardcoded branch.
Create one orchestration Run, one task per ready row with its DAG deps, one worktree per ready row
named spec-<id>-<slug> on branch spec/<id>-<slug>, and exactly one supervised worker per worktree.
Resolve the worker agent id from the local Orca install at execution time; never invent one.
Derive each worktree's port as 8091 + <sorted ready index> and give it a unique --db-path under that worktree.
Respect the per-day cost ceiling in AGENTS.md: if the pass would exceed it, create a gate instead of a worker.

Each worker gets this prompt, with its own placeholders rendered:
<worker prompt from context/spec-automation.md>

Collect every settled worker. Merge a slice only when all four hold:
  1. its Gate, re-run by you in its worktree with the derived port/db-path, exits 0;
  2. git -C <worktree> diff --name-only <base>...HEAD is a subset of its Owned files
     (skip for whole_spec legacy rows, which merge alone, never in a batch);
  3. no divergence was appended to its spec during the run;
  4. every objective box and every "How to clear it" box for that slice is checked.
Merge in dependency order with `git merge --no-ff spec/<id>-<slug>`, then run the parent spec's
global gates on the integrating checkout with a free allocated port:
  python3 .blocks/persona-smoke.py --dir persona-sanus --port <port>
  python3 .blocks/persona-docker-verify.py --dir . --port <port>
On any global-gate failure, `git revert -m 1 HEAD` the merge, reopen the slice as blocked, and gate it.
Never leave a red integrating checkout standing. Never force-merge, force-push, or rewrite history.

Gate every settled worker that is not mechanically mergeable: red gate after 15 fix attempts,
changed files outside Owned files, a divergence appended during the run, an unchecked box,
an external prerequisite (signing/notarization credentials, counsel sign-off, store accounts), or
overlapping ownership. Gate with `orca orchestration gate-create --task <task_id> --question <text>
--options '<json_array>' --json`; keep the worktree and branch untouched for inspection. Continue
independent slices. A 15-attempt failure also files a Linear issue in Specs-Issue format.

Desktop visual smoke only: for Desktop-verify: yes or Owned files naming the desktop surface,
after the gate re-run and before merge, resolve flags with `orca computer get-app-state --help`, launch
the build from that worktree, observe with `orca computer capabilities --json`,
`orca computer list-windows --json`, and `orca computer get-app-state ... --json`, and record the window/state
as text in the spec's §3. A GUI observation alone never merges anything; a contradiction gates the slice.
Never use Computer Use for editing, git, tests, builds, terminal work, or Linear.

After merging a verified slice, move a fully-checked spec to specs/done/ (same file name),
delete a cleared divergence file per its "How to clear it" section, and remove its worktree with
`orca worktree rm --worktree path:<path> --delete-branch --json`. Never remove a worktree with
unmerged or unverified work.

Record the pass outcome in the executed spec's §3: dispatched, merged, gated, and re-queued
counts plus worktree and branch names. Append §4 divergences for anything not done as written.
Check a spec §1 box only after its command exits 0. Then stop.
```

## Worker prompt

Render every placeholder. The worker may modify only `Owned files`; it may check
only this slice's boxes; it must leave every other spec alone.

```text
You are one supervised worker for one spec-automation slice.

Slice: <spec path>
Kind: <spec|divergence>
Worktree: <absolute worktree path>
Branch: spec/<id>-<slug>
Owned files: <comma-separated paths/globs, or "* (whole spec; legacy)" >
Gate: <exact command with {port} rendered and {db_path} rendered>
Port: <8091 + index>
Database: <worktree>/.spec-data/app-<index>.db
Desktop-verify: <yes|no>

Objective:
Read <spec path> completely and the nearest AGENTS.md before editing. Implement only its
objectives for this slice. Do not edit any file outside Owned files. Do not touch
specs/done/, context/PRD.md scope, other specs' objectives, or another worker's worktree.

Gate rule:
Run the exact Gate command above. Check the slice's boxes only after that command
exits 0. Do not check a box on a claim, a partial run, or another command's output.
Never fake a gate, a signature, a credential, or an external prerequisite.

Divergence rule:
On any mismatch between reality and the spec, STOP. Append a divergence to the
spec (section 4), check nothing, and report the divergence. If you cannot edit the
spec because it is outside Owned files, report the divergence text instead of
checking boxes.

Cost rule:
Respect the per-day cost ceiling in AGENTS.md. Do not start new work that would
exceed it; report instead.

Failure rule:
You may attempt a fix 15 times. After the 15th failure, file a Linear issue in
Specs-Issue format, leave the slice unchanged, and report the failure. Do not
force-merge, force-push, rewrite history, or bypass a red gate.

Desktop rule (only when Desktop-verify is yes):
After the Gate exits 0, launch the desktop build from this worktree, resolve flags with
`orca computer get-app-state --help`, observe with `orca computer capabilities --json`,
`orca computer list-windows --json`, and `orca computer get-app-state ... --json`, and record the observed
window/state as text in section 3 of the spec. A GUI observation never substitutes for the
Gate. Never use Computer Use for editing, git, tests, builds, terminal work, or Linear.

Report:
Return a worker_done receipt with the changed files, the Gate command and exit code, the
unchecked/checked box counts for this slice, any appended divergence, and any gate reason.
```

## Failure and escalation

| Fault | Action |
| --- | --- |
| Red gate after 15 attempts | Gate the task; file the Linear Specs-Issue; keep the worktree. |
| Changed file outside `Owned files` | Gate; do not merge; inspect the diff. |
| Divergence appended during the run | Gate; do not merge; let a human decide. |
| Unchecked box | Gate; reopen `blocked`; keep the branch. |
| Missing signature/notarization/counsel/credential | Gate as `OPEN`; no fake artifact. |
| Two slices share a file | `conflict` for both; human splits ownership. |
| Global gate red after merge | `git revert -m 1 HEAD`; reopen and gate the slice. |
| Cost ceiling reached | Gate remaining slices; keep dispatched work. |

## Verification

```bash
python3 .blocks/spec-queue.py --specs-dir specs
python3 .blocks/spec-queue.py --specs-dir specs --ready-only --fail-if-empty
python3 .blocks/markdown-table-check.py --path context/spec-automation.md
caveman-blocks lint .blocks/spec-queue.py
orca orchestration gate-list --json
orca worktree ps --json
orca automations list --json
```
