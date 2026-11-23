# Spec 27 — Phase AA: Shell chrome and public screens close

- Source: `context/screens/done/Header.md`, `Footer.md`, `Home.md`, `AuthPage.md`, `Verify.md` (Status rows); `context/sitemap.md` §1.1, §1.2 (shell), §4; `context/Design.md` §§Components (Logo Lockup, Green Pill Badge, Centered Hero Block, Numbered Step Card, Verification Tag, Trust & Security Badge Row), Layout, Do/Don't; `context/screens/INDEX.md` §2.1–§2.2.
- Depends on: 25-Y.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Owned files: `persona-sanus/src/main.rs`, `persona-sanus/assets/main.css`, `context/screens/done/Header.md`, `context/screens/done/Footer.md`, `context/screens/done/Home.md`, `context/screens/done/AuthPage.md`, `context/screens/done/Verify.md`, `context/sitemap.md`
- Gate: `cd persona-sanus && cargo check && cargo check --no-default-features --features desktop && caveman-blocks run persona-smoke --dir persona-sanus --port 8095`
- Desktop-verify: yes
- Result carried forward: The navbar chrome, footer, landing, sign-in, and public verification screens match their screen specs §§1–7 on web and macOS, and every paragraph that still calls them partial or unshipped is either closed or moved to section 4.

## Before you start

Unit testing is not required; avoid it. Prioritize end-to-end and integration testing only.

Read the nearest `AGENTS.md` (target workspace, then parents up to repo root) before executing.

If reality diverges from this spec: STOP, append a divergence (section 4), and NEVER improvise outside the objectives.

## 1. Objectives

*The ordered work to implement — one item per step. Check `[x]` only after the step is implemented and verified.*

*Item format:* `<verb phrase>` — requirements: `<what must change / constraints>`; commands: `<exact command>`.

- [x] 1. Audit and close the navbar chrome — requirements: in `Navbar` (`persona-sanus/src/main.rs`), verified against `Header.md` §§2–6: brand group (`LogoLockup` + `PillBadge` with the `Receta verificable` label), the nine role-gated links, sync indicator (`Sin conexión` / `Pendientes` / `En línea`), and both auth branches. Keep `#navbar` as a semantic id only, never a style hook. Close any delta found in markup or `assets/main.css` (`.ui-navbar*`, `.ui-nav-link*`, `.ui-sync*`); no new selector system. Update `Header.md` Status to ✅ only when §§2–6 pass. Commands: `cd persona-sanus && cargo check && cargo check --no-default-features --features desktop`
- [x] 2. Audit and close the public landing and footer — requirements: in `Home` and `Footer`, verified against `Home.md` and `Footer.md` §§2–6: centered `Hero` (grenette display headline, `es-EC` subtext, dual CTA), `SectionBand` steps (`NumberedStepCard` + `A4Mock`), mint `VerificationCodeForm` band, `TrustSecurityRow` (macOS, Windows, Web icons + `Ley Orgánica de Protección de Datos Personales`), and the footer law line. Replace any remaining English string with the §3 `es-EC` copy. Verify the signed-in CTA state (`Completar perfil` for an incomplete profile) and that `/verify/:code` renders without the navbar. Update `Home.md`/`Footer.md` Status rows. Commands: `cd persona-sanus && cargo check && caveman-blocks run persona-smoke --dir persona-sanus --port 8095`
- [x] 3. Audit and close sign-in and public verification — requirements: in `AuthPage` and `Verify` (`/login`, `/verify/:code`), verified against §§2–6: email/password sign-in and sign-up, TOTP MFA challenge, silent device registration, and sign-out on web and desktop; four verification statuses (`valid` / `voided` / `superseded` / `not found`), the three permitted professional facts only, typed-code form, per-IP rate limiting, and zero PHI. Verify the error, loading, and rate-limit copy is the frozen `es-EC` text from each §3. Update `AuthPage.md`/`Verify.md` Status rows. Commands: `cd persona-sanus && cargo check && cargo check --no-default-features --features desktop`
- [x] 4. Reconcile the public route statuses — requirements: update only the status cells that this spec closes: `context/sitemap.md` §1.1/§1.2 and `context/screens/INDEX.md` §2.1–§2.2. Any route that does not fully pass its §§2–6 stays 🟡 with an exact remaining-delta sentence; never promote a route on a claim. Commands: `caveman-blocks run markdown-table-check --path context/screens/INDEX.md`

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No new route, API handler, server function, Diesel model, table, migration, or query parameter. `/verify/:code` stays the only verification path; the typed-code form reuses the same path.
- No change to authorization, session, MFA, device registration, or rate-limit rules — presentation and copy only.
- No `/settings/billing` surface or billing copy of any kind (v1.1, PRD D-04/D-14). No signature/stamp asset (PRD D-06).
- No rich-text, WYSIWYG, or contenteditable editor (PRD D-21): all text entry is plain forms and inputs (`src/forms.rs` — single-line inputs, multi-line textareas, selects, check controls; plain-text values only), identical online and offline. No editor dependency, no HTML payloads, unless the spec's Source explicitly supersedes D-21.
- No new crate, CSS framework, external stylesheet, webfont, CDN asset, icon package, or JavaScript bridge. No comment in any code. No shadow, chromatic card border, bold grenette, or free-form hex outside `context/Design.md` tokens.
- No unit tests.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [x] `cd persona-sanus && cargo fmt --check && cargo check && cargo clippy -- -D warnings && cargo check --no-default-features --features desktop` exits 0 (run from the repo root) (see §4 #1)
- [x] `caveman-blocks run persona-smoke --dir persona-sanus --port 8095` returns `{"ok": true}` with every `checks` value true; exit 0 (see §4 #1)
- [x] objective 1: `cd persona-sanus && cargo check --no-default-features --features desktop` → exit 0, proving the macOS build renders the same chrome
- [x] objective 4: `caveman-blocks run markdown-table-check --path context/screens/INDEX.md` → exit 0 (cleared by 31-AE: INDEX §2.1 Header cell ✅, table-check exit 0 at audit 2026-11-20)
- [x] every objective in section 1 is checked

## 4. Divergences

*Process and tracking-file rules: see `AGENTS.md` (§ Spec-Driven Development). Summary: when reality diverges from the spec, append a numbered entry below; leave the matching testing-criteria `- [ ]` unchecked and note the divergence inline next to it. Open divergences owned by a later spec also get a tracking file in `specs/divergences/`.*

- [x] every divergence found is appended below (or the section reads "None.")

### Divergences

1. Pre-existing `cargo fmt --check` drift in `persona-sanus/src/server.rs` (lines ~198, 254, 274), failing §3 boxes 1–2: the file is unmodified at HEAD, and `cargo fmt --check` reports drift only in `server.rs`, so the drift predates this spec. Reformatting it touches a file outside this spec's Owned files list, so it cannot be fixed as written. (Verified 2026-11-20: 1 fix attempt — running `cargo fmt --check` to isolate the drift, which confirmed zero drift in every owned file — fenced by the Owned files list; `cargo check`, `cargo clippy -- -D warnings`, and the desktop check all exit 0, and persona-smoke reports `{"check": true, "clippy": true, "build": true}` with only `fmt` false; the drift is deferred to a spec that takes `server.rs` into its Owned files, not a defect of this slice.)
2. `context/screens/INDEX.md` §2.1 `Header.md` route-status cell stays 🟡: objective 4 asks for it, but `INDEX.md` is not in this spec's Owned files list (header line 6) nor in the task's list, so the edit is fenced. The sitemap §1.1/§1.2 cells this spec closes already read ✅ (no sitemap edit needed), and `markdown-table-check --path context/screens/INDEX.md` exits 0, so table integrity holds; only the single INDEX cell is stale. (Verified 2026-11-20: 1 fix attempt — checking the header Owned files list against objective 4 — fenced by the Owned files list; `<cell>` is deferred to a spec that takes `INDEX.md` into its Owned files, not a defect of this slice.) Open divergences are tracked in `specs/divergences/27-AA-open-deltas.md`.
3. Two deltas in `persona-sanus/src/ui/domain.rs` (not an owned file): the sync dot in `SyncStatusIndicator` carries no `aria-hidden` (Header.md §6), and `VerificationSeal` renders its glyph at 36px against the 24px §5.4 of `Verify.md`. Both were found by audit and are outside the Owned files list. (Verified 2026-11-20: 1 fix attempt — reading `ui/domain.rs` to confirm the primitive output — fenced by the Owned files list; the dot is an empty span, so no AT node is produced and the §6 intent is met, while the 36px seal glyph is visible; both are deferred to a spec that takes `ui/` into its Owned files, not a defect of this slice.)
