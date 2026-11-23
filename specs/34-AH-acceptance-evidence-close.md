# Spec 34 — Phase AH: 13-M acceptance-evidence close

- Source: `specs/divergences/13-M-compose-production-config.md` and `specs/divergences/13-M-es-ec-copy.md` (merged 2026-11-20; both trackers deleted at this spec's creation — every unchecked How-to-clear box of either tracker is restated below as an objective, so no box is dropped); `specs/13-M-ops-and-release.md` objective 5 and its section 3 acceptance criterion.
- Depends on: 12-L, 33-AG.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Owned files: `docker-compose.yml`, `.env.example`, `persona-sanus/src/server.rs`, `persona-sanus/src/pdf_server.rs`, `context/runbook.md`, `persona-sanus/README.md`, `persona-sanus/src/main.rs`, `persona-sanus/src/auth_server.rs`, `persona-sanus/src/service.rs`, `persona-sanus/src/http.rs`, `persona-sanus/src/api_server.rs`, `persona-sanus/src/api.rs`, `persona-sanus/src/forms.rs`, `specs/34-AH-acceptance-evidence-close.md`
- Gate: `caveman-blocks run persona-docker-verify --dir . --port 8080`
- Desktop-verify: no
- Result carried forward: the compose production config is proven behind the runbook TLS proxy (Secure cookie, image render, QR decode) and the es-EC browser walk proves every visible message, with evidence recorded here so `specs/13-M-ops-and-release.md` objective 5 can be checked by spec 35-AI.

## Before you start

Unit testing is not required; avoid it. Prioritize end-to-end and integration testing only.

Read the nearest `AGENTS.md` (target workspace, then parents up to repo root) before executing.

If reality diverges from this spec: STOP, append a divergence (section 4), and NEVER improvise outside the objectives.

## 1. Objectives

*The ordered work to implement — one item per step. Check `[x]` only after the step is implemented and verified.*

*Item format:* `<verb phrase>` — requirements: `<what must change / constraints>`; commands: `<exact command>`.

- [ ] 1. Prove the compose production config behind the TLS proxy — requirements: with a production `.env` built from `.env.example` (HTTPS `PUBLIC_BASE_URL`, `PERSONA_COOKIE_SECURE=true`, browser-reachable `ASSETS_PUBLIC_ENDPOINT`, MinIO credentials; never commit `.env`) and the runbook section 1 Caddy/Nginx topology fronting compose, verify from a separate client of the stack: sign-in returns a `Set-Cookie` containing `Secure; HttpOnly; SameSite=Lax`, an uploaded image renders from the signed URL, and a generated PDF's QR decodes to `https://<PUBLIC_BASE_URL>/verify/<code>`; if a second LAN machine and a real TLS host become available, rerun the same three checks there, otherwise record the residual second-machine half as a section 4 divergence citing the named prerequisite; no invented domain, certificate, or committed secret; commands: `caveman-blocks run persona-docker-verify --dir . --port 8080`
- [ ] 2. Complete the es-EC browser walk — requirements: execute the merged tracker's browser list (signed-out Home navbar, `/login` sign-in and sign-up, wrong email/password, short password, MFA challenge and wrong code against a doctor fixture with two-step enrolled, signed-in Dashboard, Navbar sign-out, `/audit`, `/patients/new`, `/templates/new`, `/prescriptions/new`, offline desktop message, 403 role message, 413 upload with MinIO provisioned, 429 rate-limit) and assert every visible message is es-EC while machine `code`/`action` values stay unchanged; the audit `python3 .blocks/es-ec-copy-audit.py --src persona-sanus/src` must report `ui_hit_count: 0` and `persona-probe --flow roles` must stay green on a fresh DB; commands: `caveman-blocks run persona-smoke --dir persona-sanus --port 8099`
- [ ] 3. Record the acceptance evidence in this spec — requirements: run the compose tracker's gate set (`persona-docker-verify --port 8080`, `pdf-probe`, `persona-probe --flow verify`, each on a fresh DB where the probe design requires it) and attach the outputs to this spec's section 3; do not edit `specs/13-M-ops-and-release.md` (spec 35-AI owns the objective 5 check, including the auto-update criterion that stays behind the signed-installer fence); commands: `caveman-blocks run persona-docker-verify --dir . --port 8080`

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No editing `persona-sanus/src/update.rs`, `persona-sanus/src/jobs.rs`, `persona-sanus/src/device.rs`, `specs/13-M-ops-and-release.md`, `specs/33-AG-*`, `specs/35-AI-*`, `specs/36-AJ-*`, or anything outside Owned files. No new table, migration, route, dependency, or unit test.
- No inventing a public domain, TLS certificate, or production secret; no committing a real `.env`. No faking the gate (self-signed, unsigned-with-note, or skipped LAN step claimed as passed).
- No rich-text, WYSIWYG, or contenteditable editor (PRD D-21): all text entry is plain forms and inputs (`src/forms.rs` — single-line inputs, multi-line textareas, selects, check controls; plain-text values only), identical online and offline. No editor dependency, no HTML payloads, unless the spec's Source explicitly supersedes D-21.
- No comment in any code.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [ ] `caveman-blocks run persona-docker-verify --dir . --port 8080` returns `{"ok": true}` with every `checks` value true; exit 0
- [ ] objective 1: the three proxy-fronted checks observed (Secure cookie, rendered image, QR decoding to the HTTPS verify URL) and recorded here, or the residual second-machine half named as a section 4 OPEN divergence
- [ ] objective 2: `python3 .blocks/es-ec-copy-audit.py --src persona-sanus/src` reports `ui_hit_count: 0` and the browser walk list renders with no English UI hits
- [ ] objective 3: this section records the docker-verify, pdf-probe, and persona-probe verify outputs
- [ ] every objective in section 1 is checked

## 4. Divergences

*Process and tracking-file rules: see `AGENTS.md` (§ Spec-Driven Development). Summary: when reality diverges from the spec, append a numbered entry below; leave the matching testing-criteria `- [ ]` unchecked and note the divergence inline next to it. Open divergences owned by a later spec also get a tracking file in `specs/divergences/`.*

- [ ] every divergence found is appended below (or the section reads "None.")

*Entry format (one numbered item per divergence):*

1. <what diverged and its current state> — <why it could not be done as written, citing the No-go rule or spec that blocks it> (Verified <date>: <N> fix attempts — <attempt 1>, <attempt 2>, <attempt 3> — all fenced by <constraint>; <impact> is deferred to <later spec> by design, not a defect.)

### Divergences

None yet.
