# PersonaSanus — Patients (`/patients`)

| Field | Value |
|---|---|
| Source | `context/sitemap.md` §1.2 (`/patients` — Patients — list), §2 (role matrix: Doctor, Nursing, Secretary, Administrator read-only), §3 flow 1 (issue online), §4 M0; `context/PRD.md` FR-001, FR-002, FR-003, D-04, D-06, D-21; `context/Design.md` (Paper, Ink, Mist, Hairline, Deep Teal, Bright Teal, Mint Glass, Cream, Plum, 24px cards, 8px controls, flat elevation); `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` step 13, Page adoption row `Patients`, `PageShell`, `PageHeader`, `SearchField`, `ListRow`, `EmptyState`, `LoadingState`, `ErrorText`; `context/screens/INDEX.md` §2.3 |
| Component | `Patients` — `persona-sanus/src/main.rs:2758` (body 2757–2877); composes planned `PageShell`, `PageHeader`, `SearchField`, `ListRow`, `EmptyState`, `LoadingState`, `ErrorText` from `persona-sanus/src/ui/` (the directory is planned, not shipped); reads `Signal<Option<SessionUser>>`, `Signal<Option<Me>>`, `Signal<bool>`, `Signal<String>`, and `Signal<Vec<api::Patient>>` |
| Route | `/patients` (`Route::Patients {}`) — authenticated shell; signed-out visitors redirect to `/login` (`Route::AuthPage {}`); reading requires `Me::can_read_patients()`; no query params; the search term is a signal and never a URL parameter; Administrator is read-only |
| Layout | `PageShell id="patients-page"` inside `#[layout(Navbar)]` (opens at `main.rs:51`); root `#patients-page.ui-page` > `.ui-container.ui-stack`; Paper band, hairline row borders; no full-bleed exception |
| Locale | `es-EC` on every string; `cédula` spelling; dates `DD-MM-AAAA`; 24-hour time where a timestamp appears |
| Status | ✅ Spec complete (sections 1–7). Route status: ✅ — sitemap cell: search by name and cédula, list scoped by `clinic_id`, entry to create/detail. Shipped: step-13 adoption — `PageShell`/`PageHeader`, `loaded` signal, `ListRow` rows, `LoadingState`/`EmptyState`/`ErrorText` |

The clinic patient directory. It is the entry point to an existing record and to the creation form, and it searches only the caller's own clinic. This file freezes region order, copy, geometry, and states for the shipped component; it introduces no route, endpoint, table, column, role, or permission.

## 1. Purpose

The screen performs four jobs, in order:

1. **Find.** Search the caller's clinic patients by name or cédula with a single term.
2. **Open.** Navigate to one patient record through the row action, carrying only the opaque `patient.id`.
3. **Create.** Offer the creation form only to Doctor, Nursing, and Secretary through the sole Primary action.
4. **Hold the role boundary.** Administrator sees the same directory read-only; a session without a clinic role sees the denial, never the rows.

It never renders:

- PHI outside the authorized directory rows: the rows are clinical content for the four authorized roles, and no real patient detail appears in this spec, an example card, alt text, a URL, or audit metadata. The route is `/patients`; `patient.id` is the only identifier that leaves the screen.
- Billing, price, trial, plan, entitlement, or `sync_locked` copy (D-04); a signature or stamp image (D-06); a rich-text editor, formatting toolbar, `contenteditable` surface, or formatted clinical value (D-21); a second filled action or a control outside the listed surfaces.
- Sex, phone, notes, created or updated timestamps, prescription history, edit, delete, archive, pagination, result counts, sorting, a document preview, or a QR code in the row.

One filled teal action exists at most: the Primary create action when the caller is a writer. Administrator and every other state render none.

## 2. Structure

One Paper page, a stack of 24px gaps, descending from the navbar. The header and search remain mounted while the directory swaps between rows, empty, and loading. The error line sits directly below the search in every state.

| Order | Region | id / class | Band tone | Contents |
|---|---|---|---|---|
| — | Navbar (layout) | `#navbar` · `header.ui-navbar` | Paper `#ffffff`, bottom hairline | Owned by `context/screens/done/Header.md` §2–§5; not restated here |
| 1 | Page shell | `#patients-page` · `div.ui-page` > `.ui-container.ui-stack` | Paper | 48px top padding, 80px bottom, 24px inline; stack gap 24px |
| 2 | Page header | `.ui-page-head` · `h1.ui-page-title` · `p.ui-page-subtitle` · `.ui-page-head-actions` | Paper | Page title and subtitle, then at most one Primary action at the inline end |
| 3 | Search | `.ui-field` holding `input[type=search].ui-input` | Paper | `SearchField` label, value, oninput, placeholder; `wide: true`; label visually hidden |
| 4 | Feedback | `.ui-error` | Paper | `ErrorText` fetch-failure line; role alert; only when the `message` signal is non-empty |
| 5 | Patient list | `div.ui-list` holding one `div.ui-row` per patient | Paper | `ListRow` title, two meta lines, trailing Secondary `ButtonLink` |
| 6 | Empty or loading | `.ui-empty` · `.ui-loading` | Cream `#fffded` · Paper | `EmptyState` title and conditional description; `LoadingState` spinner and label |

Rows 5–6 are mutually exclusive; feedback row 4 sits above whichever one renders. Nothing is horizontally scrollable. No table renders, so there is no caption or `th scope` surface.

### 2.1 Session finish and role

`checked` is the shell's session-check signal and `actor` is `Signal<Option<Me>>`. `Me::can_read_patients()` is true for every non-null role; `Me::can_write_patients()` is true only for `doctor`, `nursing`, and `secretary` (`persona-sanus/src/api.rs:57-66`).

| Session finish / role | Reads directory | Create action | What changes; what does not |
|---|:---:|:---:|---|
| Check in flight (`!checked`) | no | no | `PageShell` + `LoadingState`; no header, search, or rows yet. The navbar, locale, and root id are already present |
| Checked, no session, or `actor` is `None` | no | no | `navigator.replace(Route::AuthPage {})` only for the missing session; otherwise the redirect loading notice. No header, search, rows, or create |
| Role `null` or no profile role | no | no | `PageHeader` denial title + denial subtitle; no search, rows, or create |
| `administrator` | yes | no | Full header, search, and rows; create action absent; no edit/delete control anywhere on this screen |
| `doctor`, `nursing`, `secretary` | yes | yes | Full header, search, rows, and the single Primary create action. Prescription draft and reprint limits live on `/prescriptions`, not here |

Role changes only whether the directory renders and whether the Primary action exists. It never changes region order, row copy, date format, or the clinic scope of the request. The server re-checks read permission and derives the clinic on every request; the client gate is convenience, not authorization.

### 2.2 First load, search, emptying

| State | What renders | What stays fixed |
|---|---|---|
| First load, before any fetch resolves (`loaded` false) | Header, search, then directory loading state in the list slot | Root id, title, subtitle, search label and term |
| Fetch resolves with zero rows, blank term | Header, search, empty state title; no description because no search ran | Root id, header, empty search field |
| Fetch resolves with zero rows, non-blank term | Header, search, empty state title and search-miss description | Root id, header, the term remains in the field |
| Fetch resolves with rows, or a new term is typed while rows show | Header, search, one `ListRow` per patient in server order; rows remain until the next request resolves; no full-page spinner returns | Root id, header, `loaded` stays true, the field never loses focus |
| A request fails or the term is cleared | Failure keeps header, search, and the error line per §4.3; clearing refetches the unfiltered clinic list | Root id, header; no route or history change |

There is no debounce, submit button, results counter, sort control, or pagination in the shipped path. Every input event updates the `query` signal; the effect re-runs `api::fetch_patients` with the trimmed term. The page never pushes a query string or navigates while typing.

### 2.3 Web and desktop surfaces

The same component, ids, copy, and role gates serve both builds; the branch is the existing `api::fetch_patients` error path, and neither surface writes.

- **Web:** the API result is authoritative. A fetch failure renders `ErrorText` and leaves no cache rows. On first load no row renders below it; after an earlier successful fetch, the previous rows may remain until the next result replaces them.
- **Desktop:** the API is tried first. On failure the screen keeps the error, opens the local database with `desktop_local::open_local_db`, calls `desktop_local::list_patients`, deserializes each value as `api::Patient`, and replaces the row signal with the cached patients. If the local database cannot open, the current list remains untouched; values that fail to decode are skipped, so an all-unreadable cache leaves an empty list while the API error stays visible.
- **Offline desktop:** every search input retries the API, fails, and re-reads the cache. The fallback helper takes no search term and no clinic argument; the cache view is not a search result and is not re-filtered in the page, so it never hides the failure. The create action and each row action are router links; no local insert, update, delete, or outbox entry runs from this screen.

### 2.4 Empty and denied surfaces

The empty region replaces the list only after the first resolve, so an empty signal at startup is never shown as a no-patient result. It is centered in a Cream card with a hairline border and 32px padding, and it keeps the header and search interactive. The denial region replaces everything below the PageShell: the PageHeader itself carries the denial title and the single explanatory subtitle. It has no search field, row, create action, or back link. A no-role session is denied; a fetch error is not a denial and keeps the search visible.

## 3. Copy text

All strings verbatim, `es-EC`. Punctuation is part of the copy. Each string appears once.

### 3.1 Page header and primary action

| Element | Copy | Target |
|---|---|---|
| Page title (`h1`) | `Pacientes` | — |
| Page subtitle | `Busca por nombre o cédula dentro de tu clínica.` | — |
| Create action, Primary, writers only | `Nuevo paciente` | `/patients/new` (`Route::PatientNew {}`) |

The screen renders no back link. `/patients` is a top-level destination of the navbar (`context/screens/done/Header.md` §3.2), so no `Volver a...` string exists here.

### 3.2 Search

| Element | Copy |
|---|---|
| `SearchField.label` (visually hidden) | `Buscar paciente` |
| `SearchField.placeholder` | `Nombre o cédula` |

No visible hint, helper line, or search button renders. The placeholder is not the accessible name; the label is.

### 3.3 Patient rows

| Element | Copy | Target |
|---|---|---|
| `ListRow.title` | `patient.full_name` — authorized clinical content, never sample data | — |
| Meta 1 | `Cédula: {value}` | — |
| Meta 2 | `Nacimiento: {date}` | — |
| Missing identifier or date | `—` | — |
| Row action, Secondary | `Ver ficha` | `/patients/{patient.id}` (`Route::PatientDetail { id }`) |

`{value}` is the stored cédula text, shown unchanged. `{date}` is `DD-MM-AAAA`. The screen adds no age, sex, phone, notes, or prescription-count meta.

### 3.4 Empty, loading, error, redirect

| Element | Copy |
|---|---|
| Empty title, any zero-row state | `Sin pacientes` |
| Empty description, non-blank term only | `No hay pacientes para esta búsqueda.` |
| Session-check loading | `Cargando...` |
| Directory loading | `Cargando pacientes...` |
| Fetch error | `No se pudieron cargar los pacientes.` |
| Redirect notice | `Redirigiendo...` |

### 3.5 Denied

| Element | Copy |
|---|---|
| Denied title (`h1`) | `Acceso denegado` |
| Denied subtitle | `Necesitas un rol en la clínica para consultar pacientes.` |

## 4. Displayed information

### 4.1 What the screen shows

| Element | Source | Value shown |
|---|---|---|
| Static copy | tokens in §3.1–§3.2 | page title, subtitle, search label, and placeholder |
| Create action | `actor: Signal<Option<Me>>`, `Me::can_write_patients()` (`api.rs:61`) | a Primary `ButtonLink` only for `doctor`, `nursing`, `secretary`; absent for `administrator`, `null`, and signed-out |
| Search term | `query: Signal<String>`, initial empty; input `value` bound to it | typed text; never read from or written to the URL |
| Directory request | `api::fetch_patients(&term)` (`api.rs:685`) | trimmed term; blank calls `/api/patients`, non-blank calls the same endpoint with encoded `q` |
| Server scope | `service::search_patients`, then `db::search_patients` (`db.rs:1189`) | `require_patient_read`, actor `clinic_id`, the `clinic_id` filter, and `deleted_at IS NULL`; no client widening |
| Server match and order | `db::search_patients` | name substring or exact cédula; `full_name` ascending; server-bounded result set; no client sort or total |
| Rows | `patients: Signal<Vec<api::Patient>>` | the returned records, in server order |
| Row title | `patient.full_name` | stored full name, verbatim |
| Identifier meta | `patient.national_id` | stored text, or `—` when absent or empty |
| Birth meta | `patient.birth_date` rendered by `format_date` (`main.rs:4321`) | first 10 characters parsed as `%Y-%m-%d`, painted `%d-%m-%Y`; otherwise raw text, and `—` when absent |
| Row action target | `patient.id` | `Route::PatientDetail { id }`; the id is opaque and the only value placed in the URL |
| Empty or loading region | `patients()` empty and `loaded` true, else `!loaded()` after the session gate | empty title plus description when the trimmed term is non-empty; otherwise the `LoadingState` spinner |
| Error line | `message: Signal<String>` from `api::fetch_patients` | the frozen §3.4 fallback when non-empty; raw transport text is not painted |
| Session gate | `checked`, `user: Signal<Option<SessionUser>>`, `actor` | redirect notice or denial per §2.1 |
| Desktop cache | `desktop_local::open_local_db`, `desktop_local::list_patients`, `serde_json::from_value::<api::Patient>` under `#[cfg(feature = "desktop")]` | cached rows only when the API request fails; undecodable values are skipped |

`api::Patient` also carries `sex`, `phone`, `notes`, `created_at`, and `updated_at` (`api.rs:87-97`). This screen never paints them. The list exposes identity and birth date only; the detail form owns the rest.

### 4.2 Hard rules

- Server-side authorization is the boundary: `service::search_patients` requires a clinic role and derives `clinic_id` from the actor. Every online result is limited by `db::search_patients` to that clinic. Client role checks are UX gates, never the security control; the desktop fallback is a local cache read, not a server scope.
- PHI stays out of the page URL, QR, marketing copy, and audit metadata. The route is exactly `/patients`; the search lives in the `query` signal, never in history or a link, and the only patient identifier in a URL is the opaque `patient.id` on the row action. Sensitive-read audit metadata records the transport and row identity, not the term.
- Administrator is read-only: no create, edit, or delete control renders, and the Primary create action is absent. Doctor, Nursing, and Secretary share the directory and exactly one Primary action per writer viewport; with no writer role there are zero filled teal actions. Row actions are Secondary.
- No billing, price, trial, plan, entitlement, invoice, or `sync_locked` copy (D-04). No signature or stamp image (D-06). No rich-text editor, toolbar, or `contenteditable` (D-21); one plain search input is the only field.
- No new endpoint, table, column, role, or permission, and no sort, count, pagination, infinite scroll, select-all, bulk action, or saved-search control. The only request is `api::fetch_patients` against the existing patient list route; the server owns match, order, and bound.
- Data minimization: sex, phone, notes, and both timestamps are loaded and never painted; a row carries name, cédula, birth date, and the opaque id. Missing values are explicit: absent cédula or birth date paints an em dash, never a blank cell or invented value, and a malformed birth date paints its stored text rather than a guessed date.
- The desktop cache never hides loss: whenever the API fails, `ErrorText` remains visible even if cached rows render below it; cached rows are a read-only fallback, not an offline search result and not a successful fetch. The empty state is not a denial state: a no-role caller never sees a blank directory, because the denial branch replaces it.
- One `h1` per settled state: the page title in the directory state, the denial title in the denied state. No section card and no rich-text surface.

### 4.3 Dynamic states

| State | Screen behaviour |
|---|---|
| Session check in flight / first directory load | `PageShell` plus `LoadingState`; no header, search, rows, or create. Root id is stable; the directory loading label appears only after the session gate |
| No session after check / `actor` is `None` | `navigator.replace` to the auth route when the session is missing; otherwise the redirect loading notice. No directory and no second redirect |
| Role `null` / profile without a role | `PageHeader` denial title and subtitle; no search, rows, create, or back link |
| Administrator, or writer role | Administrator: header, search, rows, and row actions render, create action absent, read-only throughout. Doctor, Nursing, Secretary: same directory plus the single Primary create action, row actions Secondary |
| Loaded with zero rows | Blank term: empty title only. Non-blank term: empty title plus search-miss description; typed term remains |
| Loaded with rows | One `ListRow` per patient in server order; two meta lines; one Secondary row action |
| Fetch error, web | Error line; no cache rows. Rows from an earlier success may remain below it until the next result |
| Fetch error, desktop | Error line stays. When the local database opens, cached rows replace the list with the same title, meta, and row action; undecodable values are skipped; with none, the empty region renders. If the database cannot open, the current list remains (empty on first load). Each input retries the API, then re-reads the cache, so the cache is not term-filtered |
| Detail or create navigation | Row action pushes `/patients/{id}`; Primary pushes `/patients/new`. Both are absent for the roles that cannot use them |
| `pending_sync` / `failed` states | Not rendered here. This screen reads no outbox status; the navbar owns FR-043 and `/prescriptions` owns those filters |

## 5. Style

Every value is a `context/Design.md` token already defined in `persona-sanus/assets/main.css` `:root`. No new color, font, icon package, CDN, webfont, or shadow.

### 5.1 Color

| Use | Value | Token |
|---|---|---|
| Page canvas, header, search, rows, Primary label | `#ffffff` | `--color-paper` |
| Titles, row title, input text | `#061d29` | `--color-ink` |
| Subtitle, meta, loading label, placeholder, Secondary border and text | `#425d6d` | `--color-mist` |
| Hairlines: row border, input border, empty border | `#e5e7eb` | `--color-hairline` |
| Primary fill, brand mark, focus ring | `#006e75` | `--color-deep-teal` |
| Primary hover fill | `#0b978e` | `--color-bright-teal` |
| Empty state surface | `#fffded` | `--color-cream` |
| Fetch error text and Alert glyph | `#4d0037` | `--color-plum` |
| Focus halo | `rgba(185, 255, 232, 0.55)` | `--color-mint-halo` |

Never `#000000`. Never a chromatic border: row, input, and empty borders use the hairline only; focus uses the ring and halo. Measured contrast (WCAG AA body needs ≥ 4.5): Ink on Paper 17.28; Mist on Paper 6.96; Paper on Deep Teal 6.02; Plum on Paper 15.32; Ink on Cream 16.89; Mist on Cream 6.80. The placeholder is Mist on Paper 6.96 and is not the only label. The focus indicator reaches ≥ 3:1 against Paper as a non-text indicator.

### 5.2 Typography

| Element | Family | Weight | Size / line-height |
|---|---|---|---|
| Page title (`h1`) | `--font-mint` | 500 | 22px / 1.22, Ink |
| Page subtitle | `--font-mint` | 400 | 14px / 1.5, Mist |
| Search label, visually hidden | `--font-mint` | 500 | 14px / 1.5, Ink |
| Search input and placeholder | `--font-mint` | 400 | 16px / 1.5, Ink |
| Primary and Secondary buttons | `--font-mint` | 500 | 16px / 1.22 |
| Row title / meta lines | `--font-mint` | 500 / 400 | 18px / 1.25 Ink; 14px / 1.5 Mist |
| Empty title / description, loading label, error line | `--font-mint` | 500 / 400 | 18px / 1.25 Ink; 14px / 1.5 Mist; 14px / 1.5 Plum (error 400) |

`--font-grenette` never appears on this page: it is display-only at 46px and above and is banned from app chrome and page titles.

### 5.3 Geometry

- Base unit 4px. PageShell padding: 48px top, 80px bottom, 24px inline; container max-width 1200px, centered; stack gap 24px. Page header: flex, space-between, align start, wrap, gap 16px; title-to-subtitle gap 4px; actions gap 12px, aligned to the inline end.
- Search: full stack width (`wide: true`); field wrapper column, gap 4px; input `min-height: 44px`, padding `8px 12px`, 8px radius, 1px hairline border. No extra search margin; the stack owns the vertical rhythm.
- Rows: list gap 12px; each row Paper with 1px hairline, 24px radius, `16px 20px` padding, flex, space-between, center, gap 16px; title-to-meta gap 2px; actions wrap with 12px gap and align self start when the row stacks. Row action: 8px radius, `12px 24px`, one line; the Primary matches it and sits in `.ui-page-head-actions`.
- Empty: Cream, 1px hairline, 24px radius, 32px padding, centered; description max 48ch and only when present. Loading: flex center, gap 8px, 16px padding.
- Responsive: below 640px the header and each row become a column; the row action moves to the inline start and keeps its label. The list never forces horizontal scroll and never becomes a card grid. Elevation: flat, hairlines and paper-on-paper contrast only; no drop shadow.

### 5.4 Iconography and imagery

`ErrorText` carries its Alert glyph at 20px with a 1.5px stroke, Plum; it is the page's only icon. No avatar, patient photo, document mock, QR, signature, stamp, or illustration renders. The navbar mark and sync dot are chrome, owned by `context/screens/done/Header.md` §5.4.

## 6. Accessibility

- The navbar is the banner landmark (`Header.md` §6). The page is one directory region; it adds no table semantics because `ListRow` is a card list, not tabular data.
- Exactly one `h1`: the page title in the directory state, the denial title in the denied state. No `h2` renders because no `SectionCard` exists. The session-check and redirect notices are transient status text, not headings.
- Tab order: navbar → create action when present → search input → each row action in row order. Row surfaces, meta lines, and the spinner are not focusable. There are no hidden controls in tab order.
- Focus: `1px solid var(--color-deep-teal)`, `outline-offset: 2px`, halo `box-shadow: 0 0 0 3px rgba(185, 255, 232, 0.55)`. The ring is visible on Paper and on Cream. No focus style is removed without replacement.
- The search field's accessible name is the label from §3.2, visually hidden; the placeholder is supplementary and cannot be the only name. `LoadingState` carries `role="status"` and `aria-live="polite"`; its spinner is `aria-hidden`. `ErrorText` carries `role="alert"`; the denial is a page heading read on navigation, not a duplicate alert.
- Status is never colour-only: the error includes the Alert glyph and a full sentence; empty is text; loading is text and a spinner; every row action has a visible label. Text meets AA: Ink on Paper 17.28; Mist on Paper 6.96; Paper on Deep Teal 6.02; Plum on Paper 15.32; Ink on Cream 16.89; Mist on Cream 6.80. The placeholder is backed by the label.
- Keyboard: the search is a native search input, actions are native links or buttons, hover and focus are independent, and no row-only click target exists. Touch: the row action keeps the 12px 24px button padding, and the 12px list gap prevents overlapping targets.
- Reduced motion: the `LoadingState` spinner stops under `prefers-reduced-motion`; no other animation exists. No information depends on motion.
- Empty and error results do not steal focus; the caret stays where the search input placed it. The cached desktop list is not announced as fresh data.

## 7. Implementation notes

- Shipped: `fn Patients()` at `persona-sanus/src/main.rs:3408` (`#[component]` at `:3407`), closing at `:3545`. It renders `PageShell id="patients-page"` with `PageHeader`, `SearchField`, `ListRow`, `EmptyState`, `LoadingState`, and `ErrorText` from `persona-sanus/src/ui/` — the plan step-13 adoption is landed; the bare `div#patients-page` and legacy `.page-head` / `.search-row` / `.patient-list` markup are gone.
- Signals and gates (`main.rs:3409-3482`): `query`, `patients`, `message`, and `loaded` (initialized false, `:3416`). While `!checked()` → `PageShell` + `LoadingState` (`Cargando...` default, `ui/feedback.rs:111`); no session → `navigator.replace(Route::AuthPage {})`; `actor` absent → `LoadingState` `Redirigiendo...`; `!can_read_patients()` → denial `PageHeader` `Acceso denegado` / `Necesitas un rol en la clínica para consultar pacientes.`
- The fetch effect (`main.rs:3418-3457`) is preserved: it returns while `!checked()` or no user, reads `query()`, calls `api::fetch_patients(&term)` (`api.rs:685`), and sets `loaded` true after the desktop fallback attempt and after the success/error row assignment (`main.rs:3455`). No debounce, no URL query parameter, no second fetch.
- Header (`main.rs:3491-3503`): `Pacientes` / `Busca por nombre o cédula dentro de tu clínica.`; the Primary `ButtonLink` `Nuevo paciente` renders only when `profile.can_write_patients()`.
- Search (`main.rs:3504-3510`): `SearchField` label `Buscar paciente`, placeholder `Nombre o cédula`, `wide: true`, value bound to `query`.
- Rows (`main.rs:3522-3542`): one `ListRow` per patient with title `full_name`, meta `Cédula: {…}` and `Nacimiento: {format_date(…)}` (`ui/format.rs:89`; `DD-MM-AAAA`, raw-text fallback on a malformed date), `—` when absent, and a Secondary `ButtonLink` `Ver ficha` to `Route::PatientDetail { id }`.
- Empty/loading/error (`main.rs:3511-3520`): `LoadingState` `Cargando pacientes...` while `!loaded()`; `EmptyState` `Sin pacientes` with description `No hay pacientes para esta búsqueda.` only for a non-blank trimmed term; fetch errors paint the frozen `No se pudieron cargar los pacientes.` in `ErrorText`, never the raw transport string.
- Desktop fallback preserved (`main.rs:3441-3452`): on API failure the `#[cfg(feature = "desktop")]` block opens the local DB, reads the unfiltered `desktop_local::list_patients`, skips undecodable values, and keeps the error line above the cached rows. `api::fetch_patients` and the local helper keep their signatures; the local helper remains unfiltered by term.
- Verify, in order: `cd persona-sanus && cargo check`; `dx build --release --fullstack`; `caveman-blocks run persona-smoke --dir persona-sanus --port 8091`; `python3 .blocks/markdown-table-check.py --path context/screens/done/Patients.md --strict`.
- Do not add: a URL query or history entry for search, a sort or count control, pagination or result totals, a second Primary action, create/edit/delete controls for Administrator, a patient photo or avatar, sex/phone/notes/timestamps in the row, a document or QR preview, a billing or plan surface, a signature or stamp asset, a rich-text affordance, a new endpoint or field, or a success toast that hides the desktop cache error.

## 8. Spec references

| Spec | Connection |
|---|---|
| `specs/done/15-O-ui-tokens-actions-and-formatting.md` | `15-O` supplies the tokens, `ButtonLink`, and format helpers (`format_date`) the list renders. |
| `specs/done/16-P-ui-forms-feedback-and-layout.md` | `16-P` supplies `PageShell`, `PageHeader`, `SearchField`, `ListRow`, `EmptyState`, `LoadingState`, `ErrorText`, and `Notice`, the list whole result surface. |
| `specs/done/21-U-ui-dashboard-settings-and-patients.md` | `21-U` adopts `Patients`: `PageShell id="patients-page"`, the `loaded` gate, one Primary `Nuevo paciente`. |
| `specs/done/25-Y-ui-legacy-css-cleanup-and-docs.md` | `25-Y` removes legacy page-ID selectors so the adopted list runs only on `.ui-*` classes. |
