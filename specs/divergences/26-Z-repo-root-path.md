# Spec 26-Z — Divergence: Spec-automation repo-root path

- Source: audit 2026-11-19 of `context/` + `specs/` against `persona-sanus/` — `specs/26-Z-spec-automation-pipeline.md` header (`Repo root`) and its checked objective 8 name a repository path that does not exist on this machine; `context/spec-automation.md` was also read and names no absolute repo path. Not covered by `specs/divergences/26-Z-pilot-dispatch-fences.md` or `specs/divergences/26-Z-third-pass-unchecked-boxes.md`.
- Depends on: nothing.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Owned files: `specs/26-Z-spec-automation-pipeline.md`
- Gate: `python3 .blocks/spec-queue.py --specs-dir specs`
- Desktop-verify: no
- Result carried forward: `specs/26-Z-spec-automation-pipeline.md` names only the real repo root, and the disabled hourly automation's stored repo path is verified against it.

## Before you start

Unit testing is not required; avoid it. Prioritize end-to-end and integration
testing only.

Read the nearest `AGENTS.md` (target workspace, then parents up to repo root)
before executing.

If reality diverges from this spec: STOP, append a divergence (section 4), and
NEVER improvise outside the objectives.

## 1. Objectives

- [ ] 1. Correct the header `Repo root` — requirements: replace `/Users/sergio/projects/doctor-prescription` with `/Users/sergiobarba/projects/doctor-prescriptions` in the `specs/26-Z-spec-automation-pipeline.md` header; change no other header field; commands: `grep -n "Repo root" specs/26-Z-spec-automation-pipeline.md`
- [ ] 2. Correct the objective 8 automation command — requirements: replace `--repo path:/Users/sergio/projects/doctor-prescription` with `--repo path:/Users/sergiobarba/projects/doctor-prescriptions` in objective 8; objective 8 stays checked (the automation was created; this corrects the recorded command, it does not re-run creation); afterward the wrong string appears nowhere in the repo outside this tracker; commands: `grep -rnE "sergio/projects/doctor-prescription([^s]|$)" specs/ context/ AGENTS.md` → only matches inside `specs/divergences/26-Z-repo-root-path.md`
- [ ] 3. Verify the stored automation repo path — requirements: read the created automation (`orca automations list --json`) and confirm its repo is `/Users/sergiobarba/projects/doctor-prescriptions` and it is still disabled; if the stored path is the wrong one, do NOT edit or enable it here — append a divergence below naming the automation id and leave it disabled for the user review objective 8 already requires; commands: `orca automations list --json`

## 2. No-go

- Do not enable, run, delete, or recreate the hourly automation; objective 8 keeps it disabled until the user reviews it and the `AGENTS.md` cost ceiling.
- Do not uncheck objective 8 or rewrite any 26-Z pass evidence (§3) or divergence (§4) text; only the two path strings in objective 1–2 above may change.
- No dispatch, worktree, merge, or gate action from `context/spec-automation.md` under this tracker.
- No rich-text, WYSIWYG, or contenteditable editor (PRD D-21): all text entry is plain forms and inputs (`src/forms.rs` — single-line inputs, multi-line textareas, selects, check controls; plain-text values only), identical online and offline. No editor dependency, no HTML payloads, unless the spec's Source explicitly supersedes D-21.

## 3. Testing criteria

- [ ] `python3 .blocks/spec-queue.py --specs-dir specs` passes (run from the repo root) and still classifies `26-Z-spec-automation-pipeline` as `blocked` by `13-M`
- [ ] objective 1: `grep -n "Repo root" specs/26-Z-spec-automation-pipeline.md` shows `/Users/sergiobarba/projects/doctor-prescriptions`
- [ ] objective 2: `grep -rnE "sergio/projects/doctor-prescription([^s]|$)" specs/ context/ AGENTS.md` matches only inside this tracker file
- [ ] objective 3: `orca automations list --json` shows the "Spec queue coordinator" automation disabled, with its repo path recorded in the check evidence (or a divergence appended per objective 3)
- [ ] every objective in section 1 is checked

## 4. Divergences

- [ ] every divergence found is appended below (or the section reads "None.")

### Divergences

1. `specs/26-Z-spec-automation-pipeline.md` names `/Users/sergio/projects/doctor-prescription` as `Repo root` (header line 5) and in the checked objective 8's `orca automations create --repo path:` argument; that directory does not exist — the checkout is `/Users/sergiobarba/projects/doctor-prescriptions`. The text fix is trivial, but whether the already-created automation stored the wrong path can only be settled by reading it, and correcting the stored automation is fenced by objective 8's own user-review rule. (Verified 2026-11-19: 0 fix attempts — `grep` for both path spellings across `specs/`, `context/`, and `AGENTS.md`, plus the audit's report-only scope; the doc correction is deferred to this tracker by design, not a defect in the shipped code.)

---

*Housekeeping: once every box above is checked, delete this file; `specs/26-Z-spec-automation-pipeline.md` stays in `specs/` until its own objectives 4 and 9 clear.*
