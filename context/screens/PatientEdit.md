# PersonaSanus — PatientEdit (`/patients/:id/edit`)

| Field | Value |
|---|---|
| Source | `context/sitemap.md` §1.2 (`/patients/:id/edit` — patient edit, Doctor/Nursing/Secretary, update patient fields, audited under §6 sensitive read), §2 (patient routes by role), §3 flows 1 and 5, §4 M0; `context/PRD.md` FR-001, FR-002, §6 `patients` and `audit_log`, §9 sensitive-read audit, D-04, D-06, D-21; `context/Design.md` (Paper, Ink, Mist, Hairline, Deep Teal, Bright Teal, Blossom, Cream, Plum; 24px cards; 8px controls; flat elevation; Form Input and Textarea; Primary Action Button; Outlined Secondary Button); `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` step 13 and Page adoption row `PatientEdit` (line 223), primitives `PageShell`, `PageHeader`, `Card`, `FormGrid`, `TextField`, `SelectField`, `TextAreaField`, `ActionRow`, `Button`, `ButtonLink`, `ErrorText`, `LoadingState`, `Notice`; `context/screens/INDEX.md` §2.3 |
| Component | `PatientEdit` — `persona-sanus/src/main.rs:3358` (`#[component]` at `:3357`, body `:3358-3564`); `submit_edit_patient` at `:3319-3355`; calls `api::fetch_patient` (`api.rs:698`), `api::update_patient` (`api.rs:755`), `update_patient_dispatch` (`main.rs:959`), and `desktop_local::update_local_patient` (`desktop_local.rs:1718`); composes planned `persona-sanus/src/ui/` primitives, a directory not yet shipped |
| Route | `/patients/:id/edit` (`Route::PatientEdit { id: String }`, `main.rs:64-65`) — authenticated shell; signed-out visitors are replaced with `/login` (`Route::AuthPage {}`); Doctor, Nursing, and Secretary only; no query parameters; `id` is the only URL value and is the opaque patient id |
| Layout | Inside `#[layout(Navbar)]` (opens at `main.rs:51`, never closed). Root `PageShell id="patients-page"` → `div#patients-page.ui-page > .ui-container.ui-stack`; Paper canvas; 48px top, 80px bottom, 24px inline; 24px stack gap; no full-bleed band, sidebar, or tabs |
| Locale | `es-EC` on every string; `cédula` keeps Ecuador spelling; dates present as `DD-MM-AAAA`; 24-hour time. The native birth-date control stores and submits `AAAA-MM-DD`; no age is derived or shown |
| Status | ✅ Spec complete (sections 1–7). Route status: ✅ — sitemap §1.2: “Patient edit · Doctor, Nursing, Secretary · FR-001 · Update patient fields. Audited (§6 sensitive-read audit).” Shipped: one preloaded six-field form, name and birth-date client validation, exact-cédula `409` conflict, online PUT, desktop local update plus outbox, detail navigation, server-side load and update auditing. Target: step 13 adoption of `PageShell`, `PageHeader`, `Card`, `FormGrid`, the field primitives, `ErrorText`, and `ActionRow`; no new route, endpoint, table, column, role, field, or permission |

The edit form for one existing patient in the caller's clinic. It changes the same six demographic fields that `/patients/new` creates and never touches identity, clinic, audit, or prescription data. This file freezes region order, copy, geometry, and states for the shipped component; it introduces no route, endpoint, table, column, role, permission, or field.

## 1. Purpose
The screen performs four jobs, in order:

1. **Load one record.** Read the patient identified by the opaque route id through `api::fetch_patient`, and populate the six editable signals from the returned `api::Patient` exactly once.
2. **Show the editable identity.** Render full name, cédula, birth date, sex, phone, and notes in the shipped field order, with the current value selected or filled.
3. **Validate and save through the existing dispatch.** Reject a trimmed-empty name and a non-empty birth date that does not parse as `AAAA-MM-DD`; then use `update_patient_dispatch` for the online `PUT /api/patients/{id}` or the desktop local update plus outbox row. A cédula held by another patient is a hard `409`, not an override.
4. **Return to the record.** On success navigate to `/patients/{patient.id}`. `Cancelar` navigates back to the same detail route without writing.

It never renders:

- Patient PHI outside the authorized form and its returned error strings: no real name, cédula, birth date, phone, or notes in this spec, in the URL, in a toast, or in audit metadata. The route carries only `patient.id`; the fields are clinical content for Doctor, Nursing, and Secretary only.
- Billing, price, trial, plan, entitlement, quota, `sync_locked`, or subscription copy (D-04); a signature, stamp, upload, photo, avatar, logo, QR, or document preview (D-06); rich text, WYSIWYG, `contenteditable`, HTML, or markdown rendering (D-21).
- A patient id, clinic id, `created_at`, `updated_at`, `created_by`, audit action, or audit row as an editable field; a create, delete, archive, search, list, history, or template surface; a modal, toast, autosave, or draft store.
- A second filled teal action. The only Primary in any viewport is `Guardar cambios`; `Cancelar` and `Ver ficha` are Secondary, and the denied state renders no Primary.
## 2. Structure
One Paper column in a 24px stack. The page header mounts first, the form card second, feedback third, and the page action row last. Loading, redirect, and denial replace the card and the action row but keep the root id. The birth-date and cédula inputs are native controls, not custom widgets.

| Order | Region | id / class | Band tone | Contents |
|---|---|---|---|---|
| — | Navbar (layout) | `header.ui-navbar` → `.ui-navbar-inner` | — | Owned by `context/screens/Header.md` §2–§3; renders above the page unchanged; no screen copy repeated here |
| 1 | Page shell | `PageShell id="patients-page"` → `div#patients-page.ui-page` > `.ui-container.ui-stack` | Paper `#ffffff` | 48px top padding, 80px bottom, 24px inline; container max 1200px; 24px vertical stack gap |
| 2 | Page header | `PageHeader` → `header.ui-page-head` · `h1.ui-page-title` · `p.ui-page-subtitle` · `.ui-page-head-actions` | Paper | Title `Editar paciente`; subtitle `Los cambios se auditan en la ficha del paciente.`; one Secondary `ButtonLink` `Cancelar` → `/patients/{id}` |
| 3 | Form card | `Card` → `div.ui-card` | Paper, 1px hairline, 24px radius, 24px padding | `FormGrid columns=Two` with six fields in §3.1, in shipped order; no card heading; notes spans both columns |
| 4 | Feedback | `ErrorText` → `.ui-error` | Paper | The current `message` signal, client validation, server `409`, dispatch error, or load failure; `role="alert"`; Alert glyph; rendered only when non-empty |
| 5 | Action row | `ActionRow` → `div.ui-actions` | Paper | Exactly one Primary `Button` `Guardar cambios`; busy label `Guardando...`; `disabled=true` and `aria-busy=true` while saving |
| 6 | Session gate | `LoadingState` inside `PageShell` | Paper | `Cargando...` while `checked()` is false; no form, no fetch outcome painted |
| 7 | Actor gate | `LoadingState` inside `PageShell` | Paper | `Redirigiendo...` while `actor()` is `None`; signed-out effect replaces the route with `/login` |
| 8 | Denied block | `Notice tone=Error` + Secondary `ButtonLink` | Blossom `#ffc3e6`, Plum text | Replaces orders 3–5: title `Acceso denegado`, body `Solo doctor, enfermería y secretaría pueden editar pacientes.`, action `Ver ficha` → `/patients/{id}` |

Orders 6–8 are mutually exclusive with orders 3–5. Order 4 sits below order 3 and above order 5 whenever the card renders. Nothing is horizontally scrollable.
### 2.1 Session and role
`checked` is the shell session-check signal, `user` is `Signal<Option<SessionUser>>`, and `actor` is `Signal<Option<Me>>`. `Me::can_write_patients()` is true only for `doctor`, `nursing`, and `secretary` (`api.rs:61-66`); it is false for `administrator` and for a profile with no role. The effect replaces `/login` only when `user()` is `None` after the check.

| State | What changes | What stays |
|---|---|---|
| Check in flight (`!checked()`) | `PageShell` plus `LoadingState` `Cargando...`; no header, card, fields, Primary, or fetch | Root `patients-page`; Paper band; 24px stack |
| Checked, no session, or `actor() == None` | The effect calls `navigator.replace(Route::AuthPage {})` for no session; the render is the actor gate `Redirigiendo...` | Root id; no form paints |
| Role `administrator` or `null` | `Notice tone=Error` denial with the body and Secondary `Ver ficha` to `/patients/{id}` | Root id; no Card, no fields, no Primary |
| `doctor`, `nursing`, `secretary` | Header, Card, six fields, one Primary | Root id; same route, field order, and save dispatch |

The role gate is UX only. `service::update_patient` calls `actor.require_patient_write()` before any write (`service.rs:1016`), and the server rejects the Administrator (`service.rs:90-95`). The client sends no role or clinic id.
### 2.2 Loading and load failure
The one-shot effect reads `initialized`, `checked`, and `user`; it returns while `initialized` is true or the session check is incomplete. On the first allowed pass it spawns `api::fetch_patient(id)`, fills the six signals, and sets `initialized` true. A successful load never re-runs the request for that mount.

| Load state | Screen |
|---|---|
| Session check in flight | `PageShell` + `LoadingState` `Cargando...`; no form |
| First patient fetch in flight | Page header, then the `Cargando...` line; card fields are empty but not painted as a record |
| Fetch success | Populated Card and Action Row; loading line gone |
| Fetch failure, no matching cache row (web or desktop), or no decodable row | `ErrorText` carries the returned message; the empty Card and enabled Primary remain; `initialized` stays false |
| Fetch failure, desktop with a matching cache row | Six signals come from `desktop_local::list_patients`; loading line gone; no error shown; the read is not server-audited |

The page never claims an empty failed load is the stored record. Only the empty-name check stops a blank name write before the request; optional fields can be empty by design.
### 2.3 Surfaces and offline save
The same component, labels, role gate, and validation run on web and desktop. The branch is in `update_patient_dispatch` (`main.rs:959-979`).

- **Web:** `api::update_patient` is the only save path; `PUT /api/patients/{id}` goes to the service layer. A failure becomes the `message` text; there is no local database or outbox.
- **Desktop reachable:** the same API call. The server updates the row, records the write audit, and returns the patient; navigation targets the returned `patient.id`.
- **Desktop unreachable:** `update_patient_offline` calls `desktop_local::update_local_patient` with `fullName`, `nationalId`, `birthDate`, `sex`, `phone`, `notes`, and the actor profile id, then returns a locally assembled `Patient`. The local row is updated, an `update_patient` outbox entry is queued, sync is reprobed, and navigation targets that local id.
- **No server claim while offline:** the page never says the server saved, audited, or accepted the change. The server write and its `update_patient` audit happen only when a later sync uploads the outbox row. The local success path is the shipped behavior; §7 preserves it. A cache miss returns `Paciente no encontrado en la caché local.` in `ErrorText` and does not navigate.
### 2.4 Cancel and long notes
`Cancelar` is a router link to `/patients/{id}`, not a reset or a `history.back()` call. It writes nothing, discards the typed field signals with the unmount, and leaves the detail page to display the stored record. It is never duplicated at the bottom of the card.

Notes are a plain textarea. A long value wraps inside the control or scrolls it vertically; `resize: vertical` allows the operator to grow it. The value never widens the page, becomes HTML, or appears rendered elsewhere on this screen.
## 3. Copy text
All strings are verbatim `es-EC`; punctuation belongs to the copy. Navbar strings are owned by `context/screens/Header.md` §3 and are not repeated. No label is English.
### 3.1 Page and fields
| Element | Copy |
|---|---|
| Page title (`h1`) | `Editar paciente` |
| Page subtitle | `Los cambios se auditan en la ficha del paciente.` |
| Header action | `Cancelar` |
| Name label | `Nombres y apellidos` |
| National id label | `Cédula` |
| Birth date label | `Fecha de nacimiento` |
| Sex label | `Sexo` |
| Sex empty option | `Sin especificar` |
| Sex options | `Femenino`; `Masculino`; `Otro`; `Prefiere no decir` |
| Phone label | `Teléfono` |
| Notes label | `Notas` |

The six labels wrap native controls. There are no placeholders, required markers, helper hints, or autocomplete adornments on this screen.
### 3.2 Actions and feedback
| Element | Copy |
|---|---|
| Primary (default) | `Guardar cambios` |
| Primary (busy) | `Guardando...` |
| Empty-name validation | `Ingresa el nombre del paciente.` |
| Invalid non-empty birth date | `Ingresa una fecha de nacimiento válida.` |
| Exact cédula conflict (`409`) | `Ya existe un paciente con esta cédula.` |
| Session gate | `Cargando...` |
| Actor gate | `Redirigiendo...` |
| Denied title | `Acceso denegado` |
| Denied body | `Solo doctor, enfermería y secretaría pueden editar pacientes.` |
| Denied action | `Ver ficha` |
| Offline dispatch failure | `Sin conexión.` |
| Offline cache miss | `Paciente no encontrado en la caché local.` |
| Malformed patient response (target) | `No se pudo leer el paciente.` |

Server and local error strings other than the frozen conflict and offline strings render verbatim in `ErrorText` and are not page copy. The malformed-response row is a target replacement for the shipped English client fallback `Could not read the patient.`; §7 records it.
## 4. Displayed information
### 4.1 What the screen shows
| Element | Source | Value shown |
|---|---|---|
| Page title and subtitle, Cancel | static | `Editar paciente`, `Los cambios se auditan en la ficha del paciente.`; Cancel → `/patients/{patient.id}`, no write |
| Name | `full_name` signal ← `api::Patient.full_name`; `oninput` | loaded name, then exactly the typed text; server trim only on save |
| Cédula | `national_id` signal ← `Patient.national_id: Option<String>` | loaded cédula or empty; plain text; no checksum, mask, or country control |
| Birth date | `birth_date` signal ← `Patient.birth_date: Option<String>`; native `input[type=date]` | loaded date as `AAAA-MM-DD` in the control; localized display `DD-MM-AAAA`; optional |
| Sex | `sex` signal ← `Patient.sex: Option<String>`; select | `""`, `female`, `male`, `other`, or `unspecified`; label from §3.1; empty maps to `Sin especificar` |
| Phone and notes | `phone`/`notes` signals ← `Patient.phone`/`Patient.notes`; `input[type=tel]` and textarea | loaded value or empty; exactly the typed text; notes wrap and scroll; never HTML |
| Submitted form | `api::PatientForm` built by `form()` | the six current values in field order; no `id`, clinic, created, or updated member |
| First load | `api::fetch_patient(&patient_id)` (`api.rs:698-701`) | `GET /api/patients/{id}` once per mount behind `initialized`; `Patient.id` stays the route id |
| Desktop cache fallback | `desktop_local::open_local_db` → `desktop_local::list_patients` (`desktop_local.rs:720`) | cached row whose `id` equals the route id, deserialized as `api::Patient`; no search or clinic argument |
| Save dispatch and online write | `update_patient_dispatch(..., Some(profile))`; `PUT /api/patients/{id}` (`api.rs:755-758`) | reachable: `api::update_patient(id, &form)` with six-field `patient_body`; response `api::Patient`; no clinic or actor field |
| Offline write | `desktop_local::update_local_patient` (`desktop_local.rs:1718-1768`) | updates the local row and enqueues the `update_patient` outbox entry with the profile id |
| Saving and message | `Signal<bool>` `saving` (`main.rs:3371`); `Signal<String>` `message` | Primary disabled and relabeled `Guardando...`; second click returns early; one `ErrorText` line; cleared at each valid submit |
| Load and write audit | `service::get_patient` (`service.rs:1139-1164`); `service::update_patient` (`:1079-1087`) | load: `sensitive_read`/`patient`, patient id, `{"via":"detail","found":…}`; write: `update_patient`/`patient`, patient id, no metadata |
| Success target | returned `Patient.id` | `/patients/{id}`; no toast, banner, or modal |

The load audit is recorded only when the online `GET` reaches the service. The desktop cache fallback is a local read and produces no server audit row. The UI never fetches, prints, or links the audit log.
### 4.2 Hard rules
- Authorization is server-side. `Me::can_write_patients()` suppresses the form, but `service::update_patient` independently calls `actor.require_patient_write()`; client suppression is not the boundary. Clinic scoping is derived from the actor, and the request carries no clinic or role field.
- The route id is opaque and is the only URL value. No patient field, diagnosis, or audit metadata enters the path, query string, fragment, page title, QR, logs, or marketing copy; no `/patients/by-name/...` route exists.
- The online edit load is a §6 sensitive read. `service::get_patient` (`service.rs:1139`) wraps the lookup in a transaction and calls `record_audit` with action `sensitive_read`, entity `patient`, the patient id, and metadata `{"via":"detail","found":…}` (`service.rs:1151-1159`). That shipped action is the record; there is no invented page-local action. The desktop `list_patients` fallback is local and cannot append to the server audit log, so the first online read is the first server-audited access.
- The write is audited once as `update_patient`: `service::update_patient` records entity `patient`, the patient id, and no metadata (`service.rs:1079-1087`) inside the same immediate transaction, after the update. The update path does not emit `sensitive_read`; this screen renders neither action, nor any actor, timestamp, or audit link.
- One Primary maximum: `Guardar cambios`. `Cancelar`, `Ver ficha`, and every denial control are Secondary. No second teal-filled control, modal confirm, toast, autosave, or draft store ships here.
- No billing, price, trial, plan, quota, `sync_locked`, or subscription copy (D-04); no signature, stamp, or image upload (D-06); no rich text, WYSIWYG, `contenteditable`, HTML, or markdown rendering (D-21).
- The form edits only `full_name`, `national_id`, `birth_date`, `sex`, `phone`, and `notes`; never `id`, clinic, `created_at`, `updated_at`, `created_by`, audit fields, photos, signatures, or stamps.
- A cédula held by another patient in the same clinic is a hard `409` (`DuplicateNationalId`); it is not confirmable here, unlike the create screen's name warning. Do not add an “ignore conflict” path. A blank cédula is allowed and is not checked against other blanks.
- Birth date is optional and checked only for non-empty parseability; the client derives no age, enforces no range, and rejects no old/recent date. The server repeats the parse and owns any stronger rule. The name check is `trim().is_empty()`; the server trims and owns length. Notes are plain text, never rendered here, and carry no clinical decision support, diagnosis, medication, prescription, or template content.
- Offline desktop success is local only. The page must not say the server saved, the audit was written, or another device can see the change; the outbox row is invisible here and the navbar sync indicator owns it. The page adds no route, endpoint, query parameter, table, column, role, permission, or field.
### 4.3 Dynamic states
| State | Screen behaviour |
|---|---|
| Session check in flight | `PageShell` + `LoadingState` `Cargando...`; no header, card, fields, or Primary |
| No session | Effect replaces with `/login`; the render never becomes the form |
| Session with unresolved actor | `LoadingState` `Redirigiendo...` |
| No role or Administrator | Denial Notice `Acceso denegado` plus the body and `Ver ficha`; no Card; the server rejects the write even if the client is bypassed |
| Doctor, Nursing, Secretary | Header, Card, six fields, one Primary |
| Loading the patient | Header, then `Cargando...`; fields are empty but the record is not yet presented as loaded |
| Loaded, populated | Name and five optional values from `api::Patient`; sex displays the stored value's label; empty optionals are blank |
| Load fails, web or desktop with no cache match | `ErrorText` with the returned message; the empty Card and enabled Primary remain |
| Load fails, desktop with cache match | Cache values populate the fields; no error line; the read is not server-audited |
| Empty name on submit | `Ingresa el nombre del paciente.`; no dispatch; values stay; no navigation |
| Invalid non-empty birth date on submit | `Ingresa una fecha de nacimiento válida.`; no dispatch; values stay |
| Cédula held by another patient | Server `409`; `ErrorText` shows `Ya existe un paciente con esta cédula.`; no navigation, no override control, values stay |
| Valid submit, saving | Primary label `Guardando...`, disabled, `aria-busy=true`; second click is a no-op; fields stay visible and editable |
| Save failure, online or offline desktop | `saving` false; `ErrorText` shows the returned message or local cache miss; values stay; Primary re-enabled; no fake server success |
| Offline desktop save success | Local row updated, outbox row queued, sync reprobed, then navigation to the local patient's detail |
| Online save success | Server row updated and audited, then navigation to the returned patient's detail; the busy label disappears only because the edit screen unmounted |
| Cancel | Router link to `/patients/{id}`; no request, no local write, typed values discarded |
| Session expires mid-submit | Server rejects; the error surfaces; the next guarded navigation returns to `/login`; no optimistic server row is shown |
| 320px width | One column, labels above controls, notes full width, Primary content-width, no horizontal scroll |

`initialized` is a one-shot load guard, not a save guard: a save failure leaves it true and never refetches the record. The typed values are the source of truth after load, even if the server or cache row changed elsewhere.
## 5. Style
Every value is a `context/Design.md` token declared in `persona-sanus/assets/main.css` `:root`. No new color, font, icon package, CDN, or webfont. The planned `src/ui/` primitives are not shipped; this section specifies their target output.
### 5.1 Color
| Use | Value | Token |
|---|---|---|
| Page canvas, card surface, control surface | `#ffffff` | `--color-paper` |
| Page title, labels, control values, Primary label | `#061d29` | `--color-ink` |
| Subtitle, hints, Secondary text and border | `#425d6d` | `--color-mist` |
| Card, control, and Notice hairline | `#e5e7eb` | `--color-hairline` |
| Primary fill and focus ring | `#006e75` | `--color-deep-teal` |
| Primary hover | `#0b978e` | `--color-bright-teal` |
| Denied Notice surface | `#ffc3e6` | `--color-blossom` |
| Error text, Alert glyph, denied text | `#4d0037` | `--color-plum` |
| Focus halo | `rgba(185,255,232,0.55)` | mint glass at 55% |

Never `#000000`. Never a chromatic border: every border is the `--color-hairline` hairline. The denied surface is a flat Blossom fill with Plum text, not an outlined or shadowed card. Error text is Plum text, not a red fill.

Measured contrast: Ink on Paper 17.28:1 · Mist on Paper 6.96:1 · Paper on Deep Teal 6.02:1 · Plum on Paper 15.32:1 · Plum on Blossom 10.36:1 · Ink on Cream 16.89:1. AA for body text; AAA where the ratio clears it.
### 5.2 Typography
| Element | Family | Weight | Size / line-height |
|---|---|---|---|
| Page title (`h1`) | `--font-mint` | 500 | 22px / 1.22, Ink |
| Page subtitle | `--font-mint` | 400 | 14px / 1.5, Mist |
| Field label | `--font-mint` | 500 | 14px / 1.5, Ink |
| Input, select, textarea value | `--font-mint` | 400 | 16px / 1.5, Ink |
| Hint or empty optional value | `--font-mint` | 400 | 14px / 1.5, Mist |
| Error text | `--font-mint` | 400 | 14px / 1.5, Plum |
| Button label | `--font-mint` | 500 | 16px / 1.22, Paper on Primary; Mist on Secondary |
| Denied title | `--font-mint` | 500 | 16px / 1.5, Plum |
| Denied body | `--font-mint` | 400 | 14px / 1.5, Plum |

No `--font-grenette` on this screen: it is an application form, not a 46px+ editorial opener.
### 5.3 Geometry
- 4px base. `PageShell`: 48px top, 80px bottom, 24px inline; `.ui-container` max 1200px; `.ui-stack` 24px gap both axes. `PageHeader`: flex row, space-between, wrap, 16px gap, align start; subtitle 4px below the title; `Cancelar` at the action edge.
- `Card`: 24px radius, 1px hairline, 24px padding, Paper, no shadow; the only filled surfaces are the card, the Primary, and the denied Blossom Notice.
- `FormGrid columns=Two`: 2 columns, 16px row and column gap, 1 column at 760px and below; field order name, cédula, birth date, sex, phone, notes; notes spans both columns (`wide`).
- Controls: 8px radius, 1px hairline, 8px 12px padding, 16px / 1.5 text, min-height 44px; native date, tel, select, and textarea; labels 4px above; no placeholder-only field.
- `ErrorText`: 8px top margin, 14px / 1.5, 16px Alert glyph, no border or fill. `ActionRow`: 16px top margin, 12px gap, wraps, Primary content-width. Denied `Notice`: 20px padding, 12px icon gap, 20px glyph, `Ver ficha` follows the body.
- Elevation flat: no card, button, input, or notice shadow; the only ring is the focus halo. Below 760px one column, header actions below the title, notes full width, no horizontal scroll; at 320px labels and controls stay usable and the date value is not clipped. Long notes wrap and scroll inside the textarea; `resize: vertical` allows growth and the page never widens.
### 5.4 Iconography and imagery
Line icons only, 1.5px stroke, 20px default, Ink or Plum. The page uses the Alert glyph in `ErrorText` and the denied `Notice`; it uses no other glyph. No photography, illustration, avatar, patient photo, clinic logo, QR, document preview, signature image, or stamp image appears. The document/QR/logo glyphs allowed elsewhere are not used here.
## 6. Accessibility
- Landmarks and headings: the shell navbar is the banner (`Header.md`). `PageShell` renders `div#patients-page`, not `<main>`; do not invent a second banner, nav or form landmark. Exactly one `h1`, `Editar paciente`, from `PageHeader`; `Card` has no `h2`, and the denied title stays a paragraph.
- Tab order: only `Cancelar` → `Nombres y apellidos` → `Cédula` → `Fecha de nacimiento` → `Sexo` → `Teléfono` → `Notas` → `Guardar cambios`; denied exposes `Ver ficha` only. Loading and redirect expose no page control beyond the navbar.
- Focus: `outline: 1px solid var(--color-deep-teal); outline-offset: 2px; box-shadow: 0 0 0 3px rgba(185,255,232,0.55);` on every field, link, and button. Labels wrap visible controls; the sex empty value is `Sin especificar`, not its accessible name.
- Errors: `ErrorText` is `role="alert"` with the Alert icon; the name and birth-date messages bind to their controls through `aria-describedby` or name both fields, and `409` is a page-level alert. Do not mark every field `aria-invalid` for one failure. Busy sets `disabled` and `aria-busy="true"` with the `Guardando...` label; focus does not move on submit.
- Status is never colour-only: the error is text plus Plum, and the denied state says `Acceso denegado` with a reason. No green/red dot renders.
- Native controls only, 44px targets; date picker is keyboard-openable; no drag-only or hover-only action. The shared spinner stops under `prefers-reduced-motion: reduce`. At 200% zoom and 320px width: one column, labels above values, no horizontal scroll, no clipped date value, Primary visible.
- Contrast: Ink/Paper 17.28:1, Mist/Paper 6.96:1, Paper/Deep Teal 6.02:1, Plum/Paper 15.32:1, Plum/Blossom 10.36:1, Ink/Cream 16.89:1 — AA body text, AAA where stated.
## 7. Implementation notes
The shipped component is `fn PatientEdit()` at `persona-sanus/src/main.rs:3358` (`#[component]` at `:3357`, closing `:3564`); the route is at `:64-65`. It owns `full_name`, `national_id`, `birth_date`, `sex`, `phone`, `notes`, `message`, `saving`, `initialized`, and `patient_id = id.clone()`, plus the `user`, `actor`, `checked`, and `sync` contexts. Its render path is three early returns (`!checked`, `actor() == None`, `!profile.can_write_patients()`), then a raw `div#patients-page` with `div.page-head`, `div.form-grid` and six `label.field` controls, an optional `p.error`, and `div.actions` with one raw `button`.

`submit_edit_patient` (`:3319-3355`) returns while `saving`; validates `form.full_name.trim()` and, when non-empty, `valid_birth_date` (`:4249-4251`, `NaiveDate::parse_from_str` with `%Y-%m-%d`); sets `saving`, clears `message`, and spawns; dispatches `update_patient_dispatch(&patient_id, &form, Some(profile))`; on success clears `saving`, reprobes `sync.set(probe_sync_status().await)`, and pushes `Route::PatientDetail { id: patient.id }`; on failure clears `saving` and sets the message with no navigation. `update_patient_dispatch` (`:959-979`) calls `api::update_patient` when reachable; otherwise desktop `update_patient_offline` (`:927-957`) calls `desktop_local::update_local_patient` with the camelCase six-field JSON plus `profile.profile.id`, and web unreachable returns `Sin conexión.`. `saving` at `:3371` is not declared `mut`, but `Signal<bool>` is a copyable reactive handle: `submit_edit_patient` takes `mut saving` (`:3320`) and calls `saving.set` (`:3339`, `:3344`, `:3350`), the click closure passes the same handle (`:3549`), and the label and `disabled` read it (`:3544`, `:3559`). The busy label is wired correctly and is not a shipped bug to fix.

Sensitive read and write audit, verified in the service:

- Online load: `api::fetch_patient` (`api.rs:698-701`) calls `GET /api/patients/{id}`; `api_server::get_patient` (`:980-988`) calls `service::get_patient`, whose transaction records action `sensitive_read`, entity `patient`, the patient id, and metadata `{"via":"detail","found":…}` (`service.rs:1151-1159`). The desktop `desktop_local::list_patients` fallback is local SQLite with no audit write, so the page looks identical but only the online read is server-audited.
- Write: `service::update_patient` (`service.rs:1010-1093`) checks `require_patient_write`, derives the clinic, cleans the six fields, rejects another patient's national id with `DuplicateNationalId`, updates inside an immediate transaction, and records action `update_patient`, entity `patient`, the patient id, no metadata (`:1079-1087`), then returns the row. The update path has no `sensitive_read` call; do not add one or claim one in UI copy. Offline, the local update and outbox insert are on-device; the `update_patient` audit exists only after a later sync uploads the outbox entry.

Plan step 13 adoption (`UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md:204`, Page row `PatientEdit` at `:223`):

- Root becomes `PageShell id="patients-page"`; the raw page head becomes `PageHeader title="Editar paciente" subtitle="Los cambios se auditan en la ficha del paciente."`, with the shipped `Link.button.secondary` as a Secondary `ButtonLink` `Cancelar` in the header actions, targeting `Route::PatientDetail { id: patient_id.clone() }`.
- The raw `.form-grid` becomes `Card` + `FormGrid columns=Two`; the six controls become `TextField` for name/cédula, `TextField input_type="date"` for birth date, `SelectField` for sex, `TextField input_type="tel"` for phone, and `TextAreaField wide` for notes. Keep the shipped labels, values, `oninput`/`onchange` handlers, and option set; add no placeholder. The optional `p.error` becomes `ErrorText text=message()`; the shipped `.error` Ember Ring color is superseded by the shared ErrorText Plum.
- The `.actions` div becomes `ActionRow`; the raw button becomes the one Primary `Button` with `button_type="submit"` only if the six fields are wrapped in a real form, otherwise `"button"`, `disabled=saving()` or `loading=saving()`, and the label pair from §3.2. Do not add a second Primary or a bottom `Cancelar`.
- The `!checked` and actor-absent gates become `LoadingState` with the §3.2 labels; the denied `div.notice` becomes `Notice tone=Error` with the same title/body and a Secondary `ButtonLink` `Ver ficha` to `/patients/{id}`. Keep the guard method `can_write_patients`.
- Preserve the `initialized` one-shot guard and its placement inside the effect; preserve the six-signal clear/populate sequence and the `Patient` deserialization for the desktop cache fallback; preserve `update_patient_dispatch`, the offline branch, the sync reprobe, and the navigation target. Add no `loaded` signal: the page table does not ask for one.
- Replace the English malformed-response fallback `Could not read the patient.` in `api::fetch_patient` and `api::update_patient` with the frozen `No se pudo leer el paciente.`; do not change the HTTP methods, endpoints, payload keys, or other shared messages.

Verification commands, run from the repo root; none is recorded here as executed:

```sh
cd persona-sanus && cargo check
dx build --release --fullstack
caveman-blocks run persona-smoke --dir persona-sanus --port 8091
python3 .blocks/markdown-table-check.py --path context/screens/PatientEdit.md --strict
```

- Do not add: a new route, endpoint, query parameter, table, column, role, permission, or field; editable `id`, clinic, timestamps, audit, photo, signature, or stamp controls.
- Do not add: a second filled action, a bottom `Cancelar`, a modal, a toast, autosave, a client draft store, or optimistic detail content.
- Do not add: rich text, WYSIWYG, a formatting toolbar, `contenteditable`, HTML, or markdown rendering; notes are plain text (D-21).
- Do not add: billing, price, trial, plan, quota, `sync_locked`, or subscription copy (D-04); signature or stamp imagery (D-06); an exact-cédula “save anyway” override, an age field, a client checksum/mask, or a claim that an offline local update is server-audited.
- Do not add: a rendered audit row, an invented page-local `sensitive_read`, or a browser label; the shipped `sensitive_read` belongs to the online load service and the shipped `update_patient` to the write service.
