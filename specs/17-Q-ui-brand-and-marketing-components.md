# Spec 17 — Phase Q: UI brand and marketing components

- Source: `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` step 6; `context/Design.md` §§Components, Logo, Hero, Steps, Trust; `context/sitemap.md` §5; `context/PRD.md` D-06, D-21, FR-050.
- Depends on: 15-O, 16-P.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Result carried forward: `persona-sanus/src/ui/brand.rs` and `marketing.rs` expose `LogoLockup`, `TrustSecurityRow`, `TrustLogoStrip`, `Hero`, `SectionHeader`, `NumberedStepCard`, `A4PreviewCard`, `A4Mock`, and `VerificationTag` with the fixed fictional samples and token-only styling; the public landing and verification pages can render without any other new component.

## Before you start

Unit testing is not required; avoid it. Prioritize end-to-end and integration testing only.

Read the nearest `AGENTS.md` (target workspace, then parents up to repo root) before executing.

If reality diverges from this spec: STOP, append a divergence (section 4), and NEVER improvise outside the objectives.

This spec consumes `15-O` actions and `16-P` layout. It does not adopt `Home` or `Verify`; spec `20-T` does that. `TrustLogoStrip` is gallery-only until real partner marks exist.

## 1. Objectives

*The ordered work to implement — one item per step. Check `[x]` only after the step is implemented and verified.*

*Item format:* `<verb phrase>` — requirements: `<what must change / constraints>`; commands: `<exact command>`.

- [ ] Add the brand components — requirements: create `persona-sanus/src/ui/brand.rs` with `LogoLockup`, `TrustSecurityRow`, `TrustLogoStrip`, `TrustLogo`, and the `TrustLogo` props exactly as section 7. The logo cross is the only decorative mark. `LogoLockup` links Home with `aria-label="PersonaSanus — inicio"` and marks the glyph `aria-hidden`. `TrustSecurityRow` uses Shield, Command, Windows, and Globe and adds the visually hidden platform label. `TrustLogoStrip` uses text logos at 18px mint 500 Ink or images max-height 28px with grayscale; no borders and no cards. Add the brand declarations in section 9; commands: `cd persona-sanus && cargo check`.
- [ ] Add the marketing components — requirements: create `persona-sanus/src/ui/marketing.rs` with `Hero`, `SectionHeader`, `NumberedStepCard`, `A4PreviewCard`, `A4Mock`, `VerificationTag`, and the `AccentTone` and `A4MockKind` enums exactly as section 7. Headings use grenette 400 and are never bold. `A4Mock` is static and uses only the fictional sample values in section 7. `A4PreviewCard` has no border and no shadow. `VerificationTag` has a 6px triangular tail. Add the marketing declarations in section 9; commands: `cd persona-sanus && cargo check`; `rg -n 'font-weight: *700|font-weight: *bold' persona-sanus/src/ui/brand.rs persona-sanus/src/ui/marketing.rs`.

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No new crate, CSS framework, external stylesheet, webfont, CDN asset, icon package, or JavaScript bridge.
- No rich text, WYSIWYG, `contenteditable`, formatting toolbar, `execCommand`, HTML payload, or editor dependency. All text entry is plain controlled inputs, textareas, selects, and checks.
- No new product route, API handler, server function, Diesel model, table, migration, entitlement rule, or billing persistence.
- No change to authorization, session, device registration, sync, numbering, signing, PDF, QR, verification limits, or business validation.
- No page adoption in this spec. Do not edit `Home`, `Navbar`, `AuthPage`, `Verify`, or a settings page.
- No `TrustLogoStrip` on Home: no real partner marks exist. This component renders only in the gallery in `19-S`; never bake a fictional clinic name into a marketing page.
- No fake partner logos on Home and no endorsement claim on Home.
- No gradient except `--gradient-ember-ring`; no drop shadow on cards, tags, or images; no pure black; no chromatic card or image border.
- No new literal colour outside `:root`. No second stylesheet. No icon or image dependency.
- No rich text, signature, stamp, or patient photo (PRD D-06, D-21).
- No unit tests and no comments in Rust, CSS, or RSX.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [ ] `cd persona-sanus && cargo check` exits 0.
- [ ] `cd persona-sanus && cargo check --no-default-features --features desktop` exits 0.
- [ ] objective 1: `rg -n 'fn (LogoLockup|TrustSecurityRow|TrustLogoStrip)' persona-sanus/src/ui/brand.rs` returns 3 matches.
- [ ] objective 1: `rg -n 'TrustLogo' persona-sanus/src/ui/brand.rs` shows the struct and its `name` plus `src` fields and `Clone, PartialEq`.
- [ ] objective 1: `rg -n 'PersonaSanus — inicio|macOS, Windows, Web|Con la confianza de' persona-sanus/src/ui/brand.rs` returns the three fixed labels.
- [ ] objective 1: `rg -n 'TrustLogoStrip' persona-sanus/src/main.rs` returns no matches.
- [ ] objective 2: `rg -n 'fn (Hero|SectionHeader|NumberedStepCard|A4PreviewCard|A4Mock|VerificationTag)' persona-sanus/src/ui/marketing.rs` returns 6 matches.
- [ ] objective 2: `rg -n 'font-weight: *700|font-weight: *bold' persona-sanus/src/ui/brand.rs persona-sanus/src/ui/marketing.rs` returns no matches.
- [ ] objective 2: `rg -n 'María Fernanda Ruiz|FICHA DEL PACIENTE|Faringitis aguda|Amoxicilina 500 mg|PS-7K2F-9Q' persona-sanus/src/ui/marketing.rs` returns every fixed fictional sample.
- [ ] every objective in section 1 is checked.
- [ ] every divergence found is appended to section 4 below.

## 4. Divergences

*Process and tracking-file rules: see `AGENTS.md` (§ Spec-Driven Development).
Summary: when reality diverges from the spec, append a numbered entry below;
leave the matching testing-criteria `- [ ]` unchecked and note the divergence
inline next to it. Open divergences owned by a later spec also get a tracking
file in `specs/divergences/`.*

- [ ] every divergence found is appended below (or the section reads "None.")

*Entry format (one numbered item per divergence):*

1. <what diverged and its current state> — <why it could not be done as
   written, citing the No-go rule or spec that blocks it> (Verified <date>:
   <N> fix attempts — <attempt 1>, <attempt 2>, <attempt 3> — all fenced by
   <constraint>; <impact> is deferred to <later spec> by design, not a defect.)

### Divergences

None recorded yet.

## 5. Context

The landing and public verification surfaces need one editorial voice: a centered serif headline, mint brand mark, three-step A4 story, and privacy proof. Today those pieces are hand-built from legacy `.button`, `.badge`, and page-ID CSS. This spec builds the reusable brand and marketing layer; `20-T` wires it to `/` and `/verify/:code`, and `19-S` proves every variant in the gallery.

`A4Mock` is decorative. It never reads a real template, asset, patient, or prescription. It has no API call, no context, and no randomized or time-based content.

## 6. Reuse, do not rebuild

| Existing symbol | Decision |
|---|---|
| `forms::A4Preview` | Keep as the real document renderer `22-V` and `23-W` use. `A4Mock` is not a second document engine; it is a fixed marketing illustration with no pagination or template logic. |
| `forms::TemplateDocumentForm` | Keep. This spec does not touch it. |
| `ButtonLink`, `Button`, `Icon` | Consume `15-O`; do not duplicate their markup or focus ring. |
| `PageShell`, `Card`, `ActionRow`, `SectionBand` | Consume `16-P` where needed; do not redefine them. |
| `.ui-container`, `.ui-stack` | Consume `15-O`; marketing sections rely on their width and rhythm. |
| `Navbar`, `Route`, `App` | Keep. `20-T` replaces the shell markup; this spec adds no route. |
| `.a4-*` CSS and A4 geometry | Keep unchanged. Marketing classes are separate and must not override the real A4 projection. |
| `SyncStatus` | Not used here. `18-R` moves it. |

There is one styling system. Extend `assets/main.css` and the new `src/ui` modules only. Do not add a CSS framework, another stylesheet, or an image/icon package.

## 7. Component surface

Every symbol is re-exported from `src/ui/mod.rs`. Every enum derives `Clone, Copy, PartialEq, Eq`. Every focusable control uses the `15-O` focus ring and no drop shadow.

### Brand

- `LogoLockup`: no props. Renders a Dioxus `Link` to `Route::Home {}` with `.ui-logo` and `aria-label="PersonaSanus — inicio"`. Contains `.ui-logo-mark`, a 22px mint circle holding a 14px Deep Teal `Cross` glyph that is `aria-hidden`, and `.ui-logo-word` with the exact wordmark `PersonaSanus` at 22px mint 500 Deep Teal. Hover color is Bright Teal. The cross is the only decorative mark.
- `TrustSecurityRow`: no props. Renders `div.ui-trust` centered with a `Shield` icon at 18px Deep Teal, then the exact text `Datos protegidos · Ley Orgánica de Protección de Datos Personales` at 14px 500 Ink, then `span.ui-trust-platforms` with `Command`, `Windows`, and `Globe` icons at 18px Mist and 12px gaps. Includes the visually hidden platform label `"macOS, Windows, Web"`.
- `TrustLogoStrip`: `caption: String = "Con la confianza de…"`, `logos: Vec<TrustLogo>`. Renders `div.ui-trust-strip` with `p.ui-trust-caption` and one centered wrapping logo row. Text logos use 18px mint 500 Ink. An image logo uses `img` with `max-height: 28px`, `width: auto`, grayscale, and opacity .9. No borders and no cards. The gallery demo passes five fictitious wordmarks and the note `Logos de ejemplo; no son clientes reales.`; Home never renders this component.
- `TrustLogo`: `name: String`, `src: Option<String>`. Derives `Clone, PartialEq`.

### Marketing

- `Hero`: `title: String`, `subtitle: String`, `children`. Renders `section.ui-hero` with `h1.ui-display` title, `p.ui-hero-sub` subtitle, and the children centered below. Width is min(calc(100% - 48px), 720px), centered, white. The title is grenette 400, clamp(46px, 7vw, 60px), line-height 1.05, Ink, balanced. The subtitle is 18px/1.5 Mist, max 600px. Children may include `ActionRow`, microcopy, and `TrustSecurityRow`.
- `SectionHeader`: `title: String`, `subtitle: Option<String>`. Renders `div.ui-section-head` centered, max 640px, with `h2.ui-section-title` and optional `p.ui-section-sub`. The title is grenette 400 46px/1.15 Ink, 34px below 760. The subtitle is 18px/1.5 Mist, max 560px. No decorative element above the title.
- `NumberedStepCard`: `number: u8`, `title: String`, `body: String`, `accent: AccentTone`, `mock: A4MockKind`, `tag: Option<String>`. Renders `article.ui-step` centered: `A4PreviewCard { accent, tag }` containing `A4Mock { kind: mock }`, then `div.ui-step-number` with the number, `h3.ui-step-title` title, and `p.ui-step-body` body. No card background. Number is a 28px circle with 1px Ink border and 14px mint 500 text; title is 20px mint 500 Ink; body is 16px mint 400 Mist.
- `A4PreviewCard`: `accent: AccentTone = Mint`, `tag: Option<String>`, `children`. Renders `div.ui-a4-card.ui-a4-card--{accent}`, then `div.ui-a4-sheet` containing children, and optional `VerificationTag { label }` absolutely at top 8px right 12px. The card is 24px radius with 24px padding, no border and no shadow. Accent Mint, Cream, Blossom, or Conic.
- `A4Mock`: `kind: A4MockKind`. Renders one static fake document inside `.ui-a4-sheet`; no template data, asset lookup, or API. `Patient` shows kicker `FICHA DEL PACIENTE`, name `María Fernanda Ruiz`, and rows `Cédula` `0000000000`, `Edad` `34 años`, `Especialidad` `Medicina general`. `Medication` shows kicker `RECETA`, row `Diagnóstico` `Faringitis aguda`, medication `1. Amoxicilina 500 mg`, and detail `Cada 8 horas · 7 días`. `Verification` shows kicker `VERIFICACIÓN`, a CSS-only 7×7 grid QR glyph (no image asset and no extra gradient), code `PS-7K2F-9Q`, and a 44px conic ring containing a `Check` icon.
- `VerificationTag`: `label: String`. Renders `span.ui-verification-tag` with Plum background, Paper 13px 500 text, padding 6px 12px, radius 9999, and a 6px triangular tail below center pointing down in Plum. `A4PreviewCard` positions it absolutely.
- `AccentTone`: `Mint`, `Cream`, `Blossom`, `Conic`. Default `Mint`. Class suffix lowercase.
- `A4MockKind`: `Patient`, `Medication`, `Verification`.

## 8. Page adoption

None. `Home` and `Verify` still render legacy markup; `20-T` adopts them. The only page edit allowed by this spec is the debug-only gallery wiring in `19-S`; no release-visible marketing page changes here.

## 9. CSS contract

| Selector | Values |
|---|---|
| `.ui-logo` | inline-flex center gap 8, Deep Teal, no underline; hover Bright Teal |
| `.ui-logo-word` | mint 500 22px/1.2, letter-spacing -.01em |
| `.ui-logo-mark` | 22px circle, Mint background, Deep Teal icon, grid center |
| `.ui-logo-glyph` | 14px square, block |
| `.ui-hero` | width min(calc(100% - 48px), 720px), margin auto, text-align center, flex column align center, padding 80px 0 64px |
| `.ui-display` | grenette 400 clamp(46px, 7vw, 60px)/1.05 Ink, letter-spacing -.015em, text-wrap balance, margin 0 |
| `.ui-hero-sub` | 18px/1.5 Mist, max 600px, margin 16px auto 0 |
| `.ui-hero .ui-actions` | width 100%, justify center, margin-top 32 |
| `.ui-hero .ui-trust` | margin-top 24px |
| `.ui-microcopy` | 14px/1.5 Mist, margin-top 16 |
| `.ui-section-head` | max 640px, margin 0 auto 40px, text-align center |
| `.ui-section-title` | grenette 400 46px/1.15 Ink, text-wrap balance, margin 0; 34px below 760 |
| `.ui-section-sub` | 18px/1.5 Mist, max 560px, margin 16px auto 0 |
| `.ui-steps` | grid repeat(3, minmax(0, 1fr)), gap 24, align-start; below 900 one column, max-width 420px, margin-inline auto |
| `.ui-steps-note` | 14px/1.5 Mist, text-align center, margin 24px 0 0 |
| `.ui-step` | text-align center, min-width 0 |
| `.ui-step-number` | 28px circle, 1px Ink, mint 500 14px/1, margin 24px auto 12px |
| `.ui-step-title` | mint 500 20px/1.25 Ink, margin 0 0 8 |
| `.ui-step-body` | mint 400 16px/1.5 Mist |
| `.ui-a4-card` | position relative, radius 24, padding 24, display flex center, no border, no shadow; accent Mint, Cream, Blossom, or Conic token |
| `.ui-a4-card .ui-verification-tag` | position absolute, top 8px, right 12px |
| `.ui-a4-sheet` | width 100%, aspect-ratio 210 / 297, Paper background, radius 8, padding 18px, color Ink, mint, flex column, gap 6px, overflow hidden, text-align left |
| A4 mock internals | kicker 10px 500 Mist uppercase tracking .08em; name 18px/1.25 500 Ink; row 11px/1.4 Mist flex space-between gap 8 border-top hairline padding-top 6; med 13px/1.45 500 Ink; detail 11px/1.4 Mist; qr 56px Ink margin-top auto; ring 44px conic, white check |
| `.ui-verification-tag` | relative inline-flex, Plum background, Paper text, mint 500 13px/1.2, padding 6px 12px, radius 9999; `::after` 6px triangle below center pointing down in Plum |
| `.ui-trust` | flex center wrap gap 12, mint 500 14px/1.4 Ink, text-align center; `.ui-trust-platforms` inline-flex gap 12 Mist |
| `.ui-trust-strip` | text-align center; caption 14px Mist margin 0 0 24; logos flex center wrap gap 24px 44px; text logo mint 500 18px/1.2 Ink; img max-height 28, width auto, grayscale, opacity .9 |

No new literal colour outside `:root`; no pure black; no drop shadow on any new selector; no gradient except the conic ring and `--gradient-ember-ring`. Cards and tags use the radius scale only; images use the 8px image radius.

## 10. Critical files and anchors

| File | Anchor |
|---|---|
| `persona-sanus/src/ui/mod.rs` | add `brand` and `marketing` modules and re-exports after `15-O`/`16-P` |
| `persona-sanus/src/ui/brand.rs` | New; `LogoLockup`, `TrustSecurityRow`, `TrustLogoStrip`, `TrustLogo` |
| `persona-sanus/src/ui/marketing.rs` | New; `Hero`, `SectionHeader`, `NumberedStepCard`, `A4PreviewCard`, `A4Mock`, `VerificationTag`, `AccentTone`, `A4MockKind` |
| `persona-sanus/assets/main.css` | `:root` 1-58; add brand and marketing declarations after `16-P` blocks |
| `context/Design.md` | §§Components, Tokens, Surfaces, Elevation, Layout |
| `context/sitemap.md` | route tables §1.1-1.3; conventions §5 |
| `context/PRD.md` | D-06, D-21, FR-050 |

## 11. Assumptions and contingencies

- Scope is fixed: full v1 adoption plus presentational v1.1 billing. Do not add a billing route, a plan picker that calls an endpoint, a card form, or a `sync_locked` gate. The billing gallery in `19-S` is the only billing surface.
- The repo has no grenette or mint webfonts. Keep the existing fallback stacks; do not fetch or add fonts.
- No partner-clinic logos exist, so Home must not show `TrustLogoStrip`. If real marks arrive, pass them only from a data source; never bake branded names into marketing markup.
- Dioxus 0.7 fallbacks: if `NavigationTarget` as a component prop does not accept `Route` directly, change `ButtonLink.to` and `NavLink.to` to `Route` and keep the internal `Link`; if `Option<Element>` props break the props macro, make `badge` or children required and pass `rsx! {}` at call sites; if `FormEvent` cannot be a component callback, keep the file `<input>` in each page and share only `Field` and `.ui-file-input` styling. None of these changes the visual contract or the one-primary rule.
- If a local licensed grenette or mint font later lands, point `--font-grenette` or `--font-mint` at it. No component markup changes.
- If browser verification finds a control not in the Design.md list but required by the existing page, use the nearest existing primitive (`Badge`, `Card`, `Button Secondary`) and do not add a new component file. The component set is closed after `19-S`.

---

*Housekeeping: once a spec is fully done with no open divergences, move its
file to `specs/done/` (same file name, e.g.
`specs/done/17-Q-ui-brand-and-marketing-components.md`).
Specs with open divergences stay in `specs/` next to `TEMPLATE.md` until the
tracking file is cleared.*
