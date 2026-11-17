# PersonaSanus — TemplateNew (`/templates/new`)

| Field | Value |
|---|---|
| Source | `context/sitemap.md` §1.2 (`/templates/new`, template create, Doctor and Administrator), §2 (templates destinations by role), §3 flow 3 (personal template), §4 M1 (forms), §5 (es-EC, resource-plural routes, create under `/new`); `context/PRD.md` §5.3 FR-020/FR-021/FR-024, §5.8 FR-080–FR-085, D-04, D-06, D-21; `context/Design.md` Form Input and Textarea, Primary Action Button, Outlined Secondary Button, Surfaces, Elevation, Layout, Imagery, Tokens — Colors/Typography/Spacing & Shapes; `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` step 14 and Page adoption row `TemplateNew`, plus `PageShell`, `PageHeader`, `Card`, `FormGrid`, `TextField`, `HintText`, `ErrorText`, `ActionRow`, `Button`, `ButtonLink`, `Notice`, `LoadingState`; `context/screens/INDEX.md` §2.3 |
| Component | `TemplateNew` — `persona-sanus/src/main.rs:3679-3785` (`#[component]` at `:3679`, `fn` at `:3680`, closes before `TemplateEdit` at `:3787`). Depends on `create_template_dispatch` (`:1022-1042`) and desktop-only `create_template_offline` (`:982-1020`). Composes planned primitives from `persona-sanus/src/ui/`; that directory is not shipped |
| Route | `/templates/new` — authenticated. Requires `Me::can_manage_templates()` (`api.rs:68`: administrator or doctor). Signed out replaces `/login`; an absent actor paints the redirect gate; nursing, secretary, and unroled profiles see the denial. No query parameter. Success pushes `/templates/{id}/edit` |
| Layout | Inside `#[layout(Navbar)]` (`main.rs:51`, never closed). Target `PageShell id="templates-page"` wrapping `div#templates-page.ui-page > .ui-container.ui-stack`; Paper canvas, 48px top, 80px bottom, 24px inline, 24px stack gap; one Card; no full-bleed band, sidebar, or tabs |
| Locale | `es-EC` on every string; `plantilla`, `clínica`, `médico`. No date, time, cédula, or currency is rendered or formatted on this screen |
| Status | ✅ Spec complete (sections 1–7). Route status: ✅ — sitemap §1.2 "Create template: letterhead … + body + footer … No signature or stamp images (D-06)." Shipped: a name-only draft row with a role-derived owner label, blank-name guard, API and desktop-local insert, navigation to the editor. Target: the same single-field draft creation through `PageShell`/`PageHeader`/`Card`/`FormGrid`/`TextField`/`HintText`/`ErrorText`/`ActionRow`; letterhead, body, footer, variables, and the one library image remain editor scope |

## 1. Purpose

Create one template record and hand it to the editor. Ordered jobs:

1. Establish the owner before content exists: an Administrator creates the clinic master; a Doctor creates a personal template. The role decides, not the caller.
2. Collect exactly one plain-text field, the template name, and trim it once on submit.
3. Stop a blank or whitespace-only name locally with the service's message; send every other name to the authorized service, which owns the 120-character limit.
4. Insert a `draft` template with no current version and no `doc_json`, online through the API or on an unreachable desktop through the local store plus the outbox.
5. Navigate to that template's editor, where the letterhead, body, footer, variables, and image are defined; keep the typed name and the exact error on failure so a retry needs no retyping.

It never renders:

- Patient data: no patient name, cédula, birth date, diagnosis, medication, image, or verification code. This page's only data are the template name and the viewer's role, and no patient field reaches its URL, title, logs, or audit metadata.
- Billing, price, trial, plan, subscription, quota, or `sync_locked` copy (D-04).
- Signature or stamp upload, field, image, or placeholder (D-06).
- Rich text, a formatting toolbar, `contenteditable`, HTML, or markdown (D-21).
- An owner selector; letterhead, body, footer, color, variable, medication, or image-slot fields; an upload control; preview, publish, version, archive, or delete actions; a duplicate-name warning; an A4 sheet, QR code, or PDF control.

The create route exists only to name and open a draft; it adds no content version.

## 2. Structure

One centered page column: header, one form card, then the result surfaces in DOM order.

| Order | Region | id / class | Band tone | Contents |
|---|---|---|---|---|
| — | Navbar (layout) | `header.ui-navbar` → `.ui-navbar-inner` | — | Owned by `context/screens/Header.md` §2–§3; unchanged above the page |
| 1 | Page canvas | `PageShell id="templates-page"` → `div#templates-page.ui-page > .ui-container.ui-stack` | Paper `#ffffff` | 48px top, 80px bottom, 24px inline; 1200px container; 24px vertical stack |
| 2 | Page header | `header.ui-page-head` | Paper | `h1.ui-page-title` (the only `h1`), `p.ui-page-subtitle`, `.ui-page-head-actions` holding one Secondary `ButtonLink` to the list |
| 3 | Form card | `div.ui-card` | Paper `#ffffff`, 1px hairline | `FormGrid columns=One` with one labeled `TextField`; no card heading |
| 4 | Owner hint | `.ui-page-stack` inside the Card or directly after the grid | Paper | One `HintText` line derived from the role; no selector, badge, or icon |
| 5 | Error text | `ErrorText` after the Card, before the actions | Paper | Rendered only when the message signal is non-empty; Alert glyph, `role="alert"`; one line or wrapped server text |
| 6 | Action row | `ActionRow` → `div.ui-actions` | Paper | Exactly one Primary `Button`; content-width; busy label and disabled state while saving |
| 7 | Denied block | `Notice tone=Error` + Secondary `ButtonLink` | Blossom `#ffc3e6` | Replaces regions 3–6: denied title, reason, return link; the page header remains |
| 8 | Gate | `PageShell` + `LoadingState` | Paper | Replaces regions 2–7 while the session check is pending or the actor is absent; one label and a shared spinner |

In the allowed state the Card is the only filled surface and the create button is the only filled teal element. The hint sits with the field it describes, not in the page header.

### 2.1 Session and role state

| State | Renders | Does not render |
|---|---|---|
| `checked() == false` | Page canvas and `LoadingState` `Cargando...`; no request | Page header, Card, actions |
| Checked, no session user | The signed-out effect replaces `/login`; no form paints | Card, field, actions |
| Session user, `actor() == None` | Page canvas and `LoadingState` `Redirigiendo...` until a profile arrives | Card, field, actions |
| Administrator | Page header, Card, name field, hint with the clinic-master label, one Primary | Owner selector, clinic picker |
| Doctor | Page header, Card, name field, hint with the doctor label, one Primary | Owner selector, doctor picker |
| Nursing, Secretary, or a profile with no role | Page header and an Error `Notice` with the denial reason and a Secondary return link | Card, field, hint, Primary, error text |

The gate is UX only; `service::create_template` remains the authorization boundary. The denied state still has exactly one `h1`, from `PageHeader`.

### 2.2 Name field state

| State | Screen behaviour |
|---|---|
| Initial | Empty controlled text input, enabled; no client `required`, `maxlength`, mask, or pattern |
| Typed | Value is exactly the typed plain text; no trimming on input; no counter |
| Empty submit | No request and no navigation; the blank-name message appears in `ErrorText`; the field stays editable with its empty value |
| Whitespace-only submit | Trimmed to empty before the check; same message as empty; no request |
| Surrounding spaces | Trimmed once before submit; the trimmed value is what the request and the redirect represent; the input shows the untrimmed text until navigation |
| Longer than 120 characters | Passed to the service; its name-too-long message appears in `ErrorText`; the typed value survives; no client truncation |
| Busy | Primary shows the busy label, `loading=true`, `disabled=true`, `aria-busy`; the guard drops a second click; field and header cancel link stay enabled |
| Failure | `saving` returns false, the message is painted, the value is unchanged; a second attempt clears the message and resubmits |

### 2.3 Draft status and create outcome

| State | Screen behaviour |
|---|---|
| Before submit | No template row and no version exist; the screen offers no preview, publish, image, or variable control |
| Service insert | Row is `status = "draft"`, `current_version_id = null`, `owner_type` is `clinic` or `doctor`, `owner_id` is derived from the actor; no `template_versions` row is created |
| Success, web or reachable desktop | `api::create_template` returns the row; the page pushes the editor route for `Template.id`; the create page does not stay visible and shows no toast |
| Success, unreachable desktop | `create_template_offline` inserts the same draft shape locally with a generated client id and queues the outbox operation; the page pushes the same editor route, which reads the local row |
| Repeated name | Accepted: neither the service nor the schema checks name uniqueness; a second submit creates a second draft and navigates to it |
| Empty-name error | The service message is painted once in `ErrorText`; no row and no redirect |
| Role or owner mismatch | The service's forbidden message is painted in `ErrorText`; no row and no redirect; no client-side second guard invents a different message |
| Malformed success body | The API's read-back message is painted in `ErrorText`; no navigation, even though the row may have been inserted |
| Desktop local store failure | The returned local error text is painted in `ErrorText`; no navigation |
| Remount | A fresh empty name and empty message; no draft persistence and no optimistic row |

The session gate is the only loading state: there is no spinner during create, only a disabled busy Primary. On web an unreachable server yields the dispatch's `Sin conexión.` error. On desktop the same field serves both surfaces; reachability is probed at submit, so the page does not warn in advance, disable the Primary, or show an offline badge. A stale probe is handled by the API error path. Navigation is `push` to the editor, never to the list.

## 3. Copy text

All strings are verbatim `es-EC`; punctuation belongs to the copy. Navbar and footer strings are owned by `context/screens/Header.md` §3 and `context/screens/Footer.md` §3 and are not repeated.

### 3.1 Page header

| Element | Copy |
|---|---|
| Page title (`h1`) | `Nueva plantilla` |
| Page subtitle | `Se crea un borrador; el contenido se define en el editor.` |
| Header action | `Cancelar` |

### 3.2 Form

| Element | Copy |
|---|---|
| Name field label | `Nombre` |
| Name field placeholder | `Nombre de la plantilla` |
| Owner hint, Administrator | `Tipo: Maestra de la clínica` |
| Owner hint, Doctor | `Tipo: Plantilla del médico` |

The placeholder restates the field's subject but is not its accessible name. Both hints begin with `Tipo: ` and append the role-derived owner label.

### 3.3 Actions and state copy

| Element | Copy |
|---|---|
| Primary, default | `Crear plantilla` |
| Primary, busy | `Creando...` |
| Blank or whitespace-only name | `Escribe el nombre de la plantilla.` |
| Session check | `Cargando...` |
| Actor gate | `Redirigiendo...` |
| Denied title | `Acceso denegado` |
| Denied body | `Solo doctor y administrador pueden crear plantillas.` |
| Denied action | `Volver a plantillas` |

`Cancelar` appears once, in the header. The denied state has no create control.

### 3.4 Service and local-store errors

These render in `ErrorText` exactly as returned; they are not reworded per state.

| Source | Copy |
|---|---|
| `template_name`, empty | `Escribe el nombre de la plantilla.` |
| `template_name`, over 120 characters | `El nombre de la plantilla es demasiado largo.` |
| Administrator without a clinic, online or offline | `Sin clínica asignada.` |
| Doctor without a doctor record, online | `El rol de doctor es necesario.` |
| Doctor without a doctor record, offline | `El perfil no tiene ficha de médico.` |
| Administrator requesting a non-clinic owner | `El administrador solo crea plantillas maestras de la clínica.` |
| Non-administrator requesting a non-doctor owner | `El doctor solo crea sus propias plantillas.` |
| Inserted row not re-read | `Plantilla no encontrada.` |
| Success body not parsed | `No se pudo leer la plantilla.` |
| No desktop fallback, or web cannot reach the server | `Sin conexión.` |
| Desktop local database | The local store's own message, rendered as returned |

## 4. Displayed information

### 4.1 What the screen shows

| Element | Source | Value shown |
|---|---|---|
| Viewer and role | `actor` context, `Signal<Option<Me>>`; `Me::role` | The session profile; no role is typed into the page |
| Page title, subtitle | static | The page name and the draft explanation in §3.1 |
| Owner hint | `profile.is_administrator()` | `clinic` owner for the Administrator, `doctor` owner for the Doctor; label per §3.2 |
| Name | `name` signal, `oninput` | The typed plain text, no transformation |
| Blank-name message | `message` signal after local trim | The §3.3 blank-name string only for empty-after-trim; no request |
| Save busy | `saving` signal | Primary busy label and disabled state while true |
| Create payload | `name().trim().to_owned()`, local `owner_type` | Trimmed name plus `clinic` for the Administrator or `doctor` for the Doctor |
| API result | `api::Template` fields `id`, `status`, `owner_type` | The created draft is used internally; only `id` reaches the redirect |
| Error text | `message` signal from `Err(String)` | The exact service, API, or local-store string in §3.4 |
| Redirect target | `Route::TemplateEdit { id: template.id }` | The editor path `/templates/{id}/edit` |
| Denied state | `Me::can_manage_templates()` false | Denied title, reason, and return link from §3.3 |

The form does not display `owner_id`, `client_id`, `created_by`, timestamps, version counts, or audit fields. The client never sends `owner_id`; the service derives it from the actor and the requested `owner_type`.

### 4.2 Hard rules

- Authorization is server-side. `can_manage_templates()` hides the form from other roles, but `service::create_template` is the boundary: it accepts `clinic` only for an Administrator with a clinic and `doctor` only for an actor with a doctor record, and rejects every other pair. Do not add a client-only claim of permission.
- The owner is not a user choice. There is no owner selector, clinic picker, doctor picker, `owner_id` field, or query parameter. The page maps Administrator to the clinic-master label and Doctor to the doctor label; the service maps the same role to `owner_id`.
- The owner label is presentational. It never becomes the template name, and the template name never appears in the URL, page title, analytics, or a link query. The redirect carries only the opaque template id in the path.
- No patient PHI is collected or displayed: no name, cédula, birth date, diagnosis, medication, photo, or verification code. Do not add a patient selector, prefill, or link here.
- The name is plain text. Trim once on submit, reject empty after trim, enforce the 120-character limit in the service only, and store the trimmed value. Do not add a mask, `maxlength`, `required`, pattern, or client truncation that the shipped control lacks.
- Names are not unique. The service checks only an optional `client_id` for idempotency; this form sends neither `client_id` nor an owner id, and the schema has no unique name. Do not add a duplicate-name lookup, warning, overwrite path, or uniqueness message.
- The route creates a draft only: `status = "draft"`, `current_version_id = null`, no `template_versions` row, no `doc_json`, no content version. Do not add a body editor, version create, publish, archive, delete, or preview on this route.
- Content fields are editor scope: letterhead, body, footer, colors, variables, image slots, and medication rows belong to `/templates/:id/edit` (FR-021). This page never renders them, even as disabled placeholders.
- Images: the v1 quota is one library image per clinic, 10 MB, JPEG or PNG, private storage (FR-080–FR-085). Image selection and upload belong to the editor and clinic settings, not to draft creation; the logo does not count, signatures and stamps do not exist (D-06), and no quota meter is shown here.
- Publishing is immutable-version work in the editor (FR-024); it is not offered before a version exists.
- Exactly one Primary per viewport, the §3.3 create action. `Cancelar` and the denial return are Secondary links; there is no second filled action.
- Billing, price, trial, plan, subscription, image-quota, and `sync_locked` copy are excluded (D-04). Signature/stamp imagery is excluded (D-06). Rich text, HTML, and markdown editing are excluded (D-21).
- Desktop: when the server is reachable, use the API even on desktop. When unreachable, insert the local draft with `client_id = id` and enqueue the `create_template` outbox op; on sync, the server returns the existing row for that client id. The cache never pretends a failed local insert succeeded.
- Offline success is still a draft with no version; it is not published, signed, or quota-checked. Do not announce a publish that did not happen.
- The page adds no route, endpoint, query parameter, table, column, role, permission, validation rule, or client id field.

### 4.3 Dynamic states

| State | Screen behaviour |
|---|---|
| Session check | Canvas and loading label only; no header, card, or request |
| Signed out | The effect replaces history with `/login`; no form paints |
| Actor absent after check | Canvas and redirecting label until a profile arrives; no retry or form |
| Administrator | Form with the clinic-master owner hint and a clinic-owned create |
| Doctor | Form with the doctor owner hint and a doctor-owned create |
| Nursing or Secretary | Denied notice; return link to the template list; no fields |
| No role | Same denial as nursing; no form |
| Empty form | Primary enabled; blank name; hint visible |
| Whitespace-only name | No request; blank-name message; no navigation |
| Typed name | Primary enabled; no validation on each keystroke |
| Saving | Primary busy and disabled; the guard ignores a second click; the field and header link remain enabled |
| Repeated name | No warning; a new draft is created and opened |
| Service name error | Long or empty name message painted verbatim; no navigation; typed value retained |
| Role or owner error | Forbidden message painted verbatim; no navigation; typed value retained |
| Malformed success body | Read-back message painted; no navigation |
| Desktop, server unreachable | Local draft plus outbox; on success, navigate to the editor, which reads the local row |
| Desktop, local insert failure | Local message painted; `saving` false; no navigation |
| Web, request failure | `Sin conexión.` or the transport message painted; no local row |
| Success | `push` the editor route for the returned id; no toast, banner, modal, or inline success |
| Session expires mid-submit | The server rejects; the error is painted; the next guarded navigation replaces `/login`; no optimistic row is shown |

## 5. Style

Every value is a `context/Design.md` token declared in `persona-sanus/assets/main.css` `:root`. No new color, font, icon package, CDN, or webfont.

### 5.1 Color

| Use | Value | Token |
|---|---|---|
| Page canvas, card surface, input surface | `#ffffff` | `--color-paper` |
| Page title, field label, input text, Primary label | `#061d29` | `--color-ink` |
| Subtitle, hint, placeholder, Secondary border and text | `#425d6d` | `--color-mist` |
| Card, input, and notice hairline | `#e5e7eb` | `--color-hairline` |
| Primary fill and focus outline | `#006e75` | `--color-deep-teal` |
| Primary hover | `#0b978e` | `--color-bright-teal` |
| Error text, Alert glyph, denied text | `#4d0037` | `--color-plum` |
| Denied notice surface | `#ffc3e6` | `--color-blossom` |
| Focus halo | `rgba(185,255,232,0.55)` | mint glass at 55% |

Never `#000000`. Never a chromatic border: the Card and input use the hairline; the denied Notice is a flat Blossom fill with a hairline border, not a plum outline. No shadow except the focus halo.

Measured contrast: Ink on Paper 17.28:1 · Mist on Paper 6.96:1 · Plum on Paper 15.32:1 · Ink on Blossom 11.68:1 · Plum on Blossom 10.36:1 · Paper on Deep Teal 6.02:1. AA for body text; AAA where the ratio clears it.

### 5.2 Typography

| Element | Family | Weight | Size / line-height |
|---|---|---|---|
| Page title (`h1`) | `--font-mint` | 500 | 22px / 1.22, Ink |
| Page subtitle | `--font-mint` | 400 | 14px / 1.5, Mist |
| Field label | `--font-mint` | 500 | 14px / 1.5, Ink |
| Input value | `--font-mint` | 400 | 16px / 1.5, Ink |
| Placeholder | `--font-mint` | 400 | 16px / 1.5, Mist |
| Owner hint | `--font-mint` | 400 | 14px / 1.5, Mist |
| Error text | `--font-mint` | 400 | 14px / 1.5, Plum |
| Button label | `--font-mint` | 500 | 16px / 1.22, Paper or Mist |
| Denied title, body | `--font-mint` | 500 / 400 | 16px and 14px / 1.5, Plum |

No `--font-grenette`: this is an application form, not a 46px+ editorial opener.

### 5.3 Geometry

- 4px base. `PageShell`: 48px top, 80px bottom, 24px inline; `.ui-container` max 1200px, centered; `.ui-stack` 24px vertical gap.
- `PageHeader`: flex row, space-between, wrap, 16px gap, align start; subtitle 4px below the title; the cancel action sits at the header action edge. Below 760px the action wraps under the title.
- `Card`: 24px radius, 1px hairline, 24px padding, Paper, no shadow; spans the page stack.
- `FormGrid columns=One`: one column, 16px row and column gap; the control spans the track; no artificially tall field.
- Field: label 4px above the control in the same wrapper; input 8px radius, 1px hairline, 8px 12px padding, 16px / 1.5, min-height 44px, content-width.
- Hint: 8px below the field, 14px / 1.5, content-width; it never sits inside the input.
- `ErrorText`: 8px top margin below the Card; 16px Alert glyph, 8px gap to the line; no border or surface fill.
- `ActionRow`: 16px top margin, 12px gap, wraps; Primary is content-width with 12px 24px padding, 8px radius, min-height 44px. No duplicate cancel action at the bottom.
- Denied `Notice`: 24px radius, 20px padding, 12px icon gap, 20px Alert glyph; the Secondary return link follows the body with 16px top gap, 12px 24px padding, 8px radius.
- Gate: `LoadingState` sits in the page stack at the top-of-content rhythm; no card wrapper and no fixed viewport height.
- Flat: no card, button, input, notice, or hint shadow. The only ring is the focus halo. Radii: 8px controls and buttons, 24px cards and notices. No badge is used here.
- Responsive: 320px and 200% zoom keep one column, labels above controls, no horizontal scroll, no clipped hint or error text.

### 5.4 Iconography and imagery

Line icons only, 1.5px stroke, 24-unit viewBox, Ink or Plum. The Alert glyph appears at 16px in `ErrorText` and 20px in the denied Notice. No decorative icon, template thumbnail, letterhead preview, clinic logo, QR code, A4 sheet, signature, stamp, library image, photography, illustration, or avatar is rendered. The page has no image element and no upload control.

## 6. Accessibility

- Landmarks: the shell navbar is the banner (`Header.md`); the page canvas is a plain `div` with the `ui-page` / `ui-container` classes. Do not invent a `main` landmark in the doc; if the implementation adds one, it wraps all page states and nothing else. There is no form landmark: the control is a native input and the action is a button.
- Headings: exactly one `h1`, the PageHeader title. No `h2`: the Card has no heading and the hint is not a section. In the denied state the title is still the one `h1`; the notice title is strong text, not a heading. Do not use heading levels for size.
- Tab order: header cancel link, then name input, then Primary. In the denied state: return link only. In the gate states: no focusable control. The hint is not focusable.
- Focus: shared utility — `outline: 1px solid var(--color-deep-teal); outline-offset: 2px; box-shadow: 0 0 0 3px rgba(185,255,232,0.55);` — on the cancel link, input, Primary, return link, and notice link. Visible on Paper and Blossom. Never remove the ring without a replacement.
- Label: `TextField` wraps the visible `Nombre` label around the input, so the label is the accessible name; the placeholder is not. `autocomplete="off"` is the field default. The hint is the input's description via `aria-describedby`; it is not an error and must not be announced as one.
- Errors: `ErrorText` renders with `role="alert"`; one message region, never duplicated inline and at page level. When the failure is the empty-name or name-too-long validation, the input is marked `aria-invalid="true"` and described by the message. Role, owner, transport, and local-store failures stay page-level and do not mark `Nombre` invalid.
- Busy: the Primary carries `aria-busy="true"`, is disabled, and its accessible name changes to the busy label; the DOM node is not replaced, so focus does not jump to the body. The name field and cancel link stay usable. A second click is ignored by the guard, not by a new modal.
- Status is never colour-only: the owner is spelled out in the hint, the blank-name failure is a sentence, and the denial is a title plus a reason. There is no icon-only or red-border-only state.
- Contrast: Ink on Paper 17.28:1, Mist on Paper 6.96:1, Plum on Paper 15.32:1, Plum on Blossom 10.36:1, Paper on Deep Teal 6.02:1. All pass AA for body text; large and bold labels exceed AA. Do not use Mist for the Primary label.
- Target size: the input, both buttons, and the links are at least 44px tall; the action uses 12px by 24px padding. The focus ring is not clipped by the Card radius.
- Reduced motion: the gate spinner stops under `prefers-reduced-motion: reduce`; no other animation exists. No transition is needed for error insertion.
- Zoom and reflow: 200% zoom and 320px width keep the title, subtitle, input, hint, error, and one button in one column, with the full hint and server error wrapping rather than truncating.
- Language: the surrounding document is `es-EC`; do not set `lang="en"` on a label, hint, or error container. No English UI copy is introduced.

## 7. Implementation notes

The shipped component is `fn TemplateNew()` at `persona-sanus/src/main.rs:3680`, with `#[component]` at `:3679`, closing at `:3785` before `TemplateEdit` at `:3788`. It reads three contexts at `:3681-3683` — `Signal<Option<SessionUser>>`, `Signal<Option<Me>>`, and `Signal<bool>` checked — and owns `name`, `message`, and `saving` at `:3685-3687`. The effect at `:3689-3693` replaces `/login` when checked is true and the user is absent; `:3695-3699` renders the loading gate; `:3700-3704` renders the redirect gate; `:3705-3716` renders the denial notice; `:3717-3721` derives the owner label; `:3723-3784` renders the raw page, one `label.field` input, a `p.hint`, a conditional `p.error`, and a `div.actions` button.

Submit behavior at `:3749-3781`, preserved exactly:

- The button is disabled while `saving`; the handler returns early if `saving()` is already true.
- `let value = name().trim().to_owned()` is the single trim. If `value.is_empty()`, it sets the blank-name message and returns before any state or request.
- `owner_type` is computed from `profile.is_administrator()` as `clinic` or `doctor`; the same profile decides the hint at `:3717-3721`.
- It sets `saving`, clones the profile, clears `message`, then spawns `create_template_dispatch(&value, owner_type, Some(profile.clone()))`.
- `Ok(template)` pushes `Route::TemplateEdit { id: template.id }`; it does not clear `saving`. `Err(error)` sets `saving` false and assigns the message, which the next render paints.
- The disabled attribute is `saving()`, not the HTML `disabled` string; the guard and the disabled state both block a repeat.

`create_template_dispatch` at `:1022-1042` is the web/desktop branch: if `http::is_server_reachable().await`, it calls `api::create_template`; on desktop with no reachable server it requires the `Some(profile)` fallback and calls `create_template_offline`; on a non-desktop build it returns `Sin conexión.`. Do not change the reachability probe, add a queue here, or move the local insert into the component.

`create_template_offline` at `:982-1020` resolves the owner id from the profile: clinic for `owner_type == "clinic"`, doctor otherwise, with the two local owner errors; then `desktop_local::open_local_db` and `desktop_local::create_local_template` (`desktop_local.rs:1632-1673`). That function trims and validates the name, generates a UUIDv7, inserts the row with `status = "draft"`, `current_version_id = NULL`, and `client_id = id`, and enqueues the outbox operation `create_template` with `name`, `ownerType`, `ownerId`, `clientId`, `createdBy`. The returned `api::Template` has the generated id, the requested owner type and resolved owner id, the trimmed name, `draft`, no current version, an empty versions vector, and UTC-second timestamps. On sync, the outbox handler at `:507-515` calls `api::create_template_with_client_id` (`api.rs:822-833`); `service::create_template` returns the existing row when `find_template_by_client_id` matches, before insert (`service.rs:1390-1399`). The table declares `client_id TEXT UNIQUE` (`migrations/20261112000000_create_app_schema/up.sql:115`); `name` has no unique constraint and the only index is `idx_templates_owner` on owner type and owner id (`:120`).

Online create is `api::create_template` (`api.rs:813-820`), a POST of `{ name, ownerType }` to `/api/templates`; the handler at `api_server.rs:1040-1055` takes the actor from the session server-side and calls `service::create_template` (`service.rs:1345-1430`) with the optional owner type and client id. The service trims and validates the name (`template_name`, `:1199-1212`): empty is a bad request, more than 120 Unicode scalar values is a bad request. It then maps `(is_administrator, requested_owner)` to `(clinic, actor.clinic.id)` or `(doctor, actor.doctor.id)` and rejects every other pair. Both insert paths carry no content and create no version. This form never sends a client id; only the desktop outbox payload does.

Plan step 14 adoption:

- `PageShell id="templates-page"` replaces both raw root `div#templates-page` returns and the main root; `PageHeader` carries the shipped title and subtitle and a Secondary `ButtonLink` to `Route::Templates` as the header action. Keep one `h1`.
- `Card` + `FormGrid columns=One` wraps the one field; `TextField label="Nombre"` keeps the shipped placeholder, bound `value`, and `oninput`. Add no `wide`, `autocomplete`, `required`, `maxlength`, input type other than text, or field error duplication.
- `HintText` carries the role-derived prefix and owner label below the field. Wire it as the input description; do not turn it into a `Notice`, badge, select, or hidden input.
- `ErrorText text=message()` replaces the raw `p.error` and renders only when `message` is non-empty; the name-validation message may mark the field invalid, service/role errors must not.
- `ActionRow` holds exactly one Primary `Button`: default shipped label, `loading=saving()`, `disabled=saving()`, and the existing click closure body. Do not move the cancel action into the row; it stays a Secondary `ButtonLink` in the header.
- The guards become `LoadingState label` for the checked and actor gates while keeping `PageShell`. The denial becomes `Notice tone=Error` with the shipped title/body and a Secondary `ButtonLink` to `Route::Templates`; no Primary is rendered.
- Preserve the dispatch call, its `Some(profile.clone())` argument, the `owner_type` values, the trim-before-empty check, `saving` semantics, the error assignment, and the `navigator.push(Route::TemplateEdit { id: template.id })` target. Add no `client_id` to the online create call, no new local insert, no new content field, and no version call.
- All shipped strings are already Spanish; there is no English UI copy to replace on this page. The shared transport layer may return an English failure sentence on an unreachable request, which would paint verbatim in `ErrorText`; do not translate condition by condition here, and do not add a second fallback.

Verification commands, run from the repo root; none is recorded here as executed:

```sh
cd persona-sanus && cargo check
dx build --release --fullstack
caveman-blocks run persona-smoke --dir persona-sanus --port 8091
python3 .blocks/markdown-table-check.py --path context/screens/TemplateNew.md --strict
```

- Do not add: an owner selector, clinic picker, doctor picker, `owner_type` input, or `owner_id` field.
- Do not add: letterhead, body, footer, color, variable, medication, image-slot, preview, publish, version, archive, or delete controls to this route.
- Do not add: a duplicate-name check, name-uniqueness constraint, "already exists" notice, or overwrite/merge path.
- Do not add: image upload, library selection, logo choice, quota meter, signature, or stamp imagery (D-06).
- Do not add: rich text, WYSIWYG, formatting toolbar, `contenteditable`, HTML, or markdown input (D-21).
- Do not add: billing, price, trial, plan, subscription, quota, or `sync_locked` copy (D-04).
- Do not add: a patient selector, patient data, or any PHI to copy, URL, query, title, logs, or audit metadata.
- Do not add: a second Primary, a modal, toast, banner, autosave, client draft store, optimistic row, or advance offline warning.
- Do not add: a new route, endpoint, query parameter, table, column, role, permission, `client_id` form field, `maxlength`, `required`, or pattern.
- Do not add: a translation pass or condition-by-condition error rewriting in the client; the service strings remain authoritative.
