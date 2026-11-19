# PersonaSanus — PrescriptionPreview (`/prescriptions/:id/preview`)

| Field | Value |
|---|---|
| Source | `context/sitemap.md` §1.2 (`/prescriptions/:id/preview`, A4 preview), §2 (roles per route), §3 flows 1, 2, 5, §5 (conventions); `context/PRD.md` §5.2 FR-013/FR-014, §5.4 FR-022 and FR-030–FR-034, §5.5 FR-040–FR-043, D-01, D-04, D-06, D-21; `context/Design.md` (Tokens — Colors, Tokens — Typography, Tokens — Spacing & Shapes, Primary Action Button, Outlined Secondary Button, A4 Preview Card, Green Pill Badge, Elevation); `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` step 15, Page adoption row `PrescriptionPreview` (line 230), primitives `PageShell`, `PageHeader`, `ActionRow`, `Button`, `LinkAction`, `Notice`, `ErrorText`, `HintText`, `LoadingState`, `PrescriptionStatusBadge`, `.ui-preview-canvas`; `context/screens/INDEX.md` §2.3 |
| Component | `PrescriptionPreview` — `persona-sanus/src/main.rs:4938-5379` (`#[component]` at `:4938`, `fn` at `:4939`). Composes `forms::A4Preview` (`persona-sanus/src/forms.rs:464`), `api::fetch_prescription`, `api::fetch_prescription_preview`, `sign_prescription_dispatch`, `prescription_status_label`, and `pdf::render`. Planned primitives live in `persona-sanus/src/ui/`; that directory is planned, not shipped |
| Route | `/prescriptions/:id/preview` — authenticated; `Route::PrescriptionPreview { id: String }` at `persona-sanus/src/main.rs:93-94`. A signed-out visitor is replaced with `Route::AuthPage {}`. No query params; the route id is the only URL value |
| Layout | Inside `#[layout(Navbar)]` (opened at `main.rs:51`, never closed). `PageShell id="prescriptions-page"` → `.ui-page` → `.ui-container.ui-stack`. Header and action regions are Paper; the preview canvas is Cream; the provisional Notice is Blossom. No full-bleed exception |
| Locale | `es-EC` on every string. `receta`, `cédula` keep Ecuador spelling. Dates `DD-MM-AAAA`, 24-hour time (`%d-%m-%Y %H:%M`) |
| Status | ✅ Spec complete (sections 1–7). Route status: ✅ — sitemap cell is implemented; shipped: authenticated A4 projection, role/status sign branch, desktop provisional fallback, PDF and print links, wasm share, in-process provisional byte report; target: `PageShell`/`PageHeader`/`ActionRow`/`Notice`/`ErrorText` adoption and `.ui-preview-canvas` with the same data branches |

## 1. Purpose

An authorized clinician opens one prescription as a document, not as an editor, and does four jobs in order:

1. Inspects the A4 projection at true scale — margins, repeated header and footer, page breaks, pinned images, QR area, number, date, verification code — before issuing anything.
2. Signs and issues a draft when the actor is a doctor: first without acknowledgment, then, only after the warning conflict is surfaced, with explicit acknowledgment of unknown template variables (FR-013, FR-023).
3. Obtains the final artifact after signing, or for an already-issued prescription: downloads the PDF, opens the same URL for printing, or invokes the platform share sheet (FR-030–FR-032, FR-041).
4. Recognizes a provisional offline document and never mistakes its bytes for a final numbered, publicly verifiable prescription (FR-033, FR-043).

It never renders prescription edit controls, diagnosis or instruction inputs, medication reordering, void or supersede actions, template version pickers, image-library controls, or LLM links; those belong to the detail, history, template, and clinic screens. It never claims the browser projection is byte-identical to the renderer: `forms::A4Preview` is the projection, `pdf::render` is authoritative (FR-022, D-01). It never renders a PDF on load, in the background, or as a silent write; every render sits behind an explicit action (FR-031), and no PDF is stored.

It never places PHI in the route URL, the share payload, the QR payload, or audit metadata. The route carries only the opaque prescription id; share carries a static title and the download URL. The final filename carries patient and doctor names because FR-032 mandates it, and the server produces it at response time; this screen never builds it into a route, query, share title, or log line. Nothing here renders billing, price, trial, plan, entitlement, or `sync_locked` copy or state (v1 is a free beta, D-04); signature or stamp imagery or assets (D-06 — only the document's printed blank signature rule exists); rich text, `contenteditable`, or a formatting toolbar (D-21). The provisional sheet renders no scannable QR and no final number, and the page states that limitation.

## 2. Structure

One column: page header, one action row, conditional feedback, then the document canvas. The A4 sheet is the only large surface and is not nested in a second card. Region order is fixed for every role and status; §2.1–§2.4 change which controls and feedback appear, never the order.

| Order | Region | `id` / class | Band tone | Contents |
|---|---|---|---|---|
| — | Navbar (layout) | `header.ui-navbar`, inner `.ui-navbar-inner` | Paper | Owned by `context/screens/Header.md`; identity, sync indicator and sign-out only |
| 1 | Page shell and stack | `#prescriptions-page` · `.ui-page` > `.ui-container.ui-stack` | Paper | Page width and vertical rhythm; one root for every state |
| 2 | Page header | `header.ui-page-head`, `h1.ui-page-title`, `p.ui-page-subtitle`, `.ui-page-head-actions` | Paper | Patient-specific A4 title, status/number/date subtitle, Secondary back action |
| 3 | Action row | `.ui-actions` | Paper | One primary at most (sign or download), then print, desktop-local provisional, wasm share, conditional acknowledgment |
| 4 | Provisional notice | `.ui-notice.ui-notice--warning` | Blossom | `Notice` Warning when the payload is provisional; no action |
| 5 | Message region | `.ui-error-text` · `.ui-hint-text` | Paper | `ErrorText` for load, sign, or render failures; `HintText` for the provisional byte report |
| 6 | Preview canvas | `.ui-preview-canvas` | Cream | Horizontally scrollable, 24px padded container holding every A4 page |
| 7 | A4 sheet | `.a4-pages` > `.a4-page` | Paper | 210×297 mm page at the payload margin; header, optional images, body, footer meta, QR area, page number |
| 8 | A4 variable warning | `.a4-warnings` | Paper | Ember text under the pages when header or footer variables are invalid |

The table above is the whole page. There is no toolbar, sidebar, metadata grid, medication list, audit panel, or document history. The navbar band is not repeated copy (§3.1). The preview canvas is the document's only container; the sheet carries the paper tone and the sole shadow. Orders 6–8 sit inside the canvas, not in sibling bands.

### 2.1 Role state

All four clinic roles (`administrator`, `doctor`, `nursing`, `secretary`) reach the screen only through `Me::can_read_prescriptions()`; a session without a role gets the denied notice and no canvas or action row. Doctor is the only signing role, and only while the payload status is `draft`. Nursing, secretary, and administrator see the document, subtitle, and back action, plus download, print, local-provisional, and share controls on a non-draft payload, but never the sign primary, the acknowledgment secondary, or its busy state. Roles change the action row and only the action row; header copy, sheet geometry, QR presence, provisional notice, and error surfaces are identical.

The client gate is `profile.can_sign_prescriptions()` plus `current.status == "draft"`. It is not an ownership check: the preview payload has no owner field, so a doctor who is not the author still sees the sign control. The server's `prescription_sign_allowed` requires a draft and an absent or matching `doctor_id`; a refusal is rendered as the API message in the message region, and the screen never optimistically flips to signed.


### 2.2 Prescription status state

`draft`: for a doctor the sign group is the page's only primary; no download, print, local provisional, or share action is offered, because the PDF endpoint refuses a prescription until a verification code exists. A normal draft has no code, so the server marks its preview provisional: the warning notice is present and the QR area is a placeholder with no readable code. The authored content stays final-shaped and immutable while previewing.

`signed`: download is the primary and only filled action; print is secondary; desktop adds the local provisional render only when the payload is provisional; wasm adds share. Status, number, issue date, readable verification code, and active QR appear when the server payload carries them. Header and canvas keep draft geometry; values change.

`superseded` and `voided`: geometry and actions read exactly like `signed` — download and print remain available for the historical record, share and local-provisional rules are unchanged, and no sign action exists. Only the status label and subtitle differ; the sheet is the immutable issued version, not a re-render of current data. An unknown status uses the unknown label and is treated as non-draft: download is the only primary action.

### 2.3 Surface state

Web/wasm renders download, print, and share. Share opens the browser sheet with a static title and the absolute download URL; it contains no PHI and no local file. A cancelled, rejected, or unsupported share produces no message and no navigation.

Desktop renders the download and print links, and the local provisional render when the payload is provisional. There is no share control. The desktop loads `fetch_prescription` and `fetch_prescription_preview`; if the preview fails it opens the local database, rebuilds the payload from the cached prescription, cached template version, and local medication items, and marks it provisional. It does not invoke the renderer to print: the print control is the same URL anchor as on web. The local provisional control calls `pdf::render` in-process and reports the resulting byte length; it does not write or download a file. Desktop sign uses `sign_prescription_dispatch` with the cached prescription and profile as fallback: with the server reachable it posts; offline it consumes the next device-block number and returns a signed row. Both surfaces show the sheet at the same millimeter size, with horizontal scrolling under the fixed width.

### 2.4 Denied, loading, empty, and error state

Session check in flight: the shell root renders `LoadingState` with the shortest label; no header, actions, or canvas.

No session: no page content renders; the effect replaces the route with the auth page.

Actor unresolved after the check: the minimal root renders the redirect label; no fetch result is drawn.

No role or no read permission: a static `Notice` Error with the access-denied title and body. No page header, subtitle, canvas, actions, or data fetch. This is presentation only; the preview endpoint independently refuses.

Preview fetch in flight: root plus `LoadingState`; no provisional notice, no action row, and no empty sheet. The previous sheet is not kept, so stale clinical content never sits under a spinner.

Preview fetch failed with no local fallback: `ErrorText` carrying the API message and the single back link to prescription history. No title, status, actions, or sheet; this is the no-current branch.

Preview fetch failed but the desktop cache built a payload: the full provisional page renders, and the message is cleared. If the local lookup also fails, the error branch stands.

`A4Preview` document parse or validation failure: header and actions still render; the canvas contains only the document error text. The page does not blank the patient header or substitute a sample document.

No medication lines: the sheet still renders header, footer meta, QR area, and page number. There is no empty-state card on the canvas.

## 3. Copy text

Every string is user-visible `es-EC`, quoted with final punctuation. Structural punctuation is copy. All prescription numbers use `No.`.

### 3.1 Chrome reference

The navbar is not this page's copy. Labels, session states, sync strings, and sign-out belong to `context/screens/Header.md` §3. This page supplies no public footer band and adds no chrome string.

### 3.2 Page header

| Element | Copy |
|---|---|
| Document title (`h1`) | `Vista previa A4 — {patient_name}` |
| Fallback for `{patient_name}` | `Receta` |
| Status label (`draft`) | `Borrador` |
| Status label (`signed`) | `Firmada` |
| Status label (`superseded`) | `Reemplazada` |
| Status label (`voided`) | `Anulada` |
| Status label (other) | `Desconocida` |
| Number fragment | `No. {rx_number}` |
| Subtitle separator | ` · ` |
| Header secondary action | `Volver a la receta` |

Subtitle order is status label, number fragment when a number exists, then `{issued_at}`. A draft with no number reads `Borrador · {issued_at}`. `{issued_at}` is already localized by the API; the page adds no date format. The status label uses the `PrescriptionStatusBadge` vocabulary but remains subtitle text, because `PageHeader.subtitle` is a string.

### 3.3 Action row

| Element | Copy | Role / target |
|---|---|---|
| Sign and issue | `Firmar y emitir` | Primary, draft and sign role only; dispatches with acknowledgment false |
| Sign busy | `Firmando...` | Same control, disabled and `aria-busy="true"` while in flight |
| Acknowledge-warnings sign | `Firmar reconociendo las advertencias` | Secondary, draft and sign role, revealed only after the warning conflict |
| Download | `Descargar PDF` | Primary when the status is not draft; `GET /api/prescriptions/{id}/pdf` |
| Print | `Imprimir` | Secondary; same PDF URL in a new browsing context, `noopener noreferrer` |
| Local provisional render | `PDF provisional (local)` | Secondary desktop only, provisional non-draft payload |
| Share | `Compartir` | Secondary wasm only; Web Share sheet |
| Share title | `Receta médica` | `ShareData.title`; not visible on the page |
| Local provisional title | `Receta-{id}.pdf` | `PdfInput.title` for the in-process provisional render |
| Final filename | `{PatientName}-receta-{DD-MM-AAAA}-{DoctorName}.pdf` | Server `Content-Disposition` (FR-032) |

The sign primary and download primary are mutually exclusive. Print, local provisional, and share never become primary. There is no cancel or decline action: leaving the page is the header back action.

### 3.4 Notices and messages

| Element | Copy |
|---|---|
| Provisional notice | `Documento provisional: se muestra sin QR escaneable hasta sincronizar.` |
| Provisional bytes success | `PDF provisional generado ({bytes} bytes) con marca PENDIENTE DE SINCRONIZACIÓN.` |
| Load fallback, preview | `No se pudo cargar la vista previa.` |
| Parse fallback, preview | `No se pudo leer la vista previa.` |
| Parse fallback, prescription | `No se pudo leer la receta.` |
| Parse fallback, signed row | `No se pudo leer la receta firmada.` |
| Offline sign unavailable | `Sin conexión.` |
| Warning-acknowledgement message | `La plantilla tiene variables desconocidas.` |
| Document parse failure | `El documento no tiene el formato estructurado esperado.` |
| Header/footer variable warning | `Variables no válidas en la cabecera o el pie: {keys}` |
| Provisional render failure | the renderer or API error string, unchanged |

The acknowledgment control appears only when the returned message contains `variables desconocidas`; the screen never guesses from a warning count. `{keys}` is the renderer's comma-separated list of unresolved keys.

### 3.5 Sheet text

| Element | Copy |
|---|---|
| Meta with number and date | `Receta No. {number:06} · {date}` |
| Meta with date only | `Fecha de emisión: {date}` |
| Provisional stamp | `PENDIENTE DE SINCRONIZACIÓN` |
| Readable code | `Código de verificación: {code}` |
| Printed signature rule | `Firma: ______________________________` |
| Medication line | `• {name} · {dose} · {route} · {frequency} · {duration} · {quantity} · {notes}` |
| Provisional QR placeholder | `QR provisional` |
| QR placeholder without code | `QR de verificación` |
| QR image alternative | `Código QR de verificación` |
| Logo image alternative | `Logo de la clínica` |
| Image missing-data fallback | `Imagen: {caption}` |
| Page number | `Página {page} de {total}` |

The medication line and signature rule are renderer strings. The medication line drops empty fields and is omitted when both name and dose are empty. Letterhead, body, footer, and medication values are authorized clinical data, not copy.

### 3.6 Loading, access, and accessible names

| Element | Copy |
|---|---|
| Session-check loading label | `Cargando...` |
| Actor-resolution label | `Redirigiendo...` |
| Preview loading label | `Cargando vista previa...` |
| Access-denied title | `Acceso denegado` |
| Access-denied body | `No tienes permiso para ver esta receta.` |
| Error-state back link | `Volver a recetas` |
| Canvas accessible name | `Vista previa de la receta` |
| Page-number visually hidden prefix | `Página` |

The QR and logo alternatives are §3.5. The canonical back action label is §3.2; the error-state link is the one above.

## 4. Displayed information

### 4.1 What the screen shows

| Element | Source | Value shown |
|---|---|---|
| Document title | `preview` signal, `api::PrescriptionPreview.variables` entry with key `patient.name`; fallback at `main.rs:5144-5149` | The authorized patient's full name, or `Receta` when the pinned set lacks the entry |
| Status label | `preview.status` through `prescription_status_label` (`main.rs:4329-4337`), target `PrescriptionStatusBadge` | The status label from §3.2 |
| Number and issue date | `preview.rx_number` (`Option<i32>`), `preview.issued_at` | Number when assigned, omitted while none; server-localized signed or created timestamp as `DD-MM-AAAA HH:MM` |
| Back target | `preview.id` | Detail route for the same prescription |
| Sign eligibility | context `Me::can_sign_prescriptions()` plus `preview.status` | Sign primary only when both hold |
| Acknowledgment control | local `needs_ack`, set when the sign error contains `variables desconocidas` | Secondary acknowledgment control after a warning conflict |
| Sheet structure and values | `preview.doc_json` through `forms::parse_document`, every `preview.variables` pair keyed by `key` | Pinned version's letterhead, body, footer, images, colors; resolved variable text |
| Medications | `preview.medications` (`Vec<MedicationBlock>`) | Medication lines in body order |
| Page margin | `preview.margin_mm`, clamped 5–40 in `paginate_document` | Per-page padding on the 210 mm sheet |
| Logo and images | `preview.image_sources` (`asset_id`, `data_url` pairs) | Inline image for the pinned logo or slot, caption fallback when bytes are absent |
| QR and readable code | `preview.qr_data_url`, built server-side from the verification URL; `preview.verification_code` | QR image and the readable code line in the footer, or the provisional stamp |
| Page count | computed `layout.pages.len()` in `forms::paginate_document` | Current page and total |
| Variable warnings | `resolve_text` warnings over letterhead doctor, clinic, and footer | List of unresolved keys |
| Provisional byte report | length of the `pdf::render` result | Count of locally produced bytes; no file |
| Download URL | `preview.id` formatted as `/api/prescriptions/{id}/pdf` | Current prescription's PDF endpoint |
| Share URL | browser origin plus the download URL | Absolute download URL; no PHI |

`api::fetch_prescription` also fills the `prescription` signal. That raw row is not drawn: desktop uses it as the offline-sign fallback, and a desktop preview failure uses it for status, number, template version, timing, patient id, and items. When the server is reachable and the preview succeeds, every displayed document value comes from `PrescriptionPreview`.

### 4.2 Hard rules

- The server is the authorization boundary. The client gates (`can_read_prescriptions`, then `can_sign_prescriptions` plus `draft`) only select controls; they never grant read, sign, or PDF access. A refusal renders the API error and does not alter the sheet or navigate.
- Signed, superseded, and voided documents are immutable here. The screen renders the pinned `doc_json` and its variables and offers no edit, reorder, re-sign, void, supersede, version-regenerate, or template-pin control. Corrections happen on the detail route.
- A4 is a projection: 210×297 mm, configured margin, page breaks, repeated header and footer, embedded sheet typography. `pdf::render` is authoritative for the issued PDF. Never state or imply byte-identical output; never add a client-side PDF renderer (D-01; FR-022, FR-030).
- The browser never links `printpdf`: web downloads the server response. Desktop may invoke the shared renderer for the provisional byte report, but no PDF is stored and no issued file is cached here.
- No silent generation: loading the screen performs no PDF render attributed to the user. Download, print, and share are explicit; the provisional byte report runs only on its button. No auto-download on load, sign success, or sync.
- Provisional means provisional: no scannable QR, no final number, no verification code, and no public-verifiability claim until sync reconciles the device block and the server assigns the code (FR-014, FR-033, FR-043). The post-sync final PDF is a new render, not this provisional file renamed.
- The QR payload is the verification URL only and the image comes from `qr_data_url`; never encode patient name, cédula, diagnosis, medication, or prescription id. Never render a verification seal or a code the payload does not carry.
- No PHI in the URL: the route carries only the opaque id; share is a static title plus that URL. The FR-032 filename carries patient and doctor names server-side and is never constructed into a route, query, share title, or log line.
- At most one filled action per viewport: exactly one of sign (draft, sign role) or download (non-draft). Acknowledgment, print, local provisional, and share are outlined Secondary; no second primary appears in any state or width.
- No signature or stamp image, upload, slot, preview, or metadata (D-06). The footer's printed signature rule is text; the doctor signs paper. No signature-styled control exists here.
- No billing, price, trial, plan, entitlement, or `sync_locked` copy or state (D-04). This page has no plan surface.
- No rich text, `contenteditable`, formatting toolbar, or HTML document source (D-21). The sheet is generated from structured `doc_json` plus plain values.
- Never claim share or print success, generation, synchronization, or a number the payload does not carry. Errors stay inline; there is no toast, modal, or silent page replacement.
- No new route, endpoint, table, column, role, or permission. Use `fetch_prescription`, `fetch_prescription_preview`, `sign_prescription`, `GET /api/prescriptions/{id}/pdf`, and the existing `pdf::render`; the provisional offline path is desktop-only.
- The route param is the only identifier. Missing or blank id, unknown id, and foreign-clinic id are server errors rendered in the message region, not a local empty document.

### 4.3 Dynamic states

| State | Screen behaviour |
|---|---|
| Session check in flight (`checked == false`) | Shell root only, `LoadingState` with `Cargando...`; no header, actions, or canvas |
| No session | Effect replaces the route with the auth page; no preview content renders |
| Actor unresolved (`actor == None`) | Shell root only, label `Redirigiendo...`; no navigation and no fetch result drawn |
| No role or read permission | `Notice` Error, `Acceso denegado` plus `No tienes permiso para ver esta receta.`; no canvas, no actions |
| Preview fetch in flight | Root plus `LoadingState` with `Cargando vista previa...`; no actions and no empty sheet |
| Draft, doctor (author or not) | Status, subtitle, canvas, primary sign action, header back. Identity cannot distinguish ownership, so the control is identical; the server authorizes or refuses |
| Draft, doctor, sign busy | Sign control disabled, label `Firmando...`, `aria-busy`; no second dispatch from it; canvas stays; failure replaces the busy state |
| Draft, doctor, warning conflict | Sign error message plus acknowledgment secondary control; sign primary stays enabled; acknowledgment re-dispatches with acknowledgment true |
| Draft, nursing, secretary, administrator | Read-only canvas, header back, provisional notice when the payload has no code; no sign, download, print, local, or share control |
| Signed, any role | Canvas with code and QR when the server carries them; download primary, print secondary; desktop local only when provisional; wasm share |
| Superseded or voided | Same geometry and download/print as signed; status and subtitle show the terminal state; no sign |
| Unknown status | `Desconocida` label; treated as non-draft; download and print; no sign |
| Provisional offline (desktop local preview) | Notice Warning; margin 12 mm; no verification code; no QR image; sheet built only from `patient.name` plus local items; status and number from the cache. A signed row still shows download and local provisional, but download needs the server |
| Provisional local render success (desktop) | `HintText` with the byte report; no file written; the page does not claim a download; notice stays |
| Provisional local render failure | `ErrorText` with the renderer message; no partial file; canvas and notice stay |
| Preview load error, no local fallback | `ErrorText` with the server or API message, or the load fallback; `Volver a recetas`; no title, actions, or canvas |
| Preview load error, local fallback succeeds | Full provisional page; message cleared; no error surface |
| Preview parse or validation error | Header and action row render; canvas is replaced by the document error text; retry means leaving and reopening, not an in-page button |
| Sign success | Returned prescription replaces the `prescription` signal, sync is reprobed, `generation` increments and refetches preview and prescription. The signed branch renders when the refetch resolves. Success does not clear the message, so an earlier error can outlive it until a later message write or reload |
| Offline sign success (desktop) | Same generation bump; returned row has `status = signed`, the new `rx_number`, `rx_block_id`, `signed_at`; payload is provisional; download requires sync; local provisional reports bytes |
| Sign failure | Sign leaves busy state; message shows the API error; draft status and eligibility remain; the acknowledgment control appears only for the unknown-variables conflict |
| Download or print click | Ordinary navigation to the PDF URL, print in a new tab; no in-page busy state or toast. A server refusal replaces the tab response with the endpoint error |
| Share | Wasm opens the Web Share sheet with `Receta médica` and the absolute download URL; cancel, reject, or unsupported exits silently. Desktop has no share control |
| Narrow viewport or zoom | Canvas scrolls horizontally; the sheet keeps 210 mm width and is never scaled, reflowed, or cropped by the page |

## 5. Style

Every value is a `context/Design.md` token declared in `persona-sanus/assets/main.css` `:root`. The A4 document keeps its own print geometry and embedded fonts; the surrounding shell uses the product tokens only.

### 5.1 Color

| Use | Value | Token |
|---|---|---|
| Page canvas, header and action band | `#ffffff` | `--color-paper` |
| Page title, subtitle, action labels | `#061d29` | `--color-ink` |
| Back action text and outline | `#425d6d` | `--color-mist` |
| Hairline between header, actions, and canvas | `#e5e7eb` | `--color-hairline` |
| Primary sign/download fill | `#006e75` | `--color-deep-teal` |
| Primary hover fill | `#0b978e` | `--color-bright-teal` |
| Provisional canvas wash | `#fffded` | `--color-cream` |
| Provisional notice surface | `#ffc3e6` | `--color-blossom` |
| Notice and error text | `#4d0037` | `--color-plum` |
| Focus ring | `#006e75` | `--color-deep-teal` |
| Focus halo | `rgba(185,255,232,0.55)` | mint glass at 55% |

The A4 sheet itself is Paper. Its letterhead colors come from the pinned `doc_json` (`theme` / `letterhead.colors`), not from this page's tokens; the spec must not recolor a doctor's document. Never `#000000` on the shell. No chromatic border: the canvas seam and card edges are `--color-hairline` only.

Measured shell contrast: Ink on Paper 17.28:1 · Mist on Paper 6.96:1 · Paper on Deep Teal 6.02:1 · Plum on Paper 15.32:1 · Plum on Blossom 10.36:1. AA for body; AA large for the 22px title. The provisional stamp uses Plum text on a paper patch with a hairline, not white-on-ember.

### 5.2 Typography

| Element | Family | Weight | Size / line-height |
|---|---|---|---|
| Page title (`h1`) | `--font-mint` | 500 | 22px / 1.22, Ink |
| Subtitle | `--font-mint` | 400 | 14px / 1.5, Mist |
| Primary sign/download label | `--font-mint` | 500 | 16px / 1.22 |
| Secondary action label | `--font-mint` | 500 | 16px / 1.22 |
| Provisional Notice | `--font-mint` | 400 | 14px / 1.5, Plum; title 500 |
| `ErrorText` / `HintText` | `--font-mint` | 400 | 14px / 1.5, Plum / Mist |
| Canvas accessible name | visually hidden | 400 | 14px |
| A4 document | embedded document typography | — | 210 × 297 mm projection |

No `--font-grenette` on the shell: this is an application surface, not a marketing opener. The document's own typography is the pinned template's; it is not page chrome and must not be restyled by this screen.

### 5.3 Geometry

- 4px base. `PageShell`: 48px top, 80px bottom, 24px inline; `.ui-container` max 1200px; `.ui-stack` 24px gap. The canvas width is the available container width, not a fixed 1200px.
- Page header: flex row, space-between, align start, wrap, 16px gap; subtitle 4px below the title; the Secondary back action sits at the far end with the shared button box, 12px × 24px padding, 8px radius.
- Action row: `ActionRow`, 12px gap, wraps, 16px top margin; Primary and Secondary share the 8px radius and 12px × 24px box; one Primary maximum. The acknowledgment control joins the row only after the conflict.
- Provisional Notice: 24px radius, 20px padding, 12px icon-to-body gap, 20px Alert glyph; full card width, not centered as a banner.
- Preview canvas: 24px padding, 24px radius, Cream `#fffded`; horizontally scrollable when the sheet is wider than the viewport; no nested card, no second shadow.
- A4 sheet: 210 mm × 297 mm, 8px radius, Paper, `--shadow-paper`, `aspect-ratio: 210 / 297`; margin from `preview.margin_mm` (5–40 mm clamp), repeated header/footer, page breaks from `forms::paginate_document`; pages stack with a 16px gap. The sheet is never scaled to fit: 1 mm stays 1 mm, so a 210 mm page is roughly 794 CSS px at 96 dpi.
- Status and number line: 14px Mist, separated by ` · `; the number uses the same 6-digit padding as the PDF (`{number:06}`).
- Elevation: flat shell; the single `--shadow-paper` belongs to the A4 sheet. No button, notice, or canvas shadow.
- Responsive: below 760px the header stacks its back action under the title; the action row wraps; the canvas keeps horizontal scroll and 24px padding; 320px never forces the sheet to shrink or crops its right edge without a scroll affordance.

### 5.4 Iconography and imagery

Line icons only, 1.5px stroke, 20px default, Ink or Plum: the Alert glyph in the provisional Notice and `ErrorText`, and the optional document/QR glyphs already defined by the shared primitives. Print and share are text labels, not icon-only controls. No photography, illustration, avatar, clinic logo as page chrome, patient image, signature, stamp, or ink. The document's own logo and QR are authorized payload images rendered inside the sheet, with their alt text from §3.5; the page adds no decorative copy of them.

## 6. Accessibility

- Landmarks: the shell navbar is the banner (`Header.md`); `PageShell` is the page content. The header is a labelled `header`, the action row a plain group, the canvas a labelled region (`aria-label` `Vista previa de la receta`), and the A4 pages are document content inside it. Do not mark each page as a separate landmark.
- Headings: exactly one `h1`, `Vista previa A4 — {patient_name}`, from `PageHeader`. There is no `h2` on this page; if a help/notice block is added, it needs a real heading level, not a styled paragraph. No skipped level.
- Tab order: back action → sign primary when present → acknowledgment secondary when present → print → local provisional → share. Download is the primary in the signed branch and sits in the sign slot. The canvas is focusable only if it needs keyboard scrolling (`tabindex="0"` with a visible name); the pages, images, and QR are not focusable. Long content does not create a second tab order.
- Focus: shared utility — `outline: 1px solid var(--color-deep-teal); outline-offset: 2px; box-shadow: 0 0 0 3px rgba(185,255,232,0.55);` — on every action substitutes and on the scrollable canvas when it is focusable. Visible against Paper and Cream.
- Names: each action is its visible label; no icon-only control. `aria-busy="true"` on the sign control while `signing`; the label becomes `Firmando...`. The download and print links announce their target file. The share button names the share sheet, not "share" alone in a way that hides the PDF target.
- Status: the provisional Notice is text, not a colour band; the stamp `PENDIENTE DE SINCRONIZACIÓN` is real text inside the sheet; status labels are printed. Colour never carries "provisional", "signed", or "error" alone.
- Errors: load/render/sign failures are `ErrorText` with `role="alert"`; they are inserted, not focus-moved. The header keeps the authorized patient name only after the payload resolved; a failed load renders no invented name.
- The A4 projection is visual. The page must expose the same essential values in accessible text: status label, number when present, issue date, verification code when present, provisional stamp when present, page count. The QR image's alternative is `Código QR de verificación`; its payload is not a substitute for the readable code.
- Download/print/share: download and print are native anchors; share is a native button. Keyboard activation works without a pointer. Do not trap focus in the canvas or a notice.
- Reduced motion: no page animation; the shared spinner is the only motion and stops under `prefers-reduced-motion: reduce`.
- Zoom/reflow: at 200% and 320px the header/actions reflow; the A4 sheet does not reflow into a fake responsive document, but the canvas scrolls in both axes with keyboard access and a visible focus ring. The page never clips the sheet without a way to scroll.

## 7. Implementation notes

The shipped component is `fn PrescriptionPreview(id: String) -> Element` at `persona-sanus/src/main.rs:4939`, `#[component]` at `:4938`, closing before `PrescriptionDetail` at `:5380`. Its effect gates on `checked()` and the session, fetches `api::fetch_prescription` and `api::fetch_prescription_preview`, and on desktop rebuilds a provisional payload from the local cache when the preview endpoint fails. The rendered branch starts at `:5175`.

Signals and branches to preserve:

- `preview`, `prescription`, `message`, `signing`, `needs_ack`, `generation`; the effect depends on `generation` and resets on each sign.
- `is_draft = current.status == "draft"`, `can_sign = is_draft && profile.can_sign_prescriptions()`. This screen does not check ownership; the sign endpoint does, and its refusal stays in `message`. The detail screen's stricter owner gate is not copied here.
- `patient_name` from the `patient.name` entry, fallback `Receta`; `values` from every `PreviewVariable`; `image_sources` from `PreviewImageSource.asset_id`/`data_url`; `DocumentMeta` from code, number, issued_at, provisional, QR.
- `pdf_url = /api/prescriptions/{current.id}/pdf`; `preview_id`, `pdf_doc`, `pdf_values`, `pdf_meds`, `pdf_rx`, `pdf_issued` are cloned before `rsx!` for the provisional render.
- Sign uses `sign_prescription_dispatch(&preview_id, false, offline)`; the acknowledgment branch uses `true`; success replaces `prescription`, refreshes sync with `probe_sync_status`, increments `generation`; error sets `message`, and only the `variables desconocidas` substring reveals the acknowledgment control.
- Provisional local render builds `pdf::PdfInput` with `RenderMode::Provisional`, margin 12, title `Receta-{preview_id}.pdf`, desktop `cached_doc_assets`, and reports `bytes.len()`; it writes no file. A real local download is a separate change.
- Print is an anchor to the same PDF URL with `target="_blank"` and `rel="noopener noreferrer"`. Share is a wasm-only `web_sys::window().navigator().share_with_data` with `ShareData.title = "Receta médica"` and `url = origin + pdf_url`.
- No PDF is generated on load; `pdf::render` runs only in the provisional explicit-action branch on desktop.

Plan step 15 adoption:

- Root becomes `PageShell id="prescriptions-page"`; the raw page head becomes `PageHeader` with the existing `h4` as `title` and the existing `p` as `subtitle`; the back `Link` becomes a Secondary `ButtonLink` or `LinkAction` `Volver a la receta`.
- The action region becomes `ActionRow`; sign is `Button` Primary, acknowledgment is Secondary, download/print/share are `LinkAction`, the provisional local render is Secondary `Button`. Preserve the mutually exclusive primary.
- The provisional `.notice` becomes `Notice tone=Warning`; `.error` becomes `ErrorText`; the byte report becomes `HintText`. The `.preview-scroll` wrapper becomes `.ui-preview-canvas`; `forms::A4Preview` keeps its props and geometry unchanged.
- Add no new endpoint, route, query parameter, load-time render, or share payload field.

Verification commands, from the repo root; none is recorded here as executed:

```sh
cd persona-sanus && cargo check
dx build --release --fullstack
caveman-blocks run persona-smoke --dir persona-sanus --port 8091
python3 .blocks/markdown-table-check.py --path context/screens/PrescriptionPreview.md --strict
```

`curl` is not used: the route requires an authenticated actor, and the preview endpoint is not public. A browser pass covers a draft as doctor, a signed row as secretary, a provisional desktop row offline, and a parse failure.

- Do not add: a second primary action; a download or print control on a draft without a verification code; a silent, automatic, or on-load PDF render, download, or write; a stored PDF, cache file, or endpoint; a client-side PDF generator or `printpdf` in wasm; a new route or query param.
- Do not add: a signature or stamp image, field, or draw control; rich text, `contenteditable`, or a formatting toolbar; an edit, reorder, void, or supersede control on this page; billing, price, trial, plan, or `sync_locked` copy or state.
- Do not add: a fake QR, fake verification code, or fake final number in the provisional state; PHI in the URL, share title, QR payload, logs, or alt text; a numeric A4 scale control or a fourth page size; an invented print or download success state.

## 8. Spec references

| Spec | Connection |
|---|---|
| `specs/15-O-ui-tokens-actions-and-formatting.md` | `15-O` supplies the tokens, `Button`, `LinkAction`, and format helpers the preview actions use.
| `specs/16-P-ui-forms-feedback-and-layout.md` | `16-P` supplies the action row, loading states, and notices the sign/download states render.
| `specs/18-R-ui-domain-and-billing.md` | `18-R` builds the prescription status badge and sync-status indicators the provisional offline preview shows (FR-043).
| `specs/23-W-ui-prescriptions.md` | `23-W` adopts `PrescriptionPreview`: exactly one Primary — `Firmar y emitir` for drafts, `Descargar PDF` otherwise.
| `specs/25-Y-ui-legacy-css-cleanup-and-docs.md` | `25-Y` removes legacy page-ID selectors so the adopted preview runs only on `.ui-*` classes.

