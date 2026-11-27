# Spec <ID> — <short title>

- Source: <document, section, issue, or finding that triggered this work>.
- Depends on: <spec IDs this builds on, or "nothing"; note any incomplete dependencies>.
- Project root: `<path>`. All paths and commands below are relative to it.
- Owned files: `<paths/globs the executing worker may modify>`.
- Gate: `<exact verification command that must exit 0, or "not applicable" with reason>`.
- User-interface verification: `<required|not applicable>` — <if required, surface, workflow, and evidence; otherwise, reason>.
- Result carried forward: <one sentence describing the observable end state>.

## 1. Problem and decision

- Problem: <who is affected, when, and the cost>.
- Decision: <chosen approach and why; if undecided, list viable options>.
- Open questions: <question — owner>, or "None".

## 2. Objectives and scope

*List the ordered changes and constraints. Check an objective only after its
acceptance criteria are verified.*

- [ ] 1. <objective> — requirements: <what must change and constraints>.
- [ ] 2. <objective> — requirements: <what must change and constraints>.

### Out of scope / no-go

- <forbidden action, file, package, or behavior>.
- <applicable policy or constraint, with a source reference>.

## 3. Acceptance and verification

*Describe observable outcomes and relevant edge cases. For each, specify the
verification command or manual workflow and the expected result. Completion
requires every applicable criterion and the gate to pass.*

- [ ] <observable outcome or edge case> — verification: `<command or workflow>`
  → <expected result>.
- [ ] Gate: `<command>` passes from the project root, or <why not applicable>.
- [ ] User-interface verification: <surface, workflow, and evidence>, or <why not applicable>.
- [ ] Every objective in section 2 is checked.

Choose the least costly verification that provides meaningful evidence: unit
tests for isolated behavior, integration tests for boundaries, and end-to-end
tests for critical workflows. Add unit tests when they cover consequential edge
cases or plausible user-visible failures that are otherwise difficult to catch.
Do not add tests merely to increase coverage or pass trivially: avoid assertions
that repeat the implementation, test wiring or incidental defaults, or only
check that nothing throws. Reuse existing tests where they provide the proof.

## 4. Execution and divergences

Read the applicable project instructions before executing. If reality differs
from the spec, or a necessary change falls outside the owned files, do not
improvise outside the objectives or modify unowned files. Record the divergence
below and stop the affected work; unaffected objectives may continue within
their scope. Request a decision from <scope-change approver> before changing
scope or ownership.

For each divergence, record the current state, evidence and attempts actually
made, impact, blocker, owner, and decision or follow-up needed. Leave affected
acceptance criteria unchecked and note the divergence beside them. Do not label
a failure non-defective merely because it is deferred.

Open divergences block completion until resolved or an authorized scope change
updates the objectives and acceptance criteria. Record approved deferrals and
their owner or later spec; follow the project's divergence tracking-file rules.
Incomplete dependencies block affected objectives unless the scope-change
approver explicitly decides otherwise.

### Divergences

<Numbered entries, or "None.">

*Housekeeping: archive completed specs according to
the project's conventions.*
