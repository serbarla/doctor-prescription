# Spec 16 — Phase P: UI forms, feedback, and layout

- Source: `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` steps 4–5; `context/Design.md` §§Components, Tokens, Surfaces, Elevation, Layout, Agent Prompt Guide.
- Depends on: 15-O.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Result carried forward: `persona-sanus/src/ui/` exposes six plain controlled form controls with a shared field shell, plus badges, notices, error and hint text, empty and loading states, confirm panels, data tables, list rows, page shells and headers, cards, form and detail grids, action rows, filter bars, and section bands; their CSS lives in `assets/main.css`; both web and desktop targets compile.

## Before you start

Unit testing is not required; avoid it. Prioritize end-to-end and integration testing only.

Read the nearest `AGENTS.md` (target workspace, then parents up to repo root) before executing.

If reality diverges from this spec: STOP, append a divergence (section 4), and NEVER improvise outside the objectives.

This spec consumes only `15-O` primitives (`Icon`, `Button`, `ButtonVariant`, `IconName`, tokens, `.ui-page`, `.ui-container`, `.ui-stack`, and format helpers). It does not adopt a page. Visual proof of every state arrives in `19-S`.

## 1. Objectives

*The ordered work to implement — one item per step. Check `[x]` only after the step is implemented and verified.*

*Item format:* `<verb phrase>` — requirements: `<what must change / constraints>`; commands: `<exact command>`.

- [x] Add the form controls and field shell — requirements: create `persona-sanus/src/ui/form.rs` and implement `TextField`, `TextAreaField`, `SelectField`, `CheckboxField`, `FileField`, `SearchField`, and the private `Field` wrapper exactly as section 7 contracts. Labels wrap their controls; no generated ids. Inputs are controlled. `error: Some(_)` renders `<p class="ui-field-error" role="alert">` and suppresses the hint; hint-only renders `.ui-field-hint`. Text, area, and select set `aria-invalid` from `error.is_some()`. The focus utility from `15-O` applies to every control. Add the form declarations in section 9 to `assets/main.css`. No toolbar, no `contenteditable`, and no rich text; commands: `cd persona-sanus && cargo check`; `rg -n 'contenteditable|execCommand|innerHTML' persona-sanus/src`.
- [x] Add feedback and layout primitives — requirements: create `persona-sanus/src/ui/feedback.rs` and `layout.rs` and implement the symbols in section 7 with their exact props and markup. Cards, badges, notices, tables, and rows share Paper, 1px hairline, and 24px radius where contracted. Buttons and cards have no shadow. The spinner is the only animation and `prefers-reduced-motion: reduce` disables it. Re-export both modules from `ui/mod.rs`. Add the declarations in section 9; commands: `cd persona-sanus && cargo check`; `rg -n -P 'box-shadow:\s*(?!none\b|0 0 0 3px var\(--color-mint-halo\)|var\(--shadow-paper\)|0 2px 12px rgba\(0, 0, 0\.18\))' persona-sanus/src/ui persona-sanus/assets/main.css`.

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No new crate, CSS framework, external stylesheet, webfont, CDN asset, icon package, or JavaScript bridge.
- No rich text, WYSIWYG, `contenteditable`, formatting toolbar, `execCommand`, HTML payload, or editor dependency. Every control is a plain controlled Dioxus input, textarea, select, or check.
- No new product route, API handler, server function, Diesel model, table, migration, entitlement rule, or billing persistence.
- No change to authorization, session, device registration, sync, numbering, signing, PDF, QR, verification limits, or business validation.
- No page adoption in this spec. Do not edit page components in `main.rs` or `forms.rs` beyond the module declaration if one is required.
- No generated `id` attributes for field labels. Wrap the control in the label instead.
- No animations except the loading spinner. Do not add button, card, or hover transitions.
- No drop shadow on cards or buttons. No pure black. No chromatic card border. No second styling system.
- No rich-text editor, signature pad, stamp image control, or patient photo control (PRD D-06, D-21).
- No unit tests and no comments in Rust, CSS, or RSX.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [x] `cd persona-sanus && cargo check` exits 0.
- [x] `cd persona-sanus && cargo check --no-default-features --features desktop` exits 0.
- [x] objective 1: `rg -n 'contenteditable|execCommand|innerHTML' persona-sanus/src` returns no matches.
- [x] objective 1: `rg -n 'fn (TextField|TextAreaField|SelectField|CheckboxField|FileField|SearchField)' persona-sanus/src/ui/form.rs` returns 6 matches.
- [x] objective 1: `rg -n 'ui-field-error|ui-field-hint|aria_invalid|accent-color|file-selector-button|webkit-file-upload-button' persona-sanus/src/ui/form.rs persona-sanus/assets/main.css` returns every contracted hook.
- [x] objective 2: `rg -n 'fn (Badge|PillBadge|Notice|ErrorText|HintText|EmptyState|LoadingState|ConfirmPanel|DataTable|ListRow|PageShell|PageHeader|Card|SectionCard|FormGrid|DetailGrid|DetailItem|ActionRow|FilterBar|SectionBand)' persona-sanus/src/ui/feedback.rs persona-sanus/src/ui/layout.rs` returns 20 matches.
- [x] objective 2: `rg -n -P 'box-shadow:\s*(?!none\b|0 0 0 3px var\(--color-mint-halo\)|var\(--shadow-paper\)|0 2px 12px rgba\(0, 0, 0, 0\.18\))' persona-sanus/src/ui persona-sanus/assets/main.css` returns the one allowed pre-existing `.a4-page` literal (line 688) and nothing else. The focus ring uses the allowed contract value.
- [x] objective 2: `rg -n 'prefers-reduced-motion|ui-spin|ui-spinner' persona-sanus/assets/main.css` returns the spinner animation and the reduced-motion override.
- [x] every objective in section 1 is checked.
- [x] every divergence found is appended to section 4 below.

## 4. Divergences

*Process and tracking-file rules: see `AGENTS.md` (§ Spec-Driven Development).
Summary: when reality diverges from the spec, append a numbered entry below;
leave the matching testing-criteria `- [ ]` unchecked and note the divergence
inline next to it. Open divergences owned by a later spec also get a tracking
file in `specs/divergences/`.*

- [x] every divergence found is appended below (or the section reads "None.")

*Entry format (one numbered item per divergence):*

1. <what diverged and its current state> — <why it could not be done as
   written, citing the No-go rule or spec that blocks it> (Verified <date>:
   <N> fix attempts — <attempt 1>, <attempt 2>, <attempt 3> — all fenced by
   <constraint>; <impact> is deferred to <later spec> by design, not a defect.)

### Divergences

None.

## 5. Context

`15-O` added the tokens, layout utilities, actions, icons, and formatting helpers. Every page still renders its raw `.field`, `.form-grid`, `.button`, `.badge`, `.notice`, `.error`, `.hint`, `.page-head`, `.actions`, `.detail-grid`, `.item-card`, `.patient-row`, and `.template-row` markup. This spec supplies the typed replacements for the controls and surfaces those pages will adopt in `20-T` through `24-X`.

The field shell is private. No page may compose raw `.ui-field-label`, `.ui-input`, or `.ui-field-error` markup once this spec lands; it calls `TextField`, `TextAreaField`, `SelectField`, `CheckboxField`, `FileField`, or `SearchField`. A page that needs a different control must use the nearest primitive or stop with a divergence.

`15-O` owns `.ui-page`, `.ui-container`, `.ui-stack`, `.ui-button`, `.ui-button--*`, `.ui-nav-link`, `.ui-visually-hidden`, `.ui-mono`, `.ui-break`, `.ui-notes`, and the `.ui-text-*` utilities. This spec does not redefine them.

## 6. Reuse, do not rebuild

| Existing symbol | Decision |
|---|---|
| `forms::A4Preview` | Keep. This spec does not duplicate, wrap, or restyle its layout or pagination engine. |
| `forms::TemplateDocumentForm` | Keep. `22-V` restyles its internals with the controls added here; this spec does not recreate document state, validation, or persistence. |
| `Navbar` | Keep. `20-T` replaces only its markup and styles. |
| `Route`, `App`, page components | Keep. No route changes here; `19-S` adds `DevUi`. |
| `forms::paginate_document`, `resolve_text`, document warnings | Keep unchanged. |
| `SyncStatus`, `probe_sync_status`, desktop count helpers | Keep in place for now. `18-R` moves `SyncStatus`; `probe_sync_status` and the desktop counts stay in `main.rs`. |
| `format_*`, label helpers | Consume the `15-O` exports; never copy them back into a page or component. |
| `.a4-*` CSS | Keep unchanged; no rule in this spec may match or override `.a4-*`. |

There is one styling system. Extend `assets/main.css` and the new `src/ui` modules only. Do not create a second component stylesheet.

## 7. Component surface

All symbols live in `persona-sanus/src/ui/` and are re-exported from `src/ui/mod.rs`. Every enum derives `Clone, Copy, PartialEq, Eq`. Every interactive control uses the `15-O` focus ring and carries no drop shadow. Class suffixes are lowercase.

### Shared enums and defaults

- `SurfaceTone`: `Paper`, `Cream`, `Mint`, `Blossom`. Default `Paper`. Class suffix `paper`, `cream`, `mint`, `blossom`.
- `FormColumns`: `Two`, `One`. Default `Two`. Class suffix `two`, `one`.
- `BadgeTone`: `Neutral`, `Mint`, `Cream`, `Blossom`, `Plum`. Default `Neutral`. Class suffix `neutral`, `mint`, `cream`, `blossom`, `plum`.
- `NoticeTone`: `Info`, `Success`, `Warning`, `Error`. Default `Info`. Class suffix `info`, `success`, `warning`, `error`.

### Form controls

Fields marked `#[props(default)]` are optional; unmarked fields are required.

`Field` is private and owns the label, control, hint, and error shell plus the `wide` modifier. It renders `div.ui-field` with `.ui-field--wide` when `wide` is true. The label span is `.ui-field-label`. The text, area, select, file, and search controls compose `Field`; the checkbox does not.

- `TextField`: `label: String`, `value: String`, `oninput: EventHandler<String>`, `input_type: String = "text"`, `placeholder: String = ""`, `#[props(default)] wide: bool`, `hint: Option<String>`, `error: Option<String>`, `autocomplete: String = "off"`, `spellcheck: bool = true`, `#[props(default)] required: bool`, `#[props(default)] disabled: bool`, `#[props(default)] readonly: bool`, `min: Option<String>`, `max: Option<String>`, `step: Option<String>`, `#[props(default)] hide_label: bool`. Renders `label.ui-field` wrapping `span.ui-field-label` and `input.ui-input`. Labels wrap controls, so no generated ids are needed. `oninput` calls `event.value()`. The input is always controlled by `value`.
- `TextAreaField`: `label`, `value`, `oninput`, `placeholder`, `#[props(default)] wide`, `hint`, `error`, `#[props(default)] required`, `#[props(default)] disabled`, `#[props(default)] readonly`, `#[props(default)] hide_label`, and `rows: Option<u32>`. Renders `textarea.ui-textarea` in the same field shell. The textarea is always controlled.
- `SelectField`: `label`, `value`, `onchange: EventHandler<String>`, `#[props(default)] wide`, `hint`, `error`, `#[props(default)] disabled`, `#[props(default)] hide_label`, and `children` (the options). Renders `select.ui-select` in the same field shell. The select is controlled by `value`; `onchange` passes `event.value()`.
- `CheckboxField`: `label: String`, `description: Option<String>`, `checked: bool`, `onchange: EventHandler<bool>`, `#[props(default)] disabled: bool`, `#[props(default)] wide: bool`. Renders `label.ui-check` with a native checkbox, label text, and optional `.ui-check-description`. Adds `.ui-check--wide` when `wide`. Native checkbox is 16px with `accent-color: var(--color-deep-teal)`. `onchange` passes `event.checked()`.
- `FileField`: `label: String`, `accept: String`, `#[props(default)] disabled: bool`, `hint: Option<String>`, `on_change: EventHandler<FormEvent>`, `#[props(default)] wide: bool`. Renders `label.ui-field` wrapping `span.ui-field-label`, `input[type=file].ui-file-input`, and optional `.ui-field-hint`. The label wraps the control; it forwards the whole `FormEvent` unchanged. No controlled `value`.
- `SearchField`: `label`, `value`, `oninput: EventHandler<String>`, `placeholder: String`, `#[props(default)] wide: bool`. Renders `label.ui-field` with a visually hidden `.ui-field-label` plus `input[type=search].ui-input`. `oninput` passes `event.value()`.

Every text, area, or select field with `error: Some(_)` renders `<p class="ui-field-error" role="alert">{error}</p>` and no hint. A field with only `hint` renders `.ui-field-hint`. `TextField`, `TextAreaField`, and `SelectField` set `aria-invalid` to `true` when an error exists and `false` otherwise. Inputs are always controlled; no uncontrolled `default_value`.

### Feedback

- `Badge`: `tone: BadgeTone = Neutral`, `children`. Renders `span.ui-badge.ui-badge--{tone}`. Pill, mint 500 13px, padding 4px 12px, radius 9999. Neutral uses Paper background with Mist text and hairline border; Mint uses Mint background with Ink text; Cream uses Cream background with Ink text and hairline border; Blossom uses Blossom background with Plum text; Plum uses Plum background with Paper text. Chromatic tones have a transparent border.
- `PillBadge`: `label: String = "Receta verificable"`. Renders `span.ui-pill` with the label. Mint background, Ink 13px 500, padding 4px 12px, radius 9999, no border, white-space nowrap.
- `Notice`: `tone: NoticeTone = Info`, `children`. Renders `div.ui-notice.ui-notice--{tone}` with the matching `Icon` and `div.ui-notice-body` children. Card: 1px hairline, 24px radius, 20px padding, flex row, 12px gap, align start. Info Cream, Success Mint, Warning Blossom, Error Blossom with Plum text. `role="alert"` only for Error. The icon is 20px.
- `ErrorText`: `text: String`. Renders `p.ui-error` with the `Alert` icon and the text, `role="alert"`.
- `HintText`: `text: String`. Renders `p.ui-hint` with the text.
- `EmptyState`: `title: String`, `description: String = ""`. Renders `div.ui-empty` with `p.ui-empty-title` and, only when `description` is non-empty, `p.ui-empty-description`. Cream, 1px hairline, 24px radius, 32px padding, centered. Title 18px 500; description 14px/1.5 Mist, max 48ch, centered.
- `LoadingState`: `label: String = "Cargando..."`. Renders `div.ui-loading` with `span.ui-spinner` and the label, `role="status"`, `aria-live="polite"`. The spinner is the only animation; reduced motion disables it.
- `ConfirmPanel`: `message`, `confirm_label`, `cancel_label: String = "Cancelar"`, `busy: bool`, `on_confirm: EventHandler<MouseEvent>`, `on_cancel: EventHandler<MouseEvent>`, `children`. Renders `div.ui-confirm` with children first, then `div.ui-actions`: Secondary cancel Button and Danger confirm Button. Confirm is disabled while `busy`. Inline cream alert group, not a modal; do not trap focus.
- `DataTable`: `columns: Vec<String>`, `caption: Option<String>`, `children`. Renders `div.ui-table-wrap > table.ui-table`; when `caption` is `Some`, renders `caption` as the table caption. Each column renders `th scope="col"`. Children are plain `<tr>`/`<td>`.
- `ListRow`: `title: String`, `meta: Vec<String>`, `badge: Option<Element>`, `children` (trailing actions). Renders `div.ui-row > div.ui-row-body` with `h3.ui-row-title`, an optional badge, and `p.ui-row-meta` when meta is non-empty, then `div.ui-row-actions` children. Uses Paper, 1px hairline, 24px radius, padding 16px 20px, flex space-between center, 16px gap. Title 18px 500; meta 14px/1.5 Mist; actions right aligned, flex wrap, 12px gap.

### Layout

- `PageShell`: `id: String`, `children`. Renders `div.ui-page` with the id, then `div.ui-container.ui-stack` children.
- `PageHeader`: `title: String`, `subtitle: Option<String>`, `children` (actions). Renders `header.ui-page-head` with `h1.ui-page-title`, optional `p.ui-page-subtitle`, and `div.ui-page-head-actions` children.
- `Card`: `children`. Renders `div.ui-card`.
- `SectionCard`: `title: String`, `description: Option<String>`, `children`. Renders `div.ui-card` with `h2.ui-card-title`, optional `p.ui-card-description`, and children.
- `FormGrid`: `columns: FormColumns = Two`, `children`. Renders `div.ui-form-grid` plus `.ui-form-grid--two` or `.ui-form-grid--one`.
- `DetailGrid`: `children`. Renders `dl.ui-detail-grid`.
- `DetailItem`: `label: String`, `#[props(default)] wide: bool`, `children`. Renders `div` with `.ui-detail-wide` when wide, then `dt` label and `dd` children.
- `ActionRow`: `children`. Renders `div.ui-actions`.
- `FilterBar`: `children`. Renders `div.ui-filter-bar`.
- `SectionBand`: `id: Option<String>`, `tone: SurfaceTone = Paper`, `children`. Renders `section.ui-band.ui-band--{tone}` with the id only when `Some`, and one `div.ui-container` containing children.

## 8. Page adoption

None. This spec adds components and CSS only; pages still render their existing raw markup. Page adoption begins in `20-T`.

## 9. CSS contract

| Selector | Values |
|---|---|
| `.ui-page` | owned by `15-O`; do not redefine |
| `.ui-container`, `.ui-container--narrow`, `.ui-container--reading`, `.ui-stack` | owned by `15-O`; do not redefine |
| `.ui-page-head` | flex, space-between, align start, wrap, gap 16, margin-bottom 0 |
| `.ui-page-title` | mint 500 22px/1.22 Ink, margin 0 |
| `.ui-page-subtitle` | mint 400 14px/1.5 Mist, margin 4px 0 0 |
| `.ui-page-head-actions` | flex, wrap, align center, gap 12, margin-top 0 |
| `.ui-actions` | flex, align center, wrap, gap 12, margin-top 16 |
| `.ui-card` | Paper background, 1px hairline, 24px radius, 24px padding |
| `.ui-card-title` | mint 500 20px/1.25 Ink, margin 0 0 16px |
| `.ui-card-description` | mint 400 14px/1.5 Mist, margin -8px 0 16px |
| `.ui-field` | flex column, gap 4, min-width 0; `.ui-field--wide` grid-column 1/-1 |
| `.ui-field-label` | mint 500 14px/1.5 Ink |
| `.ui-input`, `.ui-textarea`, `.ui-select` | width 100%, Paper background, 1px hairline, radius 8, padding 8px 12px, mint 400 16px/1.5 Ink, min-height 44px; textarea min-height 96px, resize vertical; select cursor pointer |
| control focus | `border-color: var(--color-deep-teal);` plus the `15-O` focus utility; never `outline: none` without a ring |
| control disabled | Cream background, Mist text, opacity 1, cursor not-allowed |
| `.ui-input[type="color"]` | width 64px, height 44px, padding 4px |
| `.ui-input[type="file"]`, `.ui-file-input` | 14px Mist, padding 6px 8px; `::file-selector-button` and `::-webkit-file-upload-button` Paper, 1px Mist, radius 8, 6px 12px, mint 500 14px, margin-right 12px |
| `.ui-field-hint` | 14px Mist, margin 8px 0 0 |
| `.ui-field-error` | 14px Plum, margin 4px 0 0 |
| `.ui-check` | flex, align start, gap 8, 16px/1.5 Ink; checkbox 16px, accent Deep Teal, 2px top margin; description 14px Mist; `.ui-check--wide` grid-column 1/-1 |
| `.ui-badge` | inline-flex center, gap 4, padding 4px 12px, radius 9999, mint 500 13px/1.22, transparent border by default; tones per contract |
| `.ui-pill` | Mint background, Ink, mint 500 13px/1.22, padding 4px 12px, radius 9999, white-space nowrap |
| `.ui-notice` | flex, gap 12, align start, 1px hairline, 24px radius, 20px padding, text-align left; icon 20px; body 16px/1.5; title 500 |
| `.ui-error` | flex, gap 6, align center, Plum 14px/1.5, margin 8px 0 0 |
| `.ui-hint` | Mist 14px/1.5, margin 8px 0 0 |
| `.ui-empty` | center, Cream, 1px hairline, 24px radius, 32px padding; title mint 500 18px/1.25; description 14px/1.5 Mist, max 48ch, centered |
| `.ui-loading` | flex, center, gap 8, 14px Mist, 16px padding |
| `.ui-spinner` | 16px circle, 2px hairline border, 2px Deep Teal top border, `animation: ui-spin 800ms linear infinite`; reduced motion disables |
| `.ui-confirm` | Cream, 1px hairline, 24px radius, 20px padding, 16px/1.5; actions flex end wrap gap 12 margin-top 16 |
| `.ui-list` | flex column, gap 12 |
| `.ui-row` | Paper, 1px hairline, 24px radius, padding 16px 20px, flex space-between center, gap 16 |
| `.ui-row-body` | min-width 0 |
| `.ui-row-title` | mint 500 18px/1.25 Ink, margin 0 |
| `.ui-row-meta` | 14px/1.5 Mist, margin 2px 0 0 |
| `.ui-row-actions` | flex wrap gap 12 |
| `.ui-table-wrap` | overflow-x auto, Paper, 1px hairline, 24px radius |
| `.ui-table` | width 100%, border-collapse collapse; th mint 500 14px/1.4 Mist, text-align left, padding 12px 16px, border-bottom 1px hairline; td 14px/1.5 Ink, padding 12px 16px, border-bottom hairline, vertical-align top; last row no border |
| `.ui-form-grid` | grid 2 columns, gap 16; one column at max-width 760px; `--one` 1fr; `--two` repeat(2, minmax(0, 1fr)) |
| `.ui-detail-grid` | grid 2 columns, gap 16; one column at max-width 760px; `dt` 14px/1.5 Mist mint 400; `dd` 16px/1.5 Ink margin 0; `.ui-detail-wide` grid-column 1/-1 |
| `.ui-columns` | grid 2 columns gap 24 align-start; one column at max-width 900px |
| `.ui-filter-bar` | grid auto-fit minmax(180px, 1fr), gap 12, align-end, margin-bottom 24 |
| `.ui-variable-row` | grid 1fr auto, gap 12, align-end, margin-bottom 12; one column below 640 |
| `.ui-band` | padding 80px 24px; `--paper` Paper, `--cream` Cream, `--mint` Mint, `--blossom` Blossom |

No new literal colour outside `:root`; no pure black; no drop shadow except `--shadow-paper` on `.a4-page` and the focus halo. Inputs, buttons, cards, and tags use the radius scale only.

## 10. Critical files and anchors

| File | Anchor |
|---|---|
| `persona-sanus/src/main.rs` | `mod ui;` near the crate root; `enum Route` near 40 |
| `persona-sanus/src/ui/mod.rs` | `icon`, `button`, `format` declarations from `15-O`; add `form`, `feedback`, `layout` |
| `persona-sanus/src/ui/form.rs` | New; six controls and private `Field` |
| `persona-sanus/src/ui/feedback.rs` | New; badge, notice, feedback, table, list-row components |
| `persona-sanus/src/ui/layout.rs` | New; shell, header, card, grids, actions, filter bar, band |
| `persona-sanus/assets/main.css` | `:root` 1-58; add declarations after the `15-O` utilities, before legacy selectors |
| `context/Design.md` | §§Components, Tokens, Surfaces, Elevation, Layout |
| `context/sitemap.md` | route tables §1.1-1.3; role matrix §2; conventions §5 |

## 11. Assumptions and contingencies

- Scope is fixed: full v1 adoption plus presentational v1.1 billing. Do not add a billing route, a plan picker that calls an endpoint, a card form, or a `sync_locked` gate. The billing gallery in `19-S` is the only billing surface.
- The repo has no grenette or mint webfonts. Keep the existing fallback stacks; do not fetch or add fonts.
- No partner-clinic logos exist, so Home must not show `TrustLogoStrip`. If real marks arrive, pass them only from a data source.
- Dioxus 0.7 fallbacks: if `NavigationTarget` as a component prop does not accept `Route` directly, change `ButtonLink.to` and `NavLink.to` to `Route` and keep the internal `Link`; if `Option<Element>` props break the props macro, make `badge` or children required and pass `rsx! {}` at call sites; if `FormEvent` cannot be a component callback, keep the file `<input>` in each page and share only `Field` and `.ui-file-input` styling. None of these changes the visual contract or the one-primary rule.
- If the debug server path is not `target/dx/persona-sanus/debug/web/server`, use `dx serve --fullstack --port 8097` for the manual gate and keep the same curl assertions. If port 8095, 8096, or 8097 is busy, change only the port and the matching URL.
- If browser verification finds a control not in the Design.md list but required by the existing page, use the nearest existing primitive (`Badge`, `Card`, `Button Secondary`) and do not add a new component file. The component set is closed after `19-S`.

---

*Housekeeping: once a spec is fully done with no open divergences, move its
file to `specs/done/` (same file name, e.g.
`specs/done/16-P-ui-forms-feedback-and-layout.md`).
Specs with open divergences stay in `specs/` next to `TEMPLATE.md` until the
tracking file is cleared.*
