# PersonaSanus — PatientNew (`/patients/new`)

| Field | Value |
|---|---|
| Source | `context/sitemap.md` §1.1 (`/patients/new`, create patient, roles), §1.2 (deduplication warning), §2 (patients by role), §3 flow 1, §5 (no PHI in URLs); `context/PRD.md` §5.1 FR-001/FR-002, §6 `patients`, §8 `es-EC`, §9; D-04, D-06, D-21; `context/Design.md` (Form Input and Textarea, Primary/Secondary Button, Card, Section Header, Notice, Error text, Tokens, Layout); `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` step 13, Page adoption row `PatientNew` (line 227), plus `PageShell`, `PageHeader`, `Card`, `FormGrid`, `TextField`, `SelectField`, `TextAreaField`, `Notice`, `ActionRow`, `Button`, `ButtonLink`, `ErrorText`, `HintText`; `context/screens/INDEX.md` §2.3 |
| Component | `PatientNew` — `persona-sanus/src/main.rs:2953-3126` (`#[component]` at `:2953`, `fn` at `:2954`). Shares `submit_new_patient` at `:2880-2952` with the desktop local-create path. Composes planned `persona-sanus/src/ui/` primitives; that directory is not shipped yet |
| Route | `/patients/new` — authenticated; requires `Me::can_write_patients()` (`api.rs:61`: doctor, nursing, secretary). Signed-out and no-role renders replace with `/login`; the Administrator sees `Acceso denegado` and `Volver a pacientes`, not the form. No query parameter |
| Layout | Inside `#[layout(Navbar)]` (`main.rs:51`, never closed). Root `PageShell id="patients-page"` → `div#patients-page.ui-page > .ui-container.ui-stack`; one `Card` carries the form; Paper canvas, 48px top / 80px bottom / 24px inline, 24px stack gap; no full-bleed band, sidebar, or tabs |
| Locale | `es-EC` on every string. `cédula`, `receta`, `clínica`. Birth dates render and submit as `AAAA-MM-DD` in the native date control, display as `DD-MM-AAAA`; no time on this screen |
| Status | ✅ Spec complete (sections 1–7). Route status: ✅ — sitemap §1.1/§1.2: "Create patient (names, cédula, age/birth date, sex, phone, notes). Deduplication warning on cédula / likely duplicate." Shipped: form, name/birth-date validation, exact-national-id `409` conflict, likely-duplicate warning with ids, confirm retry, API and desktop-local create, detail navigation. Shipped: adopted `PageShell`/`PageHeader`/`Card`/`FormGrid`/fields, notice and error surfaces from the primitives, and one `Ver ficha` warning link per returned id |

## 1. Purpose

Create one patient profile in the caller's clinic, with the minimum identity needed for a prescription. Ordered jobs:

1. Collect the clinical identity fields: full name respected as one free-text string, national id (cédula), birth date, sex, phone, and notes.
2. Validate only what the client can decide: a non-empty name and a parseable birth date when one is typed.
3. Detect two distinct duplication cases before insert: a same-clinic row with the same national id (a hard conflict, never overridable here) and a exact-normalized-name match (a soft warning that can be confirmed).
4. Insert through the existing authorized path — `api::create_patient` online, `desktop_local::create_local_patient` on an unreachable desktop — and navigate to the new patient's detail page.
5. Keep the form usable after a recoverable error: every typed value survives, only the submit is re-enabled.

The page never renders: ages derived from the server's current date, diagnosis, medication, prescription, template, verification code, QR, billing/price/trial/plan/quota/`sync_locked` copy, signature or stamp imagery, patient photos, rich text or `contenteditable`, audit rows, or another clinic's patients. It never edits an existing patient; that is `/patients/:id/edit`. It never deletes. The dedup warning never shows a patient name, cédula, birthday, phone, or notes from the matching row — only an opaque link target and the count/relationship the server returned.

## 2. Structure

One centered page column. The header, one form card, then the result surfaces in DOM order: warning notice, error text, action row.

| Order | Region | `id` / class | Band tone | Contents |
|---|---|---|---|---|
| — | Navbar (layout) | `header.ui-navbar` → `.ui-navbar-inner` | — | Owned by `context/screens/done/Header.md` §2–§3; renders above the page unchanged |
| 1 | Page canvas | `PageShell id="patients-page"` → `div#patients-page.ui-page > .ui-container.ui-stack` | Paper `#ffffff` | 48px top, 80px bottom, 24px inline; container max 1200px; 24px vertical stack |
| 2 | Page header | `header.ui-page-head` | Paper | `h1.ui-page-title` `Nuevo paciente`; `p.ui-page-subtitle` `Los datos se guardan en la clínica actual.`; `.ui-page-head-actions` with one Secondary `ButtonLink` `Cancelar` → `/patients` |
| 3 | Form card | `div.ui-card` | Paper `#ffffff`, 1px hairline | `FormGrid columns=Two` with six controls (§3.1); no card heading |
| 4 | Duplicate notice | `Notice tone=Warning` inside the Card, after the grid | Cream `#fffded` unless `NoticeTone::Warning` maps to Blossom | Title `Posible paciente duplicado`; description `Ya existe un paciente con un nombre similar en esta clínica.`; one Secondary `Ver ficha` link per returned id; Danger `Crear de todas formas` |
| 5 | Error text | `ErrorText` below the Card | Paper | Client validation or server/local error; `role="alert"`, Alert icon |
| 6 | Action row | `ActionRow` → `div.ui-actions` | Paper | Exactly one Primary `Guardar paciente`; busy label `Guardando...`, `loading=true`, `disabled=true` |
| 7 | Denied block | `Notice tone=Error` + Secondary `ButtonLink` | Paper | Replaces regions 3–6; `Acceso denegado`, reason, `Volver a pacientes` |

The `Cancelar` link is plain navigation, not a form reset: no server row exists yet, so there is nothing to abandon. The form card is the only filled surface besides the Primary.

### 2.1 Role and session state

| State | Screen |
|---|---|
| `checked() == false` | `PageShell` + `LoadingState` `Cargando...`; no form, no API call |
| Checked, no session | Effect replaces `/login`; no form paint |
| Session, no profile / no role | `LoadingState` `Redirigiendo...`; no form |
| Doctor, Nursing, Secretary | Header, Card, six fields, Primary `Guardar paciente` |
| Administrator | Header, `Notice tone=Error` `Acceso denegado` + `Solo doctor, enfermería y secretaría pueden crear pacientes.` + Secondary `Volver a pacientes`; no Card, no Primary |
| Any other role | Same denial as Administrator |

The role gate is UX only. `POST /api/patients` remains authorized by the service layer; the client sends no role or clinic field.

### 2.2 Empty form

Initial paint has `Nombre visible` empty, `Cédula` empty, birth date empty, `Sexo` on `Sin especificar`, phone and notes empty. The Primary is enabled; pressing it with an empty name returns the inline name error, does not call the API, and does not navigate. Empty optional fields are not errors.

### 2.3 Validation error

Two client-side checks run before the request:

- `full_name.trim()` non-empty, else `Ingresa el nombre del paciente.`
- birth date empty or a valid `YYYY-MM-DD`, else `Ingresa una fecha de nacimiento válida.`

The message is one `ErrorText` below the Card. No field is marked invalid on the name check in the shipped code; the target may bind the message to the name field via `aria-describedby`, but it must not add a second red message. No request or local insert is attempted. Values remain as typed, including surrounding whitespace in the name until submit trims it.

### 2.4 Likely duplicate — name match

On a server `201`-class response with `warning: "likely_duplicate"` and `patientIds`, `create_patient` returns `PatientCreateOutcome::LikelyDuplicate(ids)`. No row was inserted. The page renders region 4:

- Title `Posible paciente duplicado`.
- Description `Ya existe un paciente con un nombre similar en esta clínica.`
- One Secondary `Ver ficha` per id, targeting `/patients/{id}` in the same tab; the operator can inspect and come back with the browser back button, and the typed form is not persisted across that navigation.
- Danger `Crear de todas formas`, which resubmits the exact captured form with `confirmDuplicate: true`.

The warning never prints a raw UUID or another patient's name. It does not say the match is by cédula. It is a name-normalization match; the exact-cédula case is region 5 and has no override. Confirming creates the patient and navigates to its detail. A second name match after confirmation cannot happen for the same payload in the same request; a new submit re-runs the check.

### 2.5 Exact cédula conflict

If `national_id` is non-empty and another patient in the same clinic already has it, the server returns `409 duplicate_national_id` with `Ya existe un paciente con esta cédula.`; `api::create_patient` surfaces that message as `Err`. The page renders it in `ErrorText`, does not show the confirm notice, and offers no override here. The operator must open the existing patient or clear/correct the cédula. A blank cédula is allowed and never matches another blank row.

### 2.6 Desktop offline

`submit_new_patient` probes `http::is_server_reachable()` only after validation. Unreachable desktop calls `desktop_local::create_local_patient` with `clinicId`, `fullName`, `nationalId`, `birthDate`, `sex`, `phone`, `notes`, and the actor profile id; the local row is the only copy until sync. Success clears `saving` and navigates to the local id. Failure leaves the typed values and shows the local error. The duplicate warning is a server outcome and cannot be produced by the local insert; offline duplicate handling is the local cache's job, not an invented client check. Web has no offline branch.

### 2.7 Save success and failure

Success is navigation to `/patients/{id}`; no toast, success banner, or modal. The Primary returns to its default label only because the screen unmounts. On API/local failure the page stays mounted, `saving` is false, the values remain, and the returned message is rendered in `ErrorText`.

## 3. Copy text

All strings verbatim, `es-EC`; punctuation belongs to the copy. Navbar strings are owned by `context/screens/done/Header.md` §3 and are not repeated here.

### 3.1 Fields

| Element | Copy |
|---|---|
| Page title (`h1`) | `Nuevo paciente` |
| Page subtitle | `Los datos se guardan en la clínica actual.` |
| Header action | `Cancelar` |
| Name label | `Nombres y apellidos` |
| National id label | `Cédula` |
| Birth date label | `Fecha de nacimiento` |
| Sex label | `Sexo` |
| Sex empty option | `Sin especificar` |
| Sex option, female | `Femenino` |
| Sex option, male | `Masculino` |
| Sex option, other | `Otro` |
| Sex option, prefer not to say | `Prefiere no decir` |
| Phone label | `Teléfono` |
| Notes label | `Notas` |

Name and cédula are single-line text inputs; notes is a two-column-wide textarea; birth date is `input[type=date]`; phone is `input[type=tel]`. Labels are visible; no placeholder duplicates a label. The sex select always starts on `Sin especificar` and is never blank.

### 3.2 Actions and feedback

| Element | Copy |
|---|---|
| Primary (default join) | `Guardar paciente` |
| Primary (busy) | `Guardando...` |
| Name validation | `Ingresa el nombre del paciente.` |
| Birth date validation | `Ingresa una fecha de nacimiento válida.` |
| Exact cédula conflict | `Ya existe un paciente con esta cédula.` |
| Duplicate notice title | `Posible paciente duplicado` |
| Duplicate notice description | `Ya existe un paciente con un nombre similar en esta clínica.` |
| Duplicate inspect action | `Ver ficha` |
| Duplicate confirm action | `Crear de todas formas` |
| Session gate | `Cargando...` |
| Actor gate | `Redirigiendo...` |
| Denied title | `Acceso denegado` |
| Denied body | `Solo doctor, enfermería y secretaría pueden crear pacientes.` |
| Denied action | `Volver a pacientes` |

Server and local-cache error strings other than the exact cédula conflict are rendered verbatim; they are not frozen page copy. The subtitle and the denied body ship in Spanish and are retained.

## 4. Displayed information

### 4.1 What the screen shows

| Element | Source | Value shown |
|---|---|---|
| Page title, subtitle | static | `Nuevo paciente`, `Los datos se guardan en la clínica actual.` |
| Cancel | static route | `/patients`; no write |
| Name | `full_name` signal, `oninput` | exactly the typed plain text; trimmed only in the submitted form |
| Cédula | `national_id` signal, `oninput` | exactly the typed plain text; optional; no client checksum or mask |
| Birth date | `birth_date` signal, native date input | `AAAA-MM-DD` in the control; `valid_birth_date` requires a parseable date |
| Sex | `sex` signal, select | one of `""`, `female`, `male`, `other`, `unspecified`; UI labels in §3.1 |
| Phone | `phone` signal, `input[type=tel]` | exactly the typed plain text; optional; no format enforcement |
| Notes | `notes` signal, textarea | exactly the typed plain text; optional; no formatting or HTML interpretation |
| Submitted form | `api::PatientForm` | the seven fields above; `id` is server-assigned and never submitted |
| Create outcome | `api::create_patient(&form, confirm).await` → `api::PatientCreateOutcome` | `Created(patient)` or `LikelyDuplicate(ids)` |
| Confirm retry | same form, `confirm=true` | adds `confirmDuplicate: true`; success is `Created` |
| Hard duplicate | server `409 duplicate_national_id` → `Err(message)` | `ErrorText` with `Ya existe un paciente con esta cédula.`; no override |
| Offline create | `desktop_local::create_local_patient(conn, json, profile_id)` | local patient id; row is provisional until the outbox syncs |
| Success target | returned `Patient.id` | `/patients/{id}`; no toast or inline success |

The form sends no `clinicId` and no `createdBy`; the server derives them from the session. The page never lists the clinic or asks for it.

### 4.2 Hard rules

- Authorization is server-side. `can_write_patients()` suppresses the form for the Administrator, but the write endpoint also rejects that role. Client suppression is not a boundary.
- Clinic scoping is server-side. The client never sends an arbitrary clinic id and never queries another clinic's patients.
- PHI stays in the form and the created record. No name, cédula, birth date, phone, or notes in the URL, page title, analytics, logs, toast, or the duplicate link's query string — `Ver ficha` carries only the opaque patient id in the path.
- The duplicate warning never reveals the matching patient's name, cédula, birth date, or other fields. It states that a similar name exists and offers navigation; the authorized detail page owns the disclosure.
- Exact-national-id conflict is not confirmable here. `Crear de todas formas` applies only to the name-match warning. Do not add a client "ignore conflict" path.
- No billing, price, trial, plan, quota, `sync_locked`, or subscription copy (D-04). No image-library quota on this screen.
- No signature or stamp upload, field, image, or placeholder (D-06). No patient photo.
- No rich text, WYSIWYG, formatting toolbar, `contenteditable`, HTML payload, or markdown rendering (D-21). All fields are plain controlled inputs/selects/textarea.
- Birth date remains optional. Do not make it required, do not compute age, and do not reject dates merely because they are old or recent; only parseability is checked here, and the server owns any stronger rule.
- Cédula remains optional and free text on this screen; do not add a checksum, 10-digit mask, or forced country format unless the server rejects it. The exact duplicate key is the server's cleaned value.
- Notes are plain text and are not rendered on this screen. No clinical decision support, diagnosis hint, or medication catalog.
- Exactly one Primary `Guardar paciente`; `Cancelar`, `Ver ficha`, and every denial action are Secondary. `Crear de todas formas` is Danger, not a second Primary.
- The page adds no route, endpoint, query parameter, table, column, role, or permission.

### 4.3 Dynamic states

| State | Screen behaviour |
|---|---|
| Session check | `PageShell` + `LoadingState` `Cargando...`; no form |
| Signed out | Replace `/login`; no form |
| Actor absent after check | `LoadingState` `Redirigiendo...`; no form |
| Allowed role | Header, Card, six fields, one Primary |
| Administrator | Header, `Notice tone=Error` `Acceso denegado`, denied body, Secondary `Volver a pacientes`; no fields |
| Empty form | Enabled Primary; blank name/birth date/cédula/phone/notes; sex `Sin especificar` |
| Name empty | `Ingresa el nombre del paciente.`; no request; values stay |
| Birth date invalid | `Ingresa una fecha de nacimiento válida.`; no request; values stay |
| Valid submit, busy | Primary `Guardando...`, disabled, `aria-busy="true"`; second click ignored |
| Exact cédula exists | `Ya existe un paciente con esta cédula.` in `ErrorText`; no confirm notice; values stay |
| Name match only | Warning notice with title, description, `Ver ficha` per id, Danger `Crear de todas formas`; Primary still visible but the confirm path is the notice action |
| Duplicate inspection | `Ver ficha` navigates to `/patients/{id}`; returning re-renders a fresh empty form, not the previous draft |
| Confirm duplicate | Danger submits the same captured payload with `confirmDuplicate`; busy state on that action; success navigates |
| Confirm failure | `ErrorText`; notice may remain; values stay |
| API failure | `ErrorText` with the returned message; Primary re-enabled; no navigation |
| Desktop offline | Local insert path; success navigates to local detail; failure shows local error; no server-duplicate warning |
| Web | No local branch; server error only |
| Success | Navigate to `/patients/{id}`; no toast, banner, or modal |
| Session expires mid-submit | Server rejects; error surfaces; next guarded navigation returns to `/login`; no optimistic row is shown |

## 5. Style

Every value is a `context/Design.md` token declared in `persona-sanus/assets/main.css` `:root`. No new color, font, icon package, CDN, or webfont.

### 5.1 Color

| Use | Value | Token |
|---|---|---|
| Page canvas, card surface, control surface | `#ffffff` | `--color-paper` |
| Page title, labels, input text, Primary label | `#061d29` | `--color-ink` |
| Subtitle, hints, Secondary text and border, placeholders | `#425d6d` | `--color-mist` |
| Card, control, and notice hairline | `#e5e7eb` | `--color-hairline` |
| Primary fill and focus ring | `#006e75` | `--color-deep-teal` |
| Primary hover | `#0b978e` | `--color-bright-teal` |
| Warning surface | `#fffded` | `--color-cream` |
| Error surface | `#ffc3e6` | `--color-blossom` |
| Error and warning text, Alert glyph | `#4d0037` | `--color-plum` |
| Focus halo | `rgba(185,255,232,0.55)` | mint glass at 55% |

Never `#000000`. Never a chromatic border: every border is the hairline. Error and warning surfaces are flat fills, not outlined or shadowed.

Measured contrast: Ink on Paper 17.28:1 · Mist on Paper 6.96:1 · Ink on Cream 16.89:1 · Ink on Blossom 11.68:1 · Plum on Paper 15.32:1 · Plum on Blossom 10.36:1 · Paper on Deep Teal 6.02:1. AA for body; AAA where the ratio clears it.

### 5.2 Typography

| Element | Family | Weight | Size / line-height |
|---|---|---|---|
| Page title (`h1`) | `--font-mint` | 500 | 22px / 1.22, Ink |
| Page subtitle | `--font-mint` | 400 | 14px / 1.5, Mist |
| Field label | `--font-mint` | 500 | 14px / 1.5, Ink |
| Input, select, textarea value | `--font-mint` | 400 | 16px / 1.5, Ink |
| Placeholder | `--font-mint` | 400 | 16px / 1.5, Mist |
| Hint | `--font-mint` | 400 | 14px / 1.5, Mist |
| Error text | `--font-mint` | 400 | 14px / 1.5, Plum |
| Duplicate notice title | `--font-mint` | 500 | 16px / 1.5, Ink |
| Duplicate notice body | `--font-mint` | 400 | 14px / 1.5, Mist |
| Button label | `--font-mint` | 500 | 16px / 1.22, Paper or Mist |
| Denied title | `--font-mint` | 500 | 16px / 1.5, Plum |
| Denied body | `--font-mint` | 400 | 14px / 1.5, Plum |

No `--font-grenette` on this screen: it is an application form, not a 46px+ editorial opener.

### 5.3 Geometry

- 4px base. `PageShell`: 48px top, 80px bottom, 24px inline; `.ui-container` max 1200px; `.ui-stack` 24px gap.
- `PageHeader`: flex row, space-between, wrap, 16px gap, align start; subtitle 4px below the title; `Cancelar` sits at the header action edge.
- `Card`: 24px radius, 1px hairline, 24px padding, Paper, no shadow.
- `FormGrid`: 2 columns, 16px column and row gap; 1 column at 760px and below. Order: name, cédula, birth date, sex, phone, notes. Notes spans both columns (`wide`); the other fields keep the two-column rhythm. No field is artificially tall.
- Controls: 8px radius, 1px hairline, 8px 12px padding, 16px / 1.5 text, min-height 44px. Textarea rows 4–5, `resize: vertical`. Native date and tel controls. Labels 4px above the control in the same field wrapper.
- `Notice` warning: 24px radius, 20px padding, 12px icon-to-body gap, 20px Alert glyph; title then description, then action row with 12px gap. The notice sits 16px below the grid, inside the Card.
- `ErrorText`: 8px top margin inside the page stack; 14px / 1.5; Alert glyph 16px; no border.
- `ActionRow`: 16px top margin, 12px gap, wraps; Primary is content-width. `Cancelar` is in the header, never duplicated at the bottom.
- Denied `Notice`: 24px radius, 20px padding, 12px icon gap, 20px Alert icon; the `Volver a pacientes` link follows the body.
- Elevation: flat. No card/button/input/notice shadow. The only ring is the focus halo. Radii: 8px controls and buttons, 24px card and notices.
- Responsive: below 760px one column, header actions below the title, notes full width, no horizontal scroll; 320px keeps the labels and controls usable.

### 5.4 Iconography and imagery

Line icons only, 1.5px stroke, 20px default, Ink or Plum. The page uses the Alert glyph in `ErrorText` and the warning notice; optionally a user/person glyph in the name field is not shipped and must not be added. No photography, illustration, avatar, clinic logo, QR, document preview, signature, or stamp image.

## 6. Accessibility

- Landmarks: the shell navbar is the banner (`Header.md`); `PageShell` is the page content; the Card is a plain `div` unless `SectionCard` is used (it is not, because there is no `h2`). No invented `form` landmark: the fields are native controls; if a form element is added, it must have an accessible name and must not change the submit path.
- Headings: exactly one `h1`, `Nuevo paciente`, from `PageHeader`. No `h2`; the duplicate notice title is a paragraph or heading level 2 only if it becomes a real section. Do not skip a level or use a heading for size.
- Tab order: `Cancelar` → `Nombres y apellidos` → `Cédula` → `Fecha de nacimiento` → `Sexo` → `Teléfono` → `Notas` → `Guardar paciente`; in the warning state, the notice's `Ver ficha` links and `Crear de todas formas` come after the Primary in DOM order only if they are truly after it. Put the warning before the action row so the confirm action is reachable after the links. In the denied state: `Volver a pacientes`.
- Focus: shared utility — `outline: 1px solid var(--color-deep-teal); outline-offset: 2px; box-shadow: 0 0 0 3px rgba(185,255,232,0.55);` — on every field, link, and button. Visible on Paper and Cream.
- Labels: every control is wrapped in its visible `label`; the date input has the `Fecha de nacimiento` label, the phone input has `Teléfono`, no placeholder-only field. The sex select's disabled `Sin especificar` is a value, not its accessible name.
- Errors: `ErrorText` is `role="alert"`; the name and birth-date messages are programmatically associated with the corresponding control (target `aria-describedby`), or the page-level error names both. The duplicate `Notice` is `role="alert"` only if it appears after submit; it must not steal focus. Do not mark every field `aria-invalid` for a single failure.
- Busy: the Primary sets `aria-busy="true"`, is disabled, and its visible label becomes `Guardando...`; a screen reader hears the change once. The form does not move focus on submit.
- Status: the duplicate warning is text (`Posible paciente duplicado`), not a red border or yellow icon alone; the exact-cédula conflict is text, not colour alone.
- Target size: at least 44px for every control unrelated to the action; the action box uses 12px × 24px padding.
- Reduced motion: no animation is needed except the shared spinner during the session gate; it stops under `prefers-reduced-motion: reduce`.
- Zoom/reflow: 200% zoom and 320px width keep one column, labels above values, no horizontal scroll, and no clipped date control.

## 7. Implementation notes

Shipped: `fn PatientNew()` at `persona-sanus/src/main.rs:3621` (`#[component]` at `:3620`), closing at `:3902`. It holds `full_name`, `national_id`, `birth_date`, `sex`, `phone`, `notes`, `warning_ids`, `message`, and `saving` (`main.rs:3627-3635`), plus the `user`, `actor`, `checked`, and `sync` contexts. The plan step-13 adoption is landed: `PageShell id="patients-page"`, `PageHeader` with Secondary `ButtonLink` `Cancelar`, `Card` + `FormGrid columns=Two` (`main.rs:3732-3838`), warning `Notice`, `ErrorText`, and `ActionRow` with one Primary `Button`. The raw `div.form-grid`, the `div.notice` printing joined UUIDs, and the `.error` paragraph are gone.

`submit_new_patient` (`main.rs:3548-3618`) is preserved:

- It returns while `saving` (`main.rs:3560-3562`), validates the trimmed name (`Ingresa el nombre del paciente.`) and a non-empty birth date (`valid_birth_date`, `main.rs:5324`; `Ingresa una fecha de nacimiento válida.`), then sets `saving`, clears `message`, and spawns.
- On `!http::is_server_reachable()`, the desktop branch (`main.rs:3574-3602`) clears `saving` before the local call, passes the camelCase JSON payload and profile id to `desktop_local::create_local_patient`, navigates to `PatientDetail` on success, and sets `message` on failure. On web the branch is not compiled and control falls through to the API call.
- On `api::create_patient(&form, confirm)` (`api.rs:703`), `Created(patient)` navigates to `PatientDetail`; `LikelyDuplicate(ids)` (`warning: "likely_duplicate"`, `api.rs:712`) clears `saving` and stores the ids — the shipped `Ya existe un paciente con un nombre similar.` message is gone; `Err` sets the message and clears `saving`. The exact-cédula `409` arrives as `Err` and renders in `ErrorText`; there is no override.

Duplicate notice shipped (`main.rs:3839-3872`): `Notice tone=Warning` with title `Posible paciente duplicado`, description `Ya existe un paciente con un nombre similar en esta clínica.`, one Secondary `ButtonLink` `Ver ficha` per id to `/patients/{id}`, and a Danger `Button` `Crear de todas formas` that resubmits the captured form with `confirm=true` (`confirmDuplicate: true`, `api.rs:707-709`). No raw UUID and no other patient's fields are printed.

Gates shipped (`main.rs:3643-3706`): `!checked()` → `PageShell` + `PageHeader` + `LoadingState` (`Cargando...`); no session → replace `/login`; `actor() == None` → `LoadingState` `Redirigiendo...`; `!can_write_patients()` → `Notice tone=Error` `Acceso denegado` with the denied body and Secondary `Volver a pacientes`. The form sends no `clinicId` and no `createdBy`.

Verification commands, run from the repo root; none is recorded here as executed:

```sh
cd persona-sanus && cargo check
dx build --release --fullstack
caveman-blocks run persona-smoke --dir persona-sanus --port 8091
python3 .blocks/markdown-table-check.py --path context/screens/done/PatientNew.md --strict
```

- Do not add: a second clinic selector, a clinic-id input, or a free-text role field.
- Do not add: cédula checksum validation, a 10-digit mask, or a country picker unless the server returns that rule.
- Do not add: age derivation, a computed age badge, or a required birth date.
- Do not add: patient photos, signatures, stamps, or any image upload (D-06).
- Do not add: rich text, a formatting toolbar, `contenteditable`, or HTML/markdown rendering for notes (D-21).
- Do not add: billing, price, trial, plan, quota, `sync_locked`, or subscription copy (D-04).
- Do not add: a second Primary, a modal, a toast, an autosave, or a client-side draft store.
- Do not add: a new endpoint, route, query parameter, table, column, role, or permission.
- Do not add: raw duplicate UUIDs, another patient's name/cédula, or a claim that the match is by cédula when it is the name warning.
- Do not add: an "ignore the 409" confirmation; an exact cédula conflict is hard.

## 8. Spec references

| Spec | Connection |
|---|---|
| `specs/done/15-O-ui-tokens-actions-and-formatting.md` | `15-O` supplies the tokens, `Button`, `ButtonLink`, and format helpers the create form uses. |
| `specs/done/16-P-ui-forms-feedback-and-layout.md` | `16-P` supplies `PageShell`, `PageHeader`, `Card`, `FormGrid`, `TextField`, `SelectField`, `TextAreaField`, warning `Notice`, `ErrorText`, and `ActionRow` the dedup path renders. |
| `specs/done/21-U-ui-dashboard-settings-and-patients.md` | `21-U` adopts `PatientNew`: `FormGrid columns: Two`, the duplicate warning with Danger `Crear de todas formas`, one Primary `Guardar paciente`. |
| `specs/done/25-Y-ui-legacy-css-cleanup-and-docs.md` | `25-Y` removes legacy page-ID selectors so the adopted form runs only on `.ui-*` classes. |
