# Spec 32 — Phase AF: Footer reconciliation and proof

- Source: `context/screens/done/Footer.md` §§1–8 (header Status at `:11` says "not shipped" and §8 says no spec adopts it; §7 records the shipped component, call sites, and the curl proof); `context/screens/INDEX.md` §2.1 (Footer ✅); `context/sitemap.md` §1.1.
- Depends on: 31-AE.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Owned files: `persona-sanus/src/main.rs`, `persona-sanus/assets/main.css`, `context/screens/done/Footer.md`, `context/screens/INDEX.md`
- Gate: `cd persona-sanus && cargo check && cargo check --no-default-features --features desktop && caveman-blocks run persona-smoke --dir persona-sanus --port 8101`
- Desktop-verify: yes
- Result carried forward: `Footer.md` reads shipped end to end (header Status, §7, §8), the §2.1 surface matrix passes its exact curl counts on web and the desktop version line renders, and anything that fails becomes section 4 — never a silent "missing".

## Before you start

Unit testing is not required; avoid it. Prioritize end-to-end and integration testing only.

Read the nearest `AGENTS.md` (target workspace, then parents up to repo root) before executing.

If reality diverges from this spec: STOP, append a divergence (section 4), and NEVER improvise outside the objectives.

## 1. Objectives

*The ordered work to implement — one item per step. Check `[x]` only after the step is implemented and verified.*

*Item format:* `<verb phrase>` — requirements: `<what must change / constraints>`; commands: `<exact command>`.

- [x] 1. Verify the shipped claims — requirements: confirm in code before asserting anything: `Footer` at `persona-sanus/src/main.rs:1289` with `enum FooterSurface { Home, Auth, Verify }`, call sites at `Home` (`:1416`), `Verify` (`:1471`), and the signed-out branch of `AuthPage` (`:1934`); `.ui-footer*` at `persona-sanus/assets/main.css:2115`; desktop-only version line via `#[cfg(feature = "desktop")]` (`:1290`). If any claim in `Footer.md` §7 does not match the code, stop and record it in section 4 instead of editing the doc to match a wrong claim. Commands: `cd persona-sanus && cargo check && cargo check --no-default-features --features desktop`
- [x] 2. Run the exact §7 proof — requirements: with the smoke server up, run every count in `Footer.md` §7 verbatim: `/` contains `id="ui-footer"` once, `href="#verificar"` twice, exactly one `ui-button--primary`, the law line exactly twice (never from the footer); `/login` contains the law line exactly once and inside `ui-footer`; `/verify/PS-TEST` contains the law line exactly once and the disclosure line exactly once (in the footer); `/dashboard` contains no `ui-footer`; the web bundle contains no `Versión ` string; a desktop build renders one version line per public page. Close any delta found in `Footer` markup or `.ui-footer*` CSS (surface matrix, copy, tokens, geometry, focus) with the §2.1 matrix as the only branch; no new primitive. Commands: `caveman-blocks run persona-smoke --dir persona-sanus --port 8101`
- [x] 3. Reconcile the stale rows — requirements: update `context/screens/done/Footer.md` so it reads shipped end to end: header Status (`:11`) to ✅ with the real component/call-site references, the §8 row that says no spec adopts `Footer` to name this spec, and any other "planned" wording. Confirm `context/screens/INDEX.md` §2.1 already shows ✅ and leave it if so. Do not edit the frozen copy (§3), structure (§2), style (§5), or accessibility (§6) sections to force a pass. Commands: `caveman-blocks run markdown-table-check --path context/screens/done/Footer.md && caveman-blocks run markdown-table-check --path context/screens/INDEX.md`

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No new route, endpoint, crate, network font, CDN, icon package, or JS bridge. No `/privacidad`, `/terminos`, `/contacto`, `/arco` link — no such route exists.
- No pricing/trial band, partner logo strip, newsletter/email field, social/app-store badges, language switcher, second filled teal action, `LinkAction` or button-shaped link, second `LogoLockup` on `/verify/:code`, hand-rolled law-line variant, or sticky footer (§7 Do-not-add list).
- No second `TrustSecurityRow` on a page whose body renders one; no duplicate disclosure line; no version line on web. No PHI, guarantee, or billing copy (D-04).
- No rich-text, WYSIWYG, or contenteditable editor (PRD D-21): all text entry is plain forms and inputs (`src/forms.rs` — single-line inputs, multi-line textareas, selects, check controls; plain-text values only), identical online and offline. No editor dependency, no HTML payloads, unless the spec's Source explicitly supersedes D-21.
- No comment in any code. No free-form hex; `.ui-footer*` stays token-only. No unit tests.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [x] `cd persona-sanus && cargo fmt --check && cargo check && cargo clippy -- -D warnings && cargo check --no-default-features --features desktop` exits 0 (run from the repo root) — CLEARED 2026-11-20: all exit 0 after worker 27 cleared server.rs fmt drift (divergence 2).
- [x] `caveman-blocks run persona-smoke --dir persona-sanus --port 8101` returns `{"ok": true}` with every `checks` value true; exit 0 — CLEARED 2026-11-20: `{"ok": true}` exit 0 (divergence 2).
- [x] objective 2: the six §7 curl counts above observed exactly, including `curl -s http://127.0.0.1:<port>/dashboard` showing no `ui-footer` and the web public pages showing no `Versión ` string
- [x] objective 3: `caveman-blocks run markdown-table-check --path context/screens/done/Footer.md` → exit 0
- [x] every objective in section 1 is checked

## 4. Divergences

*Process and tracking-file rules: see `AGENTS.md` (§ Spec-Driven Development). Summary: when reality diverges from the spec, append a numbered entry below; leave the matching testing-criteria `- [ ]` unchecked and note the divergence inline next to it. Open divergences owned by a later spec also get a tracking file in `specs/divergences/`.*

- [x] every divergence found is appended below (or the section reads "None.")

### Divergences

1. `Footer.md` §7/header line references had drifted 1–4 lines from the code (e.g. `Footer` `:1287` vs actual `:1289`, `Verify` call site `:1469` vs actual `:1473`, cfg `:1289` vs actual `:1290`, CSS `:2115` vs actual `:2116`, layout marker `:59-60` vs actual `:64-68`; the spec Source line also misdescribed `:11` as "not shipped" when it already read ✅ Shipped, while the Component row `:6` still said "not shipped"). All substantive §7 claims — component, enum variants, three call sites, CSS classes, `#[cfg(feature = "desktop")]` gate, copy strings — matched the code exactly (verified 2026-11-20). Resolved in objective 3 by correcting the header, §7, and §8 references to the real lines; no claim was edited to match a wrong fact.
2. The §3 `cargo fmt --check` and `persona-smoke` boxes stay failing on pre-existing `persona-sanus/src/server.rs` fmt drift (diffs at `server.rs:198/254/274`), already tracked as divergence 1 in `specs/divergences/27-AA-open-deltas.md`. `server.rs` is not an owned file of this spec, so it was not edited (Verified 2026-11-20: 1 attempt — `cargo fmt --check` run read-only to confirm the drift is confined to `server.rs`, while `cargo check`, `cargo clippy -- -D warnings`, and `cargo check --no-default-features --features desktop` all exit 0 — fenced by section 6 owned files; the fmt/smoke boxes are deferred to the owner of `specs/divergences/27-AA-open-deltas.md` divergence 1 by design, not a defect of this spec. The release server binary built by smoke was started manually on port 8101 so objective 2's curl counts were still observed exactly.) Cleared 2026-11-20 (worker 32): `cargo fmt --check`, `cargo check`, `cargo clippy -- -D warnings`, and `cargo check --no-default-features --features desktop` all exit 0, and `persona-smoke --port 8101` returns `{"ok": true}` with exit 0, so both §3 fmt/smoke boxes are checked; verification date 2026-11-20.
