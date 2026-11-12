# Spec NN — Phase X: <short title>

- Source: <which document and section this came from — e.g. a plan file, an
  issue, or an audit finding — and what triggered it>.
- Depends on: <spec IDs this builds on, or "nothing">.
- Repo root: `<path>`. Every path and command below is relative to it.
- Result carried forward: <one sentence: the observable end state once this spec is done>.

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

- [ ] 1. <objective> — requirements: <requirements>; commands: `<command>`
- [ ] 2. <objective> — requirements: <requirements>; commands: `<command>`

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- <forbidden action / file / package>

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [ ] `<global smoke / end-to-end command for the whole change>` passes (run
  from the repo root)
- [ ] objective N: `<exact command from section 1>` → `<expected output>`
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

*Fill in at execution time. Generic example (same slots as the entry format
above):*

1. `<check>` stays failing in `<workspace>` on the `<feature>` feature only:
   the `<work added by objective N>` lands, but the `<code it calls>` is
   explicitly excluded (section 2, spec <NN-Y>). (Verified <date>: <N> fix
   attempts — <attempt 1>, <attempt 2>, <attempt 3> — all fenced by
   <constraint>; `<what stays failing>` is deferred to spec <NN-Y> by design,
   not a defect.)

---

*Housekeeping: once a spec is fully done with no open divergences, move its
file to `specs/done/` (same file name, e.g.
`specs/done/NN-X-<short-slug>.md`).
Specs with open divergences stay in `specs/` next to `TEMPLATE.md` until the
tracking file is cleared.*
