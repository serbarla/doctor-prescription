# Spec 23 — Phase W: Prescription adoption

- Source: `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` step 15 and the Page adoption table; `context/sitemap.md` §§1.1–1.3; `context/PRD.md` FR-010–014, FR-022, FR-030–034, FR-040–043, D-06, D-21; `context/architecture.md` §3; `context/database-design.md` §4.
- Depends on: 15-O, 16-P, 17-Q, 18-R, 19-S, 20-T, 22-V.
- Repo root `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Result carried forward: `Prescriptions`, `PrescriptionNew`, `PrescriptionPreview`, and `PrescriptionDetail` render through the UI layer; every query/filter, desktop fallback, outbox update, signing/voiding/superseding dispatch, reorder/upsert/delete call, acknowledge-warnings branch, and role gate is preserved; each page has at most one filled primary action, and the detail page exposes an `allocating` state on its RX block request.

## Before you start

Unit testing is not required; avoid it. Prioritize end-to-end and integration testing only.

Read the nearest `AGENTS.md` (target workspace, then parents up to repo root) before executing.

If reality diverges from this spec: STOP, append a divergence (section 4), and NEVER improvise outside the objectives.

This is markup adoption plus one UI-only signal (`allocating`). Do not change the signatures, return values, API arguments, local fallbacks, or synchronization effects of the existing dispatch/load/reorder functions.

## 1. Objectives

*The ordered work to implement — one item per step. Check `[x]` only after the step is implemented and verified.*

*Item format:* `<verb phrase>` — requirements: `<what must change / constraints>`; commands: `<exact command>`.

- [ ] Adopt the Prescriptions list — requirements: `PageShell id="prescriptions-page"`, `PageHeader title="Recetas" subtitle="Historial con filtros por paciente, fecha y estado."`, and a Primary `ButtonLink` `Nueva receta` -> `Route::PrescriptionNew { patient_id: None, template_id: None }` only when `profile.can_create_prescriptions()`. Replace the filter row with `FilterBar` containing `SearchField label="Paciente"` bound to `query`, `SelectField label="Estado"` with the existing option values and labels, and `TextField label="Fecha" input_type="date"` bound to `date`. Add a `loaded` signal set true after the fetch resolves; render `LoadingState` while `!loaded()`, `ErrorText` for `message`, `EmptyState title="Sin recetas" description="No hay recetas para estos filtros."` when the visible list is empty, and a `ListRow` per prescription: title `patient_name`, meta status/date/`No. {n}` when present, badge `PrescriptionStatusBadge { status }`, Secondary `Abrir` -> `Route::PrescriptionDetail { id }`. When the row status is `pending_sync`, add a Cream `Badge` `Provisional · PENDIENTE DE SINCRONIZACIÓN`. When `failed_rows` is non-empty, render `SectionCard title="Errores de sincronización"` with a `ListRow` per pair: title `operation`, meta `[detail]`. Keep `failed_rows` and the desktop branch. Only Primary: `Nueva receta`; commands: `cd persona-sanus && cargo check`; `rg -n 'failed_rows|pending_sync|Provisional · PENDIENTE DE SINCRONIZACIÓN' persona-sanus/src/main.rs`.
- [ ] Adopt PrescriptionNew — requirements: `PageShell id="prescriptions-page"`, `PageHeader title="Nueva receta" subtitle="Paciente y plantilla publicada."`, header Secondary `Cancelar` -> `Route::Prescriptions { status: None }`. Put the form in `Card` > `FormGrid columns: Two`: `SelectField label="Paciente"` with the existing patient options and placeholder `Selecciona un paciente`; `SelectField label="Plantilla"` with the existing published-template options and placeholder `Selecciona una plantilla`; `TextAreaField label="Diagnóstico"`; `TextField label="Vigencia"`; `TextAreaField label="Indicaciones generales" wide`. Keep every signal, the published filter (`current_version_id` present and `current_version.status == "published"`), `ErrorText`, API/desktop dispatch, and role gate. `ActionRow` has the only Primary `Crear borrador`; commands: `cd persona-sanus && cargo check`.
- [ ] Adopt PrescriptionPreview — requirements: `PageShell id="prescriptions-page"`, `PageHeader title="Vista previa A4 — {patient_name}"` subtitle with the existing status, number, and date; header Secondary `ButtonLink` `Volver a la receta` -> `Route::PrescriptionDetail { id }`. Build an `ActionRow` that contains exactly one Primary overall: when `is_draft && can_sign`, Primary `Button` `Firmar y emitir` (loading while signing) and, when `needs_ack`, Secondary `Button` `Firmar reconociendo las advertencias`; otherwise Primary `LinkAction` `Descargar PDF` with the existing `pdf_url`, Secondary `LinkAction` `Imprimir` with the existing PDF URL, `target="_blank"`, and `rel="noopener noreferrer"`, Secondary `PDF provisional (local)` when `current.provisional`, and the wasm Secondary `Compartir` button with the existing share branch. Render a Warning `Notice` when provisional with `Documento provisional: se muestra sin QR escaneable hasta sincronizar.`; `ErrorText` for `message`; and `.ui-preview-canvas` wrapping `forms::A4Preview` with all existing arguments. Keep the signing dispatch, offline branch, acknowledge branch, sync probe, generation increment, and access guard unchanged. Only Primary: `Firmar y emitir` or `Descargar PDF`; commands: `cd persona-sanus && cargo check`; `rg -n 'sign_prescription_dispatch|needs_ack|pdf_url|provisional' persona-sanus/src/main.rs`.
- [ ] Adopt PrescriptionDetail — requirements: `PageShell id="prescriptions-page"`, `PageHeader title={current.patient_name}` subtitle with the existing status, number, and date; header Secondary `Vista previa A4` -> `Route::PrescriptionPreview { id }` and `Volver` -> `Route::Prescriptions { status: None }`. Render `RxBlockIndicator` with the existing block status, the new `allocating` signal, and the existing allocate request. Render `ErrorText` for `message`. In draft with `can_edit`, render edit fields in `Card` > `FormGrid` (Diagnóstico, Vigencia, Indicaciones generales wide) and a page `ActionRow`: when `can_sign`, Primary `Firmar receta` plus Secondary `Guardar borrador`, plus Secondary `Firmar reconociendo las advertencias` when `needs_ack`; when `profile.role == "nursing"`, Primary `Guardar borrador`; otherwise Secondary `Guardar borrador` only. In read mode render `DetailGrid`/`DetailItem`, including verification code/status and void reason when present. Render `SectionCard title="Medicamentos"`; empty list is `EmptyState` `Sin medicamentos` / `Agrega un medicamento al tratamiento.`. Each medication is `div.ui-item-card`: in edit mode `FormGrid` with the existing fields and `ActionRow` Secondary `Guardar medicamento`, Secondary `Subir`/`Bajar` with the existing first/last disabled rules, and Danger `Eliminar`; in read mode a `DetailGrid` with the same values. Add Secondary `Agregar medicamento` when `can_edit`. Keep each `LlmExplainLink` when `llm_explain_url` returns `Some`. For signed transitions render `ActionRow` Secondary `Reemplazar (corrección)` and Danger `Anular`, and `ConfirmPanel` with `TextField label="Motivo de anulación"` and Danger confirm `Confirmar anulación`. Keep every dispatch argument, reorder call, upsert/delete call, role gate, and `needs_ack` branch. Only Primary: `Guardar borrador` or `Firmar receta`; commands: `cd persona-sanus && cargo check`; `rg -n 'allocating|RxBlockIndicator|ui-item-card|ConfirmPanel|LlmExplainLink' persona-sanus/src/main.rs`.

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No new crate, CSS framework, external stylesheet, webfont, CDN asset, icon package, or JavaScript bridge.
- No rich text, WYSIWYG, `contenteditable`, formatting toolbar, `execCommand`, HTML payload, or editor dependency. All text is plain controlled inputs and textareas.
- No new product route, API handler, server function, Diesel model, table, migration, entitlement rule, or billing persistence.
- No change to authorization, session, device registration, sync, numbering, signing, PDF, QR, verification limits, or business validation.
- No change to the query/status filter semantics, desktop local fallbacks, outbox synchronization, `sign_prescription_dispatch`, `void_prescription_dispatch`, `supersede_prescription_dispatch`, `reorder_prescription_items`, `upsert_prescription_item`, `delete_prescription_item`, the acknowledge-warnings path, or any role gate.
- No extra Primary action. The list has only `Nueva receta`; the new page has only `Crear borrador`; the preview has exactly one of `Firmar y emitir` or `Descargar PDF`; the detail has only `Firmar receta` or `Guardar borrador` depending on role/state. `Guardar medicamento`, `Subir`, `Bajar`, `Eliminar`, `Agregar medicamento`, `Reemplazar (corrección)`, and `Anular` are Secondary or Danger.
- No modal. `ConfirmPanel` stays inline; focus is never trapped.
- No patient photo, signature, or stamp control (D-06); no rich-text editor (D-21).
- No real patient name, cédula, diagnosis, medication, or photo in a sample, gallery, or test fixture that renders outside a throwaway database.
- No pure black, drop shadow, chromatic card border, bold grenette, or second stylesheet.
- No unit tests and no comments in Rust, CSS, or RSX.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [ ] `cd persona-sanus && cargo check` exits 0.
- [ ] `cd persona-sanus && cargo check --no-default-features --features desktop` exits 0.
- [ ] objective 1: `rg -n 'failed_rows|pending_sync|Provisional · PENDIENTE DE SINCRONIZACIÓN' persona-sanus/src/main.rs` still returns the failed-row branch and the exact badge label.
- [ ] objective 1: source has one `Nueva receta` Primary/`ButtonLink` on the list; the filters use `FilterBar`, `SearchField`, `SelectField`, and `TextField`; `LoadingState` and `EmptyState` are present.
- [ ] objective 1: browser `/prescriptions` as a signer shows one filled `Nueva receta`, filters by patient/status/date, empty state for no match, pending rows with the Cream provisional badge, and the failed-rows card on a desktop failure.
- [ ] objective 2: browser `/prescriptions/new?patient_id=...&template_id=...` prefills patient and template, keeps only published templates, creates a draft, and lands on the detail. One filled `Crear borrador`.
- [ ] objective 3: browser `/prescriptions/:id/preview` as a draft signer shows one filled `Firmar y emitir` and no filled download. After signing (or as a non-signer) it shows one filled `Descargar PDF` and no filled sign control. `Imprimir` opens the PDF in a new tab. Provisional warning `Documento provisional: se muestra sin QR escaneable hasta sincronizar.` is visible only when provisional.
- [ ] objective 3: ack path shows Secondary `Firmar reconociendo las advertencias` after the first sign attempt returns unknown variables; signing again succeeds and does not add a second Primary.
- [ ] objective 3: desktop build shows `PDF provisional (local)` only when provisional; wasm build shows `Compartir`; both are Secondary.
- [ ] objective 4: browser `/prescriptions/:id` as a doctor draft shows one filled `Firmar receta` plus Secondary `Guardar borrador`; as nursing draft shows one filled `Guardar borrador`; as another draft role shows no filled primary and a Secondary `Guardar borrador`; as read-only/signed shows no filled primary.
- [ ] objective 4: medication add, save, reorder up/down, and delete preserve their API calls and reorder constraints; `Subir` is disabled on the first item and `Bajar` on the last; the empty list shows `Sin medicamentos`.
- [ ] objective 4: the signed transition shows Secondary `Reemplazar (corrección)` and Danger `Anular`; `Anular` opens inline `ConfirmPanel`, the reason `TextField` is required by the existing dispatch, confirm `Confirmar anulación` is Danger, cancel is Secondary `Cancelar`, and focus is not trapped.
- [ ] objective 4: while `Reservar bloque` is in flight, `allocating` disables the button and a second click sends no request; after success/failure it re-enables. `RxBlockIndicator` renders nothing when status is `None`.
- [ ] Primary count: list 1, new 1, preview 1, detail 1 only when `can_sign` or nursing draft, otherwise 0.
- [ ] Focus pass: Tab through the filters, new form, preview actions, medication fields, and void panel; every control shows the visible focus ring.
- [ ] Responsive pass: filter bar auto-fits; form/detail grids stack below 760; item actions wrap; only the A4 preview and tables scroll horizontally.
- [ ] every objective in section 1 is checked.
- [ ] every divergence found is appended to section 4 below.

## 4. Divergences

*Process and tracking-file rules: see `AGENTS.md` (§ Spec-Driven Development).
Summary: when reality diverges from the spec, append a numbered entry below;
leave the matching testing-criteria `- [ ]` unchecked and note the divergence
inline next to it. Open divergences owned by a later spec also get a tracking
file in `specs/divergences/`.*

- [ ] every divergence found is appended below (or the section reads "None.")

*Entry format (one numbered item per divergence):*

1. <what diverged and its current state> — <why it could not be done as
   written, citing the No-go rule or spec that blocks it> (Verified <date>:
   <N> fix attempts — <attempt 1>, <attempt 2>, <attempt 3> — all fenced by
   <constraint>; <impact> is deferred to <later spec> by design, not a defect.)

### Divergences

None recorded yet.

## 5. Context

The prescription pages are the highest-risk adoption because they carry the signing and synchronization state. The existing code already has the correct guards and dispatch calls; the rewrite must keep them and only change the surrounding RSX.

`Prescriptions` owns the query (`query`), status filter (`status`), date (`date`), rows, desktop `failed_rows`, and a `loading` flag. `PrescriptionNew` already owns both patient/template lists and a `loaded` flag for that fetch. `PrescriptionPreview` owns `preview`, `prescription`, `signing`, `needs_ack`, and `generation`; its signing branch calls `sign_prescription_dispatch(&preview_id, false, offline)` and recomputes sync. `PrescriptionDetail` owns the editable diagnosis/validity/instructions, medication items, `saving`, `signing`, `needs_ack`, `void_reason`, `show_void`, and `block`. The block request is currently `api::allocate_rx_block()` plus `api::fetch_rx_block_status()`; it becomes wrapped by `allocating`.

The page table's primary rule interacts with role gates:
- A doctor who can sign sees one Primary `Firmar receta` on the draft detail and one Primary `Firmar y emitir` on the preview.
- Nursing cannot sign, so on a draft detail it sees one Primary `Guardar borrador`; on the preview it sees the PDF primary because the sign branch is unavailable.
- Secretary/administrator read states have no primary.
- The list's Primary is `Nueva receta` only when create permission exists.
- `Guardar borrador`, `Subir`, `Bajar`, `Eliminar`, `Agregar medicamento`, `Reemplazar`, and `Anular` are Secondary or Danger, so they do not create a second filled primary.

## 6. Reuse, do not rebuild

| Existing symbol | Decision |
|---|---|
| `forms::A4Preview` | Keep and render the real document inside `.ui-preview-canvas`; do not duplicate the layout engine. |
| `forms::TemplateDocumentForm` | Not touched by this spec. `22-V` owns it. |
| `Navbar` | Keep; these pages render inside its layout and do not add a sign-out. |
| `Route`, `App` | Keep. No route additions, renames, or `/verify` changes. |
| `forms::paginate_document`, `resolve_text`, document warnings | Keep unchanged. |
| `SyncStatus`, `probe_sync_status`, local count helpers | Consume `18-R`; `PrescriptionDetail` still calls `probe_sync_status` after signing and allocation. |
| `sign_prescription_dispatch`, `void_prescription_dispatch`, `supersede_prescription_dispatch` | Keep the signatures, bodies, API arguments, offline tuple, history/outbox updates, and navigation. |
| `api::upsert_prescription_item`, `reorder_prescription_items`, `delete_prescription_item` | Keep the calls and arguments; only the controls change. |
| `llm_explain_url` | Keep in `main.rs` and reuse the returned `Option<String>`; do not move it into the UI module. |
| `PrescriptionStatusBadge` | Consume `18-R`; do not duplicate status labels. |
| `Badge` tones | Consume `16-P`/`18-R`; no page-local badge CSS. |
| `.a4-*` CSS, A4 geometry, QR layout | Keep. `.ui-preview-canvas` only frames the scroll. |
| `failed_rows` and desktop list fallbacks | Keep both and their `#[cfg(feature = "desktop")]` branches. |

## 8. Page adoption

### Prescriptions

- `PageShell id="prescriptions-page"`.
- `PageHeader title="Recetas" subtitle="Historial con filtros por paciente, fecha y estado."`; header Primary `ButtonLink` `Nueva receta` -> `Route::PrescriptionNew { patient_id: None, template_id: None }` when `profile.can_create_prescriptions()`.320
- `FilterBar`:
  - `SearchField label="Paciente" placeholder="Paciente"` bound to `query`.
  - `SelectField label="Estado"` bound to `status`, options in this order: `Todos los estados` (`""`), `Borradores` (`draft`), `Firmadas` (`signed`), `Reemplazadas` (`superseded`), `Anuladas` (`voided`), `Pendientes de sincronización` (`pending_sync`), `Con error de sincronización` (`failed`).
  - `TextField label="Fecha" input_type="date"` bound to `date`.
- Add `loaded` false; set false at the start of each fetch, true after every success/error/desktop branch resolves. `LoadingState` while `!loaded()`.
- `ErrorText` for `message` when non-empty.
- `EmptyState title="Sin recetas" description="No hay recetas para estos filtros."` when `visible.is_empty() && loaded()`.
- `ListRow` per visible row: title `patient_name`; meta `[status_label, format_date(created_at), "No. {n}" or "Sin número"]`; badge `PrescriptionStatusBadge { status }`; Secondary `Abrir` -> `Route::PrescriptionDetail { id }`.
- When row status is `pending_sync`, add Cream `Badge` `Provisional · PENDIENTE DE SINCRONIZACIÓN` inside the row body.
- `failed_rows`: when non-empty, `SectionCard title="Errores de sincronización"` with `ListRow title={operation} meta={vec![detail]}`. Keep the desktop branch that fills it.
- Only Primary: `Nueva receta`.

### PrescriptionNew

- `PageShell id="prescriptions-page"`.
- `PageHeader title="Nueva receta" subtitle="Paciente y plantilla publicada."`; header Secondary `ButtonLink` `Cancelar` -> `Route::Prescriptions { status: None }`.
- `Card` > `FormGrid columns: Two`:
  - `SelectField label="Paciente" value={selected_patient}` with the existing patient options and first option `Selecciona un paciente`.
  - `SelectField label="Plantilla" value={selected_template}` with the existing published-template options (value is the current version id, label is the template name) and first option `Selecciona una plantilla`.
  - `TextAreaField label="Diagnóstico"` bound to diagnosis.
  - `TextField label="Vigencia"` bound to validity.
  - `TextAreaField label="Indicaciones generales" wide` bound to instructions.
- `ErrorText` for `message`.
- `ActionRow` with Primary `Crear borrador`.
- Keep the `loaded` fetch for patients/templates and both APIs' desktop fallbacks; keep the create gate `profile.can_create_prescriptions()` and the create dispatch/navigation.
- Only Primary: `Crear borrador`.

### PrescriptionPreview

- `PageShell id="prescriptions-page"`.
- `PageHeader title={"Vista previa A4 — " + patient_name}` subtitle `"{status} · No. {n} · {date}"` (omit the number segment when absent; preserve the existing date field). Header Secondary `ButtonLink` `Volver a la receta` -> `Route::PrescriptionDetail { id }`.
- `ActionRow`:
  - `is_draft && can_sign`: Primary `Button` `Firmar y emitir`, disabled/loading while `signing`; if `needs_ack`, Secondary `Button` `Firmar reconociendo las advertencias` running the existing acknowledge dispatch.
  - Otherwise: Primary `LinkAction` `Descargar PDF` href `pdf_url`; Secondary `LinkAction` `Imprimir` href `pdf_url`, `target="_blank"`, `rel="noopener noreferrer"`; when `current.provisional`, Secondary `Button` `PDF provisional (local)` with the existing local render branch; on wasm, Secondary `Button` `Compartir` with the existing share branch.
- The desktop/provisional and wasm/share branches stay behind their existing cfg gates and keep their messages.
- Warning `Notice` when `current.provisional`: `Documento provisional: se muestra sin QR escaneable hasta sincronizar.`
- `ErrorText` for `message`.
- `div.ui-preview-canvas` > `forms::A4Preview` with `doc_json`, `values`, `margin_mm`, `medications`, `meta`, `image_sources` unchanged.
- Keep the signing and acknowledge dispatch bodies, sync probe, generation increment, and `can_sign` guard.
- Only Primary: `Firmar y emitir` or `Descargar PDF`.

### PrescriptionDetail

- `PageShell id="prescriptions-page"`.
- `PageHeader title={current.patient_name}` subtitle with existing status, number, and date; header Secondary `ButtonLink` `Vista previa A4` -> `Route::PrescriptionPreview { id }`, Secondary `ButtonLink` `Volver` -> `Route::Prescriptions { status: None }`.
- Add `let mut allocating = use_signal(|| false);` next to `block`. Pass `block()`, `allocating()`, and an `on_allocate` closure to `RxBlockIndicator`. The closure returns if `allocating()`; sets allocating true; calls the existing `api::allocate_rx_block()`; on desktop caches the block and sets `active_rx_block` exactly as now; then calls `api::fetch_rx_block_status()` and sets `block` when it succeeds; on error sets `message`; always sets `allocating` false at the end. Do not change the allocation API or desktop cache calls.
- `ErrorText` for `message`.
- Draft with `can_edit`: `Card` > `FormGrid` with `TextAreaField label="Diagnóstico"`, `TextField label="Vigencia"`, `TextAreaField label="Indicaciones generales" wide`, bound to the existing draft signals. Page `ActionRow`:
  - `can_sign`: Primary `Button` `Firmar receta`, Secondary `Button` `Guardar borrador`, and when `needs_ack` Secondary `Button` `Firmar reconociendo las advertencias`.
  - `profile.role.as_deref() == Some("nursing")`: Primary `Button` `Guardar borrador`.
  - Otherwise: Secondary `Button` `Guardar borrador` only.
  - Preserve the `saving`/`signing` disabled labels and every dispatch body.
  - If `block_state` is exhausted, keep the warning hint `Sin números disponibles: reserva un bloque para firmar.` under the primary sign action.
- Read mode: `Card` > `DetailGrid` with Diagnóstico, Vigencia, Indicaciones, verification code/status when present, and void reason when present. Keep the `—` fallbacks.
- `SectionCard title="Medicamentos"`:
  - Empty: `EmptyState title="Sin medicamentos" description="Agrega un medicamento al tratamiento."`.
  - Each item: `div.ui-item-card` with `key={index}`. Edit mode: `FormGrid` with the existing seven fields and `ActionRow` Secondary `Guardar medicamento`, Secondary `Subir` (disabled at first), Secondary `Bajar` (disabled at last), Danger `Eliminar`. Read mode: `DetailGrid` with the same values. Keep every save/reorder/delete closure and argument.
  - When `can_edit`, Secondary `Button` `Agregar medicamento` calls the existing `new_prescription_item_form` push.
  - Below each item, when `llm_explain_url` returns `Some`, render `LlmExplainLink { href: url }`. Do not duplicate the disclaimer.
- Signed transition (existing `can_transition`): `ActionRow` Secondary `Button` `Reemplazar (corrección)` and Danger `Button` `Anular`. `Anular` toggles `show_void`. When `show_void`, render `ConfirmPanel` with `TextField label="Motivo de anulación" value={void_reason}`, confirm label `Confirmar anulación`, cancel `Cancelar`, `busy: false`, and the existing void dispatch. Do not trap focus.
- Only Primary: `Guardar borrador` or `Firmar receta`.

## 9. CSS contract

| Selector | Values |
|---|---|
| `.ui-item-card` | Paper, 1px hairline, radius 24, padding 16px 20px, margin-bottom 12 |
| `.ui-preview-canvas` | owned by `22-V`; consume it, do not redefine |

No page-specific selector is added. The pages use `.ui-page`, `.ui-container`, `.ui-stack`, `.ui-page-head`, `.ui-card`, `.ui-card-title`, `.ui-form-grid`, `.ui-detail-grid`, `.ui-field`, `.ui-input`, `.ui-textarea`, `.ui-select`, `.ui-actions`, `.ui-filter-bar`, `.ui-empty`, `.ui-loading`, `.ui-row`, `.ui-notice`, `.ui-error`, `.ui-llm`, `.ui-notes`, and `.ui-badge` from earlier specs. Inputs, buttons, and item cards have no shadow; only the focus ring is allowed.

## 10. Critical files and anchors

These are hints; reread before editing.

| File | Anchor |
|---|---|
| `persona-sanus/src/main.rs` | `Prescriptions` 4360; `PrescriptionNew` 4595; `PrescriptionPreview` 4939; `PrescriptionDetail` 5380; `sign_prescription_dispatch`; `void_prescription_dispatch`; `supersede_prescription_dispatch`; `update_prescription_offline`; `llm_explain_url`; `new_prescription_item_form`; `enum Route` 48-93 |
| `persona-sanus/src/ui/domain.rs` | `PrescriptionStatusBadge`, `RxBlockIndicator`, `LlmExplainLink` |
| `persona-sanus/src/ui/form.rs` | `TextField`, `TextAreaField`, `SelectField`, `SearchField`, `Button` |
| `persona-sanus/src/ui/layout.rs` | `PageShell`, `PageHeader`, `Card`, `SectionCard`, `FormGrid`, `DetailGrid`, `DetailItem`, `ActionRow`, `FilterBar` |
| `persona-sanus/src/ui/feedback.rs` | `Badge`, `Notice`, `ErrorText`, `HintText`, `EmptyState`, `LoadingState`, `ListRow`, `ConfirmPanel` |
| `persona-sanus/assets/main.css` | Add `.ui-item-card`; consume `.ui-preview-canvas` from `22-V`; keep legacy selectors until `25-Y` |
| `context/PRD.md` | FR-010–014, FR-022, FR-030–034, FR-040–043, D-06, D-21 |

## 11. Assumptions and contingencies

- `current.created_at` is the list date. Keep the existing display choice rather than switching to `issued_at`.
- The existing list renders the pending badge when the status filter is `pending_sync`; this spec renders it when the row status is `pending_sync`, so a mixed row list is truthful. Do not change the filter values or desktop row shape.
- `"nursing"` is the exact role string used by `Me`; keep it. Do not infer nursing from a permission method because the primary-action rule depends on the role.
- The preview "PDF provisional (local)" button requires desktop/server PDF rendering. Keep its existing cfg gate. The wasm `Compartir` button keeps `web_sys`; no new dependency.
- The verify page is public, but prescription preview/detail are authenticated. Do not add a public title or public patient data to these pages.
- The detail `ConfirmPanel` is inline, not a modal. Its confirm button is Danger; cancel is Secondary. Do not trap focus or add overlay CSS.
- If a dispatch path refuses an empty void reason, keep that refusal. Do not disable confirm based on a new validation rule.
- If `allocating` cannot be passed as a `bool` prop because of a Dioxus signal mismatch, pass `allocating()` (the value), not the signal.
- If the server-side PDF branch is not available in the target feature set, keep the existing branch and labels; do not fake a download.
- If browser verification finds a control not in the Design.md list but required by an existing page, use the nearest primitive (`Badge`, `Card`, `Button Secondary`) and do not add a component file. The component set is closed.

---

*Housekeeping: once a spec is fully done with no open divergences, move its
file to `specs/done/` (same file name, e.g.
`specs/done/23-W-ui-prescriptions.md`).
Specs with open divergences stay in `specs/` next to `TEMPLATE.md` until the
tracking file is cleared.*
