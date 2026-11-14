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
pdf-probe      --base <str> [--email …] [--password ProbePass123!] Probe PDF: magic, A4 MediaBox, filename, QR payload, draft 422.
persona-probe --base <str> [--email …] [--password ProbePass123!] [--name "Probe Tester"] [--flow full]…
persona-smoke [--dir persona-sanus] [--port 8091] [--secret <str>] [--db-path ""] E2E smoke:…
sqlite-schema-check [--path ""] [--db ""] [--block 0] [--query ""] [--limit 10] Run SQL from a Markdown…
wait-health   --url <str> [--expect 200] [--timeout 60] [--interval 2] Poll a URL until it answers with the…

<!-- caveman-blocks:end -->

## Context Docs

Read the matching file in `context/` before changing that area; they are the source of truth, code follows them.


| File                         | Check it when                                                                                                                   |
| ---------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| `context/PRD.md`             | Scope, features, v1 vs v1.1 decisions, billing, non-goals — before adding or reshuffling product behaviour.                     |
| `context/architecture.md`    | System shape, crate/module layout, runtime and deploy topology, feature status — before adding a dependency, route, or service. |
| `context/database-design.md` | Schema, ids, timestamps, immutability, PHI rules, migrations — before any table, column, or Diesel change.                      |
| `context/sitemap.md`         | Routes, layout shell, auth guards, status per screen — before adding or moving a page.                                          |
| `context/Design.md`          | Colors, typography, spacing tokens, component styling — before writing UI markup or CSS.                                        |


## Coding

Do not leave comments in any code

## Testing

Unit testing is not required, avoid it, only end to end testing and integration testing are priorities

## Take into account first

Whenever you are asked to plan something, please do not assume, ask the user first before implementing.

## Bug Fixing

You may try 15 times to fix a bug, then, add it to a Linear Issue, following the approved format Specs-Issue

## Language

- Docs/PRD content in English.
- Only references to Ecuador Spanish or specific Ecuador names stay in Spanish (e.g. `es-EC`, `receta`, `cédula`, Ley Orgánica de Protección de Datos Personales, MSP/ACESS).

## Spec-Driven Development

Work is planned and executed through numbered spec files in `specs/` (e.g. `01-A-app-schema-and-db-runtime.md`), not ad hoc. A spec is the single source of truth for a bounded change; do NOT improvise outside it.

- **Create**: copy `specs/TEMPLATE.md` into a new `specs/NN-X-<short-slug>.md` (sequential number + phase letter, matching the `specs/` listing). Fill in the header (`Source`, `Depends on`, `Repo root`, `Result carried forward`), then Objectives, No-go, and Testing criteria. Every objective is a single verb sentence with its requirements and exact commands, checked off with `- [x]` when done.
- **Execute**: read the spec AND the nearest `AGENTS.md` (target workspace, then parents up to repo root) before starting. Implement only what the Objectives require; the No-go section lists what you MUST NOT do. If reality diverges from the spec, STOP and append a divergence — never silently drop it or rewrite objectives to match what you actually did.
- **Verify**: all Testing-criteria boxes checked = spec done. Keep every `- [ ]` box unchecked until it passes.
- **Divergences**: record each at the end of the spec (section 4, `### Divergences`) — what diverged, why, the fix attempts, and the impact. If a divergence stays OPEN because a later spec owns the fix, also create a tracking file in `specs/divergences/<spec-id>-<short-slug>.md` (source spec + the exact criteria it blocks, what's broken, why each fix was fenced, and a "how to clear it" checklist naming the resolving spec and re-verify commands). Link the tracking file from the spec's divergence entry. When the later spec lands and checks pass, mark the criteria `[x]` and delete the tracking file. See `specs/TEMPLATE.md` §4 for the canonical divergence format.

## Subagent Delegation

When generating subagents you may first plan the task to be completed into separate isolated phases independent from other phases, that you will put together after subagents workflows are complete, use the following omp model presets:




| Model preset                                | Policy                                                                                                                                                                               |
| ------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `deepflash-medium`, `ember-medium-subagent` | Keep subagent context low; 1 task per agent, then delete. Keep cost under $1 daily.                                                                                                  |
| `qwen-single-agent`                         | Run only 1 subagent at a time; never more than 1 under any circumstance. May or may not be available; if reachable, it is free its your priority to run it correctly when available. |
| `muse-wonder-planner`                       | NEVER use for subagents; not allowed. It orchestrates modular isolated tasks, generating worktrees and subagents.                                                                    |
