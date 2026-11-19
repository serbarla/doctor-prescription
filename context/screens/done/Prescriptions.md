# PersonaSanus — Prescriptions (`/prescriptions`)

| Field | Value |
|---|---|
| Source | `context/sitemap.md` §1.2 (`/prescriptions` — Prescriptions — history: search/filter by patient, date, status; reprint regenerates the same content; desktop cache ≤ 30 days), §1.3 (PDF download/print; sync and outbox status), §2 (Doctor own, Nursing, Secretary reprint, Administrator read), §3 flows 1, 2, 5, §4 M1/M2, §5 conventions; `context/PRD.md` FR-013, FR-030–FR-034, FR-040–FR-043, D-01, D-04, D-06, D-07, D-21; `context/Design.md` (Paper, Ink, Mist, Hairline, Deep Teal, Bright Teal, Cream, Blossom, Plum, Mint Glass; 24px cards, 8px controls, flat elevation); `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` step 15, Page adoption row `Prescriptions` (line 228), primitives `PageShell`, `PageHeader`, `FilterBar`, `SearchField`, `SelectField`, `TextField`, `ListRow`, `Badge`, `SectionCard`, `EmptyState`, `LoadingState`, `ErrorText`, `HintText`, `LinkAction`, `PrescriptionStatusBadge`; `context/screens/INDEX.md` §2.3 |
| Component | `Prescriptions` — `persona-sanus/src/main.rs:4360` (body `4359-4592`); composes planned `PageShell`, `PageHeader`, `FilterBar`, `SearchField`, `SelectField`, `TextField`, `ListRow`, `Badge`, `SectionCard`, `EmptyState`, `LoadingState`, `ErrorText`, `HintText`, `LinkAction`, and `PrescriptionStatusBadge` from `persona-sanus/src/ui/` (the directory is planned, not shipped); reads `Signal<Option<SessionUser>>`, `Signal<Option<Me>>`, `Signal<bool>`, `Signal<String>`, `Signal<Vec<api::Prescription>>`, and `Signal<Vec<(String, String)>>` |
| Route | `/prescriptions` (`Route::Prescriptions { status: Option<String> }` at `persona-sanus/src/main.rs:84-85`) — authenticated shell; signed-out visitors are replaced with `/login` (`Route::AuthPage {}`); reading requires `Me::can_read_prescriptions()` (any clinic role); optional `?status=` accepts `draft`, `signed`, `superseded`, `voided`, `pending_sync`, `failed`; patient, date, and search values are signals, never query parameters; no role redirect |
| Layout | `PageShell id="prescriptions-page"` inside `#[layout(Navbar)]` (opens at `main.rs:51`, never closed); root `.ui-page` > `.ui-container.ui-stack`; Paper canvas and Paper rows; Cream empty state; the sync-error card is a Paper `SectionCard`; no full-bleed exception |
| Locale | `es-EC` on every string; `receta` spelling; dates `DD-MM-AAAA`; no time of day is rendered |
| Status | ✅ Spec complete (sections 1–7). Route status: 🟡 — sitemap cell: implemented search/filter history by patient, date, and status with the `draft → signed → superseded/voided` lifecycle; PDF/print regenerate the same content; desktop keeps only ≤ 30 days locally and older rows regenerate on demand. Shipped: client-side patient search, status/date request, desktop pending/failed reads, outbox error rows, `Abrir`. Target: shell adoption, a `loaded` signal, status badges, row PDF/print actions, and the expired-cache offline guard |

The prescription history. It is where a clinician scans issued and working documents, narrows the list, and reopens or reprints a document. It is read-only in every role. This file freezes region order, copy, geometry, and states for the shipped component; it introduces no route, endpoint, table, column, role, or permission.

## 1. Purpose

The screen performs four jobs, in order:

1. **Find.** Search the visible history by patient name and narrow it by lifecycle state, creation date, or both.
2. **Read the state.** Show at row level whether a prescription is a draft, signed, superseded, or voided, and, on desktop, whether it is provisional or sits behind a failed sync operation.
3. **Open.** Navigate to one prescription's forms and detail through the opaque `id`.
4. **Regenerate a copy.** For rows that already have a server-backed final document, offer `Descargar PDF` and `Imprimir`, both rendered on demand from the pinned version by the one `printpdf` pipeline (FR-030–FR-033).

It never renders:

- PHI outside the authorized rows: the rows are clinical content for the roles admitted by `context/sitemap.md` §2, and this file uses no real patient name, cédula, diagnosis, medication, or photo. The route is `/prescriptions`; no patient value enters the URL, and the only URL value is the opaque prescription `id` on `Abrir` and on the PDF endpoint.
- Billing, price, trial, plan, entitlement, invoice, or `sync_locked` copy or state (D-04). v1 is a free beta and this page has no plan surface.
- A signature or stamp image, asset, slot, or preview (D-06). The printed document's blank signature rule belongs to the preview renderer, not to this list.
- A rich-text editor, formatting toolbar, `contenteditable` surface, or formatted clinical value (D-21). The filters are a plain search input, a plain select, and a plain date input.
- Prescription creation, editing, signing, voiding, superseding, template selection, medication editing, diagnosis or instruction fields, the LLM link, an A4 preview, a QR code, share, an audit panel, patient photos, or a document thumbnail. Those belong to `/prescriptions/new`, `/prescriptions/:id`, `/prescriptions/:id/preview`, the templates screens, and the audit log.
- A stored PDF or silent PDF generation. Every PDF byte comes from an explicit `Descargar PDF` or `Imprimir` action, and this screen keeps no file.

One filled teal action exists at most: the header `Nueva receta` for Doctor and Nursing. Every other action, including both PDF actions, is Secondary. Secretary and Administrator see no primary.

## 2. Structure

One Paper page, a stack of 24px gaps, descending from the navbar. Header and filters stay mounted while the record slot changes. The sync-error card follows the record slot on the desktop failed view.

| Order | Region | id / class | Band tone | Contents |
|---|---|---|---|---|
| — | Navbar (layout) | `#navbar` · `header.ui-navbar` | Paper | Owned by `context/screens/done/Header.md` §2–§5; identity, sync indicator, links only |
| 1 | Page shell | `#prescriptions-page` · `div.ui-page` > `.ui-container.ui-stack` | Paper | 48px top padding, 80px bottom, 24px inline; stack gap 24px |
| 2 | Page header | `.ui-page-head` · `h1.ui-page-title` · `p.ui-page-subtitle` · `.ui-page-head-actions` | Paper | `Recetas`, the history subtitle, and at most one Primary at the inline end |
| 3 | Filters | `.ui-filter-bar` | Paper | `SearchField` `Paciente`; `SelectField` `Estado`; `TextField` `Fecha` with date input |
| 4 | Feedback | `.ui-error-text` | Paper | `ErrorText` carrying `message`, only when non-empty, above every record |
| 5 | Prescription records | `.ui-list` with one `ListRow` per record | Paper | Title `patient_name`; meta status, date, `No.`; `PrescriptionStatusBadge`; provisional `Badge`; trailing `Abrir`, `Descargar PDF`, `Imprimir`; the expiry `HintText` |
| 6 | Empty or loading | `.ui-empty` · `.ui-loading` | Cream · Paper | `EmptyState` title and conditional description; `LoadingState` spinner and label |
| 7 | Sync errors (desktop failed view) | `SectionCard` · `h2.ui-card-title` | Paper | `Errores de sincronización`; one actionless `ListRow` per failed outbox entry, title `operation`, meta `[detail]` |

Orders 5–6 are mutually exclusive inside the record slot. Order 7 is independent and follows it, so failed outbox entries remain visible with or without prescription rows. Order 4 precedes both. Nothing is horizontally scrollable. No table renders, so `DataTable` contributes no caption or `th scope` here; `ListRow` is the list primitive.

### 2.1 Session and role

`checked` is the shell session-check signal; `actor` is `Signal<Option<Me>>`. `Me::can_read_prescriptions()` is true for every non-null role (`persona-sanus/src/api.rs:80-82`); `Me::can_create_prescriptions()` is true only for `doctor` and `nursing` (`api.rs:72-74`).

| Session / role | Reads history | Header Primary | What changes; what does not |
|---|:---:|:---:|---|
| Check in flight (`!checked`) | no | no | `PageShell` with one `LoadingState` `Cargando...`; no header, filters, or records. Navbar and root id are already attached |
| Checked, no session | no | no | The effect replaces the route with `Route::AuthPage {}`; no page content renders |
| Checked, `actor == None` | no | no | `PageShell` with one `LoadingState` `Redirigiendo...`; no navigation and no fetch |
| Role `null` / no clinic role | no | no | `PageHeader` denial title and subtitle only; no filters, rows, PDF actions, or primary |
| `doctor` | yes | yes | Own history per sitemap §2; the header Primary `Nueva receta` is the only filled action. A row's PDF actions are Secondary reprints, never edits |
| `nursing` | yes | yes | All authorized clinic history; the same header Primary creates a draft. No sign, void, or supersede control appears here |
| `secretary` | yes | no | Reprint view: rows and PDF/print actions, no header Primary; clinical content is read-only |
| `administrator` | yes | no | Read-only history: rows and PDF/print actions, no Primary, no clinical write |

The role table changes only the primary and the server's authorized result set. It never changes region order, row copy, date format, or the PDF URL shape. The client gate selects controls; the server checks the role, derives the clinic, and decides the scope. This screen sends no actor, owner, or clinic argument.

### 2.2 Status, source, and surface

The six filter values after `Todos los estados` are the sitemap lifecycle plus the two desktop outbox views. They are a view selection, not a new stored status: no row is edited into `pending_sync` or `failed` from this page.

| Selected `Estado` | Request path | What renders | PDF actions |
|---|---|---|---|
| All (`""`) | Web and online desktop: `api::fetch_prescriptions("", "", date)` | Every authorized prescription matching the chosen date | Only rows whose own status is `signed`, `superseded`, or `voided` |
| `draft`, `signed`, `superseded`, `voided` | The same call with the literal filter value | Server-filtered rows plus the client patient search | Only for `signed`, `superseded`, or `voided`; never for a draft |
| `pending_sync` | Desktop first reads `desktop_local::list_prescriptions(..., "pending_sync")` and returns on success; web and a desktop fallthrough use the API call | Desktop cached provisional rows; each carries the Cream provisional badge regardless of its stored status | None; an unsynced row has no server copy to regenerate |
| `failed` | Desktop first reads `desktop_local::list_prescriptions(..., "failed")`, then `outbox_rows()` filtered to failed | Desktop local failed prescription rows plus the `Errores de sincronización` card; web returns no matching rows | None |

The desktop pending and failed reads take only the status string: the `Fecha` signal is ignored in those two local views, and the patient search still filters the returned rows client-side. On web, the literal `pending_sync` and `failed` values reach the existing list endpoint; which stores no such status, so those views are empty and show no sync-error card.

### 2.3 Web and desktop surfaces

The same component, ids, copy, and role gates serve both builds; the branch is the existing `#[cfg(feature = "desktop")]` code and the `desktop_local` cache.

- **Web:** every filter change calls `api::fetch_prescriptions`; the server response is the only row source. PDF and print are ordinary links to `GET /api/prescriptions/:id/pdf`, and the browser streams the response. No cache fallback runs; this screen writes nothing locally.
- **Desktop online:** the local pending and failed reads win for those two filters because neither status exists on the server. The other filters call the same API. PDF and print regenerate on demand from the pinned final version; no file is stored.
- **Desktop fetch failure:** `message` keeps the API text, then `desktop_local::open_local_db()` and `desktop_local::list_prescriptions(&mut conn, "")` replace the row signal with every still-cached prescription, ignoring the selected status and date. `serde_json::from_value::<api::Prescription>` failures are skipped. If the database or the listing fails, the current rows remain untouched. A successful cache read never clears the error.
- **Offline desktop:** filters and rows remain usable; opening a row is a route change. The cache is the only source, so rows outside the local window are absent rather than greyed. A row older than the window that is still in memory has its PDF actions disabled with the connection hint. This screen performs no local insert, update, delete, or outbox write.
- **Retention:** only server-backed prescriptions created within `clinic.retention_days` (default 30) stay local (FR-042). Rows with no server copy — created offline, still in the outbox, or never synced — are never evicted and are never treated as expired. The cache relies on OS disk encryption; no app-level database encryption is added here (D-07).
- **Renderer:** download and print regenerate the same content from the pinned final version through the one `printpdf` renderer. Web streams it; desktop calls the same renderer in-process. The content is the same and not promised byte-identical across renderer versions. No PDF is stored (D-01, FR-030, FR-031).

### 2.4 Empty, denied, and error surfaces

The initial session check and the unresolved-actor redirect replace the whole page with a `LoadingState`; no header or filters flash first. Once `checked` is true and the actor has a role, the record slot owns loading, empty, and error presentation while header and filters stay interactive.

`loaded` starts false. The record slot shows `LoadingState` `Cargando recetas...` until the first request for the current view settles, whether it returns rows, no rows, or an error. A settled zero-row view shows `EmptyState` `Sin recetas`; its description appears only when the trimmed patient search, the status value, or the date value is non-empty, so an empty clinic is not described as a failed filter. A fetch error shows `ErrorText`; when the desktop fallback found cached rows they render under the error, and when it found none the error stands without a false `Sin recetas` claim.

The denial region replaces everything below the page shell with a denial `PageHeader`: title `Acceso denegado`, the single subtitle, and no filters, rows, PDF actions, primary, or back link. A no-role session is denied; a fetch failure is not a denial and keeps the filters visible. The failed outbox card is its own section, not a replacement for the prescription list.

### 2.5 Row composition and long content

Each `ListRow` carries the authorized `patient_name` as its full title, never truncated with an ellipsis; up to three meta strings in fixed order: the localized status, `format_date(created_at)`, and `No. {rx_number}` only when a number exists; the `PrescriptionStatusBadge` for the stored status; the Cream provisional badge on the pending view; and trailing actions in the order `Abrir`, `Descargar PDF`, `Imprimir`.

A long patient name wraps within the title column while the action group keeps its intrinsic width and stays top-aligned, so no action label is squeezed or hidden. Below 640px the row becomes a column and the actions wrap to the inline start. The title is never truncated, the whole row is never a link, and `Abrir` remains a visible, separately focusable link. Draft rows show `Abrir` alone. Pending and failed-view rows show `Abrir` alone. A signed-family row that is expired and offline shows `Abrir`, the two disabled PDF actions, and one hint line.

## 3. Copy text

All strings are verbatim `es-EC`; punctuation belongs to the copy. Chrome strings are owned by `context/screens/done/Header.md` §3 and `context/screens/done/Footer.md` §3 and are not repeated here.

### 3.1 Page header and primary action

| Element | Copy | Target |
|---|---|---|
| Page title (`h1`) | `Recetas` | — |
| Page subtitle | `Historial con filtros por paciente, fecha y estado.` | — |
| Header action, Primary, doctor and nursing only | `Nueva receta` | `/prescriptions/new` (`Route::PrescriptionNew { patient_id: None, template_id: None }`) |

Secretary and Administrator render no header action. There is no back link: `/prescriptions` is a top-level navbar destination (`Header.md` §3.2).

### 3.2 Filters

| Element | Copy | Kind |
|---|---|---|
| Patient search label | `Paciente` | `SearchField.label`, visually hidden |
| Patient search placeholder | `Nombre del paciente` | supplementary, never the accessible name |
| State select label | `Estado` | visible `SelectField.label` |
| Date field label | `Fecha` | visible `TextField.label` |
| Date input type | — | `date`; the browser shows the localized value |
| All-state option | `Todos los estados` | `value=""` |
| Draft option | `Borradores` | `value="draft"` |
| Signed option | `Firmadas` | `value="signed"` |
| Superseded option | `Reemplazadas` | `value="superseded"` |
| Voided option | `Anuladas` | `value="voided"` |
| Pending-sync option, desktop | `Pendientes de sincronización` | `value="pending_sync"` |
| Failed-sync option, desktop | `Con error de sincronización` | `value="failed"` |

No submit button, result counter, sort control, page-size control, or saved filter renders. The patient search narrows the visible set on input; the status and date controls request a new set on change.

### 3.3 Prescription rows

| Element | Copy | Target |
|---|---|---|
| Title | authorized `patient_name` | — |
| Metadata order | `{status} · {DD-MM-AAAA} · No. {rx_number}` | number fragment only when present |
| Draft status | `Borrador` | — |
| Signed status | `Firmada` | — |
| Superseded status | `Reemplazada` | — |
| Voided status | `Anulada` | — |
| Provisional badge | `Provisional · PENDIENTE DE SINCRONIZACIÓN` | — |
| Unknown status | `Desconocido` | — |
| Open action, Secondary | `Abrir` | `/prescriptions/{id}` (`Route::PrescriptionDetail { id }`) |
| Download action, Secondary | `Descargar PDF` | `/api/prescriptions/{id}/pdf` |
| Print action, Secondary | `Imprimir` | same PDF URL, new browsing context, `noopener noreferrer` |
| Evicted/offline hint | `Requiere conexión para regenerar esta receta.` | `HintText` under the actions |

`Descargar PDF` and `Imprimir` render only on a server-backed final row: `signed`, `superseded`, or `voided`, never on a draft, a provisional `pending_sync` row, a failed row, or an unknown status. The final filename is the server's `Content-Disposition` value (`{PatientName}-receta-{DD-MM-AAAA}-{DoctorName}.pdf`, FR-032); this page never builds it or shows it as a URL.

### 3.4 Empty, loading, error, denial

| Element | Copy |
|---|---|
| Empty title, any zero-row state | `Sin recetas` |
| Empty description, unfiltered | `Todavía no hay recetas en esta clínica.` |
| Empty description, any filter active | `No hay recetas para estos filtros.` |
| Loading label | `Cargando recetas...` |
| Fetch error fallback | `No se pudieron cargar las recetas.` |
| Denied title (`h1`) | `Acceso denegado` |
| Denied body | `Necesitas un rol en la clínica para consultar recetas.` |
| Session gate | `Cargando...` |
| Redirect gate | `Redirigiendo...` |

### 3.5 Sync errors card (desktop failed view)

| Element | Copy |
|---|---|
| Section heading (`h2`) | `Errores de sincronización` |
| Description | `Operaciones locales que no se pudieron subir.` |
| Row title | local `operation` string, verbatim |
| Row meta | local `lastError` string, verbatim |
| Empty | the card does not render when there are no failed rows |

The card has no retry, discard, or "sync now" action.

## 4. Displayed information

### 4.1 What the screen shows

| Element | Source | Value shown |
|---|---|---|
| Page title and subtitle | static | `Recetas`, `Historial con filtros por paciente, fecha y estado.` |
| Header Primary | `actor: Signal<Option<Me>>`, `Me::can_create_prescriptions()` (`api.rs:72-78`) | `Nueva receta` for doctor and nursing; absent for secretary and administrator |
| Patient term | `query: Signal<String>` from the search input | typed text, never in the URL; client-side case-insensitive substring on `patient_name` after the request returns |
| State filter | `status: Signal<String>`, seeded once from route `?status` when present | one of the §3.2 values; sent to the API for stored-status filters; selects the local views for pending/failed on desktop |
| Date filter | `date: Signal<String>` from the date input | `YYYY-MM-DD` in the control; sent as the API `date` parameter; no time zone conversion, no invented range |
| Rows (web/online desktop) | `api::fetch_prescriptions("", selected_status, selected_date)` (`api.rs:204-228`) to `api::Prescription` | `patient_name`, `status`, `created_at`, `rx_number`, and opaque `id`; server scope and order preserved |
| Rows (desktop pending) | `desktop_local::list_prescriptions(&mut conn, "pending_sync")` | cached provisional rows; stored status may still be `signed`; the view badge says provisional |
| Rows (desktop failed) | `desktop_local::list_prescriptions(&mut conn, "failed")` plus `outbox_rows()` filtered to `status == "failed"` | local failed prescriptions plus `(operation, lastError)` pairs |
| Desktop fetch fallback | `desktop_local::list_prescriptions(&mut conn, "")` | all cached prescriptions, regardless of the selected status/date; decode failures skipped |
| Row status badge | `PrescriptionStatusBadge(row.status)` | localized lifecycle label; never a raw enum |
| Provisional badge | static, only when the selected view is `pending_sync` | `Provisional · PENDIENTE DE SINCRONIZACIÓN`; Cream badge |
| Dates | `format_date(row.created_at)` (`main.rs:4321`) | `DD-MM-AAAA`; malformed values paint as stored |
| PDF actions | `row.id`, `row.status` | rendered only for `signed`/`superseded`/`voided`; the endpoint regenerates from the pinned version |
| Expiry hint | a stored final status, desktop offline, and no cached PDF can be confirmed | `Requiere conexión para regenerar esta receta.`; PDF actions disabled, `Abrir` stays enabled |
| Error line | `message: Signal<String>` | the returned message or the §3.4 fallback; stays above any cached rows |
| Denial | `actor` role missing | denial title and body instead of header actions, filters, and rows |

The list endpoint caps at 200 rows (`service::list_prescriptions`). This page adds no pagination and no total; a 200-row result is painted as returned.

### 4.2 Hard rules

- Read-only in every role. The page never creates, edits, signs, voids, supersedes, deletes, uploads, or publishes. `Nueva receta` and `Abrir` are navigation; download and print are GET links.
- The server is the authorization and scope boundary. The client role gate only decides which controls paint; it never widens the result set, and it sends no clinic, doctor, owner, or role parameter.
- **Shipped divergence:** `service::list_prescriptions` passes `doctor_id = None` to the query (`service.rs:3316`), so a Doctor receives the whole clinic's history, not only their own as `context/sitemap.md` §1.2/§2 states. This page must not hide that with a client-side filter: a client filter is not a boundary, and it would still ship the other doctors' rows to the browser. The fix belongs in the service layer, passing the actor doctor id for `Role::Doctor` while keeping clinic-wide for Administrator and the nursing/secretary scopes as the sitemap declares. The §2.1 table records the intended matrix; this bullet records the gap.
- **Shipped divergence:** the detail read (`GET /api/prescriptions/:id`) is not owner-scoped either; it accepts any role in the clinic. The resolving change belongs with `PrescriptionDetail.md` §4.2/§7, not in this list. This list only links to it.
- PHI stays out of the URL. The only route value is the optional non-PHI status token; patient term, date, and selected state are signals. `Abrir` and the PDF href carry the opaque prescription `id`, nothing else. No patient name, cédula, diagnosis, or medication enters history, analytics, a title, a log, or a thumbnail.
- The row shows only identity and lifecycle: patient name, status, creation date, prescription number. No diagnosis, medication, instructions, verification code, QR, cédula, or patient photo renders in the list.
- Signed documents are immutable everywhere. This page has no edit, sign, void, or supersede control, including for the owner doctor; those live on the detail route.
- No billing, price, trial, plan, entitlement, invoice, or `sync_locked` copy or state (D-04). No image quota, no upgrade prompt.
- No signature or stamp image, slot, or placeholder (D-06). No rich text, formatting toolbar, `contenteditable`, or formatted clinical value (D-21).
- The PDF is regenerated on demand from the pinned version; no PDF is stored on the server or in this screen, and "same content" does not promise byte-identical bytes across renderer versions (D-01, FR-030).
- The desktop cache covers only server-backed rows within `retention_days` (default 30); unsynced rows are never evicted and never treated as expired. The cache is not app-encrypted (D-07).
- A desktop fallback never hides loss: the API error stays visible above cached rows. A cache read is not a successful fetch and is not announced as fresh data.
- No sorting, pagination, result count, bulk action, select-all, saved filter, infinite scroll, or silent background refresh is added.
- One `h1` and at most one filled Primary (`Nueva receta`). Row actions are Secondary; no chromatic border; flat elevation.

### 4.3 Dynamic states

| State | Screen behaviour |
|---|---|
| Session check in flight | `PageShell` + `LoadingState` `Cargando...`; no header, filters, or fetch |
| Checked, no session | Replace with `/login`; no list paint |
| Checked, `actor == None` | `LoadingState` `Redirigiendo...`; no redirect retry, no fetch |
| No role | Denial `h1` and body; no filters, rows, PDF actions, or Primary |
| Doctor / nursing | Full header with Primary `Nueva receta`, filters, rows. Doctor scope is the §4.2 shipped divergence |
| Secretary / administrator | Full header, no Primary; filters, rows, PDF actions remain |
| First fetch, `loaded == false` | `LoadingState` `Cargando recetas...` in the record slot; header and filters stay interactive |
| Fetch resolves with rows | One `ListRow` per row in server order; badges and secondary actions |
| Fetch resolves empty, no filters | `EmptyState` `Sin recetas` + `Todavía no hay recetas en esta clínica.` |
| Fetch resolves empty, any filter | `EmptyState` `Sin recetas` + `No hay recetas para estos filtros.`; typed term stays |
| Patient term typed | No request; the visible set narrows client-side, `loaded` stays true, caret stays in the field |
| Status/date changed | New request; `loaded` stays true so the slot does not flash a first-load spinner; prior rows stay until the result replaces them |
| Fetch error, web | `ErrorText` with the returned message or the §3.4 fallback; filters stay; no rows below it |
| Fetch error, desktop with cache | `ErrorText` stays above the cached rows; the cache view ignores status and date; undecodable rows are skipped |
| Fetch error, desktop without cache | `ErrorText` only; no false empty state |
| Selected `pending_sync`, desktop | Local provisional rows; Cream provisional badge per row; no PDF/print actions; `Abrir` enabled |
| Selected `pending_sync`, web | Empty view; no provisional badge is fabricated and no sync-error card renders |
| Selected `failed`, desktop | Local failed rows plus `Errores de sincronización` card; no PDF/print; `Abrir` only when a local row exists |
| Selected `failed`, web | Empty view; no failed-row card is fabricated |
| Row is draft | Badge `Borrador`; `Abrir` only |
| Row is signed/superseded/voided | Badge and meta; `Abrir`, `Descargar PDF`, `Imprimir` |
| Row older than local retention, desktop offline | `Abrir` remains if a local row exists; PDF/print disabled; hint `Requiere conexión para regenerar esta receta.` |
| Row older than local retention, online | Cache absence is not shown; download/print fetch and regenerate on demand |
| Row has no server copy, ever | Provisional badge; no PDF/print; never described as expired |
| Long patient name | Title wraps, actions keep intrinsic width and remain visible; below 640px the row stacks and actions wrap |
| Return from detail after sign/void/supersede | The effect re-fetches on mount; the row paints the new status and number on the next result. No local mutation is applied |
| Session expires mid-view | The next request returns unauthorized; `ErrorText` paints; navigating away returns to `/login`; stale rows are not kept forever |

## 5. Style

Every value is a `context/Design.md` token declared in `persona-sanus/assets/main.css` `:root`. No new color, font, icon package, CDN, or webfont.

### 5.1 Color

| Use | Value | Token |
|---|---|---|
| Page canvas, filters, row surfaces | `#ffffff` | `--color-paper` |
| Title, row title, filter values | `#061d29` | `--color-ink` |
| Subtitle, meta, labels, placeholder, Secondary border/text | `#425d6d` | `--color-mist` |
| Row, filter, and empty hairlines | `#e5e7eb` | `--color-hairline` |
| Primary fill and focus ring | `#006e75` | `--color-deep-teal` |
| Primary hover | `#0b978e` | `--color-bright-teal` |
| Empty and provisional surfaces | `#fffded` | `--color-cream` |
| Failed surface and danger wash | `#ffc3e6` | `--color-blossom` |
| Failed text and Alert glyph | `#4d0037` | `--color-plum` |
| Focus halo | `rgba(185,255,232,0.55)` | `--color-mint-glass` |

Never `#000000`. Never a chromatic border: every border is the hairline; status is carried by the badge word, not a colored outline. Measured contrast: Ink/Paper 17.28 · Mist/Paper 6.96 · Paper/Deep Teal 6.02 · Ink/Cream 16.89 · Mist/Cream 6.80 · Plum/Paper 15.32 · Plum/Blossom 10.36. AA for body; AAA where it clears.

### 5.2 Typography

| Element | Family | Weight | Size / line-height |
|---|---|---|---|
| Page title (`h1`) | `--font-mint` | 500 | 22px / 1.22 Ink |
| Page subtitle | `--font-mint` | 400 | 14px / 1.5 Mist |
| Filter label | `--font-mint` | 500 | 14px / 1.5 Ink |
| Filter value / placeholder | `--font-mint` | 400 | 16px / 1.5 Ink / Mist |
| Primary and Secondary actions | `--font-mint` | 500 | 16px / 1.22 |
| Row title | `--font-mint` | 500 | 18px / 1.25 Ink |
| Row meta | `--font-mint` | 400 | 14px / 1.5 Mist |
| Status and provisional badge | `--font-mint` | 500 | 13px / 1.22 |
| Empty title / description | `--font-mint` | 500 / 400 | 18px / 1.25 Ink; 14px / 1.5 Mist |
| Hint / error | `--font-mint` | 400 | 14px / 1.5 Mist / Plum |

No `--font-grenette`: application chrome, not a 46px+ editorial opener.

### 5.3 Geometry

- 4px base. `PageShell`: 48px top, 80px bottom, 24px inline; `.ui-container` max 1200px; `.ui-stack` 24px gap.
- `PageHeader`: flex row, space-between, wrap, 16px gap, align start; title/subtitle 4px apart; `Nueva receta` at the inline end, 8px radius, `12px 24px`.
- `FilterBar`: auto-fit columns with a 180px minimum and 12px gap, aligned end; at 1200px the search takes the wide field and state/date share the row; below 760px one column. The search input and the date input keep `min-height: 44px`, `8px 12px` padding, 8px radius, 1px hairline. No filter card, no shadow.
- `ListRow`: 24px radius, 1px hairline, `16px 24px` padding; list gap 8px; title/meta group `min-width: 0`, 2px title/meta gap; trailing actions `display:flex`, 12px gap, wrap, top-aligned. Below 640px the row becomes a column and actions move to the inline start.
- `SectionCard` sync errors: 24px radius, 1px hairline, 24px padding, 16px below the record slot; rows inside are actionless and separated by 8px hairlines.
- `EmptyState`: Cream, 24px radius, 1px hairline, 32px padding, centered; description max 48ch, 8px under the title.
- `Badge`: 9999px radius, `4px 12px`, 13px / 1.22; provisional Cream with Ink; lifecycle badge uses the shared `BadgeTone` mapping, never a new color.
- Elevation: flat. No shadow on filters, rows, badges, or the sync card; the only ring is the focus halo. No horizontal scroll; actions wrap instead of clipping.

### 5.4 Iconography and imagery

Line icons only, 1.5px stroke, 20px default, Ink or Plum: the Alert glyph in `ErrorText` and the sync-error card. No icon is added to `Abrir`, `Descargar PDF`, or `Imprimir`; the row is already action-dense. No avatar, patient photo, clinic logo, document thumbnail, A4 sheet, QR, signature, stamp, illustration, or chart. The navbar mark and sync dot are chrome (`Header.md` §5.4), not this page.

## 6. Accessibility

- Landmarks: the navbar is the banner (`Header.md` §6). `PageShell` is the content region; the filter bar is a labelled group (`role="search"` only if it actually filters the list, with an accessible label `Filtros de recetas`); the sync-error card is a labelled `section`. The list is a list, not a table: `ListRow` roots render as `<li>` inside a `<ul>` or as a group with an accessible name. No `DataTable` is introduced.
- Headings: one `h1`, `Recetas`; the denied state has one `h1`, `Acceso denegado`. No `SectionCard` means no `h2`; if the sync-error card is a `SectionCard`, its title `Errores de sincronización` is the only `h2`. No skipped level, no label styled as a heading.
- Tab order: `Nueva receta` when present, then `Paciente` search, `Estado` select, `Fecha` input, then each row's `Abrir`, `Descargar PDF`, `Imprimir`, in row order. Disabled PDF actions are skipped by Tab; the hint is not focusable; `Abrir` always remains reachable. The sync-error card has no actions, so it adds no stops.
- Focus: `outline: 1px solid var(--color-deep-teal); outline-offset: 2px; box-shadow: 0 0 0 3px rgba(185,255,232,0.55)` on every control and link, visible on Paper and Cream. Disabled actions keep a readable label and must not receive focus.
- Labels: the search label is visible to assistive tech and is the accessible name; the placeholder `Nombre del paciente` is supplementary. `Estado` and `Fecha` are visible wrapping labels. Each row action is named by its visible text; if `ListRow` names the group by the patient, the action's accessible name remains `Abrir` plus the row context, not `Abrir` alone in an ambiguous list.
- Status: badges are text, never color alone. The provisional badge carries the full word; the failed surface carries a sentence and an Alert icon. Disabled PDF actions expose `aria-disabled="true"` and are still identified by text; the reason is the adjacent hint, not a hover-only tooltip.
- Errors and loading: `ErrorText` is `role="alert"`; `LoadingState` is `role="status"` with `aria-live="polite"`; the spinner is `aria-hidden`. The error does not move focus; cached rows are not announced as a fresh successful load.
- Keyboard and touch: filters are native controls; rows are links/buttons, never a click-only `<tr>` or `<div>`; no drag, no hover menu, no select-all. Actions keep a 44px target; the row action gap is 12px.
- Reduced motion: the spinner stops under `prefers-reduced-motion: reduce`; no other animation is introduced.
- Zoom/reflow: at 200% and 320px, filters stack one per row, rows stack title above actions, actions wrap, and nothing scrolls horizontally.

## 7. Implementation notes

The shipped component is `fn Prescriptions(status: Option<String>)` at `persona-sanus/src/main.rs:5349`, `#[component]` at `:5348`, closing at `:5694` before `PrescriptionNew` at `:5697`. It composes the adopted `ui/` primitives: `PageShell` (`id="prescriptions-page"`), `PageHeader`, `FilterBar`, `SearchField`, `SelectField`, `TextField`, `ListRow`, `Badge`, `SectionCard`, `EmptyState`, `LoadingState`, `ErrorText`, `HintText`, `LinkAction`, and `PrescriptionStatusBadge` (`persona-sanus/src/ui/`).

Shipped state (present tense):

- Signals: `rows`, `query`, `status` (seeded once from the route `status` argument, default `""`), `date` (empty), `message`, `failed_rows: Vec<(String, String)>`, `loading` (starts true), and `loaded` (starts false, `main.rs:5364`). The `loaded` gate is shipped: the record slot shows `LoadingState` `Cargando recetas...` (`main.rs:5596`) until the first request settles, and rows stay mounted while later requests are in flight.
- Session gates: signed-out replaces the route with `Route::AuthPage {}`; the read gate is `Me::can_read_prescriptions()`; a checked-but-unresolved actor renders `LoadingState` `Redirigiendo...` inside `PageShell`.
- Fetch: on mount and on signal change the effect calls `api::fetch_prescriptions("", selected_status, selected_date)` for normal views; the patient term never leaves the client. Desktop `pending_sync` reads `desktop_local::list_prescriptions(&mut conn, "pending_sync")` and returns; `failed` reads `"failed"` and then fills `failed_rows` from `desktop_local::outbox_rows()` filtered to `status == "failed"`, taking `operation` and `lastError`. Any other desktop error opens the local DB, lists `""`, replaces `rows` with decodable cached values (failures skipped), and keeps `message` visible.
- Patient search is client-side only: `query().trim().to_lowercase()` matched against `patient_name.to_lowercase().contains(...)`; server/cache order is preserved, never re-sorted; typing fires no request.
- Empty state is shipped per §3.4: a settled empty view renders `EmptyState` `Sin recetas` with `No hay recetas para estos filtros.` when the `filtered` flag (search, status, or date non-empty) holds and `Todavía no hay recetas en esta clínica.` otherwise. `message` renders as `ErrorText` above rows; a fetch error with no cached rows shows the error alone, never a false empty state.
- Rows: `ListRow` with `PrescriptionStatusBadge { status }`, meta `{prescription_status_label} · {format_date(created_at)}` plus `No. {rx_number}` when present (`main.rs:5612-5623`). Status labels come from `ui/format.rs:97-106` (`Borrador`, `Firmada`, `Reemplazada`, `Anulada`, `Provisional`, `Desconocido` fallback). The `pending_sync` view adds a Cream `Badge` `Provisional · PENDIENTE DE SINCRONIZACIÓN`.
- Actions: `Abrir` is a Secondary `ButtonLink` to `Route::PrescriptionDetail { id }` on every row. `Descargar PDF` and `Imprimir` are Secondary `LinkAction`s to `/api/prescriptions/{id}/pdf` (print with `target="_blank"`, `rel="noopener noreferrer"`) rendered only for `signed`/`superseded`/`voided`. The header Primary `Nueva receta` links to `Route::PrescriptionNew { patient_id: None, template_id: None }` and paints only when `profile.can_create_prescriptions()`.
- Expired-offline guard is shipped: `is_expired` (`main.rs:5510-5532`) is true only for a server-backed final row older than `clinic.retention_days` while desktop is offline and not in a local view; expired rows render disabled PDF/print anchors (`aria-disabled`, `tabindex="-1"`) plus `HintText` `Requiere conexión para regenerar esta receta.`; `Abrir` stays enabled. Unsynced rows are never treated as expired.
- Sync errors: a non-empty `failed_rows` renders `SectionCard` `Errores de sincronización` with one actionless `ListRow` per entry (`operation` title, `lastError` meta), independent of the record slot.

Resolved gap:

- Doctor-own scope is now enforced server-side: `service::list_prescriptions` (`persona-sanus/src/service.rs:3306-3320`) passes the actor's doctor id as `doctor_filter` for `Role::Doctor` and `None` otherwise, so a Doctor receives only their own clinic rows. The former §4.2 divergence is closed; no client-side filter is used or needed. The §4.2 bullet's shipped-divergence wording is historical and the code now matches the §2.1 matrix.

Recorded as intended: the detail read stays clinic-wide (`service::get_prescription`) — the §2 matrix (`PrescriptionDetail.md` row 52) grants non-owner doctors a read view, so only signing and transitions are owner-scoped.

Verification commands, from the repo root; none is recorded here as executed:

```sh
cd persona-sanus && cargo check
dx build --release --fullstack
caveman-blocks run persona-smoke --dir persona-sanus --port 8091
python3 .blocks/markdown-table-check.py --path context/screens/done/Prescriptions.md --strict
```

The smoke script covers `/prescriptions` for an authenticated actor; the pending and failed desktop views need a browser or the desktop probe because they read the local cache. A doctor role pass must show the intended own-scope behavior only after the service gap above closes; today it shows the clinic-wide list, and this spec must not claim otherwise.

- Do not add: a route or endpoint beyond `/prescriptions` and the existing `GET /api/prescriptions/:id/pdf`; any query parameter for patient, date, or the selected status; a sign, void, supersede, edit, delete, reorder, share, LLM, or template control; a second Primary; PDF/print for drafts, provisional rows, failed rows, or unknown statuses; a stored PDF, a PDF preview, an A4 sheet, a QR, a document thumbnail, a patient photo, or a signature/stamp asset; billing, price, trial, plan, entitlement, invoice, or `sync_locked` copy; rich text, a formatting toolbar, `contenteditable`, or a WYSIWYG editor; a sort, result count, pagination, infinite scroll, bulk action, or saved filter; a success toast; a doctor/owner filter or a client-side clinic filter; a synthetic row for an evicted cache entry; or a new field, table, column, role, permission, or API argument.

## 8. Spec references

| Spec | Connection |
|---|---|
| `specs/done/15-O-ui-tokens-actions-and-formatting.md` | `15-O` supplies the tokens, `ButtonLink`, and format helpers (`format_date`, `prescription_status_label`) the rows render. |
| `specs/done/16-P-ui-forms-feedback-and-layout.md` | `16-P` supplies `PageShell`, `PageHeader`, `FilterBar`, `SearchField`, `SelectField`, `TextField`, `ListRow`, `Badge`, `SectionCard`, `LoadingState`, `EmptyState`, and `ErrorText`, the list filter and result surface. |
| `specs/done/18-R-ui-domain-and-billing.md` | `18-R` builds `PrescriptionStatusBadge`, the badge each row renders. |
| `specs/done/23-W-ui-prescriptions.md` | `23-W` adopts `Prescriptions`: `FilterBar` filters, the `loaded` gate, one Primary `Nueva receta`. |
| `specs/done/25-Y-ui-legacy-css-cleanup-and-docs.md` | `25-Y` removes legacy page-ID selectors so the adopted list runs only on `.ui-*` classes. |

