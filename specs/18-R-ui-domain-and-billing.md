# Spec 18 — Phase R: UI domain status and billing

- Source: `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` steps 7–8; `context/Design.md` §§Components, Tokens, Surfaces, Elevation, Layout; `context/database-design.md` §4; `context/PRD.md` FR-043, FR-050, FR-080–085, D-06, D-21; `context/architecture.md` §3.
- Depends on: 15-O, 16-P.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Result carried forward: `persona-sanus/src/ui/domain.rs` exposes `SyncStatus`, `SyncStatusIndicator`, the prescription, template, account, and device status badges, `RxBlockIndicator`, `LlmExplainLink`, `VerificationSeal`, `VerificationResultCard`, `VerificationCodeForm`, `QuotaMeter`, `ImageLibrary`, and `format_bytes`; `billing.rs` exposes the presentational v1.1 billing set and `format_usd_cents`; both compile for web and desktop, expose no card or patient data, and are consumed by the gallery and page adoption specs.

## Before you start

Unit testing is not required; avoid it. Prioritize end-to-end and integration testing only.

Read the nearest `AGENTS.md` (target workspace, then parents up to repo root) before executing.

If reality diverges from this spec: STOP, append a divergence (section 4), and NEVER improvise outside the objectives.

The billing module is a gallery-only, presentational surface. It has no route, fetch, persistence, or release-visible entry. Status and image-library components are reused by release pages through later specs.

Two source ambiguities are resolved here and must be applied consistently:

- The repository has no byte-size helper. `format_bytes` is a new private helper in `domain.rs` with the exact contract in section 7; do not claim an existing one.
- `QuotaMeter` uses a 4px track per the CSS contract. The earlier `1px` phrase is read as the hairline stroke weight, not the track height.

## 1. Objectives

*The ordered work to implement — one item per step. Check `[x]` only after the step is implemented and verified.*

*Item format:* `<verb phrase>` — requirements: `<what must change / constraints>`; commands: `<exact command>`.

- [ ] Add the domain status, sync, RX, LLM, verification, and image-library components — requirements: create `persona-sanus/src/ui/domain.rs` and implement `SyncStatus`, `SyncStatusIndicator`, `PrescriptionStatusBadge`, `TemplateStatusBadge`, `AccountStatusBadge`, `DeviceStatusBadge`, `RxBlockIndicator`, `LlmExplainLink`, `VerificationSeal`, `VerificationResultCard`, `VerificationCodeForm`, `QuotaMeter`, `ImageLibrary`, and private `format_bytes` exactly as section 7. Move the existing `SyncStatus` out of `main.rs`, make its fields public, keep `Debug, Clone, PartialEq`, and add `Copy, Default`; replace the free `sync_status_label` with `SyncStatus::label` and `SyncStatus::tone`, and remove both old definitions in `main.rs`; also replace the single `sync_status_label(&status)` call in `Navbar` with `status.label()`. Import the moved type where `main.rs` provides the sync context. `VerificationResultCard` reads only `status`, `doctor_name`, `issued_at`, and `rx_number`; it must never accept or render a patient field. Re-export the module from `ui/mod.rs`; commands: `cd persona-sanus && cargo check`; `rg -n '^struct SyncStatus|^fn sync_status_label' persona-sanus/src/main.rs`; `rg -n 'patient_name|patient_id|patient_birth|patient_sex|cedula|cédula|diagnosis|medication' persona-sanus/src/ui/domain.rs`.
- [ ] Add the presentational billing components — requirements: create `persona-sanus/src/ui/billing.rs` and implement `BillingStatusCard`, `PlanCard`, `PaymentMethodCard`, `BillingBanner`, `BillingTransaction`, `TransactionTable`, and `format_usd_cents` exactly as section 7. Use integer cents only and integer division and remainder for currency. Never render `card_token`, `card_holder_enc`, `ctoken`, `authorization_code`, a PAN, a CVV, or a card number. Declare `billing` under `#[cfg(debug_assertions)]` in `ui/mod.rs`; it has no release surface; commands: `cd persona-sanus && cargo check`; `rg -n -i 'card_token|card_holder|ctoken|authorization_code|card_number|\bcvv_number\b|\bpan_number\b' persona-sanus/src/ui`; `rg -n 'f32|f64' persona-sanus/src/ui/billing.rs`.

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No new crate, CSS framework, external stylesheet, webfont, CDN asset, icon package, or JavaScript bridge.
- No rich text, WYSIWYG, `contenteditable`, formatting toolbar, `execCommand`, HTML payload, or editor dependency. All text entry is plain controlled inputs, textareas, selects, and checks.
- No new product route, API handler, server function, Diesel model, table, migration, entitlement rule, or billing persistence. The next new route is `/dev/ui` in `19-S`.
- No card form, PAN, CVV, full card number, `ctoken`, `card_token`, `card_holder_enc`, or `authorization_code` in UI props, gallery copy, or rendered markup. Billing shows status and consent only. The token is never a prop and never rendered.
- No change to authorization, session, device registration, sync, numbering, signing, PDF, QR, verification limits, or business validation.
- No page adoption in this spec. The only page edit permitted is the `SyncStatus` import required because the type moved out of `main.rs`, plus the single `Navbar` call-site replacement of `sync_status_label(&status)` with `status.label()`. Do not edit pages otherwise.
- No patient name, cédula, diagnosis, medication, or photo in the verification card. The public card receives only `api::VerificationResult` and renders only doctor, date, and number.
- No pure black, no drop shadow, no chromatic card or image border, no bold grenette, and no more than one filled primary action in a viewport.
- No second stylesheet and no CSS framework; extend `assets/main.css`.
- No unit tests and no comments in Rust, CSS, or RSX.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [ ] objective 1: `cd persona-sanus && cargo check` exits 0.
- [ ] objective 1: `cd persona-sanus && cargo check --no-default-features --features desktop` exits 0.
- [ ] objective 1: `rg -n '^struct SyncStatus|^fn sync_status_label' persona-sanus/src/main.rs` returns no matches.
- [ ] objective 1: `rg -n 'pub struct SyncStatus|fn label|fn tone' persona-sanus/src/ui/domain.rs` returns the public struct and both methods.
- [ ] objective 1: `rg -n 'patient_name|patient_id|patient_birth|patient_sex|cedula|cédula|diagnosis|medication' persona-sanus/src/ui/domain.rs` returns no matches.
- [ ] objective 1: `rg -n 'fn (VerificationSeal|VerificationResultCard|VerificationCodeForm|QuotaMeter|ImageLibrary|RxBlockIndicator|LlmExplainLink|SyncStatusIndicator)' persona-sanus/src/ui/domain.rs` returns 8 matches.
- [ ] objective 2: `rg -n 'fn (BillingStatusCard|PlanCard|PaymentMethodCard|BillingBanner|TransactionTable|format_usd_cents)' persona-sanus/src/ui/billing.rs` returns 6 matches.
- [ ] objective 2: `rg -n 'struct BillingTransaction' persona-sanus/src/ui/billing.rs` returns 1 match.
- [ ] objective 2: `rg -n 'f32|f64' persona-sanus/src/ui/billing.rs` returns no matches.
- [ ] objective 2: `rg -n -i 'card_token|card_holder|ctoken|authorization_code|card_number|\bcvv_number\b|\bpan_number\b' persona-sanus/src/ui` returns no matches.
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

Status vocabulary must match the database and API exactly, not pretty-print in one page and differ in another. The prescription statuses are `draft`, `signed`, `superseded`, `voided`, `pending_sync`, `failed`; template statuses are `draft`, `published`, `archived`; account statuses are `active` and `disabled`; device state is the `revoked` boolean. `SyncStatus` is the desktop and web sync snapshot with `online`, `pending`, and `failed`.

`VerificationResult` at `api.rs:973` exposes exactly `status`, `doctor_name`, `issued_at`, and `rx_number`. FR-050 forbids patient data on public verification. This spec makes the card structurally incapable of reading patient data: its props are the API result, and only those four fields may be referenced.

Billing is deferred to v1.1. This spec builds only presentational primitives for the future `/settings/billing` screen. No billing route, plan picker, payment call, checkout, card form, or entitlement gate lands. The module is debug-only and the only consumer in this change is `DevUiGallery`.

The current image library in `SettingsClinic` renders a raw `asset-row` list with byte counts, renders both logo and image assets, and does not expose `QuotaMeter` or the contracted hints. `ImageLibrary` is the replacement, adopted by `24-X`. The source label `Subir imagen de la biblioteca (JPEG o PNG, máximo 10 MB)` is preserved exactly.

## 6. Reuse, do not rebuild

| Existing symbol | Decision |
|---|---|
| `forms::A4Preview` | Keep as the real document projection. This spec does not render or replace A4 documents. |
| `forms::TemplateDocumentForm` | Keep. `22-V` adopts the new controls later. |
| `Navbar` | Keep. It consumes `SyncStatusIndicator` in `20-T`; the context/sign-out/router effect stays. |
| `Route`, `App`, page components | Keep. This spec adds no route and no page. |
| `forms::paginate_document`, `resolve_text`, document warnings | Keep unchanged. |
| `SyncStatus`, `probe_sync_status`, local count helpers | Move `SyncStatus` here as a public type with `Debug, Clone, Copy, PartialEq, Eq, Default`. `probe_sync_status` and the `#[cfg(feature = "desktop")]` counts stay in `main.rs`. |
| `format_date`, `format_timestamp`, label helpers | Consume the `15-O` exports; do not copy them. |
| `api::AssetRow`, `api::AssetQuota`, `api::VerificationResult`, `api::RxBlockStatus` | Reuse these shapes unchanged. Do not add fields for serialization or UI-only data. |
| `.a4-*` CSS and A4 geometry | Keep unchanged. The verification and library classes must not match `.a4-*`. |
| Old `.notice`, `.badge`, `.hint`, `.error`, `.asset-row`, `.block-indicator` CSS | Keep for now; `25-Y` deletes after every page adopts the new primitives. |

## 7. Component surface

All symbols live in `persona-sanus/src/ui/domain.rs` or `billing.rs` and are re-exported from `src/ui/mod.rs`, with `billing` under `#[cfg(debug_assertions)]`. Every component renders no drop shadow. All copy below is fixed; do not substitute synonyms. Fields marked `#[props(default)]` are optional; unmarked fields are required.

### Domain: sync and status

- `SyncStatus`: public fields `online: bool`, `pending: i64`, `failed: i64`. Derives `Debug, Clone, Copy, PartialEq, Eq, Default`. `label()` returns, in this order: `"Sin conexión"` when offline, `"Error de sincronización"` when `failed > 0`, `"Sincronizando"` when `pending > 0`, else `"En línea"`. `tone()` returns the matching slug in `offline | failed | pending | online`; failed wins over pending.
- `SyncStatusIndicator`: `status: SyncStatus`. Renders `div.ui-sync.ui-sync--{tone}` with `span.ui-sync-dot` and `status.label()` at 14px Mist. Dot precedence equals label precedence: Mist offline, Blossom failed, Cream pending, Mint online. `role="status"` and `aria-live="polite"`. When `pending > 0`, appends `· {n} pendientes`, using `1 pendiente` for one. No link and no router dependency.
- `PrescriptionStatusBadge`: `status: String`. `draft` Neutral `Borrador`; `signed` Mint `Firmada`; `superseded` Cream `Reemplazada`; `voided` Blossom `Anulada`; `pending_sync` Cream `Provisional`; `failed` Blossom `Error de sincronización`; any other Neutral `Desconocido`.
- `TemplateStatusBadge`: `status: String`. `draft` Neutral `Borrador`; `published` Mint `Publicada`; `archived` Cream `Archivada`; any other Neutral `Desconocido`.
- `AccountStatusBadge`: `status: String`. `active` Mint `Activo`; `disabled` Neutral `Deshabilitado`; any other Neutral `Desconocido`.
- `DeviceStatusBadge`: `revoked: bool`. `false` Mint `Activo`; `true` Blossom `Revocado`.
- `RxBlockIndicator`: `status: Option<api::RxBlockStatus>`, `#[props(default)] allocating: bool`, `on_allocate: EventHandler<MouseEvent>`. Renders nothing when `status` is `None`. Otherwise renders a cream hairline strip `div.ui-rx-block`. Inactive (`!active`) shows `Sin bloque de números activo.`. Active shows `Números disponibles: {remaining} de {limit}`. Active plus `exhausted` also shows `Bloque agotado. Reserva otro antes de firmar.`. A Secondary `Button` labelled `Reservar bloque` renders when inactive or exhausted, calls `on_allocate`, and is disabled while `allocating`. Status labels use `match`; no allocation for a label except the pending count string.
- `LlmExplainLink`: `href: String`. Renders `div.ui-llm` with `a` `href`, `target="_blank"`, `rel="noopener noreferrer"`, text `Explicar con LLM`, and `p.ui-hint` text `Se abre un sitio externo con el nombre del medicamento. No se envían datos del paciente.`. No router.

### Domain: verification and library

- `VerificationSeal`: `status: String`. Renders `div.ui-verify-seal.ui-verify-seal--{slug}` with `aria-hidden="true"`. Slugs are `valid`, `voided`, `superseded`, `not-found`, or `unknown`; any value other than `valid`, `voided`, `superseded`, or `not found` is `unknown`. Valid is a Mint circle with a Deep Teal `Check` inside `div.ui-verify-ring`, an 80px circle with 4px padding and a `--gradient-ember-ring` background. Voided is Blossom with a Plum `Close`. Superseded is Cream with an Ink `ArrowRight`. Not found is Paper/Mist with a hairline border and the `Qr` icon. Unknown is Paper/Mist with a hairline border and the `Info` icon.
- `VerificationResultCard`: `verification: api::VerificationResult`. Renders `div.ui-verify-card.ui-verify-card--{slug}` with eyebrow `VERIFICACIÓN PÚBLICA`, `VerificationSeal`, `h1.ui-verify-title`, `p.ui-verify-copy`, and for valid, voided, or superseded only, `dl.ui-verify-details`. Copy: valid `Receta auténtica` / `Esta receta fue emitida y firmada por un profesional autorizado en PersonaSanus.`; voided `Receta anulada` / `El médico emisor anuló esta receta. El documento ya no es válido.`; superseded `Receta reemplazada` / `Esta receta fue reemplazada por una corrección posterior. Consulta la versión vigente.`; `not found` `Receta no encontrada` / `El código no corresponde a una receta emitida. Revisa el código impreso junto al QR.`; other `Receta no verificable` / `No pudimos determinar el estado de este documento.`. Tones: Mint, Blossom, Cream, Neutral, Neutral. Details only for valid, voided, and superseded: `Médico`, `Fecha de emisión`, and `No. de receta` (`{n:06}`), with `—` for a missing value. Never reads or renders a patient field.
- `VerificationCodeForm`: `button_variant: ButtonVariant = Primary`, `on_submit: EventHandler<String>`. Owns a local value signal. Renders `form.ui-code-form` with `onsubmit` that calls `prevent_default()`, trims and uppercases the value, renders `ErrorText` `Ingresa el código de verificación.` and calls nothing when empty, otherwise clears the local error and invokes `on_submit` once. Contains `TextField` labelled `Código de verificación`, placeholder `PS-7K2F-9Q`, `autocomplete="off"`, `spellcheck=false`, and a submit `Button` with the passed variant labelled `Verificar`. The Button is `button_type="submit"`. Home passes `Secondary`; Verify uses the default.
- `QuotaMeter`: `used: i64`, `limit: i64`. Renders `div.ui-quota` with text `Uso de la biblioteca: {used} de {limit}.`, then a track and a Deep Teal fill of `used / limit` clamped to 0..1. `role="progressbar"`, `aria-valuemin="0"`, `aria-valuemax="{limit}"`, `aria-valuenow="{used}"`. No colour change at quota. When `limit <= 0`, fill is 0 and `aria-valuemax` is 1. Track is 4px with the hairline background.
- `ImageLibrary`: `assets: Vec<api::AssetRow>`, `used: i64`, `limit: i64`, `uploading: bool`, `deleting: Option<String>`, `online: bool`, `on_delete: EventHandler<String>`, `on_upload: EventHandler<FormEvent>`. Renders `QuotaMeter`, then only assets with `kind == "image"` (the logo is ignored). When no image asset exists, renders `EmptyState` `Sin imágenes` / `Sube una imagen JPEG o PNG de hasta 10 MB.`. A pending asset (`status == "pending"`) shows a Cream `Badge` `Procesando` and no thumbnail. A ready asset shows a 48px thumbnail when `url` is `Some`, else a hairline 48px placeholder with the `Image` icon; the row shows `original_name` or `Imagen {first 8 id chars}`, `{width} × {height}` with `—` for a missing dimension, and `format_bytes(size_bytes)`. Delete is a Danger `Button` labelled `Eliminar`, disabled when `deleting.is_some()` or `!online`; the online-only upload hint remains visible. Upload is `FileField` label `Subir imagen de la biblioteca (JPEG o PNG, máximo 10 MB)`, accept `image/jpeg,image/png`, wide, disabled when `!online || uploading || used >= limit`, with `on_change` forwarding the whole `FormEvent` to `on_upload`. Hint: `Subir imágenes requiere conexión.` when offline, `La biblioteca está llena.` when at quota, else `JPEG o PNG, máximo 10 MB.`. The `SettingsClinic` caller selects the first file only with `event.files().first()`.
- `format_bytes`: private. `1024`-based. Bytes print an integer plus `B`; `KB` and `MB` print one decimal. Examples: `0 -> "0 B"`, `1023 -> "1023 B"`, `1024 -> "1.0 KB"`, `10485760 -> "10.0 MB"`.

### Billing

- `BillingStatusCard`: `status: String`, `trial_ends_at: Option<String>`, `next_charge_at: Option<String>`, `grace_ends_at: Option<String>`, `#[props(default)] cancel_at_period_end: bool`, `children`. Renders a Paper card with `h2.ui-card-title` and copy. `trialing` `Período de prueba`, with `trial_ends_at` formatted by `format_date` when `Some`; `active` `Suscripción activa`, with `next_charge_at` when `Some`; `past_due` `Pago pendiente`, with `grace_ends_at` when `Some`; `locked` `Sincronización bloqueada` plus `Renueva para volver a sincronizar. La atención clínica sigue disponible.`; `canceled` `Suscripción cancelada` plus `Tus datos clínicos siguen disponibles.`; any other `Estado desconocido`. Adds `Se cancela al final del período actual.` when `cancel_at_period_end`; children render below.
- `PlanCard`: `code: String`, `name: String`, `amount_cents: i64`, `interval_months: i64`, `features: Vec<String>`, `#[props(default)] selected: bool`, `on_select: EventHandler<String>`. Renders `div.ui-plan`. Price is `format_usd_cents(amount_cents)` followed immediately by `/mes` or `/año` (`interval_months == 12` selects `/año`). A small Mist line `Impuestos incluidos` follows. Features render as a list. Selected renders a Mint `Badge` `Plan actual` and no button. Not selected renders a Secondary `Button` `Elegir {name}` that calls `on_select(code)`. Yearly also renders a Mint `Badge` `2 meses gratis`. No primary button here.
- `PaymentMethodCard`: `status: String`, `consented_at: Option<String>`, `on_revoke: EventHandler<MouseEvent>`. Active (`status == "active"`) shows `Token de tarjeta activo · PayPhone`, consent date via `format_date` when `Some`, and a Danger `Button` `Revocar token` calling `on_revoke`. Any other status shows a Neutral `Badge` `Sin token activo` and hint `No hay un token de tarjeta guardado en esta clínica.`. Footer exact: `Solo Visa y Mastercard. No se guarda el número completo ni el CVV.` Never renders token data.
- `BillingBanner`: `status: String`, `grace_ends_at: Option<String>`. `past_due` renders Warning `Notice` `Pago pendiente · gracia hasta {format_date(grace_ends_at)}.` when `Some`, else `Pago pendiente.`; never claim a fixed remaining-day count. `locked` renders Error `Notice` `Sincronización bloqueada — renueva para sincronizar. La atención clínica sigue disponible.`. `canceled` renders Warning `Notice` `Suscripción cancelada. Tus datos clínicos siguen disponibles.`. Other statuses render an empty fragment.
- `BillingTransaction`: `id: String`, `plan_name: String`, `amount_cents: i64`, `tax_cents: i64`, `status: String`, `requested_at: String`. Derives `Clone, PartialEq`.
- `TransactionTable`: `transactions: Vec<BillingTransaction>`. When empty, renders `EmptyState` `Sin transacciones`. Otherwise `DataTable` columns `Fecha`, `Plan`, `Total`, `IVA`, `Estado`; `Fecha` uses `format_date(requested_at)`, `Total` uses `format_usd_cents(amount_cents)`, `IVA` uses `format_usd_cents(tax_cents)`. Status badges: `pending` Cream `Pendiente`, `approved` Mint `Aprobada`, `canceled` Neutral `Cancelada`, `failed` Blossom `Fallida`, `reversed` Plum `Reversada`; other statuses Neutral with the raw status value.
- `format_usd_cents`: `i64` cents to a string. Integer division and remainder only; no float. Always two decimals. Dollars use `.` for thousands groups. `500 -> "$5,00 USD"`, `5000 -> "$50,00 USD"`, `4348 -> "$43,48 USD"`, `123456 -> "$1.234,56 USD"`, `-500 -> "-$5,00 USD"`.

## 8. Page adoption

None. Status and library components are consumed by `19-S` (gallery), `20-T` (navbar/verify), and `24-X` (settings). Billing is consumed only by `19-S` and never by a release route.

## 9. CSS contract

| Selector | Values |
|---|---|
| `.ui-sync` | flex center gap 6, 14px Mist |
| `.ui-sync-dot` | 8px circle |
| `.ui-sync--online` | Mint dot |
| `.ui-sync--offline` | Mist dot |
| `.ui-sync--pending` | Cream dot |
| `.ui-sync--failed` | Blossom dot |
| `.ui-sync-link` | 14px Plum underline; owned by `18-R` — `20-T` consumes it and must not redefine it |
| `.ui-verify-card` | Paper, 1px hairline, radius 24, padding 32, text-align center |
| `.ui-verify-card--valid` | Mint tone |
| `.ui-verify-card--voided` | Blossom tone |
| `.ui-verify-card--superseded` | Cream tone |
| `.ui-verify-card--not-found` | Neutral tone |
| `.ui-verify-card--unknown` | Neutral tone |
| `.ui-verify-title` | grenette 400 46px/1.15 Ink, margin 8px 0; 34px mobile |
| `.ui-verify-copy` | 16px/1.5 Mist, max 44ch, margin 0 auto 24px |
| `.ui-verify-ring` | 80px circle, padding 4px, background `var(--gradient-ember-ring)`, radius 9999, grid center |
| `.ui-verify-seal` | 72px circle, border 1px, grid center, margin 0 auto 16px; valid Mint/Deep Teal, voided Blossom/Plum, superseded Cream/Ink, not found and unknown Paper/Mist with hairline |
| `.ui-verify-details` | text-align left, margin-top 24 |
| `.ui-code-form` | grid 1fr auto, gap 12, align-end; one column below 640; `.ui-field` margin 0 |
| `.ui-llm` | flex wrap align baseline gap 8 margin-top 8; link Deep Teal; hint margin 0 |
| `.ui-library` | flex column gap 16; row flex space-between center gap 16, padding 12px 0, border-top hairline; thumb 48px square radius 8 border hairline object-fit cover, grid center Mist background |
| `.ui-rx-block` | cream background, 1px hairline, radius 8, padding 12px 16px, flex wrap space-between center gap 12; text 14px/1.5 Ink; action uses the shared Secondary button |
| `.ui-quota` | flex column gap 6, margin-bottom 16; track 4px hairline background radius 9999; fill Deep Teal height 100%, radius 9999, width from used/limit |
| `.ui-billing-plans` | grid 2 columns gap 16; one column below 760 |
| `.ui-plan` | Paper background, 1px hairline, radius 24, padding 24, flex column gap 12; selected keeps the same hairline and is marked only by the Mint `Plan actual` badge, never a coloured border or shadow |
| `.ui-plan-price` | mint 500 26px/1.22 Ink |
| `.ui-plan-interval` | mint 400 14px/1.5 Mist |
| `.ui-plan-features` | margin 0, padding-left 18px, grid gap 4, 14px/1.5 Mist |

No new literal colour outside `:root`; no pure black; no drop shadow; no chromatic card border; no gradient except `--gradient-ember-ring` (the verification ring also uses `--gradient-ember-ring`). Cards and tags use the radius scale only.

## 10. Critical files and anchors

| File | Anchor |
|---|---|
| `persona-sanus/src/main.rs` | `struct SyncStatus` 122; `fn sync_status_label` 128; `Navbar` 1376; `Verify` 1297; `SettingsClinic` 1804; `PrescriptionDetail` 5380 |
| `persona-sanus/src/api.rs` | `RxBlockStatus` 180; `AssetRow` 878; `AssetQuota` 894; `VerificationResult` 973 |
| `persona-sanus/src/ui/domain.rs` | New |
| `persona-sanus/src/ui/billing.rs` | New; debug-only |
| `persona-sanus/src/ui/mod.rs` | Add `domain`; add `billing` under `#[cfg(debug_assertions)]` |
| `persona-sanus/assets/main.css` | `:root` 1-58; append domain and billing blocks after `16-P` |
| `context/database-design.md` | §4 status values |
| `context/PRD.md` | FR-043, FR-050, FR-080–085, D-06, D-21 |
| `context/Design.md` | §§Components, Tokens, Surfaces, Elevation, Layout |

## 11. Assumptions and contingencies

- Scope is fixed: full v1 adoption plus presentational v1.1 billing. Do not add a billing route, a plan picker that calls an endpoint, a card form, or a `sync_locked` gate. This gallery is the only billing surface.
- The billing module stays `#[cfg(debug_assertions)]`. A release build must not contain `billing.rs` symbols or rendered billing markup.
- `BillingStatusCard` date fields are `Option<String>` because a status may not carry a relevant date. Render a date only when present; do not invent a placeholder date.
- `PaymentMethodCard` never receives a token. If a future caller has a token field, it is a contract error, not a prop to add here.
- `QuotaMeter` reads 4px track height from the CSS contract. The earlier `1px` phrase in the plan is treated as the hairline stroke, not the height.
- The repository has no `format_bytes` helper. Add the private `domain.rs` helper exactly as section 7; do not repurpose `api::AssetRow.size_bytes` or render a raw byte count.
- Dioxus 0.7 fallbacks: if `NavigationTarget` as a component prop does not accept `Route` directly, `ButtonLink.to` and `NavLink.to` use `Route`; if `Option<Element>` props break the props macro, make `badge` or children required and pass `rsx! {}` at call sites; if `FormEvent` cannot be a component callback, keep the file `<input>` in `SettingsClinic` and share only `Field` and `.ui-file-input` styling. None of these changes the visual contract.
- If the debug server path is not `target/dx/persona-sanus/debug/web/server`, use `dx serve --fullstack --port 8097` for the manual gate. If port 8095, 8096, or 8097 is busy, change only the port and the matching URL.
- If browser verification finds a control not in the Design.md list but required by the existing page, use the nearest existing primitive (`Badge`, `Card`, `Button Secondary`) and do not add a new component file. The component set is closed after `19-S`.

---

*Housekeeping: once a spec is fully done with no open divergences, move its
file to `specs/done/` (same file name, e.g.
`specs/done/18-R-ui-domain-and-billing.md`).
Specs with open divergences stay in `specs/` next to `TEMPLATE.md` until the
tracking file is cleared.*
