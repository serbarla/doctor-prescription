# Spec 33 — Phase AG: Desktop launch proof close

- Source: `specs/31-AE-style-a11y-and-docs-close.md` objective 4 and section 4 divergences 2–4; `specs/divergences/31-AE-desktop-launch-and-gate.md` (divergences 3 and 4 cleared 2026-11-20 per its update: gate port 8080 corrected in the 31-AE header, objective 4, and section 3, `AuthPage.md` M3 clause fixed; divergence 2 remains as objective 2 below).
- Depends on: 32-AF.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Owned files: `persona-sanus/Dioxus.toml`, `specs/31-AE-style-a11y-and-docs-close.md`, `specs/33-AG-desktop-launch-close.md`
- Gate: `cd persona-sanus && dx bundle --desktop --package-types macos --release && cargo check --no-default-features --features desktop`
- Desktop-verify: yes
- Result carried forward: the dx-packaged macOS bundle launches styled, one authenticated screen renders via `Panel`, the offline indicator is observed, evidence sits in 31-AE section 3, and 31-AE objective 4 is checked and 31-AE moves to `specs/done/`.

## Before you start

Unit testing is not required; avoid it. Prioritize end-to-end and integration testing only.

Read the nearest `AGENTS.md` (target workspace, then parents up to repo root) before executing.

If reality diverges from this spec: STOP, append a divergence (section 4), and NEVER improvise outside the objectives.

## 1. Objectives

*The ordered work to implement — one item per step. Check `[x]` only after the step is implemented and verified.*

*Item format:* `<verb phrase>` — requirements: `<what must change / constraints>`; commands: `<exact command>`.

- [ ] 1. Provide a macOS bundle identifier — requirements: in `persona-sanus/Dioxus.toml` add the bundle identifier `com.doctorprescriptions.persona` under the macOS bundle section, so `dx bundle --desktop --package-types macos --release` no longer fails with `Bundle identifier was not provided in Dioxus.toml`. No other `Dioxus.toml` field changes, and no signing or notarization is claimed here (spec 35-AI owns the release certificates); commands: `cd persona-sanus && dx bundle --desktop --package-types macos --release`
- [ ] 2. Launch the packaged bundle to one authenticated screen — requirements: build the bundle, launch `target/dx/persona-sanus/release/macos/PersonaSanus.app` on macOS, restore or complete a signed-in session (the shipped `sync_meta` session restore path the tracker already proved is acceptable; interactive GUI sign-in only via Computer Use when the assistive grant exists), navigate `Panel` to `/dashboard` and observe the authenticated screen render, then stop the Docker stack and observe the `Sin conexión` offline indicator and authenticated navbar with the server stopped; record the observed states and screenshot paths in `specs/31-AE-style-a11y-and-docs-close.md` section 3; if the `Panel` click still cannot be driven in this environment, stop, append a section 4 divergence naming the missing grant, and check nothing; commands: `cd persona-sanus && dx bundle --desktop --package-types macos --release`
- [ ] 3. Check off 31-AE objective 4 and housekeeping — requirements: only after objective 2's evidence is recorded, check `specs/31-AE-style-a11y-and-docs-close.md` objective 4 and its section 3 `every objective in section 1 is checked` box, then move that file to `specs/done/31-AE-style-a11y-and-docs-close.md`; commands: `cd persona-sanus && cargo check --no-default-features --features desktop`

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No signed, notarized, self-signed, or fake-signed installer claim: `specs/divergences/13-M-signed-installers.md` (now spec 35-AI) stays OPEN and untouched; the distributable-installer step is not part of this spec's gate.
- No editing `docker-compose.yml`, `.blocks/`, `persona-sanus/src/server.rs`, or any file outside Owned files. No new route, API, table, migration, dependency, or visual language.
- No closing any 26-Z gate, no 13-M objective edits. No rich-text, WYSIWYG, or contenteditable editor (PRD D-21): all text entry is plain forms and inputs (`src/forms.rs` — single-line inputs, multi-line textareas, selects, check controls; plain-text values only), identical online and offline. No editor dependency, no HTML payloads, unless the spec's Source explicitly supersedes D-21.
- No comment in any code. No unit tests.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [ ] `cd persona-sanus && dx bundle --desktop --package-types macos --release` exits 0 and stages `PersonaSanus.app` with styled `main-*.css` under `Contents/Resources/assets/`
- [ ] objective 2: section 3 of `specs/31-AE-style-a11y-and-docs-close.md` records the launched bundle rendering the `/dashboard` authenticated screen and the `Sin conexión` offline indicator with the stack stopped
- [ ] `cd persona-sanus && cargo check --no-default-features --features desktop` exits 0
- [ ] every objective in section 1 is checked

## 4. Divergences

*Process and tracking-file rules: see `AGENTS.md` (§ Spec-Driven Development). Summary: when reality diverges from the spec, append a numbered entry below; leave the matching testing-criteria `- [ ]` unchecked and note the divergence inline next to it. Open divergences owned by a later spec also get a tracking file in `specs/divergences/`.*

- [ ] every divergence found is appended below (or the section reads "None.")

*Entry format (one numbered item per divergence):*

1. <what diverged and its current state> — <why it could not be done as written, citing the No-go rule or spec that blocks it> (Verified <date>: <N> fix attempts — <attempt 1>, <attempt 2>, <attempt 3> — all fenced by <constraint>; <impact> is deferred to <later spec> by design, not a defect.)

### Divergences

None yet.
