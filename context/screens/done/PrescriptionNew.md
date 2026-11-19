# PersonaSanus — PrescriptionNew (`/prescriptions/new`)

| Field | Value |
|---|---|
| Source | `context/sitemap.md` §1.2 (authenticated shell: `/prescriptions/new`, Issue prescription, Doctor + Nursing draft-only, purpose and query contract), §2 (navigation: Doctor ✓, Nursing draft only, Administrator/Secretary —), §3 flows 1–2, §4 (M1 forms, M2 offline), §5 (conventions); `context/PRD.md` §3 roles, FR-010, FR-011, FR-013, FR-014, FR-020/021, FR-022, FR-040–043, D-02, D-06, D-07, D-21; `context/Design.md` tokens for color, type, spacing, radii, layout, form input, primary/secondary/ghost buttons, focus; `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` step 15, Page adoption row `PrescriptionNew` (line 229), primitive contracts for `PageShell`, `PageHeader`, `Card`, `FormGrid`, `SelectField`, `TextAreaField`, `TextField`, `ErrorText`, `Notice`, `LoadingState`, `ActionRow`, `Button`, `ButtonLink`; `context/screens/INDEX.md` §1.2–§1.3 and §2.3 |
| Component | `PrescriptionNew` — `persona-sanus/src/main.rs:4594-4837` (`#[component]` at `:4594`, `fn` at `:4595`, closes `:4836`). Composes planned `persona-sanus/src/ui/` primitives `PageShell`, `PageHeader`, `Card`, `FormGrid`, `SelectField`, `TextAreaField`, `TextField`, `ErrorText`, `Notice`, `LoadingState`, `ActionRow`, `Button`, `ButtonLink`; that directory does not exist yet, and `Button`/`ButtonLink` live in the planned `ui/button.rs` |
| Route | `/prescriptions/new` — authenticated; optional query `?patient_id=` and `?template_id=`. `Route::PrescriptionNew { patient_id: Option<String>, template_id: Option<String> }` at `persona-sanus/src/main.rs:86-90`. Client guard `Me::can_create_prescriptions()` (`api.rs:72`, `doctor` or `nursing`); signed-out and unauthenticated renders replace with `/login`. Success navigates to `/prescriptions/:id` (`Route::PrescriptionDetail`), never to a preview. The page rewrites no query parameter and adds no route |
| Layout | Inside `#[layout(Navbar)]`, opened at `persona-sanus/src/main.rs:51` and never closed, so the navbar band renders above it on web and desktop. Root is `PageShell id="prescriptions-page"` → `div#prescriptions-page.ui-page > div.ui-container.ui-stack`; Paper canvas, no full-bleed band, no per-page width. One `Card` carries the form; no sidebar, split view, sticky footer, or `h2` section card |
| Locale | `es-EC` on every string below, punctuation included. `receta`, `paciente`, `plantilla`, `vigencia`, `diagnóstico`. No dates or times are rendered or formatted on this screen; `Vigencia` is free text passed through unchanged. No English remains in the shipped picker-error surfaces (see §7) |
| Status | ✅ Spec complete (sections 1–7). Route status: ✅ — sitemap §1.2: "Create from `?patient_id=` + `?template_id=`. Prefills variables, medications, diagnosis. Query params optional — patient/template pickers shown when absent." Shipped: patient and template selection seeded from the two query values, published-template filter, free-text diagnosis/validity/instructions, API create, desktop local-create fallback, and detail navigation; no variables, medication, or diagnosis prefill and no desktop-only picker error. Target: PageShell/PageHeader/Card form adoption, query ids normalized to the values the create call actually sends, picker fetch failures surfaced, and access denial moved to `Notice`; no medication editor, signing, PDF, or billing |

## 1. Purpose

The create step of the issue path: turn applicable patient and template context into one empty draft, then hand the operator to `/prescriptions/:id` where medications and signing live. Ordered jobs:

1. Bind a draft to exactly one patient and one published template version.
2. Prefer the context supplied by the route (`patient_id`, `template_id`) so a patient or template deep link does not ask again; fall back to visible pickers when either query value is absent or does not resolve.
3. Capture the three optional plain-text fields the draft owns now: diagnosis, validity, general instructions.
4. Dispatch through the same authorized create path the deployment surface already has — `api::create_prescription` online, `desktop_local::create_local_prescription` plus outbox on an unreachable desktop — and navigate to the new draft's detail page.
5. Keep the action deliberately narrow: this screen creates a draft and immediately leaves. Medication entry, signing, PDF, and verification are later screens.

It never renders medication editing or medication rows (`PrescriptionItem`, add/remove/reorder/upsert, FR-011); a sign, issue, publish, void, or supersede action (`sign_prescription_dispatch`, `void_prescription_dispatch`, `supersede_prescription_dispatch`, FR-013); an A4 preview, PDF/print/share action, QR, verification code, or `PENDIENTE DE SINCRONIZACIÓN` placeholder (FR-022, FR-030–034, FR-043); an RX number, block indicator, pending count, or outbox failure list (FR-014, FR-042/043; D-02); billing, price, trial, plan, quota, or `sync_locked` copy (D-04, v1.1); a signature or stamp field, image, or imprint (D-06); any rich-text affordance, formatting toolbar, contenteditable surface, HTML field, or markdown hint (D-21); patient cédula, birth date, sex, phone, notes, search, duplicate check, or create/edit (the patient screens); template letterhead/body/footer fields, variable insertion, image slots, version publish, or library upload (the template screens); an audit log or sensitive-read banner (the audit screen). Neither query value carries PHI: both are opaque ids, and no free text is ever added to the URL.

Selecting a template performs no variable expansion: the page never reads `TemplateVersion.variables_spec`, `doc_json`, or medication content, and sends only the version id. The sitemap's “prefills variables, medications, diagnosis” describes a later form surface, not this component.

## 2. Structure

One page column inside the shared shell. Every region is Paper; no tinted band, no image, no decorative glyph, no shadow.

| Order | Region | `id` / class | Band tone | Contents |
|---|---|---|---|---|
| — | Navbar (layout) | `header.ui-navbar` → `.ui-navbar-inner` | — | Owned by `context/screens/done/Header.md` §2–§3; renders above this page and is never edited here |
| 1 | Page canvas | `PageShell id="prescriptions-page"` → `div#prescriptions-page.ui-page > .ui-container.ui-stack` | Paper `#ffffff` | 48px top, 80px bottom, 24px inline; container max 1200px; 24px vertical stack |
| 2 | Page header | `header.ui-page-head` | Paper | `h1.ui-page-title` `Nueva receta`; `p.ui-page-subtitle` `Paciente y plantilla publicada.`; `.ui-page-head-actions` with one `ButtonLink` Secondary `Cancelar` → `/prescriptions` |
| 3 | Form card | `div.ui-card` | Paper `#ffffff`, 1px hairline border | `FormGrid columns=Two`: patient `SelectField`, template `SelectField`, diagnosis `TextAreaField`, validity `TextField`, instructions `TextAreaField wide`; no card heading |
| 4 | Submit alert | `ErrorText` → `.ui-error` | Paper (inside the Card flow, below it) | Rendered only with a non-empty message; validation or API/local failure text, Alert icon |
| 5 | Action row | `ActionRow` → `div.ui-actions` | Paper | Exactly one `Button` Primary `Crear borrador`; while `saving`, label `Creando...`, `loading=true`, `disabled=true` |
| 6 | Denied block | `Notice tone=Error` + `ButtonLink` Secondary | Paper | Replaces regions 3–5 when the actor cannot create; `Acceso denegado`, reason line, `Volver a recetas` → `/prescriptions` |

The page header renders for an allowed actor and for an allowed actor whose pickers are still loading; the denied treatment keeps the `h1` `Nueva receta` but drops the subtitle. Nothing sits between the header and the card except the `.ui-stack` gap. The form card is the only filled surface besides the Primary button.

### 2.1 Role state — doctor, nursing, other authenticated roles, signed out

Doctor and Nursing render byte-identical forms, copy, pickers, and one Primary. Both may create only a `draft`; neither gets a signing action here. The difference is downstream and owned elsewhere: a doctor can sign on `/prescriptions/:id` or its preview, nursing cannot. Administrator and Secretary (and any role that fails `can_create_prescriptions()`) get regions 1, 2, 6: no Card, no inputs, no Primary, and no picker list read. A session with a role but `actor()` still `None` gets `LoadingState label="Redirigiendo..."` inside the page canvas, never the form. Signed out (`checked()` true, `user()` None) replaces the route with `/login` before paint.

What does not change across allowed roles: labels, hints, validation copy, button copy, published filter, API/local payload, navigation target. This page must never gain a role-aware signing control.

### 2.2 Query-supplied vs picker selection

Both signals are seeded before the lists arrive, so a valid deep link renders the header and a disabled-by-nothing form immediately, then the matching `<option>` becomes the selected option when the list resolves; an invalid link leaves the select on its empty option and the signal empty. `patient_id` is matched to `Patient.id`. `template_id` is normalized against the published list: matching `Template.id` resolves to that template's `current_version_id`; matching a published `TemplateVersion.id` is accepted only when it is the template's current version; anything else is cleared. Absent or blank query values leave both unset. The picker values selected by the operator are the same template-version ids the API takes; they are not `Template.id`.

The page does not rewrite the address bar when a select changes. Reloading therefore reapplies the original query values. Nothing is auto-submitted: a fully prefilled page still waits for `Crear borrador`.

### 2.3 Loading the two lists

Between first authenticated paint and `loaded()`, the page is complete except for options: both selects hold only their empty option, labels are visible, free-text fields are editable, and the Primary is enabled. There is no skeleton, spinner, or disabled field for list loading, and no layout reservation; options insert inside the native selects when each call settles. The single effect runs once (`loaded` guard), after `checked()` is true; it loads patients and templates independently, so one slow call does not block the other's options.

The fetch reads `/api/patients` with an empty query and `/api/templates`; on desktop, each failed call falls back to the local cache (`desktop_local::list_patients`, `list_templates`). On web, there is no local branch. If both the server and the local fallback fail, the target page shows a field error on that select; the shipped page shows nothing. Query normalization runs after both lists settle for that respective list and never overwrites a value the operator chose after paint.

### 2.4 Empty patient list

The patient select contains only `Selecciona un paciente` (`value=""`), plus the field hint `No hay pacientes disponibles.` once the list has loaded successfully and is empty. The template select, the three text fields, and the Primary are unchanged. Submitting is not disabled; it fails the selection check and shows `Selecciona paciente y plantilla.` No create call is made. A failed patient load is not an empty list: it shows `No se pudieron cargar los pacientes.` as the select's field error and suppresses the empty hint.

### 2.5 Empty published-template list

The page renders only templates whose `current_version_id` is `Some` and whose `current_version.status` is exactly `"published"`; drafts, archived templates without a published current version, and templates whose current version object is absent are filtered out even if the list call succeeds. With zero survivors the template select contains only `Selecciona una plantilla` (`value=""`) and the field hint `No hay plantillas publicadas.`; the patient select and text fields are unchanged. Submitting without a template shows `Selecciona paciente y plantilla.` and makes no create call. A failed list call is reported by the field error, not this hint analysis.

### 2.6 Submit busy

The click path validates both selections first; only a valid pair sets `saving`. While `saving` is true the Primary is disabled, exposes `aria-busy="true"`, and reads `Creando...`; a second click returns immediately and cannot issue another request or local insert. The two selects, three text fields, and `Cancelar` are not disabled by the shipped form. The text snapshot is read at click time, so edits made after the click do not change the request already in flight; they are discarded by the success navigation. On any error path `saving` returns to false and the default label returns. A successful path leaves the screen by navigation.

### 2.7 Submit validation error

When either selection normalizes to an empty string, the page sets `message` to `Selecciona paciente y plantilla.` and returns before setting `saving`; the error is a page-level `ErrorText` below the Card, with the Alert icon and `role="alert"`. Both fields keep what the operator chose and are not marked `aria-invalid` by this cross-field check. The message clears at the next submit attempt, not per keystroke. No request is attempted, no draft row is created, and the URL does not change.

### 2.8 Desktop offline

On desktop, the shipped create path first probes `http::is_server_reachable()` (a two-second `GET /health`). Reachable, or on web where the probe is compiled to `true`, uses `api::create_prescription` and the server owns clinic, doctor, number, and audit fields. Unreachable uses `desktop_local::create_local_prescription` inside the local SQLite cache, carrying the patient id, the selected template-version id, the actor's clinic id, the actor's doctor id when present, the three free-text fields, and the actor profile id; it inserts a `draft` and enqueues the `create_prescription` outbox row with the local UUID as `client_id`. Success then refreshes the sync indicator via `probe_sync_status()` and navigates to the local id's detail page.

The page itself never reports the draft as synced and never labels it provisional; that status belongs to the navbar indicator and to `/prescriptions/:id` and `/prescriptions`. A local insert failure leaves the operator on this page with the returned message and the typed values intact; there is no silent success and no second API attempt. On web the desktop branch is compiled out entirely.

### 2.9 Access denied

For an authenticated actor whose role is not `doctor` or `nursing`, the page renders the header plus a `Notice tone=Error` carrying `Acceso denegado`, `Solo doctor o enfermería pueden crear recetas.`, and a Secondary `ButtonLink` `Volver a recetas` to `/prescriptions`. It makes no patient or template list call and reads no desktop cache for this page, so no patient names can flash before the denial state. There is no Primary action, no disabled form field, and no retry control. The guard is UX only; the create endpoint still rejects an unauthorized role.

## 3. Copy text

All visible strings, verbatim, `es-EC`; punctuation belongs to the copy. Chrome copy is owned by `context/screens/done/Header.md` and is not repeated here.

### 3.1 Page header and navigation

| Element | Copy | Target |
|---|---|---|
| Page title (`h1`) | `Nueva receta` | — |
| Page subtitle | `Paciente y plantilla publicada.` | — |
| Header action | `Cancelar` | `/prescriptions` (`Route::Prescriptions { status: None }`) |
| Denied title line | `Acceso denegado` | — |
| Denied body | `Solo doctor o enfermería pueden crear recetas.` | — |
| Denied action | `Volver a recetas` | `/prescriptions` (`Route::Prescriptions { status: None }`) |

### 3.2 Form fields

| Element | Copy |
|---|---|
| Patient label | `Paciente` |
| Patient empty option | `Selecciona un paciente` |
| Patient empty-list hint | `No hay pacientes disponibles.` |
| Patient load error | `No se pudieron cargar los pacientes.` |
| Template label | `Plantilla` |
| Template empty option | `Selecciona una plantilla` |
| Template empty-list hint | `No hay plantillas publicadas.` |
| Template load error | `No se pudieron cargar las plantillas.` |
| Diagnosis label | `Diagnóstico` |
| Validity label | `Vigencia` |
| Instructions label | `Indicaciones generales` |

The three free-text controls carry no placeholder, hint, required marker, or example value on this screen. The two selects use their empty option as the visible prompt and no placeholder attribute.

### 3.3 Action, busy, and feedback

| Element | Copy |
|---|---|
| Primary (default, only primary) | `Crear borrador` |
| Primary (busy) | `Creando...` |
| Submit validation error | `Selecciona paciente y plantilla.` |
| Session gate (unchecked) | `Cargando...` |
| Actor gate (checked, no profile) | `Redirigiendo...` |

Server and local-cache error strings are not frozen page copy: an API failure sets `message` to the string returned by `api::create_prescription` or by the local creator fills that slot, and the page renders it in `ErrorText` without translation or prefix. The only page-owned error fallbacks are the four picker strings in §3.2.

## 4. Displayed information

### 4.1 What the screen shows

| Element | Source | Value shown |
|---|---|---|
| Page title | static | `Nueva receta` |
| Page subtitle | static | `Paciente y plantilla publicada.` |
| Cancel | static route | `/prescriptions`; returns without saving the form state |
| Patient options | `api::fetch_patients("")` → `Patient.id`, `Patient.full_name`; desktop fallback `desktop_local::list_patients` mapped to `api::Patient` | one row per patient, option text the full name only; the value is the opaque `Patient.id` |
| Patient selection | `selected_patient` signal seeded from route `patient_id`, normalized against the loaded patient ids | the matched full name; otherwise the empty option `Selecciona un paciente`; the signal is the patient id or `""` |
| Template options | `api::fetch_templates` → `Template.current_version_id` plus `Template.current_version.status`; desktop fallback maps `desktop_local::list_templates` | only templates with `current_version_id` present and current version status `"published"`; option text `Template.name`, value the current-version id |
| Template selection | `selected_template` signal seeded from route `template_id`, normalized to a published current-version id | the matched template name; otherwise `Selecciona una plantilla`; the signal is the version id or `""` |
| Diagnosis | `diagnosis` signal, `oninput` | exactly the typed plain text, including empty and leading/trailing whitespace |
| Vigencia | `validity` signal, `oninput` | exactly the typed plain text; no date parsing, format mask, expiry calculation, or badge |
| General instructions | `instructions` signal, `oninput` | exactly the typed plain text; no formatting, preview, or HTML interpretation |
| Create request (web or reachable desktop) | `api::create_prescription(patient, template, "", diagnosis, validity, instructions)` | server response id; the empty `client_id` is shipped, not generated here |
| Create request (unreachable desktop) | `desktop_local::create_local_prescription(conn, patient, template, clinic_id, doctor_id, diagnosis, validity, instructions, profile_id)` | local draft id plus outbox id; `clinic_id` is the profile clinic id or `""`, `doctor_id` is `Option` from the profile |
| Submit success | returned `Prescription.id` or `LocalCreatePrescription.id` | navigates to `/prescriptions/:id`; no success message, toast, or inline confirmation |
| Submit failure | `message` signal | page-level `ErrorText` above the action row; the form values stay |
| Busy | `saving` signal | Primary `Creando...`, disabled, `aria-busy="true"`; no field disabled |
| Picker failures | new per-select error signals (§7) | field-level error on that select only; the other picker and the form remain usable |

The `template_id` route value and the selected template value are different identifiers:

| Value | Identity | Where it is used |
|---|---|---|
| `template_id` query parameter | a template id, per the sitemap's create-from contract; can also be a current-version id on an existing deep link | read once by `PrescriptionNew { template_id }` at mount; never sent as unknown |
| `selected_template` | the value of the chosen option, i.e. `Template.current_version_id` | the second positional argument of `api::create_prescription`, serialized as `templateVersionId`; passed as `template_version_id` to `create_local_prescription` |
| `Template.id` | the template row's own id | never sent by this page after normalization; only used to resolve `template_id` to the current version |

The shipped code seeds `selected_template` verbatim from `template_id` while its options use `current_version_id`, so a template id never matches an option; target normalization closes that gap. The page never fetches a template by id and never resolves a version the lists did not return. The select shows names only; version number, owner, and clinic-master marker are not displayed, and duplicate template names are distinguished only by the opaque option value.

### 4.2 Hard rules

- Authorization is server-side. `Me::can_create_prescriptions()` (`api.rs:72`) gates paint, but `POST /api/prescriptions` and the authorized service layer remain the boundary; a client-side bypass or a hand-built query string cannot create a draft. The page sends no role, clinic, doctor, or creator field.
- This screen creates a `draft` and nothing else. Status is server defaulted; the page neither displays nor sets `signed`, `superseded`, `voided`, `pending_sync`, or `failed`. It never calls a signing, voiding, or superseding function.
- The template option set is exactly the published filter: `current_version_id.is_some()` and `current_version.status == "published"`. Do not substitute `Template.status`, do not include drafts, and do not add an archived filter: the version status is the shipped gate.
- The query and the option value are not the same id space. `?template_id=` is normalized to a `TemplateVersion.id` in the published list; the create payload always carries a current-version id. The page never sends a template row id as `templateVersionId`, and never silently sends an empty template while an option appears selected.
- No PHI in the URL or its metadata: `patient_id` and `template_id` are opaque ids; names, cédula, diagnosis, validity text, and instructions never travel in the query, a title, an analytics event, a log line, or a QR. PHI in the page is limited to the authorized patient full names and template names in the native selects.
- Staff-only clinical content: an allowed server session plus a doctor/nursing role. The page never permits a public or unauthenticated view and never caches patient names in a location that outlives the shell's session handling.
- Plain text only (D-21): `textarea` and `select`/`input` values are strings, rendered as text. No HTML, rich-text surface, `contenteditable`, file formatting, or markdown parsing. Nothing on this screen previews the fields.
- No billing, price, trial, plan, quota, subscription, or `sync_locked` copy (D-04). None of these states exists on this page, including on the desktop fallback; v1 has no billing states.
- No signature or stamp image, field, or placeholder (D-06). The doctor signs the printed page after `/prescriptions/:id/preview`.
- Prescription numbering is not this screen's action (FR-014, D-02): no number is requested, shown, allocated, burned, or previewed at draft creation, and no `RxBlockIndicator` renders. A number is consumed only at signing.
- Offline is a write path, not a hidden one: desktop cache keeps the draft and outbox until sync, and the page never claims a server copy exists. It never hides a local failure behind an empty success. The desktop cache is not app-encrypted (D-07), so the authorized clinic workstation and OS disk encryption remain the prerequisite; this page adds no local storage of its own.
- Exactly one Primary per viewport, `Crear borrador`; the denied state has none. `Cancelar` and `Volver a recetas` are Secondary. No chromatic border, no shadow, no icon-only action.
- This page never lists or preloads medications, never edits template content, and never signs. A template pick carries its version id only; later screens expand variables/medications from that version.
- No new route, query parameter, endpoint, API call, table, column, status string, role, or permission. Picker errors use the existing fetch results in memory; no re-fetch loop, retry timer, or extra `/api` call beyond the shipped pair.
- Server order is not guaranteed. The page renders patients and templates in the order returned, except the desktop local fallback, which returns patients by full name and templates by name; it never sorts or re-ranks server results client-side.
- A patient or template removed/invalidated after page load can still be submitted; the failure returns from the server or local store as `message`, the page stays mounted, and no optimistic draft row or navigation occurs.

### 4.3 Dynamic states

| State | Screen behaviour |
|---|---|
| Shell not checked (`checked() == false`) | PageShell renders one `LoadingState` with `Cargando...`; no header, form, or API list call |
| Checked, no session | Effect replaces the route with `/login`; no form paint and no list call |
| Checked, session, `actor()` still `None` | PageShell renders `LoadingState` with `Redirigiendo...`; passive; no redirect or retry is added by this screen |
| Actor cannot create prescriptions | Header plus `Notice` Error: `Acceso denegado`, reason, `Volver a recetas`; no Card, no fields, no Primary, no list read |
| Lists loading | Full form paints with placeholder-only selects and enabled Primary; free-text fields accept input; no spinner or disabled control |
| Lists loaded, both populated | Options insert; existing query selections resolve and display; blank query stays on both placeholders |
| Lists loaded, patients empty | Patient select keeps its placeholder and shows `No hay pacientes disponibles.`; template list and text fields unchanged; submit is allowed and catches the empty patient |
| Lists loaded, no published templates | Template select keeps its placeholder and shows `No hay plantillas publicadas.`; drafts are absent; patient list and text fields unchanged; submit is allowed and catches the empty template |
| Query patient resolves | That patient is the selected option; no user action is needed |
| Query patient does not resolve or is blank | Selection stays empty and the select shows `Selecciona un paciente`; no new error is shown at load |
| Query template is a published template id | Its current version id becomes the selection and the template name is shown |
| Query template is a published current-version id | That version id is the selection; its template name is shown |
| Query template is draft, archived-only, stale, or blank | Selection stays empty and the select shows `Selecciona una plantilla`; no new error is shown at load |
| Patient list fetch fails on web | Patient select shows `No se pudieron cargar los pacientes.`; template list and text fields still usable; submit is blocked by the empty patient check |
| Template list fetch fails on web | Template select shows `No se pudieron cargar las plantillas.`; patient list still usable; submit is blocked by the empty template check |
| Both list fetches fail | Each select shows its own field error; the Primary stays enabled; submit reports the selection check |
| Desktop, server unreachable, cache populated | Lists come from the local cache, no error banner; create writes a local draft and outbox row; navigation is local-detail |
| Desktop, server unreachable, cache empty | Each empty select shows its field hint, not a load error; create still takes the local path and would need a selection |
| Desktop, patient cache read fails, template cache reads | Only the patient select reports a load error; the create path is unaffected |
| Desktop, both API and cache fail | The field error labels the select; no silent empty dropdown; no alternate endpoint is called |
| Operator changes a picker | Selection signal updates; URL, other picker, and free-text values do not change |
| Submit with patient or template empty | `message` set to `Selecciona paciente y plantilla.`; `saving` stays false; no request or local insert; no navigation |
| Submit valid, request in flight | Primary shows `Creando...`, disabled since and `aria-busy`; one request only; fields keep their text; Cancelar remains a link |
| API returns an error | `saving` false, default label, `message` renders the returned error in `ErrorText`; selections and text remain; no navigation |
| API returns a draft | Navigation to `/prescriptions/{id}`; no success toast, no local copy is written by this branch |
| Desktop offline local insert succeeds | Sync indicator is refreshed via `probe_sync_status`; navigation to the local draft's `/prescriptions/{id}`; the draft is provisional until the outbox syncs |
| Desktop offline local insert fails | `saving` false, default label, returned local error in `ErrorText`; values remain; no navigation and no fallback API call |
| Duplicate click while saving | Ignored by the `saving` guard; no second POST, no second outbox row, no second navigation |
| Draft later signed, voided, or superseded | No change on this screen; it has already left for `/prescriptions/:id` and renders no status badge or history |

## 5. Style

### 5.1 Color

| Use | Value | Token |
|---|---|---|
| Page canvas, card surface | `#ffffff` | `--color-paper` |
| Page title, field labels, field text, primary label | `#061d29` | `--color-ink` |
| Page subtitle, hint text, Secondary action text and border, empty option text | `#425d6d` | `--color-mist` |
| Card border, control border, navbar bottom rule | `#e5e7eb` | `--color-hairline` |
| Primary fill, focus border | `#006e75` | `--color-deep-teal` |
| Primary hover fill | `#0b978e` | `--color-bright-teal` |
| Primary label, primary hover label | `#ffffff` | `--color-paper` |
| Field error and page error text, Alert glyph | `#4d0037` | `--color-plum` |
| Error Notice wash | `#ffc3e6` | `--color-blossom` |
| Focus halo | `rgba(185,255,232,0.55)` | mint glass at 55%, no separate token |
| Disabled Primary | `#006e75` at the shared `.ui-button:disabled` opacity `.55` | `--color-deep-teal` |

No `#000000`, no chromatic border: cards and controls separate by the hairline and paper/cream/mint washes only. Error uses Plum text on Blossom; no red outline. The denied `Notice` is the only tinted region and takes `NoticeTone::Error` (Blossom surface, Plum text).

### 5.2 Typography

| Element | Family | Weight | Size / line-height |
|---|---|---|---|
| Page title (`h1`) | `--font-mint` | 500 | 22px / 1.22, Ink |
| Page subtitle | `--font-mint` | 400 | 14px / 1.5, Mist, 4px top margin |
| Header Secondary `Cancelar` / `Volver a recetas` | `--font-mint` | 500 | 16px / 1.22, Mist |
| Field label (Paciente, Plantillacf, Diagnóstico, Vigencia, Indicaciones generales) | `--font-mint` | 500 | 14px / 1.5, Ink |
| Select, input, textarea value | `--font-mint` | 400 | 16px / 1.5, Ink |
| Field hint | `--font-mint` | 400 | 14px / 1.5, Mist |
| Field error and page `ErrorText` | `--font-mint` | 400 | 14px / 1.5, Plum |
| Primary label, default and busy | `--font-mint` | 500 | 16px / 1.22, Paper |
| Denied Notice body | `--font-mint` | 400 | 16px / 1.5, Plum; title line 500 |
| Loading gate label | `--font-mint` | 400 | 14px / 1.5, Mist |

No `--font-grenette` anywhere on this page: its 22px ceiling is text and UI, and the largest element is the 22px `h1`; display serif is reserved for 46px+ marketing/section openers (`Home.md`), not app chrome.

### 5.3 Geometry

- 4px base. PageShell: 48px top, 80px bottom, 24px inline padding; `.ui-container` max 1200px centered; `.ui-stack` 24px vertical gap. No second width wrapper and no narrower reading column.
- PageHeader: flex row, space-between, align start, wrap, 16px gap; subtitle 4px below the title; actions wrap with 12px gap. `Cancelar` uses the shared button box, 12px × 24px padding, 8px radius.
- Card: 24px radius, 1px hairline, 24px padding, Paper, no shadow. It is the only card on the page; no inner cards.
- FormGrid: 2 columns, 16px column and row gap; one column at 760px and below. Reading order: patient, template, diagnosis, validity, instructions; instructions spans both columns (`wide`) on two-column widths and is normal width in the one-column stack. Diagnosis is not wide, so the validity field sits to its right on desktop and below it on narrow screens; no forced row filler.
- Controls: 8px radius, 1px hairline, 8px 12px padding, min-height 44px; textareas min-height 96px and `resize: vertical`; select native chevron, cursor pointer. Labels are 4px above their control, no separate column.
- ActionRow: 16px top margin, 12px gap, wraps; Primary is content-width, not full-bleed. ErrorText sits 8px above the row via its own top margin and the 24px stack gap to the card.
- Denied `Notice`: 24px radius, 20px padding, 12px icon-to-body gap, 20px icon; the `Volver a recetas` link follows the body with 12–16px of space. It is not centered and not a modal.
- Loading gate: the shared `LoadingState` row sits at the top of the page stack, 16px padding; no full-viewport centering or decorative spinner.
- Responsive: below 760px the grid becomes one column, the page header stacks with actions below the title, and nothing scrolls horizontally. The native select may ellipsize a long patient name; it never widens the page.
- Elevation: flat. No card/button/dropdown shadow; the only ring is the focus halo. Radii are only 8px (controls, buttons), 24px (card, notice), and 9999px (the `LoadingState` spinner if circular).

### 5.4 Iconography and imagery

Only the shared 1.5px line icons, 20px default: `Alert` in `ErrorText`/`Notice` for errors, and the `LoadingState` spinner. Secondary `Cancelar` and the denied `Volver a recetas` link carry no icon and no arrow. No document, QR, shield, logo, photo, illustration, patient image, signature, or stamp appears. The page draws no imagery in any state.

## 6. Accessibility

- Landmarks: the shell's navbar is the `header` owned by `context/screens/done/Header.md`; this page's `PageHeader` is a second labeled header region containing the only `h1`. The Card and ActionRow are plain divs, so the page has no invented `main`, `nav`, `form`, or landmark labels. The native selects and buttons carry their own roles.
- Headings: exactly one `h1`, `Nueva receta`, from `PageHeader`. There is no `h2` because there is no `SectionCard`, and the denied title is body text, not a heading. Order is `h1` → none; no skipped level.
- Tab order: header `Cancelar` → `Paciente` select → `Plantilla` select → `Diagnóstico` textarea → `Vigencia` input → `Indicaciones generales` textarea → `Crear borrador`. In the denied state: `Cancelar` (if kept) → `Volver a recetas`. Nothing is focus-trapped, nothing has positive `tabindex`, links stay links and controls stay native.
- Focus: every interactive control uses the plan's utility — `outline: 1px solid var(--color-deep-teal); outline-offset: 2px; box-shadow: 0 0 0 3px rgba(185,255,232,0.55);`. The focus halo is a ring, never elevation; it is visible on Paper and on Blossom.
- Labels: both selects are wrapped by a visible `Paciente`/`Plantilla` label; the three text controls by `Diagnóstico`/`Vigencia`/`Indicaciones generales`. The empty option is a value, not the accessible name, and disappears once a real option is selected. No `aria-label` duplicates, no placeholder-only field.
- Errors and hints: each select's picker-load error renders in its field via the `SelectField` error contract (`role="alert"`, `aria-invalid: true`); the empty-list hint renders via the same field contract and is not an alert. The cross-field submit error is a page-level `ErrorText` with `role="alert"` after the Card; it is announced on insert rather than per keystroke, and because it names both controls it is not tied to one field. The denied `Notice` is `role="alert"`; the loading gate is `role="status"` with `aria-live="polite"`.
- Busy: the Primary sets `aria-busy="true"` and stays disabled until an error path re-enables it. The visible label `Creando...` changes with it; there is no duplicate off-screen “Cargando” string and no focus move on submit.
- Status never relies on colour: `Selecciona paciente y plantilla.` is printed text, the field errors are printed text with an Alert icon, the busy state is the word `Creando...` plus `disabled`, and the denied state is the words `Acceso denegado`. No badge, red border, or green tick carries the meaning alone.
- Contrast on the tokens used, computed from the hex values: Ink on Paper 17.28:1, Ink on Mint 15.25:1, Ink on Cream 16.89:1, Mist on Paper 6.96:1, Mist on Cream/Paper 6.80:1, Paper on Deep Teal 6.02:1, Plum on Paper 15.32:1, Plum on Blossom 10.36:1. All body, label, button, hint, and error pairs clear AA 4.5:1; the 22px and 16px labels also clear AAA large where applicable. No text sits below `.55` disabled opacity except a disabled busy Primary, whose interaction is closed.
- Keyboard/touch: native selects and inputs only — no custom combobox, no pointer-only row, no hover-only hint. The controls' 44px minimum height meets the touch target; the button box is 12px + label + 24px inline padding. `Enter` on a focused select opens the native picker; `Escape` closes it. Submit happens only by activating the Primary; no implicit form submit on `Enter` is wired to the page.
- Reduced motion: the only animation on this page is the shared spinner from `LoadingState` during the session/actor gate; it stops under `@media (prefers-reduced-motion: reduce)`, and all other transitions (none added here) are visual-state only. The busy label itself does not pulse or shimmer.
- Zoom/reflow: the 2-column grid becomes 1 column at 760px CSS pixels; at 400px and 320px the labels and controls remain in reading order with no horizontal scroll.

## 7. Implementation notes

The shipped symbol is `fn PrescriptionNew(patient_id: Option<String>, template_id: Option<String>) -> Element` at `persona-sanus/src/main.rs:4595`, introduced by `#[component]` at `:4594` and closed at `:4836`; the surrounding component/file range is `:4594-4837`. The route variant is `Route::PrescriptionNew { patient_id, template_id }` at `main.rs:86-90`, and `Prescriptions` is `main.rs:84-85`.

Shipped state is four signals for the selection and text: `selected_patient` (`String`, seeded `patient_id.unwrap_or_default()` at `:4604`), `selected_template` (`String`, seeded `template_id.unwrap_or_default()` at `:4605`), `diagnosis` and `validity` (each empty `String` at `:4606-4607`), and `instructions` (empty `String` at `:4608`). `message` (`:4609`), `saving` (`:4610`), and `loaded` (`:4611`) carry feedback and the one-shot fetch guard. The lists are `patients: Signal<Vec<api::Patient>>` and `templates: Signal<Vec<api::Template>>` (`:4602-4603`). The fetch is one `use_effect` (`:4613-4656`): it returns unless `!loaded() && checked()`, redirects with `navigator.replace(Route::AuthPage {})` when `user().is_none()`, then spawns `api::fetch_patients("")` and `api::fetch_templates()`; on desktop each `Err` tries `desktop_local::open_local_db()` + `list_patients`/`list_templates`, maps each `Value` through `serde_json::from_value` into `api::Patient`/`api::Template`, and finally sets `loaded`. Both `if let Ok` fallbacks are silent on total failure.

The two early gates are raw markup: `!checked()` renders `div#prescriptions-page > p` `Cargando...` (`:4658-4661`), and a missing `actor()` after the check renders the same div with `Redirigiendo...` (`:4663-4667`). The denied gate (`:4668-4679`) uses the legacy `div.notice` with `h4` `Acceso denegado`, the `p` `Solo doctor o enfermería pueden crear recetas.`, and a `Link` `Volver a recetas`; the page title and form never render there. The published vector is computed at `:4681-4690` with exactly the filter in §2.5, then rendered by the raw `div#prescriptions-page` at `:4692-4835`: `.page-head` + `h4` `Nueva receta` + subtitle + `Link.button.secondary` `Cancelar` at `:4693-4702`; `.form-grid` at `:4703-4756` with two `<label class="field">` + `<select>` pairs, one `<label class="field">` + `<textarea>` diagnosis, one with `<input>` validity, and `.field.wide` instructions; `.error` `<p>` for `message` at `:4757-4759`; `.actions` with the button at `:4760-4833`.

The submit handler (`:4764-4830`) is the boundary to preserve. It returns early while `saving`, reads both selections, blocks on `patient.is_empty() || template.is_empty()` with `message.set("Selecciona paciente y plantilla.")`, then sets `saving`, clears `message`, snapshots the three text fields and the profile, and spawns. If `!http::is_server_reachable().await`, the desktop-only branch clears `saving`, opens the local DB, calls `desktop_local::create_local_prescription` with `patient`, `template`, the clinic id from `profile.clinic` (or `""`), the doctor id from `profile.doctor`, the three texts, and `profile.profile.id`, then on `Ok(created)` sets `sync` to `probe_sync_status().await` and pushes `Route::PrescriptionDetail { id: created.id }`, on `Err` sets `message`, and `return`s regardless. Otherwise it calls `api::create_prescription(&patient, &template, "", &diagnosis, &validity, &instructions)` — the third argument is an empty `client_id` — and on `Ok` clears `saving` and pushes `Route::PrescriptionDetail { id: prescription.id }`. The `saved` values, not live signals, are what the request carries.

API call sites and types to keep: `api::fetch_patients` (`api.rs:685`; `Patient.id` and `Patient.full_name` at `:87-97`), `api::fetch_templates` (`api.rs:801`; `Template`, `:782`, `current_version_id`, `current_version`, `TemplateVersion.status` at `:768-792`), `api::create_prescription` (`api.rs:270`; the second argument is `template_version_id`, posted as `templateVersionId`), `api::Prescription` (`api.rs:146`), `Me`/`can_create_prescriptions` (`api.rs:41`, `:72`), `desktop_local::open_local_db` (`:302`), `list_patients` (`:720`), `list_templates` (`:730`), `create_local_prescription` (`:1360`), and `probe_sync_status` (`main.rs:216`). Also unchanged: `Route::Prescriptions { status: None }`, `Route::PrescriptionDetail { id }`, and the effect's `replace` to `Route::AuthPage {}`.

What plan step 15 and the Page adoption row change in this component:

- Add `persona-sanus/src/ui/` and use `PageShell` (`layout.rs`) for regions 1 and 6, `PageHeader` for the existing `h4` title and `p` subtitle and the `Link` action, `Card` + `FormGrid` for the raw `.form-grid`, `SelectField` for both `<select>` controls, `TextAreaField`/`TextField` for the three raw controls, `HintText`/`ErrorText` only via the field contract, `ActionRow`, `Button` Primary, and `ButtonLink` Secondary for `Cancelar`/`Volver a recetas`. Do not add `SectionCard`: this page has no `h2`.
- Preserve the existing visible strings unchanged when they become props: `Nueva receta`, `Paciente y plantilla publicada.`, `Cancelar`, `Paciente`, `Plantilla`, `Selecciona un paciente`, `Selecciona una plantilla`, `Diagnóstico`, `Vigencia`, `Indicaciones generales`, `Crear borrador`, `Creando...`, `Selecciona paciente y plantilla.`, `Acceso denegado`, `Solo doctor o enfermería pueden crear recetas.`, `Volver a recetas`, `Cargando...`, `Redirigiendo...`.
- Swap the two early gates to `LoadingState label="Cargando..."` and `LoadingState label="Redirigiendo..."` inside `PageShell`, keeping the same page id and the same short-circuit positions. The denied gate becomes `PageHeader` + `Notice tone=Error` with a `ButtonLink` Secondary `Volver a recetas`; do not style it as an error Card and do not add a permission claim beyond the role result.
- Add two signals beside the shipped state, `patients_error: Signal<Option<String>>` and `templates_error: Signal<Option<String>>`. In the effect, replace the swallowed `else` paths: on the API error, attempt the existing local fallback; if the local fallback is unavailable or also fails, set the signal to the fixed copy `No se pudieron cargar los pacientes.` / `No se pudieron cargar las plantillas.` and leave the corresponding vector empty. If either path succeeds, clear its signal. Do not store raw transport errors there, do not retry, do not add an endpoint, and do not merge the two failures into one banner. On desktop the network failure plus a successful cache read must stay silent, because that is the offline-offline path.
- Add the matching empty hints: when `loaded()` is true, the signal is `None`, and the vector is empty, pass `hint=Some("No hay pacientes disponibles.")` / `Some("No hay plantillas publicadas.")` to the respective `SelectField`. A non-empty vector passes no hint. Error takes precedence over empty.
- Normalize query values once, after the corresponding list has loaded and before the operator's first change. For `patient_id`, `trim()` and select only when it equals a loaded `Patient.id`; otherwise leave `selected_patient` empty so the visible placeholder and the submitted value agree. For `template_id`, `trim()`; if it equals a published `TemplateVersion.id` for the current version, keep it; if it equals a published `Template.id`, replace it with that template's `current_version_id`; anywhere else, empty it. Never overwrite a selection changed after load. The shipped direct seeding (`main.rs:4604-4605`) is the gap: it can pair a visible placeholder with a hidden non-empty id, and it seeds a template id into a field the API reads as a version id.
- Keep the published filter expression and precedence exactly (`main.rs:4681-4690`), including `current_version_id.is_some()` and `current_version.as_ref().is_some_and(|version| version.status == "published")`. Do not filter on `Template.status` and do not recreate the vector from `versions`.
- Keep the dispatch shape: the same `saving` early return, the same cross-field validation string, the same `!http::is_server_reachable()` desktop-only branch, the same `create_local_prescription` arguments (clinic id fallback `""`, doctor `Option`), the same `probe_sync_status` refresh, the same empty `client_id` on the API path, the same API argument order, and navigation to `Route::PrescriptionDetail` with the returned id from either branch. Move the desktop branch's `saving` reset to its error paths so the button cannot fire a second local insert while the first is still writing; successful branches leave the screen by navigation.
- `Cancelar` remains a `ButtonLink`/`Link` to `Route::Prescriptions { status: None }`, not form reset; the page holds no draft on the server before the Primary and therefore has nothing to clean up. Do not add a confirm modal, autosave, or client draft store.
- Do not modify `Header.md`'s chrome, the API functions, the local cache schema, or the server create contract. The only new copy strings are the four picker fallbacks and the two empty hints in §3.2; the only removed English UI string is the unreachable patient-list API error in this surface, which is not rendered by this page anyway.

Verification commands, run from the repo root; none is recorded here as executed:

```sh
cd persona-sanus && cargo check
dx build --release --fullstack
caveman-blocks run persona-smoke --dir persona-sanus --port 8091
python3 .blocks/markdown-table-check.py --path context/screens/done/PrescriptionNew.md --strict
```

- Do not add: medication editor, medication rows, `PrescriptionItem`, add/remove/reorder, or any medication prefill; that is `/prescriptions/:id` (FR-011).
- Do not add: sign, issue, publish, void, supersede, acknowledge-warnings, or any `*_dispatch` call; drafts only here (FR-013).
- Do not add: A4 preview, PDF, print, share, QR, verification code/URL, provisional `PENDIENTE DE SINCRONIZACIÓN` label, or any verification surface (FR-022, FR-030–034, FR-043).
- Do not add: RX number, block allocation, `RxBlockIndicator`, pending count, failed outbox rows, or `sync_locked` (FR-014, D-02, D-04).
- Do not add: billing, price, trial, plan, quota, subscription, or payment copy; v1 has no billing states (D-04).
- Do not add: signature or stamp fields, images, placeholders, or “digital signature” copy (D-06).
- Do not add: rich text, formatting toolbar, `contenteditable`, HTML input/preview, markdown rendering, or editor dependency (D-21).
- Do not add: cédula, birth date, sex, phone, notes, patient search, duplicate warning, or patient create/edit controls to this page; only `full_name` is displayed in the picker.
- Do not add: template letterhead/body/footer editing, variable insertion, image slots, publish, archive, or library upload; this page selects a published version only.
- Do not add: new routes, query params, endpoints, API calls, tables, columns, status strings, roles, permissions, or client-side authorization shortcuts.
- Do not add: client-side date parsing, date picker, expiry calculation, or date validation to `Vigencia`; it stays free text.
- Do not add: autosave, draft restoration, toast, modal, success banner, confirmation dialog, retry timer, or background re-fetch; success is navigation, failure is inline text.
- Do not add: English UI copy, restated navbar/footer strings, a second Primary, a filled danger/secondary action, chromatic borders, or shadows on the page.
- Do not add: medication, diagnosis, patient name, or any free text to the URL; the query carries only the two opaque ids.

## 8. Spec references

| Spec | Connection |
|---|---|
| `specs/done/15-O-ui-tokens-actions-and-formatting.md` | `15-O` supplies the tokens, `Button`, `ButtonLink`, and format helpers the draft form uses. |
| `specs/done/16-P-ui-forms-feedback-and-layout.md` | `16-P` supplies `PageShell`, `PageHeader`, `Card`, `FormGrid`, `SelectField`, `TextAreaField`, `TextField`, `ErrorText`, and `ActionRow` the form renders. |
| `specs/done/23-W-ui-prescriptions.md` | `23-W` adopts `PrescriptionNew`: patient/template selects in `FormGrid`, one Primary `Crear borrador`, published-template filter unchanged. |
| `specs/done/25-Y-ui-legacy-css-cleanup-and-docs.md` | `25-Y` removes legacy page-ID selectors so the adopted form runs only on `.ui-*` classes. |
