# PersonaSanus — Dashboard (`/dashboard`)

| Field | Value |
|---|---|
| Source | `context/sitemap.md` §1.2 (`/dashboard`, role-aware home, four staff roles, 🟡 partial), §2 (navigation by role), §3 flows 1–2 and 5, §4 M0/M2, §5 conventions; `context/PRD.md` §7 (state), FR-013, FR-040–FR-043, D-04, D-06, D-07, D-21; `context/Design.md` (§Tokens, Card, Detail Grid, Green Pill Badge, Do/Don't); `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` step 13 and Page adoption row `Dashboard` (line 225), plus `PageShell`, `PageHeader`, `Card`, `DetailGrid`, `DetailItem`, `Badge`, `ListRow`, `EmptyState`, `LoadingState`, `Button`; `context/screens/INDEX.md` §2.3 |
| Component | `Dashboard` — `persona-sanus/src/main.rs:1700-1759` (`#[component]` at `:1700`, `fn` at `:1701`). Reads `Signal<Option<SessionUser>>`, `Signal<Option<Me>>`, and `Signal<bool>` checked context; renders through planned `persona-sanus/src/ui/` primitives, which are not shipped yet |
| Route | `/dashboard` — authenticated shell route; no route guard in the enum (`persona-sanus/src/main.rs:56-57`). The page redirects with `navigator.replace(Route::AuthPage {})` once `checked()` is true and `user()` is `None`; signed-in staff of every role stay. No query parameter, no redirect for role |
| Layout | Inside `#[layout(Navbar)]` (`main.rs:51`, never closed): navbar band above, then `PageShell id="dashboard-page"` → `div#dashboard-page.ui-page > .ui-container.ui-stack`. Paper canvas, 48px top / 80px bottom / 24px inline, 24px stack gap. One `Card` `Sesión`; no full-bleed band, sidebar, or chart |
| Locale | `es-EC` on every string. `receta`, `cédula`, `clínica`. Dates `DD-MM-AAAA`; timestamps 24-hour. No currency in v1 |
| Status | ✅ Spec complete (sections 1–7). Route status: 🟡 — sitemap §1.2: "Implemented: auth-checked page that redirects signed-out users to `/login` and shows the session identity. ⬜ Role-aware content: recent prescriptions, drafts, pending sync (desktop), quick actions (New prescription, New patient, New template)." Target: adopted shell plus those role-aware blocks, all read through existing endpoints; no new endpoint, no billing, no clinical write from the Dashboard |

## 1. Purpose

The signed-in landing point. It answers "who am I, what can I do now, and what is waiting" without becoming a second work queue. Four jobs, in this order:

1. Confirm the session — display name, account email, role, clinic. A staff member with no clinic or no role gets the missing-profile path.
2. Offer the fastest legal entry point into issue work — `Nueva receta` only for doctor and nursing, `Nuevo paciente` only for doctor, nursing, and secretary, `Nueva plantilla` only for doctor and administrator.
3. Surface the small queue that needs attention — recent prescriptions, the caller's drafts, and on desktop the outbox counts already held by `Signal<SyncStatus>`.
4. For an Administrator, route to staff, clinic, and audit instead of clinical authoring.

The page never renders: another user's patients or prescriptions outside the caller's authorized clinic/role scope; PHI in a URL, title, analytics event, or log line; billing, price, trial, plan, quota, or `sync_locked` copy (D-04, v1.1); signature or stamp imagery (D-06); a rich-text or `contenteditable` surface (D-21); a duplicate sign-out control (the navbar owns it, `Header.md` §3.3); a greeting, invented counters, charts, streak, or productivity metric; a filled Primary action (the plan assigns this page none). It never signs, voids, supersedes, publishes, uploads, or changes a role.

## 2. Structure

One centered page column. Paper surfaces throughout; no tinted band, no imagery.

| Order | Region | `id` / class | Band tone | Contents |
|---|---|---|---|---|
| — | Navbar (layout) | `header.ui-navbar` → `.ui-navbar-inner` | — | Owned by `context/screens/Header.md` §2–§oses3; renders above this page unchanged |
| 1 | Page canvas | `PageShell id="dashboard-page"` → `div#dashboard-page.ui-page > .ui-container.ui-stack` | Paper `#ffffff` | 48px top, 80px bottom, 24px inline; max 1200px; 24px vertical gap |
| 2 | Page header | `header.ui-page-head` | Paper | `h1.ui-page-title` `Panel`; subtitle from the session (name, or email while the profile is absent); no header action |
| 3 | Session card | `SectionCard` `Sesión` → `div.ui-card` | Paper, 1px hairline | `DetailGrid` with `Nombre`, `Correo`, `Rol` (`Badge`), `Clínica` or `Sin clínica asignada.` |
| 4 | Quick actions | `.ui-actions` inside a `Card` | Paper | Zero to four Secondary `ButtonLink`s, role-guarded (§2.2) |
| 5 | Recent prescriptions | `SectionCard` `Recetas recientes` | Paper | Up to 5 `ListRow`s, each with patient name, status badge, date, number when present, Secondary `Abrir`; `EmptyState` when empty |
| 6 | Drafts | `SectionCard` `Borradores` | Paper | Up to 5 `ListRow`s from the same authorized set, status `Borrador`; only when the role can create; `EmptyState` when empty |
| 7 | Pending sync (desktop) | `SectionCard` `Sincronización` / sync widget | Cream `#fffded` strip inside a Paper card | Pending and failed counts, `Provisional` badge, links to the filtered prescription lists; FR-043 |
| 8 | Admin shortcuts | Links inside the quick-actions card | Paper | `Usuarios y roles`, `Clínica y biblioteca`, `Auditoría`; Administrator only |

Regions 2–4 ship in the current page; regions 5–8 are the sitemap's role-aware target, composed from existing read endpoints and the existing sync context. The page never grows a second card for the same idea: quick actions are one card, recent and drafts are distinct `SectionCard`s because they answer different questions.

### 2.1 Session check and signed-out state

While `checked()` is false, the page renders `PageShell id="dashboard-page"` and one `LoadingState` `Cargando...`; no session fields, no name, no email, no links, no API call. When `checked()` is true and `user()` is `None`, the effect replaces the route with `/login`; the page renders only the loading state until navigation, never a signed-out dashboard shell. It does not use `push`, so the back button cannot return to a stale dashboard.

When the session exists but `actor()` (`Me`) is still absent, the header and session card render the account identity from `SessionUser`; `Rol` and `Clínica` render the pending/missing state rather than guessing a role. The page never derives a role from the email, the name, or a prior render.

### 2.2 Role-aware blocks

| Role | Session card | Quick actions | Recent | Drafts | Pending sync | Admin shortcuts |
|---|---|:---:|:---:|:---:|:---:|---|
| Administrator | ✓ | Secondary `Nueva plantilla` | ✓ read-only | — | desktop only | `Usuarios y roles`, `Clínica y biblioteca`, `Auditoría`, `Mi perfil` |
| Doctor | ✓ | Secondary `Nueva receta`, `Nuevo paciente`, `Nueva plantilla` | ✓ own | ✓ own `draft` | desktop only | — |
| Nursing | ✓ | Secondary `Nueva receta` (draft), `Nuevo paciente` | ✓ authorized | ✓ `draft` | desktop only | — |
| Secretary | ✓ | Secondary `Nuevo paciente` | ✓ reprint scope | — | desktop only | — |
| No role / no profile | ✓ account only | `Completar perfil` → `/settings/profile` | — | — | — | — |
| Signed out | — | — | — | — | — | — |

Guards are the shipped `Me` methods: `can_create_prescriptions()` for `Nueva receta` (doctor, nursing), `can_write_patients()` for `Nuevo paciente` (doctor, nursing, secretary), `can_manage_templates()` for `Nueva plantilla` (administrator, doctor), `can_read_prescriptions()` for both lists (any role), `is_administrator()` for the admin shortcuts. A control the guard denies is absent, not disabled. The role matrix is UX only: every list and every create route remains server-authorized.

### 2.3 Recent prescriptions and drafts

Both lists read the same authorized response from `api::fetch_prescriptions("", "", "")`; the Dashboard adds no endpoint, no query parameter, and no client-side role filter. `Recetas recientes` keeps the first five returned rows in their server order. `Borradores` keeps the first five rows whose `status == "draft"`; a signed, superseded, or voided row never appears there.

Each row is a `ListRow`: title `patient_name`, meta status label · created date (`DD-MM-AAAA`) · `No. {rx_number}` when present, badge from `PrescriptionStatusBadge`, trailing Secondary `Abrir` to `/prescriptions/{id}`. On desktop, `pending_sync` and `failed` rows keep their badge and `Abrir` still opens the local detail; on web those statuses are not local states.

The two sections are not duplicate filters over a client cache with an age cut: the 30-day local retention and on-demand regeneration rules belong to `/prescriptions` (`Prescriptions.md` §4.3) and `PrescriptionPreview.md`. The Dashboard shows the latest authorized rows only, and its refresh is the page mount.

### 2.4 Pending sync (desktop only)

The widget reads `Signal<SyncStatus>` (`api::SyncStatus`): `online`, `pending`, `failed`. It renders only when `pending > 0 || failed > 0`; a healthy online desktop shows no widget, so the page is not a permanent sync billboard. Pending copy uses the same precedence and singular/plural rule as `Header.md` §3.3: `· {n} pendientes` / `· 1 pendiente`. The pending link goes to `/prescriptions?status=pending_sync`; the failed link uses `failed` and the same Plum treatment as the navbar. The widget never offers a "sync now" button, never drains the outbox, and never zeroes a count to look healthy.

On web the widget is absent: `probe_sync_status()` reports reachability only and both counts stay 0, because there is no local outbox on web.

## 3. Copy text

All strings verbatim, `es-EC`; punctuation belongs to the copy. Navbar strings are owned by `context/screens/Header.md` §3 and are not repeated here.

### 3.1 Page header and session card

| Element | Copy |
|---|---|
| Page title (`h1`) | `Panel` |
| Page subtitle, session present | the session display name; while absent, the account email |
| Session card heading (`h2`) | `Sesión` |
| Nombre label | `Nombre` |
| Correo label | `Correo` |
| Rol label | `Rol` |
| Clínica label | `Clínica` |
| Clinic missing | `Sin clínica asignada.` |
| Role label, administrator | `Administrador` |
| Role label, doctor | `Doctor` |
| Role label, nursing | `Enfermería` |
| Role label, secretary | `Secretaría` |
| Role label, missing | `Sin rol` |
| Profile missing action | `Completar perfil` |

### 3.2 Quick actions

| Element | Copy | Target |
|---|---|---|
| Doctor, create prescription | `Nueva receta` | `/prescriptions/new` |
| Doctor / nursing / secretary, create patient | `Nuevo paciente` | `/patients/new` |
| Doctor / administrator, create template | `Nueva plantilla` | `/templates/new` |
| Administrator, staff | `Usuarios y roles` | `/settings/users` |
| Administrator, clinic | `Clínica y biblioteca` | `/settings/clinic` |
| Any role, own profile | `Mi perfil` | `/settings/profile` |
| Administrator, audit | `Auditoría` | `/audit` |
| Missing profile | `Completar perfil` | `/settings/profile` |

### 3.3 Lists and sync

| Element | Copy |
|---|---|
| Recent section heading (`h2`) | `Recetas recientes` |
| Recent empty title | `Sin recetas recientes` |
| Recent empty description | `Todavía no hay recetas en esta clínica.` |
| Recent list action | `Ver todas` |
| Row open action | `Abrir` |
| Drafts section heading (`h2`) | `Borradores` |
| Drafts empty title | `Sin borradores` |
| Drafts empty description | `No hay borradores en curso.` |
| Sync section heading (`h2`) | `Sincronización` |
| Sync pending link | `· {n} pendientes`; singular `· 1 pendiente` |
| Sync failed link | `{n} con error` |
| Sync offline label | `Sin conexión` |
| Sync failed label | `Error de sincronización` |
| Sync pending label | `Sincronizando` |
| Sync online label | `En línea` |
| Loading gate | `Cargando...` |
| Redirect gate | `Redirigiendo...` |
| List fetch error fallback | `No se pudieron cargar las recetas.` |
| List loading fallback | `Cargando recetas...` |
| Prescription status, draft | `Borrador` |
| Prescription status, signed | `Firmada` |
| Prescription status, superseded | `Reemplazada` |
| Prescription status, voided | `Anulada` |
| Prescription status, pending sync | `Provisional` |
| Prescription status, failed | `Error de sincronización` |
| Prescription status, unknown | `Desconocido` |
| Role label, unknown | `Sin rol` |

## 4. Displayed information

### 4.1 What the screen shows

| Element | Source | Value shown |
|---|---|---|
| Session identity | `Signal<Option<SessionUser>>` context, filled from `GET /api/auth/get-session` | display name and email; the only identity shown |
| Role | `Signal<Option<Me>>` → `Me.role` | one role label, badge in the session card |
| Clinic | `Me.clinic` → `ClinicSummary.name` | clinic name, or `Sin clínica asignada.` |
| Quick actions | `Me::can_create_prescriptions`, `can_write_patients`, `can_manage_templates`, `is_administrator` | the §2.2 set; absent when denied |
| Recent rows | `api::fetch_prescriptions("", "", "")` → `Prescription.patient_name`, `.status`, `.created_at`, `.rx_number` | first five authorized rows |
| Draft rows | same response, `Prescription.status == "draft"` | first five drafts |
| Row badge | `PrescriptionStatusBadge(status)` | localized status, never a raw enum |
| Row date | `format_date(created_at)` | `DD-MM-AAAA` |
| Pending/failed counts | `Signal<SyncStatus>` (`pending`, `failed`), desktop only | integers; counts are not inferred from the recent list |
| Pending link | static route | `/prescriptions?status=pending_sync` |
| Failed link | static route | `/prescriptions?status=failed` |

The Dashboard performs no write and sends no role, clinic, doctor, or filter parameter. The server scopes the list by the session and role; the client renders the response unchanged.

### 4.2 Hard rules

- Authorization is server-side. The redirect and the guard methods only shape paint; they are not a boundary. A signed-out client that renders the shell still gets no authorized data.
- No PHI in the URL, page title, analytics, or log line. The route is bare `/dashboard`; row links carry the opaque prescription id only.
- The page never invents a second source for "recent": it does not read the local cache directly, does not call an endpoint per role, and does not merge server and local rows. The sync counts are the only local values.
- No billing, price, trial, plan, quota, `sync_locked`, or upgrade copy (D-04). No image-library quota here.
- No signature or stamp image, field, placeholder, or "digital signature" copy (D-06). The doctor signs the printed page.
- No rich text, formatting toolbar, `contenteditable`, HTML payload, or markdown rendering (D-21).
- No duplicated sign-out. The navbar owns `Cerrar sesión`; this page has no session-exit button.
- No filled Primary. All page actions are Secondary `ButtonLink`s; the viewport's one teal fill belongs to the shell or a later screen. The Dashboard's page-adoption row names no primary action.
- A denied action is absent, not disabled. A missing clinic renders the sentence, not an empty badge or a link.
- No greeting, welcome banner, motivational copy, invented counts, charts, or “good morning, Dr.” personalization.
- The page never signs, voids, supersedes, publishes, uploads, deletes, changes a role, or edits a patient/prescription. It is a read surface plus navigation.
- One `h1` (`Panel`) and one heading per `SectionCard` (`Sesión`, `Recetas recientes`, `Borradores`, `Sincronización`); no heading is styled as a card title without being a real `h2`.

### 4.3 Dynamic states

| State | Screen behaviour |
|---|---|
| Initial mount, `checked() == false` | PageShell + `LoadingState` `Cargando...`; no cards, no session text, no fetch |
| Checked, no session | Effect replaces with `/login`; the page never paints an empty dashboard |
| Session present, `Me` still absent | Header and `Sesión` card use `SessionUser`; role/clinic show the pending or missing labels; no role-guarded action is guessed |
| Session and profile, no role | `Rol` renders `Sin rol`; the direct `Completar perfil` action replaces clinical quick actions |
| Administrator | Session card, read-only recent, admin shortcuts, `Nueva plantilla`; no `Nueva receta`, no `Nuevo paciente`, no drafts |
| Doctor | All three quick actions, recent + drafts, pending widget on desktop |
| Nursing | `Nueva receta` and `Nuevo paciente`; recent + drafts; no template action |
| Secretary | `Nuevo paciente`; recent reprint rows; no drafts, no new prescription, no template |
| Lists loading | `SectionCard` bodies show `LoadingState` `Cargando recetas...`; the session card is already usable |
| Lists empty | `EmptyState`; no fabricated count, no “0 recetas” badge |
| List fetch error | `ErrorText` with the returned message or the §3.3 fallback; quick actions remain available |
| Desktop offline, queue non-empty | Pending or failed sync widget with counts and filtered links; recent rows may include provisional docs |
| Desktop offline, queue empty | No widget; no “all good” banner |
| Web | No pending/failed widget; no local-cache language |
| Click a quick action | Normal route navigation; no mutation on the Dashboard |
| Click `Abrir` | `/prescriptions/:id`; the detail screen owns every transition |
| Session expires mid-visit | Next authorized fetch/route redirects to `/login`; the page does not keep stale PHI on screen after a failed re-check |

## 5. Style

Every value is a `context/Design.md` token declared in `persona-sanus/assets/main.css` `:root`. No new color, font, icon package, CDN, or webfont.

### 5.1 Color

| Use | Value | Token |
|---|---|---|
| Page canvas, cards | `#ffffff` | `--color-paper` |
| Headings, body, values | `#061d29` | `--color-ink` |
| Subtitles, labels, meta, empty description | `#425d6d` | `--color-mist` |
| Card border, table rule, divider | `#e5e7eb` | `--color-hairline` |
| Link and active action text | `#006e75` | `--color-deep-teal` |
| Hover action text | `#0b978e` | `--color-bright-teal` |
| Pending sync wash | `#fffded` | `--color-cream` |
| Failed sync wash and danger fills | `#ffc3e6` | `--color-blossom` |
| Failed sync text | `#4d0037` | `--color-plum` |
| Focus halo | `rgba(185,255,232,0.55)` | mint glass at 55% |

Never `#000000`. Never a chromatic border; the hairline is the only border. Colour never carries status alone: role, status, and sync labels are printed text.

Measured contrast: Ink on Paper 17.28:1 · Mist on Paper 6.96:1 · Deep Teal on Paper 6.02:1 · Paper on Deep Teal 6.02:1 · Plum on Paper 15.32:1 · Plum on Blossom 10.36:1. AA for body; AA large for 22px headings.

### 5.2 Typography

| Element | Family | Weight | Size / line-height |
|---|---|---|---|
| Page title (`h1`) | `--font-mint` | 500 | 22px / 1.22, Ink |
| Page subtitle | `--font-mint` | 400 | 14px / 1.5, Mist |
| Section heading (`h2`) | `--font-mint` | 500 | 20px / 1.25, Ink |
| Card label | `--font-mint` | 500 | 14px / 1.5, Ink |
| Card value | `--font-mint` | 400 | 16px / 1.5, Ink |
| Role badge | `--font-mint` | 500 | 13px / 1.22 |
| List title | `--font-mint` | 500 | 18px / 1.25, Ink |
| List meta | `--font-mint` | 400 | 14px / 1.5, Mist |
| Action label | `--font-mint` | 500 | 16px / 1.22 |
| Empty title | `--font-mint` | 500 | 18px / 1.25, Ink |
| Empty description | `--font-mint` | 400 | 14px / 1.5, Mist |
| Sync count | `--font-mint` | 400 | 14px / 1.5, Mist |

No `--font-grenette`: this is an application screen, not a marketing hero. The display face is reserved for 46px+ section openers (`Design.md` Do/Don't).

### 5.3 Geometry

- 4px base unit. `PageShell`: 48px top, 80px bottom, 24px inline; `.ui-container` max 1200px centered; `.ui-stack` 24px vertical gap. No page-specific width wrapper.
- `PageHeader`: flex row, space-between, align start, wrap, 16px gap; subtitle 4px below title; no action at 24px top.
- `Card` and `SectionCard`: 24px radius, 1px hairline, 24px padding, Paper, no shadow.
- `DetailGrid`: two columns, 16px column gap, 24px row gap; one column below 760px. `DetailItem` label 4px above value; label 14px Mint 500 Mist, value 16px Ink.
- Quick actions: `ActionRow`, 12px gap, wraps; Secondary buttons 12px × 24px, 8px radius; no full-bleed.
- `ListRow`: 24px radius, 1px hairline, 16px 24px padding; title 18px, meta 14px, actions right aligned; rows separated by 8px, not by shadows.
- Sync strip: inside a Paper card, Cream `#fffded` wash, 12px 16px padding, 8px radius; 8px dot-to-text gap; counts as text links. No elevation.
- Empty state: Cream surface, 24px radius, 32px padding, 48ch maximum description width, centered; no icon larger than 20px.
- Loading state: shared spinner plus label on the page stack; 16px padding; no skeleton or shimmer.
- Responsive: cards stack; the session grid goes one column below 760px; list rows keep title above actions, never a horizontal scroll; buttons wrap with 12px gap.
- Elevation: flat. No card, row, button, or widget shadow; separation is hairline + wash.

### 5.4 Iconography and imagery

Line icons only, 1.5px stroke, 20px default, Ink or Deep Teal; Mist for meta. The page may use the shared lock/shield glyph beside the clinic line and the alert glyph in an error; no custom illustration. No photos, avatars, logos, patient images, signature, stamp, QR, PDF thumbnail, or chart. The role badge is a word, not a glyph. The sync dot is the shared 8px CSS circle from `Header.md` §5. Declared `BadgeTone`: Neutral for no role, Mint for a doctor/administrator, Cream for pending, Blossom/Plum for failed—only if the shared badge maps those tones; do not invent a new tone.

## 6. Accessibility

- Landmarks: the shell navbar is the banner (`Header.md`); the page content is a `main`-equivalent `PageShell` region. Each `SectionCard` is a labelled `section` whose `h2` is the accessible name. Do not add a second banner or a nested `nav`.
- Headings: exactly one `h1`, `Panel`, in `PageHeader`. Each card has one `h2` in DOM order: `Sesión` → `Recetas recientes` → `Borradores` → `Sincronización` → admin shortcuts if it is a separate card. No skipped level, no heading used for visual size.
- Tab order: `Completar perfil` (when present) → quick actions in §2.2 order → `Abrir` rows of recent → `Abrir` rows of drafts → pending/failed links → navbar. The email and clinic name are text, not links. No focus trap; no positive `tabindex`.
- Focus: the shared utility — `outline: 1px solid var(--color-deep-teal); outline-offset: 2px; box-shadow: 0 0 0 3px rgba(185,255,232,0.55);` — on each `ButtonLink`, row link, and count link. Visible on Paper and Cream.
- Labels: quick actions and row actions are real link text; no icon-only control. Role and status badges have accessible text, not a title-only tooltip. `Abrir` is repeated per row; the row title is the accessible context (the link's accessible name may be `Abrir {patient_name}` from the `ListRow` contract, otherwise the row grouping names it — say which in implementation).
- Status: role, prescription status, and sync state are words plus colour; the colour is never the only signal. Failed count is underlined or otherwise differentiated at rest, not hover-only.
- Loading: `LoadingState` is `role="status"` and `aria-live="polite"`; the initial loading gate is announced once, not per card. `ErrorText` is `role="alert"` and is inserted, not focus-moved.
- Keyboard: all actions are native links/buttons; no drag-only, no hover-only, no pointer-only control. Target size at least 44px.
- Reduced motion: the shared spinner stops under `prefers-reduced-motion: reduce`; no other animation is introduced.
- Zoom/reflow: at 200% and 320px, cards and rows reflow to one column; no horizontal scroll; the page keeps its heading order.

## 7. Implementation notes

The shipped component is `fn Dashboard()` at `persona-sanus/src/main.rs:1701`, `#[component]` at `:1700`, closing before `Settings` at `:1760`. It reads three contexts: `Signal<Option<SessionUser>>`, `Signal<Option<Me>>`, and `Signal<bool>` checked. Its current markup is one `div#echo` with `h4` `Dashboard`, the session name and email, an optional `Rol:` line, an optional `Clínica:` line or `Sin clínica asignada.`, and a duplicate `Cerrar sesión` button rendered with an English `Sign out` label. That duplicate sign-out is removed; `Header.md` §3.3 owns the action. The current page also renders an English `Loading...` gate and a `Redirecting to sign in...` fallback, which the target copy replaces with `Cargando...` and `Redirigiendo...`.

Plan step 13 adoption, and the exact delta:

- Root becomes `PageShell id="dashboard-page"`; `h4` becomes `PageHeader` `Panel` with the session subtitle. The three identity paragraphs become a `Card` `Sesión` holding `DetailGrid`/`DetailItem` for `Nombre`, `Correo`, `Rol`, `Clínica`.
- `Rol` renders `Badge` with `role_label(profile.role.as_deref())`; `Clínica` renders `clinic.name` or the sentence `Sin clínica asignada.`. No role badge is invented when `Me` is absent.
- `LoadingState` replaces both bare loading paragraphs. Keep the existing effects and their `navigator.replace` targets exactly.
- No sign-out control survives on this page.
- Do not add or change any API call for the adoption itself. The current page calls `api::fetch_me()` only through `App`; the Dashboard adds no fetch.

The role-aware blocks in §2.2–§2.4 are the sitemap target, not part of the step-13 reskin. When they land, they must use only the existing read endpoints: `api::fetch_prescriptions("", "", "")` for the two lists, and the existing `Signal<SyncStatus>` for the desktop widget. The prescription list is already role- and clinic-scoped server-side; the Dashboard must not filter by `clinic_id` in the client, must not request another patient's data, and must not add a route or query parameter. The pending and failed links are the same two routes already documented in `Header.md` §3.3. The list caps at five rows per section and links to `/prescriptions`; the cap is presentation only, not a new API parameter.

Placement and ownership:

- `Recetas recientes` and `Borradores` sit below the session card and quick actions, in that order. They read the same fetched vector exactly once; a second fetch for drafts is a defect.
- The sync widget sits below the lists on desktop and is absent on web. It is not a second `SyncStatusIndicator`: the navbar keeps the live indicator (`Header.md` §4), and the Dashboard adds the counts plus filtered links. Do not duplicate the navbar's label/dot pair.
- The Administrator's shortcuts replace clinical creation actions; the page does not show disabled `Nueva receta`/`Nuevo paciente` buttons to an Administrator. `Nueva plantilla` remains legal for an Administrator because `can_manage_templates()` includes that role.
- The no-role state shows `Completar perfil` only; it does not show empty clinical sections that could tempt a direct URL. The clinical sections are omitted until a role exists.

Verification commands, run from the repo root; none is recorded here as executed:

```sh
cd persona-sanus && cargo check
dx build --release --fullstack
caveman-blocks run persona-smoke --dir persona-sanus --port 8091
python3 .blocks/markdown-table-check.py --path context/screens/Dashboard.md --strict
```

- Do not add: a second endpoint, a role-specific dashboard route, a chart, metric cards, a streak, a greeting, a notification bell, an avatar, or a marketing band.
- Do not add: a duplicate `Cerrar sesión`; the navbar owns it.
- Do not add: a filled Primary action; the page-adoption table names none.
- Do not add: a `Nueva receta` action for administrator or secretary, a `Nuevo paciente` action for administrator, a template action for nursing or secretary, or a disabled version of any denied action.
- Do not add: signing, voiding, superseding, publishing, uploading, deleting, or role-changing controls to the Dashboard.
- Do not add: billing, price, trial, plan, quota, `sync_locked`, payment status, or upgrade copy (D-04).
- Do not add: signature/stamp imagery or fields, patient photos, or a "digital signature" badge (D-06).
- Do not add: rich text, markdown rendering, `contenteditable`, or HTML injection (D-21).
- Do not add: PHI in the URL, page title, analytics, or logs.
- Do not add: client-side clinic or role filtering that re-implements the server's scope.

## 8. Spec references

| Spec | Connection |
|---|---|
| `specs/done/15-O-ui-tokens-actions-and-formatting.md` | `15-O` supplies the tokens, `Button`, `ButtonLink`, and format helpers (`role_label`) the dashboard renders. |
| `specs/done/16-P-ui-forms-feedback-and-layout.md` | `16-P` supplies `PageShell`, `PageHeader`, `SectionCard`, `DetailGrid`, `DetailItem`, `Badge`, `ListRow`, `ActionRow`, `LoadingState`, `EmptyState`, and `Notice` the dashboard blocks use. |
| `specs/done/18-R-ui-domain-and-billing.md` | `18-R` supplies `PrescriptionStatusBadge` for the recent/draft rows (FR-013/FR-043); the desktop pending-sync counts are plain `api::SyncStatus`, not a second `SyncStatusIndicator` (this file §7). |
| `specs/done/21-U-ui-dashboard-settings-and-patients.md` | `21-U` adopts `Dashboard`: `PageShell id="dashboard-page"`, `LoadingState` while `checked` is false, session `SectionCard`, no Primary action. |
| `specs/done/25-Y-ui-legacy-css-cleanup-and-docs.md` | `25-Y` removes the legacy page-ID width rules so the adopted page runs on `.ui-page`/`.ui-container`. |
