# Spec 36 — Phase AJ: Pipeline drain and tracker reconciliation

- Source: `specs/26-Z-spec-automation-pipeline.md` objectives 4 and 9 and section 4 divergences 1 and 4; `specs/divergences/26-Z-pilot-dispatch-fences.md`, `specs/divergences/26-Z-third-pass-unchecked-boxes.md`, and `specs/divergences/13-M-stale-tracker-evidence.md` (merged 2026-11-20; all three trackers deleted at this spec's creation — the stale-tracker file defers its last three boxes to the 8 pending gates and the pilot fences, so the three form one dependency chain and one spec).
- Depends on: 34-AH.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Owned files: `specs/26-Z-spec-automation-pipeline.md`, `context/spec-automation.md`, `specs/36-AJ-pipeline-drain-and-tracker-reconcile.md`, `AGENTS.md`
- Gate: `caveman-blocks run spec-queue --specs-dir specs`
- Desktop-verify: no
- Result carried forward: the queue is drained in dependency order — the 8 third-pass gate dispositions recorded, one manual coordinator pass rerun over the refreshed queue, tracker evidence reconciled against `master`, and 26-Z objectives 4 and 9 checked (26-Z moves to `specs/done/` only when 13-M is done, otherwise the 13-M residual is named in section 4).

## Before you start

Unit testing is not required; avoid it. Prioritize end-to-end and integration testing only.

Read the nearest `AGENTS.md` (target workspace, then parents up to repo root) before executing.

If reality diverges from this spec: STOP, append a divergence (section 4), and NEVER improvise outside the objectives.

## 1. Objectives

*The ordered work to implement — one item per step. Check `[x]` only after the step is implemented and verified.*

*Item format:* `<verb phrase>` — requirements: `<what must change / constraints>`; commands: `<exact command>`.

- [ ] 1. Disposition the 8 third-pass gates — requirements: for each pending gate from Run `run_f5fe21b8142c` (`gate_6795fd03debd`, `gate_5d2534011e3e`, `gate_50185155cbe5`, `gate_4a1f6c3328b8`, `gate_49dca50d6e1a`, `gate_5b730dd6a6e0`, `gate_ff78a31d6d08`, `gate_88f111732a81`), re-run that slice's exact Gate in its preserved worktree/branch and record one disposition: `merged` (all four 26-Z objective 5 conditions hold), `gated` (the named external prerequisite is recorded in `context/spec-automation.md` with the worktree preserved), or `obsolete` (the slice tracker no longer exists on `master` because its substance was merged or absorbed into specs 34-AH/35-AI — cite the spec and the `git merge-base` evidence); never force-merge, force-push, or rewrite history; commands: `caveman-blocks run spec-queue --specs-dir specs`
- [ ] 2. Confirm the Owned-files split and rerun one coordinator pass — requirements: verify the explicit Owned-files headers of specs 33-AG through 36-AJ and the refreshed queue (no `whole_spec` overlap among the dispatch set; the AGENTS.md max-5 active-agent cap confirmed before enabling the automation, which stays disabled until that review); run one manual coordinator pass (one worktree and worker per dispatched slice on branch `spec/<id>-<slug>`, base branch resolved at runtime), and record dispatched/merged/gated/re-queued counts with worktree and branch names in `specs/26-Z-spec-automation-pipeline.md` section 3; commands: `caveman-blocks run spec-queue --specs-dir specs --ready-only --brief`
- [ ] 3. Reconcile stale-tracker evidence against master — requirements: verify each merged tracker claim on the current tree (the 12-L `bee0741` branch disposition per `specs/done/12-L-hardening.md`, the 27-AA existence correction already recorded in 26-Z, and the merged tracker deletions from this spec's Source), rerun the scanner, and confirm the counts match the refreshed queue; record the reconciliation table in this spec's section 3; commands: `caveman-blocks run spec-queue --specs-dir specs`
- [ ] 4. Close 26-Z objectives 4 and 9, and housekeeping conditionally — requirements: only after objectives 1–3 pass, check `specs/26-Z-spec-automation-pipeline.md` objectives 4 and 9 and its `every objective in section 1 is checked` criterion; move it to `specs/done/26-Z-spec-automation-pipeline.md` only if 13-M is already done (spec 35-AI objective 4), otherwise leave 26-Z in `specs/` and append a section 4 residual naming 13-M (35-AI, certificate fence) as the only blocker without un-checking objective 4; any residual split or cap count that is still OPEN is recorded in this spec's section 4 instead of checked; commands: `caveman-blocks run spec-queue --specs-dir specs`
## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No auto-merge when any 26-Z objective 5 condition is unmet, no force-merge, force-push, or history rewrite; no dispatching `conflict` or `blocked` slices, and no silently reassigning shared files to one of two overlapping slices.
- No checking `specs/13-M-ops-and-release.md` objectives 4 or 5 (spec 35-AI owns them behind the signed-installer fence) and no faking any external prerequisite (signing credentials, TLS host, counsel sign-off); no touching `specs/done/` or `context/PRD.md` scope; no new daemon, service, database, or dependency.
- No rich-text, WYSIWYG, or contenteditable editor (PRD D-21): all text entry is plain forms and inputs (`src/forms.rs` — single-line inputs, multi-line textareas, selects, check controls; plain-text values only), identical online and offline. No editor dependency, no HTML payloads, unless the spec's Source explicitly supersedes D-21.
- No unit tests; no comments in any code.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [ ] `caveman-blocks run spec-queue --specs-dir specs` exits 0 and the queue matches the dispositions in objective 1
- [ ] objective 1: the 8 gate dispositions are recorded, each citing `merged`/`gated`/`obsolete` with its evidence
- [ ] objective 2: the rerun pass shows one worktree per dispatched slice on branch `spec/<id>-<slug>` and the counts are recorded in 26-Z section 3
- [ ] objective 3: the reconciliation table is recorded in this section
- [ ] every objective in section 1 is checked

## 4. Divergences

*Process and tracking-file rules: see `AGENTS.md` (§ Spec-Driven Development). Summary: when reality diverges from the spec, append a numbered entry below; leave the matching testing-criteria `- [ ]` unchecked and note the divergence inline next to it. Open divergences owned by a later spec also get a tracking file in `specs/divergences/`.*

- [ ] every divergence found is appended below (or the section reads "None.")

*Entry format (one numbered item per divergence):*

1. <what diverged and its current state> — <why it could not be done as written, citing the No-go rule or spec that blocks it> (Verified <date>: <N> fix attempts — <attempt 1>, <attempt 2>, <attempt 3> — all fenced by <constraint>; <impact> is deferred to <later spec> by design, not a defect.)

### Divergences

None yet.
