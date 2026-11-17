# PersonaSanus — SettingsUsers (`/settings/users`)

| Field | Value |
|---|---|
| Source | `context/sitemap.md` §1.2 (`/settings/users` — Users and roles, Administrator, invite/manage, assign role, revoke access, registered devices), §2 (Administrator only), §4 M0/M3, §5 (roles, no PHI in URLs); `context/PRD.md` §3 roles, §6 `profiles`/`clinic_members`/`devices`/`sessions`, §9 device access, D-03, D-04, D-06, D-21; `specs/done/04-D-devices-and-auth-transport.md`; `context/Design.md` (Card, Section Card, Data Table, Select, Badge, Danger Button, Tokens); `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` step 16, Page adoption row `SettingsUsers` (line 233), `PageShell`, `PageHeader`, `SectionCard`, `DataTable`, `SelectField`, `Badge`, `Button`, `EmptyState`, `Notice`, `ErrorText`; `context/screens/INDEX.md` §2.3 |
| Component | `SettingsUsers` — `persona-sanus/src/main.rs:2120-2317` (`#[component]` at `:2119`, `fn` at `:2120`). Reads `actor: Signal<Option<Me>>` and `checked: Signal<bool>`; holds `users: Signal<Vec<api::UserRow>>`, `devices: Signal<Vec<api::DeviceRow>>`, `refresh: Signal<i32>`, `message: Signal<String>`. Target composes planned `src/ui/` primitives; that directory is not shipped |
| Route | `/settings/users` (`Route::SettingsUsers {}`, `main.rs:68-69`) — authenticated shell, Administrator only. No session replaces `/login`; a non-Administrator sees `Acceso denegado`; an Administrator stays. The route has no path or query parameter |
| Layout | Inside `#[layout(Navbar)]` (`main.rs:51`, never closed). `PageShell id="settings-page"` → `div#settings-page.ui-page > .ui-container.ui-stack`; Paper canvas, 48px top / 80px bottom / 24px inline, 24px stack gap; one `SectionCard` `Personal`, then one `SectionCard` `Dispositivos`; no full-bleed band, tabs, drawer, or modal |
| Locale | `es-EC` on every string. Role labels `Administrador`, `Doctor`, `Enfermería`, `Secretaría`; status `Activo`, `Deshabilitado`, `Revocado`, `Nunca`; platform `macOS`, `Windows`, `Web`, `Desconocido`; dates `DD-MM-AAAA`, times 24-hour |
| Status | ✅ Spec complete (sections 1–7). Route status: ⬜ — sitemap §1.2: planned, with §3 noting the shipped component. Shipped: Administrator gate, staff list with role select, disable/enable, remove membership, device list, revoke device, shared page message, refresh counter. Target: step-16 shell adoption with `SectionCard` + `DataTable`, row-scoped pending/error state, explicit destructive confirms, and no invite form (not shipped and no endpoint) |

## 1. Purpose

The Administrator's staff and device control room for one clinic. Four jobs, in order:

1. **Identify.** Show each clinic member by display name, account email, join date, account status, and current role.
2. **Assign.** Change exactly one member's role to Administrator, Doctor, Nursing, or Secretary; the server enforces the last-Administrator rule.
3. **Suspend or remove.** Disable an account without deleting its profile, re-enable a disabled account, or remove a profile from the clinic membership.
4. **Revoke devices.** List registered devices with platform, app version, last-seen, and state; revoke an active device so its sessions stop being accepted.

It never renders:

- Patient PHI. The only identities are staff accounts; no patient name, cédula, birth date, diagnosis, medication, prescription, or patient image is fetched or painted.
- A password, password hash, session token, cookie, MFA secret/URI, or device secret. Staff credentials are never managed here; `/settings/profile` owns the MFA/password surface for the signed-in user.
- An invite, create-user, delete-profile, hard-delete, or email-resend control. No invite endpoint ships; members appear after registration.
- Billing, price, trial, plan, entitlement, `sync_locked`, or subscription copy (D-04). No billing admin exists in v1.
- A signature or stamp image (D-06), rich text or `contenteditable` (D-21), patient photo, chart, or audit row.
- A clinic picker or tenant switch: one actor is in exactly one clinic (D-03), and the server derives it.
- A second filled Primary. This page has no Primary at all; role changes and row actions are Secondary/Danger; no “Save” button.
- A role selector for the caller's own row that is treated as safe: the server allows self-demotion when another Administrator remains, so the UI must not claim otherwise.

## 2. Structure

One centered page column, Paper, two `SectionCard`s. Each card owns one `DataTable`; empty state sits inside the table's card, not in place of the card.

| Order | Region | `id` / class | Band tone | Contents |
|---|---|---|---|---|
| — | Navbar (layout) | `header.ui-navbar` → `.ui-navbar-inner` | — | Owned by `context/screens/Header.md` §2–§3 |
| 1 | Page canvas | `PageShell id="settings-page"` → `div#settings-page.ui-page > .ui-container.ui-stack` | Paper `#ffffff` | 48px top, 80px bottom, 24px inline; 1200px container; 24px stack gap |
| 2 | Page header | `header.ui-page-head` · `h1.ui-page-title` · `p.ui-page-subtitle` | Paper | `Usuarios y roles`; `Asigna un rol o deshabilita el acceso de cada persona.`; no action |
| 3 | Feedback | `ErrorText` | Paper | One page-level message slot above the cards; `role="alert"`; only when non-empty |
| 4 | Personal card | `SectionCard` `Personal` · `DataTable` | Paper, hairline | Columns `Persona`, `Correo`, `Alta`, `Estado`, `Rol`, `Acciones`; one row per profile |
| 5 | Dispositivos card | `SectionCard` `Dispositivos` · `DataTable` | Paper, hairline | Heading `Dispositivos`; description `Equipos registrados y su último acceso.`; columns `Equipo`, `Plataforma`, `Versión`, `Última conexión`, `Estado`, `Acciones`; one row per device |
| 6 | Access denied | `Notice tone=Error` | Paper | Replaces both cards for a non-Administrator: `Acceso denegado`, the reason, Secondary `Ir a mi perfil` |
| 7 | Session gate | `LoadingState` | Paper | `Cargando...` before the session check; `Redirigiendo...` when checked without a profile |

The two cards are always in that order. A request error in one does not reorder them; the page-level `ErrorText` sits above both and the successful card keeps its data.

### 2.1 Session finish and role

`checked` is the shell session-check signal; `actor` is `Signal<Option<Me>>`. `Me::is_administrator()` (`api.rs:49-51`) is the only page gate, and `clinic_members.role` is the role authority (D-03).

| Session finish / role | Cards | What renders |
|---|---|---|
| Check in flight (`!checked`) | no | `PageShell` + one `LoadingState` `Cargando...`; no fetch |
| Checked, no session | no | `LoadingState` `Redirigiendo...`; effect replaces `/login`; no fetch |
| Checked, `actor == None` | no | Same redirect notice; effect replaces `/login`; no fetch |
| Non-Administrator | no | `Notice tone=Error` `Acceso denegado` + the reason + Secondary `Ir a mi perfil`; no fetch |
| Administrator | both | Fetches staff and devices; paints both cards even if one list is empty |

The page never paints a reduced staff table shell for a non-Administrator. Authorization is still server-side on every mutation; the client gate is UX only.

### 2.2 Staff table

One row per profile that shares the clinic. Cell contents:

- **Persona** — `display_name`.
- **Correo** — `email`.
- **Alta** — `format_date(created_at)` → `DD-MM-AAAA`.
- **Estado** — `AccountStatusBadge(status)` → `Activo` (Mint) or `Deshabilitado` (Neutral).
- **Rol** — `SelectField hide_label`, current role value, options: disabled `Sin rol`, `Administrador`, `Doctor`, `Enfermería`, `Secretaría`. A change calls `set_role(profile_id, role)` immediately; there is no save button. Empty value is ignored.
- **Acciones** — Secondary `Habilitar` when disabled; Danger `Deshabilitar` when active; Danger `Quitar del equipo` when a role exists. The remove action removes clinic membership, not the auth profile or its devices.

The page has no invite form. A new staff member appears after sign-up joins the clinic elsewhere; this screen edits existing memberships only. No pagination/search/sort is shipped.

The server refuses two lockout cases with `400` and the message `The clinic needs an administrator.`:
- changing the last Administrator to another role;
- disabling the last Administrator.

Removing the last Administrator is likewise refused. The page renders the server message verbatim; it does not pre-compute the count client-side. Self-demotion/self-disable is allowed while another Administrator remains, so the UI must not print “you cannot change your own role.” Removing the current Administrator's own membership when another admin remains is also allowed by the service; assume the row can disappear after refresh.

### 2.3 Device table

One row per registered device returned by `fetch_devices`. Cell contents:

- **Equipo** — `device_name_label`: `device.name` trimmed if non-empty, else the platform label. The row also prints `profile_name` as owner context; do not merge the two into an unlabeled cell.
- **Plataforma** — `platform_label` → `macOS`, `Windows`, `Web`, `Desconocido`.
- **Versión** — `app_version` or `—`.
- **Última conexión** — `last_seen_label` → `DD-MM-AAAA HH:MM`, or `Nunca`.
- **Estado** — `DeviceStatusBadge(revoked_at.is_some())` → `Activo` (Mint) / `Revocado` (Blossom).
- **Acciones** — active device: Danger `Revocar`; revoked device: none.

Revoking ends the device's sessions on the next request (`sessions.device_id`), so an affected client may lose its session immediately. The page does not log out other tabs locally; it re-fetches after success and relies on the server. The device list is clinic-scoped by the service; the client sends no profile or clinic id.

### 2.4 Busy, error, and refresh

The shipped component has one `refresh: Signal<i32>` and one page-level `message`. A successful role change, enable/disable, remove, or revoke increments `refresh`; the effect re-runs both fetches and the row list settles. A failure writes the server message; the row stays as it was. There is no optimistic cell edit and no per-row error placement in the shipped code.

Target adds one in-flight key for the row action (`profile_id` or `device_id`) so only that action disables and shows its busy label; the page-level `ErrorText` remains the failure surface. The key is local UI state, not an API field. A failure clears the key and re-enables the action. Do not add a toast or a modal.

The effect fetches users and then devices even if the users fetch failed. The second failure overwrites the first message; target keeps the first message if both fail and records the second in a hidden diagnostic, or shows a combined page error naming both lists. Do not claim a partial success as a complete load.

## 3. Copy text

All strings verbatim, `es-EC`; punctuation belongs to the copy. Navbar copy is owned by `Header.md` §3 and is not repeated.

### 3.1 Page and card headings

| Element | Copy |
|---|---|
| Page title (`h1`) | `Usuarios y roles` |
| Page subtitle | `Asigna un rol o deshabilita el acceso de cada persona.` |
| Staff card heading (`h2`) | `Personal` |
| Device card heading (`h2`) | `Dispositivos` |
| Device card description | `Equipos registrados y su último acceso.` |
| Access denied title | `Acceso denegado` |
| Access denied body | `Solo el administrador puede gestionar usuarios y roles.` |
| Access denied action, Secondary | `Ir a mi perfil` → `/settings/profile` |
| Session gate | `Cargando...` |
| Redirect gate | `Redirigiendo...` |

### 3.2 Staff table copy

| Element | Copy |
|---|---|
| Column 1 | `Persona` |
| Column 2 | `Correo` |
| Column 3 | `Alta` |
| Column 4 | `Estado` |
| Column 5 | `Rol` |
| Column 6 | `Acciones` |
| Role option, no role | `Sin rol` (disabled; placeholder only) |
| Role option, administrator | `Administrador` |
| Role option, doctor | `Doctor` |
| Role option, nursing | `Enfermería` |
| Role option, secretary | `Secretaría` |
| Status badge, active | `Activo` |
| Status badge, disabled | `Deshabilitado` |
| Enable action, Secondary | `Habilitar` |
| Disable action, Danger | `Deshabilitar` |
| Remove action, Danger | `Quitar del equipo` |
| Empty title | `Sin usuarios` |
| Empty description | `No hay usuarios para mostrar.` |

### 3.3 Device table copy

| Element | Copy |
|---|---|
| Column 1 | `Equipo` |
| Column 2 | `Plataforma` |
| Column 3 | `Versión` |
| Column 4 | `Última conexión` |
| Column 5 | `Estado` |
| Column 6 | `Acciones` |
| Platform, Apple | `macOS` |
| Platform, Windows | `Windows` |
| Platform, browser | `Web` |
| Platform, unknown | `Desconocido` |
| Last seen, never | `Nunca` |
| State, active | `Activo` |
| State, revoked | `Revocado` |
| Revoke action, Danger | `Revocar` |
| Empty title | `Sin dispositivos` |
| Empty description | `No hay equipos registrados.` |

### 3.4 Feedback and busy

| Element | Copy |
|---|---|
| Page error fallback | `No se pudieron cargar los usuarios y dispositivos.` |
| Role change busy | `Guardando rol...` |
| Enable busy | `Habilitando...` |
| Disable busy | `Deshabilitando...` |
| Remove busy | `Quitando...` |
| Revoke busy | `Revocando...` |
| Last-administrator refusal | `The clinic needs an administrator.` rendered verbatim from the server (target: localize at the source, do not rewrite here) |
| Unknown role refusal | `Unknown role.` rendered verbatim from the server (target: localize at the source) |

Server error strings are the source of truth for refusals and are shown unchanged. Do not add a success toast, banner, or “saved” checkmark: the refreshed row is the confirmation.

## 4. Displayed information

### 4.1 What the screen shows

| Element | Source | Value shown |
|---|---|---|
| Page title/subtitle | static | `Usuarios y roles`, `Asigna un rol o deshabilita el acceso de cada persona.` |
| Administrator gate | `Me::is_administrator()` (`api.rs:49-51`) | both cards or `Acceso denegado`; non-Administrators are denied |
| Staff rows | `api::fetch_users()` (`api.rs:617`) | `api::UserRow { profile_id, display_name, email, role, status, created_at }` |
| Join date | `row.created_at` through `format_date` | `DD-MM-AAAA` |
| Account status | `row.status` via `AccountStatusBadge` | `Activo` for active, `Deshabilitado` for disabled |
| Current role | `row.role: Option<String>` | `Administrador`, `Doctor`, `Enfermería`, `Secretaría`, or the disabled empty `Sin rol` |
| Role change | `api::set_role(profile_id, role)` (`PUT /api/users/{id}/role`) | server re-check; `refresh` on success; message on failure |
| Enable/disable | `api::disable_profile(profile_id, disabled)` (`POST /api/users/{id}/disable`) | server re-check; `refresh` on success; message on failure |
| Remove membership | `api::remove_member(profile_id)` (`DELETE /api/users/{id}`) | removes clinic membership, not the profile; `refresh` on success; message on failure |
| Device rows | `api::fetch_devices()` (`api.rs:559`) | `api::DeviceRow { id, profile_name, name, platform, app_version, last_seen_at, revoked_at }` |
| Device name | `device_name_label` (`main.rs:4297`) | trimmed `name`, else the platform label; `profile_name` stays visible as owner context |
| Platform | `platform_label` (`main.rs:4288`) | `macOS`, `Windows`, `Web`, `Desconocido` |
| Version | `app_version` | stored version, or `—` when absent |
| Last seen | `last_seen_label` (`main.rs:4307`) | `DD-MM-AAAA HH:MM`, or `Nunca` |
| Device state | `DeviceStatusBadge(revoked_at.is_some())` | `Activo` or `Revocado` |
| Revoke | `api::revoke_device(device_id)` (`POST /api/devices/{id}/revoke`) | server ends the profile's sessions on that device; `refresh` on success |
| Page error | `message: Signal<String>` | returned message or fallback; one line above the cards |

The tables are read from the API on mount and after every successful mutation. No row is edited locally before the server confirms; no count is invented; no role is derived from the email.

### 4.2 Hard rules

- The server is the authorization boundary at three layers: the page actor must be an Administrator, `service::set_member_role` / `remove_member` / `disable_profile` call `require_administrator`, and the device endpoints scope the target to the actor's clinic. Client role checks only shape paint.
- Role authority is `clinic_members.role` only (D-03). Never show a role from `profiles`, a session claim, or an email heuristic.
- The last-Administrator guard is server-side: demoting the last admin, disabling the last admin, and removing the last admin all return `400` with `The clinic needs an administrator.` The page renders that sentence verbatim and does not pre-disable the control, because a stale count could lie. The page must not claim self-protection it does not have.
- Self-service lockout is possible while another Administrator remains: a signed-in Administrator may demote, disable, or remove their own membership. The UI does not add a client-only block; it refreshes and the next request is authorized against the changed role. If a local “you” marker is added, it is informative, not a guard.
- Removing a member is not deleting an auth identity, a doctor row, or their devices. It removes clinic access only. Devices belonging to a removed profile may remain in the table until the next read; their sessions are rejected when the membership check fails, but a separate device purge is not this screen's action.
- Revoking a device ends that device's sessions on the next request. It does not sign out other devices and does not delete the device row; the row stays, its state becomes `Revocado`, and its action disappears. Re-registering the device is a separate client action on next start.
- No invite, create-user, delete-profile, password reset, MFA reset, role minting, or session editor exists here. Do not add an “invite” button that calls a nonexistent endpoint.
- No PHI: staff display name influencing care will appear; that is account identity, not patient data. Never place a patient name, cédula, diagnosis, medication, prescription number, or photo in a table, error, URL, `aria-label`, or title.
- No PHI in the URL: no staff id, email, role, or clinic id is in the path or query; actions carry ids in request bodies, not links. The route is exactly `/settings/users`.
- Plain text only (D-21): cells and errors are text, not HTML or markdown. Escape a display name/email; never `innerHTML`.
- No billing, price, trial, plan, entitlement, `sync_locked`, image quota, signature/stamp, patient photo, or audit copy (D-04, D-06). The page is staff access and devices only.
- One Primary per viewport, maximum: this page has none. Role changes are immediate Secondary controls; enable is Secondary; disable/remove/revoke are Danger. No filled teal action.
- Destructive wording must name the consequence: disable blocks access but keeps the profile; remove drops clinic membership; revoke ends that device's session. Do not soften labels with “deactivate” when the code/endpoint is not the same operation.
- Empty is per card, not a page-wide denial. Zero staff still shows the card with `Sin usuarios`; zero devices shows `Sin dispositivos`; neither is an error.
- The page never caches a result or writes an outbox row. It is web/online-only; a desktop offline request fails and shows the message.

### 4.3 Dynamic states

| State | Screen behaviour |
|---|---|
| Session check in flight | `PageShell` + `LoadingState` `Cargando...`; no cards, no fetch |
| Checked, no session | Replace `/login`; empty page during the transition |
| Checked, no profile | Replace `/login`; `Redirigiendo...` until then |
| Non-Administrator | `Notice` Error `Acceso denegado` + reason + Secondary `Ir a mi perfil`; no tables and no fetch |
| Administrator, initial fetch | Both cards paint; each table shows its own `LoadingState`/quiet body until the request settles; error line above |
| Staff empty | `Personal` card present with `Sin usuarios` / `No hay usuarios para mostrar.`; device card independent |
| Devices empty | `Dispositivos` card present with `Sin dispositivos` / `No hay equipos registrados.`; staff card independent |
| Users fetch fails | `ErrorText` above the cards; staff table suppressed/empty; device card still renders if its fetch succeeded |
| Devices fetch fails | Same error line; device table suppressed; staff table stays |
| Both fail | One error line naming the fallback; neither table fabricates rows |
| Role change in flight | That row's select disabled; label/status stay until the response; no optimistic role text |
| Role change success | `refresh` re-fetches; the row paints the new role; success needs no toast |
| Role change failure | The select returns to the stored value; server message in `ErrorText`; other rows stay usable |
| Enable/disable in flight | That row's action disabled with the §3.4 busy label; status badge stays |
| Enable/disable success | `refresh`; `Activo`/`Deshabilitado` flips; action set follows |
| Enable/disable failure | Message in `ErrorText`; status unchanged |
| Remove idle/confirm | Danger `Quitar del equipo`; if a confirm step is added, it is an inline `ConfirmPanel`, never a browser dialog or modal focus trap |
| Remove busy | The confirm button reads `Quitando...` and is disabled; the row remains until refresh |
| Remove success | `refresh`; the row disappears from the staff table; its devices may remain until the next read |
| Remove failure | Message in `ErrorText`; the row stays; confirm remains open for retry or is dismissed |
| Last Administrator refusal | `400` sentence in `ErrorText`; no optimistic demotion/disable/removal; the role/status stay |
| Unknown role refusal | `400` `Unknown role.`; the select returns to its previous value |
| Revoke idle | Active row shows Danger `Revocar`; revoked row shows no action |
| Revoke busy | That action reads `Revocando...`, disabled; other rows usable |
| Revoke success | `refresh`; row state `Revocado`; action gone |
| Revoke failure | Message in `ErrorText`; state stays `Activo`; action returns |
| Own Administrator row | Editable like any other; a save that would leave no admin is refused by the server, not a client guard |
| Long display name/email | Wraps inside its cell/block; no truncation, no horizontal page scroll; 44px target preserved |
| Unknown platform | `Desconocido`; no icon-only guess |
| Missing app version | `—` |
| Missing last seen | `Nunca` |
| Narrow viewport | Tables become stacked rows with visible cell labels (or horizontal scroll inside the table wrapper only). The page itself never scrolls horizontally; actions wrap below metadata |
| Reduced motion | No animation; the shared spinner, if any, stops under `prefers-reduced-motion` |

## 5. Style

Every value is a `context/Design.md` token declared in `persona-sanus/assets/main.css` `:root`. No new color, font, icon package, CDN, or webfont.

### 5.1 Color

| Use | Value | Token |
|---|---|---|
| Canvas, cards, table surfaces | `#ffffff` | `--color-paper` |
| Headings, names, cell text | `#061d29` | `--color-ink` |
| Subtitle, column headers, meta, hints | `#425d6d` | `--color-mist` |
| Hairlines: card, row, table rules | `#e5e7eb` | `--color-hairline` |
| Active state / focus ring | `#006e75` | `--color-deep-teal` |
| Focus halo | `rgba(185,255,232,0.55)` | `--color-mint-glass` |
| Active badge surface | `#b9ffe8` | `--color-mint-glass` |
| Disabled/revoked badge surface | `#fffded` | `--color-cream` |
| Danger text and destructive hover | `#4d0037` / `#ffc3e6` | `--color-plum`, `--color-blossom` |

Never `#000000`. Never a chromatic border; hairlines only. Danger is a text role, not a filled red button. Measured contrast: Ink/Paper 17.28 · Mist/Paper 6.96 · Paper/Deep Teal 6.02 · Ink/Mint 15.25 · Plum/Paper 15.32. AA for body.

### 5.2 Typography

| Element | Family | Weight | Size / line-height |
|---|---|---|---|
| Page title (`h1`) | `--font-mint` | 500 | 22px / 1.22 Ink |
| Page subtitle | `--font-mint` | 400 | 14px / 1.5 Mist |
| Section heading (`h2`) | `--font-mint` | 500 | 20px / 1.25 Ink |
| Table caption / column header | `--font-mint` | 500 | 14px / 1.5 Ink, start-aligned |
| Table cell | `--font-mint` | 400 | 14–16px / 1.5 Ink |
| Role select | `--font-mint` | 400 | 16px / 1.5 Ink |
| Badge | `--font-mint` | 500 | 13px / 1.22 |
| Row action label | `--font-mint` | 500 | 14–16px / 1.22 |
| Empty title / description | `--font-mint` | 500 / 400 | 18px / 1.25 Ink; 14px / 1.5 Mist |
| Error text | `--font-mint` | 400 | 14px / 1.5 Plum |

No `--font-grenette`; application chrome. Long emails/names wrap at 14–16px; never shrink to fit.

### 5.3 Geometry

- 4px base. `PageShell`: 48px top, 80px bottom, 24px inline; container max 1200px; stack gap 24px.
- `PageHeader`: title/subtitle 4px apart; no header action.
- `SectionCard`: 24px radius, 1px hairline, 24px padding, 16px gap between cards; heading 16px below.
- `DataTable`: full width, 1px hairline outer and row rules, 8px radius inner table wrapper? Follow the primitive; no custom one-off. Cell padding `12px 16px`; header background Paper (not a tint); row hover Cream at most.
- Role select: native, 8px radius, `8px 12px`, min-height 44px, intrinsic width; in a narrow cell it expands to the cell width.
- Actions: flex, wrap, 8–12px gap; each button 44px target; labels never truncate. Danger styling is text/outline, not a solid block.
- Empty state: within card body, 32px padding, Cream surface, 24px radius, title 18px; no icon unless shared primitive.
- Mobile below 760px: each table row becomes a stacked block; use visible labels (`Persona`, `Correo`, …) rather than relying on column position; 24px card padding drops to 16px; no horizontal page scroll. If the table primitive horizontally scrolls, only the table wrapper does.
- Elevation: flat; no shadow, no sticky header, no card lift.

### 5.4 Iconography and imagery

Line icons only, 1.5px stroke, 20px, Ink or Plum: `Alert` in the error notice; no platform logos, no avatar, no device glyph, no shield, no QR, no signature/stamp, no patient photo, no chart. A role or device is named by text and badge, not by icon. The Navbar owns the brand mark.

## 6. Accessibility

- Landmarks: navbar banner (`Header.md` §6); two labelled `section`s (`Personal`, `Dispositivos`) inside the content region. No nested `nav`.
- Headings: one `h1`, `Usuarios y roles`; one `h2` per `SectionCard`. The table caption is not a heading. The denied state has one `h1`, `Acceso denegado`, and no `h2`.
- `DataTable`: real `<table>` with `<caption>` naming the card (`Personal` / `Dispositivos`) and `th scope="col"` on every column header. A cell that needs an accessible name uses its visible text. Do not fake columns with `div` grid.
- Tab order: role select → `Habilitar` / `Deshabilitar` → `Quitar del equipo` per staff row; `Revocar` per active device row; then the `Ir a mi perfil` link in the denied state. Row labels give context; repeated `Revocar` links are distinguished by the row's `Equipo` text. No hidden focusable controls.
- Focus: `1px solid var(--color-deep-teal)`, offset 2px, halo `0 0 0 3px rgba(185,255,232,0.55)` on selects, buttons, and links. Never remove the native select focus without the ring.
- Labels: the role select is visually labelled by its column header `Rol`; if `hide_label` removes that association, it needs an `aria-label` that includes the member name, e.g. `Rol de {display_name}`. Do not leave an unnamed select.
- Busy: the acting control sets `aria-busy="true"` and is disabled; the busy label is visible. The rest of the table stays operable unless its own action is in flight. No focus move on completion.
- Errors: page `ErrorText` is `role="alert"`, inserted above the cards. A mutation failure does not clear the table, does not steal focus, and names the failed operation. The server's last-Administrator sentence is announced as text.
- Empty: `EmptyState` title and description are inside the card's `section`; a table with zero rows still renders its caption/headers if using `DataTable` (or the card shows `EmptyState` instead of an empty table). Choose one; the spec uses `EmptyState` instead of an empty `<table>` so a screen reader does not announce a blank grid.
- Status: badges are text; colour is supplementary. A revoked device is not disabled-only; the row prints `Revocado` and has no action.
- Contrast: Ink/Paper 17.28 · Mist/Paper 6.96 · Paper/Deep Teal 6.02 · Ink/Mint 15.25 · Plum/Paper 15.32 · Plum/Blossom 10.36. AA for body. The 1px hairline is decorative; text and focus carry meaning.
- Touch: at least 44px per action; 8–12px gaps; no drag, no swipe-to-delete, no hover-only menu.
- Reduced motion: no animation; the shared spinner respects `prefers-reduced-motion`.
- Zoom/reflow: 200% and 320px keep labels and actions visible, stack cells, and avoid clipped email addresses.

## 7. Implementation notes

The shipped component is `fn SettingsUsers()` at `persona-sanus/src/main.rs:2120`, `#[component]` at `:2119`, closing before `SettingsProfile` at `:2318`. It reads `actor` and `checked`; holds `users`, `devices`, `refresh`, `message`; one effect runs on `refresh` and `checked`.

Current behavior:

- `!checked()` paints `div#echo > p` `Cargando...`.
- A non-Administrator paints `div#echo` with `Acceso denegado` and `Solo el administrador puede gestionar usuarios y roles.`, then a `Link` `Ir a mi perfil`. It does not redirect. Target keeps the `Notice` Error surface: the hub's `/settings` redirect is a different route and does not apply here.
- The effect needs an actor; without one it replaces `/login`. With an actor it calls `api::fetch_users()` then `api::fetch_devices()`; each `Err` writes the same `message`, so the second failure overwrites the first.
- Staff rows are raw `div`; each renders `display_name`, `email`, `Alta: {format_date(created_at)}`, `Estado: {status_label(status)}`, a native `select` of `Sin rol`/`Administrador`/`Doctor`/`Enfermería`/`Secretaría`, `Quitar del equipo` when a role exists, and `Habilitar`/`Deshabilitar` by status.
- The select calls `api::set_role(profile_id, value)` on change, ignores an empty value, and bumps `refresh` on success. The buttons call `api::remove_member` / `api::disable_profile(profile_id, bool)` and bump `refresh`. Every failure writes the shared `message`; no per-row state exists.
- Devices render only when `!device_rows.is_empty()`. There is no `Sin dispositivos` empty state in the shipped markup, which is a gap: target must render the card with `EmptyState`, not hide it.
- Device rows print `{profile_name} · {device_name_label}`, `{platform_label} · Última conexión: {last_seen_label}`, `Versión: {app_version}` when set, `Estado: Revocado` when revoked, and a Danger `Revocar` only when active.
- `message` renders in an unstyled `p`; target uses `ErrorText` and does not paint success copy through it.
- No confirm panel exists for remove/disable/revoke. Add an inline `ConfirmPanel` for `Quitar del equipo` and `Revocar` if the product requires a second step; do not add a modal or a browser `confirm()`. Re-enable is direct.

Plan step 16 adoption:

- `PageShell id="settings-page"`; `PageHeader` `Usuarios y roles` with the shipped subtitle; no header action.
- `SectionCard` `Personal` holds `DataTable` with the six columns and one `<tr>` per profile. Cells are plain text; the role cell is a `SelectField hide_label`; the action cell is a wrapping `ActionRow` of Secondary/Danger buttons. The table caption is `Personal`; each `th` has `scope="col"`.
- `SectionCard` `Dispositivos` holds the description and a `DataTable` with the six columns and one `<tr>` per device. The action cell is Danger `Revocar` only for an active device. The table caption is `Dispositivos`.
- Add `EmptyState` inside each card when its vector is empty, with the §3 copy. Keep the card mounted; do not replace the page.
- Add a per-action pending key (or one `busy: Option<String>`), disable only that control, and drive the §3.4 labels. Do not optimistic-update the cell. Clear the key on success, failure, and cancel.
- Add inline confirmation for remove and revoke if required; the confirm button uses Danger, the cancel Secondary, and the panel is not focus-trapped. Re-enable is direct. The server's last-Administrator refusal remains the authority; never pre-disable based on a local count.
- Keep the `refresh` counter and the two existing API calls. Do not add an endpoint, pagination, search, role creation, invite, device purge, or session editor.
- Translate the shipped English server messages at their source (`auth_server`/`service`), not in this page: `The clinic needs an administrator.`, `Unknown role.`, `Profile not found.`, `Member not found.`, `Solo el administrador puede gestionar usuarios y roles.` belongs to this page and is already Spanish. The page prints whatever arrives.

Verification commands, from the repo root; none is recorded here as executed:

```sh
cd persona-sanus && cargo check
dx build --release --fullstack
caveman-blocks run persona-smoke --dir persona-sanus --port 8091
python3 .blocks/markdown-table-check.py --path context/screens/SettingsUsers.md --strict
```

The smoke script covers auth, not this admin table. A browser pass needs: Administrator with at least two profiles and two devices; Doctor denied; role change; disable/enable; remove with confirm; revoke; last-Administrator refusal (set a clinic with one admin); unknown role through a stale client (or service test); 320px stacked rows.

- Do not add: an invite, create-user, delete-profile, hard-delete, password reset, MFA reset, session editor, or email-resend control or endpoint.
- Do not add: billing, price, trial, plan, `sync_locked`, entitlement, or image-quota copy (D-04).
- Do not add: patient PHI, a patient picker, a prescription/template list, or clinical content beyond the staff identity columns.
- Do not add: a signature/stamp image, patient photo, avatar, platform logo, chart, or decorative icon (D-06).
- Do not add: rich text, `contenteditable`, HTML/markdown cell rendering, or `innerHTML` (D-21).
- Do not add: a client-side last-Administrator count that disables controls, a client-only self-protection block, or a role source other than `clinic_members.role` (D-03).
- Do not add: a second Primary, a filled Danger action, a modal, a browser confirm/prompt, or a toast that hides the server message.
- Do not add: pagination, search, sort, bulk role change, or select-all.
- Do not add: a new route, endpoint, table, column, role, permission, query parameter, or API argument.
- Do not add: a role value not in `administrator|doctor|nursing|secretary`, or a blank role submitted as a change.

## 8. Spec references

| Spec | Connection |
|---|---|
| `specs/15-O-ui-tokens-actions-and-formatting.md` | `15-O` supplies the tokens, `Button`, and format helpers (`format_date`, `role_label`) the table renders. |
| `specs/16-P-ui-forms-feedback-and-layout.md` | `16-P` supplies `DataTable`, `SelectField`, and the danger/secondary action buttons the roster uses. |
| `specs/18-R-ui-domain-and-billing.md` | `18-R` builds `AccountStatusBadge`, the account status cell each row shows. |
| `specs/24-X-ui-settings-detail-and-audit.md` | `24-X` adopts `SettingsUsers`: the staff `DataTable`, role change, disable/remove, and device revocation actions. |
| `specs/25-Y-ui-legacy-css-cleanup-and-docs.md` | `25-Y` removes legacy page-ID selectors so the adopted table runs only on `.ui-*` classes. |
