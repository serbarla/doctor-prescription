# Spec 31 — Phase AE: Global style, accessibility, and docs close

- Source: `context/Design.md` §§Tokens, Components, Do/Don't, Surfaces, Elevation, Layout; every `context/screens/done/*.md` Status row (§§5–6 targets); `context/screens/INDEX.md` §§1–2; `context/sitemap.md` §4 (milestones), §5; `specs/divergences/13-M-signed-installers.md` (OPEN fence, recorded here so it is not silently dropped).
- Depends on: 27-AA, 28-AB, 29-AC, 30-AD.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Owned files: `persona-sanus/assets/main.css`, `persona-sanus/src/main.rs`, `persona-sanus/src/ui/`, `context/screens/INDEX.md`, `context/screens/done/`, `context/sitemap.md`
- Gate: `cd persona-sanus && cargo fmt --check && cargo check && cargo clippy -- -D warnings && cargo check --no-default-features --features desktop && dx build --release --fullstack && caveman-blocks run persona-smoke --dir persona-sanus --port 8099 && caveman-blocks run persona-docker-verify --dir . --port 8080`
- Desktop-verify: yes
- Result carried forward: Every v1 screen satisfies the `Design.md` Do/Don't list and its screen-spec §5/§6 on web and macOS, all docs name the shipped state with no stale partial/planned row, and the remaining OPEN fences (v1.1 billing, signed installers, 26-Z gates) are recorded in their own trackers — not in screen statuses.

## Before you start

Unit testing is not required; avoid it. Prioritize end-to-end and integration testing only.

Read the nearest `AGENTS.md` (target workspace, then parents up to repo root) before executing.

If reality diverges from this spec: STOP, append a divergence (section 4), and NEVER improvise outside the objectives.

## 1. Objectives

*The ordered work to implement — one item per step. Check `[x]` only after the step is implemented and verified.*

*Item format:* `<verb phrase>` — requirements: `<what must change / constraints>`; commands: `<exact command>`.

- [x] 1. Audit the style System — requirements: in `persona-sanus/assets/main.css` and `persona-sanus/src/ui/`, verified against `context/Design.md`: all color/typography/spacing tokens defined and consumed via `var()` (replace the remaining literal hex/px outside the token layer and any literal outside `:root` with tokens); grenette at weight 400 for 46px+ headlines only, never bold, never body copy; mint for everything else; one `#006e75` filled primary per viewport; 8px radius controls, 24px cards; 1px `#e5e7eb` hairlines, never a chromatic card border; flat elevation (the single `var(--shadow-paper)` A4 lift only); centered hero/section content; the mint/cream/blossom A4 tri-tone rotation intact. Remove the last legacy hook if any remains. Do not add a selector; migrate markup to the `.ui-*` primitive. Commands: `cd persona-sanus && cargo check && cargo check --no-default-features --features desktop`
- [x] 2. Audit responsiveness and accessibility — requirements: at the `/dev/ui` gallery and one surface per screen group, verified against each screen spec §6: landmarks and heading order, tab order, visible focus ring (1px `#006e75`), labelled controls, contrast from the token pairs, and a usable narrow-width layout for the authenticated shell and the public screens. Close any delta in markup or CSS; no new breakpoint system beyond the existing one. Commands: `cd persona-sanus && cargo check && dx build --release --fullstack`
- [x] 3. Synchronise every doc with the shipped state — requirements: `context/screens/INDEX.md` §2 matches the `Route` enum and `context/sitemap.md` §1.1/§1.2 statuses; every `context/screens/done/*.md` Status row reads §1–§7-passed for the v1 scope; `SettingsBilling.md` stays deferred (v1.1) and is not counted as missing; `/dev/ui` stays documented as debug-only. Any row that cannot be honestly promoted keeps an exact remaining-delta sentence pointing at section 4. Commands: `caveman-blocks run markdown-table-check --path context/screens/INDEX.md && cd persona-sanus && cargo check`
- [ ] 4. Run the full end-to-end gates — requirements: the section 3 static, release, smoke, and docker-verify commands, plus a desktop launch smoke on macOS (the app opens, chrome and one authenticated screen render, offline indicator behaves) recorded in this spec's section 3. Do NOT claim a signed/notarized installer: `specs/divergences/13-M-signed-installers.md` stays OPEN and untouched. Commands: `cd persona-sanus && dx build --release --fullstack && caveman-blocks run persona-smoke --dir persona-sanus --port 8099 && caveman-blocks run persona-docker-verify --dir . --port 8080`

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No new visual language, selector system, page pattern, route, API, table, migration, or dependency. This spec removes and reconciles only what specs 27-AA–30-AD replaced or left stale.
- No `/settings/billing` surface or copy (v1.1, PRD D-04/D-14). No faking, self-signing, or claiming a signed installer — that fence is `specs/divergences/13-M-signed-installers.md` and clears only with real Apple/Windows credentials in the release process. No closing any 26-Z gate.
- No rich-text, WYSIWYG, or contenteditable editor (PRD D-21): all text entry is plain forms and inputs (`src/forms.rs` — single-line inputs, multi-line textareas, selects, check controls; plain-text values only), identical online and offline. No editor dependency, no HTML payloads, unless the spec's Source explicitly supersedes D-21.
- No pure black, drop shadow on UI cards/buttons, chromatic card border, bold grenette, free-form hex, or gradient beyond the ember ring and verification conic ring. No `.a4-*` geometry change (mm, page box, pagination, QR layout, print fonts).
- No comment in any code. No unit tests.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [x] `cd persona-sanus && cargo fmt --check && cargo check && cargo clippy -- -D warnings && cargo check --no-default-features --features desktop` exits 0 (run from the repo root) (cleared: 27-AA `server.rs` drift fixed; re-verified on final tree 2026-11-20, exit 0)
- [x] `cd persona-sanus && dx build --release --fullstack` exits 0
- [x] `caveman-blocks run persona-smoke --dir persona-sanus --port 8099` returns `{"ok": true}` with every `checks` value true; exit 0 (re-verified on final tree 2026-11-20: all 13 checks true)
- [x] `caveman-blocks run persona-docker-verify --dir . --port 8080` exits 0
- [x] objective 3: `caveman-blocks run markdown-table-check --path context/screens/INDEX.md` → exit 0
- [ ] every objective in section 1 is checked

## 4. Divergences

*Process and tracking-file rules: see `AGENTS.md` (§ Spec-Driven Development). Summary: when reality diverges from the spec, append a numbered entry below; leave the matching testing-criteria `- [ ]` unchecked and note the divergence inline next to it. Open divergences owned by a later spec also get a tracking file in `specs/divergences/`.*

- [x] every divergence found is appended below (or the section reads "None.")

### Divergences

1. **§3 boxes 1 and 3 blocked — `cargo fmt --check` and `persona-smoke` fail only on pre-existing `persona-sanus/src/server.rs` fmt drift (lines 198, 254, 274), a file outside this spec's Owned files; all owned files are fmt-clean.** Verified 2026-11-20: `cargo fmt --check` reports a diff in `server.rs` only; `cargo check`, `cargo clippy -- -D warnings`, and `cargo check --no-default-features --features desktop` each exit 0; `persona-smoke --port 8099` returns `{"checks": {"fmt": false, "check": true, "clippy": true, "build": true}}` exit 1. Why fenced: this spec's No-go forbids touching files outside Owned files, and reformatting `server.rs` belongs to the spec that takes it in. This was 27-AA tracking-file divergence 1 (`specs/divergences/27-AA-open-deltas.md`, since removed); section 3 box 1 records it cleared on the final tree (`cargo fmt --check` exit 0 after worker 27 cleared the drift), so this entry is closed.
2. **Objective 4's desktop launch smoke is partial: the raw debug desktop binary opens and stays alive (10s, no crash) and renders the `/` Home route, but plain `cargo run --no-default-features --features desktop` does not stage `/assets/main.css`, so the window renders unstyled (screenshot evidence) and "chrome and one authenticated screen render, offline indicator behaves" could not be visually verified.** Verified 2026-11-20: two launches (second with `target/debug/assets/` staged beside the binary) both render the Home content with UA-default styling. Why fenced: the launch method is outside Owned files and No-go scope (no packaging/asset change); the styled desktop path needs the dx-packaged bundle. No signed/notarized installer is claimed; `specs/divergences/13-M-signed-installers.md` stays OPEN and untouched.
 Open divergence 2 moved to spec 33-AG (2026-11-20) with a bundle identifier and launch proof; when spec 33-AG records section 3 evidence, objective 4 can be checked.
3. **Gate port divergence: `--port 8100` is unreachable — `docker-compose.yml` publishes only `8080:8080` and the `persona-docker-verify` block probes the given host port without remapping, so no run at 8100 can ever answer `/health` through no app defect.** Verified 2026-11-20: `--port 8100` exits 1 with `{"error": "container never healthy", "checks": {"built": true, "deployed": true, "health": false}}` while `curl /health` returns 200 on 8080 (container itself `(healthy)`); the identical block at `--port 8080` exits 0 with `{"ok": true}` and all 27 checks true. Why fenced: changing the mapping needs `docker-compose.yml`, and changing the block needs `.blocks/` — both outside this spec's Owned files and No-go scope; every sibling spec (13-M, 26-Z) uses `--port 8080`, so 8100 is a gate typo. The stack is verified end-to-end at 8080; the box above stays unchecked until a spec that owns `docker-compose.yml`/`.blocks/` corrects the gate port.
 Divergences 3 and 4 were both cleared 2026-11-20 on the final tree (gate port 8080, AuthPage one-liner); the shared tracker `specs/divergences/31-AE-desktop-launch-and-gate.md` was merged into spec 33-AG and deleted.
4. **`AuthPage.md` Status still claims the sitemap leaves MFA/device registration at M3 ⬜; the sitemap §4 M3 row already reads MFA ✅ and device ✅ (spec 04-D), so the clause is stale.** Found at audit time (2026-11-20); the row itself correctly reads ✅ Spec complete (§§1–7). One-line doc edit clears it — any spec that touches `AuthPage.md`. Tracked in `specs/divergences/31-AE-desktop-launch-and-gate.md` divergence 4.
