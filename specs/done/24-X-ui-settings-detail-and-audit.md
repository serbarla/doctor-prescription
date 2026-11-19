# Spec 24 — Phase X: Clinic, users, profile, and audit adoption

- Source: `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` step 16 and the Page adoption table; `context/sitemap.md` §§1.1–1.3, §2; `context/PRD.md` FR-080–085, FR-042; `context/database-design.md` §4; `context/Design.md` §§Components, Tokens, Surfaces, Layout.
- Depends on: 15-O, 16-P, 17-Q, 18-R, 19-S, 20-T.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Result carried forward: `SettingsClinic`, `SettingsUsers`, `SettingsProfile`, and `Audit` render through the UI layer; clinic upload/delete and MFA/password calls are unchanged; the clinic page reads sync state and disables both upload fields offline; each page exposes exactly one filled primary where section 8 names one, and none otherwise.

## Before you start

Unit testing is not required; avoid it. Prioritize end-to-end and integration testing only.

Read the nearest `AGENTS.md` (target workspace, then parents up to repo root) before executing.

If reality diverges from this spec: STOP, append a divergence (section 4), and NEVER improvise outside the objectives.

This is markup adoption plus UI-only closure extraction. No page gains an endpoint, permission, editable field, role rule, or new data shape. Keep every `api::` call and argument. Keep every `auth::` MFA/password call. Keep every desktop fallback and refresh counter.

## 1. Objectives

*The ordered work to implement — one item per step. Check `[x]` only after the step is implemented and verified.*

*Item format:* `<verb phrase>` — requirements: `<what must change / constraints>`; commands: `<exact command>`.

- [x] Adopt SettingsClinic — requirements: use `PageShell id="settings-page"` and `PageHeader title="Clínica" subtitle="Datos de la clínica, logo y biblioteca de imágenes."` with a Secondary `Volver` header action to `Route::Settings {}`. Read `Signal<SyncStatus>` from context and compute `online = sync().online`. Render `ErrorText` for the error signal. Replace the four raw sections with `SectionCard`s: `Datos de la clínica` (`FormGrid` with `TextField` Nombre, Dirección, Teléfono, and `TextField` `Plantilla del enlace LLM` with the existing placeholder, wide; `ActionRow` Primary `Guardar datos`), `Logo` (optional `img.ui-asset-preview` from `logo.url` with alt `Logo de la clínica`; `HintText` `La clínica todavía no tiene logo.` when absent; `FileField` label `Subir logo (JPEG o PNG, máximo 10 MB)`, accept `image/jpeg,image/png`, disabled when `!online || uploading()`), `Biblioteca de imágenes` (`ImageLibrary` with `library.assets`, `library.quota`, `uploading`, `deleting`, `online`, and named upload/delete callbacks; when `library` is `None`, pass 
  empty assets and `used = 0`, `limit = 0`), and `Exportación de datos` (retention hint `Retención en servidor hasta: {value}` when present, grace hint `Fin del período de gracia para exportación: {value}` when present, the existing explanation, Secondary `LinkAction` href `/api/clinic/export` label `Exportar datos (JSON)`). Extract the existing logo-upload, image-upload, and delete bodies into named closures without changing `api::upload_logo`, `api::upload_asset`, `api::delete_asset`, `http::is_server_reachable`, generation increments, or message text. `ImageLibrary` forwards the whole event to `on_upload`; the page's upload callback selects the first file with `event.files().first()`. Only Primary: `Guardar datos`; commands: `cd persona-sanus && cargo check`; `rg -n 'Signal<SyncStatus>|ImageLibrary|upload_logo|upload_asset|delete_asset|ui-asset-preview' persona-sanus/src/main.rs`.
- [x] Adopt SettingsUsers — requirements: use `PageShell id="settings-page"` and `PageHeader title="Usuarios y roles" subtitle="Asigna un rol o deshabilita el acceso de cada persona."`. Render `ErrorText` for the error signal. `SectionCard title="Personal"` contains a `DataTable` with columns `Persona`, `Correo`, `Alta`, `Estado`, `Rol`, `Acciones`; each row renders `display_name`, `email`, `format_date(created_at)`, `AccountStatusBadge { status }`, `SelectField label="Rol" hide_label` bound to `role` with the existing options (`Sin rol`, `Administrador`, `Doctor`, `Enfermería`, `Secretaría`), and actions in this order: Danger `Quitar del equipo` when a role exists (existing `remove_member` closure), Danger `Deshabilitar` when active or Secondary `Habilitar` when disabled (existing `disable_profile` closure). When there are no users, render `EmptyState title="Sin usuarios" description="No hay usuarios para mostrar."` inside the Personal card. `SectionCard title="Dispositivos" description="Equipos registrados y su último acceso."` contains a `DataTable` with columns `Equipo`, `Plataforma`, `Versión`, `Última conexión`, `Estado`, `Acciones`; `Equipo` is `{profile_name} · {device_name_label(device)}`, `Plataforma` uses `platform_label`, `Versión` uses `app_version` or `—`, `Última conexión` uses `last_seen_label`, `Estado` uses `DeviceStatusBadge`, and `Acciones` is Danger `Revocar` when not revoked, else `—`. When there are no devices, render `EmptyState title="Sin dispositivos" description="No hay equipos registrados."` inside the Dispositivos card. Keep the existing `refresh`/`refresh += 1` counters and every API closure. Access denied uses `Notice tone="Error"` and a Secondary `ButtonLink` `Ir a mi perfil` -> `Route::SettingsProfile {}`. No Primary; commands: `cd persona-sanus && cargo check`; `rg -n 'refresh|remove_member|disable_profile|revoke_device|fetch_users|fetch_devices|AccountStatusBadge|DeviceStatusBadge' persona-sanus/src/main.rs`.
- [x] Adopt SettingsProfile — requirements: use `PageShell id="settings-page"` and `PageHeader title="Mi perfil"` with the existing clinic/role subtitle. `SectionCard title="Datos del perfil"` contains `FormGrid` with `TextField`s `Nombre visible`, and for doctors `Especialidad`, `Número de licencia`, `Cédula`, `Teléfono`, `Correo` (`input_type="email"`); `ActionRow` has the only Primary `Guardar` (loading label `Guardando...` while saving). Below, `div.ui-columns` contains `SectionCard title="Verificación en dos pasos"` (only when doctor or administrator) and `SectionCard title="Cambiar contraseña"`. Two-factor card: status `HintText` `Activada. Se pedirá el código al iniciar sesión.` or `Sin activar. Actívala para proteger tu cuenta.`; when a secret exists render `p.ui-break` `Secreto: {secret}` and `p.ui-break` `URI: {uri}` plus `TextField label="Código de 6 dígitos"` and Secondary `Confirmar activación`; otherwise, when MFA is enabled render `TextField label="Contraseña para desactivar" input_type="password"` and Danger `Desactivar verificación`; otherwise render Secondary `Activar verificación`; `mfa_message` renders as `HintText`. The three branches are mutually exclusive. Password card: intro `HintText` `Al cambiarla se invalidan las demás sesiones.`, `FormGrid` with `TextField label="Contraseña actual" input_type="password"` and `TextField label="Nueva contraseña" input_type="password"`, Secondary `Cambiar contraseña`, and `password_message` as `HintText`. Keep every `auth::mfa_status`, `auth::enroll_mfa`, `auth::confirm_mfa`, `auth::disable_mfa`, `auth::change_password`, and `api::update_profile` call and its signal clearing. Only Primary: `Guardar`; commands: `cd persona-sanus && cargo check`.
- [x] Adopt Audit — requirements: use `PageShell id="audit-page"` and `PageHeader title="Auditoría" subtitle="Registro de solo anexación con retención de 5 años. Filtra por acción, entidad o texto."`. Non-admin: `Notice tone="Error"` with the existing access-denied message and a Secondary `ButtonLink` `Ir a mi perfil`. Admin: `FilterBar` with `TextField label="Acción" placeholder="Acción (p. ej. sensitive_read)"`, `TextField label="Entidad" placeholder="Entidad (p. ej. patient)"`, `TextField label="Buscar" placeholder="Buscar en entidad o metadatos"`, and Primary `Filtrar` that increments `generation`. Add `loaded` false; set false at the start of each fetch and true after it resolves. Render `LoadingState` while `!loaded()`, `ErrorText` for `message`, and `EmptyState title="Sin eventos"` when there are no rows. Otherwise `DataTable` columns `Fecha`, `Actor`, `Acción`, `Entidad`, `ID entidad`, `Metadatos`; `Fecha` uses `format_timestamp(row.at)`, `Actor` the existing fallback, `Metadatos` renders `code.ui-mono`. Keep `api::fetch_audit` and its arguments. Only Primary: `Filtrar`; commands: `cd persona-sanus && cargo check`; `rg -n 'format_timestamp|ui-mono|fetch_audit|Solo el administrador' persona-sanus/src/main.rs`.

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No new crate, CSS framework, external stylesheet, webfont, CDN asset, icon package, or JavaScript bridge.
- No rich text, WYSIWYG, `contenteditable`, formatting toolbar, `execCommand`, HTML payload, or editor dependency. All text entry is plain controlled inputs, textareas, selects, and checks.
- No new product route, API handler, server function, Diesel model, table, migration, entitlement rule, or billing persistence. `/settings/billing` stays reserved with no surface.
- No new endpoint, no new `api::` function, no new permission, no new role check, and no new editable state. `deleting` is a transient UI identifier, not an editable field.
- No card form, PAN, CVV, full card number, `ctoken`, `card_token`, `card_holder_enc`, or `authorization_code` in any page, prop, or copy.
- No change to authorization, session, device registration, sync, numbering, signing, PDF, QR, verification limits, or business validation.
- No change to the clinic/profile/users/devices/audit request arguments, success handling, or refresh counters.
- No extra Primary action. `SettingsClinic` has only `Guardar datos`; `SettingsUsers` has none; `SettingsProfile` has only `Guardar`; `Audit` has only `Filtrar`.
- No duplicate success and error text. Positive confirmation stays `HintText`; an API failure uses the surface named for that page: `ErrorText` on clinic, users, and audit, `HintText` on the profile page's existing message signals.
- No patient photo, signature, or stamp control (D-06); no rich-text editor (D-21).
- No pure black, drop shadow, chromatic card border, bold grenette, or second stylesheet.
- No unit tests and no comments in Rust, CSS, or RSX.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [x] `cd persona-sanus && cargo check` exits 0.
- [x] `cd persona-sanus && cargo check --no-default-features --features desktop` exits 0.
- [x] objective 1: `rg -n 'Signal<SyncStatus>|ImageLibrary|upload_logo|upload_asset|delete_asset|ui-asset-preview' persona-sanus/src/main.rs` returns the sync read, the library, all three APIs, and the preview class.
- [x] objective 1: source shows named logo-upload, image-upload, and delete closures, and `event.files().first()` only inside the first-file selection.
- [x] objective 1: browser `/settings/clinic` as administrator saves name/address/phone/LLM template, shows the logo preview when present, shows the no-logo hint when absent, shows quota `Uso de la biblioteca: {used} de {limit}.`, lists only image assets, marks a pending asset `Procesando` with no thumbnail, renders the ready placeholder when `url` is absent, deletes a ready image, and keeps the export link at `/api/clinic/export` labelled `Exportar datos (JSON)`.
- [x] objective 1: with the app in offline sync, both file inputs are disabled and the offline hints are visible; delete buttons are disabled; no upload or delete request starts — source-level proof: the two file inputs are disabled when `!online || uploading()`, the offline hints render when `!online`, delete controls are disabled when `!online`, and the upload/delete closures are unreachable while disabled; runtime offline state is desktop-only because `http::is_server_reachable` is constant true on wasm32.
- [x] objective 1: Primary count on `/settings/clinic`: exactly 1, `Guardar datos`; `Volver`, upload controls, delete, and export are not filled primary.
- [x] objective 2: browser `/settings/users` as administrator shows both cards, the six Personal columns, the six Device columns, role changes, `Habilitar`/`Deshabilitar`, `Quitar del equipo`, and Danger `Revocar` only on active devices; each successful action refreshes its table. Empty users/devices show the two contracted EmptyStates inside their cards. No filled primary.
- [x] objective 2: browser `/settings/users` as non-administrator shows Error `Notice` and Secondary `Ir a mi perfil`; no table and no primary.
- [x] objective 3: browser `/settings/profile` shows profile fields per role, Secondary `Activar verificación`, Secondary `Confirmar activación`, Danger `Desactivar verificación`, Secondary `Cambiar contraseña`, and exactly one filled `Guardar`. Enabling MFA reveals the secret and URI with break-any-all styling; confirming clears the secret; disabling clears the enabled state; changing the password clears both password fields and reports via HintText.
- [x] objective 4: browser `/audit` as administrator shows the filter bar, a loading state on first fetch, ErrorText on failure, `Sin eventos` for an empty result, and the six-column table with `format_timestamp` dates and monospace metadata. `Filtrar` is the only filled primary. As non-administrator it shows the Error Notice and no table or primary.
- [x] Primary count: `/settings/clinic` 1, `/settings/users` 0, `/settings/profile` 1, `/audit` 1, and each access-denied state 0.
- [x] Focus pass: Tab through clinic fields/uploads/delete/export, user role/actions, profile fields/MFA/password, and audit filters; every control shows the visible focus ring and no focus is trapped.
- [x] Responsive pass: clinic/profile columns stack below 900; form and detail grids stack below 760; user and audit tables scroll inside `.ui-table-wrap`; the page itself never scrolls horizontally.
- [x] every objective in section 1 is checked.
- [x] every divergence found is appended to section 4 below.

## 4. Divergences

*Process and tracking-file rules: see `AGENTS.md` (§ Spec-Driven Development).
Summary: when reality diverges from the spec, append a numbered entry below;
leave the matching testing-criteria `- [ ]` unchecked and note the divergence
inline next to it. Open divergences owned by a later spec also get a tracking
file in `specs/divergences/`.*

- [x] every divergence found is appended below (or the section reads "None.")

*Entry format (one numbered item per divergence):*

1. <what diverged and its current state> — <why it could not be done as
   written, citing the No-go rule or spec that blocks it> (Verified <date>:
   <N> fix attempts — <attempt 1>, <attempt 2>, <attempt 3> — all fenced by
   <constraint>; <impact> is deferred to <later spec> by design, not a defect.)

### Divergences

None recorded yet.

## 5. Context

These four pages finish the v1 screen adoption. After this spec, no release page renders a raw `.button`, `.field`, `.form-grid`, `.page-head`, `.notice`, `.badge`, `.asset-row`, `.settings-columns`, `.break-anywhere`, `.item-card`, or `.medication-block` class. The only inline-style exception is `A4Preview`'s intentional print projection and geometry, which `25-Y` preserves.

`SettingsClinic` is the only page that needs the sync context during markup adoption. It currently fetches settings and the asset library, lists raw asset rows (including logo assets, because it does not filter), uses one `uploading` signal for both logo and library uploads, and deletes without a busy state. The new composition: the logo preview/hint and logo `FileField`, then `ImageLibrary` for the filtered image list, quota, delete, and library `FileField`. Pass `online` to both; add a `deleting: Signal<Option<String>>` to identify the asset being deleted. Keep one `uploading`/`saving` model; do not add a second concurrent upload flag.

`SettingsUsers` has plain per-row blocks. Replace them with a `DataTable` inside each `SectionCard`; keep the same `api::set_role`, `api::remove_member`, `api::disable_profile`, and `api::revoke_device` closures and the same `refresh += 1` calls. The table is display-only; actions are buttons in the `Acciones` cell.

`SettingsProfile` has three independent message signals and uses raw inputs/buttons. Keep the signals; render each as `HintText` because the page table names hints, not errors. The two-factor card is conditional on `is_doctor || is_admin`; the password card is always visible. The secret and URI are plain text in `.ui-break`; they are never logged and never shown in the gallery.

`Audit` has three filters, a generation counter, rows, and one message. It needs one `loaded` flag. Non-admin access is a Notice, not a fetch. Keep the 5-year retention sentence and the filter placeholders.

## 6. Reuse, do not rebuild

| Existing symbol | Decision |
|---|---|
| `forms::A4Preview` | Not used here. Keep unchanged. |
| `forms::TemplateDocumentForm` | Not used here. `22-V` owns it. |
| `Navbar` | Adopted by `20-T`; these pages render inside it. No duplicate shell. |
| `Route`, `App` | Keep. No route additions, renames, or `/settings/billing`. |
| `forms::paginate_document`, `resolve_text`, document warnings | Not used here. Keep unchanged. |
| `SyncStatus`, `probe_sync_status`, local count helpers | Consume the `18-R` public type and `SyncStatusIndicator`. `SettingsClinic` reads the signal; it does not probe. |
| `ImageLibrary`, `QuotaMeter` | Consume the `18-R` components. Do not rebuild quota/list/upload/delete markup in `SettingsClinic`. |
| `AccountStatusBadge`, `DeviceStatusBadge` | Consume `18-R`. Do not map status labels locally. |
| `format_date`, `format_timestamp`, `last_seen_label`, `platform_label`, `device_name_label` | Consume `15-O` exports. Do not copy. |
| `api::fetch_clinic_settings`, `update_clinic_settings`, `fetch_assets`, `upload_logo`, `upload_asset`, `delete_asset`, `fetch_users`, `set_role`, `remove_member`, `disable_profile`, `fetch_devices`, `revoke_device`, `fetch_audit` | Keep the calls and arguments. Only extract bodies into named closures where the page table requires. |
| `auth::mfa_status`, `enroll_mfa`, `confirm_mfa`, `disable_mfa`, `change_password` | Keep the calls, success branches, signal clearing, and messages. |
| `.a4-*` CSS | Not touched by these pages. |

## 7. Component surface

No new Rust components. This spec consumes `PageShell`, `PageHeader`, `Card`, `SectionCard`, `FormGrid`, `ActionRow`, `FilterBar`, `DetailGrid`, `DetailItem`, `TextField`, `FileField`, `SelectField`, `Button`, `ButtonLink`, `LinkAction`, `Badge`, `Notice`, `ErrorText`, `HintText`, `EmptyState`, `LoadingState`, `DataTable`, `ImageLibrary`, `QuotaMeter`, `AccountStatusBadge`, `DeviceStatusBadge`, and `SyncStatus`.

Fixed variants:

| Control | Variant |
|---|---|
| `Guardar datos` | Primary |
| `Guardar` | Primary |
| `Filtrar` | Primary |
| `Volver`, `Ir a mi perfil` | Secondary |
| `Exportar datos (JSON)` | Secondary `LinkAction` |
| `Habilitar` | Secondary |
| `Revocar`, `Deshabilitar`, `Quitar del equipo`, `Desactivar verificación` | Danger |
| `Activar verificación`, `Confirmar activación`, `Cambiar contraseña` | Secondary |

## 8. Page adoption

### SettingsClinic

- `PageShell id="settings-page"`.
- `PageHeader title="Clínica" subtitle="Datos de la clínica, logo y biblioteca de imágenes."`; header Secondary `ButtonLink` `Volver` -> `Route::Settings {}`.
- `ErrorText` for errors; positive confirmations (`Datos de la clínica actualizados.`, `Logo actualizado.`, `Imagen agregada a la biblioteca.`, `Imagen eliminada.`) render as `HintText` from a separate status signal.
- `SectionCard title="Datos de la clínica"`: `FormGrid` with `TextField` `Nombre`, `Dirección`, `Teléfono`, and `Plantilla del enlace LLM` (placeholder `https://chat.openai.com/?q=`, wide). `ActionRow` Primary `Guardar datos`; disabled/loading label `Guardando...` while saving.
- `SectionCard title="Logo"`: when `logo.url` is Some, `img.ui-asset-preview src=url alt="Logo de la clínica"`; else `HintText "La clínica todavía no tiene logo."`. `FileField label="Subir logo (JPEG o PNG, máximo 10 MB)" accept="image/jpeg,image/png"` disabled when `!online || uploading()`, `on_change` calls the named logo closure. The closure selects only `event.files().first()`, checks `http::is_server_reachable()`, reads bytes, calls `api::upload_logo`, increments `generation`, and clears `uploading`.
- `SectionCard title="Biblioteca de imágenes"`: `ImageLibrary { assets: library.assets, used: library.quota.used, limit: library.quota.limit, uploading: uploading(), deleting: deleting(), online, on_delete: named_delete, on_upload: named_upload }`. Do not prefilter the assets in the page; the component ignores every `kind != "image"` row, including the logo. The upload closure selects only `event.files().first()`, checks reachability, reads bytes, calls `api::upload_asset`, increments `generation`, and clears `uploading`; the delete closure sets `deleting` to the asset id, calls `api::delete_asset`, increments `generation`, and clears `deleting`. Keep the existing messages.
- `SectionCard title="Exportación de datos"`: retention hint `Retención en servidor hasta: {value}` when present; grace hint `Fin del período de gracia para exportación: {value}` when present; the existing explanation `Exporte los datos de la clínica (JSON: pacientes, recetas y medicamentos) antes de que finalice el período de gracia. La exportación queda registrada en la auditoría.`; Secondary `LinkAction href="/api/clinic/export"` `Exportar datos (JSON)`.
- Keep the authorization redirects and the 1-image quota. Only Primary: `Guardar datos`.

### SettingsUsers

- `PageShell id="settings-page"`.
- `PageHeader title="Usuarios y roles" subtitle="Asigna un rol o deshabilita el acceso de cada persona."`.
- `ErrorText` for `message`.
- `SectionCard title="Personal"`:
  - If `users()` is empty: `EmptyState title="Sin usuarios" description="No hay usuarios para mostrar."`.
  - Else `DataTable columns: vec!["Persona".to_owned(),"Correo".to_owned(),"Alta".to_owned(),"Estado".to_owned(),"Rol".to_owned(),"Acciones".to_owned()]` and plain rows. Cells:
    - Persona: `display_name`.
    - Correo: `email`.
    - Alta: `format_date(created_at)`.
    - Estado: `AccountStatusBadge { status }`.
    - Rol: `SelectField label="Rol" hide_label value={row.role.clone().unwrap_or_default()}` with the existing options (`Sin rol` value empty and disabled, `Administrador`, `Doctor`, `Enfermería`, `Secretaría`); `onchange` ignores empty and calls `api::set_role`, then `refresh += 1`.
    - Acciones: Danger `Quitar del equipo` when `row.role.is_some()` calls `api::remove_member`; Danger `Deshabilitar` when active or Secondary `Habilitar` when disabled calls `api::disable_profile`; each success `refresh += 1`.
- `SectionCard title="Dispositivos" description="Equipos registrados y su último acceso."`:
  - If `devices()` is empty: `EmptyState title="Sin dispositivos" description="No hay equipos registrados."`.
  - Else `DataTable columns: vec!["Equipo".to_owned(),"Plataforma".to_owned(),"Versión".to_owned(),"Última conexión".to_owned(),"Estado".to_owned(),"Acciones".to_owned()]`:
    - Equipo: `{profile_name} · {device_name_label(device)}`.
    - Plataforma: `platform_label(&platform)`.
    - Versión: `app_version` or `—`.
    - Última conexión: `last_seen_label(last_seen_at.as_deref())`.
    - Estado: `DeviceStatusBadge { revoked: revoked_at.is_some() }`.
    - Acciones: when not revoked, Danger `Revocar` calls `api::revoke_device` and `refresh += 1`; when revoked, `—`.
- Access denied: `PageShell` > `Notice tone="Error"` with `Solo el administrador puede gestionar usuarios y roles.` and Secondary `ButtonLink` `Ir a mi perfil` -> `Route::SettingsProfile {}`.
- No Primary.

### SettingsProfile

- `PageShell id="settings-page"`.
- `PageHeader title="Mi perfil"` subtitle exactly the existing clinic/role string. Keep the no-clinic fallback.
- `SectionCard title="Datos del perfil"`:
  - `FormGrid columns: Two` with `TextField` `Nombre visible`, always; and when `is_doctor`, `Especialidad`, `Número de licencia`, `Cédula`, `Teléfono`, `Correo` (`input_type="email"`).
  - `ActionRow` Primary `Guardar`, disabled/loading while saving, label `Guardando...` while busy, existing `api::update_profile` call; success message `Perfil guardado.`.
- `div.ui-columns`:
  - When `is_doctor || is_admin`, `SectionCard title="Verificación en dos pasos"`: `HintText` status line. If `mfa_secret` is non-empty, render `p.ui-break` `Secreto: {secret}`, `p.ui-break` `URI: {uri}`, `TextField label="Código de 6 dígitos"` bound to `mfa_code`, and Secondary `Confirmar activación` calling `auth::confirm_mfa`. Else if `mfa_enabled`, render `TextField label="Contraseña para desactivar" input_type="password"` and Danger `Desactivar verificación` calling `auth::disable_mfa`. Else render Secondary `Activar verificación` calling `auth::enroll_mfa`. Render `mfa_message` as `HintText` when non-empty.
  - Always `SectionCard title="Cambiar contraseña"`: `HintText` `Al cambiarla se invalidan las demás sesiones.`; `FormGrid` with `TextField label="Contraseña actual" input_type="password"` and `TextField label="Nueva contraseña" input_type="password"`; Secondary `Cambiar contraseña` calling `auth::change_password`; clear both password signals on success; render `password_message` as `HintText`.
- Only Primary: `Guardar`.

### Audit

- `PageShell id="audit-page"`.
- `PageHeader title="Auditoría" subtitle="Registro de solo anexación con retención de 5 años. Filtra por acción, entidad o texto."`.
- Non-admin: `Notice tone="Error"` with the existing access-denied message and Secondary `ButtonLink` `Ir a mi perfil` -> `Route::SettingsProfile {}`.
- Admin `FilterBar`:
  - `TextField label="Acción" placeholder="Acción (p. ej. sensitive_read)" value={action}`.
  - `TextField label="Entidad" placeholder="Entidad (p. ej. patient)" value={entity}`.
  - `TextField label="Buscar" placeholder="Buscar en entidad o metadatos" value={q}`.
  - Primary `Filtrar` increments `generation`.
- Add `loaded` false; set false at the start of every fetch and true after the `fetch_audit` `match` resolves. `LoadingState` while `!loaded()`.
- `ErrorText` for `message` when non-empty.
- `EmptyState title="Sin eventos"` when `rows().is_empty() && loaded()`.
- Else `DataTable columns: vec!["Fecha".to_owned(),"Actor".to_owned(),"Acción".to_owned(),"Entidad".to_owned(),"ID entidad".to_owned(),"Metadatos".to_owned()]`; `Fecha` uses `format_timestamp(&row.at)`; `Actor` uses `row.actor_name.clone().unwrap_or_else(|| row.actor_id.clone().unwrap_or_default())`; `Metadatos` renders `code.ui-mono` with `row.metadata`.
- Keep `api::fetch_audit(&action_value, &entity_value, &q_value)` and the admin guard.
- Only Primary: `Filtrar`.

## 9. CSS contract

| Selector | Values |
|---|---|
| `.ui-asset-preview` | display block, max-width 240px, max-height 96px, object-fit contain, margin-bottom 12px, padding 8px, 1px hairline, radius 8, Paper background |
| `.ui-columns` | owned by `16-P`; consume it |
| `.ui-library` | owned by `18-R`; consume it |
| `.ui-quota` | owned by `18-R`; consume it |

No page-specific selector beyond `.ui-asset-preview`. The pages use `.ui-page`, `.ui-container`, `.ui-stack`, `.ui-page-head`, `.ui-card`, `.ui-card-title`, `.ui-card-description`, `.ui-form-grid`, `.ui-detail-grid`, `.ui-field`, `.ui-input`, `.ui-textarea`, `.ui-select`, `.ui-actions`, `.ui-filter-bar`, `.ui-empty`, `.ui-loading`, `.ui-row`, `.ui-table-wrap`, `.ui-table`, `.ui-notice`, `.ui-error`, `.ui-hint`, `.ui-break`, `.ui-mono`, and `.ui-badge` from earlier specs. No drop shadow.

## 10. Critical files and anchors

These are hints; reread before editing.

| File | Anchor |
|---|---|
| `persona-sanus/src/main.rs` | `SettingsClinic` 1804; `SettingsUsers` 2120; `SettingsProfile` 2319; `Audit` 2634; `Settings` 1761; `enum Route` 48-93 |
| `persona-sanus/src/ui/domain.rs` | `ImageLibrary`, `QuotaMeter`, `SyncStatus`, badge components, `format_bytes` |
| `persona-sanus/src/ui/form.rs` | `TextField`, `FileField`, `SelectField`, `Button` |
| `persona-sanus/src/ui/layout.rs` | `PageShell`, `PageHeader`, `SectionCard`, `FormGrid`, `ActionRow`, `FilterBar`, `DetailGrid`, `DetailItem`, `Card` |
| `persona-sanus/src/ui/feedback.rs` | `Notice`, `ErrorText`, `HintText`, `EmptyState`, `LoadingState`, `DataTable` |
| `persona-sanus/assets/main.css` | Add `.ui-asset-preview`; keep legacy selectors until `25-Y` |
| `context/PRD.md` | FR-080–085, FR-042, FR-043, D-06, D-21 |
| `context/database-design.md` | §4 status values |

## 11. Assumptions and contingencies

- `SettingsClinic` keeps two independent success/error strings: one `message` for errors and one `status` for success confirmations. This satisfies the shared rule that positive text is not moved into an error state. If a page prefers one string, do not style it with `ErrorText` when the content is a success.
- `SettingsUsers` keeps its current behavior of not redirecting a non-admin; it renders the Error Notice. The route guard remains the server's job.
- The user table's role `SelectField` is `hide_label`; it still needs the label `Rol` in the DOM for an accessible name. Do not pass an empty label.
- The device `Estado` column uses `DeviceStatusBadge` with `revoked: revoked_at.is_some()`. A device with no `revoked_at` is Active.
- `Audit` adds a `loaded` signal only for its first fetch; changing filters sets it false and true around the same `fetch_audit` call. Do not add a second fetch path.
- `Audit` empty state appears only after the fetch resolves; do not show it during loading.
- If the user/profile page lacks a doctor record, render only the fields it has; do not invent a doctor row.
- If an upload uses the browser's file picker and `FormEvent` cannot cross the `ImageLibrary` boundary, keep the `FileField` in `SettingsClinic` and pass only the selected first file to the upload closure; the visual contract and disabled states do not change.
- If a page's role guard renders an access-denied surface, preserve its current copy and use `Notice tone="Error"`; do not add a Primary action.
- If browser verification finds a control not in the Design.md list but required by an existing page, use the nearest primitive (`Badge`, `Card`, `Button Secondary`) and do not add a component file. The component set is closed.

---

*Housekeeping: once a spec is fully done with no open divergences, move its
file to `specs/done/` (same file name, e.g.
`specs/done/24-X-ui-settings-detail-and-audit.md`).
Specs with open divergences stay in `specs/` next to `TEMPLATE.md` until the
tracking file is cleared.*
