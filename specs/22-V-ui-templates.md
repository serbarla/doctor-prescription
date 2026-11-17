# Spec 22 — Phase V: Templates adoption

- Source: `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` step 14 and the Page adoption table; `context/sitemap.md` §§1.1–1.3, §2, §5; `context/PRD.md` FR-020/021/023/024, FR-080–085, D-06, D-21; `context/architecture.md` §3; `context/database-design.md` §4.
- Depends on: 15-O, 16-P, 17-Q, 18-R, 19-S, 20-T.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Result carried forward: `Templates`, `TemplateNew`, `TemplateEdit`, `TemplatePreview`, and `forms::TemplateDocumentForm` render through the UI layer; the form's document shape, serde validation, variable insertion, asset selection, medication add/remove, save dispatch, `on_saved` payload, A4 pagination, and generation keys are unchanged; `TemplateEdit` has exactly one filled primary, `Publicar versión`.

## Before you start

Unit testing is not required; avoid it. Prioritize end-to-end and integration testing only.

Read the nearest `AGENTS.md` (target workspace, then parents up to repo root) before executing.

If reality diverges from this spec: STOP, append a divergence (section 4), and NEVER improvise outside the objectives.

This is markup and control substitution only. Do not rewrite `parse_document`, `validate_document`, `paginate_document`, `resolve_text`, `sample_values`, `default_document_json`, `template_theme_margin`, `save_document`, or the publish/archive dispatch bodies. Do not change the `TemplateDocument` fields or their serde names.

## 1. Objectives

*The ordered work to implement — one item per step. Check `[x]` only after the step is implemented and verified.*

*Item format:* `<verb phrase>` — requirements: `<what must change / constraints>`; commands: `<exact command>`.

- [ ] Adopt the Templates list and TemplateNew — requirements: `Templates` uses `PageShell id="templates-page"`, `PageHeader title="Plantillas"` with the existing subtitle, and a Primary `ButtonLink` `Nueva plantilla` -> `Route::TemplateNew {}`. Add `loaded` set true after the existing fetch resolves and render `LoadingState` while `!loaded()`. Render `EmptyState title="Todavía no hay plantillas." description=""` when the list is empty. Each row is a `ListRow` with title `name`, meta owner/current version, badge `TemplateStatusBadge { status }`, and Secondary `ButtonLink`s `Editar` and `Vista previa` (preview only when not archived). `TemplateNew` uses `PageShell id="templates-page"`, `PageHeader title="Nueva plantilla"` with Secondary `Cancelar`, `Card` + `FormGrid`, one `TextField label="Nombre"` bound to the existing name signal, `HintText` `Tipo: {owner}`, `ErrorText` for the error, and `ActionRow` with the only Primary `Crear plantilla`. Keep create dispatch and redirect; commands: `cd persona-sanus && cargo check`.
- [ ] Adopt TemplateEdit — requirements: `PageShell id="templates-page"`; `PageHeader title={model.name}` with subtitle `{owner} · {status}`; Secondary `ButtonLink`s `Vista previa` and `Volver` using the existing targets. Keep the archived Warning `Notice` with the existing copy and hide the form while archived. Render `TemplateDocumentForm`. Below it, `ErrorText` for the page message and `ActionRow` with Primary `Publicar versión` (disabled until `saved_draft` is `Some`), Secondary `Recargar`, and Danger `Archivar plantilla`. The archive control toggles `ConfirmPanel` with message `La plantilla archivada no admite cambios.`, confirm `Confirmar archivo`, cancel `Cancelar`; confirm calls the existing archive dispatch, sets the message, increments `generation`, and refreshes the model. Keep every generation key, publish dispatch, archive dispatch, and navigation target. Only Primary: `Publicar versión`; commands: `cd persona-sanus && cargo check`.
- [ ] Adopt TemplatePreview — requirements: `PageShell id="templates-page"`; `PageHeader title="Vista previa A4" subtitle="210 × 297 mm con valores de ejemplo."` with Secondary `Volver a plantillas` and `Abrir editor`. Controls in `Card` + `FormGrid`: `SelectField` for the existing version list and `TextField` for the number margin with `min="5"`, `max="40"`, and clamping 5..40. Keep the existing selected/margin signals and the version-change margin sync. Warnings render as `Notice tone="Warning"` titled `Variables no válidas` with the joined warning list and the existing warning hint. Wrap `forms::A4Preview` in `.ui-preview-canvas` and keep every existing argument. If no version exists, keep a Warning `Notice` and a Secondary `Abrir editor` action. No Primary; commands: `cd persona-sanus && cargo check`.
- [ ] Restyle TemplateDocumentForm — requirements: replace every raw RSX `input`, `textarea`, `select`, and `button` in `forms::TemplateDocumentForm` with `TextField`, `TextAreaField`, `SelectField`, `Button`, and `Card`/`SectionCard`/`FormGrid`/`ActionRow`/`ErrorText`/`HintText`. Keep the existing `TemplateDocument` shape, signals, serde validation, `parse_document` fallback, variable insertion, asset selection, medication add/remove, row keys, `save_document` dispatch, and the exact `on_saved: EventHandler<(String, String, u32)>` payload. Keep `A4Preview` in `TemplatePreview`; replace `A4Preview`'s parse-error `p.error` with `ErrorText` and keep its pagination/geometry return unchanged. `Guardar borrador`, `Agregar imagen`, `Agregar medicamento`, and `Insertar variable` are Secondary buttons; every remove control is Danger. No Primary inside the form. Preserve the `Guardando...` busy label and disabled state; commands: `cd persona-sanus && cargo check`; `rg -n '\\b(input|textarea|select|button) \\{' persona-sanus/src/forms.rs`; `rg -n 'class: "(error|hint|field|form-grid|editor-section|medication-block|inline-block)"' persona-sanus/src/forms.rs`.

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No new crate, CSS framework, external stylesheet, webfont, CDN asset, icon package, or JavaScript bridge.
- No rich text, WYSIWYG, `contenteditable`, formatting toolbar, `execCommand`, HTML payload, or editor dependency. Body, footer, and notes stay plain textareas/inputs.
- No new product route, API handler, server function, Diesel model, table, migration, entitlement rule, or billing persistence.
- No change to authorization, session, device registration, sync, numbering, signing, PDF, QR, verification limits, or business validation.
- No change to the `TemplateDocument` shape, SCHEMA_VERSION, serde names, validation, variable expansion, asset pinning, version/archive dispatch, or `on_saved` payload. No new document field.
- No change to `A4Preview` pagination, page breaks, margin geometry, mm sizes, QR layout, variable resolution, or sample values defaults.
- No change to fetch/create/publish/archive API arguments. Only UI state and markup.
- No new signature, stamp, or patient-photo control (D-06). No rich-text editor (D-21).
- No extra Primary action. `Templates` has only `Nueva plantilla`; `TemplateNew` has only `Crear plantilla`; `TemplateEdit` has only `Publicar versión`; `TemplatePreview` has none; `TemplateDocumentForm` has none.
- No pure black, drop shadow, chromatic card border, bold grenette, or second stylesheet.
- No unit tests and no comments in Rust, CSS, or RSX.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [ ] `cd persona-sanus && cargo check` exits 0.
- [ ] `cd persona-sanus && cargo check --no-default-features --features desktop` exits 0.
- [ ] objective 4: `rg -n '\\b(input|textarea|select|button) \\{' persona-sanus/src/forms.rs` returns no matches.
- [ ] objective 4: `rg -n 'class: "(error|hint|field|form-grid|editor-section|medication-block|inline-block)"' persona-sanus/src/forms.rs` returns no matches.
- [ ] objective 4: `rg -n 'A4Preview|paginate_document|resolve_text' persona-sanus/src/forms.rs` still shows the A4 preview, paginator, and resolver, and the `A4Preview` signature is unchanged.
- [ ] objective 4: `rg -n 'on_saved|save_document|serde_json::to_string|template_theme_margin' persona-sanus/src/forms.rs` shows the same save/validation/theme calls and the `on_saved` tuple.
- [ ] Browser pass `/templates` (doctor and administrator): loading on first load, status badges, actions hidden for archived, exactly one filled `Nueva plantilla`.
- [ ] Browser pass `/templates` empty clinic: `Todavía no hay plantillas.` and no row actions.
- [ ] Browser pass `/templates/new`: `Nombre`, visible `Tipo: {owner}` hint, Secondary `Cancelar`, one filled `Crear plantilla`; creating redirects to the editor.
- [ ] Browser pass `/templates/:id/edit`: header title/owner-status, Secondary `Vista previa`/`Volver`, the archived notice, styled fields and buttons, variable insertion, image add/remove, medication add/remove, save draft, publish disabled until saved, publish enabled after save, `Recargar`, Danger `Archivar plantilla`, archive `ConfirmPanel` with `La plantilla archivada no admite cambios.` / `Confirmar archivo` / `Cancelar`, and exactly one filled `Publicar versión`.
- [ ] Browser pass `/templates/:id/preview`: `Vista previa A4`, `210 × 297 mm con valores de ejemplo.`, Secondary `Volver a plantillas` and `Abrir editor`, version select, margin number clamped to 5..40, Warning `Notice` for invalid variables, only the `.ui-preview-canvas` scrolls horizontally, and no filled primary.
- [ ] Focus pass: Tab through the list, create, editor, and preview; every control shows the visible focus ring. No focus trap.
- [ ] Responsive pass: form/detail grids are one column below 760; page head stacks below 640; steps/page layout do not overflow.
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

The template pages already have the full data flow: `Templates` fetches a list, `TemplateNew` creates a template and redirects, `TemplateEdit` resolves the latest draft/current version, and `TemplatePreview` reparses the selected version and renders the A4 projection. `forms::TemplateDocumentForm` owns the document editor state and serializes the entire document on save.

Only the markup changes. The editor currently uses raw `label.field`, `input`, `textarea`, `select`, and `button.secondary`/`button.danger` elements. Those become the shared form controls and cards. The legacy classes disappear from RSX; no new CSS is added beyond the canvas and variable row in section 9.

The page table's one-primary rule is the main regression risk. The form's save button is Secondary, every add/remove/insert control is Secondary or Danger, and `Publicar versión` is the only Primary on `TemplateEdit`. `Templates` and `TemplateNew` each have their own single Primary; `TemplatePreview` has none.

## 6. Reuse, do not rebuild

| Existing symbol | Decision |
|---|---|
| `forms::A4Preview` | Keep. Only replace its parse-error paragraph with `ErrorText`; pagination and geometry stay. |
| `forms::TemplateDocumentForm` | Keep the component and its state. Replace raw controls and legacy classes only. |
| `forms::parse_document`, `validate_document`, `paginate_document`, `resolve_text`, `document_warnings`, `sample_values`, `default_document_json`, `template_theme_margin`, `save_document` | Keep unchanged. |
| `api::fetch_template`, `create_template_version`, `update_template_version`, `publish_template`, `archive_template` | Keep the calls, arguments, and result handling unchanged. |
| `Route::{Templates, TemplateNew, TemplateEdit, TemplatePreview}` | Keep every variant, guard, and navigation target. |
| `Navbar`, page shell, cards, fields, buttons, notices, list rows, confirm panel, empty/loading states | Consume `15-O`–`20-T`. Do not duplicate markup or CSS. |
| `TemplateDocument` fields and serde names | Keep exactly. Do not add a revision, title, or UI-only field. |
| `.a4-*` CSS and A4 geometry | Keep. `.ui-preview-canvas` only provides the scroll/cream frame around the real A4 pages. |

## 7. Component surface

No new Rust components. This spec consumes `PageShell`, `PageHeader`, `Card`, `SectionCard`, `FormGrid`, `ActionRow`, `TextField`, `TextAreaField`, `SelectField`, `Button`, `ButtonLink`, `Badge`, `Notice`, `ErrorText`, `HintText`, `EmptyState`, `LoadingState`, `ListRow`, `ConfirmPanel`, and `TemplateStatusBadge`.

Fixed button variants:

| Control | Variant |
|---|---|
| `Nueva plantilla` | Primary |
| `Crear plantilla` | Primary |
| `Publicar versión` | Primary |
| `Guardar borrador` | Secondary |
| `Agregar imagen` | Secondary |
| `Agregar medicamento` | Secondary |
| `Insertar variable` | Secondary |
| `Recargar` | Secondary |
| `Vista previa` / `Volver` / `Cancelar` | Secondary |
| `Quitar imagen` / `Quitar medicamento` | Danger |
| `Archivar plantilla` | Danger |
| `Confirmar archivo` | Danger |
| `Abrir editor` / `Volver a plantillas` | Secondary |

The form's save button keeps the busy label `Guardando...` and disables while `saving`.

## 8. Page adoption

### Templates

- `PageShell id="templates-page"`.
- `PageHeader title="Plantillas"` with the existing subtitle.
- Header Primary `ButtonLink` `Nueva plantilla` -> `Route::TemplateNew {}`.
- Add `loaded` set true after the fetch resolves; `LoadingState` while `!loaded()`.
- Access denied keeps `Notice tone="Error"` and its existing copy.
- Empty: `EmptyState title="Todavía no hay plantillas." description=""`.
- Row: `ListRow title={template.name}`; meta={vec![owner_label.to_owned(), format!("Versión {n}")]}; badge `TemplateStatusBadge { status }`; actions Secondary `Editar` -> `Route::TemplateEdit { id }`, and Secondary `Vista previa` -> `Route::TemplatePreview { id }` only when `status != "archived"`.
- Only Primary: `Nueva plantilla`.

### TemplateNew

- `PageShell id="templates-page"`.
- `PageHeader title="Nueva plantilla"` with header Secondary `ButtonLink` `Cancelar` -> `Route::Templates {}`.
- `Card` > `FormGrid columns: One` > `TextField label="Nombre"` bound to the existing name signal.
- `HintText { text: format!("Tipo: {owner}") }` after the field, using the existing owner label.
- `ErrorText` for the create error.
- `ActionRow` with the only Primary `Crear plantilla`.
- Keep create dispatch, success redirect to the editor, and role gate.
- Only Primary: `Crear plantilla`.

### TemplateEdit

- `PageShell id="templates-page"`.
- `PageHeader title={model.name}` subtitle `{owner} · {status}`; actions Secondary `Vista previa` -> `Route::TemplatePreview { id }`, Secondary `Volver` -> `Route::Templates {}`.
- Archived: Warning `Notice` with the existing copy `La plantilla está archivada y no admite cambios.`; do not render the form or mutating actions.
- Active: `TemplateDocumentForm` with the existing `template_id`, `version_id`, `initial_doc`, `initial_margin_mm`, `on_saved`, and generation key.
- `ErrorText { text: message }` for the page message.
- `ActionRow`: Primary `Publicar versión` (disabled while `saved_draft().is_none()`), Secondary `Recargar`, Danger `Archivar plantilla`.
- Archive: Danger trigger toggles `ConfirmPanel`; panel message `La plantilla archivada no admite cambios.`, confirm label `Confirmar archivo`, cancel `Cancelar`, busy: false; the existing archive dispatch is immediate, confirm calls the existing archive dispatch, cancels without a call.
- Keep all publish/archive dispatch bodies and generation increments.
- Only Primary: `Publicar versión`.

### TemplatePreview

- `PageShell id="templates-page"`.
- `PageHeader title="Vista previa A4" subtitle="210 × 297 mm con valores de ejemplo."`; actions Secondary `ButtonLink` `Volver a plantillas` -> `Route::Templates {}`, Secondary `ButtonLink` `Abrir editor` -> `Route::TemplateEdit { id }`.
- Access denied stays `Notice tone="Error"`.
- Controls: `Card` > `FormGrid columns: Two` > `SelectField label="Versión"` with the existing options and `TextField label="Márgenes (mm)" input_type="number" min="5" max="40"`; on input parse `u32` and clamp 5..40. Keep the version-change margin sync.
- Warnings: when non-empty, `Notice tone="Warning"` with title `Variables no válidas`, the joined warnings, and the existing hint `En la proyección se marcan como texto de advertencia, nunca como {{clave}}.`
- Canvas: `div.ui-preview-canvas` > `forms::A4Preview` with the existing `doc_json`, `values: forms::sample_values()`, and `margin_mm` arguments. Do not add or reorder arguments.
- No version: Warning `Notice` `La plantilla todavía no tiene versiones para previsualizar.` with Secondary `Abrir editor` to `Route::TemplateEdit { id }`.
- No Primary.

### TemplateDocumentForm

- Membrete: `SectionCard title="Membrete"` > `FormGrid`. Logo `SelectField` (options: `Sin logo` plus library assets). Primary color and accent color `TextField` `input_type="color"`. Margin `TextField input_type="number" min="5" max="40"`; keep the existing clamp. Doctor details and clinic details `TextAreaField wide`. Keep all signals and oninput/onchange assignments.
- Cuerpo: `SectionCard title="Cuerpo"` > `div.ui-variable-row` with a `SelectField hide_label` for `TEMPLATE_VARIABLES` (option label `{variable_label(key)} — {key}`) and Secondary `Insertar variable`. Then `TextAreaField label="Cuerpo" hide_label wide` for body. Then `TextAreaField label="Pie de página"` for footer. Keep the variable picker signal and insertion.
- Imágenes: `SectionCard title="Imágenes"` > `HintText` `Elige imágenes de la biblioteca de la clínica. La biblioteca admite una imagen en esta versión.`; each image is a `Card` with key `image-{index}`, `FormGrid`: `SelectField label="Imagen"` (`Sin imagen` plus `kind == "image"` assets) and `TextField label="Leyenda"`, then `ActionRow` with Danger `Quitar imagen`. Keep the add-image control as Secondary `Agregar imagen`; keep the first-image default selection.
- Medicamentos: `SectionCard title="Medicamentos"`; each medication is a `Card` with key `medication-{index}` > `FormGrid` with `TextField` labels `Medicamento`, `Dosis`, `Vía`, `Frecuencia`, `Duración`, `Cantidad`, and `Notas` (wide), then `ActionRow` with Danger `Quitar medicamento`. Add-medication control Secondary `Agregar medicamento`; keep `MedicationBlock::default`.
- Save: `ErrorText` for the existing `message`; `ActionRow` with one Secondary `Button` `Guardar borrador`, disabled/loading while `saving`, label `Guardando...` while busy. Keep `save_document` and the `on_saved` payload.
- `A4Preview` parse error: replace `p { class: "error", "{error}" }` with `ErrorText { text: error }`. Do not change the success tree or the pagination/geometry return.

## 9. CSS contract

| Selector | Values |
|---|---|
| `.ui-preview-canvas` | overflow-x auto, padding 24px, Cream background, 1px hairline, radius 24 |

`16-P` owns `.ui-variable-row` (grid `1fr auto`, gap 12, align-end, margin-bottom 12; one column below 640); this spec only consumes it.

No new literal colour outside `:root`; no drop shadow; no second stylesheet. The form's cards use the shared `.ui-card` surface; no `.template-editor`/`.editor-section` style hook remains in RSX after adoption.

## 10. Critical files and anchors

These are hints; reread before editing.

| File | Anchor |
|---|---|
| `persona-sanus/src/main.rs` | `Templates` 3567; `TemplateNew` 3680; `TemplateEdit` 3788; `TemplatePreview` 4049; `enum Route` 48-93 |
| `persona-sanus/src/forms.rs` | `A4Preview` 463; `TemplateDocumentForm` 635; `SCHEMA_VERSION` 9; `TEMPLATE_VARIABLES` 11; `parse_document`; `validate_document`; `paginate_document`; `save_document` |
| `persona-sanus/src/ui/mod.rs` | Components consumed by these pages |
| `persona-sanus/assets/main.css` | Add `.ui-preview-canvas`; `.ui-variable-row` is owned by 16-P and consumed here; keep legacy selectors until `25-Y` |
| `context/PRD.md` | FR-020/021/023/024, FR-080–085, D-06, D-21 |
| `context/sitemap.md` | template routes; role matrix §2 |

## 11. Assumptions and contingencies

- The color inputs become `TextField input_type="color"`; the CSS contract `.ui-input[type="color"]` gives the 64×44 swatch. If the installed Dioxus version cannot render `type=color` through `TextField`, keep one native `input[type=color]` inside the shared `Field` shell and record it as a divergence; do not add rich text.
- The body textarea has no visible label in the legacy form; use a visually hidden `Cuerpo` label. Do not invent a new visible heading.
- The template's medication block is a `Card`, not `.ui-item-card`; `.ui-item-card` is reserved for prescription medication items in `23-W`.
- `TemplateEdit`'s page `message` currently also receives positive status text. Keep the same signal and the same calls; render it with `ErrorText` because the legacy surface is `.error` and the page table calls for `ErrorText`. Do not add a second success/error state signal.
- If `A4Preview`'s parse-error branch is not reachable in a browser pass, source-verify `ErrorText { text: error }` and the unchanged `paginate_document` call.
- If a version is absent on `/templates/:id/preview`, keep the Warning notice and Secondary editor action; do not create a fake version.
- If browser verification finds a control not in the Design.md list but required by an existing page, use the nearest primitive (`Badge`, `Card`, `Button Secondary`) and do not add a component file. The component set is closed.

---

*Housekeeping: once a spec is fully done with no open divergences, move its
file to `specs/done/` (same file name, e.g.
`specs/done/22-V-ui-templates.md`).
Specs with open divergences stay in `specs/` next to `TEMPLATE.md` until the
tracking file is cleared.*
