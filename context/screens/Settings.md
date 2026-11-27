# PersonaSanus — Settings (`/settings`)

| Field | Value |
|---|---|
| Source | `context/sitemap.md` §1.2 (Settings — index: all authenticated; hub; non-admin redirect), §2 (only the Administrator holds `/settings/users`, `/settings/clinic`, and `/audit`; every staff role holds `/settings/profile`; `/settings/billing` is v1.1), §4 (M0 ships `/settings/users`; billing is deferred); `context/PRD.md` §3 (roles, server-side authorization, D-03 role authority), D-04 (no billing tables, gates, jobs, or UI in v1), D-06 (no signature or stamp images), D-21 (forms and inputs only, no rich text); `context/Design.md` (Tokens, Surfaces, Elevation, Outlined Secondary Button, Do's and Don'ts); `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` step 13, the Page adoption `Settings` row, the `.ui-link-grid` and `.ui-link-card` CSS rows, steps 3, 5, 9, and 10; `context/screens/INDEX.md` §2.3 and §3 (`/settings` lag); `context/screens/done/Header.md` §7 (navigation coupling) |
| Component | `Settings` — `persona-sanus/src/main.rs:1761` (component `:1760-1802`). Target primitives, planned in `persona-sanus/src/ui/`: `PageShell`, `PageHeader`, `ButtonLink`, and `LoadingState`; the directory and its `layout.rs` / `button.rs` / `feedback.rs` are not shipped yet |
| Route | `/settings` — declared `persona-sanus/src/main.rs:66-67`; every authenticated role may load it; no query parameters. With no session it replaces the location with `/login` (`Route::AuthPage {}`); a non-Administrator profile is replaced with `/settings/profile` (`Route::SettingsProfile {}`); an Administrator stays and renders the hub. `/settings/billing` is not in the `Route` enum and receives no link, redirect, or placeholder |
| Layout | `PageShell id="settings-page"` inside `#[layout(Navbar)]` (`main.rs:51`); root `div#settings-page.ui-page > div.ui-container.ui-stack`; one Paper band; no full-bleed exception; adds no second header or footer band |
| Locale | `es-EC` on every string. `Configuración`, `Clínica`, `Auditoría`, and `diseño` keep their accents; no dates, times, money, or quantities render |
| Status | ✅ Spec complete (sections 1–7). Route status: ✅ — sitemap: "Hub linking clinic, users, profile, and (Administrator, v1.1) billing. Redirects to `/settings/profile` for non-admin roles." Shipped: the hub renders for the Administrator with the `Usuarios y roles`, `Clínica y biblioteca`, `Mi perfil`, and `Auditoría` cards plus the debug-only `Sistema de diseño` card, sends signed-out sessions to `/login`, and non-admin profiles to `/settings/profile`; billing stays out of v1 |

## 1. Purpose

The index for staff administration. Ordered jobs:

1. Let an Administrator choose where to work: staff and roles, clinic and library, or their own profile.
2. Give the Administrator a path to the audit trail that survives the navbar cleanup in `context/screens/done/Header.md` §7.
3. Move a signed-in non-Administrator out of this index and into the only settings surface their role owns, `/settings/profile`, before the hub paints.
4. Move a signed-out visitor to `/login` before the hub can paint.
5. In a debug build only, expose the component gallery at `/dev/ui` without making it a release-visible or product route.

It never renders:

- PHI: no patient name, cédula, diagnosis, medication, prescription number, image, or clinical content is present in any label, URL, title, or metadata. This is not a patient screen and has no authorized clinical-content role.
- Forms, editable fields, list rows, counters, or summaries. Staff entries belong to `/settings/users`; clinic data, retention, and the image library to `/settings/clinic`; profile, MFA, and password to `/settings/profile`; audit rows to `/audit`.
- Billing, price, trial, plan, payment-card, or `sync_locked` copy, whether as a card, badge, link, disabled control, or empty-state hint. Billing is v1.1 (D-04) and `/settings/billing` has no v1 surface.
- Signature or stamp imagery, or any implication that the profile stores one (D-06): the doctor signs the printed paper.
- Rich text, `contenteditable`, a formatting toolbar, or any text-entry affordance (D-21). The hub has no inputs at all.
- A filled teal action. There is no Primary control in this viewport.
- A reduced non-administrator hub, a denial panel, or a suggestion that other settings might be accessible. Non-administrators are replaced, not lingered.

## 2. Structure

One Paper page, a stack of 24px gaps descending from the Navbar. The loading/redirect branch and the Administrator hub are the only states. The hub carries links only: no forms, no list rows, no counters, no query control, no confirmation panel, and no table.

| Order | Region | id / class | Band tone | Contents |
|---|---|---|---|---|
| — | Navbar (layout) | `#navbar` · `header.ui-navbar` | Paper `#ffffff`, bottom hairline | Owned by `context/screens/done/Header.md` §2–§5; not restated here |
| 1 | Page shell | `#settings-page` · `div.ui-page` > `.ui-container.ui-stack` | Paper | 48px top padding, 80px bottom, 24px inline; stack gap 24px; present in every state |
| 2 | Loading or redirect line | `.ui-loading` | Paper | One `LoadingState` line: `Cargando...` while the session check is unfinished or the profile is absent, `Redirigiendo...` while a non-administrator is being replaced. No header and no cards in either state |
| 3 | Page header | `.ui-page-head` · `h1.ui-page-title` · `p.ui-page-subtitle` | Paper | `PageHeader` title and subtitle; no header actions, no subtitle children, no trailing metadata |
| 4 | Link grid | `nav[aria-label]` > `div.ui-link-grid` | Paper | Secondary `ButtonLink` cards, one visible label each; three today, four for an Administrator after plan step 13, five in a debug build after step 9 |

Rows 2 and 3–4 are mutually exclusive. The page renders row 2 while `checked` is false, while `actor` is `None`, and while a non-administrator is being replaced; it renders rows 3–4 only for an Administrator. Nothing is horizontally scrollable, and no table or empty-state surface exists.

### 2.1 Session finish and role

`checked` is the shell's session-check signal (`Signal<bool>`, provided at `main.rs:1234`) and `actor` is the signed-in profile (`Signal<Option<Me>>`, `main.rs:1233`). `Me::is_administrator()` reads the profile role and compares it with `"administrator"` (`persona-sanus/src/api.rs:49-51`); `clinic_members.role` is the role authority (D-03).

| Session finish / role | Hub cards | What renders | What changes |
|---|---|---|---|
| Check in flight (`checked` false) | none | `PageShell` root with one loading line | The session is not yet known: no header, no cards, no navigation |
| Checked, `actor` is `None` | none | `PageShell` root and the loading line; the effect replaces the location with `/login` | No local sign-in form; `AuthPage` owns authentication |
| Checked, profile with no role | none | Replaced with `/settings/profile` | `/settings/profile` is the completion surface for a missing role |
| `doctor` | none | `PageShell` root and `Redirigiendo...`; replaced with `/settings/profile` | No reduced hub: only the profile surface paints |
| `nursing` | none | `PageShell` root and `Redirigiendo...`; replaced with `/settings/profile` | Same replacement; the hub never exposes clinic or staff links to this role |
| `secretary` | none | `PageShell` root and `Redirigiendo...`; replaced with `/settings/profile` | Same replacement |
| `administrator` | `Usuarios y roles`, `Clínica y biblioteca`, `Mi perfil`, plus `Auditoría` after step 13 | Page header and the full link grid | The only role that settles here. Every destination stays a link; no card expands into content |

The replacement uses `navigator.replace`, not `push`, so Back does not bounce through `/settings` and re-run the effect.

### 2.2 Debug build versus release build

| Build | Cards | What changes |
|---|---|---|
| Release | The same four cards after plan step 13 | No `Sistema de diseño` card and no `/dev/ui` link anywhere in the viewport |
| Debug (`cfg(debug_assertions)`) | The same cards, then `Sistema de diseño` | The gallery card is appended last and targets `/dev/ui`; it is still Secondary and still a link |

The debug card is a development affordance, not a product route: `/dev/ui` is a developer surface with no sitemap §1.2 row, no role, makes no API call, and has no screen file (`context/screens/INDEX.md` §5). In a release build that reaches `/dev/ui` directly, plan step 9 makes `DevUi` return one `.ui-card` with `Esta vista solo existe en compilaciones de desarrollo.`; the Settings hub shows no path to it. Until plan step 9 adds `DevUi` and `Route::DevUi`, the target debug card has no destination and must not ship as a dead link.

### 2.3 Web and desktop surface

| Surface | What renders | What changes |
|---|---|---|
| Web (Chromium) | The same shell, header, and card grid for an Administrator | Session availability decides entry; the grid is identical |
| Desktop (macOS, Windows) | The same shell, header, and card grid | No desktop-only card, count, sync widget, or cache notice is added. If the desktop restores a persisted Administrator profile while the server is unreachable, the hub still paints; every destination decides its own offline behaviour |

`Settings` issues no request and reads no cached record, so no loading, pending, failed, or 30-day-retention state is local to this screen. The sync indicator stays in the Navbar (`context/screens/done/Header.md` §4) and is not repeated here.

### 2.4 Empty, denied, error, and busy states

| State | What renders | What does not render |
|---|---|---|
| Empty | Nothing: the Administrator branch always contains at least the three production cards, so an empty grid is unreachable | `EmptyState`, a placeholder card, an explanatory paragraph |
| Denied | The standard non-administrator replacement; the hub never paints for that profile | An access-denied notice, a retry control, a reduced card set, a local sign-in form |
| Error (session check) | An absent or failed profile resolves to `None`, so the no-session replacement runs | `ErrorText`, `Notice`, a stack trace, a session-error detail |
| Busy (navigation) | The activated card performs in-app navigation; the destination owns its own loading UI | A busy label, a disabled card, two destination markers |

## 3. Copy text

Every string is hard-coded `es-EC`; none is fetched. The `h1` and the link-grid landmark share one word, so it appears once here and is referenced by both roles.

| Element | Copy | Target |
|---|---|---|
| Page title (`h1`) and link-grid landmark (`nav` accessible name) | `Configuración` | `/settings` |
| Page subtitle | `Administra el acceso del personal y tu perfil.` | — |
| Card 1 label and accessible name | `Usuarios y roles` | `/settings/users` |
| Card 2 label and accessible name | `Clínica y biblioteca` | `/settings/clinic` |
| Card 3 label and accessible name | `Mi perfil` | `/settings/profile` |
| Card 4 label and accessible name, Administrator only, after plan step 13 | `Auditoría` | `/audit` |
| Card 5 label and accessible name, debug build only, after plan step 9 | `Sistema de diseño` | `/dev/ui` |
| Loading line | `Cargando...` | — |
| Redirect line, non-Administrator | `Redirigiendo...` | `/settings/profile` |

No other user-visible string belongs to this screen. There is no card body copy, meta line, badge, count, hint, empty-state description, error text, placeholder, table caption, column header, confirm or cancel label, busy label, or success message.

## 4. Displayed information

### 4.1 What the screen shows

| Element | Source | Value shown |
|---|---|---|
| Page title | static RSX literal in `PageHeader`; shipped `h4` at `main.rs:1790` | the page title from §3 |
| Page subtitle | static RSX literal; shipped `p` at `main.rs:1791` | the page subtitle from §3 |
| Link-grid landmark | static `aria-label` on `nav` | the same word as the page title, §3 |
| Card set | Dioxus signal `actor` (`Signal<Option<Me>>`) plus `Me.role` through `Me::is_administrator()` (`api.rs:49-51`); plan step 13 builds the grid only in the Administrator arm | no cards, or the cards listed below |
| Card 1 | `Route::SettingsUsers {}` | card 1 label, §3 |
| Card 2 | `Route::SettingsClinic {}` | card 2 label, §3 |
| Card 3 | `Route::SettingsProfile {}` | card 3 label, §3 |
| Card 4 | `Route::Audit {}`, added by plan step 13 inside the same Administrator arm; the variant exists at `main.rs:74-75` | card 4 label, §3 |
| Card 5 | `Route::DevUi {}`, added by plan step 9 and linked only inside `#[cfg(debug_assertions)]`; the variant does not exist yet | card 5 label, §3 |
| Loading line | `checked` false, or `actor` is `None` (`main.rs:1780-1785`) | the loading line, §3 |
| Redirect line | `checked` true and `actor` is a non-Administrator profile (`main.rs:1797-1800`) | the redirect line, §3 |
| Redirect destination | `navigator.replace` in the effect (`main.rs:1766-1778`) | `/login` for no session; `/settings/profile` for a non-administrator |

`Settings` calls no `api::` function and reads no clinic, doctor, profile status, patient, prescription, template, asset, user, device, or audit row. Its only inputs are `checked`, `actor`, and the router. The labels, subtitle, and landmark name are literals; the destinations are `Route` variants, never fetched paths.

### 4.2 Hard rules

- Server-side authorization is the boundary; card visibility is not. Each destination re-checks the role, and the API's authorized service layer enforces it again (PRD §3). A hand-typed `/settings/users`, `/settings/clinic`, or `/audit` never inherits the Administrator hub's view.
- The card set follows sitemap §2: `/settings/users` and `/settings/clinic` are Administrator-only, `/audit` is Administrator-only, and `/settings/profile` is the one settings destination every staff role holds. The hub offers no card from which a non-administrator could infer the Administrator-only surfaces.
- The hub is a link index, not a boundary and not a data surface. It renders no forms, rows, counts, PHI, clinical content, clinic fields, user fields, device data, audit metadata, or billing content.
- No `/settings/billing` card, link, disabled placeholder, price, trial, plan, `sync_locked`, or “coming soon” copy in v1 (D-04). The path is a v1.1 reservation only; it is absent from the `Route` enum and from this screen.
- `Auditoría` is Administrator-only and is the replacement path for the navbar entry removed by plan step 10. Ship the card and the removal in one change set (`context/screens/done/Header.md` §7); never drop the navbar entry first and strand `/audit`.
- `Sistema de diseño` is debug-only. It is compiled out of release builds, has no role, is not a product route, and must never be the only route to a shipped page.
- Non-administrators never settle here and never see a reduced hub. The only role-shaped behaviour is replacement to `/settings/profile`, which every staff role holds. Patients, prescriptions, and templates are not settings cards; the Navbar owns their destinations.
- One primary action per viewport, maximum: this viewport has none. Every card is a Secondary `ButtonLink`; no filled Deep Teal control exists on the page.
- The hub writes nothing: no audit event, no preference, no role change, and no last-visited marker on activation.
- `es-EC` is the locale: `Configuración`, `Clínica`, `Auditoría`, `diseño`, and `Administra` keep their accents.

### 4.3 Dynamic states

| State | Screen behaviour |
|---|---|
| Initial mount, `checked` false | `PageShell` root and one loading line. The effect waits; no navigation attempt is made and no card appears |
| Session check settles with no session (`actor` is `None`) | The effect calls `navigator.replace(Route::AuthPage {})`; the loading line remains until the replacement paint. `/login` owns the next surface |
| Session check settles with a session but no profile (or the profile request failed) | `actor` is `None`, so the same `/login` replacement runs; no access-denied panel and no role guess |
| Profile has no role | `is_administrator()` is false; replacement to `/settings/profile` runs, matching the profile-completion entry used by `Home.md` |
| Doctor | Replacement to `/settings/profile`; `Redirigiendo...` only. No staff or clinic card paints |
| Nursing | Replacement to `/settings/profile`; `Redirigiendo...` only |
| Secretary | Replacement to `/settings/profile`; `Redirigiendo...` only |
| Administrator | `PageHeader` and the card grid. Three production cards today; four after plan step 13. The hub sets no auto-focus |
| Administrator, desktop offline with a persisted profile | The hub paints the same grid from the context restored by `App` (`main.rs:1239-1268`); this screen adds no offline banner and makes no request. Destinations decide their own offline behaviour |
| Web offline before a session is established | No profile can be restored, so `actor` is `None` and the no-session replacement sends the visitor to `/login`; the hub never paints a signed-out grid |
| Debug build (`cfg(debug_assertions)`) | `Sistema de diseño` is appended last and targets `/dev/ui`; it is compiled out of release markup rather than hidden with CSS |
| Release build | Four Administrator cards; no design-system card and no `/dev/ui` link |
| Direct `/dev/ui` in release | Outside this screen: plan step 9's single `.ui-card` message. The hub never links to it |
| Role changes while the hub is mounted | The effect re-runs on the `actor` and `checked` signals. Administrator to non-administrator triggers `/settings/profile`; loss of session triggers `/login`; the grid unmounts |
| Card activated by click or Enter | The Dioxus router `Link` performs in-app navigation to the card target; no submit, no API call, no local busy state, no new tab |
| Back from a destination | The hub re-renders from context and the effect re-runs. An Administrator sees the grid; anyone else is replaced again |
| Direct `/settings/users`, `/settings/clinic`, or `/audit` as a non-administrator | The destination's own guard applies; this hub is not consulted and grants nothing |
| Empty grid | Unreachable: the Administrator branch always includes at least the three production cards, before and after step 13 |
| Fetch or mutation error on this screen | None: the hub fetches and mutates nothing. Destination errors belong to those screens |
| Pending sync or failed outbox | Not rendered here; owned by the Navbar (`context/screens/done/Header.md` §4) and `/prescriptions` |
| Reduced motion | No transition or animation is introduced by the hub. The shared spinner is suppressed under `prefers-reduced-motion`, and the loading or redirect label remains readable |

## 5. Style

Every value comes from `context/Design.md` tokens or the shared UI plan contract. No new color, font, icon package, image, gradient, or one-off class is introduced.

### 5.1 Color

| Use | Value | Token |
|---|---|---|
| Page canvas, card surfaces | `#ffffff` | `--color-paper` |
| Title, card label text | `#061d29` | `--color-ink` |
| Subtitle, loading and redirect text | `#425d6d` | `--color-mist` |
| Card border, hairline separator | `#e5e7eb` | `--color-hairline` |
| Card hover text, focus outline | `#006e75` | `--color-deep-teal` |
| Card hover surface | `#fffded` | `--color-cream` |
| Focus halo | `rgba(185, 255, 232, 0.55)` | `--color-mint-halo`, planned in UI plan step 2; Mint Glass `#b9ffe8` at 55% |
| No element on this screen | `#0b978e` | `--color-bright-teal`, reserved for a Primary hover; this viewport has no Primary |

Never `#000000`, never a chromatic card border. The card border is always the neutral `--color-hairline`; the only teal outline is the keyboard focus ring. Measured contrast: Ink on Paper 17.28:1, Mist on Paper 6.96:1, Deep Teal on Paper 6.02:1, Ink on Cream 16.89:1, Mist on Cream 6.80:1, Deep Teal on Cream 5.89:1 — all at or above WCAG AA for their text size.

### 5.2 Typography

| Element | Family | Weight | Size / line-height |
|---|---|---|---|
| Page title (`h1`) | `--font-mint` | 500 | 22px / 1.22 (`--text-heading-sm`), Ink |
| Page subtitle | `--font-mint` | 400 | 14px / 1.5 (`--text-body-sm`), Mist |
| Card label | `--font-mint` | 500 | 16px / 1.22 (`--text-body`), Ink; Deep Teal on hover |
| Loading and redirect line | `--font-mint` | 400 | 14px / 1.5 (`--text-body-sm`), Mist |

`--font-grenette` does not appear: the largest text is 22px, and the display face is reserved for 46px and above. No bold weight; 500 is the heaviest label. Labels never truncate, ellipsize, or wrap to a second line at the 220px minimum track.

### 5.3 Geometry

- Base unit 4px. Every padding, gap, and radius below snaps to the token scale.
- `PageShell`: 48px top, 80px bottom, 24px inline padding; `.ui-container` max-width 1200px, centered; `.ui-stack` gap 24px. The page header and the grid are the only stack children in the Administrator hub.
- `PageHeader`: title and subtitle stack; the subtitle starts 4px below the title, with no top margin. There are no header actions, so no empty action column is reserved.
- Link grid: CSS grid `auto-fit` with tracks `minmax(220px, 1fr)` and a 16px gap. Tracks wrap rather than overflow: the grid fits as many 220px tracks as the inline width admits and puts the rest on the next row. No horizontal scroll and no page-level breakpoint.
- Card: Paper surface, 1px Hairline border, 24px radius, 16px 20px padding, flex column, 4px gap, no shadow. The whole card is the anchor; the label is its only child.
- Secondary treatment: transparent background and Ink text on the card surface, with Deep Teal text and Cream background on hover. The Secondary variant adds no second border; the card's 1px Hairline is the only edge. The card padding keeps each target above 44px even with a single line of text.
- Radii: 24px for the card-shaped link surface; 8px is the shared control radius and is not visible here because the card surface sets the outer shape.
- Focus ring: `outline: 1px solid var(--color-deep-teal)`, `outline-offset: 2px`, plus `box-shadow: 0 0 0 3px var(--color-mint-halo)`. The halo is a ring, not elevation.
- Elevation: flat. No `box-shadow` except the focus ring; no `--shadow-paper` (that belongs to the A4 sheet). Separation is the 1px hairline plus spacing.
- Alignment: the header is start-aligned within the container, matching the other authenticated screens; the grid fills the container width. Cards are equal-height within a row, and their labels sit at the inline start with no icon column.

### 5.4 Iconography and imagery

None. The hub is a text index: no line icons, no logo, no QR glyph, no document preview, no avatar, no photography, and no illustration. Each card is identified by its visible label alone, so there is no icon-only control and no decorative mark to hide from assistive technology. The Navbar above owns the only brand mark on the page (`context/screens/done/Header.md` §3.1).

## 6. Accessibility

- Landmarks: the layout band supplies the header and navigation chrome owned by `Header.md`; this screen contributes one `nav` landmark, labelled `Configuración`. Nothing else is a landmark. No footer renders on this authenticated route (`Footer.md`).
- Headings: exactly one `h1`, the `PageHeader` title. There is no `h2` and no `h3`: the cards are sections by function but carry no heading, and turning a card label into one would misreport a link as structure. The heading order therefore has no jump.
- Links, not list items or buttons: each `ButtonLink` wraps a Dioxus router `Link`, so assistive technology announces a link with the visible label. The grid container is a `div`; it does not claim list semantics it does not have.
- Tab order follows DOM order: Navbar controls first (`Header.md` §6), then `Usuarios y roles`, `Clínica y biblioteca`, `Mi perfil`, `Auditoría`, and, in a debug build only, `Sistema de diseño`. No form control, disclosure, or extra tab stop exists inside a card.
- Focus is always visible per the shared utility: 1px Deep Teal outline, 2px offset, 3px Mint Glass halo. Hover may change text and surface, but focus must draw the ring; the card never removes it.
- Accessible names are the visible labels. There is no `aria-label` override that could diverge from the rendered text; the only hidden name is the `nav` label, which mirrors the `h1`.
- Touch and pointer: the anchor's own box is the target, above 44px in both dimensions through the card padding; adjacent cards are separated by the 16px grid gap, so one tap cannot land on two targets.
- Status is never conveyed by colour alone: the only state difference is the hover treatment, which is not required to operate the control. Each card is actionable and announced as a named link without hover.
- Contrast: Ink on Paper 17.28:1, Mist on Paper 6.96:1, Deep Teal on Paper 6.02:1, Ink on Cream 16.89:1, Mist on Cream 6.80:1, Deep Teal on Cream 5.89:1. The 1px Hairline border is decorative; the label carries the name, so the border is not a contrast dependency.
- No `role="alert"`, `aria-describedby`, `aria-busy`, table `caption`, or `th scope` appears: there are no fields, errors, tables, or submissions. The loading and redirect line uses the shared `LoadingState` contract (`role="status"`, `aria-live="polite"`).
- Reduced motion: the hub adds no animation. The shared spinner is the only motion and is disabled under `prefers-reduced-motion`; the loading or redirect label remains readable.

## 7. Implementation notes

- Shipped component: `fn Settings()` at `persona-sanus/src/main.rs:2271` (`#[component]` at `:2270`, closing before `SettingsClinic` at `:2350`). Route `/settings` (`Route::Settings`) with `DevUi` at `main.rs:65` and `fn DevUi()` at `:1475` (`ui::DevUiGallery()` at `:1477`). Reads `actor`/`checked` contexts and `use_navigator()`; calls no `api::` function.
- Redirect effect (`main.rs:2275-2287`): when `checked()`, `None` → `navigator.replace(Route::AuthPage {})`, non-Administrator → `navigator.replace(Route::SettingsProfile {})`, Administrator → nothing. Unchanged.
- Render (`main.rs:2289-2349`): `!checked() || profile.is_none()` → `PageShell id="settings-page"` + `LoadingState` (`Cargando...` is the `LoadingState` default label, `ui/feedback.rs`); Administrator arm → `PageShell` + `PageHeader` (`Configuración`, `Administra el acceso del personal y tu perfil.`) + `nav[aria-label="Configuración"]` > `div.ui-link-grid` with four Secondary `ButtonLink`s (`ui-link-card`): `Usuarios y roles` → `SettingsUsers`, `Clínica y biblioteca` → `SettingsClinic`, `Mi perfil` → `SettingsProfile`, `Auditoría` → `Audit` (inside `if profile.is_administrator()`, `:2329`, redundant with the arm but shipped); fallback → `LoadingState` `Redirigiendo...`. The debug card is `{gallery_settings_link()}` (`:2337`): `#[cfg(debug_assertions)]` arm at `:2253-2264` renders `Sistema de diseño` → `Route::DevUi {}`, `#[cfg(not(debug_assertions))]` arm at `:2266-2269` renders nothing — no `DevUi` link in release.
- Navbar (`main.rs:1509-1531`) has no `Auditoría` entry — only `Configuración` at `:1531` — so the hub `Auditoría` card is the replacement path; the coupled removal shipped with the card.
- Primitives are the shipped `src/ui/` set (`PageShell`/`PageHeader` in `ui/layout.rs`, `ButtonLink` in `ui/button.rs`, `LoadingState` in `ui/feedback.rs`); this component composes only those and no raw `div`/`h4`/`p`/`Link` remains.
- No new endpoint, API call, role, permission, query parameter, table, or column; no desktop branch. Both replacements above stay exactly as shipped.
- Verify, in order:
  - `cd persona-sanus && cargo check` — the adopted primitives and the card set compile in the debug configuration.
  - `cd persona-sanus && dx build --release --fullstack` — the release build contains no `Sistema de diseño` card and still builds every destination route.
  - `caveman-blocks run persona-smoke --dir persona-sanus --port 8091` — the shell smoke run covers `/`, `/login`, and an authenticated `/dashboard`; it has no `/settings` stage, so it proves the shell and destinations, not the hub assertions above.
  - `python3 .blocks/markdown-table-check.py --path context/screens/done/Settings.md --strict` — every table has a consistent column count.
- No route-specific `curl`: `/settings` is authenticated and the smoke script does not exercise it.
- Do not add: a `/settings/billing` card, link, placeholder, price, trial, plan, or `sync_locked` copy (D-04); cards or links for patients, prescriptions, or templates (the Navbar owns them); forms, editable fields, count badges, PHI, clinical content, or clinic, user, profile, or audit data to the hub; a reduced non-administrator hub, denial panel, or local sign-in form; signature or stamp imagery (D-06); rich text, a formatting toolbar, or `contenteditable` (D-21); an icon-only card, decorative image, QR glyph, or A4 mock; a Primary button or any filled Deep Teal action; a release-visible `Sistema de diseño` link; a new route, endpoint, role, permission, table, column, or query parameter; a second header or footer band; client-side role checks presented as authorization; or a `push` in place of either replacement redirect; an `Auditoría` entry back in the navbar; a non-`cfg(debug_assertions)`-gated `/dev/ui` link.

## 8. Spec references

| Spec | Connection |
|---|---|
| `specs/done/15-O-ui-tokens-actions-and-formatting.md` | `15-O` supplies the tokens, `ButtonLink`, and format helpers the settings hub renders. |
| `specs/done/16-P-ui-forms-feedback-and-layout.md` | `16-P` supplies `PageShell`, `PageHeader`, and the `ui-link-grid` layout whose destination tiles the hub composes. |
| `specs/done/19-S-ui-gallery-and-dev-route.md` | `19-S` builds the debug-only `/dev/ui` gallery the hub `Sistema de diseño` link targets (`#[cfg(debug_assertions)]` only). |
| `specs/done/21-U-ui-dashboard-settings-and-patients.md` | `21-U` adopts `Settings`: `PageShell id="settings-page"`, `ui-link-grid` destination tiles, preserved role redirects, no Primary. |
| `specs/done/25-Y-ui-legacy-css-cleanup-and-docs.md` | `25-Y` removes legacy page-ID selectors so the adopted hub runs only on `.ui-*` classes. |
