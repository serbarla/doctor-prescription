# PersonaSanus — Footer (public surfaces)

| Field | Value |
|---|---|
| Source | `context/sitemap.md` §1.1 (`/` :72, `/login` :73, `/verify/:code` :74 — "Only the path form is routed"), §5 (`:188` deep links); `context/Design.md` (Logo Lockup, Trust & Security Badge Row, Tokens, Surfaces, Elevation, Layout, Do/Don't); `context/PRD.md` §5.6 (FR-050, FR-033/034), §8 (`es-EC`), §9 (LOPDP, ARCO, minimization), §14 (D-04, D-06, D-21), §15 (O-01); `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` (coverage table rows "Logo Lockup" and "Trust & Security Badge Row", component contracts `LogoLockup` `:123`, `TrustSecurityRow` `:124`, `LinkAction` `:73`, steps 9–12, No-go); `context/screens/INDEX.md` §1.1 (frozen name), §1.3 (chrome strings belong here), §2.1 |
| Component | `Footer` at `persona-sanus/src/main.rs:1333` with `enum FooterSurface { Home, Auth, Verify }` at `:1326` — shipped. Composes `LogoLockup` and `TrustSecurityRow` (`persona-sanus/src/ui/brand.rs:13` and `:28`); `.ui-footer*` classes at `persona-sanus/assets/main.css:2124`, `.ui-footer-inner` `:2130` |
| Rendered on | `/` (`Home` at `main.rs:1368`, shipped root `div#home-page` `:1389`, `FooterSurface::Home` at `:1460`), `/login` (`AuthPage` at `:1633`, signed-out branch only, `FooterSurface::Auth` at `:1980`), `/verify/:code` (`Verify` at `:1465`, root `div#verify-page` `:1487`, `FooterSurface::Verify` at `:1517`) |
| Not rendered on | Every authenticated shell route (`/dashboard`, `/patients*`, `/prescriptions*`, `/templates*`, `/settings*`, `/audit`) and the debug-only `/dev/ui` gallery (plan step 9: static, no contexts — no footer) |
| Layout | Appended as the last element inside each page's own root. `Home` and `AuthPage` sit inside `#[layout(Navbar)]` (`main.rs:69`, never closed with `#[end_layout]` — `context/architecture.md` §5); `Verify` is declared before the marker (`main.rs:65-66`) and is standalone, no navbar. No layout change here. Full-bleed band, inner `.ui-container` max-width 1200px (plan CSS contract `:257`) |
| Locale | `es-EC` on every string. `receta`, `cédula` keep Ecuador Spanish spelling |
| Status | ✅ Spec complete (sections 1–7). Shipped: `Footer` at `persona-sanus/src/main.rs:1333` with `FooterSurface::{Home, Auth, Verify}` — brand row + verify anchor (Home), `TrustSecurityRow` (Auth), disclosure line (Verify), desktop-only version line; appended inside each page root at `Home` `:1460`, `Verify` `:1517`, signed-out `AuthPage` `:1980`; never in the authenticated shell. Proof counts in §7 verified by spec 32-AF |

This is the application page footer (web and desktop chrome). It is not the printed prescription's signature/QR block, assembled by `make_signature_qr_blocks` (`persona-sanus/src/forms.rs:1524-1529`) and rendered by `persona-sanus/src/pdf.rs`. Nothing in this file touches `doc_json`, the renderer, or the printed document.

## 1. Purpose

Close the three public surfaces with the minimum that a visitor holding a printed prescription or evaluating the product needs: the wordmark and its way back to `/`, one entry point to verification on Home, and each legal line exactly once per page — placed where the page body does not already render it.

Three jobs, in this order:

1. Brand and return path — the `LogoLockup` links to `/` on `/` and `/login`. On `/verify/:code` the wordmark already sits in plan step 12's `.ui-public-head`, so the footer does not repeat it there; that page's footer is legal text only.
2. Verification entry on Home — an in-page anchor to the `#verificar` band (Home.md §2 band 3), so a visitor who scrolled past the hero still reaches the code field.
3. Legal presence — the LOPDP line as one `TrustSecurityRow` instance, which the footer owns only on `/login` (the one public surface whose body renders no row), and the FR-050 disclosure limit, which the footer owns only on `/verify/:code` (the one public surface whose body never states it).

The footer is session-agnostic: no signal read, no fetch, no role check, no sync status. What it renders is decided by the surface it is placed on, never by who is signed in.

It never renders inside the authenticated shell. The clinical routes end with their own actions (sitemap §1.2, Design.md Layout: "No sidebar, no sticky header beyond the nav"), and the shell already carries the session, role, and sync state in the navbar (FR-043, `Header.md` §4). A footer band there would add chrome without an action.

## 2. Structure

One band, centered column, symmetric — the same posture as every other region (Design.md Layout, Do: "Center all hero and section content").

| Order | Region | `id` / class | Band tone | Contents |
|---|---|---|---|---|
| — | Root | `<footer id="ui-footer">` | Paper `#ffffff`, top hairline 1px `#e5e7eb` | `div.ui-container` wrapper |
| 1 | Inner | `.ui-footer-inner` | Paper | Centered column: brand row (when the surface has one), divider (same condition), then legal row |
| 2 | Brand row | `.ui-footer-brand` | Paper | `LogoLockup` (`PersonaSanus` → `/`) +, on `/` only, the plain anchor `Verificar una receta` (`<a href="#verificar">`) |
| 3 | Divider | `.ui-footer-rule` | Paper | Decorative `<hr aria-hidden="true">` hairline between the brand row and the legal row, rendered only on `/` and `/login` (the surfaces whose brand row exists) |
| 4 | Legal row | `.ui-footer-legal` | Paper | `TrustSecurityRow` (`/login` only), disclosure line (`/verify/:code` only), version line (desktop build only) |

No form, no input, no button, no icon-only control, no button-shaped link anywhere in the footer.

### 2.1 What renders per surface

| Surface | Brand row | Verify link | Law row (`TrustSecurityRow`) | Disclosure line | Version line (desktop build) |
|---|:---:|:---:|:---:|:---:|:---:|
| `/` | ✓ | ✓ | — (the hero and the mint verify band each carry the row: Home.md §3.2, §2 band 3) | — (stated once in Home §3.5) | ✓ |
| `/login` | ✓ | — | ✓ (the footer is the page's only instance: plan step 10 gives the auth card no row) | — | ✓ |
| `/verify/:code` | — (step 12's `.ui-public-head` owns the wordmark; `Header.md` §2.2) | — | — (the body always renders the row in the `Verificar otra receta` card, plan step 12) | ✓ (the footer is the page's only instance) | ✓ |
| Authenticated shell, `/dev/ui` | — | — | — | — | — |

Rules behind the matrix:

- One `TrustSecurityRow` per page; the footer renders it only on the surface whose body renders none. `/` already carries three instances in the body (hero + mint verify band + closing band, `Home.md` §3.2/§3.6/§3.7), `/verify/:code` carries one (step 12's card), `/login` carries none — so the footer adds the row on `/login` and nowhere else. It never renders a hand-rolled shield-plus-text variant: the primitive renders as defined or not at all.
- The disclosure limit (doctor, date, prescription number, nothing else) appears exactly once per page. Home states it in its mint band (Home.md §3.5). The shipped `Verify` body states it nowhere — only the four status titles and the `Doctor` / `Fecha` / `No.` fields (`main.rs:1465-1519`) — and plan step 12 adds no sentence, so on `/verify/:code` the footer is that one instance. The future `Verify.md` spec must not repeat it.
- The footer brand row is skipped on `/verify/:code`: step 12's `.ui-public-head` (`LogoLockup` + `PillBadge`) is that page's brand row and return path; a second wordmark at the bottom resolves nothing.
- No footer link can target `/verify/:code`: the route requires a `code` param and "Only the path form is routed" (sitemap §1.1 `:74`, §5 `:188`). A code-entry page at `/verify` would need a sitemap and PRD change first. On `/login` the wordmark back to `/` is the verification path, and `/verify/:code` already holds its own code form.
- The verify link keeps the exact hero label `Verificar una receta` (Home.md §3.2). Same target, same words: one term per thing.

## 3. Copy text

All strings verbatim, `es-EC`. Punctuation is part of the copy.

| Element | Copy | Target |
|---|---|---|
| Wordmark (`LogoLockup`) | `PersonaSanus` — `aria-label="PersonaSanus — inicio"` | `/` (`Route::Home {}`) |
| Verify link (`/` only) | `Verificar una receta` | in-page anchor `#verificar` |
| Law row (`/login` only) | `Datos protegidos · Ley Orgánica de Protección de Datos Personales` — the `TrustSecurityRow` contract string (plan `:124`), never reworded | none — the primitive (shield glyph, text, platform glyphs) |
| Disclosure line (`/verify/:code` only) | `Solo se muestran el médico, la fecha y el número de la receta.` | none |
| Version line (desktop build) | `Versión 0.1.0` — literal `Versión ` + `env!("CARGO_PKG_VERSION")`, never a hardcoded string | none |

No tagline, no address, no phone, no email, no opening hours, no copyright line: nothing in `context/PRD.md` or `context/sitemap.md` establishes a support channel or a legal entity to name, and `PRD.md` §15 O-01 (counsel sign-off) is still open.

## 4. Displayed information

### 4.1 What the footer shows

| Element | Source | Value shown |
|---|---|---|
| Wordmark | static | `PersonaSanus` + the mint circle and cross glyph inside `LogoLockup` (plan `:123`) |
| Verify link | static | the anchor label; the browser scrolls, no route change |
| Law row | static | the `TrustSecurityRow` primitive — shield glyph, LOPDP line, `macOS, Windows, Web` glyphs with the visually hidden platform label (plan `:124`) |
| Disclosure line | static | the FR-050 limit in words (PRD §5.6) |
| Version line | `env!("CARGO_PKG_VERSION")` at compile time — the same source as `persona-sanus/src/update.rs:210` and the value registered into `devices.app_version` (`persona-sanus/src/api.rs:580`) | `Versión {version}` |

The footer performs no network call, reads no `Signal`, and touches no context. It renders identically online and offline, signed-out and signed-in, on web and desktop, except for the version line.

### 4.2 Hard rules

- No PHI: no patient name, cédula, diagnosis, medication, or photo in any footer string, glyph, or URL. Only the opaque verification `code` travels in a URL at all (PRD §9, FR-033).
- No invented legal surface: no `/privacidad`, `/terminos`, `/contacto`, `/arco` link — no such route exists in sitemap §1 and the UI plan No-go forbids new product routes. ARCO rights are exercised inside the app through the audited patient flow (PRD §9), not through a footer form.
- No legal or medical guarantee: the footer names the law, it never claims compliance, certification, or legal validity of the PDF/QR (PRD §15 O-01 open, §13 "Legal validity of PDF/QR").
- No billing, price, plan, trial, or `sync_locked` copy (D-04). No image-quota copy.
- No invented proof: no partner logos, no testimonials, no counters, no app-store badges, no social icons.
- No button shape and no second filled teal action: `LinkAction` renders `<a class="ui-button ...">` (plan `:73`), so the footer's one link is a plain anchor — the band contains no `.ui-button` element, and the viewport's single `#006e75` filled button stays the hero CTA (Design.md Do/Don't).
- Never a duplicate of a body primitive: no second `TrustSecurityRow` on a page whose body renders one, no `LogoLockup` on `/verify/:code` (§2.1).
- No form, no email capture, no newsletter input. All text entry stays in product forms (D-21), and the footer is not one.
- No signature or stamp imagery anywhere (D-06).
- No language switcher: v1 is 100% `es-EC` (PRD §8).
- No sticky or fixed footer: it sits at the bottom of the document flow.

### 4.3 Dynamic states

| State | Footer behaviour |
|---|---|
| Signed-out visitor on `/` | brand row + verify anchor + version line (desktop); no law row |
| Signed-in staff on `/` | identical — the CTA above changes per Home.md §3.3, the footer never does |
| `/login`, signed-in branch | no footer: the signed-out branch owns it; the signed-in card is a session utility, not a public surface |
| Desktop offline | identical; the sync indicator lives in the navbar (FR-043), not the footer |
| Anchor click `Verificar una receta` | in-page scroll to `#verificar`; no route change, no fetch |
| `/verify/:code` with an unknown, voided, or superseded code | footer unchanged — its copy is static and status-independent |
| Web build | version line absent: the served bundle's version is not the desktop build the user is support-troubleshooting |

## 5. Style

Every value is a Design.md token already declared in `persona-sanus/assets/main.css` `:root`, or a contract value of `LogoLockup` / `TrustSecurityRow` (plan component table). No new color, font, icon package, CDN, or webfont.

### 5.1 Color

| Use | Value | Token |
|---|---|---|
| Footer band | `#ffffff` | `--color-paper` |
| Top rule of the band | `#e5e7eb` | `--color-hairline` |
| Wordmark | `#006e75` | `--color-deep-teal` (Logo Lockup) |
| Verify link, default | `#425d6d` | `--color-mist` |
| Verify link, hover and `:focus-visible` | `#006e75` | `--color-deep-teal` |
| Law row text | `#061d29` | `--color-ink` (`TrustSecurityRow`: mint 500 14px Ink) |
| Shield glyph | `#006e75` | `--color-deep-teal` |
| Platform glyphs (inside the row, `/login` only) | `#425d6d` | `--color-mist` |
| Disclosure and version text | `#425d6d` | `--color-mist` |

Never `#000000`. Never a chromatic border — the only border on the band is the `#e5e7eb` hairline. Never a mint or cream band here: Home's `#b9ffe8` verify band sits directly above it, and two tinted surfaces stacked read as one block (Design.md Surfaces level 2 is the signature accent, not a page base).

Measured contrast (AA needs ≥ 4.5 for body, same method as Home.md §5.1): Ink on white 17.28 · Mist on white 6.96 · Deep Teal on white 6.02. Bright Teal `#0b978e` on white measures 3.60 and is therefore not used for footer text or link hover; it stays reserved for fills and strokes on large elements (button hover, Design.md Bright Teal role).

### 5.2 Typography

| Element | Family | Weight | Size / line-height |
|---|---|---|---|
| Wordmark | `--font-mint` | 500 | 26px / 1.22 in the footer (scoped `.ui-footer-brand .ui-logo-word` override; the shared `LogoLockup` stays 22px elsewhere) |
| Verify link | `--font-mint` | 400 | 14px / 1.5, Mist |
| Law row | `--font-mint` | 500 | 14px / 1.4, Ink (`TrustSecurityRow` and `.ui-trust` contract) |
| Disclosure line | `--font-mint` | 400 | 14px / 1.5, Mist |
| Version line | `--font-mint` | 400 | 12px / 1.5, Mist (`--text-caption`) |

`--font-grenette` never appears in the footer — it is a display face for 46px+ headlines only (Design.md Don't).

### 5.3 Geometry

- Base unit 4px. Band padding-block 64px (`--spacing-64`; 24px below 640px wide), inline padding `--spacing-24`; inner `.ui-container` max-width 1200px, centered.
- Gap between the band and the region above: none beyond the hairline — the preceding band already carries its own 64–80px block padding.
- Inner column: `.ui-footer-inner` gap `--spacing-24` between the brand row, divider, and legal row.
- Brand row: single centered row, `--spacing-24` gap between wordmark and verify link; wraps to a stacked column below 640px. The footer wordmark is 26px (the `--text-heading` mint ceiling) with a 26px mark and 16px glyph — a larger brand than the 22px header lockup.
- Divider: `.ui-footer-rule` is a full-width hairline (`border-top: 1px solid var(--color-hairline)`), `max-width: var(--content-copy)`, no height, rendered only on `/` and `/login`.
- Legal row: centered, max-width 560px (`--content-copy`, plan step 2), `--spacing-16` between the law row, disclosure line, and version line; inside the row the gap between glyph and text is the primitive's 12px.
- Border radii: none — the band is a full-width edge-to-edge region; no card, no pill.
- Elevation: flat. Top hairline and whitespace only; no drop shadow on any footer element (Design.md Elevation).

### 5.4 Iconography and imagery

On `/login` only: the `TrustSecurityRow` primitive exactly as defined — `Shield` 18px Deep Teal, the law text, then `Command` / `Windows` / `Globe` 18px Mist (plan `:124`). The platform glyphs ride the primitive; the footer neither adds a second platform row nor rebuilds the row as shield-text-only. Everywhere else: no icons at all. Line icons house rule applies (1.5px stroke, monochrome Ink or Deep Teal; `Cross` stays the filled exception). No photography, no illustration, no logo strip, no QR, no signature or stamp imagery (D-06).

## 6. Accessibility

- The footer is a real `<footer>` element, the single `contentinfo` landmark on each public page. It contains no heading and never starts an `h1`–`h2` sequence (Home.md §6 heading order stays `h1` → `h2`).
- Tab order: page content, then the footer wordmark, then the verify anchor. Both use the shared `:focus-visible` utility — shipped: 2px `#0b978e` outline, `outline-offset: 2px` (`assets/main.css:172-177`); plan step 3 replaces that utility with the library focus treatment — the footer defines no focus style of its own.
- The verify link is a real `<a href="#verificar">`, keyboard-activatable, and moves focus to the `#verificar` band.
- The shield and platform glyphs are `aria-hidden`; the adjacent text and the primitive's visually hidden `macOS, Windows, Web` label carry the meaning. The law row, disclosure line, and version line are real text, selectable and read by a screen reader.
- §5.1 contrast meets AA for 12px and 14px text. No link is identified by color alone — the footer's only link is also underlined on hover and focus.

## 7. Implementation notes

- Status: shipped. `Footer` at `persona-sanus/src/main.rs:1333` with `enum FooterSurface { Home, Auth, Verify }` at `main.rs:1326`; `footer#ui-footer.ui-footer` > `.ui-container` > `.ui-footer-inner` (`persona-sanus/assets/main.css:2124`, `.ui-footer-inner` `:2130`), composes `LogoLockup` (`persona-sanus/src/ui/brand.rs:13`) and `TrustSecurityRow` (`persona-sanus/src/ui/brand.rs:28`). No signal, fetch, or context is read.
- Call sites (the §2.1 matrix is the only branch in the body): `Home` at `main.rs:1460` (`FooterSurface::Home`, last child of `#home-page`), `Verify` at `main.rs:1517` (`FooterSurface::Verify`, last child of `#verify-page`), and the signed-out branch of `AuthPage` at `main.rs:1980` (`FooterSurface::Auth`, inside `PageShell` — never the signed-in early return). Brand row on `Home`/`Auth` only; verify anchor `Verificar una receta` → `#verificar` on `Home` only; `TrustSecurityRow` on `Auth` only; disclosure line `Solo se muestran el médico, la fecha y el número de la receta.` on `Verify` only.
- Version line is gated `#[cfg(feature = "desktop")]` at `main.rs:1334` — `Versión {env!("CARGO_PKG_VERSION")}`; the web bundle renders none. No layout change: `#[layout(Navbar)]` stays unclosed, the footer is the page's last child, and `Verify` remains before the layout marker (`main.rs:65-69`).
- Poolmoney-inspired refinements: the band now uses 64px block padding, the footer wordmark renders at 26px with a 26px mark and 16px glyph (`.ui-footer-brand .ui-logo-*`), and a decorative `<hr class="ui-footer-rule" aria-hidden="true">` separates the brand row from the legal row on `/` and `/login` only. No new copy, link, or route; the §2.1 matrix and the hard rules are unchanged.
- No new route, endpoint, crate, network font, CDN, icon package, or JS bridge. `/verify/:code` renders the footer on a route whose sitemap status cell may still lag the shipped code; footer copy is static and unaffected.
- `context/screens/done/Home.md` §2 and `context/screens/done/Header.md` §4.2 already point at this file; no page-file edit is part of this change.
- Verify, in order (shipped state): `cd persona-sanus && cargo check` clean, then `dx build --release --fullstack`, then
  - `curl -s http://127.0.0.1:8080/` contains `id="ui-footer"` once, `href="#verificar"` twice (hero secondary + footer anchor), and still exactly one `ui-button--primary`; the law line appears three times — hero, mint verify band, and closing band — the footer adds none;
  - `curl -s http://127.0.0.1:8080/login` contains `Datos protegidos · Ley Orgánica de Protección de Datos Personales` exactly once, and it sits inside `ui-footer`;
  - `curl -s http://127.0.0.1:8080/verify/PS-TEST` contains the law line exactly once (the `Verificar otra receta` card) and `Solo se muestran el médico, la fecha y el número de la receta.` exactly once (the footer);
  - `curl -s http://127.0.0.1:8080/dashboard` contains no `ui-footer`;
  - the web bundle contains no `Versión ` string on the public pages; a desktop build renders exactly one per public page;
  - `caveman-blocks run persona-smoke --dir persona-sanus --port 8091` still passes;
  - `caveman-blocks run markdown-table-check --path context/screens/done/Footer.md` passes.
- Do not add: a pricing or trial band, a partner logo strip, a newsletter or email field, social or app-store badges, a privacy/terms/contact/ARCO link, a language switcher, a second filled teal action, a `LinkAction` or any button-shaped link, a second `LogoLockup` on `/verify/:code`, a hand-rolled shield-only law-line variant, or a sticky footer.

## 8. Spec references

| Spec | Connection |
|---|---|
| `specs/done/15-O-ui-tokens-actions-and-formatting.md` | `15-O` supplies the tokens, `Button`, `ButtonLink`, `LinkAction`, and `NavLink` the shipped footer band uses. |
| `specs/done/17-Q-ui-brand-and-marketing-components.md` | `17-Q` builds `LogoLockup` and `TrustSecurityRow`, the two primitives the footer composes. `specs/32-AF-footer-reconciliation-and-proof.md` adopts `Footer` and proves it (spec 32-AF). `TrustLogoStrip` is never rendered by this band. |
| `specs/done/25-Y-ui-legacy-css-cleanup-and-docs.md` | `25-Y` removes the legacy page-ID selectors so the shipped footer composes only `.ui-*` classes and tokens. |
