# Spec 25-Y — Divergence: Screens INDEX stale code references

- Source: audit 2026-11-19 of `context/` + `specs/` against `persona-sanus/` — `context/screens/INDEX.md` §1.1, §2, and its `Source` header still cite pre-split `persona-sanus/src/main.rs` line numbers and a bare plan filename; specs 15-O–25-Y (`specs/done/`) split the UI into `persona-sanus/src/ui/*` and shifted every component below `SettingsClinic`. Not covered by any existing file in `specs/divergences/`.
- Depends on: nothing.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Owned files: `context/screens/INDEX.md`, `context/database-design.md`
- Gate: `python3 .blocks/markdown-table-check.py --path context/screens/INDEX.md`
- Desktop-verify: no
- Result carried forward: every component line, enum range, and file path cited by `context/screens/INDEX.md` matches the current `persona-sanus/src/main.rs`, and the duplicated ER edge in `context/database-design.md` §3 is gone.

## Before you start

Unit testing is not required; avoid it. Prioritize end-to-end and integration
testing only.

Read the nearest `AGENTS.md` (target workspace, then parents up to repo root)
before executing.

If reality diverges from this spec: STOP, append a divergence (section 4), and
NEVER improvise outside the objectives.

## 1. Objectives

- [ ] 1. Correct the drifted component lines in the INDEX §2 tables — requirements: update only the `` `main.rs` `` line in the Component column for `Patients` 3408→3422, `PatientNew` 3621→3635, `PatientDetail` 3905→3897, `PatientEdit` 4144→4136, `Prescriptions` 5349→5344, `PrescriptionNew` 5697→5692, `PrescriptionDetail` 6627→6629, `PrescriptionPreview` 6155→6157, `Templates` 4462→4432, `TemplateNew` 4606→4583, `TemplateEdit` 4743→4720, `TemplatePreview` 5057→5037, `SettingsUsers` 2636→2638, `SettingsProfile` 2926→2932, `Audit` 3273→3283 (verified against `fn <Name>(` in `persona-sanus/src/main.rs` on 2026-11-19); re-derive each number at edit time, because the working tree is dirty and lines shift on commit; change no route, status, or copy cell; commands: `grep -n "^fn Patients(\|^fn PatientNew(\|^fn PatientDetail(\|^fn PatientEdit(\|^fn Prescriptions(\|^fn PrescriptionNew(\|^fn PrescriptionDetail(\|^fn PrescriptionPreview(\|^fn Templates(\|^fn TemplateNew(\|^fn TemplateEdit(\|^fn TemplatePreview(\|^fn SettingsUsers(\|^fn SettingsProfile(\|^fn Audit(" persona-sanus/src/main.rs`
- [ ] 2. Fix the §1.1 frozen-exception lines — requirements: `Navbar` citation `main.rs:1377` → `main.rs:1483` (line 1377 is now `NumberedStepCard`); replace "the planned public `Footer` band, which has no component yet" with the shipped `Footer` component at `main.rs:1287` (its own §2.1 row already reads ✅); keep both frozen file names unchanged; commands: `grep -n "^fn Navbar(\|^fn Footer(" persona-sanus/src/main.rs`
- [ ] 3. Fix the INDEX header code-range and plan-path references — requirements: `Route` enum citation `persona-sanus/src/main.rs:47-95` → `persona-sanus/src/main.rs:61-110`; all three bare `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` references (header `Source`, §1.3 authority table, §5 `/dev/ui` row) → `context/done-plans/UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md`; commands: `grep -n "enum Route" persona-sanus/src/main.rs; ls context/done-plans/UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md`
- [ ] 4. Remove the duplicated ER edge in `context/database-design.md` §3 — requirements: delete one of the two identical `clinics ||--o{ assets : "image library"` lines (lines 98 and 101 on 2026-11-19), keeping the `clinics ||--o| assets : "logo"` edge and every other relation; commands: `grep -c 'clinics ||--o{ assets : "image library"' context/database-design.md` → `1` after the edit

## 2. No-go

- No route, component, status-cell, or copy change in any screen file under `context/screens/done/`; this tracker owns `context/screens/INDEX.md` and the one ER line only.
- Do not rename `Header.md` or `Footer.md` (INDEX §1.1 frozen exceptions).
- No code change in `persona-sanus/`; if a cited line cannot be reproduced from the current tree, STOP and append a divergence instead of guessing a number.
- No rich-text, WYSIWYG, or contenteditable editor (PRD D-21): all text entry is plain forms and inputs (`src/forms.rs` — single-line inputs, multi-line textareas, selects, check controls; plain-text values only), identical online and offline. No editor dependency, no HTML payloads, unless the spec's Source explicitly supersedes D-21.

## 3. Testing criteria

- [ ] `python3 .blocks/markdown-table-check.py --path context/screens/INDEX.md` passes (run from the repo root)
- [ ] objective 1: a per-component comparison of every INDEX §2 `` `main.rs` `` citation against `grep -n "^fn <Name>(" persona-sanus/src/main.rs` reports 0 drifted rows
- [ ] objective 2: `grep -n "1377\|no component yet" context/screens/INDEX.md` returns no matches
- [ ] objective 3: `grep -n "UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN" context/screens/INDEX.md` shows only the `context/done-plans/` path, and `grep -n "main.rs:47-95" context/screens/INDEX.md` returns no matches
- [ ] objective 4: `grep -c 'clinics ||--o{ assets : "image library"' context/database-design.md` → `1`
- [ ] every objective in section 1 is checked

## 4. Divergences

- [ ] every divergence found is appended below (or the section reads "None.")

### Divergences

1. The INDEX citations were stale on 2026-11-19 and no fix was applied during the audit: 15 of 23 component rows drifted −30…+14 lines, the §1.1 `Navbar` citation pointed at `NumberedStepCard`, the `Footer` note denied a shipped component, the `Route` range predated the current enum, and three plan references named a file that only exists under `context/done-plans/`. (Verified 2026-11-19: 0 fix attempts — per-row comparison of INDEX citations against `fn <Name>(` lines in `persona-sanus/src/main.rs`, `grep` for the plan filename and the duplicated ER edge — all fenced by the audit's report-only scope; this is a documentation-only correction, deferred to this tracker by design, not a defect in the shipped code.)

---

*Housekeeping: once every box above is checked, delete this file; the parent specs (15-O–25-Y) are already in `specs/done/` and stay there.*
