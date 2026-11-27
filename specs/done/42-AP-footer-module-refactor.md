# Spec 42-AP — Extract the public footer module

- Source: `context/screens/done/Footer.md` §§1–8; `context/screens/INDEX.md` §§1.1, 1.3 and 2.1; `context/architecture.md` §§3 and 7; approved dedicated-module extraction decision.
- Depends on: `41-AO`, `32-AF`.
- Project root: the `doctor-prescriptions` repository root. All paths and commands below are relative to it.
- Owned files: `persona-sanus/src/main.rs`, `persona-sanus/src/footer.rs`, `context/screens/done/Footer.md`, `context/screens/INDEX.md`, `context/architecture.md`.
- Gate: `cargo check --manifest-path persona-sanus/Cargo.toml && cargo check --manifest-path persona-sanus/Cargo.toml --no-default-features --features desktop`.
- User-interface verification: required — inspect all three public footer surfaces, signed-in exclusions, keyboard anchor behavior, responsive landmarks and the native desktop version line on isolated local data using section 3.
- Result carried forward: one Footer component and one FooterSurface enum in `src/footer.rs`, preserving three public placements and the desktop/web distinction.
- Scope-change approver: the requesting repository owner.

## 1. Problem and decision

- Problem: the public footer component and its surface enum are embedded in the large `main.rs`, obscuring their bounded presentation contract and three placement boundaries. The source screen contains stale line references and contradictory historical whole-page law-row counts; those do not justify changing the shipped footer.
- Decision: extract the intact component and enum into a dedicated sibling application module, not the reusable `src/ui/` primitive layer. Preserve markup, styles, routes, copy, public placements and compile-time version behavior. This is an execution specification only; its generation does not execute the refactor.
- Open questions: None.

`41-AO` is complete in `specs/done/41-AO-header-module-refactor.md`; its extraction and approved desktop compilation fix are recorded there. Header ran before Footer because both own the extraction seam in `main.rs` and factual module references in common documentation. `specs/done/32-AF-footer-reconciliation-and-proof.md` is completed and supplies prior footer reconciliation evidence; its historical proof counts are not a replacement for current source inspection. Do not add dependencies on unrelated unfinished product or release specs. Queue ownership conflicts are scheduling constraints, not authorization to rewrite those specs.

## 2. Objectives and scope

- [x] 1. Extract the single footer component and surface enum — requirements: move `FooterSurface` into new `persona-sanus/src/footer.rs` as `#[derive(Clone, PartialEq)] pub(crate) enum FooterSurface { Home, Auth, Verify }`. Move the component intact as `#[component] pub(crate) fn Footer(surface: FooterSurface) -> Element`. Import only `dioxus::prelude::*` and `crate::ui::{LogoLockup, TrustSecurityRow}`. The inspected source inventory has no equivalent application module; do not place application footer policy in the primitive layer. Update only factual component/module references caused by extraction in `Footer.md`, `INDEX.md` and `architecture.md`; do not rewrite screen contracts or repair unrelated historical prose. The frozen documentation filename stays `Footer.md`.
- [x] 2. Preserve the root seam and all three placements — requirements: add `mod footer;` and `use crate::footer::{Footer, FooterSurface};` in `main.rs`, and delete the old component/enum definitions. Restrict `main.rs` edits to those moved definitions and their module/import seam. Preserve exactly the three existing call sites: last child in Home, last child in Verify, and last child of signed-out AuthPage's `PageShell`. Signed-in AuthPage returns earlier and remains footer-free. Root `LogoLockup` and `TrustSecurityRow` imports stay because page bodies use them. Do not move the footer into Navbar or duplicate body primitives.
- [x] 3. Preserve the surface matrix and DOM intact — requirements: brand and decorative rule on Home/Auth only; plain `Verificar una receta` anchor to `#verificar` on Home only; `TrustSecurityRow` on Auth only; exact `Solo se muestran el médico, la fecha y el número de la receta.` on Verify only. Keep `footer#ui-footer.ui-footer`, `.ui-container`, `.ui-footer-inner`, `.ui-footer-brand`, `.ui-footer-rule` with its `aria-hidden` attribute, `.ui-footer-legal`, `.ui-footer-disclosure`, `.ui-footer-version`, and all other wrapper/child attributes and classes. Preserve even the empty `.ui-footer-legal` container on web Home. Existing Home has three body `TrustSecurityRow` instances and the footer adds none; do not change rendering to reconcile contradictory historical whole-page counts.
- [x] 4. Preserve feature branches and session independence — requirements: retain both current cfg branches and their allocation behavior: desktop uses `Some(format!("Versión {}", env!("CARGO_PKG_VERSION")))`; non-desktop uses `let version: Option<String> = None;`, with no rendered web version. Keep the component free of context, signal, network and session reads. No boolean prop redesign, second enum, CSS, route, API, printed-PDF-footer or copy changes.

### Out of scope / no-go

- No product, route, role, permission, style, copy, PHI, legal or billing expansion, and no conformance redesign. No new legal/support links, law claims, duplicate body rows, footer on authenticated shell routes or navbar-owned footer. The printed prescription footer, PDF renderer and clinical documents are unrelated.
- No dependency, primitive, storage, API, server, session or sync changes. No application policy in `src/ui/`, compatibility shim, alias, forwarding component, duplicate enum or code comment.
- Do not edit unowned files or repair unrelated existing UI/CSS/documentation failures. `main.rs` ownership does not authorize changing page bodies, placements, the Route enum, root helpers or context providers. Documentation ownership permits factual component/module-reference changes only; leave historical whole-page law counts and aspirational focus wording outside this extraction.
- Preserve existing error behavior. An intact move needs no new fallback or missing-state policy. No broad test suite, release build or new test scaffolding is required for unchanged component bodies.

## 3. Acceptance and verification

Run the gate once after the final code edit, covering default web/server and desktop compilation. If a correction requires another edit, rerun the same gate after that final correction. Do not run checks between edits. Required UI observation below is additional proof of the unchanged surface, not a redesign opportunity.

Prerequisites: Cargo, `dx`, a browser, free port 8091, a fresh temporary database and native desktop launch capability. From the repository root launch:

```sh
CHROME_DB_DIR="$(mktemp -d)" &&
(cd persona-sanus && DATABASE_URL="sqlite://$CHROME_DB_DIR/chrome.db" PERSONA_ENV=development PERSONA_COOKIE_SECURE=false dx serve --fullstack --addr 127.0.0.1 --port 8091)
```

After the local service responds, open a fresh tab at `http://127.0.0.1:8091/`. Never use a production database or borrowed user tab. Use real UI interactions and read-only DOM/accessibility inspection, not injected controls or session state. On this fresh database create synthetic `chrome-admin@example.com` through `Crear cuenta`, using a synthetic name and local-only password `ChromeProof123!`; the first account is Administrator. Use this account for signed-in checks and sign out for public checks. Do not seed production data or manufacture clinical fixtures.

- [x] Objectives 1–2: module seam — verification: use LSP references/definition resolution after extraction → all three external component call sites resolve to `persona-sanus/src/footer.rs`; one component and one `FooterSurface` enum remain, with no root duplicate or wrapper. Inspect the move/import changes → only the permitted seam and factual documentation references changed, and root brand/trust imports remain.
- [x] Objectives 2–3: public surface matrix — verification: signed out, visit `/`, `/login` and `/verify/PS-TEST` → exactly one `footer#ui-footer.ui-footer` per page. Inspect within the footer, not global page text counts: Home has brand, plain anchor and decorative rule, no trust row or disclosure, and the empty legal container on web; Auth has brand, rule and one `Datos protegidos · Ley Orgánica de Protección de Datos Personales` row with no verify anchor or disclosure; Verify has no brand, rule or trust row and one exact `Solo se muestran el médico, la fecha y el número de la receta.`. Unknown-code status does not remove its footer. Do not pin contradictory historical whole-page law-row counts.
- [x] Objective 3: Home anchor — verification: keyboard-activate the footer's `Verificar una receta` link → pathname stays `/`, hash becomes `#verificar` and the existing band scrolls into view. Preserve the existing focus behavior; do not add a script to satisfy aspirational historical wording about moving focus.
- [x] Objective 2: session and route boundaries — verification: with the synthetic Administrator signed in, visit `/login`, `/dashboard`, `/patients`, `/prescriptions`, `/templates` and `/settings` → none has a footer. Visit debug `/dev/ui` → no footer. Visit signed-in `/` → its footer still renders. Inspect the signed-in AuthPage early return and three placement boundaries → none changed.
- [x] Objective 4: web version branch — verification: inspect all three web public pages → no `.ui-footer-version`. Compare both cfg branches with the original component → desktop allocation and non-desktop `None` remain intact, with no context/signal/network/session reads added.
- [x] Objective 4: native desktop version — verification: with the isolated local server still running, start a separate isolated desktop instance using the command below. Observe one visible version line matching `persona-sanus/Cargo.toml` on Home, signed-out login and an unknown-code result reached through Home's code form. Stop the desktop instance after proof. If native launch is unavailable, leave this criterion unchecked and record a concrete divergence; a compile check does not prove native rendering.
- [x] Gate: `cargo check --manifest-path persona-sanus/Cargo.toml && cargo check --manifest-path persona-sanus/Cargo.toml --no-default-features --features desktop` passes from the project root.
- [x] User-interface verification: record public-footer screenshots at widths 1440 and 375, footer landmarks and keyboard focus, surface-scoped DOM/accessibility evidence, signed-in exclusions, anchor pathname/hash/scroll outcome and native version evidence. Record existing UI/CSS failures without fixing them within this extraction.
- [x] Every objective in section 2 is checked only after its criteria, gate and required UI evidence pass.

Run this native observation command from the repository root in a separate terminal:

```sh
CHROME_LOCAL_DIR="$(mktemp -d)" && PERSONA_LOCAL_DATA_DIR="$CHROME_LOCAL_DIR" PERSONA_SERVER_ORIGIN=http://127.0.0.1:8091 cargo run --manifest-path persona-sanus/Cargo.toml --no-default-features --features desktop
```

Stop the desktop and local server after observation. Clean up only the temporary database/local-data proof fixtures created for this task.

## 4. Execution and divergences

Execute in a fresh, focused task context with a hard ceiling of **500,000 loaded tokens**. This is a maximum per execution task, not a target spec size or provider configuration change. Read applicable project instructions, this spec, relevant completed dependency results (`41-AO` must first complete, plus completed `32-AF`), `context/screens/done/Footer.md`, screen-index authority/naming rules, architecture module/shell rules, `persona-sanus/Cargo.toml` and only relevant source/component regions. For Footer this includes root imports/module declarations, the complete component/enum and both cfg branches, all three placement boundaries including signed-in AuthPage's early return, route/layout order and the `LogoLockup`/`TrustSecurityRow` contracts. Do not load all of `main.rs`, all specs or previous transcripts.

Immediately before moving symbols, run LSP references for both `Footer` and `FooterSurface` and identify the three external component call sites. Before Header extraction the enum/component were at `main.rs:1326–1367`, with Home at `:1525`, Verify at `:1583` and signed-out AuthPage at `:2046`; these are hints, not stable locations. Header extraction changes line numbers. Reread actual source regions before editing, including all placements and cfg branches. If `footer.rs` exists at execution time, reuse it only if it already owns the same component/contract; otherwise record an ownership divergence before touching it. Never overwrite an unrelated module or invent an alternate name.

If reality differs or necessary work exceeds ownership, record the current state, evidence and attempts actually made, impact, blocker, owner, and required decision under Divergences. Stop affected work and leave affected acceptance criteria unchecked with the divergence noted beside them; unaffected objectives may continue within scope. The requesting repository owner must decide before scope or ownership changes. Preserve existing error behavior rather than introducing an unapproved policy. Incomplete dependencies block affected objectives unless the requesting repository owner explicitly decides otherwise.

Open divergences block completion until resolved or an authorized scope change updates objectives and criteria. Follow project divergence tracking-file rules for approved deferrals, including an owner or later spec; required unowned tracking work needs approval, not an implicit ownership expansion. Archive the completed spec under `specs/done/` only when all applicable criteria, the gate, UI verification and objectives pass with no open divergences. Do not execute either extraction spec during specification generation.

### Divergences

1. **Resolved — native unknown-code observation:** After the repository owner authorized foreground input, the isolated desktop app accepted `PS-TEST` through Home's verification form. The unknown-code result displayed one visible `Versión 0.1.0`, matching `Cargo.toml`. No code or CSS change was needed.

### Execution evidence

- Extracted the intact component and enum into `persona-sanus/src/footer.rs`, with crate-visible declarations and the two specified imports. Only the module/import seam and removed definitions changed in `main.rs`; all three placements, root brand/trust imports, routes and signed-in AuthPage early return remain unchanged.
- Before extraction, LSP references identified Home, Verify and signed-out AuthPage at main.rs:1532, :1590 and :1951. After extraction, LSP references resolve the three consumers at main.rs:1491, :1549 and :1910 to footer.rs:13, with the single enum at footer.rs:6.
- After the last code edit, both gate commands passed: `cargo check --manifest-path persona-sanus/Cargo.toml && cargo check --manifest-path persona-sanus/Cargo.toml --no-default-features --features desktop`. Desktop emitted existing dead-code and dependency future-compatibility warnings.
- On fresh temporary SQLite, created `chrome-admin@example.com` through `Crear cuenta`. Signed-in `/login`, `/dashboard`, `/patients`, `/prescriptions`, `/templates`, `/settings` and `/dev/ui` each had zero footers; signed-in `/` had one. Signed out afterward.
- Signed-out `/`, `/login` and `/verify/PS-TEST` each had one `footer#ui-footer.ui-footer` and one accessibility `contentinfo` landmark. Home had brand, plain verification anchor, aria-hidden rule, no trust row/disclosure/version, and an empty legal container. Login had brand, rule and one trust row with the exact law text, without verification anchor/disclosure/version. Verify had the exact disclosure and no brand/rule/trust/version. Both cfg branches and their allocation behavior remain intact.
- Keyboard Tab reached the Home footer anchor with a visible 1px Deep Teal outline. Enter preserved pathname `/`, set hash `#verificar`, and scrolled to the band (scrollY 3151, band top approximately -0.36px). No focus script was added.
- Captured all three public footers at 1440 and 375px. No horizontal page overflow was observed. Home brand/link stacked at 375px; Auth law text and Verify disclosure wrapped. Auth footer remained inset within PageShell, unchanged.
- Web screenshot paths under `/var/folders/x2/nk8wfy812qg3mx9l92gxn__h0000gn/T/`: `omp-sshots-15deb575bd23aa6f.png` (Home 1440), `omp-sshots-15deb575da23aa70.png` (Auth 1440), `omp-sshots-15deb57600e3aa71.webp` (Verify 1440), `omp-sshots-15deb57621e3aa72.webp` (Home 375), `omp-sshots-15deb5763de3aa73.png` (Auth 375), `omp-sshots-15deb5765d63aa74.webp` (Verify 375).
- Native screenshots under that temporary directory's `orca-computer-use/`: `661686e5-91c3-485e-a725-a3db0703d517-screenshot.png` (Home version) and `df956cba-93f3-4529-ac5a-214f28b9361a-screenshot.png` (signed-out login version).
- Native unknown-code result screenshot: `/var/folders/x2/nk8wfy812qg3mx9l92gxn__h0000gn/T/orca-computer-use/7e749772-3ae6-4a77-8996-5c15cc0526a1-screenshot.png`. The result displayed `Receta no encontrada`, the Verify disclosure and one `Versión 0.1.0` line.
