# PersonaSanus — Audit (`/audit`)

| Field | Value |
|---|---|
| Source | `context/sitemap.md` §1.2 (`/audit` — Audit log, Administrator, actor/action/entity trail, sensitive reads, verification log, 5-year append-only), §2 (Administrator only), §4 M3, §5 (no PHI in URLs); `context/PRD.md` §6 `audit_log`, §8 traceability (5 years, append-only), §9 minimization/ARCO, FR-050; D-03, D-04, D-06, D-21; `context/database-design.md` §4/`audit_log`; `context/Design.md` (Card, Section Card, Form Input, Data Table, Tokens, Elevation); `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` step 16, Page adoption row `Audit` (line 235), `PageShell`, `PageHeader`, `FilterBar`, `TextField`, `Button`, `DataTable`, `EmptyState`, `LoadingState`, `ErrorText`, `Notice`, `code.ui-mono`; `context/screens/INDEX.md` §2.3 |
| Component | `Audit` — `persona-sanus/src/main.rs:2634-2756` (`#[component]` at `:2633`, `fn` at `:2634`). Reads `actor: Signal<Option<Me>>` and `checked: Signal<bool>`; holds `action`, `entity`, `q`, `rows: Signal<Vec<api::AuditRow>>`, `message`, `generation: Signal<u32>`. Target composes planned `src/ui/` primitives; that directory is not shipped |
| Route | `/audit` (`Route::Audit {}`, `main.rs:74-75`) — authenticated shell, Administrator only. No session replaces `/login` (`Route::AuthPage {}`); a non-Administrator paints `Acceso denegado`. The API independently returns `403` for a non-Administrator. No path or query parameter; filters are local signals |
| Layout | Inside `#[layout(Navbar)]` (`main.rs:51`, never closed). `PageShell id="audit-page"` → `div#audit-page.ui-page > .ui-container.ui-stack`; Paper canvas, 48px top / 80px bottom / 24px inline, 24px stack gap; `PageHeader`, `FilterBar`, feedback line, one `DataTable`; no full-bleed band, sidebar, drawer, modal, or per-row expansion |
| Locale | `es-EC` on every label and date. `Auditoría`, `Acción`, `Entidad`, `Buscar`, `Filtrar`, `Fecha`, `Actor`, `Metadatos`. Timestamps `DD-MM-AAAA HH:MM`; 24-hour. Metadata is machine JSON, not translated |
| Status | ✅ Spec complete (sections 1–7). Route status: ⬜ — sitemap §1.2: planned Administrator audit surface, retained 5 years, append-only; §3 notes the shipped component and spec 12-L. Shipped: Administrator gate, three free-text filters, generation-driven fetch, raw timestamp, raw metadata JSON, no pagination/export. Target: step-16 shell adoption with `FilterBar` + `DataTable`, frozen `es-EC` labels, `format_timestamp`, monospace metadata, `loaded`/`LoadingState`, and an empty state that does not flash |

## 1. Purpose

The read-only trace of what the clinic's staff and system did, and of who read clinical records. Four jobs, in order:

1. **Filter.** Narrow by action, entity type, or a free-text query over entity id and metadata.
2. **Read the trace.** Show one row per event: timestamp, actor, action, entity, entity id, metadata.
3. **Review sensitive access.** Surface the `sensitive_read` rows (patient/prescription reads) and the public `verification_attempt` rows that FR-050 and §6 require.
4. **Preserve trust.** Make clear this is an append-only, 5-year record: no edit, delete, purge, export, or “mark as reviewed” control exists here.

It never renders:

- An editor, delete, purge, archive, restore, export, download CSV/JSON, or retention-change control. The log is append-only for 5 years (§8); trimming is a server job, not this screen.
- Patient PHI beyond what the authorized audit row itself contains. No patient name/photo lookup, no diagnosis, no medication, no prescription body, no image thumbnail. A `patient_id` or `prescription_id` in metadata is an opaque identifier, not a chart.
- Raw IP addresses, user agents, session tokens, cookies, password hashes, MFA secrets, card tokens, PAN, CVV, or provider payloads. Verification logging keeps only what the law permits.
- A verification attempt counter, a per-IP search, or a live public-verification form. Public verification is `/verify/:code`.
- Billing, price, trial, plan, `sync_locked`, or subscription UI (D-04).
- Signature or stamp imagery (D-06); rich text or `contenteditable` (D-21).
- A second filled Primary. `Filtrar` is the only Primary; no per-row action is Deep Teal.
- A “clear audit log” or “delete row” affordance, even for an Administrator. Append-only is the point.

## 2. Structure

One centered column, Paper. Header, filters, feedback, one table. The table is the only large surface.

| Order | Region | `id` / class | Band tone | Contents |
|---|---|---|---|---|
| — | Navbar (layout) | `header.ui-navbar` → `.ui-navbar-inner` | — | Owned by `context/screens/Header.md` §2–§3; not restated |
| 1 | Page canvas | `PageShell id="audit-page"` → `div#audit-page.ui-page > .ui-container.ui-stack` | Paper `#ffffff` | 48px top, 80px bottom, 24px inline; 1200px container; 24px stack gap |
| 2 | Page header | `header.ui-page-head` · `h1.ui-page-title` · `p.ui-page-subtitle` | Paper | `Auditoría`; `Registro de solo anexación con retención de 5 años. Filtra por acción, entidad o texto.`; no header action |
| 3 | Filters | `FilterBar` → `.ui-filter-bar` | Paper | TextField `Acción`; TextField `Entidad`; TextField `Buscar`; Primary `Filtrar` |
| 4 | Feedback | `.ui-error-text` / `.ui-hint-text` | Paper | `ErrorText` for fetch failure; `HintText` for the empty-result sentence when the row region does not own it; nothing on success |
| 5 | Events table | `DataTable` → `.ui-table-wrap` > `table.ui-table` | Paper | `<caption>` `Eventos de auditoría`; columns `Fecha`, `Actor`, `Acción`, `Entidad`, `ID entidad`, `Metadatos`; one `<tr>` per event |
| 6 | Empty | `EmptyState` | Cream `#fffded` | `Sin eventos` + one description line; replaces the table body/slot, not the filters |
| 7 | Loading | `LoadingState` | Paper | `Cargando...` until the first fetch settles; filters stay visible |
| 8 | Denied | `Notice tone=Error` | Paper | `Acceso denegado` + reason; replaces filters, table, empty, and loading |

Regions 5–7 are mutually exclusive in the result slot. Region 3 always renders for an Administrator, including during loading, empty, and error. Region 4 is above the slot. The page is read-only; no row has an action cell.

### 2.1 Session finish and role

`checked` is the shell session-check signal; `actor` is `Signal<Option<Me>>`. `Me::is_administrator()` (`api.rs:49-51`) is the page gate; `clinic_members.role` is the role authority (D-03).

| Session finish / role | Result | What renders |
|---|---|---|
| Check in flight (`!checked`) | no fetch | `PageShell` + one `LoadingState` `Cargando...` |
| Checked, no session | no fetch | The effect replaces `/login`; `LoadingState` `Cargando...` until the replacement paints. No audit content |
| Checked, session, `actor == None` | no fetch | `LoadingState` `Cargando...`; no role is guessed |
| Non-Administrator | no fetch | `Notice` Error `Acceso denegado` + §3.4 reason; no filters or rows |
| Administrator | fetch | `PageHeader`, `FilterBar`, feedback, and the event table (or empty state) |

The client check is UX only. The `/api/audit` handler requires an Administrator and returns `403` for any other role; the UI renders that refusal as the same `Notice` Error.

### 2.2 Filters and request

| Control | State | Behaviour |
|---|---|---|
| `Acción` | local `action: Signal<String>`, initial empty | free text; not a select; does not submit on input |
| `Entidad` | local `entity: Signal<String>`, initial empty | free text; not a select; does not submit on input |
| `Buscar` | local `q: Signal<String>`, initial empty | free text over entity id and metadata |
| `Filtrar` | Primary `Button` | increments `generation`; one `api::fetch_audit(action, entity, q)`; no auto-submit on Enter is wired except the native form behavior if the control is a form |

The filter values never enter the URL, history, analytics, or a log line. The request is `GET /api/audit` with optional `action`, `entity`, and `q` query parameters built by the client helper; the server trims empties. There is no status filter, actor picker, date-range picker, page-size control, or sort control in v1 array. The table order is the server's; the page does not re-sort.

`loaded` starts false. The first effect run (empty filters) is the initial read. The result slot shows `LoadingState` until it settles. A filter submit sets `loaded` false again and shows the loading state; the previous rows can stay in memory but are not painted as current. An empty successful response paints `EmptyState` `Sin eventos`; a failed response paints `ErrorText` and keeps the filters usable. Submitting an identical filter set still refetches, because the generation increments unconditionally.

### 2.3 Table composition

`DataTable` wraps a real `<table>` in a horizontally scrollable `.ui-table-wrap` only when needed; the page itself never scrolls horizontally.

| Column | Value | Rule |
|---|---|---|
| `Fecha` | `format_timestamp(row.at)` (`main.rs:4314`) | `DD-MM-AAAA HH:MM`, 24-hour; malformed value paints as stored |
| `Actor` | `row.actor_name` else `row.actor_id` else `—` | text only; no profile fetch, no link, no avatar |
| `Acción` | `row.action` | verbatim machine token, e.g. `sensitive_read`, `create_patient`, `verification_attempt`, `set_member_role`; do not prettify or translate |
| `Entidad` | `row.entity` | verbatim machine token, e.g. `patient`, `prescription`, `clinic_member`, `verification` |
| `ID entidad` | `row.entity_id` | opaque text; wrap long ids; no link, no lookup |
| `Metadatos` | `row.metadata` (`serde_json::Value`) | serialized JSON in `<code class="ui-mono">`; escaped text, never HTML; wrap anywhere; `{}` renders as `—`? State the shipped truth: the stored value is painted as text |
| Empty table | no rows | `EmptyState` `Sin eventos`; caption is not painted with zero rows if the empty state replaces the table |

Metadata may contain `patient_id`, `prescription_id`, `email`, `via`, `found`, `status`, `clinic_id`, `role`, or `event_hash`. Those are identifiers and audit facts, not a chart; the page does not join them to a patient or template. It never renders a JSON object as interactive fields.

Rows are not editable or clickable. There is no row action, checkbox, select-all, bulk control, or expandable drawer. A long metadata value wraps inside its cell and grows the row; the table wrapper may scroll horizontally if the viewport cannot fit the six columns, but the page shell does not.

### 2.4 Append-only and retention

The screen is a read projection, not an audit writer. It calls no mutation endpoint, shows no delete/edit control, and cannot purge a row. The server owns the audit write (`record_audit`) and the 5-year retention; the page only states the policy in the subtitle. Hiding or trimming a row here would be a false record and is out of scope. The 5-year claim is a product/legal policy, not a client-side TTL: do not compute a “fresh/expired” badge, do not filter out old rows in the browser, and do not offer an export.

The audit payload has no billing rows in v1 (D-04). A future billing audit entry can appear as another `action` string without a new field, but no billing UI or filter is added now.

## 3. Copy text

All strings verbatim, `es-EC`; punctuation belongs to the copy. Navbar copy is owned by `Header.md` §3 and not repeated. Machine tokens (actions, entities, JSON keys) are data, not UI copy, and are shown unchanged.

### 3.1 Page header

| Element | Copy |
|---|---|
| Page title (`h1`) | `Auditoría` |
| Page subtitle | `Registro de solo anexación con retención de 5 años. Filtra por acción, entidad o texto.` |

### 3.2 Filters

| Element | Copy | Note |
|---|---|---|
| Action label | `Acción` | visible label |
| Action placeholder | `Acción (p. ej. sensitive_read)` | example, not a value |
| Entity label | `Entidad` | visible label |
| Entity placeholder | `Entidad (p. ej. patient)` | example, not a value |
| Search label | `Buscar` | visible label |
| Search placeholder | `Buscar en entidad o metadatos` | free text |
| Filter action | `Filtrar` | Primary; sole filled action |
| Busy | `Filtrando...` | current Primary while a request is in flight |
| Empty hint | `No hay eventos para estos filtros.` | description under `Sin eventos`; no result count |

No date range, actor select, event-type multi-select, “clear filters” button, “apply” secondary, or saved filter renders. The fields are plain text; the placeholder examples are the only hint.

### 3.3 Table

| Element | Copy |
|---|---|
| Caption | `Eventos de auditoría` |
| Column 1 | `Fecha` |
| Column 2 | `Actor` |
| Column 3 | `Acción` |
| Column 4 | `Entidad` |
| Column 5 | `ID entidad` |
| Column 6 | `Metadatos` |
| Missing actor | `—` |
| Missing metadata | `—` or the stored empty object; never an invented sentence |

### 3.4 Gates and feedback

| Element | Copy |
|---|---|
| Session/actor gate | `Cargando...` |
| Denied title | `Acceso denegado` |
| Denied body | `Solo el administrador puede ver la auditoría.` |
| Fetch error fallback | `No se pudo cargar la auditoría.` |
| Empty title | `Sin eventos` |
| Empty description | `No hay eventos para estos filtros.` |

## 4. Displayed information

### 4.1 What the screen shows

| Element | Source | Value shown |
|---|---|---|
| Page title/subtitle | static | `Auditoría`, the 5-year/filter sentence |
| Administrator gate | `Me::is_administrator()` (`api.rs:49-51`); server `/api/audit` | both controls and table, or denial; server `403` for a non-Administrator |
| Action filter | local `action: Signal<String>` | typed plain text, passed trimmed to `fetch_audit`; no URL |
| Entity filter | local `entity: Signal<String>` | typed plain text, passed trimmed; no URL |
| Free-text filter | local `q: Signal<String>` | typed plain text, passed trimmed; no URL |
| Filter submit | `generation` increment | one request per activation; no debounce, no auto-submit |
| Rows | `api::fetch_audit(action, entity, q)` → `Vec<api::AuditRow>` | server order, server scope; no client re-sort |
| Timestamp | `AuditRow.at` via `format_timestamp` | `DD-MM-AAAA HH:MM`; malformed input unchanged |
| Actor | `AuditRow.actor_name` else `.actor_id` else `—` | display name or opaque id; no profile lookup |
| Action | `AuditRow.action` | machine token verbatim |
| Entity | `AuditRow.entity` | machine token verbatim |
| Entity id | `AuditRow.entity_id` | opaque text; no link |
| Metadata | `AuditRow.metadata: serde_json::Value` | JSON text in `code.ui-mono`, escaped, wrap-anywhere |
| Empty | empty `rows` after a successful fetch and `loaded` true | `EmptyState`; not an error |
| Error | non-empty `message` | `ErrorText` + the last successful result or none; no false empty claim |
| Loading | `!loaded` after the Administrator gate | `LoadingState` in the result slot; filters visible |

The endpoint is `GET /api/audit?action=&entity=&q=`. `api::fetch_audit` builds only non-empty parameters (`api.rs:595-615`). The page sends no actor, date, clinic, page, limit, or sort parameter. There is no pagination; the server's bounded result is painted as returned.

### 4.2 Hard rules

- Server-side authorization is the boundary. The page gate hides the UI from a non-Administrator, and `/api/audit` independently requires the Administrator role and returns `403` for anyone else. A direct request must fail, not inherit a client omission.
- Append-only: no create, edit, delete, purge, trim, restore, or “mark reviewed” control exists here. The 5-year retention and trimming belong to the server job/runbook, not this screen. Hiding a row or rewriting metadata here would break the audit trail.
- The 5-year subtitle is policy, not a client TTL. Do not compute an age badge, do not filter rows by a local clock, and do not offer “delete expired”.
- Sensitive reads are recorded server-side with actor/action/entity/entity id/metadata. The page displays those rows; it does not generate a sensitive-read event by opening itself. Filtering the log is not reading a patient chart.
- The verification log (FR-050) appears as its own action/entity rows. Do not expose the raw IP; the server keeps only what the law allows. Do not add a public-code lookup here.
- No PHI beyond the authorized audit row: no patient name, cédula, diagnosis, medication, prescription body, photo, or thumbnail. Identifiers in `entity_id`/metadata are opaque and stay text; do not link them to patient/prescription/template screens from this table.
- Metadata is untrusted text: escape it, never inject HTML, never use `innerHTML`, and wrap long JSON. The page never interprets an `action` or metadata key as a control.
- Filters are non-PHI and stay in signals. Do not put an action, entity id, query, actor id, or clinic id in the URL, history, analytics, or a log line. The route is exactly `/audit`.
- One Primary per viewport: `Filtrar`. No row action is filled teal. The page has no secondary mutation action.
- No export/download, pagination, infinite scroll, select-all, bulk action, sort control, saved filter, chart, counter, or “last updated” timestamp. The table is the record.
- No billing/price/trial/plan/`sync_locked` copy or state (D-04); no signature or stamp imagery (D-06); no rich text, `contenteditable`, or HTML payload (D-21).
- No new endpoint, table, column, role, permission, or API argument. Filtering reuses `GET /api/audit`.

### 4.3 Dynamic states

| State | Screen behaviour |
|---|---|
| Session check in flight (`!checked`) | `PageShell` + `LoadingState` `Cargando...`; no filters, fetch, or table |
| Checked, no session | `Cargando...`; the effect replaces `/login`; no audit content |
| Checked, `actor == None` | `Cargando...`; no role guess and no fetch |
| Non-Administrator | `Notice` Error `Acceso denegado` + reason; no filters, table, empty, or loading |
| Administrator, first load | Header + filters + `LoadingState`; request with empty filters; no table yet |
| Filter submit, in flight | `Filtrar` reads `Filtrando...` and is disabled; prior rows are not painted as the new result; filters keep values |
| Filter submit, rows | Table with all six columns; timestamps formatted; metadata monospace; no count line |
| Filter submit, zero rows | `EmptyState` `Sin eventos` + filter description; filters stay mounted and editable |
| Fetch error | `ErrorText` with the returned message or fallback above the result slot; filters stay usable; no fabricated row |
| Identical filter resubmit | `generation` increments; another request; loading state returns |
| Empty filters | Initial/all-events read; the same state machine as any filter |
| Long metadata | JSON wraps inside the cell; row grows; table wrapper scrolls horizontally only if the six columns cannot fit; no horizontal page scroll |
| Long actor/entity id | Wraps or truncates with accessible full text; no hidden overflow without a title/tooltip; no link |
| Sensitive-read row | Painted like any event; `action`/`entity` tokens carry the meaning; no patient name is joined |
| Verification row | Painted like any event; no raw IP and no public-code lookup |
| Billing row | Absent in v1; no filter, badge, or copy exists for billing |
| Narrow viewport | Table wrapper scrolls horizontally with visible column headers; or the shared `DataTable` stacks labelled cells. The page shell never scrolls horizontally |
| Zoom/reflow | 200% and 320px keep the filters one column, the table wrapper scrollable, and all header text reachable |
| Reduced motion | No animation; the shared spinner (if any) stops under `prefers-reduced-motion` |

## 5. Style

Every value is a `context/Design.md` token declared in `persona-sanus/assets/main.css` `:root`. No new color, font, icon package, CDN, or webfont.

### 5.1 Color

| Use | Value | Token |
|---|---|---|
| Canvas, cards, table surface | `#ffffff` | `--color-paper` |
| Title, column headers, cell text | `#061d29` | `--color-ink` |
| Subtitle, placeholders, meta, actor id | `#425d6d` | `--color-mist` |
| Hairlines: card, table, row rules | `#e5e7eb` | `--color-hairline` |
| Primary fill for `Filtrar`, focus ring | `#006e75` | `--color-deep-teal` |
| Primary hover | `#0b978e` | `--color-bright-teal` |
| Empty surface | `#fffded` | `--color-cream` |
| Error text, Alert glyph | `#4d0037` | `--color-plum` |
| Error surface | `#ffc3e6` | `--color-blossom` |
| Focus halo | `rgba(185,255,232,0.55)` | `--color-mint-glass` |

Never `#000000`. Never a chromatic table/card border; hairlines only. The primary fill is the only saturated surface; metadata code uses Ink on Cream, not a syntax-highlight palette. Measured contrast: Ink/Paper 17.28 · Mist/Paper 6.96 · Paper/Deep Teal 6.02 · Ink/Cream 16.89 · Mist/Cream 6.80 · Plum/Paper 15.32 · Plum/Blossom 10.36. AA for body; the 14px metadata code clears AA.

### 5.2 Typography

| Element | Family | Weight | Size / line-height |
|---|---|---|---|
| Page title (`h1`) | `--font-mint` | 500 | 22px / 1.22 Ink |
| Page subtitle | `--font-mint` | 400 | 14px / 1.5 Mist |
| Filter label | `--font-mint` | 500 | 14px / 1.5 Ink |
| Filter input / placeholder | `--font-mint` | 400 | 16px / 1.5 Ink / Mist |
| Primary label | `--font-mint` | 500 | 16px / 1.22 Paper |
| Table caption | `--font-mint` | 500 | 14px / 1.5 Ink, start-aligned |
| Column header | `--font-mint` | 500 | 14px / 1.5 Ink |
| Cell | `--font-mint` | 400 | 14px / 1.5 Ink |
| Metadata cell | `--font-mint` monospace | 400 | 13px / 1.5 Ink on Cream |
| Empty title / description | `--font-mint` | 500 / 400 | 18px / 1.25 Ink; 14px / 1.5 Mist |
| Error | `--font-mint` | 400 | 14px / 1.5 Plum |

No `--font-grenette`: app chrome, not a 46px+ editorial opener. Machine tokens and JSON use the mono role, never the display serif; long tokens wrap at 13–14px, never shrink below 13px.

### 5.3 Geometry

- 4px base. `PageShell`: 48px top, 80px bottom, 24px inline; container max 1200px; stack gap 24px.
- `PageHeader`: stack, 4px title/subtitle gap; no actions.
- `FilterBar`: auto-fit tracks with a 180px minimum and 12px gap, aligned end; the three fields share a row on wide screens, stack one per row below 760px. Inputs 8px radius, `8px 12px`, 44px min-height, 6px below label. Primary is content-width, 8px radius, `12px 24px`.
- Feedback: `ErrorText` 8px below filters, 14px / 1.5, Alert 16px; no border.
- Table: `DataTable` 1px hairline outer and horizontal rules, 8px inner radius? Follow the primitive; 24px card radius if wrapped in a `Card`; no shadow. Cell padding `10px 12px`; header Paper, 500; zebra absent.
- Metadata cell: `code.ui-mono`, 13px / 1.5, Cream wash, 4px 8px padding, 4px radius, `overflow-wrap:anywhere`, `white-space:pre-wrap`, no max-height/scroll inside a cell. The table wrapper, not the cell, owns horizontal scrolling.
- Column widths: `Fecha` nowrap/fit, `Actor` / `Acción` / `Entidad` / `ID entidad` wrap, `Metadatos` flexible/largest. At 320px the wrapper scrolls; the page does not.
- Empty: Cream, 24px radius, 32px padding, centered, title 18px; no illustration.
- Loading: shared spinner + label, centered in the result slot; no skeleton rows.
- Elevation: flat; hairlines and wash only.

### 5.4 Iconography and imagery

Line icons only, 1.5px stroke, 20px, Ink or Plum: `Alert` in `ErrorText`; `Filter` is text, not an icon-only button. No avatar, profile image, patient photo, document thumbnail, QR, signature, stamp, illustration, chart, or coloured action icon. Metadata may contain a `sha256` or id string; it stays text, not a glyph or scannable image. The Navbar owns the brand mark.

## 6. Accessibility

- Landmarks: navbar banner (`Header.md` §6); one content region. The filter region is a labelled group (`role="search"` or `<form>`) with the filters; the table region is a real `<table>`. No nested banner or duplicate `nav`.
- Headings: one `h1`, `Auditoría`; no `h2` unless the table is wrapped in a `SectionCard` (then its title is the one `h2`). The caption is not a heading. No skipped level.
- Table semantics: real `<caption>` `Eventos de auditoría`; every `th` has `scope="col"`; `<thead>`/`<tbody>`; no layout table. Metadata JSON is one `<code>` per cell; a screen reader reads the serialized string, not a fake object UI.
- Tab order: `Acción` → `Entidad` → `Buscar` → `Filtrar` → the table wrapper if it is focusable for horizontal scrolling → nothing else. Rows are not focusable and not clickable. No hidden control in tab order.
- Focus: `outline: 1px solid var(--color-deep-teal); outline-offset: 2px; box-shadow: 0 0 0 3px rgba(185,255,232,0.55)` on the three fields, the Primary, and the focusable table wrapper. Visible on Paper and Cream; never removed.
- Labels: each field has a visible wrapping label; placeholders are supplementary examples, not the accessible names. The Primary is named `Filtrar`. The caption names the table. The metadata cell is not labeled by colour.
- Busy: `Filtrar` is disabled with `aria-busy="true"` and a visible `Filtrando...` label; one request per activation. Filters remain readable and keep their values. Focus is not moved when the result lands.
- Errors: fetch failure is `ErrorText` with `role="alert"` above the table; it does not replace the filters and does not become a fake empty result. A `403` is rendered as the denial `Notice`, not a transport error.
- Empty: `EmptyState` is a labelled region with `Sin eventos` and the description; it does not announce a table with zero headers.
- Status: no colour-only signal. The action/entity tokens and the error sentence carry the meaning. Metadata is escaped text, so no HTML injection and no interactive link inside a cell.
- Contrast: Ink/Paper 17.28 · Mist/Paper 6.96 · Paper/Deep Teal 6.02 · Ink/Cream 16.89 · Mist/Cream 6.80 · Plum/Paper 15.32 · Plum/Blossom 10.36. AA for body; metadata 13px still clears AA at Ink/Cream.
- Touch: `Filtrar` and the fields are at least 44px; no row action. The table wrapper can be scrolled by touch and keyboard.
- Reduced motion: no animation; if a shared spinner is used, it stops under `prefers-reduced-motion: reduce`.
- Zoom/reflow: 200% and 320px keep the filters one per row and the table wrapper scrollable; the page shell never overflows horizontally, and no metadata column is clipped without scroll.

## 7. Implementation notes

The shipped component is `fn Audit()` at `persona-sanus/src/main.rs:2634`, `#[component]` at `:2633`, closing before `Patients` at `:2757`. Signals: `action`, `entity`, `q`, `rows: Vec<api::AuditRow>`, `message`, `generation: u32`.

Current behavior:

- `!checked()` paints `div#audit-page > p` `Cargando...`.
- `!is_admin` (no profile or a non-Administrator) paints `div#audit-page > div.notice > h4` `Acceso denegado` + `p` `{message}`. For a non-Administrator the effect sets `Acceso denegado. Solo el administrador puede ver la auditoría.`; for no profile the string can be empty, so the notice may have a blank body. Target uses a fixed reason sentence in both cases.
- The effect reads `generation`, then `checked`; it redirects a missing actor to `Route::AuthPage {}`, but for a non-Administrator it only sets `message` and does not redirect. It calls `api::fetch_audit(&action(), &entity(), &q())` and stores rows or the error. It does not have a `loaded` signal, so an empty `rows` initially can flash `Sin eventos` before the first fetch resolves. Target adds `loaded`.
- The shipped filter controls are raw `input`s with placeholders `Acción (p. ej. sensitive_read)`, `Entidad (p. ej. patient)`, `Buscar en entidad o metadatos` and a raw `button` `Filtrar`. The button increments `generation`.
- The shipped table is a raw `<table>` with `<thead><tr><th>` for `Fecha`, `Actor`, `Acción`, `Entidad`, `ID entidad`, `Metadatos`. It has no `<caption>`, no `scope`, and no empty state. It always prints `tr` per row containing:
  - `row.at` raw (RFC3339), not `format_timestamp`;
  - `row.actor_name.clone().unwrap_or_else(|| row.actor_id.clone().unwrap_or_default())`;
  - `row.action`, `row.entity`, `row.entity_id`;
  - `row.metadata` raw `Value` (serialized JSON), no `code.ui-mono`, no wrap.
- The empty vector paints an empty `<tbody>`: no `Sin eventos`, no description.
- Every failure writes `message`; it is painted in a raw `p` above the table, not `ErrorText`; no `role="alert"`.
- The page has no pagination, export, date range, actor picker, or mutation control.

Plan step 16 adoption and the deltas:

- `PageShell id="audit-page"`; `PageHeader` `Auditoría` with the shipped subtitle unchanged; no header action.
- `FilterBar` with three `TextField`s: labels `Acción`, `Entidad`, `Buscar`; the shipped placeholder examples are supplementary. The Primary `Button` `Filtrar` is the sole filled action; busy label `Filtrando...`; the button increments `generation` only once per activation.
- Add `loaded: Signal<bool>` initialized false. The effect sets it false when a new generation starts and true in both the success and error arm after the fetch settles. The result slot renders `LoadingState` `Cargando...` while false; an empty vector only after true renders `EmptyState`; a non-empty vector renders `DataTable`.
- Replace the raw `table` with `DataTable columns=[...] caption="Eventos de auditoría"`. Each row is `<tr>` with six `<td>`; metadata is `<code class="ui-mono">{metadata.to_string()}</code>` as escaped Dioxus text. Dioxus escapes `{...}` by default; never use `inner_html` here.
- `format_timestamp(row.at)` for the date cell. `actor_name` → `actor_id` → `—` fallback.
- Denial: `Notice tone=Error` with `Acceso denegado` + `Solo el administrador puede ver la auditoría.`; no filters and no fetch. Keep the client gate and rely on the server `403` as well.
- Failure: `ErrorText` with the returned message or `No se pudo cargar la auditoría.`; keep filters. If rows from a previous success exist, say whether they stay: target keeps them under the error rather than blanking the audit trail.
- Empty: `EmptyState` `Sin eventos` / `No hay eventos para estos filtros.`; no result counter; no fake “last updated”.
- No pagination/export is added. If the server already bounds the rows, the page paints that bound; do not invent a page size or total.

Verification commands, from the repo root; none is recorded here as executed:

```sh
cd persona-sanus && cargo check
dx build --release --fullstack
caveman-blocks run persona-smoke --dir persona-sanus --port 8091
python3 .blocks/markdown-table-check.py --path context/screens/Audit.md --strict
```

The smoke script covers auth, not this Administrator table. A browser pass needs: Administrator `/audit`; Doctor denied (no filters); empty filter result; a known `sensitive_read` row; unknown action/entity; long metadata; 320px table wrapper; 200% zoom.

- Do not add: an edit, delete, purge, trim, archive, restore, “mark reviewed”, or “clear log” control.
- Do not add: CSV/JSON export, download, print, share, or copy-all of the audit log.
- Do not add: pagination, infinite scroll, result count, saved filters, actor/date/status filters, sorting, select-all, or bulk actions.
- Do not add: patient PHI, a patient/prescription/template lookup link, patient photo, clinic logo, signature/stamp image, or QR.
- Do not add: raw IP addresses, user agents, session tokens, cookies, password hashes, MFA secrets, card tokens, PAN, CVV, provider payloads, or PHI in metadata.
- Do not add: billing, price, trial, plan, `sync_locked`, entitlement, or image-quota copy (D-04).
- Do not add: rich text, `contenteditable`, HTML rendering, markdown rendering, or `inner_html` of metadata/actions (D-21).
- Do not add: a second Primary, a filled row action, or a colour-only event status.
- Do not add: a new route, endpoint, table, column, role, permission, query parameter, or API argument.

## 8. Spec references

| Spec | Connection |
|---|---|
| `specs/done/15-O-ui-tokens-actions-and-formatting.md` | `15-O` supplies the tokens, `Button`, and format helpers (`format_date`, `format_timestamp`, `role_label`) the audit table renders. |
| `specs/done/16-P-ui-forms-feedback-and-layout.md` | `16-P` supplies `PageShell`, `PageHeader`, `FilterBar`, `TextField`, `DataTable`, `LoadingState`, `EmptyState`, `ErrorText`, `Notice`, and `Button` the page uses. |
| `specs/done/24-X-ui-settings-detail-and-audit.md` | `24-X` adopts `Audit`: `PageShell id="audit-page"`, admin guard `Notice`, the `loaded` gate, `FilterBar` with primary `Filtrar`, filter-driven append-only rows. |
| `specs/done/25-Y-ui-legacy-css-cleanup-and-docs.md` | `25-Y` removes legacy page-ID selectors so the adopted table runs only on `.ui-*` classes and tokens. |
