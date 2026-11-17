# PersonaSanus — TemplatePreview (`/templates/:id/preview`)

| Field | Value |
|---|---|
| Source | `context/sitemap.md` §1.2 (`/templates/:id/preview`, Template A4 preview, owner doctor + Administrator), §2 (templates access), §3 flow 3 (publish), §4 M1, §5 (deep links, no PHI in URLs, `es-EC`); `context/PRD.md` §5.3 FR-020–FR-024, §5.4 FR-022/FR-030, §5.8 FR-080–FR-085, D-01, D-04, D-06, D-21; `context/Design.md` (A4 Preview Card, Card, outlined Secondary Button, tokens, layout, elevation); `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` step 14, Page adoption row `TemplatePreview` (line 230), `PageShell`, `PageHeader`, `Card`, `FormGrid`, `SelectField`, `TextField`, `Notice`, `HintText`, `ErrorText`, `ButtonLink`, `.ui-preview-canvas`; `context/screens/INDEX.md` §2.3 |
| Component | `TemplatePreview` — `persona-sanus/src/main.rs:4048-4222` (`#[component]` at `:4048`, `fn` at `:4049`). Composes `forms::A4Preview` (`persona-sanus/src/forms.rs:464`), `forms::template_theme_margin` (`:966`), `forms::parse_document`, `forms::document_warnings`, `forms::sample_values`, and `api::fetch_template` / `api::fetch_assets`. Planned primitives live in `persona-sanus/src/ui/`; that directory is not shipped |
| Route | `/templates/:id/preview` — authenticated; `Route::TemplatePreview { id: String }` at `persona-sanus/src/main.rs:82-83`. Signed out replaces `/login` (`Route::AuthPage {}`). No query parameter; the template `id` in the path locates the record. A doctor may open own and clinic-master templates, an Administrator any clinic master. The page is read-only; it does not publish, archive, or edit |
| Layout | Inside `#[layout(Navbar)]` (`main.rs:51`, never closed). `PageShell id="templates-page"` → `div#templates-page.ui-page > .ui-container.ui-stack`; Paper header and controls, Cream preview canvas; no full-bleed exception, sidebar, or split editor |
| Locale | `es-EC` on every string, punctuation included. `plantilla`, `receta`, `médico`, `clínica`, `versión`, `márgenes`. Dates and times are not rendered; the A4 document owns its own date sample text |
| Status | ✅ Spec complete (sections 1–7). Route status: ✅ — sitemap §1.2: "True-scale A4 preview with sample variable values, repeated header/footer, margins/page breaks, and the pinned images before publishing." Shipped: auth-checked template load, version select, 5–40 mm margin input, warning notice for unknown variables, and `forms::A4Preview` at the chosen version. Target: step-14 shell adoption plus signed image sources from the existing asset read; no editor, publish, upload, or PDF action here |

The proof surface for a template: the operator sees the A4 projection at true scale, chooses a stored version, and checks margins, repeated bands, unknown variables, and referenced images before returning to the editor. It does not edit or publish.

## 1. Purpose

Four jobs, in order:

1. **Identify.** Name the template story, owner class, and lifecycle status above the sheet.
2. **Project.** Render the selected stored version's `doc_json` with the built-in sample values at 210 × 297 mm, 1:1 at the browser's CSS millimeter.
3. **Inspect variation.** Switch among the template's versions, move the preview margin within 5–40 mm, and surface unresolved variable keys as warning text rather than `{{key}}` placeholders.
4. **Resolve brand assets.** Pass the referenced logo and image-slot objects their short-lived URLs so the projection matches the publishable document.

It never renders:

- Patient PHI. The variable values are the fictional constants from `forms::sample_values`, never a patient, cédula, diagnosis, medication, or database row. No patient is fetched, chosen, or named.
- An editor, save, publish, archive, delete, duplicate, upload, or image-picker control. Editing is `/templates/:id/edit`; the library is `/settings/clinic`. This route has no primary action.
- A PDF, print, share, QR, or verification code. The preview is a browser projection; the `printpdf` renderer is authoritative and is not invoked here (D-01).
- Billing, price, trial, plan, image quota, or `sync_locked` copy (D-04). No signature or stamp image (D-06). No rich text, formatting toolbar, `contenteditable`, or HTML document source (D-21).
- An unpersisted draft: a version that is not in `Template.versions` cannot be selected or previewed by typing an id. No synthetic “current draft” beyond the returned versions.

## 2. Structure

One centered column: header, controls, warning, canvas. The sheet is the only large surface and is not nested in a second card.

| Order | Region | `id` / class | Band tone | Contents |
|---|---|---|---|---|
| — | Navbar (layout) | `header.ui-navbar` → `.ui-navbar-inner` | Paper | Owned by `context/screens/Header.md` §2–§3; not restated |
| 1 | Page shell | `PageShell id="templates-page"` → `div#templates-page.ui-page > .ui-container.ui-stack` | Paper `#ffffff` | 48px top, 80px bottom, 24px inline; container max 1200px; 24px stack gap |
| 2 | Page header | `header.ui-page-head` · `h1.ui-page-title` · `p.ui-page-subtitle` · `.ui-page-head-actions` | Paper | `Vista previa A4 — {model.name}`; `210 × 297 mm con valores de ejemplo.`; Secondary `Volver a plantillas` and `Abrir editor`; no Primary |
| 3 | Controls | `Card` + `FormGrid columns=Two` | Paper | `SelectField` `Versión`; `TextField` `Márgenes (mm)`, numeric, 5–40 step 1 |
| 4 | Variable warning | `Notice tone=Warning` | Blossom `#ffc3e6` | Title `Variables no válidas`; comma-separated unresolved keys; hint that the projection marks them as warning text, never as a literal `{{clave}}` |
| 5 | Preview canvas | `.ui-preview-canvas` | Cream `#fffded` | 24px padding; horizontally scrollable when the 210 mm sheet exceeds the viewport; every A4 page in order |
| 6 | A4 sheet | `.a4-pages` > `.a4-page` | Paper `#ffffff` | 210 × 297 mm; the selected version's letterhead, images, body, footer, page number; 8px radius, `--shadow-paper` |
| 7 | Empty state | `EmptyState` | Cream | Only when the template has no versions; title plus `Abrir editor` link; no canvas |
| 8 | Error | `ErrorText` | Paper | Load failure above the back link; no canvas, no sample document |
| 9 | Denied | `Notice tone=Error` | Paper | `Acceso denegado`, reason; no controls, canvas, or editor link |

Regions 7–9 replace the controls and canvas. The header remains mounted in the empty state; the denied state replaces the page header content, not the shell navbar.

### 2.1 Role and surface state

`checked` gates the session; `user()` must exist; `actor()` must exist; `Me::can_manage_templates()` must be true (`api.rs:68-70`, Administrator or Doctor). The server's `get_template` applies the same clinic and owner scope to every version; nursing and secretaries get `Plantilla no encontrada.` rather than a partial preview.

| Surface / role | Controls | Canvas | What changes |
|---|---|---|---|
| Web, doctor owner | version + margin | server `doc_json`, sample values, referenced asset URLs | full preview; no publish |
| Web, doctor, clinic master | version + margin | same | master rows are readable; no ownership action |
| Web, administrator, clinic master | version + margin | same | any clinic master; no publish |
| Desktop, online | version + margin | same | same markup and strings |
| Desktop, unreachable | none | `ErrorText` | the shipped effect has no local template fallback; the preview is unavailable offline |
| No session | — | — | replace `/login` |
| No role / nursing / secretary | — | — | denied notice |

Role changes only whether the page paints; it never changes the sheet, margin bounds, version labels, or warning text. The server remains the authorization boundary; these gates are UX.

### 2.2 Version state

The version select lists `Template.versions` in server order, labelled `Versión {version_no} — {status}`. It starts on `Template.current_version` when present, otherwise the first version. A template with no versions renders the empty state instead of an empty select plus blank sheet.

Switching the version does three things: sets the selected id, resets the margin to `forms::template_theme_margin(version.theme)` clamped to 5–40, and recomputes the document warnings from that version's `doc_json`. It does not persist, publish, or re-fetch the template. The warning list is derived from the selected version only.

The margin input changes the projection, not the stored document: it never writes `theme`. A version whose theme carries a margin starts there; the operator may move within 5–40 mm. Values outside the bounds are clamped, not accepted with an error.

### 2.3 Warnings and images

`forms::parse_document` failure is the document error state: the header and controls stay, the canvas is replaced by the error text, and the page offers no retry. Unknown variables from `forms::document_warnings` are a warning state: the canvas still renders, with each unresolved key painted as warning text rather than a literal `{{key}}`, and the `Notice` lists the keys.

The shipped `A4Preview` call passes only `doc_json`, `sample_values`, and `margin_mm`. `image_sources` is empty, so:

- a document without images projects correctly;
- an image slot with no bytes paints the renderer's `Imagen: {caption}` fallback;
- a document that references a clinic asset does not show it in this preview, even though the published PDF would.

Target uses the existing asset read, not a new endpoint: `api::fetch_assets()` (`api.rs:923`) returns `AssetLibrary.assets`, each `AssetRow { id, kind, url, status, ... }`; `forms::parse_document(version.doc_json)` exposes `letterhead.logo_asset_id` and `images[].asset_id`. Build `image_sources` as `(asset_id, url)` for ready rows whose id is referenced by the selected version, and pass it to `A4Preview`. The logo and image kinds both participate; the v1 quota still applies in the editor, not here. If the asset read fails, keep `image_sources` empty and let the renderer's placeholder speak; do not block the document projection on images. Signed URLs last 15 minutes, so the asset read happens on version change and on mount, not once per app session.

### 2.4 Margin and responsive state

The margin input is numeric, 5–40, step 1, with no unit suffix beside the field because the label carries `Márgenes (mm)`. Below 5 or above 40, the displayed value clamps. Empty input falls back to the version theme margin (or 15) on blur; it does not send a zero margin.

At 760px and below: header actions stack under the title, controls become one column, canvas padding drops to 16px, and the sheet keeps its 210 mm width with horizontal scroll. At 320px: the sheet is not scaled down, cropped without a scroll affordance, or reflowed into a fake mobile page; the canvas scrolls in both axes, with a visible focus ring when it is the scroll target.

## 3. Copy text

Every visible string is `es-EC`, quoted with final punctuation where the source has it. Chrome copy belongs to `context/screens/Header.md` §3 and is not repeated.

### 3.1 Page header and actions

| Element | Copy | Target |
|---|---|---|
| Page title (`h1`) | `Vista previa A4 — {model.name}` | — |
| Page subtitle | `210 × 297 mm con valores de ejemplo.` | — |
| Back action, Secondary | `Volver a plantillas` | `/templates` (`Route::Templates {}`) |
| Editor action, Secondary | `Abrir editor` | `/templates/{id}/edit` (`Route::TemplateEdit { id }`) |

The shipped subtitle reads `Proyección 210 × 297 mm con valores de ejemplo.`; the target string is the UI plan's `210 × 297 mm con valores de ejemplo.` The shipped title is `Vista previa A4 — {model.name}` and is retained.

### 3.2 Controls and warnings

| Element | Copy |
|---|---|
| Version label | `Versión` |
| Version option | `Versión {n} — {status}` |
| Margin label | `Márgenes (mm)` |
| Warning title | `Variables no válidas` |
| Warning hint | `En la proyección se marcan como texto de advertencia, nunca como {{clave}}.` |
| No-versions title | `La plantilla todavía no tiene versiones para previsualizar.` |
| No-versions action | `Abrir editor` |
| Load error fallback | `No se pudo cargar la vista previa.` |
| Loading gate | `Cargando...` |
| Redirect gate | `Redirigiendo...` |
| Denied title | `Acceso denegado` |
| Denied body | `Solo doctor y administrador pueden ver la vista previa.` |

Status labels in the version options are `Borrador`, `Publicada`, `Archivada`, `Desconocido` through `version_status_label`.

### 3.3 A4 document

| Element | Copy |
|---|---|
| Missing image fallback | `Imagen: {caption}` |
| Provisional QR label | `QR provisional` |
| Verification QR label | `QR de verificación` |
| Page number | `Página {page} de {total}` |
| Printed signature rule | `Firma: ______________________________` |

The document body, letterhead, footer, variables, and medication lines are stored template content plus `forms::sample_values`; they are not page-owned copy; never a real patient value. The page adds no caption, watermark, or “example” badge over the sheet.

## 4. Displayed information

### 4.1 What the screen shows

| Element | Source | Value shown |
|---|---|---|
| Page title | `model.name` and the route | `Vista previa A4 — {name}` |
| Subtitle | static | `210 × 297 mm con valores de ejemplo.` |
| Header actions | `model.id` | Secondary `Volver a plantillas` and `Abrir editor` |
| Template identity | `api::fetch_template(id)` → `api::Template` | authorized template record for this path id |
| Role gate | `Me::can_manage_templates()` (`api.rs:68`) | Administrator/Doctor paint; other roles denied |
| Version options | `Template.versions` | one per stored version: `Versión {version_no} — {version_status_label(status)}` |
| Initial version | `Template.current_version` else `versions.first()` | current version preferred |
| Version document | `api::TemplateVersion.doc_json` | parsed by `forms::parse_document`; no HTML, no raw text injection |
| Margin | `forms::template_theme_margin(version.theme)`; local override; clamped 5–40 | per-page padding in mm |
| Sample values | `forms::sample_values()` | static fictional values for the supported variables |
| Variable warnings | `forms::document_warnings(&document)` | list of unresolved keys; empty hides the `Notice` |
| Logo/image ids | parsed `letterhead.logo_asset_id` and `images[].asset_id` | matched against `AssetRow.id`, not painted raw |
| Image URLs | `api::fetch_assets()` → ready `AssetRow.url` | signed 15-minute URL passed as `A4Preview.image_sources`; absent when the read fails or the row is not ready |
| A4 geometry | `forms::A4Preview` (`forms.rs:464`) | 210 × 297 mm sheet, repeated header/footer, page breaks, page number, QR/verification area |
| No versions | `Template.versions.is_empty()` after load | empty state title + editor link; no select, no canvas |
| Error | `message` non-empty after fetch | `ErrorText` with the returned message or §3.2 fallback; no fabricated document |
| Denial | no session/role or non-manager | redirect or denial notice; no controls or document |

The page issues one template read on mount and one asset read on mount/version change when the selected document references assets. It never reads patients, prescriptions, audit rows, or billing. It never writes.

### 4.2 Hard rules

- Server-side authorization is the boundary. The page checks `can_manage_templates()` and the server's `get_template` checks clinic plus owner access on every read; the client gate is UX only.
- The sheet is a projection, not an issued PDF. 210 × 297 mm at CSS 1 mm = 1 px-equivalent; `printpdf` remains authoritative (D-01, FR-022). Never claim byte-identical output and never call `pdf::render` from this screen.
- The preview never publishes, archives, saves, deletes, duplicates, uploads, or changes an asset. Publish and archive stay in `/templates/:id/edit`.
- The margin control is a projection control. It does not write `theme`, `doc_json`, or any stored value; no autosave exists.
- Only stored versions are selectable. The page does not synthesize a draft, fetch an arbitrary version by id, or accept typed version input.
- No PHI: `forms::sample_values` are constants, not database data. No patient name, cédula, birth date, diagnosis, medication, or real prescription number appears. The template's own letterhead/body may contain generic sample prose, but the page must not join it to a patient record.
- No PHI in the URL: only the opaque template `id` is in the path. Asset URLs are signed and short-lived, not page copy; they must not be placed in a visible link, title, analytics event, or log line.
- Plain text only (D-21): document values are structured text resolved by the forms renderer. No HTML payload, rich-text editor, `contenteditable`, or markdown rendering exists here.
- No billing, price, trial, plan, image quota, or `sync_locked` copy (D-04). The v1 one-image quota belongs to the editor and clinic settings; this preview neither enforces nor displays it.
- No signature or stamp image, field, or placeholder (D-06). The printed signature rule is text owned by the document footer, not a drawn signature.
- No primary action. `Volver a plantillas` and `Abrir editor` are Secondary; `Abrir editor` is the same route the editor owns. There is no save/publish/download/print/share.
- One `h1` in the header and no `h2`; the controls `Card` has no heading, so no section level is introduced.
- Unknown variables are warning text, never literal `{{key}}` in the projection (FR-023). Missing image bytes are a labelled placeholder, not a broken image or invented logo.
- No new route, endpoint, table, column, role, or permission. The only reads are `api::fetch_template` and `api::fetch_assets`.

### 4.3 Dynamic states

| State | Screen behaviour |
|---|---|
| Session check in flight | `PageShell` + `LoadingState` `Cargando...`; no header, controls, or canvas |
| Checked, no session | Effect replaces with `/login`; no template paint |
| Checked, `actor == None` | `LoadingState` `Redirigiendo...`; no fetch, no retry |
| Not a manager | Denied `Notice`; no version select, margin, canvas, or editor link |
| Template fetch in flight | Header + controls skeleton? No—`LoadingState` occupies the page; no empty sheet |
| Fetch error | `ErrorText` with the returned message or `No se pudo cargar la vista previa.`; Secondary back only; no canvas |
| Template with no versions | `EmptyState` title `La plantilla todavía no tiene versiones para previsualizar.` + `Abrir editor`; no select, no margin, no sheet |
| Versions present | Select seeded to current or first; margin seeded from the selected theme; canvas renders |
| Version changed | Select updates; margin resets to that version’s theme margin clamped 5–40; warnings re-derive; asset URLs re-read; no write |
| Margin edited | Canvas re-paginates at the new mm; value clamps 5–40; no write; version select unchanged |
| Margin cleared | On blur, reverts to the version theme margin (or 15); never zero |
| Unknown variables | `Notice` Warning with keys; canvas still renders, unresolved keys as warning text, never `{{key}}` |
| Parse failure | Controls stay; canvas replaced by the document error text; no retry button |
| Referenced assets ready | Logo/image slots paint the signed URL |
| Referenced asset pending/missing | Slot paints `Imagen: {caption}`; the document still projects; no upload control |
| Asset read fails | Canvas still renders with placeholders; no page-level error replaces the document; no new endpoint |
| Archived template | Same read-only projection; no publish/archive control exists here to hide |
| Desktop offline | No local fallback in the shipped effect: fetch error; back link only. Do not claim cached preview |
| Signed URLs expire | The projection may lose an image until the next read; do not cache URLs or claim permanence |
| Mobile / 320px | Controls one column; canvas scrolls horizontally; sheet not scaled or cropped |
| Reduced motion | No animation; no transition on select, margin, or warning |

## 5. Style

Every value is a `context/Design.md` token declared in `persona-sanus/assets/main.css` `:root`. No new color, font, icon package, CDN, or webfont.

### 5.1 Color

| Use | Value | Token |
|---|---|---|
| Page canvas, header, controls card | `#ffffff` | `--color-paper` |
| Page title, labels, control values | `#061d29` | `--color-ink` |
| Subtitle, hints, Secondary action text and border | `#425d6d` | `--color-mist` |
| Card and control hairlines | `#e5e7eb` | `--color-hairline` |
| Focus ring | `#006e75` | `--color-deep-teal` |
| Preview canvas wash | `#fffded` | `--color-cream` |
| Warning notice surface | `#ffc3e6` | `--color-blossom` |
| Warning notice text and Alert glyph | `#4d0037` | `--color-plum` |
| Focus halo | `rgba(185,255,232,0.55)` | `--color-mint-glass` |

The A4 sheet is Paper; its letterhead colors and image content come from the selected `doc_json`, never from this page's palette. The page does not recolor a stored document. Never `#000000`; every border is the hairline. Measured contrast: Ink/Paper 17.28 · Mist/Paper 6.96 · Paper/Deep Teal 6.02 · Plum/Blossom 10.36 · Ink/Cream 16.89 · Mist/Cream 6.80. AA for body.

### 5.2 Typography

| Element | Family | Weight | Size / line-height |
|---|---|---|---|
| Page title (`h1`) | `--font-mint` | 500 | 22px / 1.25 Ink |
| Page subtitle | `--font-mint` | 400 | 14px / 1.5 Mist |
| Control label | `--font-mint` | 500 | 14px / 1.5 Ink |
| Select value, margin input | `--font-mint` | 400 | 16px / 1.5 Ink |
| Header action | `--font-mint` | 500 | 16px / 1.22 |
| Warning title | `--font-mint` | 500 | 16px / 1.5 Ink |
| Warning keys and hint | `--font-mint` | 400 | 14px / 1.5 Plum / Mist |
| Empty title | `--font-mint` | 500 | 18px / 1.25 Ink |
| Error text | `--font-mint` | 400 | 14px / 1.5 Plum |
| A4 document | embedded document typography | — | 210 × 297 mm projection |

No `--font-grenette` on the shell: this is an application surface. The template's own typography is the pinned document's, not page chrome, and is never restyled by this screen.

### 5.3 Geometry

- 4px base. `PageShell`: 48px top, 80px bottom, 24px inline; `.ui-container` max 1200px; `.ui-stack` 24px gap.
- `PageHeader`: flex row, space-between, wrap, 16px gap, align start; title/subtitle 4px apart; actions Secondary box `12px 24px`, 8px radius, 12px gap.
- Controls `Card`: 24px radius, 1px hairline, 24px padding; `FormGrid` 2 columns, 16px gap; below 760px one column. Select and input `min-height: 44px`, `8px 12px` padding, 8px radius, 1px hairline. Margin field width fits three digits; no stepper chrome beyond the native number input.
- Warning `Notice`: 24px radius, 20px padding, 12px icon-to-body gap, 20px Alert glyph; keys wrap; sits 16px below the controls and 16px above the canvas.
- Preview canvas: Cream, 24px radius, 24px padding; horizontal scroll when the sheet is wider; the A4 sheet is 210 mm × 297 mm, 8px radius, Paper, `--shadow-paper`; pages stack with a 16px gap.
- At 760px and below: header actions stack; controls one column; canvas padding 16px. At 320px: the sheet keeps 210 mm and the canvas scrolls; no clipped right edge.
- Elevation: flat shell; the A4 sheet is the only shadowed surface. No card hover lift, no badge shadow, no sticky control bar.

### 5.4 Iconography and imagery

Line icons only, 1.5px stroke, 20px default, Ink or Plum: the Alert glyph in the warning/`ErrorText`. Header actions are text, not icons. No page illustration, stock photo, patient image, clinic logo as chrome, signature, stamp, or QR code outside the A4 sheet. The sheet may render the template's referenced logo/images and its footer QR area; those are document content from the pinned version, with alt text from §3.3. The page adds no decorative document mock.

## 6. Accessibility

- Landmarks: the navbar is the banner (`Header.md` §6). `PageShell` is the content region; the controls `Card` is a plain group; the preview canvas is a labelled region (`aria-label` `Vista previa de la receta`) and the only scrollable surface when the sheet overflows for the current viewport (`tabindex="0"` only when it actually overflows, never by default).
- Headings: exactly one `h1`, `Vista previa A4 — {name}`. No `h2`; the controls card has no heading. The A4 document's internal headings are non-interactive text inside the panel and do not enter the page outline as heading landmarks.
- Tab order: `Volver a plantillas` → `Abrir editor` → `Versión` select → `Márgenes (mm)` input → canvas scroll area when focusable. The warning Notice and empty state add no controls beyond the editor link; the denied state has no controls.
- Focus: shared utility — `outline: 1px solid var(--color-deep-teal); outline-offset: 2px; box-shadow: 0 0 0 3px rgba(185,255,232,0.55);` — on both header links, select, number input, and scrollable canvas. Visible on Paper and Cream. Never remove it for the select’s native focus.
- Labels: `Versión` and `Márgenes (mm)` wrap their controls; the unit is in the margin label, not a suffix that screen readers may skip. Numeric input exposes `min=5`, `max=40`, `step=1`; no `aria-label` duplicates the visible name. The canvas has a concise accessible name, not the document's body.
- Status: the warning title and the paragraph above the sheet are the complete signal. The note is not an arbitrary tooltip. The version option includes its own status word, not only a badge colour. The margin never announces a raw out-of-range value; it clamps and displays the applied value.
- Loading and error: `LoadingState` is `role="status"` and `aria-live="polite"`; the spinner is `aria-hidden`. `ErrorText` is `role="alert"`, inserted without moving focus. A parse failure is text in the canvas, not a silent blank.
- Keyboard and touch: native `select` and `input[type=number]`; no custom combobox, no slider-only margin, no drag. The canvas horizontal scroll is keyboard-reachable when it overflows. All targets are at least 44px.
- Reduced motion: no transition, spinner, or page animation on this screen; if a shared spinner is used during load, it stops under `prefers-reduced-motion: reduce`. The A4 pages do not fade or animate on version change.
- Zoom/reflow: at 200% and 320px the shell reflows to one column; the A4 sheet keeps its physical 210 mm width and the canvas scrolls. The document is never cropped without a scroll affordance, and no horizontal scroll appears on the page shell itself.
- Do not rely on the visual sheet alone: the page title, version option text, margin value, warning keys, and parse error carry the information in text.

## 7. Implementation notes

The shipped component is `fn TemplatePreview(id: String)` at `persona-sanus/src/main.rs:4049`, `#[component]` at `:4048`, closing before `template_status_label` at `:4223`. Signals: `template: Option<api::Template>`, `selected: Option<String>`, `margin: u32` (initial 15), `message: String`; `load_id = id.clone()`.

Behavior to preserve:

- The effect gates on `checked` and `user`; a missing session replaces `/login`; it calls `api::fetch_template(&load_id)` once (no generation/reload signal).
- On success it picks `found.current_version.as_ref().or(found.versions.first())`, sets `selected` to that version id, sets margin to `forms::template_theme_margin(version.theme)` (15 when no version), and stores the template.
- On failure `message` holds the API error; the page shows `ErrorText` + `Volver a plantillas`. There is **no desktop local fallback** in this component, so a desktop offline preview is an error, not a cached sheet. Record this honestly; do not add cache copy.
- The role gate is `profile.can_manage_templates()`; denial copy is `Solo doctor y administrador pueden ver la vista previa.`; no back link in the denial.
- The version select's `onchange` sets `selected`, resets `margin` to `template_theme_margin` for the chosen version, and does not refetch. Options use `version.id` as value and `Versión {version_no} — {version_status_label(status)}` as text.
- The margin input is `type=number`, `min=5`, `max=40`, no `step` in the shipped markup; it parses to `u32` and clamps 5–40, keeping the old value on parse failure.
- Warnings: `forms::parse_document(&version.doc_json)` then `forms::document_warnings(&document)`; `warnings.join(", ")`; a non-empty list paints `Variables no válidas: {joined}` plus the hint. Empty list hides the notice.
- Canvas: `div.preview-scroll` > `forms::A4Preview { doc_json: version.doc_json.clone(), values: forms::sample_values(), margin_mm: margin() }`. The shipped call does **not** pass `medications`, `meta`, `image_sources`; `A4Preview` defaults medications to the document blocks, meta to empty, and image sources to an empty vector. This preview therefore has no verification code/QR and no image bytes. Do not claim otherwise.
- Page head: `h4` `Vista previa A4 — {model.name}` and `p` `Proyección 210 × 297 mm con valores de ejemplo.`; actions are `Link.button.secondary` `Volver a plantillas` to `Templates` and `Abrir editor` to `TemplateEdit`.
- No Primary, no publish, no archive, no upload, no PDF/print/share, no save.

Plan step 14 adoption:

- Root becomes `PageShell id="templates-page"`; `h4` becomes `PageHeader` `h1` `Vista previa A4 — {model.name}` with the target subtitle `210 × 297 mm con valores de ejemplo.`; `Volver a plantillas` and `Abrir editor` become Secondary `ButtonLink`s.
- Controls become `Card` + `FormGrid columns=Two`: `SelectField` `label="Versión"`, `value=selected`, `onchange` preserving the reset; `TextField` `label="Márgenes (mm)"`, `input_type="number"`, `min=5`, `max=40`, `step=1`, `value=margin`, `oninput` preserving the clamp. No primary/action row.
- Warning becomes `Notice tone=Warning` with the title `Variables no válidas` and one line of keys; the explanatory hint stays `HintText` or paragraph. Do not use `ErrorText`: the document is previewable.
- Canvas wrapper becomes `.ui-preview-canvas`; `forms::A4Preview` keeps `doc_json`, `values: forms::sample_values()`, `margin_mm`, and add `image_sources` as specified below. The `.a4-*` projection classes and page geometry are untouched.
- Image sources: parse the selected `doc_json`; collect `letterhead.logo_asset_id` plus each `images[].asset_id`; call `api::fetch_assets()` once per selected version; keep `kind == "logo" || kind == "image"`, `status == "ready"`, non-empty `url`; pass `(asset_id, url)` pairs. This uses the existing asset endpoint and the doctor's patient-read entitlement (`service::list_assets`), adds no route. A failed asset read leaves the vector empty and does not replace the document with an error; the renderer's placeholder is the honest fallback. Re-read on version change because URLs expire in 15 minutes.
- Empty state: `EmptyState` title only plus a Secondary `ButtonLink` `Abrir editor`; no `SelectField`, no margin field, no canvas.
- Error state: `ErrorText` with the frozen fallback; Secondary `Volver a plantillas`; no empty canvas.
- Do not add a reload/generation signal, polling, autosave, publish, or upload. The shipped component fetches once.

Verification commands, from the repo root; none is recorded here as executed:

```sh
cd persona-sanus && cargo check
dx build --release --fullstack
caveman-blocks run persona-smoke --dir persona-sanus --port 8091
python3 .blocks/markdown-table-check.py --path context/screens/TemplatePreview.md --strict
```

A browser pass is needed for: a template with one version and no images; a version with a referenced logo/image; a version with an unknown variable; a template with zero versions; and an archived template. The desktop offline case must show the error, not a cached sheet.

- Do not add: an editor, save, publish, archive, delete, duplicate, rename, restore, upload, or image-picker control.
- Do not add: a PDF, print, share, QR, or verification-code action; no `pdf::render` call.
- Do not add: a margin persistence write, autosave, debounce-save, or “apply to template” action.
- Do not add: a version create/delete control, version number input, or a synthetic draft version.
- Do not add: patient PHI, a patient picker, real sample values, or a name from the session.
- Do not add: billing, price, trial, plan, image quota, or `sync_locked` copy.
- Do not add: signature or stamp imagery, a digital-signature badge, or a signing affordance.
- Do not add: rich text, `contenteditable`, HTML source, markdown rendering, or a formatting toolbar.
- Do not add: a new route, endpoint, table, column, role, permission, or client-side owner filter.
- Do not add: a second `h1`, painting the raw template id, or claiming byte-identical renderer output.
