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

json-peek     --path <path> [--depth 2]         Shape of a JSON or JSONL file: keys, row count, one sample.…
persona-docker-verify [--dir .] [--port 8080] [--email …] [--password VerifyPass123!] Verify compose stack:…
persona-probe --base <str> [--email …] [--password ProbePass123!] [--name "Probe Tester"] [--flow full]…
sqlite-schema-check --path <path> [--block 0] [--query ""] [--limit 10] Run a Markdown fenced SQL block in…
wait-health   --url <str> [--expect 200] [--timeout 60] [--interval 2] Poll a URL until it answers with the…
<!-- caveman-blocks:end -->

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

## Subagent Delegation
When generating subagents you may first plan the task to be completed into separate isolated phases independent from other phases, that you will put together after subagents workflows are complete, use the following omp model presets:

| Model preset | Policy |
|---|---|
| `deepflash-medium`, `ember-medium-subagent` | Keep subagent context low; 1 task per agent, then delete. Keep cost under $1 daily. |
| `qwen-single-agent` | Run only 1 subagent at a time; never more than 1 under any circumstance. May or may not be available; if reachable, it is free its your priority to run it correctly when available. |
| `muse-wonder-planner` | NEVER use for subagents; not allowed. It orchestrates modular isolated tasks, generating worktrees and subagents. |
