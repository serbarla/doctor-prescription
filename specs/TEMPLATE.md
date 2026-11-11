# Spec NN — Phase X: <short title>

- Source: <where this spec came from — plan file + section, or audit finding>.
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

- [ ] `bun run build && bun run test` passes (repo root)
- [ ] objective N: `<command>` → `<expected output>`
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

*Fill in at execution time. Example (from spec 02-B, kept as reference):*

1. Workspace `typecheck`/`lint` stay red in `@acme/api` only: its
   `@acme/db/client`, `@acme/db/schema`, and `desc`/`eq` imports are removed by
   objectives 1 and 7, but rewriting them is explicitly excluded (section 2,
   spec 04). … api red is deferred to spec 04 by design, not a defect.)

---

*Housekeeping: once a spec is fully done with no open divergences, move its
file to `specs/done/` (e.g. `specs/done/01-A-nx-in-turborepo-nextjs-tanstack-out.md`).
Specs with open divergences stay in `specs/` next to `TEMPLATE.md` until the
tracking file is cleared.*
