# Spec 19 — Phase S: UI gallery and developer route

- Source: `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` step 9; `context/Design.md` §§Components, Tokens, Surfaces, Elevation, Layout; `context/architecture.md` §3; `context/sitemap.md` §1.3, §5.
- Depends on: 15-O, 16-P, 17-Q, 18-R.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Result carried forward: debug builds expose `/dev/ui` as a static, navbar-free gallery with sections `tokens`, `layout`, `buttons`, `forms`, `badges`, `feedback`, `data`, `brand`, `marketing`, `status`, `verification`, `library`, and `billing`; release builds return one `.ui-card` with `Esta vista solo existe en compilaciones de desarrollo.` and contain no gallery or billing symbols; debug `Settings` links to the gallery.

## Before you start

Unit testing is not required; avoid it. Prioritize end-to-end and integration testing only.

Read the nearest `AGENTS.md` (target workspace, then parents up to repo root) before executing.

If reality diverges from this spec: STOP, append a divergence (section 4), and NEVER improvise outside the objectives.

This is the visual proof for `15-O` through `18-R`. It is a component index, not a product page. It renders fixed sample values only: no page-level context read, no API calls, no session, no router-state read, no patient data, no clinic data, and no token data.

## 1. Objectives

*The ordered work to implement — one item per step. Check `[x]` only after the step is implemented and verified.*

*Item format:* `<verb phrase>` — requirements: `<what must change / constraints>`; commands: `<exact command>`.

- [ ] Add the static design-system gallery — requirements: create `persona-sanus/src/ui/gallery.rs` with `DevUiGallery` and render a `PageShell id="dev-ui-page"` wrapping `div.ui-gallery`. Open with `LogoLockup`, `h1` `Sistema de diseño`, the line `Componentes reutilizables de PersonaSanus. Solo desarrollo.`, and a Secondary `ButtonLink` `Volver al inicio` to `Route::Home {}`. Add exactly these section roots: `section id="gallery-tokens"`, `section id="gallery-layout"`, `section id="gallery-buttons"`, `section id="gallery-forms"`, `section id="gallery-badges"`, `section id="gallery-feedback"`, `section id="gallery-data"`, `section id="gallery-brand"`, `section id="gallery-marketing"`, `section id="gallery-status"`, `section id="gallery-verification"`, `section id="gallery-library"`, and `section id="gallery-billing"`. Every section has an `h2` and static samples. Include every component and state required by section 7, including every enum variant, all five verification results, all billing states, the `None`, inactive, active, and exhausted RX block states, disabled and loading buttons, wide and error fields, a table, an empty state, the trust strip, and an icon strip covering every `IconName`. Under the trust strip render `Logos de ejemplo; no son clientes reales.`. Use only the constant fictional samples in section 7. No page-level context, signal, API call, session, navigator, or router-state read, and no external image; built-in `Link`/`NavLink` behavior is allowed. The gallery is exempt from the one-primary-per-viewport rule; commands: `cd persona-sanus && cargo check`; `rg -n 'use_context|use_signal|use_navigator|use_resource|spawn|navigator|api::fetch|api::upload|api::delete' persona-sanus/src/ui/gallery.rs`; `rg -n 'gallery-(tokens|layout|buttons|forms|badges|feedback|data|brand|marketing|status|verification|library|billing)' persona-sanus/src/ui/gallery.rs`.
- [ ] Add the debug route and its compile-time gates — requirements: add `#[route("/dev/ui")] DevUi {}` to `Route` beside `Verify` and immediately before the `#[layout(Navbar)]` line that opens the authenticated layout (current `main.rs:49-51`), so `/dev/ui` renders with no navbar like `/verify/:code`. `DevUi` returns `ui::DevUiGallery` in a debug build. The non-debug branch returns one `div.ui-card` with the exact text `Esta vista solo existe en compilaciones de desarrollo.` and no gallery markup. In `ui/mod.rs`, declare `gallery` and `billing` only under `#[cfg(debug_assertions)]` and re-export `DevUiGallery` under the same gate. Add a debug-only `Sistema de diseño` link in `Settings` to `Route::DevUi`; the release settings surface must not contain the link or the string. Do not add the gallery to the navbar; commands: `cd persona-sanus && cargo check`; `cd persona-sanus && cargo check --release`; `rg -n -F '#[cfg(debug_assertions)]' persona-sanus/src/ui/mod.rs persona-sanus/src/main.rs`.

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No new crate, CSS framework, external stylesheet, webfont, CDN asset, icon package, or JavaScript bridge.
- No rich text, WYSIWYG, `contenteditable`, formatting toolbar, `execCommand`, HTML payload, or editor dependency. All text entry is plain controlled inputs, textareas, selects, and checks.
- No new product route, API handler, server function, Diesel model, table, migration, entitlement rule, or billing persistence. `/dev/ui` is the only route addition; it contains no API calls.
- No billing route or billing surface in release. `/settings/billing` stays reserved with no route, and the gallery cannot navigate to it.
- No card form, PAN, CVV, full card number, `ctoken`, `card_token`, `card_holder_enc`, or `authorization_code` in gallery copy or rendered markup. Billing shows status and consent only.
- No change to authorization, session, device registration, sync, numbering, signing, PDF, QR, verification limits, or business validation.
- No real patient name, cédula, diagnosis, medication, or photo in `/dev/ui`, Home, the base, the navbar, or public verification. Gallery samples are fictional and static.
- No fake partner logos or endorsement claim. `TrustLogoStrip` is gallery-only and carries the sample note.
- No pure black, drop shadow on UI cards or buttons, chromatic card or image border, bold grenette, or more than one filled primary action in a viewport. The gallery is the sole exemption for the primary-action count because it is a component index.
- No release-visible gallery link and no unconditional `gallery` or `billing` module declaration.
- No unit tests and no comments in Rust, CSS, or RSX.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [ ] objective 1: `cd persona-sanus && cargo check` exits 0.
- [ ] objective 1: `rg -n 'use_context|use_signal|use_navigator|use_resource|spawn|navigator|api::fetch|api::upload|api::delete' persona-sanus/src/ui/gallery.rs` returns no matches.
- [ ] objective 1: `rg -n 'gallery-(tokens|layout|buttons|forms|badges|feedback|data|brand|marketing|status|verification|library|billing)' persona-sanus/src/ui/gallery.rs` returns 13 matches.
- [ ] objective 2: `cd persona-sanus && cargo check --release` exits 0.
- [ ] objective 2: `rg -n -F '#[cfg(debug_assertions)]' persona-sanus/src/ui/mod.rs persona-sanus/src/main.rs` returns the gates around `billing`, `gallery`, `DevUiGallery`, the settings link, and the `DevUi` debug branch.
- [ ] objective 2: `rg -n -A2 -F '#[route("/dev/ui")]' persona-sanus/src/main.rs` shows `DevUi {}` and `#[layout(Navbar)]` after the route.
- [ ] debug gallery proof: `rg -n 'gallery-(tokens|layout|buttons|forms|badges|feedback|data|brand|marketing|status|verification|library|billing)' persona-sanus/src/ui/gallery.rs` returns every section.
- [ ] release branch source proof: `rg -n -A6 'fn DevUi' persona-sanus/src/main.rs` shows a `#[cfg(debug_assertions)]` branch returning `DevUiGallery` and a `#[cfg(not(debug_assertions))]` branch returning only the `.ui-card` development message. The full HTTP release absence check runs in `25-Y`.
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

`DevUiGallery` is the contract proof for every shared symbol. It replaces ad hoc visual checking with one debug-only page that renders all component states against fixed sample values. It is not a product route and must not survive into a release bundle.

The route lives beside `Verify` and before the authenticated layout:

```rust
#[route("/verify/:code")]
Verify { code: String },
#[route("/dev/ui")]
DevUi {},
#[layout(Navbar)]
```

The root is `PageShell id="dev-ui-page"` wrapping `div.ui-gallery`. `PageShell` provides `.ui-page` and `.ui-container.ui-stack`; the gallery adds `display: flex; flex-direction: column; gap: 48px`. The gallery head is custom (not `PageHeader`) so it can open with the logo lockup.

The section list is fixed. `gallery-tokens` includes the token names and values, focus states, and the complete icon strip. `gallery-layout` demonstrates every layout primitive. `gallery-buttons` demonstrates all button variants and navigation actions. `gallery-forms` demonstrates all six controls, including wide, disabled, read-only, hint, error, and rows variants. `gallery-badges` demonstrates every badge tone and the pill. `gallery-feedback` demonstrates every notice tone, error and hint, empty state, loading state, and confirm panel. `gallery-data` demonstrates a captioned table and a list row with badge plus actions. `gallery-brand` demonstrates the logo lockup, trust row, and the fictitious trust strip. `gallery-marketing` demonstrates the hero, section header, three numbered steps, all A4 accents, all A4 mocks, and the verification tag. `gallery-status` demonstrates all sync, prescription, template, account, device, and RX block states. `gallery-verification` demonstrates all seal and result-card states and both code-form variants. `gallery-library` demonstrates quota, pending, ready, placeholder, empty, uploading, deleting, offline, and full states. `gallery-billing` demonstrates every billing status, plan, payment method, banner, and transaction state.

## 6. Reuse, do not rebuild

| Existing symbol | Decision |
|---|---|
| `forms::A4Preview` | Keep as the real renderer. The gallery `marketing` section uses `A4Mock` only. |
| `forms::TemplateDocumentForm` | Keep. Do not mount it in the gallery; `22-V` proves its states in the real editor. |
| `Navbar` | Keep. `/dev/ui` is outside the navbar layout. Do not add a gallery nav item. |
| `Route`, `App`, page components | Keep. Add `DevUi` and the debug settings link only. |
| `forms::paginate_document`, `resolve_text`, document warnings | Keep unchanged. |
| `SyncStatus`, `probe_sync_status`, local count helpers | The gallery imports the `18-R` domain types. It never calls `probe_sync_status` or mounts the sync context. |
| `format_date`, `format_timestamp`, label helpers | Consume the `15-O` exports. Do not copy their output into constants. |
| `.a4-*` CSS and A4 geometry | Keep unchanged. The gallery does not render the real A4 projection. |

There is one styling system. Extend `assets/main.css` and the new `src/ui` modules only. Do not add a CSS framework, another stylesheet, or an image/icon package.

## 7. Component surface

`DevUiGallery` has no props and no contexts. All callbacks are no-ops. All samples are constants in this spec or in the owning component. Every section renders a visible `h2`; use Spanish labels for the section headings.

### Fixed section roots

`gallery-tokens`, `gallery-layout`, `gallery-buttons`, `gallery-forms`, `gallery-badges`, `gallery-feedback`, `gallery-data`, `gallery-brand`, `gallery-marketing`, `gallery-status`, `gallery-verification`, `gallery-library`, `gallery-billing`.

### Sections and required samples

- `tokens`: color swatches, type scale, spacing, radii, surfaces, elevation note, the shared focus ring example, and the icon strip. The icon strip renders 13 `Icon` rows, one per `IconName` variant: `Cross`, `Shield`, `Command`, `Windows`, `Globe`, `Check`, `Close`, `Alert`, `Info`, `Qr`, `ArrowRight`, `Lock`, `Image`; each row shows the icon and the variant name in `.ui-mono`. A caption states the only shadow is the focus halo or `--shadow-paper`.
- `layout`: `PageShell` is the gallery root itself; show `PageHeader` (title plus subtitle plus an action), `Card`, `SectionCard` with and without a description, `FormGrid` with `Two` and `One`, `DetailGrid` with normal and wide `DetailItem`, `ActionRow`, `FilterBar`, and all four `SectionBand` tones `Paper`, `Cream`, `Mint`, `Blossom`.
- `buttons`: `Button` `Primary`, `Secondary`, `Ghost`, `Danger`; disabled Primary; loading Secondary with the label `Guardando…`; `ButtonLink` Primary/Secondary to `Route::Home {}`; `LinkAction` Secondary with `href="#gallery-brand"` and one with `target="_blank"` plus `rel="noopener noreferrer"`; `NavLink` for `Route::Home {}` and `Route::SettingsProfile {}`.
- `forms`: `TextField` empty and filled, email type, number with min/max/step, date with `input_type="date"`, required; one `wide`; one with a hint; one with an error; one disabled; one read-only; one hidden label; `TextAreaField` with rows and wide; `SelectField` with three options, one hint variant and one error variant; `CheckboxField` with description; `FileField` with accept `image/jpeg,image/png`; `SearchField` with placeholder.
- `badges`: `Badge` tone `Neutral`, `Mint`, `Cream`, `Blossom`, `Plum` with labels `Neutral`, `Mint`, `Cream`, `Blossom`, `Plum`; `PillBadge` default.
- `feedback`: `Notice` tone `Info`, `Success`, `Warning`, `Error` with fixed sample titles and body; `ErrorText` `No se pudo guardar el cambio.`; `HintText` `Máximo 10 MB.`; `EmptyState` `Sin registros` / `Todavía no hay registros para mostrar.` and one with the default empty description; `LoadingState` default and one with `Verificando receta…`; `ConfirmPanel` sample message `Esta acción no se puede deshacer.` with confirm `Confirmar`, cancel `Cancelar`, and a sample child.
- `data`: `DataTable` with caption `Ejemplo de tabla` and columns `Nombre`, `Correo`, `Estado`, three plain rows; `ListRow` with title `María Fernanda Ruiz`, meta `vec!["Cédula 0000000000".to_owned(), "Nacimiento 01/01/1990".to_owned()]`, a Neutral badge `Borrador`, and a Secondary `Abrir` child.
- `brand`: `LogoLockup`; `TrustSecurityRow`; `TrustLogoStrip` with caption `Con la confianza de…` and five text logos `Clínica Demo Norte`, `Consultorio Demo Sur`, `Centro Demo Andes`, `Clínica Demo Costa`, `Salud Demo`; under the strip render the note `Logos de ejemplo; no son clientes reales.`
- `marketing`: a compact `Hero` titled `Hero de ejemplo` with subtitle `Título y subtítulo de demostración.`; `SectionHeader` with and without subtitle; three `NumberedStepCard`s with numbers 1–3 and titles `Elige el paciente`, `Completa la receta`, `Firma y verifica`, accents `Mint`, `Cream`, `Blossom`, mocks `Patient`, `Medication`, `Verification`, and tag `Verificable` on the third; `A4PreviewCard` for each accent with an `A4Mock` matching each kind; one `VerificationTag` `Verificable` standalone.
- `status`: `SyncStatusIndicator` for `online`, `offline`, `pending` (1 and 3 pending), and `failed`; `PrescriptionStatusBadge` for `draft`, `signed`, `superseded`, `voided`, `pending_sync`, `failed`, and `unknown`; `TemplateStatusBadge` for `draft`, `published`, `archived`, and `unknown`; `AccountStatusBadge` for `active`, `disabled`, and `unknown`; `DeviceStatusBadge` for both `false` and `true`; an empty `div` proving `RxBlockIndicator` renders nothing for `None`; `RxBlockIndicator` inactive, active with remaining numbers, active exhausted, and allocating disabled, all with no-op `on_allocate`.
- `verification`: `VerificationSeal` for `valid`, `voided`, `superseded`, `not found`, and `unknown`; `VerificationResultCard` for valid, voided, superseded, `not found`, and unknown, using fixed fictional doctor `Dra. Ejemplo Uno`, date `2026-01-15`, and number `7` (details only where contracted); `VerificationCodeForm` with default Primary, and another with Secondary; both use a no-op `on_submit`.
- `library`: `QuotaMeter` used `1` of `5`, used `5` of `5`, and used `0` of `0`; `ImageLibrary` empty; pending asset named `Pendiente de ejemplo`; ready asset with a 1x1 transparent PNG `data:` URI (no network fetch); ready asset with no URL; uploading; deleting `asset-1`; offline; full at quota. Fictional image rows use ids `asset-1` and `asset-2`, width `1200`, height `1600`, size `524288`, and names `Imagen de ejemplo`.
- `billing`: `format_usd_cents` examples `$5,00 USD`, `$50,00 USD`, `$43,48 USD`, `$1.234,56 USD`, and `-$5,00 USD`; `BillingStatusCard` for `trialing`, `active`, `past_due`, `locked`, `canceled`, and `unknown`, one with `cancel_at_period_end: true`; `PlanCard` monthly unselected and yearly selected with `2 meses gratis`; `PaymentMethodCard` active and revoked; `BillingBanner` past_due with and without grace date, locked, canceled, and active; `TransactionTable` with one row per status `pending`, `approved`, `canceled`, `failed`, `reversed`, and an empty table. All dates are fixed fictional strings. No token, card number, PAN, CVV, or authorization code value is rendered or stored in a prop.

## 8. Page adoption

Only two page edits are allowed:

- `Settings` gains a debug-only Secondary `Sistema de diseño` link to `Route::DevUi`. It must be removed from a release build by compile-time gating (not merely hidden with CSS). Later, `21-U` keeps this link inside the settings link grid.
- `DevUi` is added to `Route` beside `Verify` and before the line `#[layout(Navbar)]`. It renders `DevUiGallery` in debug builds and the single `.ui-card` guard message in release builds. It is not added to the navbar or any other release-visible link list.

## 9. CSS contract

| Selector | Values |
|---|---|
| `.ui-gallery` | display flex column gap 48; PageShell owns width, margin, and inline padding |
| `.ui-gallery-head` | text-align center; h1 mint 500 26px/1.22 Ink; p 14px Mist |
| `.ui-gallery-section` | display flex column gap 16; its h2 is mint 500 20px/1.25 Ink |
| `.ui-gallery-grid` | grid auto-fit minmax(220px, 1fr), gap 16, align start |
| `.ui-gallery-frame` | Paper, 1px hairline, 24px radius, 20px padding |
| `.ui-gallery-label` | mint 500 12px/1.5 Mist uppercase, letter-spacing .08em |

The gallery may render several `.ui-button--primary` elements because it is a component index. It must not add a second filled primary action to any product page. It uses no pure black and no drop shadow.

## 10. Critical files and anchors

| File | Anchor |
|---|---|
| `persona-sanus/src/main.rs` | `enum Route` 48-93; `Verify` route variant 49-50; `#[layout(Navbar)]` 51; `App` 1222; `Home` 1281; `Navbar` 1376; `AuthPage` 1461; `Settings` 1761 |
| `persona-sanus/src/ui/mod.rs` | Module list from `15-O`–`18-R`; add `#[cfg(debug_assertions)] mod gallery;` and `mod billing;`; gate `pub use gallery::DevUiGallery;` |
| `persona-sanus/src/ui/gallery.rs` | New; static `DevUiGallery` and all 13 sections |
| `persona-sanus/assets/main.css` | `:root` 1-58; append gallery declarations after `18-R` blocks |
| `context/sitemap.md` | §1.1-1.3 route tables; §5 conventions |
| `context/architecture.md` | §3 client modules |
| `context/Design.md` | §§Components, Tokens, Surfaces, Elevation, Layout |

## 11. Assumptions and contingencies

- Scope is fixed: full v1 adoption plus presentational v1.1 billing. Do not add a billing route, a plan picker that calls an endpoint, a card form, or a `sync_locked` gate. This gallery is the only billing surface.
- The gallery is a debug-only developer surface. It is not product navigation, not a release feature, and not linked from the navbar.
- The repo has no grenette or mint webfonts. Keep the existing fallback stacks; do not fetch or add fonts.
- No partner-clinic logos exist. The five names in the gallery are fictitious demo wordmarks; the note must stay. Home must not show `TrustLogoStrip`.
- Dioxus 0.7 fallbacks: if `NavigationTarget` as a component prop does not accept `Route` directly, `ButtonLink.to` and `NavLink.to` use `Route`; if `Option<Element>` props break the props macro, make `badge` or children required and pass `rsx! {}` at call sites; if `FormEvent` cannot be a component callback, keep the file `<input>` in `SettingsClinic` and share only `Field` and `.ui-file-input` styling. None of these changes the visual contract.
- If the debug server path is not `target/dx/persona-sanus/debug/web/server`, use `dx serve --fullstack --port 8095`. If port 8095, 8096, or 8097 is busy, change only the port and the matching URL.
- If a sample value would require an API, context, or real clinic record, replace it with a fixed fictional constant in `gallery.rs`; never wire the gallery to live data.
- If browser verification finds a control not in the Design.md list but required by the existing page, use the nearest existing primitive (`Badge`, `Card`, `Button Secondary`) and do not add a new component file. The component set is closed after this spec.

---

*Housekeeping: once a spec is fully done with no open divergences, move its
file to `specs/done/` (same file name, e.g.
`specs/done/19-S-ui-gallery-and-dev-route.md`).
Specs with open divergences stay in `specs/` next to `TEMPLATE.md` until the
tracking file is cleared.*
