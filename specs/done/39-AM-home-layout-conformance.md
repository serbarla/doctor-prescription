# Spec 39-AM — Match Home layout and visual tokens

- Source: `context/screens/Home.md` §§2, 3, 5 and 6; `context/Design.md` Tokens, Layout and Components.
- Depends on: completed `20-T` and `25-Y`; no incomplete dependency.
- Project root: the `doctor-prescriptions` repository root. All paths and commands below are relative to it.
- Owned files: `persona-sanus/assets/main.css`.
- Gate: `curl -fsS http://127.0.0.1:8091/ -o /dev/null` with the local fullstack server described in §3.
- User-interface verification: required — inspect Home in a fresh loopback browser tab and retain desktop/mobile screenshots and the computed values in §3.
- Result carried forward: Home keeps the five existing bands and exact copy while its hero, mobile headings, and trust glyphs match the new design.

## 1. Problem and decision

- Problem: the existing composition is already the new design, but shared styles shorten Home's hero padding, shrink its section headlines on narrow screens, and miss the trust shield since `Icon` now renders a wrapper.
- Decision: close those three deltas with Home-scoped declarations and preserve the rest of the existing layout; do not rebuild `Home` or extend primitives.
- Open questions: None.

## 2. Objectives and scope

- [x] 1. Match Home hero and section geometry — requirements: in `persona-sanus/assets/main.css`, add `#home-page > .ui-hero` next to `.ui-hero` with `width: 100%`, `max-width: var(--content-display)`, and `padding: var(--spacing-80) var(--spacing-24)` at every viewport, using the existing global border-box sizing; leave the shared mobile Hero rule intact and let the Home selector's specificity override it. Inside the existing `@media (max-width: 760px)` that shrinks `.ui-section-title`, add `#home-page .ui-section-title { font-size: var(--text-heading-lg); }`, including Cierre; preserve the shared heading font, weight, and line height. Keep `.ui-band`, `.ui-container`, `.ui-features`, `.ui-steps` and their existing 640/860px breakpoints; keep the 420px Hero visual, A4 mock, chips, band order, copy, and CTA markup unchanged.
- [x] 2. Restore the Home trust-shield color — requirements: add `#home-page .ui-trust > .ui-icon-wrap { color: var(--color-deep-teal); }` next to `.ui-trust > svg`, targeting the direct shield wrapper rendered by `TrustSecurityRow` via `Icon`; keep platform glyphs Mist, trust text Ink, and shared icon behavior unchanged.

### Out of scope / no-go

- No Rust, route, auth, data, shared style changes, CSS tokens, custom fonts, new assets, elevation, or extra Home sections. No billing, signature/stamp imagery, fake partner marks, or real patient data (Home §4.2). Do not change the historical Footer, Header or Verify contracts to do this work.

## 3. Acceptance and verification

Use a dedicated loopback dev server, with Cargo and `dx` installed and port 8091 free. From repo root run `HOME_DB_DIR="$(mktemp -d)" && (cd persona-sanus && DATABASE_URL="sqlite://$HOME_DB_DIR/home.db" PERSONA_ENV=development PERSONA_COOKIE_SECURE=false dx serve --fullstack --addr 127.0.0.1 --port 8091)`; wait until `/` responds, then open `http://127.0.0.1:8091/` in a new scratch tab, not a borrowed user tab.

- [x] Layout and copy — verification: at 1440×900 and 375×812, capture the full Home page and compare against Home §§2–3 → Navbar, Hero, Características, Cómo funciona, Verifica una receta, Cierre, then Footer; five bands, no extra surface; cream and mint backgrounds reach both edges, inner content is capped at 1200px, no horizontal scrolling, exact `es-EC` copy, signed-out Hero CTA `Emitir receta` and one filled `.ui-button--primary`, outlined Cierre with the same `/login` href. Confirm the desktop/larger breakpoint shows two feature columns and three step columns; at 860px and at 375px both grids show one column.
- [x] New visual conformance — verification: in the browser's computed styles compare 1440px and 375px widths → `.ui-hero` is centered, capped at 720px including its horizontal padding, with 80px top/bottom and 24px inline; Hero title is 60px and 46px respectively, and all four Home `h2` headings are 46px/1.15 at both widths. The regular bands measure 80px block padding at 1440px and 64px at 375px. The Hero preview is capped at 420px, uses 32px card padding, and chips wrap without overflow. Sample each of the three Home trust rows: the direct shield wrapper is `rgb(0, 110, 117)`, `.ui-trust-platforms` is `rgb(66, 93, 109)`, and the law line occurs in those three rows, not the footer.
- [x] Gate: `curl -fsS http://127.0.0.1:8091/ -o /dev/null` passes from the project root.
- [x] User-interface verification: attach or link desktop and mobile Home screenshots, and record the grid and computed-style observations above.
- [x] Every objective in section 2 is checked.

Verification: desktop screenshot `/tmp/spec39-home-1440.png`; mobile screenshot `/tmp/spec39-home-375.png`. At 1440px, feature and step grids have two and three columns; both have one column at 860px and 375px. Hero widths are 720px and 375px at 1440px and 375px, with `80px 24px` padding. Hero titles are 60px and 46px; all four section headings are 46px with 52.9px line height. Band block padding is 80px and 64px. Preview widths are 420px and 327px with 32px card padding. All three shield wrappers are `rgb(0, 110, 117)` and platform glyphs are `rgb(66, 93, 109)`. Neither viewport scrolls horizontally.

## 4. Execution and divergences

Read the nearest project instructions before executing. In a fresh task context, read only Home §§2–6, the applicable Design components/tokens, `Home` in `main.rs`, `TrustSecurityRow` in `ui/brand.rs`, `Icon` in `ui/icon.rs`, and the relevant CSS rules and media queries. Keep loaded task material under 50k tokens; use range reads instead of loading entire Rust/CSS files or historical specs. If an objective differs from reality or needs an unowned file, stop the affected work and ask the repository owner before changing scope. Record the state, evidence, actual attempts, impact, blocker and owner; leave blocked criteria unchecked. Apply project tracking-file and archive rules after verification.

### Divergences

None.
