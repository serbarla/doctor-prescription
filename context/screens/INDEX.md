# PersonaSanus — Screen specs index

| Field | Value |
|---|---|
| Source | `context/sitemap.md` §1.1–§1.3 (routes, non-routes), §2 (navigation by role), §3 (flows), §4 (current vs. target), §5 (conventions); `context/Design.md`; `context/PRD.md`; `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` (Page adoption table); the `Route` enum at `persona-sanus/src/main.rs:47-95` |
| Purpose | Track which `context/screens/<Name>.md` specs exist and which are missing, and fix naming, shape, and write order for the ones not written yet. This file defines no screen: every screen's structure, copy, style, and states live in its own file |
| Doc status key | ✅ spec complete (sections 1–7 written) · 🟡 partial (file exists, sections incomplete) · ⬜ missing (no file) |
| Route status key | Copied verbatim from `context/sitemap.md` §1.1/§1.2 — ✅ implemented · 🟡 partial · ⬜ planned. It tracks **code**, not this doc set; the two axes are independent (`Footer.md` is a complete spec with no shipped component; `SettingsUsers` ships while its sitemap cell still reads ⬜) |
| Component column | Dioxus component and its current definition line in `persona-sanus/src/main.rs` (6,177 lines) |
| Count | 24 specs in scope — 23 written, 1 deferred (`SettingsBilling.md`, v1.1) |
| Locale | `es-EC` on every quoted label. `receta`, `cédula` keep Ecuador Spanish spelling |
| Status | ✅ Complete for v1 — 23 of 24 written. `SettingsBilling.md` stays deferred by PRD D-04/D-14 |

## 1. Rules for this doc set

### 1.1 Naming

- File name = the Dioxus component of the `Route` enum, PascalCase: `/prescriptions/:id/preview` → `PrescriptionPreview.md`. Route params never appear in a file name.
- Frozen exceptions: `Header.md` is the `Navbar` component (`main.rs:1377`); `Footer.md` is the planned public `Footer` band, which has no component yet. Do not rename either.
- One file per screen. No `Patients-list.md` / `Patients-detail.md` splits, no per-role files, no `.md` for a component or a CSS class — components are specified inside the screen file that uses them.

### 1.2 Shape (identical for every file)

1. H1 `PersonaSanus — <Name> (<route or surface>)`, then the header field table. Rows: `Source`, `Component`, `Route` (chrome files use `Rendered on` / `Not rendered on` instead), `Layout`, `Locale`, `Status`. `Source` names the sitemap sections, the PRD sections/FRs, and the `Design.md` blocks the file draws on.
2. `## 1. Purpose` — what the screen is for, in ordered jobs, plus what it must never render (PHI rules, billing copy per D-04, signature/stamp images per D-06, rich text per D-21).
3. `## 2. Structure` — ordered region table: order, region, `id` / class, band tone, contents. Subsection per state group (per role, per status, per surface).
4. `## 3. Copy text` — every user-facing string verbatim in `es-EC`, punctuation included, in tables keyed by region.
5. `## 4. Displayed information` — `4.1` what the screen shows and where each value comes from; `4.2` hard rules (PHI, immutability, guard behaviour); `4.3` dynamic states (loading, empty, error, offline, status transitions).
6. `## 5. Style` — `5.1` Color, `5.2` Typography, `5.3` Geometry, `5.4` Iconography and imagery, always token-referenced (`--font-mint`, `--spacing-24`, …), never a free-form hex outside `context/Design.md`.
7. `## 6. Accessibility` — landmarks, heading order, tab order, focus, labels, contrast.
8. `## 7. Implementation notes` — current shipped code and what changes, the exact verify commands (`cargo check`, `dx build --release --fullstack`, `curl`, `caveman-blocks run persona-smoke …`), and the closing **Do not add** list.

### 1.3 Authority order (never restate, always reference)

| Question | Owner |
|---|---|
| Route, roles, guard, status | `context/sitemap.md` |
| Feature scope, v1 vs. v1.1, FR ids | `context/PRD.md` |
| Tokens, palette, type, elevation, do/don't | `context/Design.md` |
| Component names, `PageShell` ids, the one primary action per page | `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` |
| Region order, copy strings, geometry, states | the screen file itself |

A screen file never introduces a route, an endpoint, a table, a column, or a role. Chrome strings belong to `Header.md` / `Footer.md`; a page file points at them (`Home.md` §3.1) instead of repeating them. Shared primitives are defined once in the UI plan and only *used* by screen files.

## 2. Index

### 2.1 Shell chrome (no route)

| Screen doc | Surface | Component · `main.rs` | Route status | Screen spec |
|---|---|---|---|:---:|
| `Header.md` | `Navbar` band — `/`, `/login`, and every shell route; not `/verify/:code` | `Navbar` · `:1377` | 🟡 | ✅ |
| `Footer.md` | Public footer band (`/`, `/login`, `/verify/:code`) | not shipped | ⬜ | ✅ |

### 2.2 Public, no auth (sitemap §1.1)

| Screen doc | Route | Component · `main.rs` | Route status | Screen spec |
|---|---|---|:---:|:---:|
| `Home.md` | `/` | `Home` · `:1282` | ✅ | ✅ |
| `AuthPage.md` | `/login` | `AuthPage` · `:1462` | ✅ | ✅ |
| `Verify.md` | `/verify/:code` | `Verify` · `:1297` | ⬜ — see §3 | ✅ |

### 2.3 Authenticated shell (sitemap §1.2)

| Screen doc | Route | Component · `main.rs` | Route status | Screen spec |
|---|---|---|:---:|:---:|
| `Dashboard.md` | `/dashboard` | `Dashboard` · `:1701` | 🟡 | ✅ |
| `Patients.md` | `/patients` | `Patients` · `:2758` | ✅ | ✅ |
| `PatientNew.md` | `/patients/new` | `PatientNew` · `:2954` | ✅ | ✅ |
| `PatientDetail.md` | `/patients/:id` | `PatientDetail` · `:3128` | 🟡 | ✅ |
| `PatientEdit.md` | `/patients/:id/edit` | `PatientEdit` · `:3358` | ✅ | ✅ |
| `Prescriptions.md` | `/prescriptions` | `Prescriptions` · `:4360` | 🟡 | ✅ |
| `PrescriptionNew.md` | `/prescriptions/new` | `PrescriptionNew` · `:4595` | ✅ | ✅ |
| `PrescriptionDetail.md` | `/prescriptions/:id` | `PrescriptionDetail` · `:5380` | 🟡 | ✅ |
| `PrescriptionPreview.md` | `/prescriptions/:id/preview` | `PrescriptionPreview` · `:4939` | ✅ | ✅ |
| `Templates.md` | `/templates` | `Templates` · `:3567` | ✅ | ✅ |
| `TemplateNew.md` | `/templates/new` | `TemplateNew` · `:3680` | ✅ | ✅ |
| `TemplateEdit.md` | `/templates/:id/edit` | `TemplateEdit` · `:3788` | ✅ | ✅ |
| `TemplatePreview.md` | `/templates/:id/preview` | `TemplatePreview` · `:4049` | ✅ | ✅ |
| `Settings.md` | `/settings` | `Settings` · `:1761` | ⬜ — see §3 | ✅ |
| `SettingsClinic.md` | `/settings/clinic` | `SettingsClinic` · `:1804` | ✅ | ✅ |
| `SettingsUsers.md` | `/settings/users` | `SettingsUsers` · `:2120` | ⬜ — see §3 | ✅ |
| `SettingsProfile.md` | `/settings/profile` | `SettingsProfile` · `:2319` | ⬜ — see §3 | ✅ |
| `SettingsBilling.md` | `/settings/billing` (v1.1) | not routed | ⬜ v1.1 | ⬜ |
| `Audit.md` | `/audit` | `Audit` · `:2634` | ⬜ — see §3 | ✅ |

## 3. Where `context/sitemap.md` lags the code

The §1.1/§1.2 status cells below read ⬜ while the component ships. The sitemap stays the authority, so the fix is the sitemap, not this index. Update the status cell, then drop the `— see §3` marker from §2.

| Route | Sitemap says | Code today |
|---|---|---|
| `/verify/:code` | ⬜ | `Verify` renders the four `es-EC` statuses (`Receta auténtica` / `anulada` / `reemplazada` / `no encontrada`), the typed-code field, and per-IP rate limiting — `specs/done/09-I-public-verification.md`, objectives 1–4 all checked |
| `/settings` | ⬜ | `Settings` (`main.rs:1761`) renders the admin hub (`Usuarios y roles`, `Clínica y biblioteca`, `Mi perfil`) and redirects non-admins to `/settings/profile`, signed-out to `/login` — the sitemap's own purpose row |
| `/settings/users` | ⬜ in §1.2, ✅ in §4 (M0) | `SettingsUsers` (`main.rs:2120`) lists staff, changes role, disables/removes, and revokes devices — spec 02-B shipped |
| `/settings/profile` | ⬜ | `SettingsProfile` (`main.rs:2319`) holds the profile fields (`license_no`, `national_id`, contact — spec 02-B), TOTP enrolment (`mfa_secret`/`mfa_uri`/code) and password change (spec 12-L) |
| `/audit` | ⬜ | `Audit` (`main.rs:2634`) filters by action/entity/query, is admin-guarded client-side and server-side, and renders the append-only rows — spec 12-L shipped |

## 4. Backlog — specs to write, in order

Ordered by the primary flows of `context/sitemap.md` §3, so each file lands before the code work that needs it. Copy the §1.2 shape, then check the box and update the matching §2 row.

**Wave 1 — issue path (flows 1–2, PRD target < 60 s)**

- [x] 1. `PrescriptionNew.md` — patient/template pickers from `?patient_id=&template_id=`, draft creation, one primary `Crear borrador`.
- [x] 2. `PrescriptionDetail.md` — the largest state matrix in the app: `draft` vs. `signed`, four roles, sign/void/supersede, per-medication blocks, the external LLM link (FR-012), `RxBlockIndicator`.
- [x] 3. `PrescriptionPreview.md` — true-scale A4 1:1, `Firmar y emitir`, PDF/print/share actions, provisional offline state with `PENDIENTE DE SINCRONIZACIÓN` and placeholder QR (FR-030–034, FR-043).
- [x] 4. `AuthPage.md` — sign in / sign up, `es-EC` copy from the UI plan; shipped MFA challenge documented, device registration silent.
- [x] 5. `Dashboard.md` — role-aware blocks, drafts, pending-sync widget (desktop), quick actions.
- [x] 6. `Patients.md` — search by name/cédula, clinic scoping, duplicate warnings.
- [x] 7. `PatientNew.md` — create form + deduplication notice.
- [x] 8. `PatientDetail.md` — demographics, role-filtered actions, deep link into `/prescriptions/new`.
- [x] 9. `PatientEdit.md` — same fields, sensitive-read audit note (§6).

**Wave 2 — history and retention (flow 5, M2 states)**

- [x] 10. `Prescriptions.md` — status filter (`draft → signed → superseded/voided`, FR-013), pending/failed sync rows, 30-day local-cache states and on-demand regeneration (desktop).

**Wave 3 — templates (flow 3)**

- [x] 11. `Templates.md` — own + clinic master list, status and current version, `Use for prescription` deep link.
- [x] 12. `TemplateNew.md` — create with owner/type hint.
- [x] 13. `TemplateEdit.md` — letterhead/body/footer forms, variable insertion, image slots with the v1 one-image quota prompt, `Publish` pinning `asset_id`s (FR-020–024, FR-080–085).
- [x] 14. `TemplatePreview.md` — A4 preview with sample values and pinned images.

**Wave 4 — public verification (flow 4)**

- [x] 15. `Verify.md` — QR target page, four statuses, typed-code form (FR-034), zero PHI, rate-limit and `not found` shapes, no shell chrome beyond `Header`/`Footer`.

**Wave 5 — admin and trust (M0/M3)**

- [x] 16. `Settings.md` — hub plus the role redirects; no content beyond the links.
- [x] 17. `SettingsClinic.md` — clinic data, logo, image library (1 × 10 MB in v1), retention dates, `llm_url_template`, export (FR-080–085).
- [x] 18. `SettingsUsers.md` — staff table, role changes, disable/remove, device revocation (§9).
- [x] 19. `SettingsProfile.md` — profile fields, TOTP MFA, password change, no signature/stamp asset (D-06).
- [x] 20. `Audit.md` — filters, actor/action/entity table, sensitive-read and verification logs, 5-year append-only framing (§8).

**Wave 6 — deferred, do not open yet**

- [ ] 21. `SettingsBilling.md` — v1.1 only (PRD D-04, D-14). Blocked: no billing table set is live and `/settings/billing` is unrouted. Write it only when the v1.1 billing spec opens; until then v1 surfaces must render no price, trial, or plan copy.

## 5. Surfaces with no spec of their own

Sitemap §1.3 non-routes fold into the screen that hosts them; no separate file, and none of them is a route.

| Surface | Folded into | Why |
|---|---|---|
| *Explain with LLM* per medication | `PrescriptionDetail.md` | Opens an external tab from `clinics.llm_url_template` with the medication name only; no PHI, no page |
| Download PDF · Print · Share | `PrescriptionPreview.md` (+ the row actions in `Prescriptions.md`) | `GET /api/prescriptions/:id/pdf` streams the `printpdf` render; the desktop writes it in-process. No PDF is stored |
| Image upload / delete | `SettingsClinic.md` and `TemplateEdit.md` | Quota, format, and MinIO rules are one set; both hosts restate nothing |
| Sync / outbox indicator | `Header.md` (+ the dashboard widget in `Dashboard.md`) | FR-043 lives in the shell; the pending count links to `/prescriptions?status=pending_sync` |
| PayPhone checkout + confirmation | `SettingsBilling.md` (v1.1) | External payment button; no card form in PersonaSanus |
| Renewal / retention / purge jobs | `context/runbook.md`, `context/architecture.md` | In-process scheduler, no surface |
| Auth API (`/api/auth/*`) | `context/architecture.md` | Endpoints, not Dioxus routes |
| `/dev/ui` design-system gallery | `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` step 9/18 | Debug-only, no role, not a product route; sitemap §1.3 gets the row, not a screen file |
| `/dev/db` SQLite browser | `persona-sanus/src/dev_db.rs` | Loopback-only axum tool in debug builds; never a Dioxus screen |

## 6. Maintenance

- A screen spec lands: fill the file per §1.2, check its §4 box, set its §2 row to ✅, update the `Count` and `Status` rows.
- A route is added, renamed, or dropped: fix `context/sitemap.md` first, then §2 of this file. The `Route` enum in `main.rs` is the tie-breaker for what exists.
- A §3 lag is cleared: update the sitemap status cell and remove the marker.
- Keep §1.1/§1.2/§5 as the only normative sections; everything else is state and goes stale on purpose.
- Verify table integrity after editing: `caveman-blocks run markdown-table-check --path context/screens/INDEX.md`.
