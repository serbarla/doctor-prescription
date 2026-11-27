<!-- caveman-blocks:start -->

BLOCKS. Scripts in this repo are reusable blocks in .blocks/. Rules:

1. Check the block index below first. If a block fits, run it. If one almost fits, add a param to it.
2. A script you would run twice, or over ~10 lines, becomes a block, not a heredoc or a /tmp file.
3. Inputs are params. No hardcoded paths, ids, ports or dates.
4. Print one small JSON object: the answer, not the data. Filter, count and truncate in code.
5. Exit 0 on success, non-zero with {"error": ...} on failure.
6. Safe to re-run: idempotent, no prompts, cleans up after itself.
7. Header first: name, summary, params, effects, example. caveman-blocks lint checks it.
8. Compose: call existing blocks with caveman-blocks run instead of copying their code.

asset-probe   --base <str> [--email …] [--password ProbePass123!] [--name "Probe Tester"] Probe image…
json-peek     --path <path> [--depth 2]         Shape of a JSON or JSONL file: keys, row count, one sample.…
markdown-table-check --path <path> [--glob *.md] [--skip fixtures/] [--report 5] [--strict] Verify every…
persona-docker-verify [--dir .] [--port 8080] [--email …] [--password VerifyPass123!] Verify compose stack:…
pdf-probe      --base <str> [--email …] [--password ProbePass123!] [--hvq] Probe PDF: magic, A4 MediaBox, filename, QR payload, draft 422; --hvq checks landscape MediaBox and HVQ text.
persona-probe --base <str> [--email …] [--password ProbePass123!] [--name "Probe Tester"] [--flow full|hvq|…] Probe persona-sanus …
persona-smoke [--dir persona-sanus] [--port 8091] [--secret <str>] [--db-path ""] E2E smoke:…
update-channel-verify [--repo .] [--port 8092] Integration probe: compiles the real update.rs/dev…
spec-queue    [--specs-dir specs] [--ready-only] [--fail-if-empty] [--brief] Classify unchecked specs and…
sqlite-schema-check [--path ""] [--db ""] [--block 0] [--query ""] [--limit 10] Run SQL from a Markdown…
wait-health   --url <str> [--expect 200] [--timeout 60] [--interval 2] Poll a URL until it answers with the…

<!-- caveman-blocks:end -->

## Context Docs

Read the matching file in `context/` before changing that area; they are the source of truth, code follows them.


| File                         | Check it when                                                                                                                                                          |
| ---------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `context/PRD.md`             | Scope, features, v1 vs v1.1 decisions, billing, non-goals — before adding or reshuffling product behaviour.                                                            |
| `context/architecture.md`    | System shape, crate/module layout, runtime and deploy topology, feature status — before adding a dependency, route, or service.                                        |
| `context/database-design.md` | Schema, ids, timestamps, immutability, PHI rules, migrations — before any table, column, or Diesel change.                                                             |
| `context/sitemap.md`         | Routes, layout shell, auth guards, status per screen — before adding or moving a page.                                                                                 |
| `context/screens/INDEX.md`   | Per-screen design specs in `context/screens/done/` — which exist, which are missing, and the shape each follows — before writing, renaming, or deleting a screen file. |
| `context/Design.md`          | Colors, typography, spacing tokens, component styling — before writing UI markup or CSS.                                                                               |
| `specs/done/`                | Where done specs are moved into after done with no divergences                                                                                                         |


## Coding

Do not leave comments in any code

## Take into account first

Whenever you are asked to plan something, please do not assume, ask the user first before implementing.

## Bug Fixing

You may try 15 times to fix a bug, then, add it to a Linear Issue, following the approved format Specs-Issue

## Language

- Docs/PRD content in English.
- Only references to Ecuador Spanish or specific Ecuador names stay in Spanish (e.g. `es-EC`, `receta`, `cédula`, Ley Orgánica de Protección de Datos Personales, MSP/ACESS).

## Spec-Driven Development

Work through numbered specs in `specs/` (e.g. `01-A-app-schema-and-db-runtime.md`), not ad hoc. A spec is the single source of truth for a bounded change.

- **Create**: copy `specs/TEMPLATE.md` to a sequential `specs/NN-X-<short-slug>.md`, matching the `specs/` listing. Fill every header field (`Source`, `Depends on`, `Project root`, `Owned files`, `Gate`, `User-interface verification`, `Result carried forward`) and sections 1–4. Define the problem, decision and open questions; ordered objectives and no-go constraints; and acceptance criteria with observable outcomes, edge cases, exact verification commands or workflows, and expected results. Specify why the gate or UI verification is not applicable when omitted.
- **Execute**: read the spec and applicable project instructions before starting. Implement only the objectives and modify only owned files; follow the no-go constraints. If reality differs or required work falls outside the owned files, do not improvise: record a divergence and stop affected work. Unaffected objectives may continue within scope. Request a decision from the scope-change approver before changing scope or ownership. Incomplete dependencies block affected objectives unless the approver explicitly decides otherwise.
- **Verify**: check an objective only after its acceptance criteria pass. Keep every criterion unchecked until verified. A spec is complete only when every applicable acceptance criterion and the gate pass, UI verification is complete when required, and every objective is checked.
- **Divergences**: in section 4 (`### Divergences`), record the current state, evidence and attempts actually made, impact, blocker, owner, and decision or follow-up needed. Leave affected criteria unchecked and note the divergence beside them. Open divergences block completion until resolved or an authorized scope change updates the objectives and criteria.