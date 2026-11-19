# PersonaSanus — SettingsClinic (`/settings/clinic`)

| Field | Value |
|---|---|
| Source | `context/sitemap.md` §1.2 (`/settings/clinic` — clinic data, logo, image library, timezone, `llm_url_template`, retention, export), §2 (Administrator only), §4 M1/M3, §5 (images, signed URLs, no PHI in keys); `context/PRD.md` FR-020, FR-080–FR-085, §6 `clinics`/`assets`, §8 `es-EC`, §9 images, D-04, D-05, D-06, D-12, D-21; `context/Design.md` (Card, Section Card, Form Input, File Input, Green Pill Badge, Tokens, Surfaces, Elevation); `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` step 16, Page adoption row `SettingsClinic` (line 232), `ImageLibrary`, `QuotaMeter`, `PageShell`, `PageHeader`, `SectionCard`, `FormGrid`, `TextField`, `FileField`, `LinkAction`, `ErrorText`, `HintText`; `context/screens/INDEX.md` §2.3 |
| Component | `SettingsClinic` — `persona-sanus/src/main.rs:1804-2118` (`#[component]` at `:1803`, `fn` at `:1804`). Reads `actor: Signal<Option<Me>>` and `checked: Signal<bool>`; holds `name`, `address`, `phone`, `llm_url`, `message`, `saving`, `uploading`, `generation`, `settings: Option<api::ClinicSettings>`, `library: Option<api::AssetLibrary>`. Target composes planned `src/ui/` primitives and `ImageLibrary`/`QuotaMeter`; that directory is not shipped. `Header.md` owns the shell band; this file owns nothing above the page |
| Route | `/settings/clinic` (`Route::SettingsClinic {}`, `main.rs:72-73`) — authenticated shell, Administrator only. No session replaces `/login`; a non-Administrator replaces `/settings/profile`; an Administrator stays. No query or path parameter |
| Layout | Inside `#[layout(Navbar)]` (`main.rs:51`, never closed). `PageShell id="settings-page"` → `div#settings-page.ui-page > .ui-container.ui-stack`; Paper canvas, 48px top / 80px bottom / 24px inline, 24px stack gap; four `SectionCard`s in order: clinic data, logo, image library, export. No full-bleed band, tabs, sidebar, or preview pane |
| Locale | `es-EC` on every string. `Clínica`, `Dirección`, `Teléfono`, `Retención`, `gracia`, `imagen`, `versión`. Server dates render as sent (`DD-MM-AAAA` target); no money, no time |
| Status | ✅ Spec complete (sections 1–7). Route status: ✅ — sitemap §1.2 ships Administrator-only clinic settings (name/address/phone/logo, timezone, `llm_url_template`, local retention days, one-image library, export). Shipped: clinic data + LLM template save, logo upload, one-image library upload/delete, read-only retention/grace dates, JSON export; timezone and retention days are returned/read-only, not editable. Target: step-16 `SectionCard`/`ImageLibrary`/`QuotaMeter` adoption with named callbacks and offline-disabled uploads |

## 1. Purpose

The clinic-branding and data-retention console for one Administrator. It owns five jobs:

1. **Identity.** Edit the clinic's display name, address, and phone; the server supplies timezone and the tenant id.
2. **Branding.** Upload or replace the one logo (a `kind='logo'` asset that does not consume the library quota) and preview the current image.
3. **Library.** Show the clinic image library's one v1 slot, the current image and its metadata, and allow deletion only when no published version or retained prescription pins it.
4. **LLM template.** Edit the per-clinic `llm_url_template` that `/prescriptions/:id` fills with a medication name; it is a URL template, never a patient payload.
5. **Retention visibility.** Show the server retention and export-grace dates read-only, and offer the existing JSON export so the clinic can leave before the grace ends.

It never renders:

- Patient PHI. No patient name, cédula, diagnosis, medication, prescription object, or patient image is fetched or painted. The image library is branding content, not clinical attachment storage; object keys carry no PHI (FR-083).
- Billing, price, trial, plan, entitlement, `sync_locked`, or subscription copy (D-04). Billing is v1.1 and unrouted; `/settings/billing` has no link here.
- A signature or stamp image, upload slot, or preview (D-06). The logo is the only branding image.
- A rich-text editor, formatting toolbar, `contenteditable`, HTML document source, or markdown renderer (D-21).
- A second filled Primary. `Guardar datos` is the only Primary; file inputs are native file controls; delete is Danger and only on an existing library image.
- A clinic picker, tenant switch, role editor, staff table, audit row, patient search, prescription list, template list, or PDF/QR preview.
- A `.pdf` / `.svg` / WebP upload path: the accept list is JPEG and PNG only, and the server rejects the rest.

## 2. Structure

One centered page column, Paper throughout, four `SectionCard`s in a 24px stack. The page header is the only header; the navbar is not restated.

| Order | Region | `id` / class | Band tone | Contents |
|---|---|---|---|---|
| — | Navbar (layout) | `header.ui-navbar` → `.ui-navbar-inner` | — | Owned by `context/screens/Header.md` §2–§3 |
| 1 | Page canvas | `PageShell id="settings-page"` → `div#settings-page.ui-page > .ui-container.ui-stack` | Paper `#ffffff` | 48px top, 80px bottom, 24px inline; container max 1200px; 24px stack gap |
| 2 | Page header | `header.ui-page-head` · `h1.ui-page-title` · `p.ui-page-subtitle` · `.ui-page-head-actions` | Paper | `Clínica`; `Datos de la clínica, logo y biblioteca de imágenes.`; Secondary `Volver` → `/settings` |
| 3 | Clinic data card | `SectionCard` `Datos de la clínica` · `div.ui-card` | Paper, 1px hairline | `FormGrid columns=Two`: `Nombre`, `Dirección`; `Teléfono`; `Plantilla del enlace LLM` wide; read-only timezone; Primary `Guardar datos` |
| 4 | Logo card | `SectionCard` `Logo` | Paper, hairline | 48px/`img.ui-asset-preview` current logo or the one-line empty hint; `FileField` `Subir logo…`; upload hint |
| 5 | Library card | `SectionCard` `Biblioteca de imágenes` · `ImageLibrary` | Paper, hairline | `QuotaMeter`; one image row with metadata + Danger `Eliminar`; pending row `Procesando`; `FileField` upload; empty state; online-only hint |
| 6 | Export card | `SectionCard` `Exportación de datos` | Paper, hairline | read-only retention and grace dates, explanation, Secondary `LinkAction` `Exportar datos (JSON)` |
| 7 | Feedback | `ErrorText` | Paper | One page-level `role="alert"` line above the cards, or field-level input errors inside the data card |
| 8 | Session/redirect gates | `LoadingState` | Paper | `Cargando...` before the session/profile check; `Redirigiendo...` for a non-Administrator |

Regions 3–6 render in that order for an Administrator only. Region 7 does not replace any card; a settings-save failure leaves the form values, and an asset failure leaves the clinic fields usable. There is no modal: the delete action is direct Danger, with the server refusal in region 7.

### 2.1 Role or surface state

`SettingsClinic` requires an Administrator profile (`Me::is_administrator()`, `api.rs:49-51`) and a session. The effect:
- no session → `navigator.replace(Route::AuthPage {})`;
- non-Administrator → `navigator.replace(Route::SettingsProfile {})`;
- Administrator → fetches settings and the asset library.

| State | What renders |
|---|---|
| `checked` false | `PageShell` + `LoadingState` `Cargando...`; no cards and no fetch |
| Checked, no session | Replace `/login`; no page content |
| Checked, no profile | Same: render `Cargando...` until the replace |
| Non-Administrator | Replace `/settings/profile`; render `Redirigiendo...` until then. No reduced clinic hub |
| Administrator, web online | Full four-card config; save, logo upload, library upload/delete live |
| Administrator, desktop online | Same four cards and same requests; logo states that cached images render offline, but uploads/deletes need the server |
| Administrator, desktop offline | Clinic fields still edit locally; both upload controls disabled with `Subir imágenes requiere conexión.`; library delete disabled; export disabled or explains it requires the server |
| Settings fetch fails | `ErrorText` with the returned message or fallback; the data card fields stay empty; the library card still renders if its read succeeded |
| Assets fetch fails | `ErrorText` from the asset read; the clinic data card still usable; no image rows are invented; the quota meter is absent |

### 2.2 Clinic data card

Fields: `Nombre` (required by the server), `Dirección`, `Teléfono`, `Plantilla del enlace LLM` (full-width, placeholder `https://chat.openai.com/?q=`). `Guardar datos` submits all four strings in one `PUT /api/clinic`. It is busy-disabled with `Guardando...` until the request settles.

The timezone (`ClinicSettings.timezone`) is returned by the API but `update_clinic_settings` does not accept it. It is a read-only fact on this screen (a hint line), not an editable input; the profile/diagnostics own the value. The local retention **days** (`retention_days`) is not in `ClinicSettings` and has no field here. The card shows `server_retention_until` and `data_grace_ends_at` as read-only dates in the export card; it never claims the days value is editable.

Success replaces the local `settings` snapshot from the next read and shows a success `HintText`; the page does not optimistically rewrite the server row. Failure keeps every typed value intact and shows `ErrorText`.

### 2.3 Logo card

The logo is the clinic's `kind='logo'` asset. `ClinicSettings.logo.url` renders a 48px preview (or the shipped CSS box) with alt `Logo de la clínica`; no logo renders `La clínica todavía no tiene logo.` The file field accepts `image/jpeg,image/png` and is labelled `Subir logo (JPEG o PNG, máximo 10 MB)`. The server validates magic bytes/decode, strips EXIF/GPS, enforces 10 MiB, stores the object privately, and returns a 15-minute signed URL. Upload is online-only; offline disables the field and shows the connection hint. A new logo replaces the old one; the logo does not count against the one-image library quota (FR-084). The page has no crop, color, or remove-logo control.

### 2.4 Image library card

`ImageLibrary` lists only `kind='image'` rows. v1 quota is **1** image, at most **10,485,760 bytes** (10 MiB), JPEG/PNG only; WebP/SVG are rejected by the server, not silently converted. The quota meter copy is `Uso de la biblioteca: {used} de {limit}.`; it never turns red or changes meaning because the fill is a progress indicator, not a validation state. A pending upload row shows a Cream `Procesando` badge and no thumbnail; a ready row shows a 48px thumbnail when its signed `url` exists, else the `Image` icon placeholder on a hairline. The row shows the original name or `Imagen {first 8 id chars}`, `{width} × {height}` (an em dash for a missing dimension), and `{format_bytes(size_bytes)}`. Delete is Danger, disabled offline or while the delete is in flight; the server refuses when a published template version or retained prescription pins the asset (FR-085Rev, D-05).

The upload field label is `Subir imagen de la biblioteca (JPEG o PNG, máximo 10 MB)`, accept `image/jpeg,image/png`. It is disabled when offline, uploading, or `used >= limit`; the hint is `Subir imágenes requiere conexión.` offline, `La biblioteca está llena.` at the quota, otherwise the 10 MB rule. Empty library: `Sin imágenes` / `Sube una imagen JPEG o PNG de hasta 10 MB.` The logo upload does not appear in this list and does not decrement the quota.

The page does not crop, resize, or transform images. The server's stored dimensions and size are what render. Bucket keys and signed URLs are never user-facing copy; a failed URL leaves the placeholder and the row metadata. Re-read the library after upload/delete; the signed URL is short-lived.

### 2.5 Export card

The export card shows the two server dates when present:
- `Retención en servidor hasta: {date}`
- `Fin del período de gracia para exportación: {date}`

and the explanation `Exporte los datos de la clínica (JSON: pacientes, recetas y medicamentos) antes de que finalice el período de gracia. La exportación queda registrada en la auditoría.` The action is a Secondary `LinkAction` to `/api/clinic/export` labelled `Exportar datos (JSON)`. It is a server GET; the page does not build a client archive, does not put PHI in the URL, and does not open a modal. It requires an online session and a server-authorized Administrator. When the dates are absento, the lines are absent rather than invented.

## 3. Copy text

All strings verbatim, `es-EC`; punctuation belongs to the copy. Navbar copy is owned by `Header.md` §3 and not repeated.

### 3.1 Page and clinic data

| Element | Copy |
|---|---|
| Page title (`h1`) | `Clínica` |
| Page subtitle | `Datos de la clínica, logo y biblioteca de imágenes.` |
| Back action | `Volver` |
| Clinic data heading (`h2`) | `Datos de la clínica` |
| Name label | `Nombre` |
| Address label | `Dirección` |
| Phone label | `Teléfono` |
| LLM template label | `Plantilla del enlace LLM` |
| LLM placeholder | `https://chat.openai.com/?q=` |
| Save action | `Guardar datos` |
| Save busy | `Guardando...` |
| Save success | `Datos de la clínica actualizados.` |
| Save failure | returned server message, unchanged |
| Read-only timezone | `Zona horaria: {timezone}` |

### 3.2 Logo

| Element | Copy |
|---|---|
| Logo heading (`h2`) | `Logo` |
| Image alt | `Logo de la clínica` |
| No logo | `La clínica todavía no tiene logo.` |
| File label | `Subir logo (JPEG o PNG, máximo 10 MB)` |
| Upload success | `Logo actualizado.` |
| Upload failure | returned server message, unchanged |
| Offline hint | `Subir imágenes requiere conexión.` |
| Unreadable file | `No se pudo leer el archivo.` |

### 3.3 Image library

| Element | Copy |
|---|---|
| Library heading (`h2`) | `Biblioteca de imágenes` |
| Quota meter | `Uso de la biblioteca: {used} de {limit}.` |
| Upload label | `Subir imagen de la biblioteca (JPEG o PNG, máximo 10 MB)` |
| Delete action | `Eliminar` |
| Deleting state | `Eliminando...` |
| Pending badge | `Procesando` |
| Offline hint | `Subir imágenes requiere conexión.` |
| Full hint | `La biblioteca está llena.` |
| Empty title | `Sin imágenes` |
| Empty description | `Sube una imagen JPEG o PNG de hasta 10 MB.` |
| Success | `Imagen agregada a la biblioteca.` |
| Delete success | `Imagen eliminada.` |
| Pinned/retained refusal | the server's sentence, unchanged |
| Unreadable file | `No se pudo leer el archivo.` |

No “remove background”, crop, rotate, reorder, alt-text, caption, gallery, or multi-image copy exists in v1. The one-image quota is not a billing tier.

### 3.4 Export

| Element | Copy |
|---|---|
| Export heading (`h2`) | `Exportación de datos` |
| Retention line | `Retención en servidor hasta: {date}` |
| Grace line | `Fin del período de gracia para exportación: {date}` |
| Explanation | `Exporte los datos de la clínica (JSON: pacientes, recetas y medicamentos) antes de que finalice el período de gracia. La exportación queda registrada en la auditoría.` |
| Export action, Secondary | `Exportar datos (JSON)` → `/api/clinic/export` |

### 3.5 Gates and fallbacks

| Element | Copy |
|---|---|
| Session gate | `Cargando...` |
| Non-Administrator redirect | `Redirigiendo...` |
| Denied (if a guard paints before redirect) | `Acceso denegado` · `Solo el administrador puede gestionar la clínica.` |
| Settings error fallback | `No se pudo cargar la configuración de la clínica.` |
| Asset error fallback | `No se pudo cargar la biblioteca de imágenes.` |

## 4. Displayed information

### 4.1 What the screen shows

| Element | Source | Value shown |
|---|---|---|
| Page title/subtitle | static | `Clínica`, `Datos de la clínica, logo y biblioteca de imágenes.` |
| Name | `api::fetch_clinic_settings` → `ClinicSettings.name`; local `name` signal | clinic display name; editable |
| Address | `ClinicSettings.address: Option<String>` | stored text or empty; editable |
| Phone | `ClinicSettings.phone: Option<String>` | stored text or empty; editable |
| Timezone | `ClinicSettings.timezone` | `Zona horaria: {timezone}` read-only; no control |
| LLM template | `ClinicSettings.llm_url_template: Option<String>` | stored URL template or empty; editable; placeholder is illustrative, never a value |
| Save | `api::update_clinic_settings(name, address, phone, llm_url_template)` | one PUT; success hint or server error |
| Logo | `ClinicSettings.logo` (`AssetRow`, `kind='logo'`) | 48px signed-URL preview or empty hint; alt `Logo de la clínica` |
| Logo upload | `api::upload_logo(file_name, bytes)` | one file, JPEG/PNG, 10 MiB; online only; then re-read |
| Library | `api::fetch_assets` → `AssetLibrary.assets` filtered to `kind == "image"` | zero or one row; never the logo |
| Quota | `AssetLibrary.quota.used`, `.limit` | `Uso de la biblioteca: {used} de {limit}.` |
| Image metadata | `AssetRow.original_name`, `.width`, `.height`, `.size_bytes`, `.status`, `.url` | name or short-id fallback; dimensions with `—`; 1024-based size; 48px preview only for `ready` + `url` |
| Pending image | `AssetRow.status == "pending"` | Cream `Procesando` badge, no thumbnail |
| Delete | `api::delete_asset(asset_id)` | Danger; server refusal when pinned/retained; then re-read |
| Retention | `ClinicSettings.server_retention_until` | `Retención en servidor hasta: {date}` read-only |
| Grace | `ClinicSettings.data_grace_ends_at` | `Fin del período de gracia para exportación: {date}` read-only |
| Export | static href `/api/clinic/export` | Secondary `Exportar datos (JSON)`; server-authorized GET; audited server-side |
| Offline | `Signal<SyncStatus>`; upload also probes `http::is_server_reachable()` | disable both uploads and delete; hint copy §3.2/§3.3 |
| Error | `message: Signal<String>` | `ErrorText` with the returned message or §3.5 fallback |

No field here is a hidden API input: the four data strings are exactly what `update_clinic_settings` sends. The timezone and retention dates are read-only. The image asset ids are internal; the row may show a shortened id only when it has no original name.

### 4.2 Hard rules

- Server-side authorization is the boundary. Page paint requires an Administrator, but the service layer re-checks every settings read/write and every asset read/delete. A non-Administrator is replaced, never shown a reduced hub.
- The image quota is v1 flat: **1 image per clinic**, **10,485,760 bytes**, JPEG/PNG only. The logo is `kind='logo'` and does not consume the slot. WebP/SVG are rejected server-side; do not accept them in the file input or convert them in the client. v1.1 tiers are absent.
- Upload is online-only. Never queue, retry in the background, or claim success offline. Deletes are also online-only and additionally blocked when a published version or retained prescription pins the asset (FR-085, D-05).
- Storage is private: no public object URL, no bucket key or raw object path in the DOM, only short-lived signed URLs (15 minutes). Keys carry no names, cédulas, or PHI (FR-083).
- The clinic logo does not consume the library slot and is not listed as a library image (FR-084). There is no signature or stamp asset anywhere (D-06).
- Retention and grace dates are server facts, read-only here. The screen never computes them from `retention_days`, never offers an edit, and never promises that export after the grace still succeeds. The export is offered before the grace ends.
- No billing, price, trial, plan, `sync_locked`, or subscription surface (D-04). No `/settings/billing` link, even disabled.
- No rich text, WYSIWYG, `contenteditable`, HTML payload, or markdown renderer (D-21). The LLM field is a plain URL template, not a chat surface.
- One Primary per viewport: `Guardar datos`. File fields are native controls; `Eliminar` is Danger; `Volver` and `Exportar datos (JSON)` are Secondary. No second filled teal action.
- No new endpoint, route, table, column, role, or permission. Reuse `GET /api/clinic`, `PUT /api/clinic`, `GET /api/assets`, `POST /api/clinic/logo`, `POST /api/assets`, `DELETE /api/assets/{id}`, and `GET /api/clinic/export`.
- A failed settings read and a failed asset read are separate failures: each has its own fallback; neither fabricates the other's data, and neither clears the other's typed values.
- The page never claims an image renders offline beyond the cached signed URL's lifetime; an expired URL becomes a placeholder, not an invented thumbnail.

### 4.3 Dynamic states

| State | Screen behaviour |
|---|---|
| Session check in flight | `PageShell` + `LoadingState` `Cargando...`; no cards or fetch |
| Checked, no session | Replace `/login`; no content |
| Checked, no profile | `Cargando...` until the redirect; no cards |
| Non-Administrator profile | Replace `/settings/profile`; `Redirigiendo...`; no cards |
| Administrator, settings + assets loading | Four cards painted; fields empty/quiet; image area no fabricated row; loading covers only unresolved values |
| Administrator, settings fetch error | `ErrorText` with returned message or fallback; clinic fields stay editable? No—use empty values and keep `Guardar datos` enabled only when the field source is known; prefer paint error and disable save until reload |
| Administrator, assets fetch error | Clinic data card usable; `ErrorText` in the library card; quota meter absent; no delete/upload success claim |
| Save idle | Primary `Guardar datos` enabled; four fields editable |
| Save busy | Primary `Guardando...`, disabled, `aria-busy`; fields keep values; no double PUT |
| Save success | Success hint `Datos de la clínica actualizados.`; values remain; `settings` snapshot refreshed from the next read |
| Save failure | `ErrorText` with the server message; typed values stay; no optimistic rename in the navbar or elsewhere |
| Logo empty | Empty hint; file field enabled when online |
| Logo present | 48px preview with alt; file field replaces it |
| Logo uploading | File field disabled/spinner; one file only; page-level error on failure |
| Logo offline | File field disabled; offline hint; preview may still paint from a still-valid signed URL |
| Library empty | `Sin imágenes` + description + file field (unless offline/full) |
| Library one ready image | Thumbnail, name, dimensions, size, Danger `Eliminar`; quota `1 de 1` |
| Library full | File field disabled; `La biblioteca está llena.`; delete is the only way to free the slot |
| Library pending | `Procesando` badge; no thumbnail; delete hidden/disabled until ready? State the shipped choice: pending rows are listed but not deletable if the service says so |
| Deleting | That row's `Eliminar` disabled with `Eliminando...`; other rows remain; success re-reads; refusal shows the pinned/retained sentence |
| Delete blocked by pin | Server `409`/error message in `ErrorText`; row stays; no optimistic removal |
| Desktop offline | Both upload fields disabled with connection hint; delete disabled; export disabled or explains; clinic date fields still readable; save requires server |
| Narrow viewport | One column; thumbnail above metadata; file field full width; no horizontal scroll; 44px targets |

## 5. Style

Every value is a `context/Design.md` token declared in `persona-sanus/assets/main.css` `:root`. No new color, font, icon package, CDN, or webfont.

### 5.1 Color

| Use | Value | Token |
|---|---|---|
| Canvas, cards, controls | `#ffffff` | `--color-paper` |
| Titles, field labels, values | `#061d29` | `--color-ink` |
| Subtitles, hints, metadata, Secondary border/text | `#425d6d` | `--color-mist` |
| Hairlines | `#e5e7eb` | `--color-hairline` |
| Primary fill, focus ring | `#006e75` | `--color-deep-teal` |
| Primary hover | `#0b978e` | `--color-bright-teal` |
| Pending badge surface | `#fffded` | `--color-cream` |
| Danger text and error surface | `#4d0037` / `#ffc3e6` | `--color-plum`, `--color-blossom` |
| Quota fill | `#006e75` | `--color-deep-teal` |
| Focus halo | `rgba(185,255,232,0.55)` | `--color-mint-glass` |

Never `#000000`. Never a chromatic card border; hairlines only. The quota fill is the only bar color intake, and it does not turn red at 100% — the text and disabled upload carry the state. Measured contrast: Ink/Paper 17.28 · Mist/Paper 6.96 · Paper/Deep Teal 6.02 · Ink/Cream 16.89 · Mist/Cream 6.80 · Plum/Blossom 10.36. AA for body.

### 5.2 Typography

| Element | Family | Weight | Size / line-height |
|---|---|---|---|
| Page title (`h1`) | `--font-mint` | 500 | 22px / 1.22 Ink |
| Page subtitle | `--font-mint` | 400 | 14px / 1.5 Mist |
| Section heading (`h2`) | `--font-mint` | 500 | 20px / 1.25 Ink |
| Field label / hint | `--font-mint` | 500 / 400 | 14px / 1.5 Ink / Mist |
| Input, textarea, select value | `--font-mint` | 400 | 16px / 1.5 Ink |
| Primary / Secondary / Danger label | `--font-mint` | 500 | 16px / 1.22 |
| Quota meter text | `--font-mint` | 400 | 14px / 1.5 Mist |
| Image row name | `--font-mint` | 500 | 16px / 1.5 Ink |
| Image row meta | `--font-mint` | 400 | 14px / 1.5 Mist |
| Pending badge | `--font-mint` | 500 | 13px / 1.22 |
| Empty title / description | `--font-mint` | 500 / 400 | 18px / 1.25 Ink; 14px / 1.5 Mist |
| Error text | `--font-mint` | 400 | 14px / 1.5 Plum |

No `--font-grenette`; app chrome only. Long LLM URLs and file names use `overflow-wrap:anywhere` at 14–16px, never shrink below 14px.

### 5.3 Geometry

- 4px base. `PageShell`: 48px top, 80px bottom, 24px inline; `.ui-container` max 1200px; `.ui-stack` 24px.
- PageHeader: stack, 4px title/subtitle gap; back action at the inline end; below 640px it moves under the subtitle.
- Section cards: Paper, 1px hairline, 24px radius, 24px padding, 16px vertical gap; no shadow.
- FormGrid: 2 columns, 16px gap; one column below 760px. LLM field spans both (`wide`). Inputs 8px radius, `8px 12px` padding, 1px hairline, min-height 44px. Labels 4–8px above controls.
- Logo preview: 48px square, 8px radius, hairline; object-fit contain; empty hint text 14px Mist. FileField full width; native file control styled per plan, 8px radius.
- Library row: flex, 12px gap, thumbnail 48px sq, text block `min-width:0`, Danger at the end; pending badge inline; below 640px row stacks. Quota meter: 1px hairline track, 4px height? Use plan `QuotaMeter` height; fill Deep Teal, clamp 0..1.
- Export: two hint lines then paragraph, max 72ch, Secondary link after 12px gap.
- Elevation flat; no card hover lift; no page scrollbar besides content. 320px no horizontal scroll.

### 5.4 Iconography and imagery

Line icons only, 1.5px stroke, 20px default, Ink or Deep Teal: `Image` placeholder for a missing thumbnail, `Alert` in errors. The logo preview is the customer's uploaded image, not an icon; it carries `alt="Logo de la clínica"`. No stock photos, illustrations, signature/stamp images, QR graphics, patient images, or decorative marks. The library thumbnail is the uploaded asset's signed URL, never a generated mock.

## 6. Accessibility

- Landmarks: navbar banner (`Header.md` §6) and one content region. Each `SectionCard` is a labelled `section` with its `h2`; the page header owns the only `h1`, `Clínica`. No nested `main`, no duplicate banner.
- Heading order: `h1` → four `h2` in DOM order: `Datos de la clínica`, `Logo`, `Biblioteca de imágenes`, `Exportación de datos`. No `h3` unless the image tray needs a group label; none is added.
- Tab order: `Volver` → Nombre → Dirección → Teléfono → Plantilla del enlace LLM → Guardar datos → logo file input → library delete button (if present) → library file input → Exportar datos (JSON). Disabled controls are skipped. No hidden input receives focus.
- Focus: `1px solid var(--color-deep-teal)`, offset 2px, halo `0 0 0 3px rgba(185,255,232,0.55)` on each field, file input, button, link, and delete. Visible on Paper and Cream; never removed for the native file control.
- Labels: each field is wrapped by its visible label. The file label includes the format and size rule, so the accessible name is not “Choose file” alone. Buttons are named by visible text. The logo `img` has the exact alt; a missing logo uses text, not an empty image.
- Errors: one page-level `ErrorText` with `role="alert"` and Alert icon; field errors use the shared field contract with `aria-invalid="true"` and `aria-describedby`. The success message is `role="status"`, not an alert. A failure does not move focus; it stays next to the card/action that caused it.
- Busy: `Guardar datos` sets `aria-busy="true"` and `disabled`; its label becomes `Guardando...`; one PUT. Uploads disable the file input and expose the page-level status; no duplicate file read. Deleting sets the row's button disabled and label `Eliminando...`.
- Quota: `QuotaMeter` is `role="progressbar"`, `aria-valuemin=0`, `aria-valuemax={limit}`, `aria-valuenow={used}`, and its text is the accessible name. At full, the meter does not alarm; the disabled file field and `La biblioteca está llena.` text carry the state.
- Status: the pending badge is text, not colour alone; the error banner carries text and an icon; no state relies on hover.
- Touch: inputs and buttons at least 44px; file control target at least 44px; image rows wrap rather than clip at 320px.
- Reduced motion: the shared spinner stops under `prefers-reduced-motion: reduce`; no other animation is introduced. Zoom/reflow: 200% and 320px keep one column and no horizontal scroll.

## 7. Implementation notes

The shipped component is `fn SettingsClinic()` at `persona-sanus/src/main.rs:1804`, `#[component]` at `:1803`, closing before `SettingsUsers` at `:2120`. It reads `actor` and `checked` contexts; its effect is keyed on `generation` and performs the two redirects described in §2.1, then fetches `api::fetch_clinic_settings()` and `api::fetch_assets()` and sets the local snapshots. It has no `loaded` signal and no `online` read.

Current markup, before the plan:

- Root `div#settings-page`, `.page-head` with `h4` `Clínica`, the shipped subtitle, and a Secondary `Link` `Volver` to `Settings`.
- One page-level `p.error` for `message`.
- `section.editor-section` `Datos de la clínica`, `div.form-grid` with four raw `label.field` inputs (name, address, phone, LLM wide), `div.actions` with a raw `button` `Guardar datos`/`Guardando....`.
- `section.editor-section` `Logo`: optional `img.clinic-logo` with the signed URL and alt, else `p.hint`; raw file input with `accept="image/jpeg,image/png"`, disabled while `uploading`; the handler probes `is_server_reachable`, reads bytes, calls `api::upload_logo`, sets `Logo actualizado.` or the error, increments `generation`, clears `uploading`.
- `section.editor-section` `Biblioteca de imágenes`: `p.hint` `Uso de la biblioteca: {used} de {limit}.` when the snapshot exists; a `.asset-row` loop with `strong` name (`original_name` else raw id), a hint with `width × height · size bytes · sha256` prefix; a Danger `button` `Eliminar` calling `api::delete_asset`, then `Imagen eliminada.` + generation; a raw file input `Subir imagen de la biblioteca...` that checks reachability, reads bytes, `api::upload_asset`, sets `Imagen agregada a la biblioteca.` + generation, clears `uploading`.
- `section.editor-section` `Exportación de datos`: optional retention/grace `.hint` lines, explanation `.hint`, `a.button.secondary` to `/api/clinic/export`.

Plan step 16 deltas and gaps:

- Root becomes `PageShell id="settings-page"`. The page head becomes `PageHeader` `Clínica` with the shipped subtitle and Secondary `ButtonLink` `Volver` to `/settings`. Four `SectionCard`s carry the current `h2` labels: `Datos de la clínica`, `Logo`, `Biblioteca de imágenes`, `Exportación de datos`.
- Data card uses `FormGrid columns=Two` with `TextField` `Nombre`, `Dirección`, `Teléfono`, and `TextField wide` `Plantilla del enlace LLM`; `ActionRow` holds the one Primary `Guardar datos` with `loading: saving()`. Add a read-only hint for `Zona horaria: {timezone}`. Do not add a days input: `retention_days` is not in `ClinicSettings` and the shipped update does not accept it. Record this sitemap/API gap in §7 rather than inventing a field.
- Logo card uses a 48px `img.ui-asset-preview` when `clinic.logo.url` is present (halt on error), the empty hint otherwise, and `FileField` with the exact label/accept; keep the upload body in a named closure `upload_logo`. Disable it when `!online || uploading()` and show the offline hint. The handler keeps the reachability probe as a defense, because the context may be stale.
- Library card uses `ImageLibrary` with `assets`, `used`, `limit`, `uploading`, `deleting`, `online`, `on_delete`, `on_upload`; keep delete/upload bodies in named closures `delete_image` / `upload_image`; they call the same `api::delete_asset` / `api::upload_asset` and increment `generation` on success. The logo does not appear in `assets`. `QuotaMeter` uses `used`/`limit`.
- Export card becomes a `SectionCard` with two `HintText` date lines, one explanation, and a Secondary `LinkAction` to `/api/clinic/export`. Disable it offline; do not synthesize a client file.
- Keep `message` page-level but move success copy to `HintText`/`Notice Success`; only failures stay in `ErrorText`. Today a success and a failure share `.error`, so the target must split them. This is copy/status wiring, not a new state.
- The shipped image row prints the raw asset id and a 12-character sha256 prefix. Target uses `original_name` or `Imagen {first 8 id chars}` and never shows the hash; the hash is metadata, not user copy. The `0 × 0` fallback becomes `—`.
- The shipped file handler sets `uploading` false on all paths; keep that. Do not add a per-row pending flag beyond the existing pending status from `ImageLibrary`.
- Keep the `generation` counter as the refresh trigger. A successful save, logo upload, library upload, or delete increments it; the effect re-reads. Do not optimistically mutate the library vector.
- The sync context is `Signal<SyncStatus>`; pass `online` into both upload fields and the library. The data form may stay editable offline, but `Guardar datos` needs the server; on failure, keep values and show the transport error. There is no outbox for settings or images.

Verification commands, from the repo root; none is recorded here as executed:

```sh
cd persona-sanus && cargo check
dx build --release --fullstack
caveman-blocks run persona-smoke --dir persona-sanus --port 8091
python3 .blocks/markdown-table-check.py --path context/screens/SettingsClinic.md --strict
```

A browser pass as Administrator and as Doctor is needed for the redirect; an offline desktop pass for both disabled upload fields and delete; upload a JPEG, a PNG, a WebP, and a file over 10 MiB to prove the server refusals render in `ErrorText`.

- Do not add: a billing/plan/price/trial/`sync_locked` card or link (D-04); a clinic picker or tenant switch; staff, role, audit, patient, prescription, or template content.
- Do not add: a signature or stamp upload, preview, placeholder, or copy (D-06); a patient photo field.
- Do not add: rich text, WYSIWYG, `contenteditable`, HTML, or markdown editing (D-21); the LLM field is a plain URL template.
- Do not add: a second Primary, a filled Danger action, a modal, a toast, or a confirmation dialog.
- Do not add: a second image slot, a multi-image gallery, crop/resize/reorder, WebP/SVG acceptance, or a client-side quota override (v1 is 1 × 10 MiB; v1.1 tiers are absent).
- Do not add: upload/delete queueing offline, a background retry, or a “sync later” claim.
- Do not add: an editable timezone, retention-days input, or computed retention date.
- Do not add: a public object URL, raw bucket key, or image URL in copy/analytics/logs.
- Do not add: a new route, endpoint, table, column, role, permission, or API argument.

## 8. Spec references

| Spec | Connection |
|---|---|
| `specs/done/15-O-ui-tokens-actions-and-formatting.md` | `15-O` supplies the tokens, `Button`, `ButtonLink`, `LinkAction`, and format helpers the clinic page renders. |
| `specs/done/16-P-ui-forms-feedback-and-layout.md` | `16-P` supplies `PageShell`, `PageHeader`, `SectionCard`, `FormGrid`, `FileField`, `ActionRow`, `HintText`, and `ErrorText` the four sections use. |
| `specs/done/18-R-ui-domain-and-billing.md` | `18-R` builds `ImageLibrary`, `QuotaMeter`, and `format_bytes` (1024-based), the image library and quota surface. |
| `specs/done/24-X-ui-settings-detail-and-audit.md` | `24-X` adopts `SettingsClinic`: sync-gated uploads, extracted upload/delete closures, one Primary `Guardar datos`. |
| `specs/done/25-Y-ui-legacy-css-cleanup-and-docs.md` | `25-Y` removes legacy page-ID selectors so the adopted page runs only on `.ui-*` classes. |
