# PersonaSanus — Header (`Navbar`, app shell)

| Field | Value |
|---|---|
| Source | `context/sitemap.md` §1.2 (authenticated shell: persistent nav, session/role context, sync indicators), §1.3 (Sync / outbox status non-route), §2 (navigation by role), §5 (conventions); `context/Design.md` (Logo Lockup, Green Pill Badge, Ghost Nav Button, Primary Action Button, Layout "Navigation is a minimal top bar" + build step 3, Elevation, Do/Don't "one `#006e75` filled button per viewport"); `context/PRD.md` §3 (roles), FR-043, D-04, D-06, D-21; `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` steps 6, 7, 10, 17 and its CSS rows `.ui-pill`, `.ui-navbar*`, `.ui-nav-link*`, `.ui-logo*`, `.ui-sync*`; `context/screens/INDEX.md` §1.1 (frozen name), §5 (sync indicator folds here) |
| Component | `Navbar` — `persona-sanus/src/main.rs:1377`, opened by `#[layout(Navbar)]` at `main.rs:51`. Primitives, all planned in `persona-sanus/src/ui/` (the directory does not exist yet): `LogoLockup` + `PillBadge` (`brand.rs`, plan step 6), `NavLink`, `Button` `Ghost`, `ButtonLink` `Ghost` (`button.rs`, plan step 4), `SyncStatusIndicator` (`domain.rs`, plan step 7) |
| Rendered on | `/`, `/login`, and every authenticated shell route. `#[layout(Navbar)]` (`main.rs:51`) opens on `Home {}` and is never closed with `#[end_layout]`, so it wraps every variant declared after it through `PrescriptionDetail` (`main.rs:52-92`) |
| Not rendered on | `/verify/:code` — `Verify` is declared at `main.rs:49-50`, **before** the layout marker, so it is a standalone route and renders no band (dioxus-router 0.7 binds `#[layout]` to the variants declared after it). Also not the debug-only `/dev/ui` gallery, which plan step 9 places outside the layout block and which is not a `Route` variant yet |
| Layout | Full-bleed Paper band, `header#navbar.ui-navbar`, bottom hairline 1px `#e5e7eb`, `min-height: 64px`; inner `.ui-navbar-inner` max-width 1200px. Owned by the layout, never inside `PageShell`, never repeated by a page |
| Locale | `es-EC` on every string. The fifth label is `Configuración`, frozen here (§3); `context/Design.md` Layout and build step 3 spell it `Ajustes` — the label authority for chrome strings is this file (INDEX §1.3), `Design.md` is not edited for it |
| Status | 🟡 Partial. Shipped: the whole session/role/sync logic and both auth branches, on nine links — `Home`, `Dashboard`, `Pacientes`, `Recetas`, `Plantillas`, `Mi perfil`, `Usuarios`, `Auditoría`, `Configuración` — with `Sign in` / `Sign out`, the merged sync label `Sin conexión` / `Pendientes` / `En línea`, and no brand group, no pill, no `LogoLockup`. Styled by `#navbar` (`persona-sanus/assets/main.css:156-193`, fixed `height: 64px`, padding `0 32px`). Target markup per plan step 10; structure, copy, and states are frozen by this file |

The application header band, web and desktop chrome. Not the `PageHeader` component of the plan (`h1` + page actions inside each page, `layout.rs`) and not the `.ui-public-head` brand row that plan step 12 gives the `Verify` page. One band, one definition, above every screen it is rendered on.

## 1. Purpose

The persistent orientation strip. Four jobs, in this order:

1. Brand and return path — `LogoLockup` links to `/`. No separate `Home` text link (plan step 10).
2. Move between the five work areas — exactly five destinations, role-scoped, the sitemap §2 set: `/dashboard`, `/patients`, `/prescriptions`, `/templates`, `/settings`. All four staff roles live in this shell.
3. Carry session identity and the auth action — `Iniciar sesión` when signed out; the account email and `Cerrar sesión` when signed in.
4. Surface sync state — `SyncStatusIndicator` plus the pending and failed count links of FR-043, signed-in only (sitemap §1.3 folds the whole outbox indicator here).

The band is session-aware but never blocking: signed-out visitors see the brand and one action; the first SSR paint is the signed-out group, which flips after hydration (`context/architecture.md` §5 rules). The band never renders PHI, never billing, price, trial, plan, or `sync_locked` copy (D-04), never legal text (the law line lives in the hero, `Home.md` §3.2, and the public `Footer`, `Footer.md` §3), never a signature or stamp image (D-06), never a filled teal action.

## 2. Structure

One band, three zones, symmetric posture (`Design.md` Layout: "logo + 5 menu items … centered, with … actions flush right").

| Order | Region | `id` / class | Band tone | Contents |
|---|---|---|---|---|
| — | Root | `<header id="navbar" class="ui-navbar">` | Paper `#ffffff`, bottom hairline 1px `#e5e7eb` | `div.ui-navbar-inner` wrapper, grid `1fr auto 1fr` |
| 1 | Brand group | `.ui-navbar-brand` | Paper | `LogoLockup` (`PersonaSanus` → `/`, mint circle + cross glyph) + `PillBadge` `Receta verificable` |
| 2 | Links | `nav.ui-navbar-links` | Paper | Zero to five `NavLink`s, session- and role-guarded (§2.1) |
| 3 | Account group | `.ui-navbar-account` | Paper | Signed-in: `SyncStatusIndicator`, `{n} pendientes` link, `{n} con error` link, `.ui-navbar-user` email, `Cerrar sesión` (Ghost `Button`). Signed-out: `Iniciar sesión` (Ghost `ButtonLink`) |

Nothing else sits in the band: no search field, no avatar, no bell, no hamburger, no language switcher, no legal line.

### 2.1 What renders per session and role

| Session / role | `Panel` | `Pacientes` | `Recetas` | `Plantillas` | `Configuración` | Right group |
|---|:---:|:---:|:---:|:---:|:---:|---|
| No session | — | — | — | — | — | `Iniciar sesión` |
| Session, no profile or `role: null` | ✓ | — | — | — | ✓ | indicator + counts + email + `Cerrar sesión` |
| Administrator | ✓ | ✓ | ✓ | ✓ | ✓ | same |
| Doctor | ✓ | ✓ | ✓ | ✓ | ✓ | same |
| Nursing | ✓ | ✓ | ✓ | — | ✓ | same |
| Secretary | ✓ | ✓ | ✓ | — | ✓ | same |

The guards are the shipped `Me` methods, unchanged (`persona-sanus/src/api.rs`): `Panel` (`Route::Dashboard {}`) and `Configuración` (`Route::Settings {}`) render for any session; `Pacientes` requires `can_read_patients()` (`api.rs:57`) and `Recetas` requires `can_read_prescriptions()` (`api.rs:80`) — both true for any non-null role; `Plantillas` requires `can_manage_templates()` (`api.rs:68` — `administrator | doctor`), matching sitemap §2 where Nursing and Secretary have no `/templates`. The brand group renders in every state, signed-out included.

Four entries leave the bar and one label changes (plan step 10): the text `Home` link (the logo goes Home), `Mi perfil`, `Usuarios`, `Auditoría`; `Dashboard` becomes `Panel`. They are the only current paths to `/settings/profile`, `/settings/users`, and `/audit`, so their removal lands in the same change set as plan step 13 / the plan's `Settings` page row, which gives the Settings hub the `Mi perfil`, `Usuarios y roles`, and admin `Auditoría` cards. Today the hub carries those three links (`main.rs:1792-1794`) but no `Auditoría` one: dropping `Auditoría` from the bar before step 13 makes `/audit` unreachable.

### 2.2 Layout graph (recorded, not changed here)

`#[layout(Navbar)]` opens at `main.rs:51` and stays open to the end of the enum, which is why the shell band also wraps the public `/` and `/login` (`context/architecture.md` §5 rule, which states the same split). Consequences this file accepts:

- `/verify/:code` renders no band, so plan step 12's `.ui-public-head` (`LogoLockup` + `PillBadge`) is its only brand row — no duplicate brand to resolve.
- `/` and `/login` are wrapped, so `Home.md` §2 keeps its navbar row and `Footer.md` stays inside this layout. When `#[end_layout]` and a public layout group land, those two routes move to the public group and this file covers the authenticated shell band only.

No layout-graph edit is part of this spec.

## 3. Copy text

All strings verbatim, `es-EC`. Punctuation is part of the copy.

### 3.1 Brand group

| Element | Copy | Target |
|---|---|---|
| Wordmark (`LogoLockup`) | `PersonaSanus` — `aria-label="PersonaSanus — inicio"` | `/` (`Route::Home {}`) |
| Pill badge (`PillBadge`) | `Receta verificable` | none — static span |

### 3.2 Links

| Element | Copy | Target |
|---|---|---|
| Nav link 1 | `Panel` | `/dashboard` (`Route::Dashboard {}`) |
| Nav link 2 | `Pacientes` | `/patients` (`Route::Patients {}`) |
| Nav link 3 | `Recetas` | `/prescriptions` (`Route::Prescriptions { status: None }`) |
| Nav link 4 | `Plantillas` | `/templates` (`Route::Templates {}`) |
| Nav link 5 | `Configuración` | `/settings` (`Route::Settings {}`) |
| Active link | same label, Deep Teal weight 500 (`ui-nav-link--active`) | — |

### 3.3 Account group and sync

| Element | Copy | Target |
|---|---|---|
| Signed-out action | `Iniciar sesión` (Ghost) | `/login` (`Route::AuthPage {}`) |
| Signed-in identity | the session email, verbatim, no label | none — static text |
| Signed-in action | `Cerrar sesión` (Ghost) | action, not a link |
| Sync label (exactly one, precedence §4.3) | `Sin conexión` · `Error de sincronización` · `Sincronizando` · `En línea` | none |
| Pending link | `· {n} pendientes`, singular `· 1 pendiente` | `/prescriptions?status=pending_sync` (`Route::Prescriptions { status: Some("pending_sync".to_owned()) }`) |
| Failed link | `{n} con error` | `/prescriptions?status=failed` (`Route::Prescriptions { status: Some("failed") }`) |

`pending_sync` and `failed` are the shipped status values of the `/prescriptions` filter `select` (`main.rs:4532-4533`, options `Pendientes de sincronización` and `Con error de sincronización`); the band adds no filter state. The label set is the plan's `SyncStatus::label` contract (`domain.rs`, step 7): precedence offline → failed → pending → online. The shipped free function `sync_status_label` (`main.rs:128-136`) merges pending and failed into `Pendientes`; plan step 7 deletes it in favour of `SyncStatus::label` / `SyncStatus::tone` — the four labels above are the contract.

The band links both counts, per sitemap §1.3 and INDEX §5 ("the pending count links to `/prescriptions?status=pending_sync`"). `SyncStatusIndicator` itself stays link-free per its plan contract; the band renders the two count links after it (see §7 for the one contract delta).

## 4. Displayed information

### 4.1 What the header shows

| Element | Source | Value shown |
|---|---|---|
| Signed-out vs signed-in group | `Signal<Option<SessionUser>>` context, filled by `App` from `GET /api/auth/get-session` | email + `Cerrar sesión`, or `Iniciar sesión` |
| Link set | `Signal<Option<Me>>` context and the §2.1 guards | zero to five links |
| Active link | router current route, via `NavLink` `active_class` | Deep Teal 500 on the matching link |
| Sync indicator | `Signal<SyncStatus>` context; the band's own route-change `use_effect` (`main.rs:1386-1392`) re-runs `probe_sync_status()` (`main.rs:216`) on every navigation | label + dot + tone class |
| Pending link | `SyncStatus.pending > 0`, counted by `local_pending_count()` (`main.rs:139`, desktop; `:201` no-op stub returning 0 off desktop) | `· {n} pendientes` |
| Failed link | `SyncStatus.failed > 0`, counted by `local_failed_count()` (`main.rs:145` / `:205`) | `{n} con error` |
| Sign-out | click → `auth::sign_out()`, `clear_persisted_session()` (`main.rs:189` / `:194`), `user` and `actor` cleared, `navigator.push(Route::Home {})` | band flips to the signed-out group |

The band adds no fetch beyond these contexts and the route effect it already owns.

### 4.2 Hard rules

- No PHI in the band. The only identity shown is the account holder's own session email — identity, not patient data. No patient name, cédula, diagnosis, or medication ever appears here, and no link leaves the band carrying more than opaque ids or a status value (`sitemap.md` §5).
- Exactly five destinations when signed in. No sixth entry, no dropdown, no hamburger, no grouped "more" menu; overflow wraps (§5.3).
- No filled teal action anywhere in the band. `Design.md` allows one `#006e75` filled button per viewport and `Home.md` §3.3 reserves it for the hero CTA; `Design.md` build step 3 asks the nav strip for a teal `Emitir receta` button, and plan step 10 resolves it: the right group is ghost-only. Page-level primary actions (`Nueva receta` on `/prescriptions`, `Firmar y emitir` on the preview) live in each page's `PageHeader`, never in the bar.
- No link to a route the sitemap does not list, and no link that strands a shipped page: the §2.1 removals land only with the Settings hub cards that replace them.
- No billing, price, plan, trial, or `sync_locked` copy (D-04). `sync_locked` is v1.1; the indicator never renders it in v1.
- No legal line: the LOPDP text belongs to the hero (`Home.md` §3.2) and the public `Footer` (`Footer.md` §2.1) — never a third instance in the band.
- No signature or stamp imagery (D-06); the mint cross glyph and the pill are the band's only decoration.
- No language switcher: v1 is 100% `es-EC` (PRD §8).
- Not sticky, not fixed: the band sits at the top of document flow; `#navbar` carries no positioning rule and none is added (`Design.md` Layout: "no sticky header beyond the nav").
- Never hide sync loss (FR-043): whenever `pending > 0` the pending link renders, and whenever `failed > 0` the failed link renders — both counts always reachable, never colour-only.

### 4.3 Dynamic states

| State | Header behaviour |
|---|---|
| Session check in flight | Signed-out group renders (SSR first paint), then flips after hydration; `min-height: 64px` holds the band height, so nothing below shifts |
| Desktop offline | `Sin conexión` + `· {n} pendientes` link; every link and action stays enabled — offline issuance is supported (FR-040) |
| `pending > 0`, online | `Sincronizando` + pending link |
| `failed > 0` | `Error de sincronización` + the failed link, alongside the pending link when both counts are non-zero |
| Web build | `probe_sync_status()` reports reachability only; both counts are 0, so neither count link renders (FR-043 counting is desktop behaviour) |
| Click `Cerrar sesión` | sign-out runs, contexts clear, router pushes `/`; band becomes the signed-out group |
| Route change | active link follows the router; the same effect re-probes sync — no request per navigation beyond the one the shipped bar already makes |

## 5. Style

Every value is a `Design.md` token already declared in `persona-sanus/assets/main.css` `:root`. No new colour, font, icon package, CDN, or webfont.

### 5.1 Colour

| Use | Value | Token |
|---|---|---|
| Band background | `#ffffff` | `--color-paper` |
| Bottom rule | `#e5e7eb` | `--color-hairline` |
| Wordmark | `#006e75`, hover `#0b978e` | `--color-deep-teal`, `--color-bright-teal` |
| Logo mark | `#b9ffe8` circle, `#006e75` cross glyph | `--color-mint-glass`, `--color-deep-teal` |
| Pill badge | background `#b9ffe8`, text `#061d29` | `--color-mint-glass`, `--color-ink` |
| Nav link default | `#061d29` | `--color-ink` |
| Nav link hover | text `#006e75`, background `#fffded` | `--color-deep-teal`, `--color-cream` |
| Nav link active | `#006e75` weight 500 | `--color-deep-teal` |
| Ghost actions (`Iniciar sesión`, `Cerrar sesión`) | transparent, text `#061d29`, hover `#006e75` | `--color-ink`, `--color-deep-teal` |
| Email, sync label, pending link | `#425d6d`, link hover `#006e75` | `--color-mist`, `--color-deep-teal` |
| Sync dot | online `#b9ffe8`, pending `#fffded`, failed `#ffc3e6`, offline `#425d6d` | `--color-mint-glass`, `--color-cream`, `--color-blossom`, `--color-mist` |
| Failed link | `#4d0037` | `--color-plum` |
| Focus ring | `#0b978e` 2px outline, `outline-offset: 2px` (`assets/main.css:149-154`) | `--color-bright-teal` |

Never `#000000`. Never a chromatic border — the band's only border is the `#e5e7eb` hairline. Bright Teal is a fill/stroke only (3.60 on white, under the 4.5 body threshold): it never carries band text. The sync dot colour never encodes state alone; the text label always sits beside it (§6).

Measured contrast (WCAG AA needs ≥ 4.5 for body, same method as `Home.md` §5.1): Ink on white 17.28 · Ink on mint 15.25 · Ink on cream 16.89 · Mist on white 6.96 · Mist on cream 6.80 · Deep Teal on white 6.02 · Deep Teal on cream 5.89 · Plum on white 15.32. Focus ring Deep Teal/Bright Teal against Paper ≥ 3:1 as a non-text indicator.

### 5.2 Typography

| Element | Family | Weight | Size / line-height |
|---|---|---|---|
| Wordmark | `--font-mint` | 500 | 22px / 1.22, letter-spacing −0.01em (Logo Lockup) |
| Pill badge | `--font-mint` | 500 | 13px / 1.22 |
| Nav links | `--font-mint` | 400; active 500 | 16px / 1.22, Ink |
| Ghost action text | `--font-mint` | 400 | 16px (Ghost Nav Button) |
| Email | `--font-mint` | 400 | 14px, Mist, ellipsis |
| Sync label and both count links | `--font-mint` | 400 | 14px, Mist / Plum |

`--font-grenette` never appears in the band — display face, 46px+ only (`Design.md` Don't).

### 5.3 Geometry

- Base unit 4px. Band full-bleed, `min-height: 64px`; `.ui-navbar-inner` max-width 1200px, margin auto, padding `8px var(--spacing-24)`, grid `1fr auto 1fr`, gap `--spacing-16`, `align-items: center`, account group `justify-self: end`.
- Brand group: flex, centered, gap `--spacing-12`, `min-width: 0`; `.ui-logo` gap `--spacing-8`; mark a 22px circle holding a 14px cross glyph.
- Links: `nav.ui-navbar-links` flex, centered, wrap, gap `--spacing-4`; each link padding `8px 12px`, radius 8px, no underline.
- Account group: flex, centered, `justify-content: flex-end`, gap `--spacing-12`, `min-width: 0`; `.ui-sync` gap 6px; email `.ui-navbar-user` max-width 180px, nowrap, ellipsis.
- Pill: padding `4px 12px`, radius 9999px, nowrap.
- Responsive (plan §responsive): at 980px the inner grid becomes flex-wrap and the links take a full-width second row, still centered; below 760px the email hides and `Cerrar sesión` still names the action. No hamburger at any width.
- Elevation: flat. Hairline only; no drop shadow on any band element (`Design.md` Elevation).

### 5.4 Iconography and imagery

The filled `Cross` glyph inside the logo mark (`Icon` `Cross`, `fill="currentColor"`, `aria-hidden`) is the band's only icon. The sync dot is an 8px CSS circle, not an icon. No avatar, no chevron, no hamburger, no bell, no platform glyphs (those ride the hero trust row, `Home.md` §3.2), no photography. Any future icon follows the house rule: line-based, 1.5px stroke, monochrome Ink or Deep Teal.

## 6. Accessibility

- The band is a real `<header>` (banner landmark) containing one real `<nav>` for the five links. It holds no heading, so page heading order (`h1` → `h2`, `Home.md` §6) is untouched.
- Tab order: wordmark → each visible nav link → pending link (when present) → failed link (when present) → `Cerrar sesión` or `Iniciar sesión`. The pill, email, and sync dot are non-focusable. Every link and button shows the `:focus-visible` outline at `assets/main.css:149-154`.
- The active page is marked by Deep Teal colour **and** weight 500 — never colour alone. Both count links state their count in text; the failed link is additionally underlined, so `#4d0037` is never the only signal.
- `SyncStatusIndicator` carries `role="status"` and `aria-live="polite"` (plan contract); the dot is `aria-hidden` because the label text always names the state.
- The wordmark's `aria-label="PersonaSanus — inicio"` names its destination; the email is real selectable text; hiding it below 760px removes no operable affordance.
- §5.1 contrast meets AA for 13px–22px text. Ghost actions are identified by label, not colour.

## 7. Implementation notes

- The shipped `Navbar` (`main.rs:1377`) keeps its component identity, its three contexts, the route-change effect (`main.rs:1386-1392`), the role guards, and the sign-in/sign-out handlers — plan "Reuse, do not rebuild". Only markup and styles change: the `div#navbar` children become `header.ui-navbar` > `.ui-navbar-inner` with `LogoLockup` + `PillBadge` (`brand.rs`), `NavLink` (`button.rs`), and `SyncStatusIndicator` (`domain.rs`), per plan step 10.
- `#navbar` survives as a semantic id on `header.ui-navbar`, never a style hook (plan step 17). The `#navbar`-prefixed rules at `assets/main.css:156-193` — including the fixed `height: 64px` and `padding: 0 var(--spacing-32)`, which the target replaces with `min-height: 64px` and `8px 24px` — are deleted once the last page adopts the new band. New CSS is `.ui-navbar`, `.ui-navbar-inner`, `.ui-navbar-brand`, `.ui-navbar-links`, `.ui-nav-link`, `.ui-nav-link--active`, `.ui-navbar-account`, `.ui-navbar-user`, `.ui-sync` (+ `--online|--offline|--pending|--failed`, `.ui-sync-dot`, `.ui-sync-link`), `.ui-logo` (+ `-word`, `-mark`), `.ui-pill` — the plan's CSS rows for those classes and nothing else.
- One delta against the plan, recorded because sitemap §1.3 and INDEX §5 make the pending count a link while `SyncStatusIndicator` is contractually link-free: in the band the `· {n} pendientes` suffix is host-rendered as `Link.ui-sync-link.ui-sync-link--pending` instead of being emitted by the indicator, and `.ui-sync-link` gains a `--pending` tone (Mist text, Deep Teal hover) beside the existing Plum default for the failed link. `SyncStatusIndicator` keeps its plain, unlinked suffix for surfaces with no link of their own (the `Dashboard.md` widget). No other primitive contract changes.
- Do not change the layout graph here. `/verify/:code` already sits outside `#[layout(Navbar)]` (§2.2), so nothing about the plan step 12 public head conflicts with this band. If `#[end_layout]` and a public layout group land, `/` and `/login` move out and this file covers the shell band only; `context/architecture.md` §5 already records the current split.
- The `Mi perfil` / `Usuarios` / `Auditoría` removal is coupled to plan step 13 (Settings hub gains the `Auditoría` card). Land them together or keep `Auditoría` in the bar until step 13 ships.
- `context/screens/Home.md` keeps its §2 navbar row, points at this file for §3.1 copy, and defers the sync row to §4 here; no Home band, CTA, or hero copy changes.
- Verify, in order: `cd persona-sanus && cargo check`; `dx build --release --fullstack`; then
  - `curl -s http://127.0.0.1:8080/` contains `class="ui-navbar"`, `PersonaSanus`, `Receta verificable`, and `Iniciar sesión`; contains no `Sign in`, no `Mi perfil`, and no `ui-button--primary` inside the band (the hero keeps exactly one);
  - `curl -s http://127.0.0.1:8080/verify/PS-TEST` contains no `ui-navbar` — the layout claim of §2.2, held by the route declaration order;
  - a signed-in browser pass on `/dashboard` shows exactly five links with `Panel` active, the email, and `Cerrar sesión`, and no `Home`, `Usuarios`, or `Auditoría` entries;
  - desktop with a queued prescription: the band shows `Sincronizando` and `· {n} pendientes`, and that link lands on `/prescriptions?status=pending_sync` with the select preset to `Pendientes de sincronización`;
  - `caveman-blocks run persona-smoke --dir persona-sanus --port 8091` still returns `"ok": true`;
  - `caveman-blocks run markdown-table-check --path context/screens/Header.md` passes.
- Do not add: a sixth nav entry, a dropdown or hamburger, a filled teal action in the band, a search field, an avatar, a notification bell, a sticky or fixed position, a language switcher, legal text, or a sync label that hides a non-zero count.

## 8. Spec references

| Spec | Connection |
|---|---|
| `specs/15-O-ui-tokens-actions-and-formatting.md` | `15-O` supplies the tokens, `Icon`, `Button`, `NavLink`, and format helpers (`role_label`, `status_label`) the navbar band renders. |
| `specs/16-P-ui-forms-feedback-and-layout.md` | `16-P` supplies the badge, notice, and layout primitives reused by the navbar's session and sync groups. |
| `specs/17-Q-ui-brand-and-marketing-components.md` | `17-Q` builds `LogoLockup`, the brand mark the navbar renders at the left. |
| `specs/18-R-ui-domain-and-billing.md` | `18-R` builds `SyncStatusIndicator` (FR-043), the sync state the navbar's right group shows. |
| `specs/20-T-shell-auth-home-and-verify.md` | `20-T` adopts the navbar: five role-guarded destinations, no Home text link, session-aware right group. |
| `specs/25-Y-ui-legacy-css-cleanup-and-docs.md` | `25-Y` removes the legacy `#navbar` style hook; the id stays semantic on `header.ui-navbar`. |
