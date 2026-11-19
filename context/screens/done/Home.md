# PersonaSanus — Home (`/`)

| Field | Value |
|---|---|
| Source | `context/sitemap.md` §1.1 (`/` Home), §2 (nav by role), §5 (conventions); `context/Design.md` (Hero, Section Header, Numbered Step Card, A4 Preview Card, Verification Tag, Green Pill Badge, Logo Lockup, Trust & Security Badge Row, Tokens, Layout, Do/Don't); `context/PRD.md` §1, §3, §4 flows 1/4/6, FR-001/002/010/013/021/022/030–034/043/050, D-04, D-06, D-21 |
| Component | `Home` — `persona-sanus/src/main.rs:1282`; primitives planned in `persona-sanus/src/ui/` (`Hero`, `SectionBand`, `SectionHeader`, `NumberedStepCard`, `A4PreviewCard`, `A4Mock`, `VerificationTag`, `VerificationCodeForm`, `TrustSecurityRow`, `LogoLockup`, `PillBadge`, `ActionRow`, `ButtonLink`, `LinkAction`) |
| Route | `/` — public, no auth guard |
| Layout | Inside `#[layout(Navbar)]`. Root is `<div id="home-page">`; never wrapped in `PageShell` — the tinted bands must stay full-bleed. Page max-width 1200px via `.ui-container` inside each band |
| Locale | `es-EC` on every string. `receta`, `cédula` keep Ecuador Spanish spelling |
| Status | 🟡 Partial. Shipped: minimal `#hero` with the wordmark, one English line, and links to `/login` + `/dashboard` (`assets/main.css:196-229`). Target described below; copy frozen by `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` step 11 and this file |

## 1. Purpose

Landing screen for three audiences at once: the unauthenticated clinic visitor evaluating the product, the signed-in staff member who wants the fastest path into their work, and the public verifier who arrives with a printed code. One job per visitor: emit a prescription, or verify one.

The Home screen never renders: PHI (§4.2); billing, price, trial, or plan copy — v1 is a free beta, PRD D-04, §5.7 deferred; signature or stamp imagery — the doctor signs the printed paper, D-06; any rich-text affordance — all clinical text entry stays in forms and inputs, D-21. It never states a security or legal guarantee beyond the data-protection law line.

## 2. Structure

Vertical, centered, symmetric. Bands alternate white and tinted wash. No sidebar, no hero image, no photography. The region order is identical for every visitor; the state groups (§2.1–§2.3) change only the Hero primary CTA row of §3.3.

| Order | Region | `id` / class | Band tone | Contents |
|---|---|---|---|---|
| — | Navbar (layout) | `#navbar` · `header.ui-navbar`, inner `.ui-navbar-inner` | Paper `#ffffff`, bottom hairline 1px `#e5e7eb` | Owned by `context/screens/Header.md` — `LogoLockup` + `PillBadge`, zero to five role-guarded `NavLink`s when a session exists, session-aware right group |
| 1 | Hero | — | Paper `#ffffff` | `h1` display headline, subtitle, `ActionRow` (one filled teal + one outlined), microcopy line, `TrustSecurityRow` |
| 2 | Cómo funciona | `#como-funciona` | Paper `#ffffff` | `SectionHeader` + `ui-steps` grid of three `NumberedStepCard`s + centered note |
| 3 | Verifica una receta | `#verificar` | Mint `#b9ffe8` | `SectionHeader` + `Card` holding `VerificationCodeForm` (Secondary) + `TrustSecurityRow` |

Nothing else renders on this screen. `TrustLogoStrip` is excluded until real clinic marks exist (`UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` No-go). The header band renders above this screen as the layout's first element; its structure and copy are owned by `context/screens/Header.md`. The public footer band renders as the page's last element; its structure and copy are owned by `context/screens/Footer.md`.

### 2.1 Signed-out visitor (default)

Bands 1–3 as ordered. The Hero CTA is the `no session` row of §3.3 (`Emitir receta` → `/login`). The Hero secondary CTA anchors to band 3, the visitor's verify path. No region reads a session that does not exist.

### 2.2 Signed-in staff

Same three bands, same order. Only the Hero CTA row changes, resolved by role in §3.3 rows 2–4; no other Home region reads the session. The sync indicator, when a session exists, is rendered by the navbar band, not by Home (§4.1, `context/screens/Header.md` §4).

### 2.3 Public verifier

Band 3 renders for every visitor regardless of session or role — it is the surface for someone arriving with a printed code, and the anchor target of the Hero secondary CTA. Submitting the code navigates to `/verify/:code`; the verification result surface is `context/screens/Verify.md`, never Home. Home renders no verification result.

## 3. Copy text

All strings verbatim, `es-EC`. Punctuation is part of the copy.

### 3.1 Navbar

The navbar band is shared chrome, not Home copy: its strings, role guards, session states, and style are owned by `context/screens/Header.md` §2–§5. Home renders nothing into the bar; it only reads the same `Signal<Option<Me>>` context the bar uses (§3.3, §7).

### 3.2 Hero

| Element | Copy |
|---|---|
| Headline (`h1`) | `Recetas claras, verificables y listas para firmar.` |
| Subtitle | `PersonaSanus reúne pacientes, plantillas y recetas en un espacio sereno, pensado para el trabajo clínico en Ecuador.` |
| Primary CTA | see §3.3 |
| Secondary CTA | `Verificar una receta` → anchor `#verificar` |
| Microcopy line | `Diseñado para clínicas y consultorios · Datos protegidos` |
| Trust row | `Datos protegidos · Ley Orgánica de Protección de Datos Personales` + visually hidden platform label `macOS, Windows, Web` |

### 3.3 Primary CTA — resolved before the `rsx!` block, in this exact order

| Condition (in order) | Label | Target |
|---|---|---|
| No session | `Emitir receta` | `/login` (`Route::AuthPage {}`) |
| Session and `Me::can_create_prescriptions()` (role `doctor` or `nursing`) | `Emitir receta` | `/prescriptions/new` (`patient_id: None, template_id: None`) |
| Session with a role that cannot create (`administrator`, `secretary`) | `Ver recetas` | `/prescriptions` (`status: None`) |
| Session with no profile or no role | `Completar perfil` | `/settings/profile` |

Exactly one filled teal button per viewport, always this one. The Secondary CTA and the verify-band button are outlined.

### 3.4 Cómo funciona (`#como-funciona`)

| Element | Copy |
|---|---|
| Section headline (`h2`) | `Cómo funciona` |
| Section subtext | `Del paciente a la receta firmada en tres pasos.` |

| Step | Title | Body | Card wash | Mock | Tag |
|---|---|---|---|---|---|
| 1 | `Elige el paciente` | `Busca por nombre o cédula, o crea la ficha en segundos. El historial de recetas queda junto al paciente.` | Mint `#b9ffe8` | `A4MockKind::Patient` | — |
| 2 | `Completa la receta` | `Tu plantilla con tus variables: diagnóstico, instrucciones y medicamentos, escritos en un formulario simple.` | Cream `#fffded` | `A4MockKind::Medication` | — |
| 3 | `Firma y verifica` | `Vista previa A4, PDF listo para imprimir y un QR que cualquiera puede verificar sin exponer datos del paciente.` | Blossom `#ffc3e6` | `A4MockKind::Verification` | `Verificable` |

Step bodies describe FR-001/002 (search and dedup by cédula), FR-010/021 (form-based entry, variables), and FR-022/030/033/050 (A4 preview, PDF, QR verification). No claim about offline behavior in v1 copy beyond §4.3 indicators.

| Element | Copy |
|---|---|
| Note under the grid | `Vistas de ejemplo del formato A4.` |

### 3.5 Verifica una receta (`#verificar`)

| Element | Copy |
|---|---|
| Section headline (`h2`) | `Verifica una receta` |
| Section subtext | `Ingresa el código impreso junto al QR. La página muestra únicamente el médico, la fecha y el número de la receta: nunca los datos del paciente.` |
| Field label | `Código de verificación` |
| Field placeholder | `PS-7K2F-9Q` |
| Button | `Verificar` (Secondary / outlined) |
| Empty-field error | `Ingresa el código de verificación.` |

## 4. Displayed information

### 4.1 What the screen shows

| Element | Source | Value shown |
|---|---|---|
| CTA label + route | session (`GET /api/auth/get-session`) and `Me.role` via `can_create_prescriptions()` | one of the four rows in §3.3 |
| Step-card A4 mocks | hardcoded constants in `A4Mock`; never from the API | `Patient`: `FICHA DEL PACIENTE`, `María Fernanda Ruiz`, rows `Cédula` / `Edad` / `Especialidad`. `Medication`: `RECETA`, `Diagnóstico / Faringitis aguda`, `1. Amoxicilina 500 mg`, `Cada 8 horas · 7 días`. `Verification`: `VERIFICACIÓN`, a CSS QR glyph, `PS-7K2F-9Q`, a 44px conic ring with a check |
| Verification tag on step 3 | static | `Verificable` |
| Verify form | local signal only | typed code; on submit it is trimmed, uppercased, and pushed to `/verify/{CODE}` (`Route::Verify`) |
| Trust row | static | shield glyph + law line + macOS/Windows/Web icons |
| Sync indicator (signed-in only) | rendered by the header band, not by `Home` — `context/screens/Header.md` §4 | `En línea` / `Sincronizando` / `Sin conexión` / `Error de sincronización` + pending count (FR-043); counts are desktop-only; not rendered for signed-out visitors |

The A4 mock content is fictional and constant. It is the only patient-shaped text on this screen, and it never touches the database, the API, or the outbox.

### 4.2 Hard rules

- No PHI: no real name, cédula, diagnosis, medication, or photo in any Home string, mock, alt text, or URL. Only the opaque verification code travels in the URL (sitemap §5, FR-033).
- No signature or stamp imagery anywhere — the doctor signs the printed paper (D-06).
- No billing, price, plan, trial, `sync_locked`, or image-quota copy in v1 (D-04).
- No invented proof: no fake partner logos, no testimonials, no counters ("+500 clínicas"), no medical-accuracy or legal-validity guarantees (PRD §15 O-01 is open).
- No text-entry surface other than the verification code input; all clinical text entry stays in forms and inputs (D-21), and Home contains no rich-text affordance.
- Verification copy must state the disclosure limit exactly once on this screen (§3.5) — doctor, date, prescription number, nothing else.
- Home renders no verification result; the result belongs to `/verify/:code` (`context/screens/Verify.md`).

### 4.3 Dynamic states

| State | Home behaviour |
|---|---|
| Session check in flight | Navbar renders the signed-out right group; the Hero CTA renders the `no session` row. The page never blocks on the session fetch and never flashes a second headline |
| Signed-in, role missing or `profiles` incomplete | CTA becomes `Completar perfil` → `/settings/profile` |
| Signed-in, desktop offline | CTA still points at `/prescriptions/new` — offline issuance is supported (FR-040); only the sync indicator changes |
| Verification submit with empty code | `ErrorText` `Ingresa el código de verificación.` below the field; no navigation |
| Anchor click `Verificar una receta` | in-page scroll to `#verificar`; no route change |

## 5. Style

Every value is a Design.md token already defined in `persona-sanus/assets/main.css` `:root`. No new color, font, icon package, CDN, or webfont.

### 5.1 Color

| Use | Value | Token |
|---|---|---|
| Page canvas, card surfaces | `#ffffff` | `--color-paper` |
| Headings, body on white | `#061d29` | `--color-ink` |
| Muted body, section subtext, step body, outlined-button border and text | `#425d6d` | `--color-mist` |
| Hairlines: navbar bottom rule, card borders | `#e5e7eb` | `--color-hairline` |
| Filled CTA, active nav, wordmark, shield glyph | `#006e75` | `--color-deep-teal` (hover `#0b978e` `--color-bright-teal`) |
| Mint band, step-1 card wash, pill badge | `#b9ffe8` | `--color-mint-glass` |
| Step-2 card wash | `#fffded` | `--color-cream` |
| Step-3 card wash | `#ffc3e6` | `--color-blossom` |
| `Verificable` tag | `#4d0037` | `--color-plum` |
| Verification ring | `conic-gradient(rgb(255,176,164) 0deg, rgb(255,73,44) 0deg, rgb(255,73,44) 360deg)` | `--gradient-ember-ring` |

Never `#000000`. Never a chromatic border.

Measured contrast (WCAG AA needs ≥ 4.5 for body): Ink on white 17.28 · Ink on mint 15.25 · Ink on cream 16.89 · Ink on blossom 11.68 · Mist on white 6.96 · Mist on cream 6.80 · Mist on mint 6.14 · white on deep teal 6.02 · white on plum 15.32.

### 5.2 Typography

| Element | Family | Weight | Size / line-height |
|---|---|---|---|
| Hero `h1` | `--font-grenette` (fallback `Source Serif 4` → `ui-serif, Georgia`) | 400, never bold | `clamp(46px, 7vw, 60px)` / 1.05, `text-wrap: balance` |
| Hero subtitle | `--font-mint` | 400 | 18px / 1.5, max-width 600px, Mist |
| Section headline (`h2`) | `--font-grenette` | 400 | 46px / 1.15, centered |
| Section subtext | `--font-mint` | 400 | 18px / 1.5, max-width 560px, Mist |
| Step title | `--font-mint` | 500 | 20px / 1.25, Ink |
| Step body | `--font-mint` | 400 | 16px / 1.5, Mist |
| Step number in circle | `--font-mint` | 500 | 14px |
| Button label | `--font-mint` | 500 | 16px |
| Microcopy, steps note | `--font-mint` | 400 | 14px, Mist |
| Pill badge | `--font-mint` | 500 | 12–13px, Ink on mint |
| Verification tag | `--font-mint` | 500 | 13px, white on plum |
| Trust row text | `--font-mint` | 500 | 14px, Ink |
| Code input and label | `--font-mint` | 400 / label 500 14px | 16px input, Ink on white |

Reality check: `assets/main.css` declares no `@font-face`. `grenette` and `mint` fall back to `Source Serif 4` / `Inter`, then system serif/sans — Georgia renders the display face on machines without the custom fonts. `assets/fonts/NotoSans-Regular.ttf` is embedded by the PDF renderer, not loaded by the web page. Bundling the licensed faces is a separate change.

### 5.3 Geometry

- Base unit 4px. Hero: centered column, max-width 720px, `--spacing-80` block padding, `--spacing-24` inline padding.
- Bands: 80px vertical padding, 64px below 640px wide; inner `.ui-container` max-width 1200px; 64–80px between sections.
- Steps grid: 3 columns, `--spacing-24` gap, single column below 860px, centered.
- Card radii: 24px for `A4PreviewCard`, `Card`, and section cards; 8px for the inner A4 mock, buttons, and inputs; 9999px for pills and tags.
- Padding: buttons 12px 24px; cards 24px; pill 4px 12px; tag 6px 12px.
- Number circle: 28px diameter, 1px `#061d29` border, transparent fill.
- Elevation: flat. Separation comes from hairlines and tint contrast — no drop shadow on any Home element.

### 5.4 Iconography and imagery

Line icons only, 1.5px stroke, monochrome Ink or Deep Teal (Mist for the platform glyphs). No photography, no illustration, no patient imagery. The only document visuals are the three decorative A4 mocks and the static QR glyph inside the Verification mock.

## 6. Accessibility

- Heading order `h1` (Hero) → `h2` (`Cómo funciona`, `Verifica una receta`); the mock document titles are styled spans, not headings.
- Keyboard: tab order navbar → primary CTA → secondary CTA → verify field → verify button; visible 1px `#006e75` focus ring on every control.
- The A4 mocks are `aria-hidden` decoration; the `Verificable` tag text carries the meaning.
- Anchor CTA is a real `<a href="#verificar">`; the code field is a real labeled `input` with `autocomplete="off"`, `spellcheck="false"`, and an inline error wired by `aria-describedby`.
- Icon-only glyphs carry a visually hidden label (`macOS, Windows, Web`).

## 7. Implementation notes

- Current shipped code is the minimal `#hero` block in `persona-sanus/src/main.rs:1282-1295`; the target markup is step 11 of `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md`, which builds the primitives this spec consumes (`persona-sanus/src/ui/` does not exist yet).
- `Home` stays inside the Navbar layout — the bar itself is specified in `context/screens/Header.md`. `context/architecture.md:227` records that `#[layout(Navbar)]` is not closed with `#[end_layout]`, so the navbar currently wraps `/` and `/login` too — Home assumes a navbar is present.
- CTA resolution reads the same `Signal<Option<Me>>` context the Navbar uses; Home adds no fetch, no endpoint, and no permission logic beyond `can_create_prescriptions()`.
- Verify, in order:
  - `cargo check` passes;
  - `dx build --release --fullstack` succeeds;
  - `curl -s http://127.0.0.1:8080/` contains `Recetas claras, verificables y listas para firmar.` and exactly one `ui-button--primary`;
  - `caveman-blocks run persona-smoke --dir persona-sanus --port 8091` returns `"ok": true` (its `home` stage covers `/`);
  - `caveman-blocks run markdown-table-check --path context/screens/Home.md` passes.
- Do not add: a `TrustLogoStrip`, a pricing or trial band, a screenshot carousel, a newsletter form, a verification result on this page, or any second filled teal action.

## 8. Spec references

| Spec | Connection |
|---|---|
| `specs/done/15-O-ui-tokens-actions-and-formatting.md` | `15-O` supplies the tokens, `Button`, `ButtonLink`, `LinkAction`, and format helpers Home CTA row and copy use. |
| `specs/done/16-P-ui-forms-feedback-and-layout.md` | `16-P` supplies `SectionBand`, `Card`, `ActionRow`, `Notice`, `EmptyState`, and `LoadingState` the Home bands compose. |
| `specs/done/17-Q-ui-brand-and-marketing-components.md` | `17-Q` builds `Hero`, `SectionHeader`, `NumberedStepCard`, `A4PreviewCard`, `A4Mock`, `VerificationTag`, `TrustSecurityRow`, and `LogoLockup`, Home core regions. `TrustLogoStrip` is never rendered by Home (`17-Q` §7). |
| `specs/done/18-R-ui-domain-and-billing.md` | `18-R` builds `VerificationCodeForm`, the form rendered in the `#verificar` band (Secondary variant on Home). |
| `specs/done/20-T-shell-auth-home-and-verify.md` | `20-T` adopts Home: one filled primary CTA, no partner strip, no `TrustLogoStrip`, es-EC copy frozen by the UI plan. |
| `specs/done/25-Y-ui-legacy-css-cleanup-and-docs.md` | `25-Y` removes the legacy page-ID width rules in `assets/main.css`, leaving Home bands on tokens and `.ui-container`. |
