# PersonaSanus — Templates (`/templates`)

| Field | Value |
|---|---|
| Source | `context/sitemap.md` §1.2 (`/templates` — Templates — list: doctor's own plus clinic master, administrator master; name, status and current version; actions New template, Use for prescription), §2 (Doctor own plus master, Administrator master, Nursing and Secretary no access), §3 flow 3 (`/templates/new` to editor to preview to publish), §4 M1 (forms, PDF, logo and one-image upload), §5 (resource-plural paths, no PHI in URLs, `es-EC`, deep links); `context/PRD.md` FR-020, FR-021, FR-024, FR-080–FR-085, D-04, D-06, D-21; `context/Design.md` (Paper, Ink, Mist, Hairline, Deep Teal, Bright Teal, Mint Glass, Cream, Plum; primary and outlined secondary buttons; green pill badge; 24px cards; 8px controls; flat elevation; no photography); `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` step 14 and Page adoption table row `Templates`, plus `PageShell`, `PageHeader`, `ListRow`, `TemplateStatusBadge`, `EmptyState`, `LoadingState`, `ErrorText`, `ButtonLink`; `context/screens/INDEX.md` §2.3 |
| Component | `Templates` — `persona-sanus/src/main.rs:3566-3678` (`#[component]` at `:3566`, `fn` at `:3567`, function closes at `:3677`). Reads `Signal<Option<SessionUser>>`, `Signal<Option<Me>>`, `Signal<bool>`, `Signal<Vec<api::Template>>`, `Signal<String>`, and the unset `refresh` signal. Target composition: planned `PageShell`, `PageHeader`, `ListRow`, `TemplateStatusBadge`, `EmptyState`, `LoadingState`, `ErrorText`, `ButtonLink` from `persona-sanus/src/ui/`; that directory is not shipped |
| Route | `/templates` — authenticated list (`Route::Templates {}` at `persona-sanus/src/main.rs:76-77`), inside `#[layout(Navbar)]`; no path parameter or query. Signed out, the effect replaces the route with `/login` (`Route::AuthPage {}`). Page guard `Me::can_manage_templates()` (`api.rs:68-70`): `administrator` or `doctor`; `nursing`, `secretary`, a null role, and a missing profile get the denial surface. The published-version deep link targets `/prescriptions/new?template_id={current_version_id}` (`Route::PrescriptionNew`), whose guard is doctor or nursing |
| Layout | `PageShell id="templates-page"` inside `#[layout(Navbar)]`, opened at `main.rs:51` and never closed; `/verify/:code` is declared at `:49-50`, before it. Root `div#templates-page.ui-page` > `.ui-container.ui-stack`; Paper canvas, hairline row borders; no full-bleed band, sidebar, or split view |
| Locale | `es-EC` on every visible string, punctuation included. `plantilla`, `receta`, `médico`, `clínica`, `versión`, `vigente`. No date or time renders here, so no `DD-MM-AAAA` or 24-hour surface exists |
| Status | ✅ Spec complete (sections 1–7). Route status: ✅ — sitemap §1.2: doctor own plus clinic master, administrator master; list name, status, current version; actions New template, Use for prescription. Shipped: auth-checked list, client role gate, owner and status labels, current-version line, edit and preview links for non-archived rows; no separate loading state or badge primitive. Target: plan-step adoption plus the published-version deep link; no editor, publish, archive, upload, or delete on this page |

The template shelf: a quiet index of the clinic's reusable prescription documents. It is a list, not an editor. This file freezes region order, copy, geometry, and states for the route; it introduces no route, endpoint, table, column, role, or permission.

## 1. Purpose

The screen performs five jobs, in order:

1. **Inventory.** Show the templates the signed-in actor may manage: the doctor's own templates with the clinic master, or the clinic master alone for an administrator. The server owns the scope.
2. **Orient.** Report each template's owner class, lifecycle status, and current version, so an operator can tell a personal draft from a published master or an archived record.
3. **Create.** Offer the sole Primary entry to the existing creation route, which makes an empty draft and hands off to the editor.
4. **Inspect or change.** Open a non-archived template in the existing editor, or in the read-only A4 preview.
5. **Start a prescription.** For a published template whose current version is published, offer the sitemap deep link into the issue flow carrying that version id.

It never renders:

- Patient PHI. Templates hold brand content, letterhead, fixed body and footer text, and variable placeholders; no patient name, cédula, diagnosis, medication, birth date, phone, or photo appears here, including in an example row, alt text, tooltip, or URL.
- Billing, price, trial, plan, entitlement, invoice, quota usage, or `sync_locked` copy (D-04); v1 has no billing state.
- A signature or stamp image, field, or placeholder (D-06). The printed signature line is footer content owned by the editor, not an asset on this list.
- A rich-text editor, formatting toolbar, `contenteditable` surface, HTML field, or markdown hint (D-21).
- An embedded template editor, A4 preview, image upload or picker, medication editor, publish, archive, delete, duplicate, or rename control. Publishing lives in `/templates/:id/edit`; image selection lives in the editor and the clinic library; deletion is not a v1 action.
- A search field, owner filter, status filter, clinic picker, sort control, result count, pagination, select-all, or bulk action. The page paints the server's ordered rows without client filtering.
- A table. The list is a stack of rows, so there is no caption, column header, or `th scope`.
- A second filled action. The only Primary is the header's creation action; every row action is Secondary and an archived row has none.

## 2. Structure

One Paper page, a stack of 24px gaps under the navbar. The header remains mounted while the list area swaps between loading, empty, error, and rows. There is no card heading and no inner section card; the row cards are the only repeated surface.

| Order | Region | id / class | Band tone | Contents |
|---|---|---|---|---|
| — | Navbar (layout) | `#navbar` · `header.ui-navbar` | Paper `#ffffff`, bottom hairline | Owned by `context/screens/done/Header.md` §2–§5; not restated here |
| 1 | Page shell | `#templates-page` · `div.ui-page` > `.ui-container.ui-stack` | Paper | 48px top, 80px bottom, 24px inline; container max 1200px; 24px stack gap |
| 2 | Page header | `header.ui-page-head` · `h1.ui-page-title` · `p.ui-page-subtitle` · `.ui-page-head-actions` | Paper | Page title and subtitle, then one Primary `ButtonLink` at the inline end |
| 3 | Feedback | `.ui-error` | Paper | `ErrorText` fetch-failure line, Alert icon, `role="alert"`; only when the message signal is non-empty |
| 4 | Template list | `div.ui-list` holding one `ListRow` per template | Paper | Row title `template.name`; meta owner and template status; version line; `TemplateStatusBadge`; trailing Secondary actions |
| 5 | Empty or loading | `.ui-empty` · `.ui-loading` | Cream `#fffded` · Paper | `EmptyState` title only, or `LoadingState` spinner and label |
| 6 | Denied surface | Same shell and header regions; no list, empty, or loading | Paper | `PageHeader` with the denial title and reason; no action, row, or back link |

Regions 4–6 are mutually exclusive; feedback region 3 sits above whichever one renders. Nothing scrolls horizontally. The list is a card stack, not tabular data.

### 2.1 Session and role

`checked` is the shell's session-check signal, `user` is `Signal<Option<SessionUser>>`, and `actor` is `Signal<Option<Me>>`. `Me::can_manage_templates()` is the page gate (`api.rs:68-70`); it is true only for `administrator` and `doctor`.

| Session finish / role | Rows | Primary | What changes; what does not |
|---|---|---|---|
| Check in flight (`!checked`) | no | no | `PageShell` plus one `LoadingState`; no header, rows, empty state, or error. Root id and locale are stable |
| Checked, no session | no | no | The effect replaces the route with `/login`; the paint is the redirect loading notice, never the shelf |
| Checked, session present, `actor` is `None` | no | no | `PageShell` plus the redirect loading notice; passive, no retry and no second redirect |
| Role `null` | no | no | Denial surface: the title and reason. No list request result, create, row action, or deep link |
| `nursing` or `secretary` | no | no | Denial surface: the title and reason. No list request result, create, row action, or deep link |
| `administrator` | clinic master | yes | Full header and rows; the server returns clinic-owned templates. Create makes a clinic master. No deep link: an administrator cannot create prescriptions |
| `doctor` | own plus clinic master | yes | Full header and rows; the server returns the doctor's own templates then the clinic master. Create makes a doctor-owned template. Eligible published rows expose the deep link |

Role changes only the server row scope and whether the published-version deep link exists. It never changes region order, owner labels, status badges, version formatting, or the header Primary for an allowed manager. The server re-checks authorization; the client guard is UX, not the security control.

### 2.2 Loading, empty, error, and return

| State | What renders | What stays fixed |
|---|---|---|
| First authenticated paint, `loaded` false | Header and Primary, then one `LoadingState` in the list slot | Root id, title, subtitle, Primary, 24px stack rhythm |
| Fetch resolves with zero rows, no error | Header and Primary, then `EmptyState` title only; no description | Root id, header, empty copy, list-slot position |
| Fetch fails | Header and Primary, then `ErrorText`; empty state suppressed | Root id, header, Primary, list-slot position |
| Fetch resolves with rows | One `ListRow` per template in server order; loading and empty disappear | Root id, header, Primary, row geometry |
| Return after publish | The route mounts again and the effect fetches once; the published version is current in the row. No toast, no optimistic row, no in-place mutation | Root id, header, Primary, effect, request |
| While still mounted | No polling, stream, automatic refresh, or refresh button. A change elsewhere appears on the next mount, not by timer | Every region |

The list starts in loading, not empty. This prevents the “no templates” card from flashing before the first fetch resolves. An error suppresses the empty state even though the row vector is empty. Feedback is page-level, not a row badge.

### 2.3 Row status and current version

The template status drives the badge, the meta line, edit and preview, and the deep link. The current-version object drives the version line and the target id. They are read separately; the page never turns a template status into a version status or sends a template id as a version id.

| `template.status` | Badge and meta | Version line | Edit and preview | Deep link |
|---|---|---|---|---|
| `draft` | `Borrador` | current version when present, else `Sin versión publicada.` | visible | absent |
| `published` | `Publicada` | current version when present, else `Sin versión publicada.` | visible | only for a published current version and an actor who can create prescriptions |
| `archived` | `Archivada` | current version when present, else `Sin versión publicada.` | hidden | absent |
| other | `Desconocido` | current version when present, else `Sin versión publicada.` | visible | absent |

A present version prints `Versión vigente: {n} ({status})`, where `{status}` is `Borrador`, `Publicada`, `Archivada`, or `Desconocido` through `version_status_label`. An absent version, including a stale `current_version_id` that resolves to no object, prints `Sin versión publicada.`. The badge is `TemplateStatusBadge` with the template status; it sits beside the title and is not replaced by the version status. The meta line repeats the template status as words, so status is never colour-only. Version numbers are stored integers, never indexes, never zero-padded, never recalculated. “Vigente” names the pinned current version; the page does not claim that a draft is published.

An archived row keeps its title, meta, badge, and version line but renders no action. Draft and unknown rows keep edit and preview but never get the deep link. A published row gets the deep link only when a current version exists, its status is exactly `published`, and the actor passes `Me::can_create_prescriptions()` (`api.rs:72`). On this route that means a doctor: nursing is already denied by the page guard, and an administrator can manage masters but cannot issue a prescription. Missing, stale, draft, archived, and unknown versions produce no link.

### 2.4 Web and desktop

Both builds render the same component, id, copy, order, and gates. There is no separate desktop list in the shipped component, and none is added here.

- **Web:** `api::fetch_templates()` is the only source. A failure shows `ErrorText`; no rows are fabricated.
- **Desktop:** the same request is the only source. The component has no `#[cfg(feature = "desktop")]` branch and never opens the local database, so it neither reads nor displays a cached template list and cannot show a stale version number. If the server is unreachable, the result is the fetch-error state, not a local list.
- **Offline and cache loss:** there is no offline list branch and no cached rows whose failure a local success could hide. Every action here is navigation; editing, image selection, draft saving, publishing, and archiving stay in the editor routes and their own surfaces.

### 2.5 Denied and empty distinction

The denial surface replaces everything below the chrome, not the navbar. It is a single `PageHeader`: `Acceso denegado` is the only `h1`, the reason is the subtitle, and there are no actions, rows, counts, or back link. The navbar is the return path. An empty clinic is not a denial: it keeps the title, subtitle, Primary, and empty card. A fetch failure is not a denial: it keeps the header and Primary and shows the error line. A signed-out request is a redirect, not a denial surface.

## 3. Copy text

All visible strings verbatim, `es-EC`; punctuation is part of the copy. Chrome copy belongs to `context/screens/done/Header.md` and is not repeated here. `{owner}` is the owner label, `{n}` is the current version number, and `{status}` is a status label from §3.2.

### 3.1 Page header, primary, loading, denial

| Element | Copy | Target |
|---|---|---|
| Page title (`h1`) | `Plantillas` | — |
| Page subtitle | `Plantillas del médico y maestras de la clínica, con su versión vigente.` | — |
| Primary action, allowed managers only | `Nueva plantilla` | `/templates/new` (`Route::TemplateNew {}`) |
| Session-check loading label | `Cargando...` | — |
| Redirect loading label | `Redirigiendo...` | — |
| Denied title (`h1`) | `Acceso denegado` | — |
| Denied reason | `Solo doctor y administrador pueden gestionar plantillas.` | — |

The denied state renders no secondary action: no `Volver`, no `Iniciar sesión`, and no link to settings or another list.

### 3.2 Owner and status labels

| Element | Copy |
|---|---|
| Owner meta for `owner_type` `doctor` | `Plantilla del médico` |
| Owner meta for `owner_type` `clinic` | `Maestra de la clínica` |
| Owner meta for any other `owner_type` | `Plantilla` |
| Badge and meta status for `draft` | `Borrador` |
| Badge and meta status for `published` | `Publicada` |
| Badge and meta status for `archived` | `Archivada` |
| Badge and meta status for any other value | `Desconocido` |

The meta line is exactly `{owner} · {status}`, with the middle dot and spaces shown. The same four status words label the current version through `version_status_label`; no fifth version state is added. An unknown owner and unknown status render explicitly, never hidden or guessed.

### 3.3 Row title and version line

| Element | Copy |
|---|---|
| Row title | `template.name` shown verbatim, never as sample data |
| Version line, version present | `Versión vigente: {n} ({status})` |
| Version line, version absent | `Sin versión publicada.` |

The owner id, template id, timestamps, `doc_json`, `variables_spec`, theme, and asset ids are never painted. `{n}` is the integer version number; `{status}` is the current version label from §3.2.

### 3.4 Row actions

| Element | Copy | Target |
|---|---|---|
| Deep link, Secondary, eligible published rows | `Usar para receta` | `/prescriptions/new?template_id={current_version_id}` (`Route::PrescriptionNew`) |
| Edit, Secondary | `Editar` | `/templates/{template.id}/edit` (`Route::TemplateEdit { id }`) |
| Preview, Secondary | `Vista previa` | `/templates/{template.id}/preview` (`Route::TemplatePreview { id }`) |

The use action's query value is the published current version id, never the template id. The row title supplies context for repeated labels; where a link list would be ambiguous, the accessible name appends the template name: `Editar {name}`, `Vista previa de {name}`, `Usar {name} para una receta`. Those patterns are not visible extra copy and add no punctuation to the labels.

### 3.5 Empty and error

| Element | Copy |
|---|---|
| Empty title | `Todavía no hay plantillas.` |
| Fetch error fallback | `No se pudo cargar la lista de plantillas.` |
| Session gate, unchecked | `Cargando...` |
| Actor gate, checked without a profile | `Redirigiendo...` |

`EmptyState` has no description: the title is the complete message and the Primary above it is the next step. The fetch fallback is the page-owned line in `ErrorText`; raw transport text, status codes, and parsing detail are not page copy. No success, archive, publish, upload, quota, or offline toast exists on this list.

## 4. Displayed information

### 4.1 What the screen shows

| Element | Source | Value shown |
|---|---|---|
| Page title | static | `Plantillas` |
| Page subtitle | static | The list purpose, naming doctor templates and clinic masters with their current version |
| Primary | `actor: Signal<Option<Me>>`, `Me::can_manage_templates()` (`api.rs:68`) | one Primary `ButtonLink` to the creation route; the only filled teal action |
| Session gate | `checked`, `user`, `actor` | loading label before the check; redirect label when checked without a profile; auth replace when the session is missing |
| Role gate | `Me::can_manage_templates()` | header and rows only for `administrator` and `doctor`; denial title and reason otherwise |
| Row scope | `api::fetch_templates()` (`api.rs:801`) to `service::list_templates` (`service.rs:1275-1317`) | administrator: clinic master; doctor: own then clinic master; no page-side owner or clinic filter |
| Row order | returned `Vec<api::Template>` | server order; doctor and clinic groups each arrive newest-created first (`db.rs:1350-1361`); no client sort by name or version |
| Row title | `Template.name` | stored template name verbatim; the only identifier the row leads with |
| Owner meta | `Template.owner_type` via `template_owner_label` (`main.rs:4241-4247`) | `Plantilla del médico`, `Maestra de la clínica`, or `Plantilla` for another value |
| Status meta and badge | `Template.status` via `template_status_label` (`main.rs:4223-4230`) and `TemplateStatusBadge` | the status word in both the meta line and the pill; colour is supplementary |
| Current version | `Template.current_version` and `TemplateVersion.version_no` | present: the version number and one status label; absent: the explicit no-published-version line |
| Version status | `TemplateVersion.status` via `version_status_label` (`main.rs:4232-4239`) | `Borrador`, `Publicada`, `Archivada`, or `Desconocido` |
| Row links | `Template.id` and `Template.current_version.id` | edit and preview use the template id; the use link uses the current-version id, never the template id |
| Empty slot | `templates` empty, `loaded` true, `message` empty | empty title only; no description, no fabricated count |
| Loading slot | `loaded` false | spinner and loading label; no skeleton, placeholder card, or fake row |
| Error slot | non-empty `message` after settlement | frozen fallback in `ErrorText`; list and empty state suppressed |
| Denied slot | actor fails `can_manage_templates()` | denial heading and reason; no list result painted |

`api::Template` also carries `owner_id`, `current_version_id`, `created_at`, `updated_at`, and `versions` (`api.rs:780-794`). The list paints none of them. It never reads `TemplateVersion.doc_json`, `variables_spec`, or `theme`; the row's only version facts are `version_no` and `status`. Template names and brand metadata are authorized clinic configuration, not patient PHI, but they stay behind the authenticated route.

### 4.2 Hard rules

- Server-side authorization is the boundary. The page gate is UX; the API requires an authenticated actor, and `service::list_templates` derives owner scope from that actor. The client sends no clinic, owner, doctor, or role filter and never widens the list.
- The page adds no client-side scope rule. It does not compare `owner_id` with the session, split “mine” and master, or drop clinic rows for a doctor. It paints the returned collection in server order. The absence of an owner filter is the contract, not unfinished UI.
- The deep link is a version link, not a template link. It exists only for a published template whose `current_version` is present and `published`, and only for an actor who can create prescriptions. Draft, archived, unknown, versionless, stale, and non-published-current rows have no use action. It navigates; it never publishes, clones, renames, or opens an editor.
- Archived rows are read-only here: edit and preview disappear, and no replacement action appears. Draft and published rows keep both. The page never offers archive, unarchive, delete, duplicate, or restore, and it never publishes from a row.
- No PHI in the route or its metadata. The URL is `/templates`; row links carry only opaque template ids in the path or the opaque current-version id as `template_id`. No patient identity, cédula, diagnosis, medication, template body, or version document travels in a URL, title, analytics event, log line, or QR code.
- Templates are brand content plus variables, not a patient record. The page does not resolve a variable value, expand `patient.*` or `medications[]`, or show a clinical document preview. A name is painted only because the operator must choose a template.
- Plain text only (D-21): names, owner labels, version labels, and buttons are text. No HTML is injected, no rich-text surface exists, and `doc_json` is never parsed or rendered here.
- No billing, price, trial, plan, image quota, or `sync_locked` copy (D-04). The v1 one-image quota belongs to the editor and clinic settings; this page shows no upload control and no quota badge (FR-080–FR-085).
- No signature or stamp image, field, or placeholder (D-06). The printed signature line is footer content in the template document, not a control or asset here.
- At most one filled teal action per viewport: the header Primary when the actor can manage templates; zero in loading, denial, and redirect surfaces. Use, edit, and preview are Secondary. There is no icon-only or row-only click target.
- One `h1` per settled surface: `Plantillas` in the allowed list, loading, empty, and error states; `Acceso denegado` in the denied state. No `h2` renders because the page has no `SectionCard`.
- Status is never colour-only: the badge carries text, the meta line repeats it, and the version line prints its own status. A missing owner or status gets an explicit fallback, never a blank or a guess.
- No table, client cache merge, polling, or new endpoint. `api::fetch_templates()` is the only read; row actions are router navigation only. The list never claims a write succeeded.

### 4.3 Dynamic states

| State | Screen behaviour |
|---|---|
| Session check in flight | `PageShell` plus the session loading label; no header, rows, empty state, error, or API call |
| Checked, signed out | Effect replaces the route with `/login`; only the redirect loading notice remains. The back button cannot reveal a stale shelf because the route was replaced |
| Checked, session, `actor` still `None` | Redirect loading notice; no header, rows, or retry |
| Role `null`, nursing, or secretary | Denial surface only: heading and reason. No Primary, list, row link, or request result |
| Administrator | Header and Primary; clinic master rows from the server; edit and preview on non-archived rows; no use link |
| Doctor | Header and Primary; own then clinic master rows from the server; edit and preview on non-archived rows; use link on eligible published rows |
| First load, `loaded` false | Header and Primary plus loading state; empty and error withheld |
| Fetch resolves with zero rows | Empty title only; Primary stays; no description, count, or sample row |
| Fetch fails | Error line with the frozen fallback; Primary stays; empty state suppressed |
| Row `draft` | Neutral `Borrador` badge; owner and `Borrador` meta; version line when present; edit and preview; no use link |
| Row `published`, published current version | Mint `Publicada` badge; owner and `Publicada` meta; version line; edit and preview; use link when eligible |
| Row `published`, current version absent | Mint badge plus `Sin versión publicada.`; edit and preview; no use link |
| Row `published`, current version draft or archived | Mint badge; version line with that version status; edit and preview; no use link |
| Row `archived` | Cream `Archivada` badge; version line when present; no actions, no link, no row tab stop |
| Row status unknown | Neutral `Desconocido` badge; unknown owner fallback; edit and preview; no use link; version status labelled separately |
| Long template name | Title wraps at word boundaries and breaks a long unbroken token as a last resort; no truncation, ellipsis-only tooltip, clipping, or horizontal scroll. The row grows vertically and actions move below the text at narrow width |
| Deep-link activation | Navigates to `/prescriptions/new?template_id={current_version_id}`; the version id seeds the template picker. No local write, publish, or template mutation occurs |
| Return after publish | The route mounts again and the effect fetches once; the published version appears as current with the use link when eligible. No inserted row, toast, or background refresh before that fetch resolves |
| Return after archive | The row appears archived with no edit, preview, or use action; the list offers no confirm panel or undo |
| Desktop, server unreachable | Fetch error; no cached rows, because the page never reads the local template cache. Navigation links remain links and their destinations own their error states |
| Desktop, server reachable | Same rows, order, and labels as web; geometry follows viewport width, not platform |
| Reduced motion | The loading spinner stops; no other animation exists; no state depends on motion |

## 5. Style

Every value is a `context/Design.md` token declared in `persona-sanus/assets/main.css` `:root`. No new color, font, icon package, CDN, or webfont.

### 5.1 Color

| Use | Value | Token |
|---|---|---|
| Page canvas, row cards | `#ffffff` | `--color-paper` |
| Page title, row title, status text | `#061d29` | `--color-ink` |
| Subtitle, owner/version meta, hint | `#425d6d` | `--color-mist` |
| Card hairline, list divider | `#e5e7eb` | `--color-hairline` |
| Primary fill, focus ring | `#006e75` | `--color-deep-teal` |
| Primary hover | `#0b978e` | `--color-bright-teal` |
| Empty state surface | `#fffded` | `--color-cream` |
| Draft/archived badge surfaces | `#fffded` / `#ffc3e6` | `--color-cream`, `--color-blossom` |
| Published badge | `#b9ffe8` | `--color-mint-glass` |
| Failed text, Alert glyph | `#4d0037` | `--color-plum` |
| Focus halo | `rgba(185,255,232,0.55)` | mint glass at 55% |

Never `#000000`. Never a chromatic card border: the row outline is the hairline; only the badge fill may be chromatic. Status words carry the meaning, not the badge color.

Measured contrast: Ink/Paper 17.28 · Mist/Paper 6.96 · Paper/Deep Teal 6.02 · Ink/Cream 16.89 · Mist/Cream 6.80 · Ink/Mint 15.25 · Plum/Paper 15.32. AA for body; AAA where it clears.

### 5.2 Typography

| Element | Family | Weight | Size / line-height |
|---|---|---|---|
| Page title (`h1`) | `--font-mint` | 500 | 22px / 1.22 Ink |
| Page subtitle | `--font-mint` | 400 | 14px / 1.5 Mist |
| Primary and Secondary actions | `--font-mint` | 500 | 16px / 1.22 |
| Row title | `--font-mint` | 500 | 18px / 1.25 Ink |
| Row meta | `--font-mint` | 400 | 14px / 1.5 Mist |
| Version line | `--font-mint` | 400 | 14px / 1.5 Mist |
| Status badge | `--font-mint` | 500 | 13px / 1.22 |
| Empty title | `--font-mint` | 500 | 18px / 1.25 Ink |
| Error text | `--font-mint` | 400 | 14px / 1.5 Plum |

No `--font-grenette`: application chrome, not a 46px+ editorial opener. Version numbers are normal figures, not a display face.

### 5.3 Geometry

- 4px base. `PageShell`: 48px top, 80px bottom, 24px inline; `.ui-container` max 1200px; `.ui-stack` 24px gap.
- `PageHeader`: flex row, space-between, wrap, 16px gap, align start; subtitle 4px below title; Primary at the inline end, 8px radius, `12px 24px`.
- List: `div.ui-list`, 12px gap; each `ListRow` Paper, 1px hairline, 24px radius, `16px 20px` padding; title/meta group `min-width: 0`; title-to-meta 2px; meta-to-version 2px; trailing actions flex, centered, 12px gap, wrap.
- Badge: 9999px radius, `4px 12px`, 13px / 1.22, inline after the title or at the row's trailing edge; it never shrinks below its word.
- Below 860px: title and meta take the full row width; actions move under the text, start-aligned. Below 640px: title and actions stack; each link keeps a 44px target and the row grows instead of clipping.
- Empty: Cream, 24px radius, 1px hairline, 32px padding, centered, max 48ch for a description if one is ever added (none here). Loading: flex center, 8px gap, 16px padding.
- Elevation: flat. No card, row, or button shadow; the focus halo is the only ring. No horizontal scroll at 320px.
- The page is a single column, symmetric, with no sidebar, split pane, table header, sticky toolbar, or density toggle.

### 5.4 Iconography and imagery

Line icons only, 1.5px stroke, 20px default, Ink or Plum: the Alert glyph in the fetch `ErrorText`. The Primary, row links, and badges carry no icon; a template row is text and status, not a document thumbnail. No clinic logo, template preview, A4 sheet, QR, signature, stamp, patient photo, avatar, illustration, or chart renders on this page. The navbar mark belongs to `Header.md` §5.4.

## 6. Accessibility

- Landmarks: the navbar is the banner (`Header.md` §6). `PageShell` is the content region. The list is a real list: `ul` > `li`; each row title is the item's accessible context. The denial surface is a `PageHeader`, not a second banner and not a dialog.
- Headings: one `h1`, `Plantillas`, in allowed, loading, empty, and error states; one `h1`, `Acceso denegado`, in the denied state. No `h2` because no `SectionCard`. No skipped level and no heading used for visual size.
- Tab order: `Nueva plantilla` → each non-archived row's `Usar para receta` when present → `Editar` → `Vista previa`, in row order. An archived row adds no tab stop; its text remains readable but its actions are absent, not disabled. No focus trap and no positive `tabindex`.
- Focus: `outline: 1px solid var(--color-deep-teal); outline-offset: 2px; box-shadow: 0 0 0 3px rgba(185,255,232,0.55)` on the Primary and every row link. The ring is visible on Paper and Cream and is never clipped by the row radius.
- Labels: `Nueva plantilla`, `Editar`, `Vista previa`, and `Usar para receta` are visible text, never icon-only. Repeated row actions take the template name into their accessible name as §3.4 says. The owner/status meta and version line are text, not a `title` tooltip.
- Status: the badge word and the meta line both name the state; color is supplementary. An unknown status is printed `Desconocido`, never blank. The version line prints `Sin versión publicada.` rather than hiding the absence.
- Empty and error: `EmptyState` is a heading plus text; `ErrorText` is `role="alert"` and is inserted, not focus-moved. Loading is `role="status"` with `aria-live="polite"`; the spinner is `aria-hidden`. The empty state does not claim a filter miss when no filter exists.
- Keyboard and touch: every action is a native link or button; there is no row-click-only surface, no drag, and no hover-only menu. Each target is at least 44px; the action gap is 12px.
- Reduced motion: the spinner stops under `prefers-reduced-motion: reduce`; no other animation is introduced.
- Zoom/reflow: at 200% and 320px the header stacks, each row becomes one column, actions wrap below the text, and the page does not scroll horizontally.

## 7. Implementation notes

The shipped component is `fn Templates()` at `persona-sanus/src/main.rs:3567`, `#[component]` at `:3566`, closing before `TemplateNew` at `:3679`. It reads `user`, `actor`, and `checked` contexts, holds `templates: Signal<Vec<api::Template>>`, `message`, and `refresh: Signal<i32>`, and runs one effect keyed on `refresh`.

Current behavior, in order:

- `!checked()` returns `div#templates-page` with `p` `Cargando...`; a missing `actor()` after the check returns the same root with `Redirigiendo...`.
- `!profile.can_manage_templates()` returns `div#templates-page` with `div.notice` (`h4` `Acceso denegado`, the reason `Solo doctor y administrador pueden gestionar plantillas.`) and no list link.
- The effect calls `api::fetch_templates()` and stores the result or the error in `message`. It has no `loaded` signal, so an empty vector paints the empty hint immediately after the session gate and before the fetch resolves; target adds `loaded` as §2.2 says.
- The raw root has no `PageShell`; the header is `.page-head` with `h4` `Plantillas`, the shipped subtitle `Plantillas del médico y maestras de la clínica, con su versión vigente.`, and `Link.button` `Nueva plantilla`.
- A non-empty `message` paints `p.error`; the list is `.template-list`; each row is `div.template-row` with two or three `p.patient-meta` lines and a trailing `.actions` div.
- Non-archived rows render `Link.button.secondary` `Editar` to `Route::TemplateEdit { id }` and `Vista previa` to `Route::TemplatePreview { id }`. Archived rows render no actions.
- The list paints `template_owner_label(&template.owner_type) · template_status_label(&template.status)` and either `Versión vigente: {version.version_no} ({version_status_label(&version.status)})` or `Sin versión publicada.`

Plan step 14 adoption:

- Root becomes `PageShell id="templates-page"`; the raw page head becomes `PageHeader` with the existing `h4` as `title` and the existing `p` as `subtitle`; the raw `Link.button` becomes a Primary `ButtonLink` `Nueva plantilla` in the header actions.
- Add `loaded: Signal<bool>` initialized false, set true in both the success and error arms of the fetch, after the vector assignment. The list slot renders `LoadingState` while false; `EmptyState` only when `loaded && rows.is_empty() && message.is_empty()`; `ErrorText` when the message is non-empty, with the empty state suppressed.
- Replace `.template-list`/`template-row` with one `ListRow` per template: title `template.name`; meta `[owner_label, status_label]`; a trailing `TemplateStatusBadge` carrying the template status; the version line as the row's supporting text. Do not add a second badge inside the meta line; the meta words and the badge already repeat the state deliberately.
- Actions remain Secondary `ButtonLink`s: `Editar`, `Vista previa` when the template is not archived. Add `Usar para receta` only when `template.status == "published"`, the current version exists and its status is exactly `"published"`, and `profile.can_create_prescriptions()`; target `Route::PrescriptionNew { patient_id: None, template_id: Some(current_version_id) }`. The query carries the version id, never `template.id`.
- Translation ownership: the page uses `template_owner_label`, `template_status_label`, and `version_status_label` from `main.rs`; target moves them to `src/ui/format.rs` unchanged as part of plan step 3. Do not fork them here.
- The fetch error currently lands in `message` and is painted raw. Target keeps the signal for diagnostics but paints the frozen `es-EC` fallback from §3.5. A fetch failure is not an empty list.
- There is no desktop cache branch in this component; do not add one. The local template cache belongs to the editor and the desktop sync layer, not to this index.

Verification commands, from the repo root; none is recorded here as executed:

```sh
cd persona-sanus && cargo check
dx build --release --fullstack
caveman-blocks run persona-smoke --dir persona-sanus --port 8091
python3 .blocks/markdown-table-check.py --path context/screens/done/Templates.md --strict
```

The smoke script covers authenticated navigation; a browser pass is needed for the three row states (draft, published with a current version, archived) and for the administrator vs doctor scope.

- Do not add: an embedded editor, publish, archive, delete, duplicate, rename, restore, upload, image picker, quota badge, or medication control.
- Do not add: search, owner filter, status filter, clinic picker, sort, result count, pagination, select-all, bulk action, saved view, polling, or background refresh.
- Do not add: a table, caption, `th scope`, chart, thumbnail, A4 mock, QR, patient photo, clinic logo preview, signature, or stamp.
- Do not add: patient PHI, a patient field, a patient picker, or a pre-filled variable value.
- Do not add: billing, price, trial, plan, entitlement, image quota, or `sync_locked` copy.
- Do not add: rich text, WYSIWYG, `contenteditable`, HTML payload, or markdown rendering.
- Do not add: a second Primary, a filled row action, an icon-only control, or a row click that is not a link.
- Do not add: a new route, endpoint, table, column, role, permission, or a client-side clinic/owner filter.

## 8. Spec references

| Spec | Connection |
|---|---|
| `specs/done/15-O-ui-tokens-actions-and-formatting.md` | `15-O` supplies the tokens, `ButtonLink`, and format helpers (`template_status_label`, `template_owner_label`) the list renders. |
| `specs/done/16-P-ui-forms-feedback-and-layout.md` | `16-P` supplies `PageShell`, `PageHeader`, `ListRow`, `EmptyState`, `LoadingState`, and `Notice`, the list result surface. |
| `specs/done/18-R-ui-domain-and-billing.md` | `18-R` builds `TemplateStatusBadge`, the status badge each row renders. |
| `specs/done/22-V-ui-templates.md` | `22-V` adopts `Templates`: `PageShell id="templates-page"`, the `loaded` gate, one Primary `Nueva plantilla`. |
| `specs/done/25-Y-ui-legacy-css-cleanup-and-docs.md` | `25-Y` removes legacy page-ID selectors so the adopted list runs only on `.ui-*` classes. |

