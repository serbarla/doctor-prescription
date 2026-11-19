# Spec 21 — Phase U: Dashboard, settings, and patient adoption

- Source: `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` step 13 and the Page adoption table; `context/sitemap.md` §§1.1–1.3, §2, §5; `context/PRD.md` FR-001/003; `context/database-design.md` §4; `context/Design.md` §§Components, Tokens, Surfaces, Layout.
- Depends on: 15-O, 16-P, 17-Q, 18-R, 19-S, 20-T.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Result carried forward: `Dashboard`, `Settings`, `Patients`, `PatientNew`, `PatientDetail`, and `PatientEdit` render only through the UI primitives and page shell; every guard, redirect, fetch, navigation target, and dispatch path is unchanged; each page has at most one filled primary action, named in section 8; `Patients` exposes a loading state from a `loaded` signal set after the fetch resolves.

## Before you start

Unit testing is not required; avoid it. Prioritize end-to-end and integration testing only.

Read the nearest `AGENTS.md` (target workspace, then parents up to repo root) before executing.

If reality diverges from this spec: STOP, append a divergence (section 4), and NEVER improvise outside the objectives.

This spec is markup adoption. Do not change authorization, session, API arguments, desktop fallback data, deduplication rules, or business validation. The only new state is `Patients::loaded`; `Dashboard` keeps its existing `checked` context.

## 1. Objectives

*The ordered work to implement — one item per step. Check `[x]` only after the step is implemented and verified.*

*Item format:* `<verb phrase>` — requirements: `<what must change / constraints>`; commands: `<exact command>`.

- [x] Adopt Dashboard — requirements: use `PageShell id="dashboard-page"` and `PageHeader title="Panel"` with the session subtitle `{name} · {email}`. Render `LoadingState` while `checked` is false. Keep the signed-out `navigator.replace(Route::AuthPage {})` guard. In the signed-in branch render `SectionCard title="Sesión"` with `DetailGrid` items `Nombre` (session.name), `Correo` (session.email), `Rol` (Badge with `role_label(profile.role)` when a profile exists, else Neutral `Sin rol`), and `Clínica` (`profile.clinic.name` when present, else exact `Sin clínica asignada.`). Remove the duplicate page-level `Cerrar sesión` control; the navbar owns sign-out. No Primary action; commands: `cd persona-sanus && cargo check`.
- [x] Adopt Settings — requirements: use `PageShell id="settings-page"` and `PageHeader title="Configuración" subtitle="Administra el acceso del personal y tu perfil."`. Render `div.ui-link-grid` with Secondary `ButtonLink` `ui-link-card`s `Usuarios y roles` -> `Route::SettingsUsers {}`, `Clínica y biblioteca` -> `Route::SettingsClinic {}`, `Mi perfil` -> `Route::SettingsProfile {}`, admin-only `Auditoría` -> `Route::Audit {}`, and a debug-only `Sistema de diseño` -> `Route::DevUi {}`. Keep the existing redirects and the compile-time gate on the debug link. No Primary action; commands: `cd persona-sanus && cargo check`.
- [x] Adopt Patients — requirements: use `PageShell id="patients-page"` and `PageHeader title="Pacientes" subtitle="Busca por nombre o cédula dentro de tu clínica."`; render the Primary `ButtonLink` `Nuevo paciente` to `Route::PatientNew {}` only when `profile.can_write_patients()`. Add `let mut loaded = use_signal(|| false);`, set it true once after every fetch branch resolves (including the desktop fallback), and render `LoadingState` while `!loaded()`. Replace the search input with `SearchField label="Buscar paciente"` bound to `query`; keep the existing search effect and arguments. Render `EmptyState title="Sin pacientes" description="No hay pacientes para esta búsqueda."` when the list is empty. Render each result as `ListRow` with title `full_name`, meta `["Cédula {national_id or —}", "Nacimiento {format_date(birth_date) or —}"]`, and a Secondary `Ver ficha` child to `Route::PatientDetail { id }`. Keep the desktop local fallback. The only Primary is `Nuevo paciente`; commands: `cd persona-sanus && cargo check`.
- [x] Adopt PatientNew — requirements: use `PageShell id="patients-page"`, `PageHeader title="Nuevo paciente"` with a Secondary `Cancelar` header action to the existing target route. Put the fields in `Card` > `FormGrid columns: Two`: `TextField` for names, cédula, birth date (`input_type="date"`, `step="1"`), and tel; `SelectField` for sex; `TextAreaField wide` for notes. Keep every signal, initial value, validation, dedup check, and `submit_new_patient` call. When `warning_ids` is non-empty, render a Warning `Notice` with the existing warning message and a Danger `Button` `Crear de todas formas` that calls the existing confirmed submit. Render `ErrorText` only for a non-warning error. `ActionRow` has the only Primary, `Guardar paciente`; commands: `cd persona-sanus && cargo check`.
- [x] Adopt PatientDetail — requirements: use `PageShell id="patients-page"` and `PageHeader title={full_name} subtitle="Ficha del paciente"`. Header actions, in order: Secondary `Editar` when `can_write_patients`, else Neutral `Badge` `Solo lectura`; Primary `Nueva receta` when `can_create_prescriptions()`; Secondary `Historial de recetas` using the existing target. Keep every guard and navigation target. Render demographics in `Card` > `DetailGrid`/`DetailItem`. Delete is a Danger `Eliminar paciente` inside a `ConfirmPanel` with confirm `Confirmar eliminación`; keep the delete API and navigation. Use `ErrorText` for errors. The only Primary is `Nueva receta` and only when allowed; commands: `cd persona-sanus && cargo check`.
- [x] Adopt PatientEdit — requirements: use `PageShell id="patients-page"`, `PageHeader title="Editar paciente"` with a Secondary `Cancelar` header action. Put the same fields in `Card` > `FormGrid`; keep every signal, initial value, and validation. Use `ErrorText` for errors and `ActionRow` with the only Primary `Guardar cambios`; keep the existing update dispatch and navigation; commands: `cd persona-sanus && cargo check`.

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No new crate, CSS framework, external stylesheet, webfont, CDN asset, icon package, or JavaScript bridge.
- No rich text, WYSIWYG, `contenteditable`, formatting toolbar, `execCommand`, HTML payload, or editor dependency. Text fields remain plain controlled inputs, textareas, selects, and checks.
- No new product route, API handler, server function, Diesel model, table, migration, entitlement rule, or billing persistence.
- No change to authorization, session, device registration, sync, numbering, signing, PDF, QR, verification limits, or business validation.
- No change to API call arguments except setting `Patients::loaded` after the existing call resolves. Do not add, remove, or reorder a query/filter parameter.
- No `loaded` signal outside `Patients`. Dashboard keeps `checked`; Settings, PatientNew, PatientDetail, and PatientEdit do not gain a loading gate.
- No new editable state. Every input stays bound to its existing signal.
- No extra Primary action. Dashboard and Settings have none; the primaries are exactly the ones in section 8.
- No duplicate page sign-out on Dashboard; the navbar keeps sign-out.
- No patient photo, signature, or stamp control (D-06). No rich-text editor (D-21).
- No pure black, drop shadow, chromatic card border, bold grenette, or second stylesheet.
- No unit tests and no comments in Rust, CSS, or RSX.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [x] `cd persona-sanus && cargo check` exits 0.
- [x] `cd persona-sanus && cargo check --no-default-features --features desktop` exits 0.
- [x] objective 1: `Dashboard` source has `PageShell id="dashboard-page"`, `PageHeader`, `SectionCard`, `DetailGrid`, and no page-level sign-out dispatch.
- [x] objective 2: `Settings` source has `PageShell id="settings-page"`, `div.ui-link-grid`, four normal links plus the gated `Sistema de diseño` link, and no Primary button.
- [x] objective 3: `Patients` source has `PageShell id="patients-page"`, `SearchField`, `LoadingState`, `EmptyState`, `ListRow`; `rg -n 'loaded\.set\(true\)' persona-sanus/src/main.rs` shows the Patients fetch assignment.
- [x] objective 3: browser `/patients` shows a loading state on first load, the empty copy for a no-match query, Cédula/Nacimiento meta, and one `Ver ficha` action per row.
- [x] objective 4: browser `/patients/new` shows labelled fields, a Card + two-column grid, the duplicate Warning + Danger `Crear de todas formas` only after the dedup branch, and one Primary `Guardar paciente`; creating navigates to the detail.
- [x] objective 5: browser `/patients/:id` shows Secondary `Editar` or Neutral `Solo lectura` per role, Primary `Nueva receta` only with create permission, Secondary `Historial de recetas`, a details Card, an inline delete `ConfirmPanel`, and no modal focus trap.
- [x] objective 6: browser `/patients/:id/edit` shows the same fields, Secondary `Cancelar`, Primary `Guardar cambios`, and a successful save updates the detail.
- [x] Primary count: Dashboard 0, Settings 0, Patients 1 when `can_write_patients` else 0, PatientNew 1, PatientDetail 1 when `can_create_prescriptions` else 0, PatientEdit 1.
- [x] Focus pass: Tab through `/patients`, `/patients/new`, `/patients/:id`, and `/patients/:id/edit`; every control shows the visible focus ring.
- [x] Responsive pass: form and detail grids are one column below 760; page head stacks below 640; only tables or the A4 preview scroll horizontally, never the page.
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

None.

## 5. Context

These six pages cover the administrative frame of the app. They are independent of the template and prescription work except for their navigation targets, so they can land before or after `22-V`, `23-W`, and `24-X`.

`Dashboard` currently owns a duplicate `Cerrar sesión`/`Sign out` button. The navbar already signs out, so the page version is removed to leave one sign-out in the shell. `Settings` is the hub; `SettingsUsers`, `SettingsClinic`, and `SettingsProfile` are separate pages handled in `24-X`. The hub links to all of them, with `Auditoría` only for administrators and `Sistema de diseño` only in debug builds.

`Patients` currently has a raw search input, a legacy `patient-list` loop, and no loading state. It fetches through `api::fetch_patients(&term)` and, on failure, falls back to `desktop_local::list_patients`; both remain. The empty message is currently a positive `.hint` and becomes an `EmptyState` only when the fetch has completed and the list is empty.

`PatientNew`, `PatientDetail`, and `PatientEdit` share demographic fields but have different header actions and post-submit behavior. Preserve those differences. The `PatientNew` duplicate path is not an error: it is a Warning with an explicit override. Do not render the same text twice as an error.

## 6. Reuse, do not rebuild

| Existing symbol | Decision |
|---|---|
| `forms::A4Preview` | Not used by these pages. Keep unchanged. |
| `forms::TemplateDocumentForm` | Not used here. `22-V` owns it. |
| `Navbar` | Adopted by `20-T`. These pages render inside its layout; do not duplicate the shell or sign-out. |
| `Route`, `App` | Keep. No route additions or renames. |
| `forms::paginate_document`, `resolve_text`, document warnings | Not used here. Keep unchanged. |
| `SyncStatus`, `probe_sync_status`, local count helpers | Not used by these pages. Keep unchanged. |
| `format_date`, `role_label`, `sex_label`, `status_label`, `last_seen_label`, `platform_label`, `device_name_label` | Consume the `15-O` exports. Do not copy. |
| `submit_new_patient`, `submit_patient_update`, delete closures, dedup machinery | Keep every body and argument. This spec only changes the markup that calls them. |
| Existing API arguments and desktop fallbacks | Keep exactly. Do not add a `loaded` argument to a fetch. |
| `.a4-*` CSS | Not touched by these pages. |

## 7. Component surface

No new Rust components. This spec consumes the primitives named in section 8.

## 8. Page adoption

### Dashboard

- `PageShell id="dashboard-page"`.
- `PageHeader title="Panel"` with subtitle `{session.name} · {session.email}`.
- `LoadingState` while `!checked()`.
- Signed-out guard: keep `navigator.replace(Route::AuthPage {})`.
- Signed-in: `SectionCard title="Sesión"` > `DetailGrid` > `DetailItem` for `Nombre`, `Correo`, `Rol` (Badge `role_label(profile.role)`; Neutral `Sin rol` when no profile), `Clínica` (`profile.clinic.name` when present, else exact `Sin clínica asignada.`).
- Remove the page-level sign-out button. No primary action.

### Settings

- `PageShell id="settings-page"`.
- `PageHeader title="Configuración" subtitle="Administra el acceso del personal y tu perfil."`.
- `div.ui-link-grid` with Secondary `ButtonLink { extra_class: "ui-link-card" }` cards:
  - `Usuarios y roles` -> `Route::SettingsUsers {}`.
  - `Clínica y biblioteca` -> `Route::SettingsClinic {}`.
  - `Mi perfil` -> `Route::SettingsProfile {}`.
  - `Auditoría` -> `Route::Audit {}`, administrator only.
  - `Sistema de diseño` -> `Route::DevUi {}`, `#[cfg(debug_assertions)]` only; release builds must not contain this link or this string.
- Keep the existing redirects. No primary action.

### Patients

- `PageShell id="patients-page"`.
- `PageHeader title="Pacientes" subtitle="Busca por nombre o cédula dentro de tu clínica."`; header action Primary `ButtonLink` `Nuevo paciente` -> `Route::PatientNew {}` when `profile.can_write_patients()`.
- `SearchField label="Buscar paciente" placeholder="Nombre o cédula" value={query}` with the existing `oninput` query update.
- Add `let mut loaded = use_signal(|| false);` to the component. Set `loaded.set(true)` once after the fetch `match` resolves in the success, API-error, and desktop-fallback paths. Do not set it before the spawn completes downloads.
- Render `LoadingState` while `!loaded()`.
- Keep the existing access-denied `Notice` Error.
- Render `ErrorText { text: message }` when `message` is non-empty. Place it after the loading gate and before the list or empty state. Do not move a positive status into it.
- Empty list: `EmptyState title="Sin pacientes" description="No hay pacientes para esta búsqueda."`.
- Each row: `ListRow title={patient.full_name} meta={vec![format!("Cédula {}", patient.national_id.clone().unwrap_or_else(|| "—".to_owned())), format!("Nacimiento {}", patient.birth_date.as_ref().map(|value| format_date(value)).unwrap_or_else(|| "—".to_owned()))]}`, with a Secondary `ButtonLink` `Ver ficha` -> `Route::PatientDetail { id }`.
- Keep the desktop `desktop_local::list_patients` fallback and the existing mapping.
- Only Primary: `Nuevo paciente`.

### PatientNew

- `PageShell id="patients-page"`; `PageHeader title="Nuevo paciente"` with header Secondary `ButtonLink` `Cancelar` to the current cancel target (preserve it).
- `Card` > `FormGrid columns: FormColumns::Two` with:
  - `TextField label="Nombres completos"` bound to the existing names signal.
  - `TextField label="Cédula"` bound to national id.
  - `TextField label="Fecha de nacimiento" input_type="date" step="1"` bound to birth date.
  - `TextField label="Teléfono" input_type="tel"` bound to phone.
  - `SelectField label="Sexo"` with the existing options; preserve `sex` value.
  - `TextAreaField label="Notas" wide` bound to notes.
- Keep every existing signal, required-field check, birth-date validation, dedup request, and `submit_new_patient` call.
- When `warning_ids` is non-empty, render `Notice tone="Warning"` with the duplicate message and a Danger `Button` `Crear de todas formas` calling the existing confirmed submit.
- Render `ErrorText` for `message` only when `warning_ids` is empty.
- `ActionRow` with one Primary `Button` `Guardar paciente`.

### PatientDetail

- `PageShell id="patients-page"`; `PageHeader title={patient.full_name} subtitle="Ficha del paciente"`.
- Header actions, current order:
  - Secondary `ButtonLink` `Editar` -> `Route::PatientEdit { id }` when `profile.can_write_patients()`; otherwise Neutral `Badge` `Solo lectura`.
  - Primary `ButtonLink` `Nueva receta` -> `Route::PrescriptionNew { patient_id: Some(id.clone()), template_id: None }` when `profile.can_create_prescriptions()`.
  - Secondary `ButtonLink` `Historial de recetas` to the existing target.
- Demographics in `Card` > `DetailGrid` / `DetailItem`.
- Delete block: Danger `Button` `Eliminar paciente` toggling the existing confirmation state; `ConfirmPanel` with the existing message, Danger confirm `Confirmar eliminación`, Secondary cancel `Cancelar`, busy from the existing deleting signal, and the existing delete API/navigation.
- Use `ErrorText` for the error signal.
- Only Primary: `Nueva receta` when allowed; otherwise none.

### PatientEdit

- `PageShell id="patients-page"`; `PageHeader title="Editar paciente"` with header Secondary `ButtonLink` `Cancelar`.
- `Card` > `FormGrid` with the same fields as `PatientNew`, bound to the initial values and the existing edit signals.
- Keep the existing validation, update dispatch, and navigation.
- `ErrorText` for `message`; `ActionRow` with one Primary `Button` `Guardar cambios`.

## 9. CSS contract

| Selector | Values |
|---|---|
| `.ui-link-grid` | grid auto-fit minmax(220px, 1fr), gap 16 |
| `.ui-link-card` | Paper surface, 1px hairline, 24px radius, flex column, align start, gap 4, padding 16px 20px, Ink text, no underline; hover Deep Teal text, Cream background |

No page-specific selector is added. `Dashboard`, `Settings`, and the patient pages use `.ui-page`, `.ui-container`, `.ui-stack`, `.ui-page-head`, `.ui-card`, `.ui-card-title`, `.ui-form-grid`, `.ui-detail-grid`, `.ui-field`, `.ui-input`, `.ui-textarea`, `.ui-select`, `.ui-actions`, `.ui-empty`, `.ui-loading`, `.ui-row`, `.ui-notice`, `.ui-error`, and `.ui-badge` from earlier specs. Inputs and buttons have no shadow; only the focus ring is allowed.

## 10. Critical files and anchors

These are hints; reread before editing.

| File | Anchor |
|---|---|
| `persona-sanus/src/main.rs` | `Dashboard` 1701; `Settings` 1761; `Patients` 2758; `PatientNew` 2954; `PatientDetail` 3128; `PatientEdit` 3358; `enum Route` 48-93 |
| `persona-sanus/src/ui/mod.rs` | `15-O`–`20-T` exports |
| `persona-sanus/src/ui/form.rs` | Controls used by the patient pages |
| `persona-sanus/src/ui/layout.rs` | `PageShell`, `PageHeader`, `Card`, `SectionCard`, `FormGrid`, `DetailGrid`, `DetailItem`, `ActionRow` |
| `persona-sanus/assets/main.css` | Add `.ui-link-grid` and `.ui-link-card`; keep legacy selectors until `25-Y` |
| `context/sitemap.md` | route table; role matrix §2 |
| `context/PRD.md` | FR-001, FR-003 |

## 11. Assumptions and contingencies

- The `Neutral` control for `Solo lectura` is a `Badge` with `BadgeTone::Neutral`; the closed button variants are Primary, Secondary, Ghost, and Danger, so there is no Neutral button.
- The dashboard "session subtitle" uses only session fields: `{name} · {email}`. Do not add marketing copy.
- Patients' `loaded` signal is set after the fetch resolves; it is not a live loading indicator and not reset on every keystroke. If the existing effect re-fetches on every query, reset it at the start of that effect and set it again at the end, but keep the same API argument.
- The duplicate warning is a positive warning, not an API error. Keep its message in the `Notice`; do not render it a second time in `ErrorText`.
- If `ButtonLink` cannot carry `extra_class: "ui-link-card"` in the installed Dioxus version, wrap the link in `div.ui-link-card` and keep the button classes; the layout and one-primary rule do not change.
- If a page's role guard renders an access-denied surface, preserve its current copy and use `Notice tone="Error"`; do not add a Primary action.
- If browser verification finds a control not in the Design.md list but required by an existing page, use the nearest primitive (`Badge`, `Card`, `Button Secondary`) and do not add a new component file.

---

*Housekeeping: once a spec is fully done with no open divergences, move its
file to `specs/done/` (same file name, e.g.
`specs/done/21-U-ui-dashboard-settings-and-patients.md`).
Specs with open divergences stay in `specs/` next to `TEMPLATE.md` until the
tracking file is cleared.*
