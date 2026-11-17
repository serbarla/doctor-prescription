# PersonaSanus — PrescriptionDetail (`/prescriptions/:id`)

| Field | Value |
|---|---|
| Source | `context/sitemap.md` §1.2 (forms and detail), §2 (roles), §3 flows 1, 2, 5, 6, §4 M1/M2, §5 (PHI, locale); `context/PRD.md` §3, §4 flows 1/2/5/6, FR-010–FR-014, FR-030–FR-034, FR-040–FR-043, FR-050, D-02/04/06/07/21; `context/Design.md` (buttons, inputs, surfaces, elevation, layout, do/don't); `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` step 15 and the `PrescriptionDetail` page-adoption row; `context/screens/INDEX.md` §2.3 |
| Component | `PrescriptionDetail` — `persona-sanus/src/main.rs:5379-6177` (`#[component]` at `:5379`, `fn` at `:5380`, final component); composes planned primitives in `persona-sanus/src/ui/` (directory not shipped): `PageShell`, `PageHeader`, `Card`, `SectionCard`, `FormGrid`, `DetailGrid`, `DetailItem`, `ActionRow`, `Button`, `ButtonLink`, `TextField`, `TextAreaField`, `EmptyState`, `ErrorText`, `HintText`, `Notice`, `ConfirmPanel`, `RxBlockIndicator`, `LlmExplainLink`, `PrescriptionStatusBadge` |
| Route | `/prescriptions/:id` — authenticated shell; opaque `id` only, no query params; signed out redirects to `/login` (`Route::AuthPage {}`). Read: Doctor (own), Nursing (draft), Secretary (read/reprint), Administrator (read-only). The signed transition supersedes into a new row and navigates to its `/prescriptions/:id`; A4 preview is `/prescriptions/:id/preview`; back is `/prescriptions` |
| Layout | Inside `#[layout(Navbar)]` (`main.rs:51`; never closed, so `:92` wears the shell bar — see `Header.md`). `PageShell id="prescriptions-page"` renders `div.ui-page > div.ui-container.ui-stack`; Paper with hairline separators throughout. No band or full-bleed exception; Cream only in the RX strip, empty state, and void panel |
| Locale | `es-EC` on every string; `receta`, `cédula`; dates `DD-MM-AAAA`; 24-hour clock |
| Status | ✅ Spec complete (sections 1–7). Route status: 🟡 — sitemap §1.2: “Forms and inputs only over the A4 template (plain-text inputs/textareas; no rich text — D-21), online and offline. `draft`: editable. `signed`: locked, read-only with *Download PDF*, *Print*, *Share*, *Void* (reason required), *Supersede* (new version referencing previous). Per-medication *Explain with LLM* (FR-012, external — §3 below).” Shipped: auth-gated detail, block strip, draft and item forms, sign/acknowledge, supersede/void, read grid, external LLM link; desktop cache and mutation fallbacks. Target: step 15 primitive adoption and an `allocating` signal. PDF, print, and share stay on the preview route |

## 1. Purpose

One prescription, identified by an opaque id: inspect it, edit a draft, or read a signed document without changing it. Jobs, in order:

1. Name the patient, state, number, date, and code state from the loaded prescription.
2. Edit a draft’s diagnosis, validity, general instructions, and free-text medication rows.
3. Persist those edits online, or through the desktop cache and outbox when the server is unreachable.
4. Make the number block explicit: show the remaining count, reserve a block when none is active, and stop an offline signature when the block is exhausted.
5. Read a signed row as immutable, then expose only its two legal mutations: a supersede that creates a new row, number, and verification code, or a void that requires a reason and leaves an audit trail.

It never renders:

- PHI in the URL, query, analytics, gallery, or an external request. The path carries only the opaque prescription id; the LLM tab gets the medication name only, after an explicit click (FR-012, sitemap §5).
- Billing, price, trial, plan, or `sync_locked` copy; v1 is a free beta (D-04, PRD §5.7).
- A signature, stamp, upload control, or signature canvas; the doctor signs the printed paper (D-06).
- A rich-text editor, formatting toolbar, `contenteditable`, WYSIWYG surface, or document HTML (D-21).
- An edit affordance on a signed, superseded, or voided row; a nursing sign; a secretary or Administrator edit; or a client-only permission decision (sitemap §1.2–§3).
- A second database copy, an invented `superseded_by` field, an audit endpoint, PDF/print/share, or billing control.

## 2. Structure

| Order | Region | `id` / class | Band tone | Contents |
|---|---|---|---|---|
| — | Navbar | `header.ui-navbar` | Paper `#ffffff` | Shared shell band, owned by `context/screens/Header.md`; this page adds nothing |
| 1 | Header | `#prescriptions-page` · `div.ui-page > div.ui-container.ui-stack`; `header.ui-page-head` | Paper | `PageHeader`: only `h1` is the authorized patient name; subtitle is state, number, date; actions are Secondary `Vista previa A4` and Secondary `Volver` |
| 2 | RX block | `.ui-rx-block` | Paper; Cream strip `#fffded` | `RxBlockIndicator`: count, no-block, exhausted warning, Secondary `Reservar bloque` |
| 3 | Page message | `.ui-notice--error` / `.ui-error` | Paper; Blossom `#ffc3e6` | Load, save, item, sign, void, and supersede errors; absent when empty |
| 4 | Body | `.ui-card` | Paper | Draft: `Card` + `FormGrid` and save/sign `ActionRow`. Read: `Card` + `DetailGrid`/`DetailItem` |
| 5 | Medicamentos | `.ui-card`; `div.ui-item-card` | Paper; Cream empty state | `SectionCard` with `h2`; empty `EmptyState`; one item card + `FormGrid` + `ActionRow` per medication; Secondary `Agregar medicamento` |
| 6 | Signed transition | `.ui-actions` | Paper | Signed row owned by the signed-in doctor: Secondary `Reemplazar (corrección)`, Danger `Anular` |
| 7 | Void confirmation | `.ui-confirm` | Paper; Cream panel | `ConfirmPanel`: `TextField` `Motivo de anulación`, Danger `Confirmar anulación`, Secondary `Cancelar` |

Regions 3–7 are conditional. Region 2 renders only when the block signal holds a value; region 7 only while the void panel is open. The navbar is not part of this page and its copy is not repeated.

### 2.1 Session and role states

| Session / role | `draft` | `signed` | `superseded` / `voided` |
|---|---|---|---|
| No session or no profile | `Redirigiendo...`, then `/login`; no clinical content | same | same |
| Doctor, owner | Fields and items editable; Secondary `Guardar borrador`; Primary `Firmar receta`; no transition | Read-only `DetailGrid`; Secondary `Reemplazar (corrección)` and Danger `Anular` | Read-only; no transition |
| Doctor, non-owner | Fields and items editable; Secondary `Guardar borrador`; no sign | Read-only; preview only; no transition | Read-only; preview only |
| Nursing | Fields and items editable; `Guardar borrador` is the only Primary; no sign | Read-only; preview only; cannot sign | Read-only; preview only |
| Secretary | Read-only; A4 preview for reprint; no save/sign/transition | same | same |
| Administrator | Read-only clinic history; no save/sign/transition | same | same |

Role is `clinic_members.role` from the authenticated server session (PRD D-03). The page reads the `Me` context and never decides authorization itself: a capability gate can only omit a control, and the server rejects an out-of-scope request.

### 2.2 Prescription status states

| Status | What changes | What does not |
|---|---|---|
| `draft` | Field form and item editors appear with actions; no verification fields or transition | Header identity, preview/back links, section title |
| `signed` | Body becomes static `DetailGrid`; edit actions disappear; owner doctor gains replace and void; verification code and code state appear when present | Header identity unaffected; preview/back stay |
| `superseded` | Badge `Reemplazada`; static; no replace or void; the old number remains in the subtitle | Header identity and preview link |
| `voided` | Badge `Anulada`; static; the void reason is appended; no replace or void | Header identity and preview link |
| `pending_sync` / `failed` | Not prescription status values here: an offline-signed desktop row keeps `status = signed`; its outbox state lives only in the shell indicator and the history filter | The detail header never rewrites `Firmada` to a sync label |

### 2.3 Web and desktop surface

| Surface | What changes | What does not |
|---|---|---|
| Web | A load error is final; no local cache. Item writes, reorder, and delete require the server | Copy, role gates, status vocabulary, one-primary rule |
| Desktop, online | Server path; a successful block allocation is also cached (`cache_rx_block`, `set_meta("active_rx_block")`) | Controls and state meaning |
| Desktop, offline load | `fetch_prescription` failure reads the matching local row and `local_prescription_items`; no hit keeps the error surface | Route id, header composition, no-PHI rules |
| Desktop, offline draft save | `update_prescription` failure uses `update_prescription_offline`; cached row updated, outbox queued | State stays `draft`; no server-copy claim |
| Desktop, offline item save | `upsert_prescription_item` failure uses its offline branch; cached row re-read locally | Item id/position are the cache’s |
| Desktop, offline reorder/delete | No local fallback and no outbox operation: the API error is shown and the local order/row stays | Controls remain visible; failure is never silent |
| Desktop, offline sign/void/supersede | The corresponding dispatch local branch consumes a block or queues the outbox; signed PDF stays provisional | Role gates and confirmation copy are unchanged |
| No session | `Redirigiendo...` until `/login` replaces the route; no fetch and no patient content | No alternate local-only screen |

### 2.4 Loading, empty, error, denied

| State | What changes | What does not |
|---|---|---|
| Session check or fetch in flight | `Cargando...` under `#prescriptions-page` | No title guess; PageShell padding |
| No medication rows | `EmptyState` `Sin medicamentos.`; `Agregar medicamento` still renders in edit mode | `Medicamentos` heading |
| Load failed, no cached row | `ErrorText` with the returned message and `Volver a recetas` | No invented patient name or status; no empty form |
| Denied | `Notice` Error `Acceso denegado`, `No tienes permiso para ver esta receta.`, and `Volver a recetas`; no field, code, or reason | The route keeps its opaque id; no public redirect |
| Mutation failed | `ErrorText` with the message; last loaded values stay | Heading and medication list stay visible |
| Save/sign/allocate busy | Acting control disabled; save/sign swap labels; allocation sets `aria-busy` | Other fields stay; no modal overlay |

## 3. Copy text

Chrome copy is by reference to `Header.md` §3 and `Footer.md` §3. Every string below is page-owned and appears once.

### 3.1 Page header

| Element | Copy | Target |
|---|---|---|
| Title (`h1`) | authorized `patient.full_name` | none |
| Subtitle | `{estado} · No. {número} · {fecha}`; omit `No.` and separator when `rx_number` is absent | none |
| Preview action | `Vista previa A4` | `/prescriptions/{id}/preview` |
| Back action | `Volver` | `/prescriptions` |

`estado` is one §3.6 label; `fecha` is `DD-MM-AAAA`.

### 3.2 RX number block

| Element | Copy |
|---|---|
| Active block | `Números disponibles: {remaining} de {limit}` |
| No active block | `Sin bloque de números activo.` |
| Exhausted warning | `Bloque agotado. Reserva otro antes de firmar.` |
| Allocate action / hidden group label | `Reservar bloque` / `Bloque de números` |

The allocate control keeps its label while `allocating`; it is disabled and exposes `aria-busy="true"` instead of a busy string.

### 3.3 Draft fields and actions

| Element | Copy | Tone |
|---|---|---|
| Diagnosis label | `Diagnóstico` | field label |
| Validity label | `Vigencia` | field label |
| General instructions label | `Indicaciones generales` | field label |
| Save / busy | `Guardar borrador` / `Guardando...` | Secondary for a doctor owner; Primary for nursing |
| Sign / busy | `Firmar receta` / `Firmando...` | Primary, owner doctor on a signable draft |
| Warning acknowledgement | `Firmar reconociendo las advertencias` | Secondary, only after the variables warning |
| Exhausted hint | `Sin números disponibles: reserva un bloque para firmar.` | Hint below sign |

A doctor-owner draft has exactly one filled Primary: sign; save is outlined. A nursing draft has one filled Primary: save; there is no sign control.

### 3.4 Medicamentos

| Element | Copy |
|---|---|
| Section heading (`h2`) | `Medicamentos` |
| Empty state | `Sin medicamentos.` |
| Field labels | `Medicamento` · `Dosis` · `Vía` · `Frecuencia` · `Duración` · `Cantidad` · `Notas` |
| Item and add actions | `Guardar medicamento` · `Subir` · `Bajar` · `Eliminar` · `Agregar medicamento` |
| External link and disclaimer | `Explicar con LLM` · `Se abre un sitio externo con el nombre del medicamento. No se envían datos del paciente.` |

Item save, movement, and add are Secondary; delete is Danger; the explainer is a Deep Teal link to a new tab. Every field starts empty: no placeholder and no example clinical data.

### 3.5 Signed transition and void confirmation

| Element | Copy | Tone |
|---|---|---|
| Correction | `Reemplazar (corrección)` | Secondary |
| Start void | `Anular` | Danger |
| Reason label | `Motivo de anulación` | required field label |
| Confirm void | `Confirmar anulación` | Danger |
| Dismiss | `Cancelar` | Secondary |

The reason is required; confirm cannot be activated while the trimmed value is empty. No placeholder is added.

### 3.6 State badges

| Element | Copy |
|---|---|
| Prescription state | `Borrador` · `Firmada` · `Reemplazada` · `Anulada` |
| Desktop outbox state | `Provisional` · `Error de sincronización` |
| Unexpected value | `Desconocido` |

All are text, not colour-only. The outbox labels belong to the history surface; this route’s payload yields one of the four prescription states or the unknown fallback.

### 3.7 Verification code in read mode

| Element | Copy |
|---|---|
| Labels | `Código de verificación` · `Estado del código` |
| Values | `Válido` · `Anulado` · `Reemplazado` · `Desconocido` |

The value is a display translation of `verification.status`; the API token is never UI text.

### 3.8 Loading, denied, and fallback errors

| Context | Copy |
|---|---|
| Loading and redirect | `Cargando...` · `Redirigiendo...` |
| Denied | `Acceso denegado` · `No tienes permiso para ver esta receta.` · `Volver a recetas` |
| Unreadable payloads | `No se pudo leer la receta.` · `No se pudo leer el medicamento.` · `No se pudo leer el orden de medicamentos.` · `No se pudo leer la receta firmada.` · `No se pudo leer la corrección.` · `No se pudo leer el bloque de números.` |
| Offline and missing records | `Sin conexión.` · `El perfil no tiene ficha de médico.` · `Receta no encontrada en la caché local.` |
| Signing limits | `Bloque de números agotado. Quedan {n} números.` · `Escribe el motivo de anulación.` |
| Invalid transitions | `Solo un borrador puede firmarse.` · `Solo una receta firmada puede anularse.` · `Solo una receta firmada puede reemplazarse.` |

An API message is shown verbatim when non-empty; these are local fallbacks when a response cannot be read. Endpoint and symbol names are §4/§7 material, never copy.

## 4. Displayed information

### 4.1 What the screen shows

| Element | Source | Value shown |
|---|---|---|
| Patient title | `api::fetch_prescription(id)` → `api::Prescription.patient_name` | Authorized clinical content; no separate patient fetch |
| State and number | `Prescription.status` via `prescription_status_label` (`main.rs:4329`); `Prescription.rx_number` | §3.6 label (`Desconocido` for the helper’s old `Desconocida`); `No. {n}` after signing, omitted otherwise |
| Date | `Prescription.created_at` via `format_date` (`main.rs:4321`) | `DD-MM-AAAA` |
| Draft fields | `draft_diagnosis`, `draft_validity`, `draft_instructions` signals, seeded from `Prescription.diagnosis`, `.validity`, `.instructions` | Plain editable text; read mode is value or `—` |
| Medication rows | `Signal<Vec<api::PrescriptionItemForm>>`, seeded from `Prescription.items` and `desktop_local::local_prescription_items` | `med_name`, `dose`, `route`, `frequency`, `duration`, `quantity`, `notes`; entered values only |
| Verification | `Prescription.verification.code`, `.status` | Printed code when present; §3.7 display state |
| Void | `Prescription.void_reason` | Authorized clinical content on a voided row |
| RX block and allocation | `api::fetch_rx_block_status()` → `RxBlockStatus.active`, `.remaining`, `.limit`, `.exhausted`; `api::allocate_rx_block()` → `RxBlockInfo` | §3.2 text; desktop caches the new block, then status is re-fetched; no local count invented |
| Rights | `Me::can_create_prescriptions()`, `Me::can_sign_prescriptions()`, `Prescription.doctor_id`, `Me.doctor.id`, `Prescription.status` | Edit, sign, and transition gates from §2.1 |
| LLM URL | `llm_explain_url(Me.clinic.llm_url_template, item.med_name)` | Configured template or default chat query; medication name only |
| Sync | `SyncStatus` context refreshed by `probe_sync_status()` | Shell indicator and pending/failed links; not a page field |

The draft is not auto-saved. The save action writes the three text fields; each medication row has its own save action. Signing signs the persisted draft, so unsaved text or a new unsaved item is not part of the signature.

### 4.2 Hard rules

- The server is authoritative. Gates only omit controls; every read and mutation is authorized server-side by clinic and role. No client gate is treated as permission.
- Signed is immutable. `signed`, `superseded`, and `voided` render no inputs, save, or sign. Correction is supersede only: a new row with its own `rx_number` from a block and its own verification code; the old row keeps its number and becomes `superseded` (FR-013, D-16).
- No signature or stamp image, upload, or signature pad exists here; the doctor signs printed paper (D-06).
- A void requires a non-empty trimmed reason. The reason travels in the request body, never the URL; the server records the audit trail. This page calls no audit endpoint and invents no `superseded_by`.
- Nursing cannot sign or publish; Secretary cannot edit; the Administrator never signs or edits clinical content and reads history read-only (sitemap §2–§3; PRD §3, D-18).
- A non-owner doctor can edit a draft in the shipped role gate (`can_edit = draft && can_create_prescriptions()`) and the shipped service allows the same; only signing and signed transitions require `doctor_id` to match. This is broader than the sitemap's “Doctor (own)” phrasing and is recorded as a shipped divergence in §7. A doctor never sees sign/transition on a draft that is not theirs.
- PHI stays out of URLs: only the opaque id in the path. The preview target reuses that id; the LLM tab gets `med_name` only, only on an explicit click, and the app never fetches an LLM itself (FR-012).
- Desktop cache does not hide loss. A cache hit replaces only a failed load; mutation errors remain `ErrorText`; pending/failed counts stay in the shell; an offline-signed PDF is provisional with no active QR or final number until sync (FR-040–FR-043).
- No billing/price/trial/plan/`sync_locked` copy (D-04); no rich text or `contenteditable` (D-21); no new endpoint, table, column, role, permission, or field.
- One primary action at most: sign on a doctor-owner draft, or save on a nursing draft. Every other action is Secondary or Danger; read-only states have none.

### 4.3 Dynamic states

| State | Screen behaviour |
|---|---|
| Session check / signed out / no role | `Cargando...` until `checked`; then `Redirigiendo...` and `/login` for no user or no profile; no clinical paint |
| Load in flight / error / cache hit | `Cargando...`; error → `ErrorText` + `Volver a recetas`; desktop error → matching cached row and items replace the error, while the shell still shows offline or pending |
| Draft, editable | Edit card, item rows, Secondary add; save labelled; the owner doctor also gets Primary sign, a non-owner doctor gets no sign and no transition row |
| Draft, nursing | Same edit surface; save is the only Primary; no sign and no transition row |
| Signed | Static `DetailGrid`, code and state when present; owner doctor gets replace and void; all others get neither |
| Superseded / voided | Static read; `Reemplazada` / `Anulada`; no transition; voided also shows the reason |
| Denied | `Acceso denegado`, `No tienes permiso para ver esta receta.`, `Volver a recetas`; no title, code, or reason |
| Save in flight / success / failure | Disabled with `Guardando...` and `aria-busy`; success replaces the row, keeps `draft`, clears error; failure shows `ErrorText` and preserves typed values |
| Save offline | Desktop: local row updates, outbox queues, `probe_sync_status()` refreshes shell; page still says `Borrador`, no server claim. Web: API error, nothing cached |
| Item save / add | Success reloads row and items; failure shows message with edited row intact; desktop-offline caches and queues; web-offline does not create; add appends one blank row and persists nothing until save |
| Reorder / delete | Sends saved item ids or item id, reloads on success; first/last disabled; offline or server error is shown and order/row unchanged; delete of an unsaved row is a no-op |
| Empty medications | §3.4 empty state; add only in edit mode |
| Sign in flight / success | Disabled with `Firmando...`; success becomes `Firmada`, gains number/code state, drops editing, adds owner transition |
| Sign with unknown variables | Error plus Secondary `Firmar reconociendo las advertencias`; retry sends `acknowledgeWarnings = true` only |
| Sign offline | Desktop consumes the local block, becomes signed, queues outbox, shows no code, preview is provisional with placeholder QR, shell shows pending. Web shows `Sin conexión.` and stays a draft |
| Sign with inactive/exhausted block | Strip warns and offers allocation; offline attempt reports the exhausted message; no number is borrowed silently |
| Allocation | `allocating` disables the control and sets `aria-busy`; success re-fetches the strip and caches on desktop; failure shows message, keeps last status, invents no count |
| Void panel | Required reason; destructive confirm unavailable until trimmed value is non-empty; focus is not trapped. Cancel closes without submitting; success yields `Anulada`, shows reason, hides transition, code becomes `Anulado`; failure keeps the panel |
| Supersede | Success navigates to the new row’s id with its own `Firmada`, number, and code; old row becomes `Reemplazada` on reload. Failure leaves the signed row unchanged |
| Pending / failed outbox | Not values returned by `api::fetch_prescription`; no badge is fabricated. Shell and `/prescriptions?status=pending_sync|failed` carry them; a cached signed row may still be provisional in the preview PDF |
| LLM link | Only when `llm_explain_url` returns a URL for a non-empty name; activation opens a new tab with that name; no patient field, diagnosis, instruction, code, or id; rendering sends nothing |
| Read-only / navigation | No inputs, save, sign, or transition; preview/back remain. Save does not navigate; signing re-renders same row read-only; supersede is the only navigation, to the new row; back `/prescriptions`; preview `/prescriptions/:id/preview` |

## 5. Style

Every value is a `context/Design.md` token; no local palette or new literal colour.

### 5.1 Color

| Use | Value | Token |
|---|---|---|
| Canvas and cards | `#ffffff` | `--color-paper` |
| Titles, headings, values, input text | `#061d29` | `--color-ink` |
| Subtitle, labels, meta, hints, disclaimer | `#425d6d` | `--color-mist` |
| Borders, dividers, input borders | `#e5e7eb` | `--color-hairline` |
| Primary fill / label; hover | `#006e75` / `#ffffff`; `#0b978e` | `--color-deep-teal`, `--color-paper`, `--color-bright-teal` |
| Secondary action | transparent, `#425d6d` border/text | `--color-mist` |
| Danger action | transparent, `#4d0037` border/text | `--color-plum`; hover wash `--color-blossom` |
| RX strip, empty state, confirm panel | `#fffded` with hairline | `--color-cream`, `--color-hairline` |
| Error surface | `#ffc3e6` with `#4d0037` | `--color-blossom`, `--color-plum` |
| Focus ring / halo | `#006e75` / `rgba(185,255,232,0.55)` | `--color-deep-teal`; `--color-mint-glass` |

Measured WCAG AA: Ink/Paper 17.28:1, Ink/Cream 16.89:1, Ink/Mint 15.25:1, Ink/Blossom 11.68:1, Mist/Paper 6.96:1, Mist/Cream 6.80:1, Paper/Deep Teal 6.02:1, Paper/Plum 15.32:1, Plum/Blossom 10.36:1. Never `#000000`. No chromatic card/image border, no gradient, no shadow beyond the focus halo.

### 5.2 Typography

| Element | Family | Weight | Size / line-height |
|---|---|---|---|
| Page title (`h1`) | `--font-mint` | 500 | 22px / 1.22 Ink |
| Subtitle | `--font-mint` | 400 | 14px / 1.5 Mist |
| Section heading (`h2`) | `--font-mint` | 500 | 20px / 1.25 Ink |
| Field label / detail label | `--font-mint` | 500 | 14px / 1.5 Ink / Mist |
| Input, textarea, detail value | `--font-mint` | 400 | 16px / 1.5 Ink |
| Button | `--font-mint` | 500 | 16px / 1.5 |
| Badge, pill, item meta, hint, error | `--font-mint` | 500 / 400 | 13–14px / 1.5 |
| Read medication name | `--font-mint` | 500 | 18px / 1.25 Ink |
| Empty title / description / loading | `--font-mint` | 500 / 400 / 400 | 18px / 1.25 Ink; 14px / 1.5 Mist |

`--font-grenette` is display-only at 46px+ and never appears here: no app chrome, page title, section title, or button. Type tops out at 22px.

### 5.3 Geometry

- Base 4px. PageShell: 48px top, 80px bottom, 24px inline; container max 1200px; stack gap 24px.
- PageHeader: title/subtitle 4px apart; actions right, 12px gap, wrapping; below 640px actions move below the title block. No extra header rule.
- Cards: Paper, 1px hairline, 24px radius, 24px padding, no shadow, 16px vertical gap. Item cards: 24px radius, `16px 20px` padding, 12px bottom margin.
- FormGrid: two columns, 16px gap; one column below 760px; `wide` spans both; textareas 3 rows; labels 8px above controls. ActionRow: 12px gap, wrap, 16px top margin.
- RX strip: Cream, 1px hairline, 8px radius, `12px 16px` padding, 8px between status/warning/action; between header and body.
- Buttons: 8px radius, `12px 24px` padding, 8px icon gap; Secondary and Danger 1px border, no fill. Inputs/selects: 8px radius, `8px 12px`; textareas 12px.
- EmptyState: Cream, 24px radius, 1px hairline, 32px padding, centered, 8px title/description gap, description max 48ch. ConfirmPanel: Cream, 24px radius, hairline, 20px padding, 12px gaps. DetailGrid: 2 columns, 24px/16px gaps, `wide` spans both, one column below 760px.
- No nested cards, no horizontal scroll, no elevation. Spinner only motion; `prefers-reduced-motion: reduce` disables it; no transitions.

### 5.4 Iconography and imagery

Line icons only, 1.5px stroke, 24-unit view box, Ink or Deep Teal (Mist for the external hint). `Alert` may mark an error, `Info` a warning, `Qr` stays exclusive to verification. No photographs, patient images, illustrations, signature, stamp, or A4 sheet; the only document visual lies behind the preview link. Icons never carry state alone.

## 6. Accessibility

- Landmarks: the shell `header.ui-navbar` remains the banner; the page is one content region via `PageShell`. `PageHeader` owns the single `h1` (authorized patient name); `SectionCard` owns the only `h2`, `Medicamentos`. Heading order is `h1` → `h2` with no skipped level, styled span promoted, or second page title.
- DOM tab order: preview → back → allocation when enabled → diagnosis → validity → instructions → save → sign when present → each item’s fields in position order → item save → up (skipped when disabled) → down (skipped when disabled) → delete → add → replace → void → reason → confirm void → cancel. Read mode exposes only links and buttons. Reordering is keyboard-operable via buttons, never drag-only.
- Labels: each input and textarea has a programmatic wrapping label; §3.3–§3.5 names are the accessible names. Per-item actions use `aria-describedby` pointing at that row’s `Medicamento` field so repeated up/down/delete controls are not anonymous.
- Focus: `1px solid var(--color-deep-teal)` at 2px offset, plus `box-shadow: 0 0 0 3px rgba(185,255,232,0.55)`; never removed, visible on all native buttons, links, inputs, and textareas.
- Hints and errors: the LLM disclaimer is the link’s `aria-describedby`; errors are wired the same way and set `aria-invalid="true"`. The void reason is `required`/`aria-required="true"`; its empty rule disables confirm and is explained before submission, never a silent dead control.
- Live regions: page errors use `role="alert"`; `Guardando...`, `Firmando...`, allocation, and block replacement sit in a polite `aria-live`/`role="status"` region; the acting control sets `aria-busy`. Loading uses `role="status"` with `aria-live="polite"`.
- Status is never colour-only: badge text names the state; warnings and errors pair text with an icon. No table exists here, so no caption/`th scope` is introduced.
- Contrast: AA or better for every pair used — Ink/Paper 17.28, Ink/Cream 16.89, Ink/Mint 15.25, Ink/Blossom 11.68, Mist/Paper 6.96, Mist/Cream 6.80, Paper/Deep Teal 6.02, Paper/Plum 15.32, Plum/Blossom 10.36. Focus is a solid Deep Teal ring, not a colour-only change.
- `ConfirmPanel` is inline: no focus trap, no modal overlay, no autofocus steal; Danger actions are labelled. External navigation is explicit in the visible disclaimer. Reduced motion disables the spinner. Action gaps are 12px, field padding 8px 12px, and nothing depends on pointer precision or drag.

## 7. Implementation notes

- Current code: `PrescriptionDetail` at `persona-sanus/src/main.rs:5380-6177`, the final component. Signals: `prescription`, `items: Vec<api::PrescriptionItemForm>`, `block: Option<api::RxBlockStatus>`, `message`, `draft_diagnosis`, `draft_validity`, `draft_instructions`, `initialized`, `saving`, `signing`, `needs_ack`, `void_reason`, `show_void`, `prescription_id` (fixed from `id`).
- Load: `use_effect` waits for `checked`, redirects when `user` is absent, calls `api::fetch_prescription` (`api.rs:230-233`; `GET /api/prescriptions/{id}`), seeds draft signals and `Prescription.items`, then tries `desktop_local::open_local_db`, `list_prescriptions`, `local_prescription_items` on API failure. A hit replaces the error; a miss leaves it. `api::fetch_rx_block_status` (`api.rs:475-479`; `GET /api/rx/block-status`) follows; its error is ignored and the strip stays hidden.
- Gates, unchanged: `can_edit = is_draft && profile.can_create_prescriptions()` at `main.rs:5548`; `can_sign` at `:5549-5556`; `can_transition` at `:5557-5565`. Helpers are `api.rs:72-78`; the server remains authoritative.
- Preserved dispatch: `api::update_prescription` (`api.rs:293-309`), `upsert_prescription_item` (`:324-333`), `delete_prescription_item` (`:336-341`), `reorder_prescription_items` (`:344-355`), `sign_prescription_dispatch` (`main.rs:905`), `void_prescription_dispatch` (`:1123`), `supersede_prescription_dispatch` (`:1202`); desktop `update_prescription_offline` (`:792`), `upsert_prescription_item_offline` (`:860`), `sign_prescription_offline` (`:739`), `void_prescription_offline` (`:1086`), `supersede_prescription_offline` (before `:1202`); reload `reload_prescription` (`:4838`). Reorder/delete have no offline branch and add none.
- Current markup is one-off: `.page-head`, `.block-indicator`, `.form-grid`, `.field`, `.actions`, `.detail-grid`, `.notes`, `.medication-list`, `.item-card`, `.item-read`, `.llm-link`, `.warning`, `.hint`, `.error`, `.notice`. `prescription_status_label` (`:4329-4337`) supplies status text; `:5747` prints raw `verification.status`; both are replaced by adopted primitives and §3.6/§3.7. `llm_explain_url` (`:4339-4353`) and its `{med}`/`{medication}` substitution stay unchanged.
- Step 15 deltas: wrap in `PageShell id="prescriptions-page"`; replace `.page-head` with `PageHeader title=current.patient_name`, the §3.1 subtitle, and Secondary `ButtonLink`s for preview/back. Replace `.block-indicator` with `RxBlockIndicator status=block() allocating=allocating() on_allocate=…`; add `allocating` around the existing `api::allocate_rx_block()` call, set before request, clear in both outcomes, keep the desktop cache, and re-fetch only via `api::fetch_rx_block_status()`.
- Draft deltas: `Card` + `FormGrid` + `TextAreaField`/`TextField`; `ActionRow` order sign Primary plus save Secondary when `can_sign`, save Primary for nursing, otherwise save Secondary only. Keep save bodies/desktop fallback and sign dispatch arguments; preserve the warning path with Secondary `Firmar reconociendo las advertencias`.
- Medication deltas: `SectionCard` `Medicamentos`, empty `EmptyState`, `div.ui-item-card` + `FormGrid` + `ActionRow` per item; Secondary save/up/down, Danger delete, disabled boundaries, Secondary add. Keep upsert/reorder/delete ids and ordering, and the absence of offline reorder/delete. Render `LlmExplainLink` only when `llm_explain_url` returns `Some`.
- Read/void deltas: `Card` + `DetailGrid`/`DetailItem` with code, §3.7 state, and stored reason; `ActionRow` Secondary replace and Danger void; inline `ConfirmPanel` with `TextField` bound to `void_reason`; keep trimmed reason, `void_prescription_dispatch`, and server-side audit. Add no supersede/audit endpoint. English token mapping at `:5747`: `valid` → `Válido`, `voided` → `Anulado`, `superseded` → `Reemplazado`; status enum maps to `Borrador`, `Firmada`, `Reemplazada`, `Anulada`, and the old `Desconocida` fallback becomes `Desconocido`.
- Desktop/web branches stay: cache read/item load, update/item-upsert fallbacks, sign/void/supersede dispatch, block cache, and the below-760px/below-640px layouts. This component contains no PDF/print/share/QR/A4 call; those belong to `/prescriptions/:id/preview`.
- Verify, from the repo root:
  - `python3 .blocks/markdown-table-check.py --path context/screens/PrescriptionDetail.md --strict`
  - `cd persona-sanus && cargo check`
  - `cd persona-sanus && cargo check --no-default-features --features desktop`
  - `cd persona-sanus && dx build --release --fullstack`
  - `caveman-blocks run persona-smoke --dir persona-sanus --port 8091` (expects `"ok": true`; shell/auth paths only)
- Do not add: a new route or endpoint; billing, price, trial, plan, or `sync_locked` copy; a signature or stamp image, upload, file field, or signature canvas; a rich-text, WYSIWYG, `contenteditable` surface or formatting toolbar; a `superseded_by` field, audit fetch, or audit page; an in-place edit of a signed row; a second filled teal action; drag-only reordering; PDF, print, share, QR, or A4 rendering here; a patient name, cédula, diagnosis, medication, or verification code in the URL; automatic LLM navigation or any PHI in the external link; English UI copy; a modal focus trap in the void confirmation; an invented success animation, badge, or count.
