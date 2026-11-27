# Spec 38 — Phase AL: HVQ two-panel template system v2

- Source: `HVQ_TEMPLATE_SYSTEM_PLAN.md` (HVQ two-panel template system v2 plan) and the HVQ photo (`.orca/drops/template copy.jpg`, 4096×3072) showing the Hospital Vozandes Quito prescription sheet; what triggered it is that the current system cannot recreate that sheet (single-column portrait line flow, 14 whitelisted `{{variables}}`, no tables/panels/landscape — verified in `persona-sanus/src/forms.rs`, `pdf.rs`, `service.rs`, `schema.rs`).
- Depends on: 05-E, 08-H (both in `specs/done/`).
- Repo root: `/home/orca/projects/doctor-prescription`. Every path and command below is relative to it.
- Owned files: `persona-sanus/src/forms.rs`, `persona-sanus/src/pdf.rs`, `persona-sanus/src/service.rs`, `persona-sanus/src/db.rs`, `persona-sanus/src/schema.rs`, `persona-sanus/src/desktop_local.rs`, `persona-sanus/src/api.rs`, `persona-sanus/src/api_server.rs`, `persona-sanus/src/main.rs`, `persona-sanus/assets/main.css`, `persona-sanus/assets/fonts/NotoSans-Bold.ttf`, `persona-sanus/migrations/20261123000000_hvq_clinical_fields/`, `persona-sanus/fixtures/golden-a4.json`, `persona-sanus/fixtures/golden-hvq.json`, `.blocks/pdf-probe.py`, `.blocks/persona-probe.py`, `.blocks/persona-smoke.py`, `.blocks/INDEX.md`, `AGENTS.md`, `specs/38-AL-hvq-template-v2.md`
- Gate: `python3 .blocks/pdf-probe.py --base http://127.0.0.1:8080` plus the new HVQ probe from objective 7 (`python3 .blocks/pdf-probe.py --base http://127.0.0.1:8080 --hvq`)
- Desktop-verify: yes (desktop reuses `pdf::render`, `main.rs` provisional path)
- Result carried forward: a doctor/admin can open `/templates/new`, pick the built-in preset "HVQ — Recetario ambulatorio (2 paneles)", adjust texts/colors/logo/margins with plain forms, publish it, create a prescription filling the new fields (HCL and allergies on the patient; recetario type, vigencia 1/3 días, signos de alarma, recomendaciones on the prescription; quantity as a number per item), and download a PDF that reproduces the photo's layout — A4 landscape, two panels, 6-col MEDICACIÓN table with `40 (CUARENTA)`, 4-col INDICACIONES table with numbered rows + `Observación:` sub-rows, vigencia checkbox ☒/☐, `EDAD: 20a 7m 11d`, prescriber block with Registro ACCESS, blank firma/sello/recepción areas, plus the existing QR/verification footer per panel. Existing v1 templates keep working via auto-migration.

## Before you start

Unit testing is not required; avoid it. Prioritize end-to-end and integration testing only.

Read the nearest `AGENTS.md` (target workspace, then parents up to repo root)
before executing.

If reality diverges from this spec: STOP, append a divergence (section 4), and
NEVER improvise outside the objectives.

## 1. Objectives

*The ordered work to implement — one item per step. Check `[x]` only after the
step is implemented and verified.*

*Item format:* `<verb phrase>` — requirements: `<what must change / constraints>`; commands: `<exact command>`.

- [x] 1. Create this spec file from `specs/TEMPLATE.md` — requirements: `specs/38-AL-hvq-template-v2.md` exists, filled from the plan (header, objectives = plan Steps 2–7, No-go = plan Step 8, Testing criteria = plan Verification section); this box is checked once the file exists, all other boxes stay unchecked until their implementation passes; commands: `cp specs/TEMPLATE.md specs/38-AL-hvq-template-v2.md` then edit in place
- [x] 2. Extend the clinical data model (migration + mirrors + plumbing) — requirements: one new migration `persona-sanus/migrations/20261123000000_hvq_clinical_fields/` (`up.sql`/`down.sql`, following the naming pattern of `20261113000000_hardening_mfa_audit`; confirm first whether existing down files use `DROP COLUMN` or table rebuilds and match their style), SQLite `ALTER TABLE … ADD COLUMN`, all nullable except where noted:
  | Table.Column | Type | Used for |
  |---|---|---|
  | `patients.medical_record_no` | `TEXT` | HCL (`722846-774928` in photo). Clinic-scoped: `CREATE UNIQUE INDEX idx_patients_clinic_hcl ON patients(clinic_id, medical_record_no) WHERE medical_record_no IS NOT NULL` (partial unique index works in SQLite; mirror intent in `desktop_local.rs` without the index — desktop tables have no indexes today, `desktop_local.rs:42-70`) |
  | `patients.allergies` | `TEXT` | `ALERGIAS:` (left panel). Distinct from admin `patients.notes` |
  | `prescriptions.recetario_type` | `TEXT NOT NULL DEFAULT 'ambulatorio'` | `RECETARIO: AMBULATORIO`. CHECK `recetario_type IN ('ambulatorio')` in migration so a future type is a migration, not silent free text |
  | `prescriptions.validity_days` | `INTEGER` | Vigencia checkbox: `1` → ☒ 1 DIA, `3` → ☒ 3 DIAS, `NULL` → both ☐. CHECK `validity_days IN (1,3)`. The existing free-text `prescriptions.validity` column stays (legacy display string) but the HVQ checkbox binds only to `validity_days` |
  | `prescriptions.warning_signs` | `TEXT` | `SIGNOS DE ALARMA:` (right panel) |
  | `prescription_items.quantity_number` | `INTEGER` | Numeric quantity; rendered as `{n} ({WORDS})` (objective 5). Existing free-text `quantity` stays as fallback/legacy text |
  `prescriptions.instructions` is **repurposed as** `RECOMENDACIONES NO FARMACOLÓGICAS:` (it already exists, ≤8000 chars, UI label "Indicaciones generales" at `main.rs:6152` — relabel to "Recomendaciones no farmacológicas" in objective 6). No new column for it. Diagnosis stays the existing free-text `prescriptions.diagnosis` (multiline authoring reproduces `ALOPECIA AREATA (CIE-10: L63)\nDERMATITIS NUMULAR (CIE-10: L30.0)` exactly; no CIE-10 table). Mirror every column in `persona-sanus/src/schema.rs` (columns in the `patients`, `prescriptions`, `prescription_items` `diesel::table!` blocks, schema.rs:118-134, 196-219, 221-237, same order as the migration appends them), `persona-sanus/src/desktop_local.rs` (inline `CREATE TABLE IF NOT EXISTS` within `desktop_local.rs:28-140` **and** `ALTER TABLE … ADD COLUMN` guards for existing desktop DBs — first check how `desktop_local.rs` currently handles schema evolution; if it only runs `CREATE TABLE IF NOT EXISTS`, add idempotent `ALTER` attempts ignoring "duplicate column" errors following any existing pattern found there; extend the desktop local read/write SQL and outbox payload JSON keys, `desktop_local.rs:776`, `:1367-1427` area), `persona-sanus/src/db.rs` (row structs `Patient` (:316), `Prescription` (:1715), `PrescriptionItem` (:1766), their `New*`/`*Fields` companions (`PatientFields` :339, prescription fields :1749, item fields :1784), and the insert/update/select functions that enumerate columns, physician-style Diesel field-by-field, no `..Default` shortcuts), `persona-sanus/src/service.rs` (`PatientInput`/`PatientUpdate` (:150-168), `PrescriptionInput`/`PrescriptionUpdate` (:2195-2208), `PrescriptionItemInput` (:2210-2219) gain `medical_record_no: Option<String>`, `allergies: Option<String>`, `recetario_type: Option<String>`, `validity_days: Option<i32>`, `warning_signs: Option<String>`, `quantity_number: Option<i32>`; reuse `clean_rx_text`-style trimming (`service.rs:2262`); `recetario_type` defaults to `"ambulatorio"`, reject anything else with `AuthError::Unprocessable("Tipo de recetario no válido.")`; `validity_days` must be `1` or `3` else `AuthError::Unprocessable("La vigencia debe ser de 1 o 3 días.")`; `quantity_number` must be `0..=100_000` else `AuthError::Unprocessable("La cantidad no es válida.")`), `persona-sanus/src/api.rs` + `api_server.rs` (client DTOs `api::Patient` :87, `api::PrescriptionItem` :122, `api::Prescription` :146 and server request/response structs `PatientCreateRequest`/`PrescriptionCreateRequest`/`PrescriptionItemRequest` in `api_server.rs:108-200` area, `patient_json`/`prescription_json`/`prescription_item_json` emitters :276-360, existing serde alias convention camelCase JSON keys: `medicalRecordNo`, `allergies`, `recetarioType`, `validityDays`, `warningSigns`, `quantityNumber`; no endpoint paths change). Every new field is optional; existing rows read as `NULL` → variables render `—` (objective 4) and checkboxes render ☐; no backfill. All code lives in the files listed under "Owned files" unless stated. No comments in code (repo rule). No new crates. No unit tests (repo rule: E2E/integration only). No rich-text editor (D-21). Ordered so the tree builds and existing probes pass after each step: objectives 2–3 are data/model foundations (independently mergeable), 4–5 are layout/render, 6 is editor UI, 7 is probes/fixtures; commands: `cargo check --manifest-path persona-sanus/Cargo.toml` and boot the server on a scratch DB per `persona-smoke` `--db-path` convention to confirm the migration, then the `sqlite-schema-check` block (objective 7 / testing criteria)
- [x] 3. Replace `TemplateDocument` with the v2 section model plus v1 auto-migration in `forms.rs` — requirements: replace the v1 struct (keeping the name `TemplateDocument` and the `doc_json` storage — no DB change) with `schema_version` 2. Exact types:
  ```rust
  pub const SCHEMA_VERSION: u32 = 2;
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  #[serde(deny_unknown_fields)]
  pub struct TemplateDocument {
      pub schema_version: u32,
      pub page: PageSetup,
      pub letterhead: Letterhead,
      pub panels: Vec<Panel>,
      #[serde(default)]
      pub images: Vec<ImageSlot>,
      #[serde(default)]
      pub medications: Vec<MedicationBlock>,
  }
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  #[serde(deny_unknown_fields)]
  pub struct PageSetup {
      pub orientation: Orientation,
      pub panel_count: u32,
  }
  #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
  #[serde(rename_all = "snake_case")]
  pub enum Orientation { Portrait, Landscape }
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  #[serde(deny_unknown_fields)]
  pub struct Panel {
      pub title: String,
      pub sections: Vec<Section>,
  }
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  #[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
  pub enum Section {
      Letterhead,
      PatientFields { fields: Vec<FieldKind> },
      Validity { label_1_day: String, label_3_days: String },
      Diagnosis { show_allergies: bool },
      FreeText { title: String, text: String },
      MedicationTable { style: MedicationTableStyle, show_observations: bool },
      IndicationList { show_observations: bool },
      WarningSigns,
      Recommendations,
      PrescriberBlock,
      ReceptionBlock,
      SignatureQr,
  }
  #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
  #[serde(rename_all = "snake_case")]
  pub enum MedicationTableStyle { Full, Compact }
  #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
  #[serde(rename_all = "snake_case")]
  pub enum FieldKind {
      RecetarioType, IssueDate, IssueTime, PatientNationalId, PatientHcl,
      PatientName, PatientAge, PatientSex,
  }
  ```
  Rules (exact, implementer does not choose): `Letterhead`, `Diagnosis`, `FreeText.text`, `PrescriberBlock`, `ReceptionBlock` texts resolve through the existing `resolve_text` (`{{variable}}` syntax unchanged); `Letterhead` renders `letterhead.logo_asset_id` + `letterhead.doctor_details` + `letterhead.clinic_details` exactly as today; `Letterhead` must be the first section of every panel that shows it; validation does not enforce position (editor always inserts it first — objective 6). `PatientFields` renders each `FieldKind` as a labelled value pair using the fixed Spanish labels from the photo: `RECETARIO:`, `FECHA:`, `HORA:`, `CI:`, `HCL:`, `APELLIDOS Y NOMBRES:`, `EDAD:`, `SEXO:`; labels are constants in code, not editable; field order in the array is render order. `MedicationTable { style: Full }` = left-panel MEDICACIÓN table: columns exactly `["Datos del medicamento", "Cantidad", "Dosis / Posología", "Vía de administración", "Frecuencia de la administración", "Duración del tratamiento"]` with the photo's full header captions stored as section fields `header_captions: Vec<String>` (6 entries, defaults = photo text: `"DATOS DEL MEDICAMENTO (DENOMINACIÓN COMÚN INTERNACIONAL (DCI), FORMA FARMACÉUTICA Y CONCENTRACIÓN/DISPOSITIVO MÉDICO (TAMAÑO, CALIBRE, VOLUMEN...))"`, `"CANTIDAD (EN NÚMEROS Y LETRAS)"`, `"DOSIS / POSOLOGÍA"`, `"VÍA DE ADMINISTRACIÓN"`, `"FRECUENCIA DE LA ADMINISTRACIÓN"`, `"DURACIÓN DEL TRATAMIENTO"`); column width weights (mm, panel-relative, sum = panel inner width): `[34%, 12%, 15%, 13%, 14%, 12%]`. `MedicationTable { style: Compact }` = a single-panel portrait 4-column table: columns `["Medicamento", "Dosis / Posología", "Vía", "Frecuencia"]`, weights `[40%, 20%, 18%, 22%]`, no quantity/duration; keep it as the migration target for v1 medication lists that want a table instead of bullets. `IndicationList { show_observations }` = right-panel INDICACIONES: 4-column table (headers `"DENOMINACIÓN COMÚN INTERNACIONAL (DCI)-MEDICAMENTO"`, `"DOSIS / POSOLOGÍA"`, `"VÍA DE ADMINISTRACIÓN"`, `"FRECUENCIA DE LA ADMINISTRACIÓN"`, weights `[42%, 20%, 18%, 20%]`), first cell prefixed `"{position}) "` (1-based), and when `show_observations` and `medication.notes` is non-empty, a sub-row spanning all 4 columns with text `"Observación: {notes}"` (wrapped); distinct section type (not a table style) because of numbering + sub-rows. `Validity` renders `VIGENCIA: ☐ {label_1_day} ☒ {label_3_days}`; labels default `"1 DIA"` / `"3 DIAS"` (photo spelling, no accent), editable section text. `Diagnosis { show_allergies }` renders `DIAGNÓSTICO:` + resolved `prescription.diagnosis` (multiline preserved), plus `ALERGIAS:` + `patient.allergies` when `show_allergies`. `WarningSigns` renders `SIGNOS DE ALARMA:` + `prescription.warning_signs`. `Recommendations` renders `RECOMENDACIONES NO FARMACOLÓGICAS:` + `prescription.instructions`. `PrescriberBlock` renders `DATOS DEL PRESCRIPTOR:` + `APELLIDOS Y NOMBRES: {{doctor.name}}`, `NÚMERO DE REGISTRO(ACCESS): {{doctor.license_no}}`, `NÚMERO DE CONTACTO-PRESCRIPTOR: {{doctor.contact}}`, `FIRMA:` + blank line (12 mm), `SELLO MÉDICO:` + blank area (18 mm). `ReceptionBlock` renders `RECEPCIÓN MEDICACIÓN DEL PACIENTE:` + `FIRMA: ______  N° CÉDULA: ______  PARENTESCO: ______` (static blanks, no variables). `SignatureQr` renders the existing system footer (Receta No./fecha, verification code or `PENDIENTE DE SINCRONIZACIÓN`, signature line, QR — existing `DocumentMeta` behaviour, moved into a section so each HVQ panel gets one; QR size 18 mm for panel layout vs 25 mm single-panel). `medications: Vec<MedicationBlock>` and `images` stay on the document (template-level sample/default medications, same semantics as today: prescription items override at render via `paginate_document`'s existing `medications` parameter path). `Panel.title` renders as the section heading (`MEDICACIÓN:` / `INDICACIONES:`) above the first table/list section of that panel; empty title renders nothing. **v1→v2 migration** — `parse_document` (`forms.rs:112`): try parse as v2 (`schema_version == 2`) → validate; else try parse as a private `TemplateDocumentV1` struct (exact copy of today's struct, kept private) with `schema_version == 1` → convert: `page = PageSetup { orientation: Portrait, panel_count: 1 }`; one panel, `title: ""`, sections in order: `Letterhead`, `FreeText { title: "", text: v1.body }`, (only when `v1.diagnosis` non-empty after trim) `FreeText { title: "Diagnóstico", text: v1.diagnosis }`, always `MedicationTable{Compact, show_observations: true}` (empty medication list renders nothing per objective 5's empty rule, so the empty case is invisible), `FreeText { title: "", text: v1.footer }`, `SignatureQr`; `letterhead`, `images`, `medications` copied verbatim. `TemplateDocument::default()` becomes the single-panel portrait v2 equivalent of today's default (same texts). A second constructor `TemplateDocument::hvq_preset()` (objective 6) builds the HVQ two-panel document. **Validation** (`validate_document`, `forms.rs:143`): keep all existing text caps; add: `schema_version == 2` after migration (error message unchanged for incompatible: `"La versión del esquema del documento no es compatible."` — v1 input never errors, it migrates); `panel_count` ∈ {1, 2}; `panels.len() == panel_count as usize`; landscape requires `panel_count == 2`, portrait requires `panel_count == 1` (error `"La orientación y el número de paneles no coinciden."`); per panel ≤ 12 sections; `MedicationTable.header_captions.len()` must equal the column count for its style (6 Full / 4 Compact; IndicationList captions are constants, not fields); every `FreeText.title` ≤ 120 chars, `FreeText.text` ≤ 40000 (same cap as old body); unknown `FieldKind`/section type impossible via serde (`deny_unknown_fields` + tagged enum). `document_warnings` (`forms.rs:275`) extended to scan all `FreeText.text`/resolved texts and panel titles across all sections. **Variables** — extend `TEMPLATE_VARIABLES` (14 → 22), `variable_label`, `sample_values` in the same style:
  | New key | Label (es) | Source (objective 4) | Sample value |
  |---|---|---|---|
  | `patient.hcl` | Historia clínica (HCL) | `patients.medical_record_no` | `722846-774928` |
  | `patient.allergies` | Alergias del paciente | `patients.allergies` | `Penicilina` |
  | `doctor.license_no` | Registro profesional (ACCESS) | `doctors.license_no` | `1803693264` |
  | `prescription.recetario_type` | Tipo de recetario | `prescriptions.recetario_type` uppercased | `AMBULATORIO` |
  | `prescription.validity` | Vigencia | `validity_days` → `"1 DIA"`/`"3 DIAS"` | `3 DIAS` |
  | `prescription.time` | Hora de emisión | `issued_at` time `HH:MM` (Ecuador UTC-5) | `13:19` |
  | `prescription.warning_signs` | Signos de alarma | `prescriptions.warning_signs` | `Fiebre mayor a 38.5 °C` |
  | `prescription.instructions` | Recomendaciones no farmacológicas | `prescriptions.instructions` | `Reposo e hidratación` |
  | `patient.age_detail` | Edad detallada | new `age_detail` (objective 4) | `20a 7m 11d` |
  Existing `patient.age` keeps its current `"{years} años"` value for migrated v1 templates; the new detailed age is a **separate** key `patient.age_detail` (22nd variable) used by `FieldKind::PatientAge`. Total `TEMPLATE_VARIABLES` = 22 entries. `canonical_variables()` in `service.rs` picks the list up automatically (it serializes `TEMPLATE_VARIABLES`). `MedicationBlock` (`forms.rs:74`) gains `quantity_number: Option<i32>` with `#[serde(default)]` (so old JSON parses). It mirrors `prescription_items.quantity_number` at render; template-editor sample medications may set it too; commands: `cargo check --manifest-path persona-sanus/Cargo.toml`
- [x] 4. Bind the new fields in data projection (`service.rs`) with computed age and Spanish number-to-words — requirements: extend `PrescriptionDocument` (`service.rs:3039`) with `patient_hcl: Option<String>`, `patient_allergies: Option<String>`, `doctor_license_no: Option<String>`, `recetario_type: String`, `validity_days: Option<i32>`, `warning_signs: Option<String>`, `instructions: Option<String>`; populate in `prescription_document()` (`service.rs:3314`) from the already-loaded `patient`/`doctor`/`prescription` rows (no new queries). `variables()` (`service.rs:3086`): add the 8 new keys (fallback `—` for missing, same pattern as existing); `prescription.recetario_type` renders uppercased (or `—` when empty, verified so missing values never re
- [x] 5. Replace the line layout with a geometric layout model and update both renderers (`forms.rs`, `pdf.rs`, `main.css`) — requirements: one shared layout function for preview + PDF (core invariant from specs 05-E/08-H: one projection, no second engine). Replace `DocumentPage { lines, images }` internals but keep `paginate_document` name/signature:
  ```rust
  pub struct DocumentLayout {
      pub primary: String,
      pub accent: String,
      pub page_width_mm: f32,
      pub page_height_mm: f32,
      pub margin_mm: u32,
      pub pages: Vec<LayoutPage>,
      pub images: Vec<ImageSlot>,
      pub logo_asset_id: Option<String>,
  }
  pub struct LayoutPage {
      pub panels: Vec<LayoutPanel>,
  }
  pub struct LayoutPanel {
      pub x_mm: f32,
      pub width_mm: f32,
      pub blocks: Vec<LayoutBlock>,
  }
  pub enum LayoutBlock {
      Text { lines: Vec<String>, size_pt: f32, bold: bool, color: TextColor },
      Rule,
      Table { header: Vec<String>, rows: Vec<TableRow>, col_weights: Vec<f32> },
      Image { slot_index: usize },
      Spacer { height_mm: f32 },
      Qr,
  }
  pub struct TableRow {
      pub cells: Vec<String>,
      pub span_all: bool,
      pub bold: bool,
  }
  pub enum TextColor { Primary, Accent, Ink, Muted }
  ```
  Page geometry: portrait → 210×297, 1 panel at `x = margin`, width `210 − 2·margin`; landscape → 297×210, 2 panels: gutter 6 mm; panel width `(297 − 2·margin − 6) / 2`; panel 0 x = margin, panel 1 x = margin + panel_width + 6; `paginate_document(document, values, margin_mm, medications, meta)` keeps its signature; geometry derives from `document.page`; `DocumentMeta` unchanged. Block production per section (all text via `resolve_text` + existing `wrap_text`, chars per line `= (width_mm / (1.95 * size_pt / 10.0)).floor().max(8)`): `Letterhead`: logo at the top-right of **each** panel that contains a `Letterhead` section, 18 mm box; doctor lines 13 pt bold Primary, clinic lines 8 pt Muted, then `Rule` in Primary (today: 14 pt/8.5 pt — panel scale is smaller; these exact sizes are the v2 spec). `Text` blocks gain `bold_prefix: Option<String>` — when set, the PDF renderer draws the prefix in bold then the rest regular on the same line, and CSS renders `<strong>{prefix}</strong>{rest}`; this is the only inline-emphasis mechanism, no other markup; used for `PatientFields` (prefix = `"{LABEL} "`, full line 8.5 pt Ink), `Observación:` sub-rows and section titles. `Validity`: one line, 8.5 pt: prefix `"VIGENCIA: "`, then `☒`/`☐` glyph + ` {label_1_day}  ` + glyph + ` {label_3_days}`, glyph = `☒` (U+2612) when `validity_days` matches, else `☐` (U+2610); Noto Sans includes both glyphs — verify once with the probe fixture text extraction; if a glyph is missing in the vendored font subset, fallback: draw `[X]`/`[ ]` text instead, and record which path was taken in a section 4 divergence. `Diagnosis`: title line bold 8.5 pt `"DIAGNÓSTICO:"`, diagnosis lines 8.5 pt (multiline preserved), then if `show_allergies`: `"ALERGIAS:"` bold + value (empty → blank line space reserved 2 lines). `FreeText`: optional bold title line, then wrapped text 9 pt. `MedicationTable Full`: `Table` block, header = `header_captions`, one row per medication: cell0 = `name` (`notes` **never** in Full cells); cell1 (Cantidad) = if `quantity_number` is `Some(n)` → `"{n} ({quantity_in_words(n)})"`, else the legacy free-text `quantity` verbatim, else `""`; remaining cells = dose/route/frequency/duration verbatim; `show_observations` on Full is **ignored** (left panel of photo has none). `MedicationTable Compact`: 4 cols (Medicamento/Dosis/Vía/Frecuencia); `show_observations: true` appends after each medication row a `span_all` row `"Observación: {notes}"` when notes non-empty. `IndicationList`: as defined in objective 3 (numbered first cell, optional `Observación:` span rows). Empty rule (all three table/list types): 0 medications → emit **no** Table block (and no panel title for it). `WarningSigns`/`Recommendations`: bold title + wrapped value 8.5 pt; empty value → title + one blank line (the printed blank area is the point). `PrescriberBlock`: bold title `"DATOS DEL PRESCRIPTOR:"`, field lines with `bold_prefix` (`"APELLIDOS Y NOMBRES: "` + doctor.name, `"NÚMERO DE REGISTRO(ACCESS): "` + license, `"NÚMERO DE CONTACTO-PRESCRIPTOR: "` + contact), then `Spacer 12mm` after FIRMA line, `Spacer 18mm` after SELLO line. `ReceptionBlock`: bold title `"RECEPCIÓN MEDICACIÓN DEL PACIENTE:"`, one Text line `"FIRMA: __________________    N° CÉDULA: __________________    PARENTESCO: __________________"` 8 pt. `SignatureQr`: existing footer content (Receta No./date line, verification/provisional line, `"Firma: ______________________________"`) as Text 7.5 pt Muted + `Qr` block (18 mm in panels, 25 mm when `panel_count == 1`). Panel `title`: emitted as bold 10 pt Primary Text block immediately before the first `MedicationTable`/`IndicationList` block of that panel (or as the first block if no table exists and title non-empty). `images` (document-level `ImageSlot`s): rendered on page 1 in the first panel after its `Letterhead` block at panel width × 29 mm + caption, as today. **Pagination**: Text line = `size_pt * 0.52 mm` per line; Table row height = max cell wrapped lines × 4.2 mm + 1.5 mm padding; header row likewise with 7 pt captions; Spacer = its height; Qr = its size + 2 mm; usable panel height = `page_height − 2·margin`; flow blocks top→bottom per panel; when a block doesn't fit, start a new page **with all panels repeated**: on continuation pages, only `Letterhead`, `PatientFields`, `Validity` sections re-emit (the "fixed" sections), then the overflowing flow continues; `SignatureQr` emits only on the last page, in every panel; tables split by row across pages (header row repeats on each continuation page for that table). This keeps the common HVQ case (≤ ~8 medications) on exactly 1 page, which the probe asserts. Delete `medication_line` and the old `DocumentPage.lines` path (clean cutover; `paginate_document` is the only layout path — grep for callers: `forms.rs` `A4Preview`, `pdf.rs:70`, `pdf_server.rs` via `PdfInput` — all updated in this step). **PDF renderer** (`pdf.rs`): `PAGE_WIDTH/PAGE_HEIGHT` become per-layout values from `DocumentLayout.page_width_mm/page_height_mm`; `PdfPage::new(Mm(w), Mm(h), ops)`; add a bold font: vendor `NotoSans-Bold.ttf` next to the existing regular (SIL OFL, same `LICENSE` file covers the family — add the file from the Noto Sans distribution; if the executing environment has no network, extract bold from any locally installed Noto/DejaVu bold TTF and rename — the invariant is: a real bold TTF embedded, file named `persona-sanus/assets/fonts/NotoSans-Bold.ttf`, `Fonts { regular, bold }` handles in `pdf.rs:42`); new primitives `push_rect` (stroked rectangle via 4 `DrawLine` or `Op::DrawRectangle` — confirm printpdf 0.12.9 API first; lines are guaranteed to exist since `push_rule` uses them), `push_table` (cell borders 0.4 pt `#111111`, header bold 6.5 pt centered, body 8 pt, cell text wrapped with the same char heuristic, row heights from layout); checkbox glyphs are plain text; `render_page` iterates `LayoutPage.panels` with absolute x offsets; QR, logo, images reuse existing `push_image`; provisional mode unchanged semantically (placeholder QR frame + `PENDIENTE DE SINCRONIZACIÓN`). **Preview** (`A4Preview`, `forms.rs:472` + `assets/main.css:210-334`): render the same `DocumentLayout`: `.a4-page` gets `width/height` from layout mm (landscape → `297mm × 210mm`, class modifier `.a4-page--landscape`), panels as flex row with the layout gutter, blocks as HTML: Text → `<p>` (with `<strong>` for `bold_prefix`), Table → real `<table>` with `col_weights` as `<colgroup>` percentages, Qr → existing `.a4-qr` at 18/25 mm; replace `.a4-header/.a4-body/.a4-footer` internals but keep class names where sensible; add `.a4-panels`, `.a4-panel`, `.a4-table` (borders `0.4pt solid`, header centered bold 6.5 pt, cells 8 pt, `border-collapse: collapse`), `.a4-panel-title`; font sizes in CSS use pt to mirror PDF; when `medications` prop is `None`, use `document.medications`; add `sample_medications()` in `forms.rs` returning the 4 photo medications: Betametasona+Ácido Fusídico (Fusimed B) 1 aplicaciones/vía tópica/cada 12 horas/30 días/qty 1/notes "Aplicar poca cantidad en el area afectada 8am y 8pm"; Glicolic H Loción 1 aplicaciones/vía tópica/cada 24 horas/30 días/qty 1/notes "Aplicar en la mancha del rostro todas las noches"; Prednisona 20mg/sólido oral/tabletas (Pharmab) 20 miligramos/vía oral/cada 12 horas/30 días/qty 40/notes "2 TABLETAS JUNTAS 7AM DESPUES DEL DESAYUNO 5 DIAS, LUEGO 1 1/2 TABLETAS LUEGO SOLO 1 TABLETA POR 5 DIAS LUEGO MANTENER TRES VECES"; Gluconato de Zinc 50mg 1 tableta/vía oral/cada 24 horas/90 días/qty 90/notes "1 tableta despues de la merienda" — used by `/templates/:id/preview` when the document has ≥1 table/list section; keep old behaviour otherwise; commands: `cargo check --manifest-path persona-sanus/Cargo.toml` and `cd persona-sanus && dx build --release --fullstack`
- [x] 6. Build the editor UI plus HVQ preset plus prescription/patient form fields (`forms.rs` editor, `main.rs`) — requirements: **Preset** — `pub fn hvq_preset_document() -> TemplateDocument` in `forms.rs`, exactly: `page: PageSetup { orientation: Landscape, panel_count: 2 }`; `letterhead`: `logo_asset_id: None`, `doctor_details: "{{doctor.name}}\n{{doctor.specialty}}"`, `clinic_details:
- [x] 7. Extend probes and fixtures (`.blocks/`, `fixtures/`) — requirements: extend `.blocks/pdf-probe.py`: it currently asserts A4 portrait MediaBox 595.28×841.89 (header :4-11, constants :36-37); add param `--hvq` (bool flag): when set, expect landscape MediaBox (841.89×595.28 ±1 pt), and assert extracted PDF text contains: `MEDICACIÓN`, `INDICACIONES`, `CANTIDAD (EN`, `40 (CUARENTA)`, `Observación:`, `VIGENCIA`, `☒` or `[X]` (whichever objective 5 shipped), `DATOS DEL PRESCRIPTOR`, `RECEPCIÓN`; reuse whatever the block already does for QR/filename (it parses PDF streams with `zlib` — extend that path; if the block does **not** extract text, add a minimal FlateDecode stream text extractor inside the block…

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No rich-text/WYSIWYG/contenteditable/HTML/CSS template authoring; no new editor dependency (D-21). No Typst/Handlebars/Tera/headless-Chromium/second layout engine; `printpdf` stays the only PDF engine and `paginate_document` the only layout function.
- No structured CIE-10 table, no forma farmacéutica/concentración item columns, no digital reception capture, no signature/stamp image columns (blank printed areas, as in the photo).
- No change to template versioning semantics (published = immutable, pinning, asset pinning), auth/roles, QR payload (no PHI), PDF filename rule, or `variables_spec` canonical generation.
- No removing/renaming existing API fields or the free-text `prescriptions.validity` / `prescription_items.quantity` columns (legacy fallback).
- No comments in code; no unit tests; no formatter/linter tail beyond what `persona-smoke` already runs.
- No rich-text, WYSIWYG, or contenteditable editor (PRD D-21): all text entry is plain forms and inputs (`src/forms.rs` — single-line inputs, multi-line textareas, selects, check controls; plain-text values only), identical online and offline. No editor dependency, no HTML payloads, unless the spec's Source explicitly supersedes D-21.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done. Prerequisites: server built and running from `persona-sanus/` (`cargo run` / existing supervised setup), `PUBLIC_BASE_URL` unset (defaults `http://127.0.0.1:8080` per `pdf_server.rs:164`). Run from repo root unless stated.*

- [x] `cargo check --manifest-path persona-sanus/Cargo.toml` exits 0 (run from the repo root)
- [x] `cd persona-sanus && dx build --release --fullstack` exits 0 (web + server compile the shared `forms.rs`)
- [x] migration: fresh DB boot (delete scratch DB, start server per `persona-smoke --db-path` convention) → server reaches health 200 (`wait-health --url http://127.0.0.1:8080/…` using the health path persona-smoke uses — confirm path from that block); `sqlite-schema-check --db <scratch.db> --query "SELECT name FROM pragma_table_info('patients') WHERE name IN ('medical_record_no','allergies')"` → both rows; same for `prescriptions(validity_days, warning_signs, recetario_type)` and `prescription_items(quantity_number)`
- [x] existing behaviour preserved: `python3 .blocks/pdf-probe.py --base http://127.0.0.1:8080` → `ok: true` (portrait golden flow, QR payload, filename, draft 422 unchanged)
- [x] existing behaviour preserved: `python3 .blocks/persona-probe.py --base http://127.0.0.1:8080 --flow templates` → all checks true (v1-created default template still saves/publishes/immutable — it now round-trips through v1→v2 migration on parse)
- [x] HVQ end-to-end: on 2026-11-27, `python3 .blocks/pdf-probe.py --base http://127.0.0.1:18085 --hvq` passed on a fresh database, including landscape dimensions, HCL, doctor license, quantity, observations, validity, prescriber and reception assertions. The composed HVQ fixture uses two medications; the template preview uses four.
- [x] Preview parity: on 2026-11-27, the four-medication template preview and two-medication signed preview each showed one landscape page with both panels. The downloaded signed PDF showed the same one-page layout and tables. Browser screenshots: template `/var/folders/x2/nk8wfy812qg3mx9l92gxn__h0000gn/T/orca-browser-screenshots/4237884d-f240-4ea1-abfc-a66df00f01f4-screenshot.png`, signed preview `/var/folders/x2/nk8wfy812qg3mx9l92gxn__h0000gn/T/orca-browser-screenshots/f59c3972-c52a-4d4b-a234-9ee4558b22b6-screenshot.png`, PDF `/var/folders/x2/nk8wfy812qg3mx9l92gxn__h0000gn/T/orca-browser-screenshots/b6583b9f-157f-420a-9942-ebe0d427c0f7-screenshot.png` (temporary browser captures).
- [x] Offline/desktop: `caveman-blocks run persona-smoke --dir persona-sanus` passed on 2026-11-27. On the same date, an isolated desktop release build cached a two-medication HVQ fixture, then displayed its preview with the server stopped. The provisional PDF action saved a one-page A4 landscape PDF. Text extraction confirmed the cached HCL, allergies, doctor license, quantity, validity, warning signs, recommendations, observations, and provisional marker.
- [x] every objective in section 1 is checked

## 4. Divergences

*Process and tracking-file rules: see `AGENTS.md` (§ Spec-Driven Development).
Summary: when reality diverges from the spec, append a numbered entry below;
leave the matching testing-criteria `- [ ]` unchecked and note the divergence
inline next to it. Open divergences owned by a later spec also get a tracking
file in `specs/divergences/`.*

- [x] every divergence found is appended below (or the section reads "None.")

*Entry format (one numbered item per divergence):*

1. <what diverged and its current state> — <why it could not be done as written, citing the No-go rule or spec that blocks it> (Verified <date>: <N> fix attempts — <attempt 1>, <attempt 2>, <attempt 3> — all fenced by <constraint>; <impact> is deferred to <later spec> by design, not a defect.)

### Divergences

*Fill in at execution time. Generic example (same slots as the entry format
above):*

1. Spec ID is `38-AL` not `37-AK` as the plan's Decision 3 states — `specs/37-AK-icons-crate-integration.md` already exists (created 2026-11-23, still has an open objective 7 divergence), so the next free number `38` and next phase letter `AL` are used per `specs/TEMPLATE.md` sequential-numbering rule. — The plan's "next free spec number is 37" was true when the plan was written but is no longer true; overwriting `37-AK` would destroy an in-flight spec. (Creation-time divergence, verified 2026-11-23 against the `specs/` listing; no implementation impact — all `37-AK` references in objective 7's delivery text and the plan's Verification §7 mean this file, `38-AL`.)
2. `persona-smoke` health stage fails in this sandbox: `persona-smoke` reports `fmt/check/clippy/build` all `true` but `health: false` ("server never healthy") — the smoke block spawns a child server that cannot bind in this environment. The identical release binary started manually on the same scratch DB serves `/health` 200 with `foreign_keys: true, journal_mode: wal`. — Environment constraint, not a code defect; the manual run is the substitute evidence. (Verified 2026-11-23: 1 fix attempt — running the release binary by hand with the same env/DB persona-smoke uses — fenced by sandbox process rules; criteria box "offline/desktop" stays unchecked for this reason.)
3. Desktop `cargo check --no-default-features --features desktop` cannot run here: the desktop feature graph pulls `openssl-sys 0.9.118`, whose build script needs system OpenSSL headers (`/usr/include/openssl` absent, `pkg-config openssl` not found). `cargo check`, `cargo clippy -- -D warnings`, `dx build --release --fullstack` all pass, so the shared `forms.rs`/`pdf.rs` compile for web+server; the desktop compile is unverified in this sandbox. — Environment constraint, not a code defect. (Verified 2026-11-23: 1 fix attempt — desktop check — fenced by missing system headers; desktop binary verification deferred to a machine with OpenSSL dev files; criteria box "offline/desktop" stays unchecked.)
4. `persona-probe --flow rx` `reorder_items` failed before this spec but was not caused by it: creating a draft seeds prescription items from the template's sample `medications`, so the probe's reorder of only its 2 added items returns 400 "El orden debe incluir todos los medicamentos." At HEAD the seeding loop and strict reorder check are identical code; only the probe needed fixing. Fixed within objective 7's Owned files: the probe now fetches the draft's current items and reorders all of them. (Verified 2026-11-23: debug run showed reorder 400 with 3 items present; probe fix makes `pdf-probe` portrait `ok: true`.)
5. `persona-probe --flow hvq` used the fixed `nationalId: 1724122799` for every run, so a second run on the same DB failed `create_patient` with `DuplicateNationalId`. Fixed within objective 7's Owned files: the HVQ patient now uses a per-run unique `nationalId` (`f"9{run[:9]}"`, same pattern as other flows). The HCL/allergies assertions are unchanged. (Verified 2026-11-23: fresh-DB HVQ probe `ok: true`.)
6. Initial preview parity evidence was unavailable. Resolved on 2026-11-27 by the browser preview and PDF comparison recorded in section 3.
7. The original path `.orca/drops/template copy.jpg` was absent. On 2026-11-27, the scope-change approver supplied `context/references/prescription-model.jpeg` as the reference image, resolving the missing-reference blocker.
8. `persona-smoke` initially failed its child-server health stage. On 2026-11-27, after the authorized formatting correction, `caveman-blocks run persona-smoke --dir persona-sanus` passed every stage, including health and auth probes. Desktop compilation also passed; provisional rendering remains unverified under divergence 12.
9. Safari login automation initially failed. Resolved on 2026-11-27 using the isolated browser session and visual checks recorded in section 3.
10. On 2026-11-27, Chromium initially showed two pages and a portrait-dimensions subtitle for the HVQ template and signed previews. The approver authorized compact layout; pagination and subtitle were corrected. Subsequent browser checks showed one landscape page for the four-medication template and signed prescription, and the signed PDF also had one page.
11. On 2026-11-27, the scope-change approver authorized compacting the prescribed layout where necessary to satisfy the four-medication single-page requirement, and adding `.blocks/persona-smoke.py` to the owned files if needed to resolve divergence 8. Exact dimensions and blank-space requirements yield to the single-page requirement where they conflict.
12. Offline prescription preview previously supplied only `patient.name` as a template variable. The approver authorized extending the projection and local data plumbing; available HVQ fields are now projected, with doctor details only when the cached doctor matches the prescription. Resolved on 2026-11-27 by the isolated offline desktop preview and provisional PDF verification recorded in section 3.
13. The initial pagination update produced a PDF with most text invisible because text positions were relative; after absolute positioning, overlapping table headers exposed a wrapping defect. Corrected positioning and shared wrapping were verified visually on 2026-11-27; the signed PDF displayed both tables and text on one page. The HVQ and portrait PDF probes passed on separate fresh databases.
14. The initial HVQ probe omitted the required doctor license and did not assert it. The fixture now sets `1803693264`; the HVQ PDF probe asserts that license and HCL. Both checks passed on a fresh database on 2026-11-27.
15. The desktop provisional PDF button discarded the rendered bytes. On 2026-11-27, the approver authorized exposing the PDF and an isolated desktop launch for verification. The action now saves the PDF in the protected desktop data directory and displays its path. An isolated desktop bundle and database verified the offline preview and saved PDF as recorded in section 3.

1. `<check>` stays failing in `<workspace>` on the `<feature>` feature only:
   the `<work added by objective N>` lands, but the `<code it calls>` is
   explicitly excluded (section 2, spec <NN-Y>). (Verified <date>: <N> fix
   attempts — <attempt 1>, <attempt 2>, <attempt 3> — all fenced by
   <constraint>; `<what stays failing>` is deferred to spec <NN-Y> by design,
   not a defect.)

---

*Housekeeping: once a spec is fully done with no open divergences, move its
file to `specs/done/` (same file name, e.g.
`specs/done/38-AL-hvq-template-v2.md`).
Specs with open divergences stay in `specs/` next to `TEMPLATE.md` until the
tracking file is cleared.*

## 5. Context and anchors (from the plan; non-normative — objectives are authoritative)

The attached photo (`.orca/drops/template copy.jpg`, 4096×3072) shows a Hospital Vozandes Quito (HVQ) prescription sheet: **one A4 landscape sheet, two panels side by side**. Both panels repeat the same institutional header (HVQ logo, `HOSPITAL VOZANDES QUITO HVQ S.A. / Villalengua Oe237 y Av. 10 de Agosto / Telf.: (02) 4007 - 100`, form code `5-189113` on the left panel) and the same patient/prescriber data. They differ below the header:

| Zone | Left panel | Right panel |
|---|---|---|
| Title/Section | `MEDICACIÓN:` | `INDICACIONES:` |
| Table columns (6 vs 4) | Datos del medicamento (DCI, forma farmacéutica y concentración/dispositivo médico) · Cantidad (en números y letras) · Dosis/Posología · Vía de administración · Frecuencia de la administración · Duración del tratamiento | Denominación Común Internacional (DCI)-Medicamento · Dosis/Posología · Vía de administración · Frecuencia de la administración |
| Rows | one row/medication; quantity rendered `1 (UNO)`, `40 (CUARENTA)`, `90 (NOVENTA)` | numbered `1)…4)`; each medication followed by a wrapped `Observación: …` sub-row (current `notes`) spanning the table |
| Extra fields above table | `DIAGNÓSTICO:` (multi-line, each with CIE-10 e.g. `ALOPECIA AREATA (CIE-10: L63)`), `ALERGIAS:` | `SIGNOS DE ALARMA:`, `RECOMENDACIONES NO FARMACOLÓGICAS:` |
| Footer | `RECEPCIÓN MEDICACIÓN DEL PACIENTE: FIRMA ___ N° CÉDULA: ___ PARENTESCO: ___` + prescriber block | prescriber block only |
| Prescriber block (both) | `DATOS DEL PRESCRIPTOR: APELLIDOS Y NOMBRES: <doctor name>`, `FIRMA:` blank line, `NÚMERO DE REGISTRO(ACCESS): <license_no>`, `NÚMERO DE CONTACTO-PRESCRIPTOR: <phone>`, `SELLO MÉDICO:` blank area (signature/stamp are hand-applied in the photo — print leaves them blank) | same |
| Shared header rows (both) | `RECETARIO: AMBULATORIO` · `VIGENCIA: ☐ 1 DIA ☒ 3 DIAS` (checkbox) · `FECHA: Quito, 26 (DIA(DD)) 10 (MES(MM)) 2026 (AÑO(AAAA))` · `HORA: 13:19` · `CI: 1724122799` · `HCL: 722846-774928` · `APELLIDOS Y NOMBRES: <patient>` · `EDAD: 20a 7m 11d` | same |

**Current system cannot recreate this.** Verified in `persona-sanus/src/forms.rs`, `pdf.rs`, `service.rs`, `schema.rs`:

- Document model is `TemplateDocument { schema_version: u32 (=1), letterhead, body: String, footer: String, diagnosis: String, images: Vec<ImageSlot>, medications: Vec<MedicationBlock> }` with `#[serde(deny_unknown_fields)]` (`forms.rs:13-45`). Free-text only; `resolve_text` (`forms.rs:243`) supports exactly the 14 whitelisted `{{variables}}` in `TEMPLATE_VARIABLES` (`forms.rs:15-30`), no loops/conditionals/per-medication fields.
- Layout is a single-column **portrait** A4 (210×297 mm hard-coded, `pdf.rs:14-15`, `forms.rs:386` `210 - 2*margin`) line flow: `paginate_document` (`forms.rs:378`) wraps text at `content_width/1.95` chars, medications flatten to one bullet line via `medication_line` (`forms.rs:291`). No tables, panels, columns, checkboxes, bold, or landscape anywhere. `A4Preview` (`forms.rs:472`) + `.a4-*` CSS (`assets/main.css:210-334`, portrait flex) and `pdf.rs:render_page` (`pdf.rs:254`, `printpdf` ops, single `NotoSans-Regular.ttf`, regular weight only) are the two renderers sharing that line layout.
- Template storage/versioning already exists and is reused unchanged: `templates`/`template_versions`/`template_version_assets` (`schema.rs:137-171`, migration `20261112000000_create_app_schema/up.sql:110-142`), immutable published versions, pinning at prescription creation, publish pins assets (`service.rs:1212-1664`, `api_server.rs:1029-1145`, routes `/api/templates*`). Only `margin_mm` lives in `theme` (`forms.rs:1117`).
- Data gaps vs the photo (field-by-field scout verified): no `patients.medical_record_no` (HCL), no `patients.allergies`, no `prescriptions.recetario_type`, `prescriptions.validity` is free text (no 1/3-day enum) and is **not** carried by `PrescriptionDocument` (`service.rs:3039-3062`), no `prescriptions.warning_signs`, `prescriptions.instructions` exists but never reaches `PrescriptionDocument`/variables, `doctors.license_no` exists (`db.rs:289`) but is not in `PrescriptionDocument` and has no template variable, `prescription_items.quantity` is free text (no Spanish number-to-words anywhere), `patient_age()` (`service.rs:3073`) returns years only (photo needs `20a 7m 11d`), no `prescription.time` variable, no structured CIE-10 (diagnosis is one free-text string — multiline text reproduces the photo exactly, so no CIE-10 schema is added), no forma farmacéutica/concentración split (photo itself embeds it in the medication name's second line — kept as free text in `med_name`), no signature/stamp columns (photo shows hand-applied signature/stamp → print leaves blank spaces; QR + verification code remain the system's authenticity mechanism), reception block is blank on paper (static template lines, no reception table).

**Decisions locked (user-selected, do not re-ask):**

1. **Scope = typed composable sections.** v2 document is a page setup + an ordered list of typed sections placed into 1 panel (portrait, current behaviour) or 2 panels (landscape, HVQ). Sections are fixed types with typed fields, edited with plain forms only (PRD D-21: no rich text/WYSIWYG/contenteditable/HTML). Future hospital forms are composed from the same section types without new Rust code. A free-form grid designer and a hard-coded HVQ-only layout were rejected.
2. **Data model is extended** with the missing clinical fields (migration + `schema.rs` + `desktop_local.rs` mirrors + API/service/UI plumbing), instead of overloading existing free-text fields.
3. **Deliverable = one new spec file** (this file) created from `specs/TEMPLATE.md`, then implemented. The plan targeted `37-AK`; `38-AL` is used instead because `37-AK` already exists (see section 4 divergence 1).
4. **Compatibility = auto-migrate v1→v2 on parse.** `parse_document` accepts `schema_version: 1` JSON and converts it to the v2 section model; saving always writes v2. Published pinned v1 versions therefore keep rendering (through the migrated model) — no dual engine, no DB data migration for `doc_json`.

### Critical files & anchors

- `persona-sanus/src/forms.rs` — `SCHEMA_VERSION`/`TEMPLATE_VARIABLES` (:13-30), `TemplateDocument`/`MedicationBlock` (:34-82), `parse_document`/`validate_document` (:112-186), `paginate_document` (:378), `A4Preview` (:472), `TemplateDocumentForm` (:653), `template_theme_margin` (:1117). The entire v2 model, migration, layout, editor, preview live here.
- `persona-sanus/src/pdf.rs` — `PAGE_WIDTH/PAGE_HEIGHT` (:14-15), `PdfInput`/`render` (:30-52), `render_page` (:254). Landscape/panel/table/bold rendering; printpdf 0.12.9 pinned in `Cargo.toml:27,42,71`.

- `persona-sanus/src/service.rs` — `PrescriptionDocument`/`patient_age`/`variables`/`medications` (:3039-3178), `prescription_document()` (:3314), template CRUD (:1212-1664), Input structs (:2195-2219). Data binding for every new variable.
- `persona-sanus/src/schema.rs` + `persona-sanus/src/desktop_local.rs` (:28-140 DDL) + new migration dir — column mirrors; desktop sync payload keys.
- `persona-sanus/assets/main.css` (:210-334) — `.a4-*` preview parity for landscape/panels/tables.

### Assumptions & contingencies

- **Bold font availability**: objective 5 requires a real bold TTF at `persona-sanus/assets/fonts/NotoSans-Bold.ttf`. If the execution sandbox has no network and no system bold font to copy, fallback: render "bold" blocks with the regular font at +0.5 pt size and record a Divergence in the spec (visual weight differs from photo; layout/text identical). Do not synthesize bold by double-striking text.
- **Checkbox glyphs**: if U+2610/U+2612 are absent from the vendored Noto Sans (probe text extraction shows missing-glyph boxes or empty text), fallback: render `[ ]` / `[X]` ASCII in both PDF and preview, and note it in the spec. The probe accepts either (objective 7).
- **printpdf rectangle API**: if printpdf 0.12.9 lacks a rectangle op in the vendored source, tables/boxes are drawn with `DrawLine` segments (already used by `push_rule`) — no API risk, only more ops.
- **`desktop_local.rs` schema evolution**: if existing desktop DBs cannot take `ALTER TABLE` in its current init flow, fallback: bump nothing; add columns via `CREATE TABLE IF NOT EXISTS` for new DBs and treat missing columns on old desktop DBs as `NULL` (desktop reads use explicit column lists — if a column is missing SQLite errors, so the ALTER-with-duplicate-column-tolerance path is required; if neither works, record a Divergence and gate desktop HVQ rendering to online-created DBs).
- **First-version seeding location** (objective 6): if the first draft's `doc_json` is seeded client-side instead of server-side, implement `preset` selection entirely in `main.rs`/`api.rs` (create template → create version with preset JSON) and drop the `TemplateCreateRequest.preset` field. Either way the API accepts the preset through exactly one path; do not build both.
- **Panel continuation**: HVQ prescriptions with many medications (> ~10) will paginate to a second landscape sheet with repeated fixed sections and the QR footer only on the last page. This is accepted behaviour (photo shows the single-page common case); the probe asserts the 4-medication case stays on 1 page.
- **Quantity words scope**: quantities above 9999 or non-numeric legacy text render the raw `quantity` string (objective 5 rule). Accepted; the numeric field is the HVQ path.
