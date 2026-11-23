# PersonaSanus — SettingsProfile (`/settings/profile`)

| Field | Value |
|---|---|
| Source | `context/sitemap.md` §1.2 (`/settings/profile` — My profile, all staff, doctor details, no signature/stamp asset), §2 (profile route for every role), §4 M0/M3; `context/PRD.md` §3 roles, §6 `profiles`/`doctors`, §9 MFA and sessions, D-03, D-06, D-21; `specs/done/02-B-profiles-clinics-and-roles.md`, `specs/done/12-L-hardening.md`; `context/Design.md` (Card, Section Card, Form Input, Primary/Secondary/Ghost/Danger Button, Tokens, Layout, Elevation); `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` step 16, Page adoption row `SettingsProfile` (line 234), `PageShell`, `PageHeader`, `SectionCard`, `FormGrid`, `TextField`, `Button`, `ActionRow`, `HintText`, `ErrorText`; `context/screens/INDEX.md` §2.3 |
| Component | `SettingsProfile` — `persona-sanus/src/main.rs:2318-2632` (`#[component]` at `:2318`, `fn` at `:2319`). Reads `actor: Signal<Option<Me>>` and `checked: Signal<bool>`; holds `initialized`, `display_name`, `specialty`, `license_no`, `national_id`, `phone`, `email`, `message`, `saving`, `mfa_enabled`, `mfa_secret`, `mfa_uri`, `mfa_code`, `mfa_message`, `mfa_password`, `current_password`, `new_password`, `password_message`. Target composes planned `src/ui/` primitives; that directory is not shipped |
| Route | `/settings/profile` (`Route::SettingsProfile {}`, `main.rs:70-71`) — authenticated shell, every staff role. No session replaces `/login`; a checked session with no profile is a redirect state. No path or query parameter |
| Layout | Inside `#[layout(Navbar)]` (`main.rs:51`, never closed). `PageShell id="settings-page"` → `div#settings-page.ui-page > .ui-container.ui-stack`; Paper canvas, 48px top / 80px bottom / 24px inline, 24px stack gap; `Datos del perfil` card, then `div.ui-columns` with `Verificación en dos pasos` and `Cambiar contraseña` cards |
| Locale | `es-EC` on every string. `perfil`, `contraseña`, `verificación`, `cédula`, `licencia`, `clínica`. Dates are not rendered; no time, money, or image copy |
| Status | ✅ Spec complete (sections 1–7). Route status: ✅ — sitemap §1.2: display name, specialty, license, cédula, phone/email plus TOTP enrolment and password change. Shipped: step-16 shell adoption with `SectionCard` + `FormGrid` + field primitives; profile fields, TOTP enrolment/confirm/disable, password change; no new auth method, no signature/stamp asset |

## 1. Purpose

The signed-in staff member's own account and clinical identity. Four jobs, in order:

1. **Show the account.** Display the editable display name and the read-only clinic/role context from `Me`.
2. **Complete the doctor identity.** For a Doctor, collect specialty, professional license number, cédula, phone, and contact email. These are the values templates print and prescriptions carry.
3. **Protect the account.** Enrol or disable one TOTP factor; the server gates Doctor and Administrator sign-in when it is enabled.
4. **Rotate the password.** Change the current password with a new one; the server ends every other session and issues the current client a fresh one.

It never renders:

- Patient PHI. No patient name, cédula, diagnosis, medication, prescription, image, or record is fetched or painted. The only cédula here is the staff member's own professional identity, an account field.
- A signature or stamp image, upload slot, preview, or "digital signature" copy (D-06). The doctor signs the printed paper.
- Billing, price, trial, plan, entitlement, `sync_locked`, or subscription copy (D-04). Account settings are not billing.
- A rich-text editor, formatting toolbar, `contenteditable`, HTML field, or markdown renderer (D-21). Every value is a plain input.
- A role selector, clinic picker, staff table, device list, audit row, patient list, prescription list, or template list exception. Those live on their own settings surfaces.
- A password reset, forgot-password link, email-verification resend, recovery code, session list, or “remember this device” control.
- A second filled Primary. `Guardar` is the only Primary; MFA and password actions are Secondary, disable is Danger.
- An MFA secret in a URL, `title`, analytics event, log line, or pre-filled after reload. The secret/URI are shown once in the page body and never persisted by the client.

## 2. Structure

One centered page column, Paper. Page header, profile card, two MFA/password cards side by side on desktop, stacked below 760px.

| Order | Region | `id` / class | Band tone | Contents |
|---|---|---|---|---|
| — | Navbar (layout) | `header.ui-navbar` → `.ui-navbar-inner` | — | Owned by `context/screens/done/Header.md` §2–§3; not restated |
| 1 | Page canvas | `PageShell id="settings-page"` → `div#settings-page.ui-page > .ui-container.ui-stack` | Paper `#ffffff` | 48px top, 80px bottom, 24px inline; 1200px container; 24px stack gap |
| 2 | Page header | `header.ui-page-head` · `h1.ui-page-title` · `p.ui-page-subtitle` | Paper | `Mi perfil`; `Clínica: {name} · Rol: {role}` or `Sin clínica asignada. Un administrador debe asignarte un rol.`; no actions |
| 3 | Profile card | `SectionCard` `Datos del perfil` · `Card` + `FormGrid` | Paper, hairline | `Nombre visible` always; doctor fields `Especialidad`, `Número de licencia`, `Cédula`, `Teléfono`, `Correo`; Primary `Guardar`; page-level `ErrorText`/`HintText` |
| 4 | MFA card | `SectionCard` `Verificación en dos pasos` | Paper, hairline | Status line; when off: Secondary `Activar verificación`; when enrolling: secret/URI text, `Código de 6 dígitos`, Secondary `Confirmar activación`; when on: password field and Danger `Desactivar verificación`; hint/error line |
| 5 | Password card | `SectionCard` `Cambiar contraseña` | Paper, hairline | `Contraseña actual`, `Nueva contraseña`, Secondary `Cambiar contraseña`; hint/error line |
| 6 | Columns wrapper | `div.ui-columns` | Paper | Holds regions 4 and 5; two columns above 760px, one below |
| 7 | Gates | `LoadingState` | Paper | `Cargando...` before the session check; `Redirigiendo...` when checked without a profile |

Region 4 renders only when the actor is a Doctor or an Administrator; nursing and secretaries still see their profile and password cards, but no MFA card. Region 5 renders for every staff role.

### 2.1 Role and field matrix

| Role | Nombre visible | Especialidad | Número de licencia | Cédula | Teléfono | Correo | MFA card | Password card |
|---|---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| Administrator | ✓ | — | — | — | — | — | ✓ | ✓ |
| Doctor | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Nursing | ✓ | — | — | — | — | — | — | ✓ |
| Secretary | ✓ | — | — | — | — | — | — | ✓ |

The server is the role authority (`clinic_members.role`, D-03). The client hides fields by `is_doctor()` and the MFA card by `is_doctor() || is_administrator()`, but `update_profile` and the MFA endpoints must enforce the same rule; a hidden field is not a trusted write. The page never lets a non-Doctor promote themselves to Doctor or set a license/national id.

### 2.2 Profile load and save

The effect runs once (`initialized` guard) after `checked`; a missing session replaces `/login`. It sets `display_name` from `profile.display_name`; when `Me.doctor` is present it also sets `specialty`, `license_no`, `national_id`, `phone`, and `email` from the doctor record (each empty when absent). It then asks `auth::mfa_status()` once and sets `mfa_enabled` on success; a status failure is not a page error and leaves the toggle off until the next read.

`Guardar` sends `update_profile(display_name, specialty, license_no, national_id, phone, email)` — all six strings on every role. The server ignores the five doctor-only fields for a non-Doctor (verify in the authorized service); the page does not send a second role-specific shape. On success it replaces `actor` with the returned `Me` and shows `Perfil guardado.`; on failure it shows the returned message and keeps every typed value. There is no offline branch: a desktop without the server fails the same as web.

Specialty, license number, and cédula are professional identity, not patient PHI. They still never enter the URL, page title, analytics, or a log line. The page shows no clinic id, profile id, or auth user id.

### 2.3 TOTP enrolment and disable

The MFA card is a state machine, not a free-form form:

1. **Off.** `Sin activar. Actívala para proteger tu cuenta.` + Secondary `Activar verificación`.
2. **Enrolling.** `auth::enroll_mfa()` returns `secret` and `uri`; both render as selectable text in a `.ui-break` block, never as a QR image. The hint explains scanning the URI or copying the secret. One `Código de 6 dígitos` field and Secondary `Confirmar activación` follow. The page clears `mfa_secret`/`mfa_uri` and hides the code field after a successful confirm.
3. **On.** `Activada. Se pedirá el código al iniciar sesión.` + `Contraseña para desactivar` + Danger `Desactivar verificación`.
4. **Done.** `Verificación activada.` / `Verificación desactivada.` hint with the same state as above.

`confirm_mfa(code)` accepts the current 6-digit TOTP; empty input shows a local hint and sends nothing. `disable_mfa(password)` requires the current password; an empty value sends nothing. A failed confirm keeps the secret/URI and code field so the operator can retry. A failed disable keeps the password field; do not reveal whether the password or the code failed. The secret and URI are bearer secrets: no copy of them in localStorage, sessionStorage, cookies, analytics, or logs; a page reload before confirm re-enrols and discards the old pending secret.

The page does not claim MFA is enforced for every role. It is enforced at sign-in for Doctor and Administrator when enabled; Nursing and Secretary do not get the card and cannot enrol from here.

### 2.4 Password change

Two masked fields: `Contraseña actual`, `Nueva contraseña`; Secondary `Cambiar contraseña`. The client checks only non-empty; the server enforces the eight-character minimum and current-password match. On success the server deletes every session, inserts a new one for this client, and `auth::change_password` returns it; the page clears both fields and shows `Contraseña cambiada. Sesión renovada.` On failure it keeps both fields and shows the returned message. No password is ever echoed, trimmed, logged, or placed in a URL; no show-password toggle ships.

### 2.5 Dynamic states

| State | Screen behaviour |
|---|---|
| Session check in flight | `PageShell` + `LoadingState` `Cargando...`; no cards or fetch |
| Checked, no session | Replace `/login`; no card |
| Checked, no profile | `Redirigiendo...`; no card |
| Profile load, Doctor | Profile card with six fields; MFA card; password card |
| Profile load, Administrator | Profile card with display name only; MFA card; password card |
| Profile load, Nursing/Secretary | Profile card with display name only; password card; no MFA |
| MFA status in flight | MFA card may paint with the last known state; no false “Activada” before the read resolves. A failed read leaves the off state |
| Save idle/typed | Primary `Guardar` enabled; doctor fields editable |
| Save busy | Primary `Guardando...`, disabled, `aria-busy`; fields keep values; one PUT |
| Save success | `Perfil guardado.`; `actor` updated; values remain |
| Save failure | `ErrorText` with server message; typed values retain; no optimistic context update |
| MFA off | Off copy + `Activar verificación` |
| MFA enrolling | Secret and URI selectable text; code field; `Confirmar activación`; `Activar verificación` gone |
| MFA confirm busy | Confirm disabled; one request; no double factor |
| MFA confirm success | `Verificación activada.`; secret/URI cleared; toggle on; code field gone |
| MFA confirm failure | Returned message in `ErrorText`; secret/URI/code stay; retry allowed |
| MFA on | On copy; password field for disable; Danger `Desactivar verificación` |
| MFA disable busy | Disable disabled; one request |
| MFA disable success | `Verificación desactivada.`; toggle off; password field cleared |
| MFA disable failure | Returned message; password field stays; toggle still on |
| Password idle/typed | `Cambiar contraseña` enabled when both fields non-empty |
| Password busy | Action disabled; one request |
| Password success | `Contraseña cambiada. Sesión renovada.`; both fields cleared; other sessions ended server-side |
| Password failure | Returned message; both fields retain; no session claim |
| Desktop offline | Profile save and both password/MFA actions fail at the transport; no outbox, no local auth cache. The shipped signed-in context may still paint from a restored session |
| Narrow viewport | Fields one column; MFA/password cards stack; secret/URI wrap; 44px targets; no horizontal page scroll |
| Reduced motion | No animation; only any global button transition suppressed |

## 3. Copy text

All strings verbatim, `es-EC`; punctuation belongs to the copy. Navbar copy is owned by `Header.md` §3 and not repeated.

### 3.1 Page and profile

| Element | Copy |
|---|---|
| Page title (`h1`) | `Mi perfil` |
| Subtitle, clinic and role | `Clínica: {clinic} · Rol: {role}` |
| Subtitle, no clinic | `Sin clínica asignada. Un administrador debe asignarte un rol.` |
| Profile card heading (`h2`) | `Datos del perfil` |
| Display name label | `Nombre visible` |
| Specialty label | `Especialidad` |
| License label | `Número de licencia` |
| Cédula label | `Cédula` |
| Phone label | `Teléfono` |
| Email label | `Correo` |
| Primary | `Guardar` |
| Busy | `Guardando...` |
| Success | `Perfil guardado.` |
| Failure | returned server message, unchanged |

### 3.2 MFA card

| Element | Copy |
|---|---|
| Card heading (`h2`) | `Verificación en dos pasos` |
| Off copy | `Sin activar. Actívala para proteger tu cuenta.` |
| Activate action, Secondary | `Activar verificación` |
| Enrolling hint | `Escanea el URI o copia el secreto en tu app de autenticación.` |
| Secret line | `Secreto: {secret}` |
| URI line | `URI: {uri}` |
| Code label | `Código de 6 dígitos` |
| Confirm action, Secondary | `Confirmar activación` |
| On copy | `Activada. Se pedirá el código al iniciar sesión.` |
| Disable action, Danger | `Desactivar verificación` |
| Disable password label | `Contraseña para desactivar` |
| Activated success | `Verificación activada.` |
| Disabled success | `Verificación desactivada.` |
| Failure | returned server message, unchanged |

### 3.3 Password card

| Element | Copy |
|---|---|
| Card heading (`h2`) | `Cambiar contraseña` |
| Explanation | `Al cambiarla se invalidan las demás sesiones.` |
| Current label | `Contraseña actual` |
| New label | `Nueva contraseña` |
| Action, Secondary | `Cambiar contraseña` |
| Busy | `Cambiando...` |
| Success | `Contraseña cambiada. Sesión renovada.` |
| Failure | returned server message, unchanged |

### 3.4 Gates

| Element | Copy |
|---|---|
| Session gate | `Cargando...` |
| Redirect gate | `Redirigiendo...` |

## 4. Displayed information

### 4.1 What the screen shows

| Element | Source | Value shown |
|---|---|---|
| Page title/subtitle | `actor: Signal<Option<Me>>` → `Me.clinic.name`, `Me.role` through `role_label` | `Mi perfil`; `Clínica: … · Rol: …` or the no-clinic sentence |
| Display name | `Me.profile.display_name`; local `display_name` | editable plain text |
| Specialty | `Me.doctor.specialty`; local `specialty` | editable plain text, Doctor only |
| License | `Me.doctor.license_no`; local `license_no` | editable plain text, Doctor only |
| Cédula | `Me.doctor.national_id`; local `national_id` | editable plain text, Doctor only; account identity, not patient PHI |
| Phone | `Me.doctor.phone`; local `phone` | editable plain text, Doctor only |
| Email | `Me.doctor.email`; local `email` | editable plain text, Doctor only; account contact, not patient contact |
| Save | `api::update_profile(display_name, specialty, license_no, national_id, phone, email)` | one request, all six strings; updated `Me` on success |
| MFA state | `auth::mfa_status()` | enabled/disabled; off by default |
| Enrolment | `auth::enroll_mfa()` → `secret`, `uri` | selectable text; no QR image; no persistence |
| Confirm | `auth::confirm_mfa(code)` | enables TOTP; clears pending secret/URI on success |
| Disable | `auth::disable_mfa(password)` | requires the current password; turns TOTP off |
| Password | `auth::change_password(current, new)` | server minimum 8; deletes other sessions; returns a fresh session |

The page never fetches patients, prescriptions, templates, assets, users, devices, audit rows, billing, or a session list. It reads only the shell `Me` context and the two auth helpers.

### 4.2 Hard rules

- Server-side authorization is the boundary. `AuthPage` proves identity; every auth helper re-checks the current session and actor on the server. The hidden MFA card for nursing/secretary is UX, not permission.
- Role authority is `clinic_members.role` only (D-03). This page never edits a role, a clinic membership, or another profile. It edits the signed-in actor's own profile fields and credentials.
- Doctor-only fields are not silently writable by a non-Doctor. `Especialidad`, `Número de licencia`, `Cédula`, `Teléfono`, and `Correo` are omitted for Administrator/Nursing/Secretary; the server must reject or ignore attempts to set them from those roles. Do not render disabled doctor fields as decoration.
- The TOTP secret and URI are bearer secrets. Show them once, in the page body, as text; never a QR code, image URL, `alt`, analytics event, log line, `localStorage`, `sessionStorage`, cookie, or prefilled value. Clear them after confirm and on unmount where the runtime allows.
- MFA disable requires the current password; never a code-only or one-click disable. A failed password keeps the factor on.
- Password change requires the current password and a new value of at least eight characters (server). It ends every other session; the current client receives a new session. The page does not promise “same session” or “all sessions unchanged.”
- No password reset, forgot-password, email verification, recovery code, backup-code, or “remember me” control exists on this page. Support flows are out of scope and must not be invented.
- No signature or stamp asset, field, upload, preview, or “digital signature” copy (D-06). No patient photo.
- No billing/price/trial/plan/`sync_locked` copy or state (D-04). No image quota here; that is `/settings/clinic`.
- No rich text, `contenteditable`, HTML payload, or markdown renderer (D-21). All values are plain inputs.
- No PHI in the URL, title, analytics, or logs. The route is exactly `/settings/profile`; no email, phone, cédula, or license is in the path/query.
- One Primary per viewport: `Guardar`. `Activar verificación`, `Confirmar activación`, and `Cambiar contraseña` are Secondary; `Desactivar verificación` is Danger. No second filled teal action.
- No new endpoint, route, table, column, role, permission, or auth method. The page uses `update_profile`, `mfa_status`, `enroll_mfa`, `confirm_mfa`, `disable_mfa`, `change_password`.

### 4.3 Surface notes

- **Web:** same fields and calls; the session refresh uses the returned cookie. The page never writes tokens to storage.
- **Desktop:** `persist_local_session` may be updated by the auth transport after a password change; the page itself does not write the file. A transport failure is shown as text terminal. No offline update branch exists.
- **Clinic absent or role absent:** the subtitle states it; the profile card still edits the display name, and password change still works. MFA visibility follows `is_doctor() || is_administrator()`, so a role-less profile does not get the card.
- **Long secret/URI:** wrap with `overflow-wrap:anywhere` and `word-break:break-word` inside `.ui-break`; no horizontal page scroll.
- **Zero MFA status:** until `mfa_status` returns, do not claim “Activada”; show the off copy or a neutral hint. A status failure is not disguised as off: the target leaves the toggle in its disabled rendering, adds the neutral hint `No se pudo comprobar el estado.`, and never claims enabled. If that hint is not shipped, use the off copy without the “Activada” claim.
- **After enable:** the next sign-in on this account requires a TOTP code; the current session stays valid. The page never auto-signs-out after enrolment.
- **After disable:** the next sign-in stops requiring a code; existing sessions are not retroactively revoked by this toggle unless the auth server does so. State only what the endpoint does.

## 5. Style

Every value is a `context/Design.md` token declared in `persona-sanus/assets/main.css` `:root`. No new color, font, icon package, CDN, or webfont.

### 5.1 Color

| Use | Value | Token |
|---|---|---|
| Canvas, cards, inputs | `#ffffff` | `--color-paper` |
| Titles, labels, values | `#061d29` | `--color-ink` |
| Subtitle, hints, meta, Secondary border/text | `#425d6d` | `--color-mist` |
| Hairlines | `#e5e7eb` | `--color-hairline` |
| Primary fill, focus ring, check accent | `#006e75` | `--color-deep-teal` |
| Primary hover | `#0b978e` | `--color-bright-teal` |
| Secret/URI wash, pending, code cell | `#fffded` | `--color-cream` |
| Danger text, Alert glyph | `#4d0037` | `--color-plum` |
| Danger hover wash, error surface | `#ffc3e6` | `--color-blossom` |
| Focus halo | `rgba(185,255,232,0.55)` | `--color-mint-glass` |

Never `#000000`. Never a chromatic card border; hairlines only. Danger is a text role unless a destructive confirm needs a Blossom wash. Measured contrast: Ink/Paper 17.28 · Mist/Paper 6.96 · Paper/Deep Teal 6.02 · Ink/Cream 16.89 · Mist/Cream 6.80 · Plum/Paper 15.32 · Plum/Blossom 10.36. AA for body.

### 5.2 Typography

| Element | Family | Weight | Size / line-height |
|---|---|---|---|
| Page title (`h1`) | `--font-mint` | 500 | 22px / 1.22 Ink |
| Page subtitle | `--font-mint` | 400 | 14px / 1.5 Mist |
| Section heading (`h2`) | `--font-mint` | 500 | 20px / 1.25 Ink |
| Field label | `--font-mint` | 500 | 14px / 1.5 Ink |
| Input value | `--font-mint` | 400 | 16px / 1.5 Ink |
| Hint / status copy | `--font-mint` | 400 | 14px / 1.5 Mist |
| Secret / URI | `--font-mint` | 400 | 14px / 1.5 Ink; `overflow-wrap:anywhere` |
| Primary / Secondary / Danger label | `--font-mint` | 500 | 16px / 1.22 |
| Error text | `--font-mint` | 400 | 14px / 1.5 Plum |

No `--font-grenette`; application chrome. Secret/URI never shrink below 14px; long tokens wrap rather than truncate.

### 5.3 Geometry

- 4px base. `PageShell`: 48px top, 80px bottom, 24px inline; container max 1200px; stack gap 24px.
- `PageHeader`: stack, 4px title/subtitle gap; no actions.
- Profile `SectionCard`: 24px radius, 1px hairline, 24px padding. `FormGrid` 2 columns, 16px gap; one column below 760px. Fields: display name, specialty, license, cédula, phone, email. All normal width except a long email may span at the grid's discretion; do not fake a wide field.
- `ui-columns`: two equal columns, 24px gap above 760px; one column below. MFA first, password second in DOM order.
- MFA/password sections: 24px radius, 24px padding, 16px between blocks. Secret/URI text block full width, 8px radius, Cream wash, 12px padding. Code/password inputs 44px min-height, 8px radius, `8px 12px`.
- Action rows: 12px gap, wrap, 16px top margin. Primary is content-width.
- Empty/status: no empty card; the MFA off state is a sentence, not an illustration.
- Elevation flat. No shadow, no sticky save bar, no divider heavier than the hairline.
- 320px: one column, labels above values, buttons wrap; secret/URI wrap; no horizontal page scroll.

### 5.4 Iconography and imagery

Line icons only, 1.5px stroke, 20px, Ink or Plum: `Alert` in errors; `Check` may mark a verified MFA state, `Lock` may sit in the MFA heading if the shared primitive supplies it, but neither is required. The page does not render a QR code, a scannable barcode, a signature, a stamp, an avatar, a clinic logo, a patient image, or an illustration. The secret/URI are text, not images; do not turn them into a QR at render time. The Navbar owns the brand mark.

## 6. Accessibility

- Landmarks: navbar banner (`Header.md` §6); one content region. The profile card and the two settings cards are labelled `section`s with `h2`s: `Datos del perfil`, `Verificación en dos pasos`, `Cambiar contraseña`. `ui-columns` is presentational; it changes visual columns, not heading order or DOM order.
- Headings: one `h1`, `Mi perfil`; three `h2` in DOM order. No `h3` needed. Visual reordering must not change DOM order.
- Tab order: Nombre visible → Especialidad → Número de licencia → Cédula (input `type="text"` unless a tested numeric input; no forced numeric masking) → Teléfono (`tel`) → Correo (`email`) → Guardar → Activar verificación / Código de 6 dígitos → Confirmar activación → Contraseña para desactivar → Desactivar verificación → Contraseña actual → Nueva contraseña → Cambiar contraseña. Disabled/hidden branches are skipped; no hidden field remains focusable.
- Focus: `outline: 1px solid var(--color-deep-teal); outline-offset: 2px; box-shadow: 0 0 0 3px rgba(185,255,232,0.55);` on every input and button. Visible on Paper and Cream; never removed.
- Labels: every input is wrapped in its visible label; passwords are `type="password"` with `autocomplete="current-password"` / `"new-password"`. The MFA code uses `autocomplete="one-time-code"` and `inputmode="numeric"`; it is not a password field, so the code stays readable. Secret/URI are selectable text; if a copy affordance is ever added, it must be a real button with an accessible name, not a clickable div.
- Errors: `ErrorText` is `role="alert"` and tied to the card/action that failed with `aria-describedby` where possible. Success messages are `role="status"`/polite. A failed save does not clear fields; a failed password change retains both; a failed MFA confirm retains the secret.
- Busy: the acting button sets `aria-busy="true"` and `disabled`, with the visible `Guardando...`/`Cambiando...` label. Focus does not move to a loading state.
- Status: MFA copy is words, not a coloured toggle alone. The off/on state is in text; a switch, if added, needs `aria-checked` and a label. The secret is never hidden only in a placeholder.
- Security: no `autocomplete="off"` on password fields (password managers are allowed); no password in a `title`; no secret in a URL; no copy-on-focus that silently overwrites the clipboard.
- Contrast: Ink/Paper 17.28 · Mist/Paper 6.96 · Paper/Deep Teal 6.02 · Ink/Cream 16.89 · Plum/Paper 15.32. AA for body; focus ring ≥3:1.
- Keyboard/touch: native inputs/buttons; 44px targets; no drag, no hover-only reveal, no modal, no focus trap.
- Reduced motion: no animation; only a global button colour transition may be suppressed.
- Zoom/reflow: 200% and 320px stack the columns, preserve DOM order, and wrap the secret/URI without clipping.

## 7. Implementation notes

- Shipped component: `fn SettingsProfile()` at `persona-sanus/src/main.rs:2926` (`#[component]` at `:2925`, closing before `Audit` at `:3273`). Reads `actor`/`checked`; signals `initialized`, `display_name`, `specialty`, `license_no`, `national_id`, `phone`, `email`, `message` (+`message_error`), `saving`, `mfa_enabled`, `mfa_secret`, `mfa_uri`, `mfa_code`, `mfa_message` (+`mfa_message_error`), `mfa_password`, `mfa_busy`, `current_password`, `new_password`, `password_message` (+`password_message_error`), `password_busy`.
- Load effect (`main.rs:2940-2980`): once (`initialized` guard) after `checked`; no session → `navigator.replace(Route::AuthPage {})`; seeds `display_name` from `Me.profile.display_name` and, for `Me.doctor`, specialty/license/national id/phone/email; calls `auth::mfa_status()` once and stores the boolean (failure swallowed, toggle stays off). Gates render `PageShell id="settings-page"` + `LoadingState` (`Cargando...`) / `LoadingState` `Redirigiendo...` (checked without profile).
- Render (`main.rs:2982-3272`): `PageShell` + `PageHeader` (`Mi perfil`, `Clínica: {name} · Rol: {role}` or `Sin clínica asignada. Un administrador debe asignarte un rol.` at `:3000-3001`); `SectionCard` `Datos del perfil` (`FormGrid columns=Two`, `TextField`s `Nombre visible`, and for Doctor only `Especialidad`/`Número de licencia`/`Cédula`/`Teléfono`/`Correo` (`input_type="email"`), second blank `FormGrid` row for non-Doctor); per-card `ErrorText`/`HintText` split on `message_error`; `ActionRow` Primary `Guardar`/`Guardando...` (`:3086`) via `api::update_profile` with all six strings, success `Perfil guardado.` + `actor` replaced, failure keeps values.
- MFA card (`Verificación en dos pasos`, only `is_doctor() || is_administrator()`): off copy + Secondary `Activar verificación` → `auth::enroll_mfa()` → `Secreto: {secret}` / `URI: {uri}` in `p.ui-break` (selectable text, never QR) + hint `Escanea el URI o copia el secreto en tu app de autenticación.` + `Código de 6 dígitos` `TextField` (`autocomplete="one-time-code"`, `inputmode="numeric"` at `:3107-3108`, `ui/form.rs:54,74` carries `inputmode`) + Secondary `Confirmar activación`/`Confirmando...` (`mfa_busy`): confirm disabled when `mfa_code().trim().is_empty()` (empty guard shipped), success clears secret/URI/code and sets `Verificación activada.`, failure keeps pending values. On: `Contraseña para desactivar` + Danger `Desactivar verificación`/`Desactivando...`, disabled when password empty (empty guard shipped), success `Verificación desactivada.`, failure keeps password. Per-card message split `ErrorText`/`HintText` on `mfa_message_error`.
- Password card (`Cambiar contraseña`): `HintText` `Al cambiarla se invalidan las demás sesiones.`, `FormGrid columns=Two` with `Contraseña actual`/`Nueva contraseña` (password type/autocomplete), Secondary `Cambiar contraseña` disabled when either field empty or `password_busy` (empty guard shipped), `auth::change_password` success clears both fields and sets `Contraseña cambiada. Sesión renovada.`, failure keeps both. Messages split `ErrorText`/`HintText` on `password_message_error`; `ui-columns` holds the MFA and password `SectionCard`s.
- Verification commands, from the repo root; none is recorded here as executed:

```sh
cd persona-sanus && cargo check
dx build --release --fullstack
caveman-blocks run persona-smoke --dir persona-sanus --port 8091
python3 .blocks/markdown-table-check.py --path context/screens/done/SettingsProfile.md --strict
```

The smoke script covers sign-in/MFA at a high level, not the profile card. A browser pass needs: Doctor profile save and reload; Administrator view (no doctor fields, MFA card, password card); Nursing view (no MFA card); MFA enrol with a real TOTP app, reload before confirm, confirm, sign out and sign in with code; disable with wrong and right password; password change with wrong current, short new, and valid new, then prove another session was ended; 320px and 200% zoom.

- Do not add: a signature/stamp field, upload, preview, placeholder, or “digital signature” copy (D-06).
- Do not add: a patient field, patient picker, or patient PHI; this is a staff account screen.
- Do not add: a role selector, clinic picker, staff table, device list, audit row, patient/prescription/template list, or image library (those belong to their settings screens).
- Do not add: billing, price, trial, plan, entitlement, `sync_locked`, or subscription copy (D-04).
- Do not add: password reset, forgot-password, email verification, recovery codes, backup codes, session list, or “remember this device” controls.
- Do not add: a QR image, QR library, scannable secret, secret in a URL/`title`/analytics/log, or secret in browser storage.
- Do not add: rich text, WYSIWYG, `contenteditable`, HTML/markdown rendering, or a formatting toolbar (D-21).
- Do not add: a second Primary, a filled Danger button, a modal, a toast, or an autosave.
- Do not add: a new route, endpoint, table, column, role, permission, or API argument.
- Do not add: a neutral MFA-status-failure hint claiming `Activada`, a doctor field for a non-Doctor, or a length/strength meter client-side (server enforces).

## 8. Spec references

| Spec | Connection |
|---|---|
| `specs/done/15-O-ui-tokens-actions-and-formatting.md` | `15-O` supplies the tokens, `Button`, and format helpers the profile page renders. |
| `specs/done/16-P-ui-forms-feedback-and-layout.md` | `16-P` supplies `PageShell`, `PageHeader`, `SectionCard`, `FormGrid`, `TextField`, `ErrorText`, `HintText`, and `ActionRow` the data, MFA, and password cards use. |
| `specs/done/24-X-ui-settings-detail-and-audit.md` | `24-X` adopts `SettingsProfile`: profile fields, TOTP enrolment, password change, one Primary `Guardar`. |
| `specs/done/25-Y-ui-legacy-css-cleanup-and-docs.md` | `25-Y` removes legacy page-ID selectors so the adopted page runs only on `.ui-*` classes. |
