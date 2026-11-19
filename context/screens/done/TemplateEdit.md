# PersonaSanus — TemplateEdit (`/templates/:id/edit`)

| Field | Value |
|---|---|
| Source | `context/sitemap.md` §1.2 (`/templates/:id/edit` — Template forms; Owner doctor, Administrator for master), §2 (roles), §3 flow 3 (personal template) and flow 9 (image upload), §4 M1; `context/PRD.md` FR-020–FR-024, FR-040–FR-043, FR-080–FR-085, D-01, D-04, D-06, D-07, D-21; `context/Design.md` (Tokens — Colors, Typography, Spacing & Shapes; Form Input and Textarea; Primary Action Button; Outlined Secondary Button; Elevation; Imagery; Do's and Don'ts); `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` step 14, Page adoption row `TemplateEdit` (line 226), primitives `PageShell`, `PageHeader`, `Notice`, `SectionCard`, `FormGrid`, `ActionRow`, `Button`, `ButtonLink`, `TextField`, `TextAreaField`, `SelectField`, `ErrorText`, `HintText`, `ConfirmPanel`, `LoadingState`; `context/screens/INDEX.md` §2.3 |
| Component | `TemplateEdit` — `persona-sanus/src/main.rs:3788-4046` (`#[component]` at `:3787`); fills the document through `forms::TemplateDocumentForm` — `persona-sanus/src/forms.rs:636-964`; calls `api::fetch_template` (`api.rs:808`), `publish_template_dispatch` (`main.rs:1063`), `api::archive_template` (`api.rs:871`); labels from `template_owner_label` (`main.rs:4241`) and `template_status_label` (`main.rs:4223`); planned primitives live in `persona-sanus/src/ui/` (that directory is planned, not shipped) |
| Route | `/templates/:id/edit` — `Route::TemplateEdit { id: String }` (`persona-sanus/src/main.rs:80-81`); authenticated shell; `checked` false shows the shell loading state; a missing session is replaced with `/login` (`Route::AuthPage {}`); `Me::can_manage_templates()` is false for `nursing`, `secretary`, and no role, which get the denial; no query parameters; the route id is the only URL value |
| Layout | Inside `#[layout(Navbar)]` (opens at `main.rs:51`, never closed, so every authenticated route wears it). `PageShell id="templates-page"` → `.ui-page` → `.ui-container.ui-stack`: 48px top padding, 80px bottom, 24px inline, 24px stack gap. Paper canvas; archived Warning is a Cream band; flat elevation. No full-bleed exception |
| Locale | `es-EC` on every string; `receta` and `cédula` keep Ecuador spelling; dates `DD-MM-AAAA`; 24-hour time; `·` joins subtitle and variable labels |
| Status | ✅ Spec complete (sections 1–7). Route status: ✅ — the sitemap cell is implemented, guarded by `can_manage_templates()`. Shipped: form-only editor over versioned `doc_json`, existing-asset selection with no upload, draft save, publish gated on `saved_draft`, irreversible archive. Target scope: `PageShell`/`PageHeader`, `SectionCard`/`FormGrid`/`ActionRow` controls, Warning and Success `Notice`, `ErrorText`, `ConfirmPanel`; no new route, endpoint, role, field, or upload surface |

The editor for one reusable prescription template. It edits a doctor's own template, or a clinic master as Administrator, and changes only the template document. This file freezes region order, copy, geometry, and states for the shipped component; it introduces no route, endpoint, table, column, role, permission, or field.

## 1. Purpose

The screen performs five jobs, in order:

1. **Identify.** Name the template from `model.name` and state its ownership and life cycle: `Plantilla del médico` or `Maestra de la clínica`, then `Borrador`, `Publicada`, or `Archivada`.
2. **Fill the header.** Edit the letterhead logo selection, doctor and clinic details, primary and accent colors, and A4 page margin.
3. **Shape the document.** Edit the body and footer as plain text, and insert a supported variable token at the end of the body.
4. **Attach structured content.** Select existing clinic library assets into image slots with a caption, and add, edit, or remove medication rows.
5. **Gate the life cycle.** Save a draft, publish that saved draft as an immutable version that pins its asset ids, or archive after a two-step confirmation.

It never renders:

- Patient PHI. It edits a reusable template, not a patient record: no patient name, cédula, diagnosis, medication, image, or prescription content. The route carries only the opaque template id; no PHI appears in a URL, QR, marketing copy, or audit metadata.
- Billing, price, trial, plan, entitlement, invoice, or `sync_locked` copy (D-04).
- A signature or stamp image, upload, slot, or preview (D-06). The footer holds plain text, including a printed signature line; the doctor signs paper.
- A rich-text editor, WYSIWYG surface, formatting toolbar, `contenteditable`, or HTML rendering (D-21). Body and footer are plain textareas; the document is structured JSON.
- An image upload or delete control, file picker, second asset endpoint, version history, preview canvas, patient picker, PDF action, or unarchive action.
- More than one filled teal action: the sole Primary is `Publicar versión`, disabled until a saved draft id exists. Archived and denied states render none.

## 2. Structure

One Paper column in a 24px stack. Header and archived notice mount before the editor; the editor is one form; feedback follows the form; the page action row closes the view. Archived replaces the form **and** that action row with the Warning notice. The confirmation panel is transient and belongs to the page action row.

| Order | Region | id / class | Band tone | Contents |
|---|---|---|---|---|
| — | Navbar (layout) | `#navbar` · `header.ui-navbar` | Paper | Owned by `context/screens/done/Header.md`; identity, sync indicator, sign-out only |
| 1 | Page shell | `#templates-page` · `div.ui-page` > `.ui-container.ui-stack` | Paper | Width, 24px stack gap, 48px top and 80px bottom padding, 24px inline |
| 2 | Page header | `.ui-page-head` · `h1.ui-page-title` · `p.ui-page-subtitle` · `.ui-page-head-actions` | Paper | Title `model.name`; subtitle owner/status; Secondary `Vista previa`, `Volver` |
| 3 | Archived notice | `.ui-notice.ui-notice--warning` | Cream | `La plantilla está archivada y no admite cambios.`; no action. Archived only; replaces orders 4–7 |
| 4 | Document form | `div.ui-template-editor` holding four `SectionCard`s | Paper | Membrete, Cuerpo, Imágenes, Medicamentos; form actions close this region |
| 5 | Form actions | `.ui-actions` | Paper | Secondary `Guardar borrador`; `Guardando...` while busy |
| 6 | Feedback | `.ui-notice.ui-notice--success` · `.ui-error-text` | Paper | One positive `Notice` or one `ErrorText` |
| 7 | Page actions | `.ui-actions` | Paper | Primary `Publicar versión`; Secondary `Recargar`; Danger `Archivar plantilla` |
| 8 | Archive confirm | `ConfirmPanel` | Cream | Message, confirm, cancel; renders only after the Danger trigger |

Orders 3 and 4–7 are mutually exclusive. Order 8 belongs to order 7. Nothing is horizontally scrollable.

### 2.1 Session, role, and authority

| State | What changes | What stays |
|---|---|---|
| Session check in flight (`!checked`) | `PageShell` plus `LoadingState` `Cargando...` | Root id `templates-page` |
| No session after the check | `navigator.replace(Route::AuthPage {})`; no editor | Route match is transient |
| `user` present, `actor` unresolved (`None`) | `LoadingState` `Redirigiendo...`; no fetch | Root id |
| No role, `nursing`, `secretary` | Denial `PageHeader`: `Acceso denegado` / `Solo doctor y administrador pueden editar plantillas.`; Secondary `Volver a plantillas` | No form, actions, fetch, or archived notice |
| `doctor`, own template | Full header, form, feedback, page actions | Publish enabled only with `saved_draft` |
| `doctor`, clinic master | Same editor; the read succeeds, writes refuse through `template_access(..., write: true)` | Ownership is not inferred client-side |
| `administrator`, clinic master | Same editor; writes succeed | Editing a doctor-owned template is refused server-side |
| `administrator`, doctor-owned template | Load refusal (`Plantilla no encontrada.`) plus `Volver a plantillas`; a desktop cached copy may paint, but writes fail | No optimistic edit or archive |

The client gate only selects the visible screen (`Me::can_manage_templates()`, `api.rs:68-70`). The server re-runs `service::template_access` (`service.rs:1247-1273`) on every read and write: administrator and nursing read clinic masters, a doctor reads own and clinic masters, and a write requires Administrator or the owning doctor. A refusal never navigates.

### 2.2 Template, draft, and content state

| State | What renders | What is fixed |
|---|---|---|
| Archived (`model.status == "archived"`) | Header subtitle ends `Archivada`; Cream Warning notice. Form, Save, Publish, Reload, and Archive are absent; `Vista previa` and `Volver` remain | Root id and header shape. No unarchive control or endpoint exists |
| Published, draft present | Base is the draft `doc_json`; `saved_draft` holds its id; Publish enabled; the form's saved pointer is that draft | The published version stays current until Publish |
| Published, no draft | Base is `current_version`, else the first version, else `forms::default_document_json`; `saved_draft` is `None`; Publish disabled | First Save creates a new draft; Publish never promotes the current one |
| Draft template, no versions | Default document (default letterhead, variable sample text, one empty medication row), margin 15; Publish disabled | Save creates version 1 with status `draft` |
| Unrecognized version status | Base is the first version; `saved_draft` stays `None`; Publish disabled | No guessed draft |
| Malformed `doc_json` | The form substitutes `TemplateDocument::default()`; Publish stays governed by `saved_draft` | The editor paints no parse error; preview paints `ErrorText` |
| Unknown variables | No editor warning, no inline mark, no publish block; text is stored as typed | Warning belongs to preview and pre-signing (`Variables no válidas`) |
| Asset pending or missing | Row can be selected or paints blank; no library error line | Publish refuses a non-ready or unavailable id with the server message |
| Quota full | No prompt, rejection, or effect on Save and Publish here | The 1-image v1 quota is counted by `/api/assets` and enforced at upload on `/settings/clinic` |

Initial selection is `draft.or(current_version).or(versions.first())`. `forms::template_theme_margin` clamps the base theme's `margin_mm` to 5–40 and defaults to 15. Parse failure is silent and replaced, never displayed inflated. Zero medications or zero image slots is valid; the section and its add button remain, with no EmptyState invented.

### 2.3 Save, publish, and archive

| Action | Idle | Busy | Success | Failure |
|---|---|---|---|---|
| Save draft | Secondary `Guardar borrador` | Disabled, label `Guardando...` | In-form `Borrador guardado.`; `on_saved` stores the version id in `saved_draft`; Publish enables | Error text; typed document stays; `saved_draft` unchanged |
| Publish | Primary `Publicar versión`; disabled without `saved_draft` | Target disabled `Publicando...`; shipped has no publishing signal | Success `Notice`; `saved_draft` cleared; `generation += 1`; template refetched; Publish disabled again | Error `ErrorText`; `saved_draft` kept; Publish enabled for retry |
| Archive | Danger `Archivar plantilla` | Target `ConfirmPanel` `busy` disables confirm and cancel; shipped has none | Success `Notice`; `saved_draft` cleared; archived model replaces the form and the whole action row | Error `ErrorText`; target panel remains open; nothing changes locally |

Publish sends exactly `saved_draft`, never `current_version_id` and never a new version id. Saving validates, serializes, and sends `PUT /api/templates/{template_id}/versions/{version_id}` for that draft; with no draft it creates one through `POST /api/templates/{template_id}/versions`. Publish does not create a version: it marks the draft `published` and makes it current. Editing after publish needs a new Save and a new Publish. Editing before Save leaves the current draft id; there is no dirty state.

### 2.4 Web, desktop, and offline surfaces

| Surface | Behaviour |
|---|---|
| Web/wasm | Load, save, publish, and archive call the API. Transport failure paints in the feedback slot; no queue |
| Desktop online | Identical to web. `fetch_assets` feeds the same selects; API results are authoritative |
| Desktop load failure | `fetch_template` failure falls back to `desktop_local::open_local_db` and `list_templates`, matches `id`, fills empty `versions` through `list_template_versions` plus `get_template_version`, and paints the cached template with no offline badge. No match leaves the API error plus `Volver a plantillas` |
| Desktop save failure or offline | No local template-version save, no outbox entry, no provisional draft. The API error paints; typed values live only in the form signals |
| Desktop publish offline | `publish_template_dispatch` calls `desktop_local::publish_local_template`: marks the draft `published`, sets `current_version_id` and template `published`, enqueues `publish_template_version`, returns the local clone. The page reports success, clears `saved_draft`, and reloads from cache. This is a queued publish; no provisional marker exists here |
| Desktop archive offline | `api::archive_template` is called directly. There is no local archive branch or outbox operation; it fails with the transport message and nothing changes |
| Loading and denial | Loading renders `PageShell` plus `Cargando...`; `actor == None` renders `Redirigiendo...`; denial renders the `PageHeader` and `Volver a plantillas`. None paints a form or page action row. Load failure without cache paints `ErrorText` and that back link, with no `h1` |

## 3. Copy text

All strings are `es-EC`, quoted with punctuation. Dynamic values are prose. Chrome strings belong to `context/screens/done/Header.md` §3.

### 3.1 Page header

| Element | Copy | Target |
|---|---|---|
| Page title (`h1`) | `{model.name}` | — |
| Subtitle, doctor-owned / clinic master / unknown | `Plantilla del médico · {status}` / `Maestra de la clínica · {status}` / `Plantilla · {status}` | — |
| Status labels | `Borrador` / `Publicada` / `Archivada` / `Desconocido` | — |
| Secondary header action | `Vista previa` | `/templates/{id}/preview` |
| Secondary header action | `Volver` | `/templates` |

### 3.2 Editor sections and fields

| Element | Copy |
|---|---|
| `Membrete` section `h2` and labels | `Membrete` · `Logo de la clínica` · `Color principal` · `Color de acento` · `Márgenes (mm)` · `Datos del médico` · `Datos de la clínica` |
| Logo empty option | `Sin logo` |
| Asset option label | `{original_name}`, else `Imagen {first 8 characters of id}` |
| `Cuerpo` section and fields | `Cuerpo` · `Variable` · `Insertar variable` · `Pie de página` |
| Body textarea accessible name | `Cuerpo de la receta` |
| Variable option | `{variable_label} — {key}` |
| `Imágenes` section and fields | `Imágenes` · `Imagen` · `Sin imagen` · `Leyenda` · `Quitar imagen` · `Agregar imagen` |
| Image library hint | `Elige imágenes de la biblioteca de la clínica. La biblioteca admite una imagen en esta versión.` |
| `Medicamentos` section and fields | `Medicamentos` · `Medicamento` · `Dosis` · `Vía` · `Frecuencia` · `Duración` · `Cantidad` · `Notas` |
| Medication controls | `Quitar medicamento` · `Agregar medicamento` |
| Save controls | `Guardar borrador` · `Guardando...` |

Variable labels come from `forms::variable_label` for the 14 supported keys. No field renders a `placeholder`; the labels above are the only names. `Variable` and `Cuerpo de la receta` are target accessible names for controls that shipped unlabeled, not new fields.

### 3.3 Page actions, notices, and redirects

| Element | Copy | Kind |
|---|---|---|
| Publish | `Publicar versión` | Primary |
| Publish busy | `Publicando...` | Primary, disabled |
| Reload | `Recargar` | Secondary |
| Archive trigger | `Archivar plantilla` | Danger |
| Archive confirmation | `La plantilla archivada no admite cambios.` | panel copy |
| Archive confirm / cancel | `Confirmar archivounces` / `Cancelar` | confirm / cancel |
| Archived notice | `La plantilla está archivada y no admite cambios.` | Warning |
| Success messages | `Borrador guardado.` / `Versión publicada e inmutable.` / `Plantilla archivada.` | success |
| Session check / actor resolution | `Cargando...` / `Redirigiendo...` | status |
| Denial title / body | `Acceso denegado` / `Solo doctor y administrador pueden editar plantillas.` | heading / sentence |
| Denied or load-error back link | `Volver a plantillas` | Secondary link |

Validation strings (`El documento admite como máximo 10 imágenes.`, `Los colores deben usar el formato #RRGGBB.`, `El campo … supera el tamaño permitido.`), server strings (`La plantilla está archivada.`, `Una versión publicada es inmutable.`, `La plantilla referencia una imagen que no está disponible.`), and transport text paint verbatim in the error slot. §7 freezes `es-EC` fallbacks for English transport strings.

### 3.4 Warning owned by preview

| Element | Copy | Where |
|---|---|---|
| Invalid-variable warning title | `Variables no válidas` | `/templates/:id/preview`, `Notice` Warning |
| Invalid-variable warning detail | `Variables no válidas: {keys}` | preview document warning line |

Neither string paints here. Publishing does not check variables; the warning exists before signing an issued prescription.

## 4. Displayed information

### 4.1 What the screen shows

| Element | Source | Value shown |
|---|---|---|
| Template identity | `template: Signal<Option<api::Template>>` set by `api::fetch_template(&load_id)` (`api.rs:808`) | The `api::Template` row; `model = template()` |
| Page title and subtitle | `model.name`; `template_owner_label(&model.owner_type)` plus `template_status_label(&model.status)` | The stored name, then the two labels from §3.1 joined by ` · ` |
| Draft id | `model.versions.iter().find(status == "draft")` | `saved_draft: Signal<Option<String>>`; also the form's `version_id` prop |
| Base version | `draft.or(model.current_version).or(model.versions.first())` | Source of `initial_doc` and margin |
| Initial document | `base.doc_json`, else `forms::default_document_json()` (`forms.rs:101`) | Parsed `TemplateDocument`; malformed JSON becomes default |
| Initial margin | `base.theme` via `forms::template_theme_margin` (`forms.rs:966`) | 5–40 mm; default 15 |
| Archived flag | `model.status == "archived"` | Hides the form and the page action row |
| Form key | `{model.id}-{generation}` | Remount key; `generation` increments on Reload and publish success |
| Assets | `use_resource(api::fetch_assets)` (`api.rs:923`) → `AssetLibrary.assets` (`AssetRow`, `api.rs:878-890`) | Option ids, labels, kinds; no thumbnails |
| Library quota | `AssetLibrary.quota` | Limit 1; used counts image assets; logo excluded; static hint only |
| Letterhead | `document().letterhead` (`forms.rs:43-49`) | Logo id, doctor and clinic text, primary and accent colors |
| Body and footer | `document().body`, `.footer` | Plain text, including typed or inserted `{{key}}` tokens |
| Image slots | `document().images` (`Vec<ImageSlot>`, `forms.rs:58-64`) | Asset id and caption per row |
| Medications | `document().medications` (`Vec<MedicationBlock>`, `forms.rs:66-76`) | Seven plain-text fields per row |

The base is `doc_json` only. `variables_spec` is neither painted nor editable; the picker offers the canonical 14 keys in `forms::TEMPLATE_VARIABLES` (`forms.rs:11-26` Schlüssel).

### 4.2 Hard rules

- Server-side authorization is the boundary. The client `can_manage_templates()` gate is UX; reads authorize in `get_template`, writes in `template_access(..., write: true)`. Ownership is never inferred from the header or a cached copy.
- PHI stays out. A template is reusable clinical formatting, not a patient record: no patient name, cédula, diagnosis, medication, photo, or prescription paints, and no `forms::sample_values` entry appears. The URL carries only `template.id`.
- No billing, price, trial, plan, entitlement, invoice, or `sync_locked` copy or state (D-04). No signature or stamp image, field, upload, or preview (D-06); the footer is text. No rich text, `contenteditable`, toolbar, or HTML persistence (D-21).
- `doc_json` is structured JSON built from form fields by serde and validated by `forms::validate_document` (`forms.rs:136-178`) and `service::template_document`; it is never HTML. Limits: colors `#RRGGBB`, doctor and clinic details 4000 characters, body 40000, footer 4000, images 10, caption 500, medications 50, most medication fields 500, notes 1000.
- Publish requires an existing saved draft id and targets exactly that id. It never publishes `current_version_id`, never creates a version, and is disabled when no draft exists. Save updates only a server draft; a published version is immutable (`Una versión publicada es inmutable.`).
- Publish pins every asset id referenced by the just-published document: each image slot plus the letterhead logo, sorted and deduplicated. Every id must belong to the actor's clinic and be `status == "ready"`, else publish is refused with `La plantilla referencia una imagen que no está disponible.` Missing ids are not pinned.
- Images are selected from the existing clinic library only. There is no upload, file input, delete, asset endpoint, or quota prompt here. V1 allows 1 library image per clinic; the logo is excluded. Upload and delete live on `/settings/clinic` and are online-only.
- Archive is irreversible here: it sets `templates.status = "archived"`. There is no unarchive endpoint or control; versions and pins are not deleted. An archived template cannot save, publish, or archive again because the state hides every control.
- One filled action per viewport: `Publicar versión`, only while non-archived and `saved_draft` is `Some`. It is disabled, not hidden, when no saved draft exists. `Recargar` is Secondary; Save themes and every add/remove are Secondary except each remove, which is Danger.
- Save truth: saving is an online API call on web and desktop. A desktop save failure or offline Save is not queued or written locally and is not labeled provisional; it paints the API error and keeps values only in form signals. Publish truth: an offline desktop publish writes the local version and outbox and returns success, but this screen carries no provisional marker; `Header.md` owns pending and failed sync (FR-043).
- Unknown-variable warnings are not this screen's job. The editor accepts any `{{key}}` text, and Publish never calls `forms::document_warnings`. `/templates/:id/preview` and the pre-signing check paint the warning.
- No version picker, history, diff, duplicate, rename, delete, unarchive, preview canvas, patient field, or PDF action. Reload is the only recovery control, and it still needs the API or desktop cache.

### 4.3 Dynamic states

| State | Screen behaviour |
|---|---|
| Session check in flight | `PageShell`, `LoadingState` `Cargando...`; no header, form, or actions |
| Signed out after the check | `navigator.replace(Route::AuthPage {})`; no content and no fetch |
| Actor unresolved, user present | `PageShell`, `LoadingState` `Redirigiendo...`; no fetch result |
| No role, nursing, or secretary | Denial `PageHeader` plus `Volver a plantillas`; no fetch, form, notice, primary, or archive |
| Doctor, own template | Header, form, page actions; Publish disabled until Save; all writes accepted |
| Doctor, clinic master | Header and form paint from the read; Save, Publish, Archive each surface `Plantilla no encontrada.`; no read-only badge is invented |
| Administrator, clinic master | Full edit; publish and archive accepted |
| Administrator, doctor-owned template | Load refusal plus back. A desktop cached copy may paint, but every write is refused; no local success is faked |
| Load failure, no cache | `ErrorText` with the API message; `Volver a plantillas`; no `h1`, form, or actions |
| Load failure, desktop cache hit | Cached template paints as the editor with no offline badge; writes still hit the server and fail when unreachable |
| Archived | Header subtitle ends `Archivada`; Cream Warning notice; no form, Save, Publish, Reload, Archive, confirm, or unarchive. `Vista previa` remains |
| Published with a draft | Draft is the base; `saved_draft` some; Publish enabled; Save updates that draft |
| Published, no draft | Current is the base, else the first version; `saved_draft` none; Publish disabled with unchanged label; first Save creates a draft |
| No versions | Default document and margin 15; `saved_draft` none; Save creates version 1 |
| Draft present at load | `saved_draft` some; Publish enabled immediately; editing does not clear it because there is no dirty gate |
| `saved_draft` none or some | None: Primary disabled, Save enabled. Some: Primary enabled and publishes that exact id |
| Saving, save success, save failure | `Guardando...` and Save disabled; then in-form `Borrador guardado.` and a set or updated draft id; or validation or API text, the old draft id, and unsaved fields still in the form |
| Publish cycle | Busy: target `Publicando...`, disabled, `aria-busy`; shipped has no signal. Success: `Versión publicada e inmutable.`, `saved_draft = None`, `generation += 1`, refetch, form remounts on the new current document, Publish disabled. Failure: error text, draft retained, Publish enabled, document unmodified |
| Reload | `generation += 1` without clearing `message` or `saved_draft`; fetch resets the draft id and template. Disabled unless status is `draft` or `published` |
| Archive idle and confirm | Danger `Archivar plantilla`; then `ConfirmPanel` with `La plantilla archivada no admite cambios.`, `Confirmar archivo`, `Cancelar`; no request until confirm |
| Archive busy, success, failure | Busy: target confirm and cancel disabled under `busy`; shipped has none. Success: archived template replaces the model, `saved_draft = None`, form and page actions disappear, Warning plus `Plantilla archivada.` Failure: error text, target panel remains open, model unchanged |
| Unknown variables | Accepted and saved; no inline mark; no publish block; warning only in preview or before signing |
| Parse error in initial doc | Form shows the default document with no error; Save would overwrite the malformed draft. Only preview paints `ErrorText` |
| Assets, library, and quota | Pending or missing asset is selectable or blank here; Publish refuses a non-ready or unavailable id. A failed library leaves empty options and no error; `Agregar imagen` appends a blank slot. Quota full has no prompt or effect here: `/api/assetsenge` counts 1 and `/settings/clinic` refuses the upload |
| Desktop offline save | Save fails with the transport message; typed values stay only in form memory; no outbox entry, provisional draft, or local version |
| Desktop offline publish | Local publish, outbox entry, and local current version; the page reports `Versión publicada e inmutable.` and reloads from cache; server confirmation is pending under Header's sync indicator |
| Desktop offline archive | No local branch; transport error; editor and actions stay |
| Medication empty, add, remove, reorder | The section and `Agregar medicamento` remain for zero rows; add appends an empty seven-field row; remove drops that row and later rows shift up; no reorder control exists, so order is array order |

## 5. Style

Every value is a `context/Design.md` token already defined in `persona-sanus/assets/main.css` `:root`. No new color, font, icon package, CDN, webfont, or shadow.

### 5.1 Color

| Use | Value | Token |
|---|---|---|
| Canvas, cards, header, fields | `#ffffff` | `--color-paper` |
| Titles, labels, values | `#061d29` | `--color-ink` |
| Subtitle, hints, Secondary text and border | `#425d6d` | `--color-mist` |
| Card, field, and divider hairlines | `#e5e7eb` | `--color-hairline` |
| Primary fill, focus ring, brand mark | `#006e75` | `--color-deep-teal` |
| Primary hover fill | `#0b978e` | `--color-bright-teal` |
| Success notice surface | `#b9ffe8` | `--color-mint-glass` |
| Archived Warning surface | `#fffded` | `--color-cream` |
| Error text and Alert glyph | `#4d0037` | `--color-plum` |
| Warning Alert glyph (non-text) | `#ff492c` | `--color-ember-ring` |
| Focus halo | `rgba(185, 255, 232, 0.55)` | `--color-mint-halo` |

Never `#000000`. Cards and fields carry the hairline only; the archived notice is a Cream fill with a hairline, not a chromatic border. Danger is Plum text on a hairline outline, never a filled red. The Primary label is Paper on Deep Teal.

### 5.2 Typography

| Element | Family | Weight | Size / line-height |
|---|---|---|---|
| Page title (`h1`) | `--font-mint` | 500 | 22px / 1.22, Ink |
| Page subtitle | `--font-mint` | 400 | 14px / 1.5, Mist |
| Section heading (`h2`) | `--font-mint` | 500 | 20px / 1.25, Ink |
| Field labels | `--font-mint` | 500 | 14px / 1.5, Ink |
| Inputs, textareas, selects | `--font-mint` | 400 | 16px / 1.5, Ink |
| Image hint | `--font-mint` | 400 | 14px / 1.5, Mist |
| Notices and error text | `--font-mint` | 400 | 14px / 1.5, Ink or Plum |
| Buttons and links | `--font-mint` | 500 | 16px / 1.22 |

`--font-grenette` never appears: it is display-only at 46px and above and banned from app chrome and page titles.

### 5.3 Geometry

- Base 4px. PageShell: 48px top, 80px bottom, 24px inline; container max-width 1200px; stack gap 24px. Page header: flex, space-between, wrap, align start, 16px gap; title-to-subtitle 4px; actions 12px gap, inline end.
- Editor: four `SectionCard`s in a 24px vertical stack. Each is Paper, 1px hairline, 24px radius, 24px padding; heading to grid 16px. `FormGrid` is two equal columns with 16px gaps, one column below 640px.
- Controls: `min-height: 44px`, `8px 12px` padding, 8px radius, 1px hairline. Textareas start at 96px; body 240px; footer 96px. Labels sit 4px above their control.
- Variable row: select grows, Secondary button intrinsic, 12px gap; wraps below 640px. Image slot: select, caption, and Danger remove on one row above 900px, then wrap, then one column. Medication block: seven fields with `Notas` wide, Danger remove after the grid at inline start, hairline between rows.
- Archived notice and `ConfirmPanel`: Cream, 1px hairline, 24px radius, 24px padding (notice `16px 20px`), glyph-to-text 12px, panel 16px top gap and 12px action gap.
- Page action row: flex, wrap, 12px gap, inline end, order Primary, Secondary, Danger. The form action row repeats this flex at its section end. Below 640px all rows and grids become one column; controls keep 44px height; no horizontal scroll.
- Elevation: flat. Hairline separation and Paper-on-Paper contrast only; no shadow on the editor, notice, panel, or controls.

### 5.4 Iconography and imagery

Line icons only, 1.5px stroke, Ink or Deep Teal. `ErrorText` carries its Alert glyph at 20px in Plum; the archived Warning carries the same glyph at 20px in Ember Ring. No photos, logo previews, asset thumbnails, A4 mock, QR, signature, stamp, or illustration paints here; assets appear as option text. The navbar mark and sync dot are chrome, owned by `context/screens/done/Header.md` §5.4.

## 6. Accessibility

- Landmarks: navbar banner (`Header.md` §6) and one `main` containing the editor. This is a form, not a directory or data table; no caption or `th scope` surface exists.
- Exactly one `h1` per readable settled state: `model.name` in the editor and archived states, `Acceso denegado` in denial. The four `SectionCard`s own `h2`s in order `Membrete`, `Cuerpo`, `Imágenes`, `Medicamentos`. Feedback and notices are paragraphs. Loading and load error have no `h1`.
- Tab order is DOM order: `Vista previa`, `Volver` → logo select, both color inputs, margin, doctor details, clinic details → variable picker, `Insertar variable`, body, footer → each image select, caption, remove, then `Agregar imagen` → each medication row's seven fields and remove, then `Agregar medicamento` → `Guardar borrador` → `Publicar versión`, `Recargar`, `Archivar plantilla`. Archived leaves only the two header links. The confirm panel follows the Danger trigger.
- Focus per plan: `1px solid var(--color-deep-teal)`, `outline-offset: 2px`, halo `box-shadow: 0 0 0 3px rgba(185, 255, 232, 0.55)`. Visible on Paper and Cream. Never remove focus without a replacement.
- Labels: every control keeps its visible label text; the section `h2`s name their groups. The body textarea's accessible name is `Cuerpo de la receta`; the picker's is `Variable`. Errors are programmatically associated with the failed action, not by proximity.
- Errors use `role="alert"`. Success and the archived notice use `role="status"` with `aria-live="polite"`. Publish sets `aria-busy`; Save exposes its busy state and disabled attribute. Disabled Publish is programmatically unavailable, not merely pale.
- Status is never colour-only: draft, published, archived, saved, and failed are explicit labels or full sentences. Icons have adjacent text. Notices do not steal focus.
- Contrast AA: Ink on Paper 17.28; Mist on Paper 6.96; Paper on Deep Teal 6.02; Plum on Paper 15.32; Ink on Cream 16.89; Ink on Mint Glass 15.25; Mist on Mint Glass 6.14; Mist on Cream 6.80. Disabled Publish keeps a readable Ink label on Paper with a muted fill; no state depends on the tone.
- Keyboard and touch: native inputs, selects, textareas, buttons, and links; no drag-only ordering and no hover-only action. Controls are 44px tall with 12px action gaps. No animation exists here; `LoadingState` spinners stop under `prefers-reduced-motion`.
- Narrow viewport and zoom: one column, no horizontal scroll, labels stay associated, and the archived notice wraps without truncation.

## 7. Implementation notes

- Shipped: `TemplateEdit` at `main.rs:3788-4046`, declared under the navbar layout at `:80-81`. Its effect reruns on `generation`, returns while `!checked()`, replaces to `AuthPage` when `user` is `None`, then calls `api::fetch_template(&load_id)`. Success finds `versions.iter().find(status == "draft")`, writes `saved_draft`, and stores the template. Failure opens `desktop_local`, calls `list_templates`, matches `id`, fills empty `versions` with `list_template_versions` plus `get_template_version`, deserializes `api::Template`, then writes both signals; the API error paints only if `template()` is still `None`.
- Shipped markup: bare `div#templates-page`, `.page-head` with `h4` `model.name` and a `template_owner_label · template_status_label` paragraph, `.actions` holding Secondary links to `TemplatePreview` (`Vista previa`) and `Templates` (`Volver`); `div.notice` for archived; `forms::TemplateDocumentForm` with `key: "{model.id}-{generation}"`; `p.error` for `message`; `.actions` with Publish, Reload, and a conditional archive pair. The success strings (`Borrador guardado.`, `Versión publicada e inmutable.`, `Plantilla archivada.`) also paint through `p.error { class: "error" }`, including inside the form; that is the shipped defect, not the target.
- Plan step 14 and the page row rebuild the shell: `PageShell id="templates-page"`; `PageHeader title=model.name subtitle="{owner} · {status}"` with Secondary `ButtonLink` `Vista previa` and `Volver`; archived Warning `Notice`; `ActionRow` with Primary `Publicar versión` disabled until `saved_draft`, Secondary `Recargar`, Danger `Archivar plantilla`; archive confirmation moved from the inline Danger/Secondary pair to `ConfirmPanel` message `La plantilla archivada no admite cambios.`, confirm `Confirmar archivo`, cancel `Cancelar`; feedback uses Success `Notice` for the three positive strings and `ErrorText` for failures. The shipped strings are unchanged; no endpoint, route, role, field, or upload control is added.
- Keep `TemplateDocumentForm` (`forms.rs:636-964`) as the only editor. Its props stay `template_id: String`, `version_id: Option<String>`, `initial_doc: String`, `initial_margin_mm: u32`, `on_saved: EventHandler<(String, String, u32)>`; callers keep the same arguments and the page keeps the callback body. Inside, replace raw controls with `TextField`, `TextAreaField`, `SelectField`, `Button`, `SectionCard`, `FormGrid`, `ActionRow`, `ErrorText`, and `HintText`, preserving every control named in §4.1 and the `TemplateDocument` shape. Do not add a formatting toolbar, `FileField`, or a new field.
- Save stays `forms::save_document` (`forms.rs:589-633`): validate, serialize the struct, build theme `{ "margin_mm": margin }`, then `api::update_template_version` when `saved_version` is some and `api::create_template_version` when none. `saved_version` starts as `version_id`; `on_saved` receives `(version.id, doc_json, margin)`. Base selection stays `draft.or(current_version).or(first)`, margin stays `template_theme_margin`, and the page callback keeps updating `saved_draft`. Route the draft success to a Success `Notice` and stop painting it twice.
- Publish stays `publish_template_dispatch(&template_id, &version_id, Some((template, profile)))` (`main.rs:1063-1083`): server `api::publish_template` (`api.rs:862-869`), desktop offline `publish_template_offline` → `desktop_local::publish_local_template` (`desktop_local.rs:1675-1716`). It requires `saved_draft`; there is no publish-current and no version-creating publish. On success keep `saved_draft = None`, `generation += 1`, replace `template`, and paint `Versión publicada e inmutable.` in a Success `Notice`. Target adds a publishing signal for `Publicando...`; no second publish call.
- Server publish (`service.rs:1562-1644`) already pins and freezes: refuses archived templates; returns the current row when the version is already published; flips `draft` to `published` (0 rows yields `La versión ya está publicada y es inmutable.`); sets `templates.current_version_id` and `status = "published"`; parses the structured document; collects image-slot asset ids plus the letterhead logo id; sorts and dedupes; requires clinic ownership and `status == "ready"`; inserts one `template_version_assets` row per id; writes the audit row `publish_template_version`. Draft editing is refused by `update_template_version_draft`'s `status = "draft"` filter (`db.rs:1448-1468`) and surfaces `Una versión publicada es inmutable.`
- Archive stays `api::archive_template(&template_id)` through `service::archive_template` (`service.rs:1646-1686`), which sets `templates.status = "archived"` with `archive_template_row` (`db.rs:1502-1513`) and writes an `archive_template` audit row. There is no unarchive API. The target two-step panel preserves that dispatch and adds `busy`; no optimistic archive.
- Image quota source: `GET /api/assets` returns `AssetLibrary.assets` plus `quota { limit: 1, used }` from `api_server.rs:1172-1193`; `used` counts rows with `kind == "image"` and `deleted_at.is_none()`, so `kind == "logo"` is excluded (FR-082, FR-084). The editor consumes only `assets`, paints the static `forms` hint, and never uploads. Upload and delete (`api::upload_asset` `:929`, `api::delete_asset` `:939`) belong to `/settings/clinic`; v1 keeps 10 MB, JPEG or PNG, private storage, and 15-minute signed-read URLs.
- Unknown variables: insertion is `format!("{{{{{key}}}}}")` appended to `document.body` (`forms.rs:656-660`); there is no footer insertion, cursor insertion, rich text, or inline validation. `forms::document_warnings` (`forms.rs:267-281`) is called by `TemplatePreview` (`main.rs:4142-4207`), which owns `Variables no válidas`. Publishing does not check it.
- English-to-`es-EC` replacements: shipped load, save, publish, and archive slots paint raw API text, including `http.rs` transport lines (`Could not reach the server: …`, `Request failed (…)`, `Could not read the server response: …`). Target keeps the raw string in the signal for diagnostics but paints frozen fallbacks: load `No se pudo cargar la plantilla.`, Save `No se pudo guardar el borrador.`, Publish `No se pudo publicar la versión.`, Archive `No se pudo archivar la plantilla.`, library `No se pudo cargar la biblioteca de imágenes.` (the last is target-only). Spanish server and validation strings pass unchanged. Positive strings move from `.error` to Success `Notice`; no positive text becomes an error state.
- Preserve both surfaces: web calls the API; desktop keeps the cache read on template-load failure and the local publish plus outbox branch. Do not add a local draft-save branch, do not queue archive, and do not let a cached copy hide a server refusal. Keep the generation key and dispatch bodies behaviorally identicalech.
- Verify, in order: `cd persona-sanus && cargo check`; `dx build --release --fullstack`; `caveman-blocks run persona-smoke --dir persona-sanus --port 8091`; `python3 .blocks/markdown-table-check.py --path context/screens/done/TemplateEdit.md --strict`. These are commands to run; this file records no pass claim.
- Do not add: a rich-text editor, WYSIWYG, formatting toolbar, `contenteditable`, HTML body, or Markdown preview; a signature or stamp image, upload, field, or placeholder; an image upload or delete control, file picker, drag-and-drop asset zone, quota prompt, or second asset endpoint on this route; a new route, role, permission, table, column, API argument, or asset kind; patient data, PHI, sample values, a patient picker, or prescription content; billing, price, trial, plan, entitlement, invoice, payment, or `sync_locked` copy or state; a second Primary action, filled Danger button, or extra primary in archived or denied states; an unarchive or restore action; version history, diff, duplicate, rename, delete-template, PDF, or print action; a draft-save outbox branch or a fake offline save; optimistic publish or archive; a colour-only archived, saved, busy, or error indication; or English UI copy.

## 8. Spec references

| Spec | Connection |
|---|---|
| `specs/done/15-O-ui-tokens-actions-and-formatting.md` | `15-O` supplies the tokens, `Button`, `ButtonLink`, and format helpers the editor renders. |
| `specs/done/16-P-ui-forms-feedback-and-layout.md` | `16-P` supplies `PageShell`, `PageHeader`, `SectionCard`, `Card`, `FormGrid`, `TextField`, `TextAreaField`, `SelectField`, `HintText`, `ConfirmPanel`, `ErrorText`, and `ActionRow` substituted into `forms::TemplateDocumentForm`. |
| `specs/done/22-V-ui-templates.md` | `22-V` adopts `TemplateEdit`: raw inputs replaced by `TextField`/`TextAreaField`/`SelectField`, one Primary `Publicar versión`, archive `ConfirmPanel`. |
| `specs/done/25-Y-ui-legacy-css-cleanup-and-docs.md` | `25-Y` removes legacy page-ID selectors so the adopted editor runs only on `.ui-*` classes. |
