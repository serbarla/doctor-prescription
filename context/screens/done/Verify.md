# PersonaSanus — Verify (`/verify/:code`)

| Field | Value |
|---|---|
| Source | `context/sitemap.md` §1.1 (`/verify/:code`, `:74`: public verifier, QR target, four statuses, no PHI, per-IP rate limit, typed-code re-entry, only the path form); §2 (public verifier, no account); §3 flow 4 (`:158`); §4 M1/M3 (`:174`, `:176`); §5 deep links (`:188`); `context/PRD.md` FR-033/FR-034/FR-050 (`:85-:95`), §9 minimization (`:212`), D-04/D-06/D-21 (`:291`, `:293`, `:308`); `context/Design.md` Logo Lockup, Trust & Security Badge Row, Tokens, Surfaces, Elevation, Layout, Do/Don't; `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` step 12 (`:202`), Page adoption shared rules (`:212`), component contracts (`:99`, `:124`, `:149-:153`); `context/screens/INDEX.md` §2.2 |
| Component | `Verify` — `persona-sanus/src/main.rs:1297`; calls `api::fetch_verification` (`persona-sanus/src/api.rs:980`). Primitives are planned in `persona-sanus/src/ui/` (the directory does not exist yet): `LogoLockup` (`brand.rs`), `PillBadge` (`feedback.rs`), `LoadingState`, `Notice` (`feedback.rs`), `VerificationResultCard`, `VerificationSeal`, `VerificationCodeForm` (`domain.rs`), `SectionCard` (`layout.rs`), `TrustSecurityRow` (`brand.rs`) |
| Route | `/verify/:code` — public, no auth guard, no role check, no session requirement. `code` is a required path param leaf. Declared `main.rs:49-50`, before `#[layout(Navbar)]` at `:51`. Only the path form is routed; there is no hash form and no code-less `/verify` page. |
| Layout | No `PageShell` and no `#[layout(Navbar)]`: the route is declared before the layout marker at `main.rs:51`, so it renders standalone. Target root `<div id="verify-page" class="ui-public">`; `.ui-public-head` (max 1200px, 24px padding, bottom hairline); `.ui-public-main` (max 720px, margin 48px auto 80px, 24px inline padding) holds the conditional region and the re-entry `SectionCard`; the footer band is appended inside the root by `Footer.md` §2.1 (target; component not shipped) with the disclosure line only |
| Locale | `es-EC` on every string. `receta`, `cédula` keep Ecuador Spanish spelling. Dates `DD-MM-AAAA`; times 24-hour. The date detail uses `format_date` |
| Status | ✅ Spec complete (sections 1–7). Route status: ⬜ — Planned (`context/sitemap.md` §1.1 `:74`); shipped: minimal `Verify` with four `es-EC` statuses, three English detail labels, `Cargando...`, a bare error paragraph, the typed-code field, and per-IP rate limiting (`main.rs:1297-1374`, `api.rs:973-984`, `service.rs:3129-3203`, `verify_server.rs:18-117`). Target: public shell, result card, re-entry section card, footer disclosure band |
## 1. Purpose
One public surface answers one question for the holder of a printed prescription: does the opaque code next to the QR correspond to a real, currently valid PersonaSanus record, and if not, what happened to it? Ordered jobs:

1. Read the `code` path param on mount and fetch its public record through `api::fetch_verification`; never require an account or a session.
2. While the fetch is in flight, show the loading label (§3.3) in the conditional region and keep the re-entry form usable.
3. Render exactly one of four known outcomes — `valid`, `voided`, `superseded`, `not found` — with a fixed `es-EC` title and body. Any other status string becomes the generic outcome (§3.3).
4. For `valid`, `voided`, and `superseded` only, show the three permitted professional facts (§3.4). Use the em dash fallback for a missing value. `not found` and unknown statuses show no details.
5. Offer FR-034 re-entry: the printed code can be typed when the QR cannot be scanned or the camera path is unavailable.
6. Enforce minimization and abuse limits server-side. The screen displays no PHI and provides no shortcut around the per-IP rate limiter; a 429 surfaces its own message.

It never renders:

- PHI: no patient name, cédula, birth date, diagnosis, medication, notes, clinic-internal identifiers, or patient photo. The professional name, issue date, and prescription number are the entire disclosed record.
- Billing, price, trial, plan, entitlement, or `sync_locked` copy — v1 is a free beta (D-04).
- Signature or stamp imagery; the doctor signs the printed paper (D-06). The doctor name is text.
- Rich text or a rich-text surface; the sole input is a plain single-line text field (D-21).
- A PDF or QR download, a print affordance, a share affordance, or a generated QR image. This page verifies; it does not produce or export the document.
- A second route form: no code-less `/verify`, no hash fragment, no query-param verification page.
- A patient or medication field, and no "search another patient" affordance.
- The navbar, app-shell navigation, or a public marketing hero.
- Raw audit metadata, actor identity, or IP address.
- A fifth status distinction. Unknown status strings collapse into the generic state.

## 2. Structure
One public column, centered, symmetric. The head, main, and footer are landmarks. The re-entry card is always present; only the region above it changes.

| Order | Region | id / class | Band tone | Contents |
|---|---|---|---|---|
| 1 | Public head | `.ui-public-head` (`header`) | Paper `#ffffff`, bottom hairline 1px `#e5e7eb` | `LogoLockup` (wordmark + cross glyph, links `/`) and `PillBadge` `Receta verificable` at opposite ends |
| 2 | Public main | `.ui-public-main` (`main`), inner `ui-stack` | Paper | Visually hidden page `h1`; then the conditional result region |
| 3 | Conditional result region | `.ui-verify-inline`, inner `.ui-verify-card` | Paper | Exactly one of: `LoadingState`; `Notice` Error carrying the fetch message; `VerificationResultCard` |
| 4 | Re-entry card | `SectionCard` `.ui-card`; form `.ui-code-form` | Paper, hairline | `h2`; `VerificationCodeForm` (Primary); `TrustSecurityRow`; optional field `ErrorText` |
| 5 | Footer band | `<footer id="ui-footer">` | Paper, top hairline | Disclosure line only, target and not shipped (owned by `context/screens/done/Footer.md` §2.1) |
Nothing else renders. There is no navbar band: `/verify/:code` is declared before `#[layout(Navbar)]` (`main.rs:49-51`), so the shell bar does not wrap it. The public head is the page's only brand row. No public marketing hero, no hero image, no trust-logo strip, no sidebar, no duplicate logo in the footer.

### 2.1 Session and role
`Verify` reads no session, `Me`, role, clinic, or permission context. The same markup, copy, and requests render for a signed-out visitor, a Doctor, Nursing, Secretary, and an Administrator. There is no session-check loading state and no redirect on a signed-in session: this is a public endpoint, not an authenticated page. The only access boundary is the server's per-IP limiter; the client never gates the request.

### 2.2 Verification status
| Status | Result card | Seal | Details |
|---|---|---|---|
| `valid` | valid row of §3.3 | Mint surface, Deep Teal check, 80px conic ring | all three labels (§3.4) |
| `voided` | voided row of §3.3 | Blossom surface, Plum close | all three labels (§3.4) |
| `superseded` | superseded row of §3.3 | Cream surface, Ink arrow | all three labels (§3.4) |
| `not found` | not-found row of §3.3 | Paper surface, Mist Qr glyph, hairline | none |
| other | generic row of §3.3 | Paper surface, Mist Info glyph, hairline | none |
The status token is a lookup key only; it is never printed. The card tone is carried by the decorative seal, whose meaning is also stated in words by the title and body. The card surface stays Paper so two tinted bands never stack.

### 2.3 Fetch surface
Initial mount has `loaded = false`, no result, and an empty message: the conditional region is the `LoadingState` only, never blank. A successful fetch sets the result and becomes `VerificationResultCard`. A failed fetch sets the message and becomes `Notice` Error; the result card does not render beside it. Each new fetch begins by setting `loaded` false, clearing the message, and clearing the old result, so no previous outcome survives a code change or a retry. The page never reads the desktop local cache or the outbox: `pending_sync`, `failed`, and offline provisional rows are not verification states.

### 2.4 Code entry
The field starts empty even when the route already carries a code; the route code is displayed only through the result, never echoed above the form. Empty submit shows `ErrorText` and sends no request. A non-empty submit trims leading and trailing whitespace, uppercases, writes the value into `requested`, and calls `navigator.replace(Route::Verify { code })`. Internal spaces and dashes are left for the server canonicalizer. Re-submitting the identical canonical value still writes `requested`, so the same-page case refetches instead of silently doing nothing. A different code replaces the history entry, clears the old result at fetch start, and shows the new outcome. The field accepts an opaque code of any length; there is no client-side prefix, length, or character validation and no enumeration of candidate codes. The typed value is not persisted.
### 2.5 Web, desktop, and width
The web target is the public verifier. The desktop binary renders the same component and the same networked `fetch_verification` call; there is no local lookup, cache, or provisional branch to reconcile. Offline on either target is the fetch-error state. On mobile the public head keeps its 24px padding, the main column is single, the code form stacks the input above the Primary button, and the details grid becomes one column. Copy and region order do not change by surface or width.
## 3. Copy text
All strings are `es-EC`; punctuation, ellipsis, and accents are part of the copy. Chrome strings are referenced, not restated: the navbar lives in `context/screens/done/Header.md` §3 and is absent here; footer copy is owned by `context/screens/done/Footer.md` §3.

### 3.1 Public head
| Element | Copy | Target |
|---|---|---|
| Wordmark | `PersonaSanus` — `aria-label="PersonaSanus — inicio"` | `/` (`Route::Home {}`) |
| Pill badge | `Receta verificable` | none — static span |
### 3.2 Page heading
| Element | Copy |
|---|---|
| Page `h1`, visually hidden | `Verificación de receta` |
The page has no visible title or subtitle. This hidden `h1` is the only `h1`; the result title and the re-entry section title are `h2`.
### 3.3 Conditional region
| Element | Copy |
|---|---|
| Loading label | `Verificando receta…` |
| Result eyebrow | `VERIFICACIÓN PÚBLICA` |
| Fetch-failure notice body | — (dynamic; §4.1) |
| Rate-limited body (429) | `Demasiadas solicitudes. Inténtalo de nuevo en un minuto.` |
| Unreadable verification response | `No se pudo verificar la receta.` |

| Status | Title (`h2`) | Body |
|---|---|---|
| `valid` | `Receta auténtica` | `Esta receta fue emitida y firmada por un profesional autorizado en PersonaSanus.` |
| `voided` | `Receta anulada` | `El médico emisor anuló esta receta. El documento ya no es válido.` |
| `superseded` | `Receta reemplazada` | `Esta receta fue reemplazada por una corrección posterior. Consulta la versión vigente.` |
| `not found` | `Receta no encontrada` | `El código no corresponde a una receta emitida. Revisa el código impreso junto al QR.` |
| other | `Receta no verificable` | `No pudimos determinar el estado de este documento.` |
### 3.4 Result details
| Element | Copy |
|---|---|
| Details label | `Médico` |
| Details label | `Fecha de receta` |
| Details label | `Número de receta` |
| Missing value | `—` |
These three rows render only for `valid`, `voided`, and `superseded`. They never render for `not found` or an unknown status.
### 3.5 Re-entry card
| Element | Copy |
|---|---|
| Section heading (`h2`) | `Verificar otra receta` |
| Field label | `Código de verificación` |
| Field placeholder | `PS-7K2F-9Q` |
| Empty-field error | `Ingresa el código de verificación.` |
| Submit button (Primary, default and in flight) | `Verificar` |
| Trust row text (primitive-owned, plan `:124`) | `Datos protegidos · Ley Orgánica de Protección de Datos Personales` |
| Trust row hidden platform label | `macOS, Windows, Web` |
The code field also carries `autocomplete="off"` and `spellcheck="false"`; those are behavior attributes, not visible copy.
### 3.6 Footer
No footer copy belongs to this file. The band renders the disclosure line defined by `context/screens/done/Footer.md` §3, and only on `/verify/:code` (§2.1). It does not render the footer wordmark, the Home verify link, the law row, or any second `TrustSecurityRow`; the re-entry card's row is the page's one instance. The page must render the disclosure line, but it must not duplicate that string here.
## 4. Displayed information
### 4.1 What the screen shows
| Element | Source | Value shown |
|---|---|---|
| Route code | route param `code`, seeded into the `requested` signal (`main.rs:1303`) | never echoed in the body; sent percent-encoded as `/api/verify/{code}` by `fetch_verification` |
| Loading label | `loaded` signal is false | the loading string (§3.3) |
| Fetch error | `message` signal set from `api::fetch_verification`'s `Err` | the transport/server string verbatim, e.g. the 429 message |
| Eyebrow | static | the eyebrow (§3.3) |
| Result title/body | `verification.status` matched by `VerificationResultCard` | one row of §3.3; unknown becomes the generic row |
| Doctor | `verification.doctor_name: Option<String>` | professional display name, or `—` when absent |
| Date | `verification.issued_at: Option<String>` through `format_date` | `DD-MM-AAAA`, or `—` when absent |
| Prescription number | `verification.rx_number: Option<i32>` | six digits, zero-padded (`{value:06}`), or `—` when absent |
| Typed code | local `typed` signal in `VerificationCodeForm` | what the visitor types; trimmed and uppercased on submit only |
| Section title | static | the section heading (§3.5) |
| Trust line | static `TrustSecurityRow` | shield glyph + LOPDP text + platform glyphs with the hidden label |
| Disclosure line | `context/screens/done/Footer.md` §3 | legal line; footer-owned, not repeated here |
| Session/role | none read | identical for signed-in and signed-out visitors |
| Storage | none | no localStorage, cookie, service-worker, or result cache |
The four API status values are `valid`, `voided`, `superseded`, and `not found`. They are the only specialized outcomes; the match is exact and the client does not lowercase the token.
### 4.2 Hard rules
- Server-side authorization and rate limiting are the boundary. The endpoint is intentionally unauthenticated; the client never adds a session gate, role gate, clinic gate, or a client-only "verified" claim. A signed-in session grants nothing extra.
- The only route-revealed value is the opaque `code`. No patient name, cédula, diagnosis, medication, phone, address, or photo may enter the URL, the request body, the result card, the HTML title, or any log this screen owns.
- The screen never generates, encodes, downloads, or prints a QR or PDF. The seal's Qr glyph is a decorative icon, not a scannable code.
- Details are limited to the three labels in §3.4, and only for `valid`, `voided`, and `superseded`. `not found` and unknown statuses show none. A missing field shows `—`, never invented text.
- Any status outside those four renders the generic card. Do not add a further specialized status, a color-coded unknown branch, or a raw status label.
- One filled primary action per viewport: `Verificar` in the re-entry card. The logo is a text link, the seal is inert, and no second filled teal control renders. The form button stays Primary while a fetch is in flight.
- The page reads no session; signed-in and signed-out renderings are identical. It is not an authenticated shell and must not acquire shell chrome.
- The page stores nothing client-side: no localStorage, sessionStorage, cookie, service-worker cache, or in-memory result across navigations. A reload refetches. The desktop cache never hides a loss here because this screen has no offline fallback.
- Rate limiting is per IP and server-side (default 10 requests/60 s, `VERIFY_RATE_LIMIT_PER_MINUTE`); it covers both `/verify/` and `/api/verify/`. The client neither counts, resets, nor bypasses it, and a 429 is not treated as a PHI distinction.
- No client-side code enumeration: no prefix searches, no checksum/length validation, no lookup on input, and no suggested codes. One submit issues at most one `fetch_verification` for the composed path.
- No billing, price, trial, plan, entitlement, `sync_locked`, signature, stamp, rich-text, or contenteditable surface (D-04, D-06, D-21).
- No new route, no `/verify` code-less page, no query-param alternate form, no hash form; the typed code uses the same `/verify/:code` path as the QR (sitemap §5).
- The re-entry card always renders, including during a later load and after an error, so a failed code can be corrected on the same page.
- Audit status/actor/IP are server-side; the screen shows no audit metadata, raw IP, or attempt count.
- The footer is target-only and footer-owned. Until `Footer` ships, the page must not hand-roll the disclosure line; the body still renders the one `TrustSecurityRow`.
### 4.3 Dynamic states
| State | Screen behaviour |
|---|---|
| Session check not read | Renders immediately for every visitor; no session spinner, redirect, or role branch |
| Initial mount | `loaded` false, no result, empty message; only `LoadingState` in the conditional region |
| Loading a new code | Old result and old message are cleared first; loading replaces the card; the re-entry card remains |
| `valid` | Mint seal, valid title and authenticity body, three populated details |
| `voided` | Blossom/Plum seal, voided title and issuer body, three details |
| `superseded` | Cream/Ink seal, replaced title and correction body, three details |
| `not found` | Neutral Qr seal, not-found title and recheck body, no details, no enumeration |
| Unknown extra status | Neutral Info seal, generic title and body, no details; never the raw status |
| Fetch error / offline | `Notice` Error with the fetch message; no result card or stale details; the form remains |
| Rate limited (429) | Error notice with the 429 message (§3.3); no timer, countdown, or client retry loop |
| Typed field empty | No request; the empty-field `ErrorText` (§3.5) is announced below the field |
| Retype same code | `requested` is written unconditionally and the route is replaced with the canonical value; loading clears the old card and refetches |
| New code replaces old | `navigator.replace` avoids a new history entry; the old result is cleared at fetch start |
| Long code | Opaque input, no truncation, no echo, no length error; the server returns its normal outcome or 429 |
| Submit busy | `Verificar` stays enabled; the result region shows loading; no busy label or duplicate button |
| Success navigation | No navigation on a result; submitting replaces the current route with the canonical `/verify/:code` |
| Mobile | Single column, stacked code form, one-column details; order and copy unchanged |
| Signed-in vs. signed-out | Identical; no session read, email, sync indicator, or sign-out control |
| Pending sync / failed rows | Not applicable; the page never reads the outbox, prescription cache, or sync state |
| Status transitions | None client-side; a new status appears only after a new fetch. No polling, websocket, or auto-refresh |
## 5. Style
All values are `context/Design.md` tokens already defined in `persona-sanus/assets/main.css` `:root`. Paper surfaces, hairline separation, 24px cards, no shadow; the page is a result sheet, not a marketing hero.
### 5.1 Color
| Use | Value | Token |
|---|---|---|
| Canvas, head, cards, footer | `#ffffff` | `--color-paper` |
| Title, body, detail values | `#061d29` | `--color-ink` |
| Eyebrow, supporting copy, detail labels, footer disclosure | `#425d6d` | `--color-mist` |
| Card border, head rule, footer rule, not-found seal outline | `#e5e7eb` | `--color-hairline` |
| Primary `Verificar`, wordmark, cross glyph, valid check | `#006e75` | `--color-deep-teal` |
| Primary hover, focus outline | `#0b978e` | `--color-bright-teal` |
| Pill badge, valid seal surface | `#b9ffe8` | `--color-mint-glass` |
| Voided seal surface | `#ffc3e6` | `--color-blossom` |
| Voided close, empty-field error | `#4d0037` | `--color-plum` |
| Superseded seal surface | `#fffded` | `--color-cream` |
| Valid seal 80px ring | `conic-gradient(rgb(255, 176, 164) 0deg, rgb(255, 73, 44) 0deg, rgb(255, 73, 44) 360deg)` | `--gradient-ember-ring` |
| Focus halo | `rgba(185, 255, 232, 0.55)` | `--color-mint-halo` |
| Status seal tone | Mint / Blossom / Cream / Paper (neutral) | `--color-mint-glass` / `--color-blossom` / `--color-cream` / `--color-paper` |
Never `#000000`. Cards and controls have a 1px hairline border, never a chromatic border; the only saturated rings are the valid seal's decorative conic ring and the focus outline. Measured AA contrast: Ink on Paper 17.28 · Ink on Mint 15.25 · Ink on Cream 16.89 · Mist on Paper 6.96 · Deep Teal on Mint 5.31 · Plum on Blossom 10.36 · Paper on Deep Teal 6.02.
### 5.2 Typography
| Element | Family | Weight | Size / line-height |
|---|---|---|---|
| Hidden page `h1` | `--font-mint` | 500 | 22px / 1.22, Ink |
| Result eyebrow | `--font-mint` | 500 | 14px / 1.5, Mist, uppercase, tracking `.08em` |
| Result title `h2` | `--font-mint` | 500 | 22px / 1.22, Ink |
| Result body | `--font-mint` | 400 | 16px / 1.5, Mist, max 44ch |
| Detail label | `--font-mint` | 400 | 14px / 1.5, Mist |
| Detail value | `--font-mint` | 400 | 16px / 1.5, Ink |
| Section title `h2` | `--font-mint` | 500 | 20px / 1.25, Ink (`SectionCard` contract) |
| Form label | `--font-mint` | 500 | 14px / 1.5, Ink |
| Form input | `--font-mint` | 400 | 16px / 1.5, Ink |
| Empty-field error | `--font-mint` | 400 | 14px / 1.5, Plum |
| Button | `--font-mint` | 500 | 16px / 1.22, Paper |
| Trust row, footer disclosure | `--font-mint` | 500 / 400 | 14px / 1.4, Ink; 14px / 1.5, Mist |
The result card's visible hierarchy is 14 / 16 / 22. `--font-grenette` is not used on this route: no display headline, no marketing title, no section opener. The lower `SectionCard` keeps the primitive's 20px `h2`.
### 5.3 Geometry
- Base unit 4px. Root `.ui-public` has `min-height: 100vh`; Paper; no page-level max width so the footer can run full bleed.
- Head: full width, content max 1200px, 24px block and inline padding, flex space-between center, 16px gap; 1px bottom hairline. Mobile keeps 24px inline.
- Main: max 720px reading width, `margin: 48px auto 80px`, 24px inline padding; `ui-stack` gap 24px. This is the page's one column.
- Conditional wrapper: max 560px, centered, width 100%. Result card: Paper, 1px hairline, 24px radius, 32px padding, centered text; below 640px the padding drops to 24px.
- Seal: 72px circle, 1px border, grid center, margin `0 auto 16px`; valid adds an 80px conic ring with 4px padding and 9999px radius. Eyebrow above the seal; title margin 8px 0; body ends 24px above details.
- Details: real `dl`, text-align left, two columns with 16px / 24px gaps, 24px top margin; one column below 760px. Value sits 4px below its label.
- Re-entry `SectionCard`: 24px radius, 1px hairline, 24px padding; heading margin `0 0 16px`. Code form grid `1fr auto`, 12px gap, align-end; one column below 640px. Input and button radius 8px; input padding 8px 12px; button padding 12px 24px; trust row margin-top 24px.
- Footer band: full bleed, Paper, 1px top hairline, 24px block padding; disclosure line max 720px in its container. It is the page's last child inside `#verify-page`.
- Elevation flat: no box-shadow on head, cards, seal, form, or footer; only the focus halo.
### 5.4 Iconography and imagery
- Line icons 1.5px stroke, no fills except the wordmark cross. Seal glyphs are 24px: Check (Deep Teal on Mint), Close (Plum on Blossom), Arrow (Ink on Cream), Qr and Info (Mist on Paper). The logo cross is 14px in a 22px Mint circle.
- `TrustSecurityRow` uses an 18px Deep Teal shield and 18px Mist platform glyphs; icon-only platforms keep the primitive's visually hidden platform label.
- No photography, illustration, patient imagery, document mock, or scannable QR graphic. The Qr seal glyph is an icon; the only ring is the valid seal's decorative conic ring. The seal and all glyphs are `aria-hidden`.
## 6. Accessibility
- Landmarks: `header.ui-public-head`, `main.ui-public-main`, and the footer band; no `nav` element because the head has no site navigation. The wordmark is the header's only link (accessible name per §3.1); the pill is plain text.
- One `h1`, visually hidden (§3.2). The result title and the re-entry `SectionCard` title are `h2`; no `h3` and no duplicate `h1`. The order is stable across loading, result, and error states.
- Tab order: wordmark link → code input → submit action (§3.5). Loading and status text are not focusable; the result card contains no interactive target; focus never moves or traps when a result or error lands.
- The code field has a programmatic label (§3.5), `autocomplete="off"`, `spellcheck="false"`, and an inline error linked with `aria-describedby`. `ErrorText` is `role="alert"` and appears on empty submit only.
- `LoadingState` is `role="status"`, `aria-live="polite"`; its spinner stops under `prefers-reduced-motion`. Its label is the only busy announcement.
- The fetch `Notice` Error is `role="alert"`; the rate-limit message uses the same surface. Focus stays on the submit button; no countdown or `aria-live` timer is invented.
- Result details are a real `dl` with `dt` / `dd` in order (§3.4); the fallback is text. `VerificationSeal` is `aria-hidden`, so status is never icon- or color-only, and the title and body carry it in words.
- Focus: `1px solid var(--color-deep-teal)`, offset 2px, halo `0 0 0 3px rgba(185, 255, 232, 0.55)`. Every control is keyboard-operable; Enter submits the native form; there is no hover-only action and no keyboard trap. No UI data table renders, so no table caption or scope is needed.
- Contrast is AA or better for all text: Ink on Mint 15.25, Mist on Paper 6.96, Plum on Blossom 10.36, Paper on Deep Teal 6.02.
- The footer disclosure line is real, selectable text at the end of the `footer` landmark; it is not a link or a second live region.

## 7. Implementation notes
- Shipped: `Verify` at `persona-sanus/src/main.rs:1419` (`fn Verify(code: String)`), declared before `#[layout(Navbar)]` (`main.rs:59-60`), so it renders no navbar. It builds `result` / `message` / `loaded` / `requested` signals; `use_effect` reads `requested()`, clears `loaded`/`message`/`result` at fetch start, spawns `api::fetch_verification`, sets `result` or `message`, then `loaded` true (`main.rs:1420-1438`).
- Rendered root is `div#verify-page.ui-public` (`main.rs:1440`): `.ui-public-head` with `LogoLockup` (`persona-sanus/src/ui/brand.rs:13`) + `PillBadge` (`persona-sanus/src/ui/feedback.rs:64`), `main.ui-public-main.ui-stack` with visually hidden `h1` `Verificación de receta`, then exactly one of `LoadingState` `Verificando receta…` (`main.rs:1449`) / `VerificationResultCard` (`persona-sanus/src/ui/domain.rs:197`) / `Notice` Error with the fetch message; the re-entry `SectionCard` `Verificar otra receta` with `VerificationCodeForm` (Primary; trims, uppercases, `requested.set` + `navigator.replace(Route::Verify { code })` at `main.rs:1457-1465`) + `TrustSecurityRow` renders on every state; `Footer { surface: FooterSurface::Verify }` at `main.rs:1469` closes the page.
- Result card (shipped, `domain.rs:197-252`): title/body map `valid` → `Receta auténtica`, `voided` → `Receta anulada`, `superseded` → `Receta reemplazada`, `not found` → `Receta no encontrada`, other → `Receta no verificable`; details (`Médico` / `Fecha de receta` via `format_date` / `Número de receta` `{value:06}`, `—` fallback) render only for `valid`/`voided`/`superseded`.
- API/server truth (unchanged): `api::fetch_verification` GETs `/api/verify/{code}`; `service::verify_code` returns the four statuses with `None` fields for `not found`; `VerifyRateLimiter` (per-IP, default 10/60s) returns the §3.3 429 message, rendered verbatim by the error `Notice`. No desktop cache, outbox, or offline fallback exists or is added; offline is the error `Notice`.
- Verify, in order:
  - `cd persona-sanus && cargo check`
  - `dx build --release --fullstack`
  - `curl -fsS http://127.0.0.1:8080/verify/PS-TEST | grep -F 'Código de verificación'`; repeat once for the section heading (§3.5) and `ui-public-head`. `curl -fsS http://127.0.0.1:8080/verify/PS-TEST | grep -F 'ui-navbar'` must find nothing.
  - The route is rate-limited with the API: 10 requests per 60 s per IP by default. Do not loop the curl checks; an 11th page request returns the 429 JSON instead of HTML.
  - `caveman-blocks run persona-smoke --dir persona-sanus --port 8091` returns `"ok": true` but does not cover `/verify/:code`: its `ORDERED` stages are `fmt`, `check`, `clippy`, `build`, `health`, `home`, `login`, `dashboard`, `signup`, `session`, `signout`, `session_after_signout`, `signin` (`.blocks/persona-smoke.py:38-44`), and `persona-probe`'s default `FULL` flow has the same list. The probe's `verify` flow checks the `/api/verify/{code}` statuses, 429, and no-PHI payload, not this page's copy or geometry.
  - `python3 .blocks/markdown-table-check.py --path context/screens/done/Verify.md --strict`
- Do not add: a navbar or any shell chrome; a hand-rolled footer or a second disclosure line; a footer wordmark; patient, cédula, diagnosis, medication, or clinical-note fields or values; a PDF, QR download, print, share, or export affordance; a generated QR image; a code-less `/verify` route, a hash form, or a query-param form; a new endpoint, table, column, role, or permission; a second Primary action; a public marketing hero or trust-logo strip; a signature or stamp image; billing, price, trial, plan, or `sync_locked` copy; rich text, a WYSIWYG, or contenteditable; localStorage/sessionStorage/cookie caching or a desktop verification cache; an outbox or `pending_sync` branch; the shipped English labels `Doctor`, `Fecha`, `No.`; the shipped `Cargando...` string; a fifth status title or a raw API status; a 429 countdown, retry loop, or client-side rate-limit display.

## 8. Spec references

| Spec | Connection |
|---|---|
| `specs/done/15-O-ui-tokens-actions-and-formatting.md` | `15-O` supplies the tokens, `Button`, `ButtonLink`, and format helpers the verification page uses. |
| `specs/done/16-P-ui-forms-feedback-and-layout.md` | `16-P` supplies `PillBadge`, `LoadingState`, `Notice`, `SectionCard`, `ErrorText`, and `HintText` the typed-code form and result branch render. |
| `specs/done/17-Q-ui-brand-and-marketing-components.md` | `17-Q` builds `LogoLockup` and `TrustSecurityRow`, composed on this public page. `SectionHeader` and `VerificationTag` belong to `Home`/`A4PreviewCard`, not this page. |
| `specs/done/18-R-ui-domain-and-billing.md` | `18-R` builds `VerificationCodeForm`, `VerificationResultCard`, and `VerificationSeal`, the whole result surface. |
| `specs/done/20-T-shell-auth-home-and-verify.md` | `20-T` adopts `Verify`: doctor, date, and prescription number only, plus the always-available path to another code. |
| `specs/done/25-Y-ui-legacy-css-cleanup-and-docs.md` | `25-Y` removes the legacy selectors so the adopted verification markup runs only on `.ui-*` classes. |
