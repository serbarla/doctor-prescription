# Divergence tracking: 13-M PRD §12 acceptance evidence

Source spec: `specs/13-M-ops-and-release.md`, objective 5 ("walk the `context/PRD.md` §12 acceptance checklist and record each item's evidence") and its testing criterion "objective 5: every `context/PRD.md` §12 acceptance criterion has recorded evidence (checked box + where it was proven)" — both marked `[x]`.
Resolving spec: none — requires re-walking §12 in a remediation pass. 13-M stays in `specs/`.

Severity: P2 — the release acceptance checklist is claimed proven in 13-M, but the checklist itself shows no proof. A reader of `context/PRD.md` §12 cannot tell which criteria pass, and one criterion cannot pass yet.

## What's broken

- `context/PRD.md:249-263` (§12): all 13 acceptance boxes are `- [ ]`. Verified 2026-11-19: 0 checked, 13 unchecked. No evidence links, spec references, or probe names are recorded next to any criterion in the PRD.
- The last criterion, "Auto-update applies only a manifest with a valid signature and rolls back on a bad signature or failed launch", depends on 13-M objective 4, which is unchecked, and on `specs/divergences/13-M-signed-installers.md` (OPEN, external signing credentials). It cannot have recorded passing evidence while that tracker is open.
- 13-M §3 marks the evidence criterion `[x]` with no pointer to where the walk lives. If the walk exists, it is not in `context/PRD.md` §12, `context/runbook.md`, or 13-M §3/§4 as checked on 2026-11-19.

## Why each fix was fenced

- This audit is report-only: checking PRD boxes requires re-running the E2E flows (offline desktop sign/sync, QR scan, retention trim, update rollback), not editing checkboxes on prior claims.
- The auto-update criterion is fenced by the same external prerequisite as 13-M objective 4 (Apple Developer ID + Windows code-signing certificate, 13-M §2 No-go forbids faking it).

Verified 2026-11-19: 0 fix attempts — checkbox count on `context/PRD.md` §12 (0/13) read against 13-M objectives 4–5 and §3. Fenced by the missing E2E re-walk and the signing-credentials prerequisite.

## How to clear it

- [ ] Re-run or cite, per §12 criterion, the exact probe/spec evidence (`.blocks/persona-probe.py`, `pdf-probe.py`, `persona-docker-verify.py`, desktop offline flow) and check the box in `context/PRD.md` §12 only where the evidence passes today.
- [ ] Leave the auto-update criterion unchecked until `13-M-signed-installers.md` clears; note the blocker inline next to it.
- [ ] Add the evidence pointer (file + command + date) next to each checked criterion, or link the walk document from 13-M §3.
- [ ] Reconcile 13-M objective 5 / its §3 criterion: uncheck them until the PRD walk above is recorded.
- [ ] Delete this file once every §12 box is either checked with evidence or explicitly marked blocked with its tracker.
