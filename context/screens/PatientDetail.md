# PersonaSanus — PatientDetail (`/patients/:id`)

| Field | Value |
|---|---|
| Source | `context/sitemap.md` §1.2 (`/patients/:id` — Patient detail; Doctor, Nursing, Secretary role-filtered, Administrator read-only; demographics, edit/delete actions, New prescription, prescription history link), §2 (role matrix), §3 flows 1 and 5, §4 M0/M1; `context/PRD.md` FR-001, FR-002, FR-003, FR-010, FR-013, D-04, D-06, D-21; `context/Design.md` (Paper, Ink, Mist, Hairline, Deep Teal, Bright Teal, Cream, Plum, Blossom, Ember Ring, Mint Glass; Card and 24px radius; 8px controls; flat elevation; line icons); `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` step 13 (line 204), Page adoption row `PatientDetail` (line 222), plus `PageShell`, `PageHeader`, `Badge`, `Button`, `ButtonLink`, `Card`, `DetailGrid`, `DetailItem`, `ActionRow`, `ConfirmPanel`, `ErrorText`, `LoadingState`; `context/screens/INDEX.md` §2.3 |
| Component | `PatientDetail` — `persona-sanus/src/main.rs:3128` (`#[component]` at `:3127`; body closes at `:3317`, before `submit_edit_patient` at `:3319`); composes `PageShell`, `PageHeader`, `Badge`, `Button`, `ButtonLink`, `Card`, `DetailGrid`, `DetailItem`, `ActionRow`, `ConfirmPanel`, `ErrorText`, and `LoadingState` planned in `persona-sanus/src/ui/` (the directory is not shipped yet); reads `Signal<Option<SessionUser>>`, `Signal<Option<Me>>`, and `Signal<bool>`; owns `Patient`, `message`, `confirm_delete`, and `deleting` signals |
| Route | `/patients/:id` (`Route::PatientDetail { id: String }`) — declared at `main.rs:62-63`; authenticated shell; signed-out visitors replace with `/login` (`Route::AuthPage {}`); any clinic role may read, the Administrator is read-only; no query parameters; `:id` is the opaque `patient.id` and the only value in the URL |
| Layout | `PageShell id="patients-page"` inside `#[layout(Navbar)]` (opens at `main.rs:51` and is never closed); root `div#patients-page.ui-page` > `.ui-container.ui-stack`; Paper canvas, 24px stack gap, 48px top / 80px bottom / 24px inline; no full-bleed band, sidebar, tab strip, or modal layer |
| Locale | `es-EC` on every string. `cédula` and `receta` keep Ecuador spelling. Birth dates render `DD-MM-AAAA`; no time is displayed on this page |
| Status | ✅ Spec complete (sections 1–7). Route status: 🟡 — sitemap §1.2: “Demographics, edit/delete actions, New prescription, and the prescription history link are implemented. Actions: New prescription, Edit (not for the read-only Administrator).” Shipped: demographics, Edit, inline delete confirmation, New prescription, general history link, desktop cache fallback. Target: the plan step 13 adoption below, frozen Spanish failure copy, and a cache fallback that never hides the live-read failure |

The patient's clinical identity card. It reads one clinic-scoped record, exposes the authorized follow-up actions for it, and offers a guarded soft delete. It introduces no route, endpoint, table, column, role, permission, or field.

## 1. Purpose

The screen performs five jobs, in order:

1. **Identify.** Show the one patient record the route names: full name as the title, cédula, birth date, sex, phone, and notes.
2. **Orient.** State the record kind with the subtitle `Ficha del paciente`, and make the return path to the directory explicit without competing with the actions.
3. **Act.** Offer the role-filtered next steps: edit the record, start a new prescription already bound to this patient, or open prescription history. The Administrator has no edit or delete path; the Secretary has no prescription-creation path.
4. **Delete.** Give a writer a two-step, inline soft delete that says what disappears and what is audited. Never a modal, a hard delete, a cascade, or a success banner.
5. **Bound.** Keep the server's clinic scope and write policy as the security boundary. The desktop cache only fills a failure; it never changes permission or invents fields.

It never renders:

- PHI outside an authorized clinic role. The record is clinical content; its name, cédula, birth date, sex, phone, and notes render only to a caller the server admits, and none of them appears in the URL, a marketing string, a QR code, a PDF, or audit metadata. The audit row carries the opaque patient id plus `via` and `found`, never a clinical value.
- Billing, price, trial, plan, entitlement, quota, or `sync_locked` copy (D-04). This page has no plan state and no upgrade path.
- A signature or stamp image (D-06); a patient photo, avatar, or illustration.
- A rich-text editor, formatting toolbar, `contenteditable` surface, or formatted clinical value (D-21). Notes are read-only plain text.
- An edit input, save control, or unsaved-change affordance. Editing belongs to `/patients/:id/edit`.
- Created or updated timestamps, the patient id, clinic identity, prescription count, embedded prescription list, document preview, PDF, QR, or billing.
- A hard-delete confirmation. The only destructive action is the soft delete described in §4.

## 2. Structure

One centered Paper page: header, one identity card, then the delete and return actions. The page has no tinted band and no nested navigation. Regions 1–3 are shared by every role that can read; role changes only the action set.

| Order | Region | id / class | Band tone | Contents |
|---|---|---|---|---|
| — | Navbar (layout) | `#navbar` · `header.ui-navbar` | Paper `#ffffff`, bottom hairline | Owned by `context/screens/Header.md` §2–§5; not restated here |
| 1 | Page shell | `#patients-page` · `div.ui-page` > `.ui-container.ui-stack` | Paper | 48px top padding, 80px bottom, 24px inline; stack gap 24px; max width 1200px |
| 2 | Page header | `header.ui-page-head` · `h1.ui-page-title` · `p.ui-page-subtitle` · `.ui-page-head-actions` | Paper | Full name as title; `Ficha del paciente`; at most three actions in the inline-end group |
| 3 | Identity card | `div.ui-card` holding `dl.ui-detail-grid` | Paper, 1px hairline | Five `DetailItem` pairs: Cédula, Fecha de nacimiento, Sexo, Teléfono, Notas (wide) |
| 4 | Feedback | `.ui-error` | Paper | One `ErrorText` alert line when the load failed or the delete failed; absent otherwise |
| 5 | Delete control | `.ui-actions` with Danger `Button`; or `.ui-confirm` | Paper; Cream panel | Danger `Eliminar paciente` trigger, replaced by the inline `ConfirmPanel` while confirmation is open |
| 6 | Return action | `.ui-actions` with Secondary `ButtonLink` | Paper | `Volver a pacientes` → `/patients` |

Regions 5 and 6 are not exclusive: the confirm panel owns its confirm and cancel controls, while region 6 remains the page's return link. No table renders, so there is no caption or `th scope` surface here.

### 2.1 Session and role

| Session / role | Reads | Header actions | Delete | What changes; what does not |
|---|:---:|---|:---:|---|
| Before identity: check in flight, no session, or `actor` is `None` | no | none | none | `PageShell` plus `LoadingState` `Cargando...` while checking. With no session, the effect replaces to `Route::AuthPage {}` and the transient `Redirigiendo...` notice renders; with the session but no profile yet, the same notice renders without a second navigation. No record fields, actions, or API request in either branch |
| Profile with no role | no | none | none | `PageHeader` denial title and subtitle; no card, actions, or return link |
| Administrator | yes | `Solo lectura`; `Historial de recetas` | absent | Read-only card and history. No `Editar`, no `Nueva receta`, no delete; the server also rejects patient writes |
| Doctor | yes | `Editar`; Primary `Nueva receta`; `Historial de recetas` | present | The full writer surface with exactly one Primary |
| Nursing | yes | `Editar`; Primary `Nueva receta`; `Historial de recetas` | present | Same regions as Doctor. Nursing drafts are unsigned on `/prescriptions/:id`, not here |
| Secretary | yes | `Editar`; `Historial de recetas` | present | Writer for patient data, but `can_create_prescriptions()` is false, so no Primary action exists |

Role changes only the header action set and whether the delete region exists. It never changes region order, field labels, date format, notes wrapping, or clinic scope. The client gates are UX; the server re-checks `require_patient_read` and `require_patient_write` on every request.

### 2.2 Load, failure, and the desktop cache

| State | What renders | What stays fixed |
|---|---|---|
| First load, no record and no message | `PageShell` plus `LoadingState` `Cargando...` in the page slot | Root id, Paper canvas, stack gap |
| Loaded | Header, identity card, delete trigger for writers, return link | Root id, five field labels, missing-value em dash, date format |
| Web failure, or desktop failure with no usable cache | `ErrorText` with the frozen load-failure line and one Secondary `Volver a pacientes`; no empty card | Root id, Paper canvas; no patient identity is guessed |
| Desktop API failure, matching cached row | The cached identity card renders read-only; the load-failure line remains above it; return link stays | Same labels and formats; no edit inputs; no silent success; an unreadable database or skipped row falls back to the failure surface |
| Record not found (404) | The same load-failure surface and return link | `api::fetch_patient` returns only a `String`, so this page has no separate not-found panel to invent |

A cached row is a read-only fallback, not a successful live read: it never removes the failure message, never enables a local delete, and never grants a role. An existing patient with all optional values absent is still a loaded patient: the card paints each label with `—`, not an `EmptyState`.

### 2.3 Actions and the delete state machine

| State | Controls | What happens |
|---|---|---|
| Idle, `administrator` | `Solo lectura` badge; `Historial de recetas`; `Volver a pacientes` | No patient mutation control renders |
| Idle, `secretary` | `Editar`; `Historial de recetas`; Danger `Eliminar paciente`; return | No Primary; the Secretary cannot create a prescription from this page |
| Idle, Doctor or Nursing | `Editar`; Primary `Nueva receta`; Secondary `Historial de recetas`; Danger `Eliminar paciente`; return | One filled teal action per viewport |
| Confirm open, `confirm_delete` true | Inline `ConfirmPanel`: the confirmation sentence, Danger `Confirmar eliminación`, Secondary `Cancelar`; the return link remains | Not a modal; focus is not trapped; no route change; Cancelar clears the flag |
| Deleting, `deleting` true | Confirm control busy `Eliminando...`; confirm and cancel disabled; trigger absent | One request at a time; the page does not remove the record locally |
| Delete result | Success: no panel, `navigator.push(Route::Patients {})`, list remounts against the server. Failure: `ErrorText` with the frozen delete-failure line; confirmation stays open; controls re-enable | Success leaves the page; failure keeps the record and URL and allows retry; the raw transport string is never painted |

The history link opens the general `/prescriptions` route with no status and no patient parameter: `Route::Prescriptions { status: None }`. It is not a filtered view of this patient's prescriptions. Prescription or template status is not read, rendered, or changed by this page, so no status transition repaints it.

### 2.4 Web and desktop surfaces

The same component, ids, copy, role gates, and navigation targets serve both builds. The only branch is the existing `#[cfg(feature = "desktop")]` load fallback.

- **Web and desktop load:** `api::fetch_patient` is authoritative. On desktop failure the screen opens the local database with `desktop_local::open_local_db`, calls `desktop_local::list_patients`, and scans for a row whose `id` matches the route id, deserializing the match as `api::Patient`. A hit renders the cached card; a miss or an unreadable database leaves the live error. Each load retries the API first; the fallback is not a background refresh.
- **Offline desktop:** deleting still calls only `api::delete_patient`, shows `Eliminando...`, and on failure returns to the same page with the delete-failure line. There is no local delete, no outbox row, and no fake success; navigation to `/patients` occurs only after `Ok(())`. Below 760px the detail grid becomes one column and `Notas` remains wide; below 640px the page-header action group wraps; DOM order does not change.

## 3. Copy text

All strings are verbatim `es-EC`; punctuation belongs to the string. Chrome copy is owned by `context/screens/Header.md` §3 and is not repeated here.

### 3.1 Page header and actions

| Element | Copy | Target |
|---|---|---|
| Page title (`h1`) | `{patient.full_name}` — authorized clinical content | — |
| Page subtitle | `Ficha del paciente` | — |
| Edit action, Secondary, `can_write_patients` | `Editar` | `/patients/{patient.id}/edit` (`Route::PatientEdit { id }`) |
| Read-only marker, Neutral badge, otherwise | `Solo lectura` | — |
| New prescription action, Primary, `can_create_prescriptions` | `Nueva receta` | `Route::PrescriptionNew { patient_id: Some(patient.id), template_id: None }` |
| History action, Secondary | `Historial de recetas` | `Route::Prescriptions { status: None }` |
| Delete trigger, Danger, `can_write_patients`, confirm closed | `Eliminar paciente` | — |
| Return link, Secondary | `Volver a pacientes` | `Route::Patients {}` |

`Nueva receta` is the page's only Primary action and renders only for Doctor and Nursing. Administrator and Secretary see no Primary. `Eliminar paciente` is Danger, never Primary. Every control is labelled by its visible text; the page adds no `aria-label` or visually hidden copy.

### 3.2 Patient fields

| Element | Copy |
|---|---|
| Field label | `Cédula` |
| Field label | `Fecha de nacimiento` |
| Field label | `Sexo` |
| Field label | `Teléfono` |
| Field label | `Notas` |
| Missing optional value | `—` |
| Birth date display pattern | `DD-MM-AAAA` |
| Sex labels, stored `female` / `male` / `other` / `unspecified` / absent or unrecognized | `Femenino` / `Masculino` / `Otro` / `Prefiere no decir` / `Sin especificar` |

`Notas` is always present as a label, even when its body is `—`; there is no “no notes” helper line. A stored empty string is painted empty, not replaced with invented text. The em dash has no trailing period.

### 3.3 Delete confirmation

| Element | Copy |
|---|---|
| Confirmation message (`ConfirmPanel.message`) | `¿Eliminar este paciente? La ficha se oculta de la búsqueda y queda registrada en auditoría.` |
| Confirm action, Danger, default | `Confirmar eliminación` |
| Confirm action, Danger, busy | `Eliminando...` |
| Cancel action, Secondary, default | `Cancelar` |

The sentence is the shipped confirmation copy; the plan fixes the two action labels and the destructive variant. No toast, success banner, or “patient deleted” string exists because a successful delete navigates to the directory.

### 3.4 Loading, redirect, failure, denial

| Element | Copy |
|---|---|
| Session-check and first-record loading / signed-out redirect | `Cargando...` / `Redirigiendo...` |
| Load failure / delete failure | `No se pudo cargar la ficha del paciente.` / `No se pudo eliminar el paciente.` |
| Denied title (`h1`) and subtitle | `Acceso denegado` / `No tienes permiso para ver esta ficha.` |

There is no success copy. The redirect notice is transient; the load-failure line and `Volver a pacientes` are the only load-error surface. The failure lines are the frozen replacements for raw English API/transport strings (§7).

## 4. Displayed information

### 4.1 What the screen shows

| Element | Source | Value shown |
|---|---|---|
| Page title and subtitle | `patient.full_name` from `api::fetch_patient(&id)` (`api.rs:698`); static | Stored full name, verbatim, plus `Ficha del paciente`; authorized clinical content |
| Cédula | `api::Patient.national_id` (`api.rs:87-97`) | Stored text unchanged; `—` when absent. No formatter, validation, or derived mask |
| Birth date | `patient.birth_date` → `optional_date` (`main.rs:4253`) → `format_date` (`main.rs:4321`) | First 10 characters parsed as `%Y-%m-%d` and painted `%d-%m-%Y`; a parse failure paints the stored text; `None` paints `—` |
| Sexo | `patient.sex` → `sex_label` (`main.rs:4260`) | `female` → `Femenino`; `male` → `Masculino`; `other` → `Otro`; `unspecified` → `Prefiere no decir`; any other value or absent → `Sin especificar` |
| Teléfono | `patient.phone` | Stored text unchanged; `—` when absent. No format normalization or link |
| Notas | `patient.notes` | Stored plain text with authored line breaks preserved (`white-space: pre-wrap`); `—` when absent; wide item |
| Edit and read-only controls | `actor.role` + `Me::can_write_patients()` (`api.rs:61`) | Secondary `/patients/{id}/edit` link for Doctor, Nursing, Secretary; otherwise Neutral `Solo lectura` badge with no handler |
| New prescription | `Me::can_create_prescriptions()` (`api.rs:72`) | Primary for Doctor, Nursing; absent for Secretary and Administrator |
| History link | static | Secondary link to the general `Route::Prescriptions { status: None }`; no patient id carried |
| Delete controls | `Me::can_write_patients()`, `confirm_delete()`, `deleting()`, `api::delete_patient` | Danger trigger, inline confirmation, and busy copy; absent for Administrator; no local mutation before `Ok(())` |
| Load failure and session gate | `message` from `api::fetch_patient` error; `checked`, `user`, `actor` | Frozen failure line plus `Volver a pacientes`; otherwise loading, redirect, denial, or content per §2.1; raw error text is never painted |

The page does not display `patient.id`, `patient.created_at`, `patient.updated_at`, the clinic, a prescription count, a photo, a signature, a stamp, an asset, a plan, a price, or a quota. The cédula and birth date are shown as stored patient fields, not as identifiers or age calculations.

### 4.2 Hard rules

- **Server authorization and audit are the boundary.** `service::get_patient` requires a clinic role, derives `clinic_id` from the actor, and calls `db::find_patient_by_id`; `service::soft_delete_patient` requires the doctor, nursing, or secretary write policy. Every detail lookup records `sensitive_read` for entity `patient` with metadata `via: detail` and `found`; the metadata contains no name, cédula, birth date, sex, phone, or notes. Client `can_read_patients`, `can_write_patients`, and `can_create_prescriptions` checks are UX gates only; forging them cannot read another clinic or delete as Administrator.
- **Administrator is read-only.** The role sees demographics and the history link with a `Solo lectura` badge. No `Editar`, no `Nueva receta`, no delete control exists, and the server rejects a patient write even if one is fabricated.
- **One Primary action per viewport, maximum.** `Nueva receta` is the only Primary and appears only for Doctor and Nursing. Administrator and Secretary render zero filled teal actions. `Editar`, `Historial de recetas`, `Volver a pacientes`, and `Cancelar` are Secondary; both delete controls are Danger.
- **Delete is soft, audited, server-confirmed, and not offline-queued.** The only call is `api::delete_patient` → `delete_patient` → `service::soft_delete_patient`, which sets `deleted_at` and `updated_at` and records `delete_patient` in one transaction. No hard delete, cascade, archive toggle, local removal, outbox row, or success toast exists. Navigation to `/patients` happens only after success; failure keeps the record and confirmation open.
- **No new endpoint, table, column, role, permission, field, or route.** The page reuses `/api/patients/:id` for `GET` and `DELETE`, `api::Patient`, and the four shipped roles; it adds no query parameter and no history parameter.
- **PHI stays out of the URL and every non-clinical surface.** The route is exactly `/patients/:id`; the only identifier in the URL is the opaque `patient.id`. Name, cédula, birth date, sex, phone, notes, and diagnosis never enter history, a QR code, a PDF, marketing copy, or audit metadata. The page produces no document, so it has no PDF or QR action.
- **No billing, signature, stamp, photo, or rich text.** D-04 bans price, trial, plan, entitlement, quota, and `sync_locked` copy. D-06 bans signature and stamp imagery; no patient photo or avatar renders. D-21 bans a rich-text editor, formatting toolbar, `contenteditable`, and formatted values; notes are read-only plain text.
- **Read-only demographics, no hidden activity.** The card is a description list; changing a field belongs to `/patients/:id/edit`. `created_at`, `updated_at`, the clinic, and any prescription count are excluded even when available. The history link is a static navigation entry, not an embedded list.
- **Data minimization and cache honesty.** Optional fields paint `—` when `None`; sex paints its label or `Sin especificar`; a malformed birth date paints its stored text; no age or date arithmetic is invented. Long notes are preserved in the wide cell and never clamped or elided. A desktop cache hit is a read-only fallback for a failed API load: the failure line and return link remain, delete still requires the API, and no local write or permission elevation follows.

### 4.3 Dynamic states

| State | Screen behaviour |
|---|---|
| Session check in flight, or signed out | `PageShell` + `LoadingState` `Cargando...` while checking; no request, header, or card. Signed out then replaces with `/login` and shows the transient `Redirigiendo...` notice with no patient data |
| Signed in, no profile role | Denial `PageHeader`: `Acceso denegado` / `No tienes permiso para ver esta ficha.`; no card, actions, or return link |
| Loading record | `LoadingState` `Cargando...`; labels and values absent until the record resolves |
| Loaded, authorized role | Full-name title and five fields. Administrator: `Solo lectura`, history, return; no edit, create, or delete. Secretary: `Editar`, history, delete, return; no `Nueva receta` and no Primary. Doctor or Nursing: same card plus `Editar`, one Primary `Nueva receta`, `Historial de recetas`, delete, return |
| Load error, web or desktop without a cache match | Error line and `Volver a pacientes`; no fields and no unrelated patient substituted |
| Load error, desktop with matching cache | Cached fields render read-only; error line and return stay visible; no edit inputs or local delete |
| Not found (404) | Same load-error surface; no separate empty-state copy because the client cannot distinguish the error classes |
| Optional values absent | Every label still renders; its value is `—`; sex uses `Sin especificar`; this is not an `EmptyState` |
| Long notes | Wide `Notas` value keeps authored line breaks, wraps within the card, and is not clamped; page height grows |
| Delete idle or confirmation open | Danger `Eliminar paciente`; opening replaces it with the inline `ConfirmPanel` sentence and controls; no modal, focus trap, or route change |
| Delete busy | Confirm label `Eliminando...`; confirm and cancel disabled; one request in flight |
| Delete success or failure | Success: no toast, `navigator.push` to `/patients`, the detail unmounts. Failure: delete-failure `ErrorText`, confirmation stays open, record and URL unchanged |
| Prescription or template status, pending sync, failed outbox | Not read and not rendered; no transition repaints this page. The navbar owns FR-043 operation state and `/prescriptions` owns those filters |

## 5. Style

Every value is a `context/Design.md` token. The page adds no color, font, icon package, CDN, webfont, gradient, or shadow.

### 5.1 Color

| Use | Value | Token |
|---|---|---|
| Page canvas, card surface, Primary text | `#ffffff` | `--color-paper` |
| Page title, field values; Secondary emphasis and focus ring | `#061d29` / `#006e75` | `--color-ink` / `--color-deep-teal` |
| Subtitle, field labels, return text, badge text; hairline borders and separators | `#425d6d` / `#e5e7eb` | `--color-mist` / `--color-hairline` |
| Confirmation panel surface; failure text and delete label; Danger border and hover fill | `#fffded` / `#4d0037` / `#ff492c` / `#ffc3e6` | `--color-cream` / `--color-plum` / `--color-ember-ring` / `--color-blossom` |
| Primary hover fill; focus halo | `#0b978e` / `rgba(185, 255, 232, 0.55)` | `--color-bright-teal` / `--color-mint-glass`, 55% alpha |

Never `#000000`. Never a chromatic border except the Danger control's Ember Ring outline: the card, confirm panel, and badge use the hairline only. Body text is Ink on Paper; labels are Mist on Paper; failure text is Plum on Paper. Measured WCAG ratios: Ink on Paper 17.28; Mist on Paper 6.96; Paper on Deep Teal 6.02; Plum on Paper 15.32; Ink on Cream 16.89; Mist on Cream 6.80; Plum on Blossom 10.36; Ember Ring on Paper 3.36 for the non-text danger boundary while Plum text is 15.32. The focus ring is Deep Teal on Paper 6.02; its mint halo is supplemental, never the only signal.

### 5.2 Typography

| Element | Family | Weight | Size / line-height |
|---|---|---|---|
| Page title (`h1`) | `--font-mint` | 500 | 22px / 1.22, Ink |
| Page subtitle and detail label (`dt`) | `--font-mint` | 400 | 14px / 1.5, Mist |
| Detail value (`dd`) and confirmation message | `--font-mint` | 400 | 16px / 1.5, Ink |
| Buttons and links | `--font-mint` | 500 | 16px / 1.22 |
| Read-only badge | `--font-mint` | 500 | 13px / 1.22, Mist |
| Failure line | `--font-mint` | 400 | 14px / 1.5, Plum |

`--font-grenette` never appears on this page: it is display-only at 46px and above and is banned from app chrome and page titles. The em dash sits in the value style, not as a separate caption.

### 5.3 Geometry

- Base unit 4px. `PageShell` padding: 48px top, 80px bottom, 24px inline; `.ui-container` max 1200px, centered; `.ui-stack` gap 24px. Page header: flex, space-between, align start, wrap, gap 16px; title-to-subtitle gap 4px; action group flex-wrap, center, gap 12px.
- Identity card: Paper, 1px hairline, 24px radius, 24px padding. `DetailGrid`: two equal columns, 16px gap. Each `DetailItem`: label above value, 4px gap. `Notas` is wide and spans both columns; it uses `.ui-notes` with `white-space: pre-wrap`.
- Actions: `.ui-actions` gap 12px, wraps, 16px top margin. Buttons: 8px radius, `12px 24px` padding, min-height 44px, one line, no shadow. The Danger trigger is transparent with a 1px Ember Ring border and Plum text; hover fills Blossom. Confirm panel: Cream, 1px hairline, 24px radius, 20px padding; message first, actions 16px below; inline flow with no backdrop.
- Spacing rhythm uses 4/8/12/16/20/24/32/48/80. Responsive: detail grid drops to one column at 760px; header stacks and actions wrap at 640px. Flat elevation only: paper-on-paper contrast plus hairlines; no drop shadow.

### 5.4 Iconography and imagery

`ErrorText` carries one 20px Alert line glyph at 1.5px stroke in Plum; it is the page's only icon. Text links and buttons carry no icons. No avatar, patient photo, document mock, A4 sheet, QR, signature, stamp, logo, or illustration renders inside the page. The navbar mark and sync dot are chrome owned by `context/screens/Header.md` §5.4.

## 6. Accessibility

- The navbar is the banner landmark (`Header.md` §6). `PageShell` renders a `div`, not a `main`, so this page adds no landmark of its own. Exactly one `h1` is rendered by `PageHeader` per data-bearing state: the full name when loaded, `Acceso denegado` when denied; loading and redirect are transient and carry none. No `h2` renders: the card uses `dl`/`dt`/`dd`, not headings.
- Tab order: navbar chrome → `Editar` when present → `Nueva receta` when present → `Historial de recetas` → `Eliminar paciente` or, when open, `Confirmar eliminación`, then `Cancelar` → `Volver a pacientes`. Fields and the read-only badge are not focusable; no hidden control sits in tab order.
- Focus: `1px solid var(--color-deep-teal)`, `outline-offset: 2px`, halo `box-shadow: 0 0 0 3px rgba(185, 255, 232, 0.55)`. Hover and focus are independent; no style is removed without a replacement. The inline confirm never traps focus.
- The title names the page; each `dt` labels its adjacent `dd`, so read-only text needs no input label or `aria-describedby`. `LoadingState` carries `role="status"` and `aria-live="polite"`; load and delete failures are one `ErrorText` with `role="alert"`, and the sentence, not the Alert glyph, carries the meaning.
- Busy delete changes the confirm control to `Eliminando...` and disables both delete actions; no second submit is possible. No toast or success live region exists because success is navigation. The read-only state is a text badge, not colour alone. No table renders, so no `caption` or `th scope` is promised; the description list is read in label/value order.
- Notes keep authored line breaks and extend the page instead of clipping or creating a keyboard-only scroll region. Contrast is AA for text and non-text boundaries: Ink on Paper 17.28; Mist on Paper 6.96; Paper on Deep Teal 6.02; Plum on Paper 15.32; Ink on Cream 16.89; Mist on Cream 6.80; Plum on Blossom 10.36; Ember Ring on Paper 3.36. Keyboard and touch use native `a`/`button` labels, 44px minimum height, 12px gaps, and no hover-only action. Only the spinner animates, and `prefers-reduced-motion: reduce` stops it.

## 7. Implementation notes

- Shipped `fn PatientDetail(id: String)` is at `persona-sanus/src/main.rs:3128`, `#[component]` at `:3127`, body through `:3317`; `submit_edit_patient` starts at `:3319`, and the route variant is `main.rs:62-63` under the never-closed `#[layout(Navbar)]` at `:51`. The markup is a bare `div#patients-page` with `.page-head` (`h4` full name, `p` `Ficha del paciente`), a `.badge` `Solo lectura` branch, `.detail-grid` of five `.field` rows, `.actions` for `Nueva receta` and `Historial de recetas`, raw `.error` for `message`, `.notice` delete confirmation, and a final `.actions` return `Link`; CSS is at `assets/main.css:292-296`, `:298-308`, `:386-409`, `:417-419`, `:452-457`, `:471-477`, `:479-490`, and `:502-510`.
- Plan step 13 and the Page adoption row `PatientDetail` (`UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md:222`) replace that markup with `PageShell id="patients-page"`; `PageHeader title=patient.full_name subtitle="Ficha del paciente"`; header Secondary `Editar` when `can_write_patients` else Neutral badge `Solo lectura`; Primary `Nueva receta` when `can_create_prescriptions` and none otherwise; Secondary `Historial de recetas`; `Card` + `DetailGrid` of five `DetailItem`s; delete `ConfirmPanel` with Danger `Eliminar paciente` and confirm `Confirmar eliminación`; `ErrorText`. Adopt `Badge tone=Neutral`, `ActionRow`, and `LoadingState label="Cargando..."`; preserve `confirm_delete`, `deleting`, and the delete API.
- Preserve the shipped effect (`main.rs:3139-3174`) and its authorization branches: return while `!checked()`; replace to `Route::AuthPage {}` with no `user`; call `api::fetch_patient(&load_id)` once; set `patient` on success. `profile.can_read_patients()` gates the card, `profile.can_write_patients()` gates `Editar` and delete, and `profile.can_create_prescriptions()` gates `Nueva receta`; the server still runs `require_patient_read`/`require_patient_write`. Do not change the fetch argument, add a second fetch, or add query state.
- Preserve the desktop fallback exactly (`main.rs:3152-3167`): `#[cfg(feature = "desktop")]`, `desktop_local::open_local_db`, the unfiltered `desktop_local::list_patients`, a scan by `id`, and `serde_json::from_value::<api::Patient>`. Do not add a local helper, search argument, or write. Target keeps the failure surface visible on a cache hit; shipped clears it by setting `message` only when no patient resolved. That visibility change does not alter the fallback calls.
- Preserve delete and navigation: `api::delete_patient(&patient_id)` runs in `spawn` with `deleting` true and a second-click guard; `Ok(())` pushes `Route::Patients {}`, `Err` clears `deleting` and stores the message. No local delete, outbox, toast, or optimistic removal. Targets: `Editar` → `Route::PatientEdit { id: patient.id.clone() }`; `Nueva receta` → `Route::PrescriptionNew { patient_id: Some(patient.id.clone()), template_id: None }`; `Historial de recetas` → `Route::Prescriptions { status: None }`; `Volver a pacientes` and delete success → `Route::Patients {}`. **The history link stays general**: no patient id or filter is appended, and patient filtering belongs to `/prescriptions`, not this page.
- Copy deltas: paint `No se pudo cargar la ficha del paciente.` instead of the raw `api::fetch_patient`/transport string; paint `No se pudo eliminar el paciente.` instead of the raw delete error; replace the shipped no-role denial body `Necesitas un rol en la clínica para consultar pacientes.` with `No tienes permiso para ver esta ficha.` inside the `Acceso denegado` page header. Keep `Ficha del paciente`, `Editar`, `Solo lectura`, `Nueva receta`, `Historial de recetas`, `Eliminar paciente`, the confirmation sentence, `Confirmar eliminación`, `Cancelar`, `Eliminando...`, `Volver a pacientes`, `Cargando...`, and `Redirigiendo...` verbatim.
- **Plan confirm copy versus shipped.** The plan supplies the Danger trigger label `Eliminar paciente` and confirm label `Confirmar eliminación`; the shipped component supplies the confirmation sentence `¿Eliminar este paciente? La ficha se oculta de la búsqueda y queda registrada en auditoría.` and the busy label `Eliminando...`. Keep all four; do not shorten the sentence into a modal title or add a success state. No excluded field is added: no `created_at`, `updated_at`, `id`, clinic, prescription count, photo, signature, stamp, billing, quota, PDF, QR, or edit input; the edit route remains the only writer.
- Verification commands, in order: `cd persona-sanus && cargo check`; `dx build --release --fullstack`; `caveman-blocks run persona-smoke --dir persona-sanus --port 8091`; `python3 .blocks/markdown-table-check.py --path context/screens/PatientDetail.md --strict`.
- Do not add: an inline edit form, a hard delete, an archive toggle, a cascade, a delete endpoint beyond `api::delete_patient`, a local delete, an outbox row, a success toast, a modal, a focus trap, created/updated timestamps, the patient id, clinic identity, a prescription count, an embedded prescription list, a patient photo or avatar, a signature or stamp image, an asset preview, billing/price/trial/plan/quota/`sync_locked` copy, a PDF or QR action, a rich-text or `contenteditable` surface, a patient-filtered history parameter, a second history link, a second Primary action, or an `Editar`/delete control for the read-only Administrator.

## 8. Spec references

| Spec | Connection |
|---|---|
| `specs/15-O-ui-tokens-actions-and-formatting.md` | `15-O` supplies the tokens, `Button`, `ButtonLink`, and format helpers (`format_date`, `sex_label`) the ficha renders. |
| `specs/16-P-ui-forms-feedback-and-layout.md` | `16-P` supplies `DetailGrid`, `Badge`, `ConfirmPanel`, and `ErrorText` the detail regions use. |
| `specs/21-U-ui-dashboard-settings-and-patients.md` | `21-U` adopts `PatientDetail`: header action order, demographics `Card`, inline delete confirm, one Primary `Nueva receta`. |
| `specs/25-Y-ui-legacy-css-cleanup-and-docs.md` | `25-Y` removes legacy page-ID selectors so the adopted ficha runs only on `.ui-*` classes. |
