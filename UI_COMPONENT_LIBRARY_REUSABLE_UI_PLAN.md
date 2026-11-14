# Reusable UI component library

- Source: `context/Design.md` §Components, §Tokens, §Surfaces, §Elevation, §Layout, §Agent Prompt Guide; screens and states in `context/sitemap.md`, `context/PRD.md` §5, `context/architecture.md` §3/§7, `context/database-design.md` §4.
- Depends on: 01-A through 14-N (all current app surfaces); no new backend or data dependency.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Result carried forward: one reusable Dioxus UI layer in `persona-sanus/src/ui/` implements every Design.md component and the repeated app patterns; all v1 screens render through it, a debug-only `/dev/ui` gallery proves every state, and release builds expose no gallery.

## Context

`persona-sanus/src/main.rs` is 6,177 lines and owns every page inline. Only two reusable render components exist: `forms::A4Preview` (`persona-sanus/src/forms.rs:463`) and `forms::TemplateDocumentForm` (`:635`). `assets/main.css` has the Design.md tokens, but most components are one-off classes (`.button`, `.badge`, `.notice`, `.page-head`, `.editor-section`) or absent.

The request is to develop every reusable component stated by `Design.md` and required by the other context files, skipping what already exists. The selected scope is full v1 adoption: all existing pages use the new primitives. Also include presentational v1.1 billing components, rebuild Home as the centered editorial landing, and add a debug-only `/dev/ui` gallery. There is still no billing route, billing API, billing persistence, or card form in this change.

Hard boundaries: no rich-text, WYSIWYG, contenteditable, formatting toolbar, or formatted-text surface (PRD D-21); every text control stays a plain Dioxus input, textarea, select, or check; no patient photos or signature/stamp images (D-06); no PHI in the marketing page or gallery; public verification renders only doctor, date, and prescription number (FR-050). Add no crate, network font, CDN, icon package, API endpoint, or route other than `/dev/ui`. Authorization remains server-side; the UI only omits actions that the existing role helpers already omit. Preserve every signal, `use_effect`, API call, guard, route, and dispatch path unless a step explicitly says to extract a closure verbatim.

## No-go

- No rich text, WYSIWYG, `contenteditable`, formatting toolbar, `execCommand`, HTML payload, or editor dependency. All text entry is plain controlled inputs, textareas, selects, and checks.
- No new crate, CSS framework, external stylesheet, webfont, CDN, icon package, or JS bridge.
- No new product route, API handler, server function, Diesel model/table/migration, entitlement rule, or billing persistence. `/dev/ui` is the only route addition; it contains no API calls. `/settings/billing` stays reserved with no surface.
- No card form, PAN, CVV, full card number, `ctoken`, `card_token`, or `card_holder_enc` in UI props, gallery copy, or rendered markup. Billing shows status and consent only.
- No change to authorization, session, device registration, sync, numbering, signing, PDF, QR, verification limits, or business validation. Componentization is markup-only unless a step says to extract a closure verbatim.
- No real patient name, cédula, diagnosis, medication, or photo in `/dev/ui`, Home, the base, the nav, or public verification. Gallery and Home use only the constant fictional samples in this plan. The verification card receives only the existing `VerificationResult` fields and never accepts patient data.
- No fake partner logos or endorsements on Home. `TrustLogoStrip` is gallery-only until real marks exist.
- No pure black, drop shadow on UI cards or buttons, chromatic card/image border, bold grenette, or more than one filled primary action in a viewport.

## Design.md coverage

| Design.md component | Symbol | Status |
|---|---|---|
| Primary Action Button | `Button` `Primary` | New; `.ui-button--primary` |
| Outlined Secondary Button | `Button` `Secondary` | New; `.ui-button--secondary` |
| Ghost Nav Button | `Button` `Ghost`, `NavLink` | New; `.ui-button--ghost`, `.ui-nav-link` |
| Form Input and Textarea | `TextField`, `TextAreaField`, `SelectField`, `CheckboxField`, `FileField`, `SearchField` | New; plain controlled Dioxus controls |
| Green Pill Badge | `PillBadge` | New; `.ui-pill` |
| Logo Lockup | `LogoLockup`, `Icon` `Cross` | New; `.ui-logo` |
| Centered Hero Block | `Hero` | New; `.ui-hero` |
| Numbered Step Card | `NumberedStepCard` | New; `.ui-step` |
| A4 Preview Card | `A4PreviewCard`, `A4Mock` | New decorative card; the real A4 document renderer `forms::A4Preview` already exists and is skipped |
| Verification Tag | `VerificationTag` | New; `.ui-verification-tag` |
| Trust Logo Strip | `TrustLogoStrip` | New; gallery only because no real clinic marks exist |
| Section Header | `SectionHeader` | New; `.ui-section-head` |
| Trust & Security Badge Row | `TrustSecurityRow` | New; `.ui-trust` |
| Colors, type, spacing, radii, surfaces, elevation | CSS custom properties | Tokens already exist in `assets/main.css`; extend line-heights, semantic layout values, focus halo, and paper shadow only |
| Existing A4 preview | `forms::A4Preview` | Skip; keep and reuse |
| Existing template editor | `forms::TemplateDocumentForm` | Skip; keep and restyle its internals |

Context coverage: `sitemap.md` shell/nav/roles (Navbar, `PageHeader`, `NavLink`); PRD FR-043 ( `SyncStatusIndicator` ), FR-050 ( `VerificationResultCard` and field-limited code form), FR-080–085 ( `ImageLibrary`/`QuotaMeter`), FR-022/030–034 (unchanged `A4Preview` + `.ui-preview-canvas`), D-21 (plain controlled controls only), D-06 (no signature/stamp controls); `architecture.md` §3 (new `src/ui` module with no SQL/HTTP/storage and a debug-only gallery); `database-design.md` §4 (badges for prescription/template/account/device/billing statuses); `runbook.md` (no runtime or deploy change; release bundle omits the gallery).

## Reuse, do not rebuild

| Existing symbol | Decision |
|---|---|
| `forms::A4Preview` | Keep using it for the true-scale document projection. Only update its surrounding canvas classes and shared primitives. Do not duplicate the layout/pagination engine. |
| `forms::TemplateDocumentForm` | Keep as the template editor. Restyle its internals with the new fields, buttons, and sections; do not recreate its document state or persistence. |
| `Navbar` | Keep the component, router current-route effect, session/sync contexts, role guards, and sign-out/sign-in handlers. Replace its markup and styles; embed `LogoLockup`, `PillBadge`, `SyncStatusIndicator`. |
| `Route`, `App`, page components | Keep. Add only `DevUi`. Do not add `/verify` or `/settings/billing`. |
| `forms::paginate_document`, `resolve_text`, document warnings | Keep unchanged. |
| `SyncStatus`, `probe_sync_status`, local count helpers | Move the existing type and label logic into `src/ui/domain.rs`; `probe_sync_status` and the `#[cfg(feature = "desktop")]` counts stay in `main.rs`. Reuse them from every page. |
| `format_date`, `format_timestamp`, status/role/platform/sex label helpers | Move verbatim into `src/ui/format.rs`; keep `valid_birth_date`, `llm_explain_url`, `new_prescription_item_form`, and all dispatch/submit functions in `main.rs`. |
| `.a4-*` CSS and A4 geometry | Keep; only swap the literal `#111111` for Ink and reduce the page shadow to `--shadow-paper`. Do not change mm sizes, pagination, or QR layout. |

New components live only in `persona-sanus/src/ui/`. There is no second styling system: extend `assets/main.css`; do not add a CSS framework or another stylesheet.

## New component surface

All symbols are re-exported from `src/ui/mod.rs`; `main.rs` and `forms.rs` import them explicitly. All interactive controls render with `aria-*` where stated, a visible `:focus-visible` ring, and no drop shadow. `Button` and `LinkAction` share the `.ui-button` class so their geometry is identical.

| Symbol | File | Props and exact behavior |
|---|---|---|
| `Button` | `button.rs` | `variant: ButtonVariant = Primary`, `button_type: String = "button"`, `disabled: bool`, `loading: bool`, `extra_class: String`, `onclick: EventHandler<MouseEvent> = no-op`, `children`. Renders `button.ui-button.ui-button--{variant}`; sets `type`, `disabled: disabled || loading`, `aria-busy: loading`. No shadow. |
| `ButtonLink` | `button.rs` | `#[props(into)] to: NavigationTarget`, `variant`, `active_class: Option<String>`, `extra_class`, `children`. Renders Dioxus `Link` with `.ui-button` classes and passthrough active class. Used only for in-app navigation. |
| `LinkAction` | `button.rs` | `href: String`, `variant`, `target: Option<String>`, `rel: Option<String>`, `extra_class`, `children`. Renders `<a class="ui-button ...">`; `target`/`rel` are omitted when `None`. Used for PDF, print, share, export, external LLM. |
| `NavLink` | `button.rs` | `#[props(into)] to: NavigationTarget`, `label: String`. Renders `Link.ui-nav-link` with `active_class: "ui-nav-link--active"`. |
| `Icon` | `icon.rs` | `name: IconName`, `size: u16 = 20`, `extra_class: String`. Renders a 24-unit `svg` with RSX attrs `view_box="0 0 24 24"`, `aria_hidden="true"`, `focusable="false"`, `stroke="currentColor"`, `stroke_width="1.5"`, `stroke_linecap="round"`, `stroke_linejoin="round"` (DOM names `viewBox`, `stroke-width`). |

`ButtonVariant`: `Primary`, `Secondary`, `Ghost`, `Danger`. `IconName`: `Cross`, `Shield`, `Command`, `Windows`, `Globe`, `Check`, `Close`, `Alert`, `Info`, `Qr`, `ArrowRight`, `Lock`, `Image`. The filled `Cross` uses `fill="currentColor"` and no stroke; all others are line icons.

Enums: `SurfaceTone` is `Paper | Cream | Mint | Blossom`; `AccentTone` adds `Conic`; `A4MockKind` is `Patient | Medication | Verification`; `FormColumns` is `Two | One`; `BadgeTone` is `Neutral | Mint | Cream | Blossom | Plum`; `NoticeTone` is `Info | Success | Warning | Error`. Every enum derives `Clone, Copy, PartialEq, Eq`. Defaults are `ButtonVariant::Primary`, `SurfaceTone::Paper`, `AccentTone::Mint`, `FormColumns::Two`, `BadgeTone::Neutral`, `NoticeTone::Info`. Class suffixes are lowercase: `ui-band--paper|cream|mint|blossom`, `ui-a4-card--mint|cream|blossom|conic`, `ui-button--primary|secondary|ghost|danger`, `ui-badge--neutral|mint|cream|blossom|plum`, `ui-notice--info|success|warning|error`, `ui-sync--online|offline|pending|failed`, `ui-form-grid--one|two`, `ui-verify-card--valid|voided|superseded|not-found|unknown`, `ui-verify-seal--valid|voided|superseded|not-found|unknown`. `TrustLogo` derives `Clone, PartialEq`.

Forms in `form.rs`:

| Symbol | Props |
|---|---|
| `TextField` | `label: String`, `value: String`, `oninput: EventHandler<String>`, `input_type: String = "text"`, `placeholder: String = ""`, `wide: bool`, `hint: Option<String>`, `error: Option<String>`, `autocomplete: String = "off"`, `required: bool`, `disabled: bool`, `readonly: bool`, `min/max/step: Option<String>`, `hide_label: bool`. Wraps `label.ui-field`; label span `.ui-field-label`; input `.ui-input`. |
| `TextAreaField` | Same text/error/wide props plus `rows: Option<u32>`; `textarea.ui-textarea`. |
| `SelectField` | `label`, `value`, `onchange: EventHandler<String>`, `wide`, `hint`, `error`, `disabled`, `hide_label`, `children` (options). Renders `select.ui-select` inside the same field wrapper. |
| `CheckboxField` | `label: String`, `description: Option<String>`, `checked: bool`, `onchange: EventHandler<bool>`, `disabled: bool`, `wide: bool`. Native checkbox, `accent-color` deep teal. |
| `FileField` | `label: String`, `accept: String`, `disabled: bool`, `hint: Option<String>`, `on_change: EventHandler<FormEvent>`, `wide: bool`. Native file input styled with `.ui-file-input`; label wraps the control. |
| `SearchField` | `label`, `value`, `oninput: EventHandler<String>`, `placeholder: String`, `wide: bool`. Renders a visually hidden label plus `input[type=search].ui-input`. |

Every field with `error: Some(_)` renders `<p class="ui-field-error" role="alert">{error}</p>`; a field with only `hint` renders `.ui-field-hint`. `TextField`, `TextAreaField`, `SelectField` set `aria-invalid: error.is_some()`. Inputs are always controlled. Event callbacks pass `event.value()`; checkbox passes `event.checked()`. Labels wrap the control, so no generated ids are needed. A private `Field` wrapper owns the label/control/hint/error shell and the `wide` class; the text, area, select, file, and search controls compose it.

Feedback in `feedback.rs`:

| Symbol | Props and exact behavior |
|---|---|
| `Badge` | `tone: BadgeTone = Neutral`, `children`. Pill, 13px mint 500, padding 4px 12px, radius 9999, background by tone: Neutral paper/Mist text/hairline border; Mint mint/Ink; Cream cream/Ink/hairline; Blossom blossom/Plum; Plum plum/Paper. Chromatic tones have a transparent border. |
| `PillBadge` | `label: String = "Receta verificable"`. Mint background, Ink 13px 500, 4px 12px, 9999px, no border. The brand mark. |
| `Notice` | `tone: NoticeTone = Info`, `children`. Card: hairline border, 24px radius, 20px padding, icon + body. Info cream, Success mint, Warning blossom, Error blossom with Plum text. `role="alert"` only for Error. |
| `ErrorText` | `text: String`. 14px Plum line with Alert icon; role alert. |
| `HintText` | `text: String`. 14px Mist line. |
| `EmptyState` | `title: String`, `description: String = ""`. Centered cream card, 24px radius, hairline, 32px padding; 18px 500 title; description 14px Mist, max 48ch, centered, rendered only when non-empty. |
| `LoadingState` | `label: String = "Cargando..."`. Spinner plus label, `role="status"`, `aria-live="polite"`; animation disabled by `prefers-reduced-motion`. |
| `ConfirmPanel` | `message`, `confirm_label`, `cancel_label = "Cancelar"`, `busy`, `on_confirm: EventHandler<MouseEvent>`, `on_cancel`, `children`. Inline cream alert group, not a modal; focus is never trapped; children render above the two actions. |
| `DataTable` | `columns: Vec<String>`, `caption: Option<String>`, `children`. `<div class="ui-table-wrap"><table class="ui-table">`; column headers use `th scope="col"`; children are `<tr>`/`<td>`. |
| `ListRow` | `title: String`, `meta: Vec<String>`, `badge: Option<Element>`, `children` (trailing actions). White card, 24px radius, hairline, title 18px 500, meta 14px Mist, actions right aligned. |

Layout and brand in `layout.rs` and `brand.rs`:

| Symbol | Props and exact behavior |
|---|---|
| `PageShell` | `id: String`, `children`. `<div id class="ui-page"><div class="ui-container ui-stack">{children}</div></div>`. `ui-page` = 48px top, 80px bottom, 24px inline padding. `ui-container` = max 1200px, centered. `ui-stack` = 24px vertical gap. |
| `PageHeader` | `title`, `subtitle: Option<String>`, `children` (actions). `<header class="ui-page-head">`; `h1.ui-page-title` (22px mint 500 Ink), `p.ui-page-subtitle` (14px Mist), `.ui-page-head-actions`. |
| `Card` | `children`. `<div class="ui-card">`. Paper, hairline, 24px radius, 24px padding, no shadow. |
| `SectionCard` | `title`, `description: Option<String>`, `children`. Card with `h2.ui-card-title` (20px mint 500 Ink), optional `.ui-card-description` (14px Mist). |
| `FormGrid` | `columns: FormColumns = Two`, `children`. `.ui-form-grid`; two columns, one on mobile; `.ui-form-grid--one` for a single column. |
| `DetailGrid` | `children`. `<dl class="ui-detail-grid">`; two columns, one on mobile. |
| `DetailItem` | `label`, `wide: bool`, `children`. `<div><dt>label</dt><dd>children</dd></div>`; wide spans both columns; `dd` inherits 16px Ink body. |
| `ActionRow` | `children`. `.ui-actions`; 12px gap, wraps, 16px top margin. |
| `FilterBar` | `children`. `.ui-filter-bar`; auto-fit columns at 180px minimum, 12px gap, aligned end. |
| `SectionBand` | `id: Option<String>`, `tone: SurfaceTone = Paper`, `children`. `<section class="ui-band ui-band--{tone}">` with `.ui-container` inside; 80px block padding, 64px below 640. |
| `LogoLockup` | None. Dioxus `Link` to Home with `.ui-logo`, wordmark `PersonaSanus` 22px mint 500 Deep Teal, and a 22px mint circle containing a 14px Deep Teal cross glyph. `aria-label="PersonaSanus — inicio"`; glyph is aria-hidden. |
| `TrustSecurityRow` | None. Centered flex row: `Shield` 18px Deep Teal, text `Datos protegidos · Ley Orgánica de Protección de Datos Personales` 14px 500 Ink, then Command, Windows, Globe icons 18px Mist at 12px gaps. Adds a visually hidden platform label `"macOS, Windows, Web"`. |
| `TrustLogoStrip` | `caption = "Con la confianza de…"`, `logos: Vec<TrustLogo>`. `TrustLogo { name: String, src: Option<String> }`. Centered caption 14px Mist; one centered wrapping row with 44px column gap. Text logo 18px mint 500 Ink; image max-height 28px, grayscale. No borders or cards. The gallery demo passes five fictitious wordmarks and the note `Logos de ejemplo; no son clientes reales.` |

Marketing in `marketing.rs`:

| Symbol | Props and exact behavior |
|---|---|
| `Hero` | `title`, `subtitle`, `children`. White, max 720px, centered; `h1.ui-display` grenette 400, clamp(46px, 7vw, 60px), line-height 1.05, Ink, balanced text; subtitle 18px/1.5 Mist max 600px; children render below the subtitle, centered, and may include `ActionRow`, microcopy, and `TrustSecurityRow`. |
| `SectionHeader` | `title`, `subtitle: Option<String>`. Centered max 640px, `h2.ui-section-title` grenette 400 46px (34px mobile), Ink; subtitle 18px/1.5 Mist max 560px. No decorative element above the title. |
| `NumberedStepCard` | `number: u8`, `title`, `body`, `accent: AccentTone`, `mock: A4MockKind`, `tag: Option<String>`. Centered column: `A4PreviewCard` above, then 28px circle with 1px Ink border and 14px mint 500 number, 20px mint 500 title, 16px mint 400 Mist body. No card background. |
| `A4PreviewCard` | `accent: AccentTone = Mint`, `tag: Option<String>`, `children`. 24px radius, 24px padding, no border, no shadow; accent mint/cream/blossom/conic; inner `.ui-a4-sheet` white, 8px radius, `aspect-ratio: 210 / 297`, clipped. Optional `VerificationTag` absolute top right. |
| `A4Mock` | `kind: A4MockKind`. Static fake document: `Patient` shows `FICHA DEL PACIENTE`, `María Fernanda Ruiz`, rows Cédula/Edad/Especialidad; `Medication` shows `RECETA`, `Diagnóstico / Faringitis aguda`, `1. Amoxicilina 500 mg`, `Cada 8 horas · 7 días`; `Verification` shows `VERIFICACIÓN`, a CSS QR glyph, `PS-7K2F-9Q`, and a 44px conic ring with Check. |
| `VerificationTag` | `label: String`. Plum pill, Paper 13px 500, 6px 12px, radius 9999, 6px triangular tail pointing down; `ui-verification-tag`. |

Domain components in `domain.rs`:

| Symbol | Props and exact behavior |
|---|---|
| `SyncStatus` | Existing public fields `online: bool`, `pending: i64`, `failed: i64`; `#[derive(Debug, Clone, Copy, PartialEq, Default)]`. `SyncStatus::label` chooses in order: offline `"Sin conexión"`, failed `"Error de sincronización"`, pending `"Sincronizando"`, else `"En línea"`. `SyncStatus::tone` returns the matching `offline | failed | pending | online` slug. |
| `SyncStatusIndicator` | `status: SyncStatus`. Inline dot plus `status.label()`. Dot precedence is the label precedence: Mist offline, Blossom failed, Cream pending, Mint online. Text is 14px Mist; `role="status"`, `aria-live="polite"`. When `pending > 0`, suffix `· {n} pendientes` with singular `1 pendiente`. No link or router dependency. |
| `PrescriptionStatusBadge` | `status: String`. Draft Neutral `Borrador`; signed Mint `Firmada`; superseded Cream `Reemplazada`; voided Blossom `Anulada`; pending_sync Cream `Provisional`; failed Blossom `Error de sincronización`; other Neutral `Desconocido`. |
| `TemplateStatusBadge` | `status: String`. Draft Neutral `Borrador`; published Mint `Publicada`; archived Cream `Archivada`; other Neutral `Desconocido`. |
| `AccountStatusBadge` | `status: String`. Active Mint `Activo`; disabled Neutral `Deshabilitado`; other Neutral `Desconocido`. |
| `DeviceStatusBadge` | `revoked: bool`. Mint `Activo` or Blossom `Revocado`. |
| `RxBlockIndicator` | `status: Option<api::RxBlockStatus>`, `allocating: bool`, `on_allocate: EventHandler<MouseEvent>`. In a cream hairline strip. Inactive shows `Sin bloque de números activo.`; active shows `Números disponibles: {remaining} de {limit}`; exhausted also shows `Bloque agotado. Reserva otro antes de firmar.`. The `Button` Secondary `Reservar bloque` renders when inactive or exhausted, and is disabled by `allocating`. Renders nothing when status is None. |
| `LlmExplainLink` | `href: String`. External `a` (target `_blank`, rel `noopener noreferrer`) `Explicar con LLM`, plus 14px Mist disclaimer `Se abre un sitio externo con el nombre del medicamento. No se envían datos del paciente.`. |
| `VerificationSeal` | `status: String`. 72px circle. Valid is Mint with a Deep Teal check inside an 80px conic-gradient ring. Voided is Blossom with a Plum close. Superseded is Cream with an Ink arrow. `not found` is Paper/Mist with a hairline and the Qr icon. Unknown is Paper/Mist with a hairline and the Info icon. `aria-hidden`. |
| `VerificationResultCard` | `verification: api::VerificationResult`. Eyebrow `VERIFICACIÓN PÚBLICA`. Title and copy: valid `Receta auténtica` / `Esta receta fue emitida y firmada por un profesional autorizado en PersonaSanus.`; voided `Receta anulada` / `El médico emisor anuló esta receta. El documento ya no es válido.`; superseded `Receta reemplazada` / `Esta receta fue reemplazada por una corrección posterior. Consulta la versión vigente.`; `not found` `Receta no encontrada` / `El código no corresponde a una receta emitida. Revisa el código impreso junto al QR.`; other `Receta no verificable` / `No pudimos determinar el estado de este documento.`. Tones Mint, Blossom, Cream, Neutral, Neutral. Details only for valid/voided/superseded: `Médico`, `Fecha de emisión`, `No. de receta` (`{n:06}`), each `—` fallback. Never accepts or renders a patient field. |
| `VerificationCodeForm` | `button_variant: ButtonVariant = Primary`, `on_submit: EventHandler<String>`. Local value signal; labeled `Código de verificación`; placeholder `PS-7K2F-9Q`; `autocomplete="off"`; `spellcheck="false"`; form `onsubmit` prevents default, trims and uppercases, shows `ErrorText` `Ingresa el código de verificación.` when empty, otherwise calls `on_submit` once; Button `button_variant`, label `Verificar`, `button_type="submit"`. Home passes `Secondary`; Verify uses the default. |

Image library in `domain.rs`:

| Symbol | Props and exact behavior |
|---|---|
| `QuotaMeter` | `used: i64`, `limit: i64`. Text `Uso de la biblioteca: {used} de {limit}.` plus 1px hairline track and Deep Teal fill of `used/limit` clamped 0..1; `role="progressbar"`, `aria-valuemin=0`, `aria-valuemax=limit`, `aria-valuenow=used`. No colour change at quota. If `limit <= 0`, the fill is 0 and `aria-valuemax` is 1. |
| `ImageLibrary` | `assets: Vec<api::AssetRow>`, `used`, `limit`, `uploading`, `deleting: Option<String>`, `online`, `on_delete: EventHandler<String>`, `on_upload: EventHandler<FormEvent>`. QuotaMeter, list, empty state, file field. Lists only `kind == "image"`; the logo is ignored. A pending asset shows a Cream `Procesando` badge and no thumbnail. A ready asset shows a 48px thumbnail when `url` exists, else a hairline placeholder with the `Image` icon; the row also shows original name or `Imagen {first 8 id chars}`, `{width} × {height}` (`—` for a missing dimension), and `{format_bytes(size_bytes)}` (1024-based; one decimal for `KB`/`MB`, integer `B`). Delete uses Danger Button, disabled when deleting or offline; the online-only hint stays visible. |
| `ImageLibrary` upload | File label `Subir imagen de la biblioteca (JPEG o PNG, máximo 10 MB)`, accept `image/jpeg,image/png`. Disabled when `!online || uploading || used >= limit`; hint is `Subir imágenes requiere conexión.` when offline, `La biblioteca está llena.` when at quota, otherwise the 10 MB hint. Calls `on_upload` only with the first file. Empty list renders `EmptyState` `Sin imágenes` / `Sube una imagen JPEG o PNG de hasta 10 MB.` |

Presentational v1.1 billing in `billing.rs`; no fetch, route, or persistence:

| Symbol | Props and exact behavior |
|---|---|
| `BillingStatusCard` | `status`, `trial_ends_at`, `next_charge_at`, `grace_ends_at: Option<String>`, `cancel_at_period_end: bool`, `children`. Statuses and copy: `trialing` `Período de prueba` with trial end; `active` `Suscripción activa` with next charge; `past_due` `Pago pendiente` with grace end; `locked` `Sincronización bloqueada` with `Renueva para volver a sincronizar. La atención clínica sigue disponible.`; `canceled` `Suscripción cancelada` with `Tus datos clínicos siguen disponibles.`; unknown `Estado desconocido`. Adds `Se cancela al final del período actual.` when `cancel_at_period_end`. |
| `PlanCard` | `code`, `name`, `amount_cents`, `interval_months`, `features: Vec<String>`, `selected: bool`, `on_select: EventHandler<String>`. Price from `format_usd_cents`; suffix `/mes` or `/año`; small Mist line `Impuestos incluidos`. Selected shows Mint `Plan actual`, no button. Otherwise Secondary `Elegir {name}` calling `on_select(code)`. Yearly also shows Mint `2 meses gratis`. No primary button here. |
| `PaymentMethodCard` | `status`, `consented_at: Option<String>`, `on_revoke: EventHandler<MouseEvent>`. Active: `Token de tarjeta activo · PayPhone`, consent date, Danger `Revocar token`. Revoked: Neutral `Sin token activo`, hint. Never renders the token. Footer exact: `Solo Visa y Mastercard. No se guarda el número completo ni el CVV.` |
| `BillingBanner` | `status`. past_due Warning `Pago pendiente · quedan 7 días de gracia.`; locked Error `Sincronización bloqueada — renueva para sincronizar. La atención clínica sigue disponible.`; canceled Warning `Suscripción cancelada. Tus datos clínicos siguen disponibles.`; other renders nothing. |
| `BillingTransaction` | Plain `Clone + PartialEq` struct: `id`, `plan_name`, `amount_cents`, `tax_cents`, `status`, `requested_at`. |
| `TransactionTable` | `transactions: Vec<BillingTransaction>`. `DataTable` columns `Fecha`, `Plan`, `Total`, `IVA`, `Estado`; money via `format_usd_cents`; EmptyState `Sin transacciones` when empty. Status badges: pending Cream `Pendiente`, approved Mint `Aprobada`, canceled Neutral `Cancelada`, failed Blossom `Fallida`, reversed Plum `Reversada`. |
| `format_usd_cents` | Integer cents to `$` + grouped dollars + `,` + two decimals + ` USD`: `500 -> "$5,00 USD"`, `5000 -> "$50,00 USD"`, `4348 -> "$43,48 USD"`, `123456 -> "$1.234,56 USD"`, `-500 -> "-$5,00 USD"`. Integer division/remainder only, no float, always two decimals, `.` groups thousands. |

`DevUiGallery` lives in `gallery.rs`, uses every symbol above, and renders static sample values only. Each section root is `<section id="gallery-{name}">`; required names are `tokens`, `layout`, `buttons`, `forms`, `badges`, `feedback`, `data`, `brand`, `marketing`, `status`, `verification`, `library`, `billing`. It is a component index, so it is exempt from the one-primary-per-viewport rule. It is not linked from a release-visible surface. `Settings` adds a debug-only link to `/dev/ui`.

## Approach

Build this as a tree: steps 2-9 add the UI layer, CSS, and gallery; steps 10-16 adopt screens and can be done in any page order after step 9 because the shared contracts no longer change; step 17 removes the legacy CSS only after every screen is migrated; step 18 records the docs. Run `cargo check` after each step. Do not leave comments in Rust, CSS, or RSX. Do not add tests; proof is the e2e checks below.

- [ ] 1. Record the spec. Create `specs/15-O-ui-component-library.md` from `specs/TEMPLATE.md` with Source `context/Design.md §Components and §Tokens; context/sitemap.md; context/architecture.md §3`, Depends on `01-A through 14-N`, Repo root `/Users/sergiobarba/projects/doctor-prescriptions`, and the Result carried forward above. Copy this plan's Context, No-go, Reuse, Component surface, Page adoption, CSS contract, Critical files, Verification, and Assumptions into the matching spec sections. Set section 1 Objectives to the actions in plan steps 2-18, renumbered 1-17, all unchecked. Omit this step 1 from that list. Never check a box until its listed command passes.

- [ ] 2. Establish tokens and layout rhythm in `persona-sanus/assets/main.css`. Keep existing token names. Add `--leading-caption: 1.5`, `--leading-body-sm: 1.5`, `--leading-body: 1.5`, `--leading-body-lg: 1.5`, `--leading-subheading: 1.25`, `--leading-heading-sm: 1.22`, `--leading-heading: 1.22`, `--leading-heading-lg: 1.15`, `--leading-display: 1.05`; keep the existing `--font-weight-regular/medium`, add `--content-display: 720px`, `--content-copy: 560px`, `--section-gap: 80px`, `--section-gap-compact: 64px`, `--element-gap: 8px`, `--element-gap-lg: 16px`, `--radius-pill: 9999px`, `--color-mint-halo: rgba(185, 255, 232, 0.55)`, `--shadow-paper: 0 1px 2px rgba(6, 29, 41, 0.06)`, `--navbar-height: 64px`. Replace the invalid `--section-gap: 64-80px` and `--element-gap: 8-16px`; never leave an invalid length. Add `*, *::before, *::after { box-sizing: border-box; }`. Add `.ui-page`, `.ui-container` (1200px), `.ui-container--narrow` (720px), `.ui-container--reading` (960px), `.ui-stack` (24px gap), `.ui-visually-hidden`, `.ui-mono`, `.ui-break`, `.ui-notes`, `.ui-text-caption/body-sm/body/body-lg/subheading`. No pure black. No new stylesheet, no CSS framework.

- [ ] 3. Add `src/ui/mod.rs`, `icon.rs`, and `button.rs`; add `mod ui;` to `persona-sanus/src/main.rs`. Implement the exact symbols in the tables. `Icon` uses inline SVG path data; `Cross` is the filled exception. `Button` sets a safe `type` (default `button`) and never uses inline style or JS. `ButtonLink` wraps the Dioxus `Link` so a link remains an announcement, not a fake button. `LinkAction` omits `target` and `rel` when `None`. All focus states use the same utility: `outline: 1px solid var(--color-deep-teal); outline-offset: 2px; box-shadow: 0 0 0 3px var(--color-mint-halo);`. No shadow otherwise. Also add `src/ui/format.rs` and move these helpers from `main.rs` verbatim: `format_date`, `format_timestamp`, `last_seen_label`, `role_label`, `status_label`, `platform_label`, `device_name_label`, `sex_label`, `optional_date`, `prescription_status_label`, `template_status_label`, `version_status_label`, `template_owner_label`. Re-export them from `ui/mod.rs`; replace their use in `main.rs` with one explicit import list. Keep `valid_birth_date`, `llm_explain_url`, `new_prescription_item_form`, and all dispatch/submit functions in `main.rs`. Verification: `cargo check`; `rg -n '^fn (format_date|format_timestamp|last_seen_label|role_label|status_label|platform_label|device_name_label|sex_label|optional_date|prescription_status_label|template_status_label|version_status_label|template_owner_label)' persona-sanus/src/main.rs` returns nothing.

- [ ] 4. Add `form.rs` with the six controls from the contract and the field CSS: hairline control, 8px radius, 8px 12px padding, 16px Ink mint 400; label 14px mint 500 Ink; placeholder Mist; focus uses the step-3 ring. Checkbox is native 16px with `accent-color: Deep Teal`. File input uses the same control shell, 14px Mist text, and a secondary mini file-selector button; include `::file-selector-button` and `::-webkit-file-upload-button`. Disabled controls keep readable contrast (cream background, mist text) and `cursor: not-allowed`. There is no toolbar, no contenteditable, no rich text. Verification: `cargo check`; `rg -n 'contenteditable|execCommand|innerHTML' persona-sanus/src` returns nothing.

- [ ] 5. Add `feedback.rs` and `layout.rs`. Implement `Badge`, `PillBadge`, `Notice`, `ErrorText`, `HintText`, `EmptyState`, `LoadingState`, `ConfirmPanel`, `DataTable`, `ListRow`, `PageShell`, `PageHeader`, `Card`, `SectionCard`, `FormGrid`, `DetailGrid`, `DetailItem`, `ActionRow`, `FilterBar`, `SectionBand` exactly as contracted. Card family: paper, 1px hairline, 24px radius, 24px padding, no shadow. Badges, notices, table, and list rows use the same hairline and 24px radius vocabulary. The spinner is the only animation; honor `prefers-reduced-motion`. Verification: `cargo check`; source style audit contains no `box-shadow` except the focus halo and `--shadow-paper`.

- [ ] 6. Add `brand.rs` and `marketing.rs`. Implement `LogoLockup`, `TrustSecurityRow`, `TrustLogoStrip`, `Hero`, `SectionHeader`, `NumberedStepCard`, `A4PreviewCard`, `A4Mock`, and `VerificationTag`. Colour and geometry come only from tokens and the values in the component table. The logo cross is the only decorative mark in the brand lockup. The hero/section headings are grenette 400 and never bold. Verification: `cargo check`; `rg -n 'font-weight: *700|font-weight: *bold' persona-sanus/src/ui persona-sanus/assets/main.css` returns only intentional A4 document styles, if any.

- [ ] 7. Add `domain.rs` status, sync, RX, LLM, verification, and image-library components from the contract. Move the existing `SyncStatus` struct from `main.rs` into this file; make its fields public, add the same `Debug, Clone, PartialEq` derives plus `Copy` and `Default`, replace the free `sync_status_label` with `SyncStatus::label` and `SyncStatus::tone`, and remove the old definitions. Import the moved type where the context is provided. `PrescriptionStatusBadge`, `TemplateStatusBadge`, `AccountStatusBadge`, `DeviceStatusBadge`, and `SyncStatusIndicator` must use string labels via `match`; no allocation for a label except the count string. `VerificationResultCard` only reads `status`, `doctor_name`, `issued_at`, `rx_number`; any compile attempt to read patient data is a defect. `ImageLibrary` uses the existing `api::AssetRow`/`AssetQuota` shapes and byte-size formatting; `format_bytes` prints `B`, `KB`, or `MB` with one decimal for non-bytes. Verification: `cargo check`; `rg -n -i 'card_token|card_holder|ctoken|authorization_code|card_number|\bcvv_number\b|\bpan_number\b' persona-sanus/src/ui` returns nothing.

- [ ] 8. Add `billing.rs` with the exact presentational component set. Use integer cents only. Render currency with `format_usd_cents`. Never render `card_token`, `card_holder_enc`, `ctoken`, `authorization_code`, PAN, CVV, or card numbers; status, consent date, plan price, amount, tax, and transaction status only. Verification: `cargo check`; `rg -n -i 'card_token|card_holder|ctoken|authorization_code|card_number|\bcvv_number\b|\bpan_number\b' persona-sanus/src/ui` returns nothing.

- [ ] 9. Add `gallery.rs` and the `#[route("/dev/ui")] DevUi {}` variant to `Route` outside the `#[layout(Navbar)]` block. In `ui/mod.rs`, declare `gallery` and `billing` only under `#[cfg(debug_assertions)]` and re-export `DevUiGallery` under the same gate. `DevUi` returns `ui::DevUiGallery` in a debug build; the non-debug branch returns a single `.ui-card` with `Esta vista solo existe en compilaciones de desarrollo.`. The gallery root is `PageShell id="dev-ui-page"` wrapping `div.ui-gallery`; it opens with `LogoLockup`, an `h1` `Sistema de diseño`, the line `Componentes reutilizables de PersonaSanus. Solo desarrollo.`, and a Secondary `ButtonLink` `Volver al inicio`; it is static, with no contexts, API calls, session, or router state. Include every component and state in the tables, including all four verification results, all billing states, both RX block states, all status badge tones, disabled/loading buttons, wide/error fields, table, empty state, trust strip, all enum variants, and an icon strip covering every `IconName`. Under the trust strip add `Logos de ejemplo; no son clientes reales.` Add a debug-only `Sistema de diseño` link in `Settings` to `Route::DevUi`. Verification: `cargo check`.

- [ ] 10. Adopt the shell and auth screens. In `Navbar`, replace `#navbar` internals with `header.ui-navbar` > `.ui-navbar-inner`; brand group uses `LogoLockup` + `PillBadge`; centre is `nav.ui-navbar-links` of `NavLink` (keep the current role guards with exactly five destinations: Panel, Pacientes, Recetas, Plantillas, Configuración (the sitemap's five); remove Mi perfil, Usuarios, and Auditoría from the bar because Settings links to them; remove the old text Home link because the logo goes Home); right group uses `SyncStatusIndicator`, optional failed link `{n} con error` to the filtered prescriptions route, session email `.ui-navbar-user`, and `Button` Ghost `Cerrar sesión`; signed-out right uses `ButtonLink` Ghost `Iniciar sesión`. Keep the router effect. In `AuthPage` signed-out, use `PageShell id="auth-page"` > `div.ui-auth-shell` (max 440) > `.ui-auth-head` with `h1.ui-auth-title` (`Bienvenido de nuevo` or `Crea tu cuenta`), then `.ui-segmented` of two Ghost `Button`s (`Iniciar sesión` active when `!sign_up_mode`, `Crear cuenta` active when `sign_up_mode`), then a `Card` with `FormGrid columns: One`: `Nombre` only for sign-up, `Correo` with `input_type="email"` and `autocomplete="email"`, `Contraseña` with `input_type="password"` and `autocomplete="current-password"`/`"new-password"`, optional `Código de verificación` when a ticket exists; `ErrorText` for `error`; one Primary `Button` whose label is `Procesando...` while submitting, else `Iniciar sesión`/`Crear cuenta`, calling the existing `submit` closure. When `mfa_ticket` is non-empty, show only the code field with Primary `Verificar código` (existing `verify_mfa`) and Secondary `Volver` (clears the ticket and error). Keep the click closures; do not add a native form submit. Signed-in branch uses `PageShell`, `PageHeader` `Sesión iniciada`, `Card`, `DetailGrid`/`DetailItem` for `Nombre`/`Correo`, and `Button` Ghost `Cerrar sesión`. Rename visible English strings to the es-EC copy table. Verification: `cargo check`; a browser pass on `/login` covers both signed-out modes, MFA field, and signed-in card.

- [ ] 11. Rebuild `Home` as the landing. Its root is `<div id="home-page">`; do not wrap it in `PageShell` because the bands must stay full-bleed. Keep it inside the navbar layout. Resolve the CTA before RSX in this exact order: no session -> `Route::AuthPage {}` with label `Emitir receta`; session plus a profile with `can_create_prescriptions()` -> `Route::PrescriptionNew { patient_id: None, template_id: None }` with label `Emitir receta`; session plus a profile with a role but no create permission -> `Route::Prescriptions { status: None }` with label `Ver recetas`; session with no profile or no role -> `Route::SettingsProfile {}` with label `Completar perfil`. Render a centered `Hero` with title `Recetas claras, verificables y listas para firmar.` and subtitle `PersonaSanus reúne pacientes, plantillas y recetas en un espacio sereno, pensado para el trabajo clínico en Ecuador.`. Inside the Hero render an `ActionRow` holding the primary `ButtonLink` (label/routes above) and a Secondary `LinkAction` `Verificar una receta` `href="#verificar"`, then `p.ui-microcopy` with `Diseñado para clínicas y consultorios · Datos protegidos`, then `TrustSecurityRow`. Then a `SectionBand` id `como-funciona` with `SectionHeader` `Cómo funciona` and subtitle `Del paciente a la receta firmada en tres pasos.`, `ui-steps` containing three `NumberedStepCard`s: 1 `Elige el paciente`, 2 `Completa la receta`, 3 `Firma y verifica`; accents Mint, Cream, Blossom; mocks Patient, Medication, Verification; step 3 tag `Verificable`; then a centered `p.ui-steps-note` `Vistas de ejemplo del formato A4.`. Then a Mint `SectionBand` id `verificar`; inside it, a `div.ui-verify-inline` holds a centered `SectionHeader` `Verifica una receta` / `Ingresa el código impreso junto al QR. La página muestra únicamente el médico, la fecha y el número de la receta: nunca los datos del paciente.`; `Card` containing `VerificationCodeForm { button_variant: Secondary }` wired to `navigator.push(Route::Verify { code })`; then `TrustSecurityRow`. Do not render `TrustLogoStrip` on Home because no real partner marks exist. Verification: `cargo check`; `curl /` after the build contains the hero title and exactly one `ui-button--primary`.

- [ ] 12. Adopt `Verify` (`/verify/:code`) to the public shell. Root `<div id="verify-page" class="ui-public">`; `.ui-public-head` holds `LogoLockup` + `PillBadge`; `.ui-public-main` is a `ui-stack`. The conditional region is only `LoadingState` `Verificando receta…`, `Notice` Error for fetch failure, and `VerificationResultCard`. Below it, always render `SectionCard` `Verificar otra receta` with `VerificationCodeForm` (default Primary) and `TrustSecurityRow`. Keep the existing `requested` fetch signal; at the start of each fetch set `loaded` false, clear `message`, and clear the old result; wire the form's `on_submit` to trim/uppercase, set `requested`, and call `navigator.replace(Route::Verify { code })` so re-typing on the same page refetches. Preserve the no-auth route. The page must never insert a patient name or medication field. Verification: `cargo check`; browser unknown code shows `Receta no encontrada` and doctor/date/number only for found results.

- [ ] 13. Adopt `Dashboard`, `Settings`, `Patients`, `PatientNew`, `PatientDetail`, `PatientEdit`. Apply the page table below. Keep every guard and navigation target. `Settings` adds the admin `Auditoría` link and the debug-only `Sistema de diseño` link; the other pages add a `loaded` signal only where the table says so, set it after each fetch resolves, and never change API arguments unrelated to that flag. The named primary action is the only `.ui-button--primary` on that page.

- [ ] 14. Adopt `Templates`, `TemplateNew`, `TemplateEdit`, `TemplatePreview`, and `forms::TemplateDocumentForm`. Replace the form's raw controls with `TextField`, `TextAreaField`, `SelectField`, `Button`, `Card`/`SectionCard`, `FormGrid`, `ActionRow`, `ErrorText`, and `HintText`. Keep the `TemplateDocument` shape, serde validation, variable insertion, asset selection, medication add/remove, save dispatch, and `on_saved` payload unchanged. `A4Preview` stays; replace its parse-error `p.error` with `ErrorText` and keep the pagination/geometry return unchanged. `Guardar borrador`, `Agregar imagen`, `Agregar medicamento`, and `Insertar variable` are Secondary buttons; each remove control is Danger; the page's `Publicar versión` is the sole Primary on `TemplateEdit`.

- [ ] 15. Adopt `Prescriptions`, `PrescriptionNew`, `PrescriptionPreview`, `PrescriptionDetail`. Apply the page table and the one-primary rule. Keep query/status logic, desktop local fallbacks, outbox sync updates, `sign_prescription_dispatch`, `void_prescription_dispatch`, `supersede_prescription_dispatch`, reorder/upsert/delete API calls, the acknowledge-warnings path, and role gates exactly. Use `ListRow` per prescription; raw `div.ui-item-card` + `FormGrid` per medication item; `ConfirmPanel` for void. Add an `allocating` signal around the existing RX-block request.

- [ ] 16. Adopt `SettingsClinic`, `SettingsUsers`, `SettingsProfile`, `Audit`. Apply the page table. `SettingsClinic` reads `Signal<SyncStatus>` and passes `online` to `ImageLibrary`; extract the existing upload/delete bodies into named closures without changing API calls. No page gains a new endpoint, permission, or editable state.

## Page adoption

Shared rules: `PageShell` owns width and root id. `PageHeader` owns one `h1`, the subtitle, and header actions. The existing visible `h4` becomes `PageHeader.title` unchanged; the existing explanatory `p` becomes `PageHeader.subtitle` unchanged; do not invent page copy beyond this table and the auth/Home/Verify copy named elsewhere. A page has at most one `Primary` action. API errors keep their current surface (usually `ErrorText`); pre-existing positive text is not moved into a new error state. Table rows are plain `<tr><td>` passed to `DataTable`.

| Page | Adoption | Only primary |
|---|---|---|
| `Dashboard` | `PageShell id="dashboard-page"`; `PageHeader` `Panel` with session subtitle; `Card` `Sesión`; `DetailGrid` `Nombre`, `Correo`, `Rol` (`Badge`), `Clínica` or `Sin clínica asignada.`; `LoadingState` while `!checked()`. Remove the duplicate page sign-out; the navbar keeps it. | None |
| `Settings` | `PageShell id="settings-page"`; `PageHeader` `Configuración`; `div.ui-link-grid` with Secondary `ButtonLink` cards `Usuarios y roles`, `Clínica y biblioteca`, `Mi perfil`, admin `Auditoría`; debug-only Secondary `Sistema de diseño`. Keep redirects. | None |
| `Patients` | `PageShell id="patients-page"`; `PageHeader` `Pacientes` with Primary `Nuevo paciente` when `can_write_patients`; `SearchField label="Buscar paciente"`; `loaded` set true after fetch; `LoadingState`; `EmptyState` `Sin pacientes` / `No hay pacientes para esta búsqueda.`; `ListRow` title full_name, meta `[Cédula …, Nacimiento …]`, Secondary `Ver ficha`. Keep fetch and desktop fallback. | `Nuevo paciente` |
| `PatientNew` | `PageShell id="patients-page"`; `PageHeader` `Nuevo paciente`, header Secondary `Cancelar`; `Card` + `FormGrid`; `TextField` names/cédula/birthdate(step `date`)/tel; `SelectField` sex; `TextAreaField wide` notes; `ErrorText`; duplicate `Notice` Warning with Danger `Crear de todas formas`; `ActionRow` Primary `Guardar paciente`. Keep `submit_new_patient`. | `Guardar paciente` |
| `PatientDetail` | `PageShell id="patients-page"`; `PageHeader title=full_name subtitle="Ficha del paciente"`; header controls: Secondary `Editar` when `can_write_patients`, else Neutral `Solo lectura`; Primary `Nueva receta` when `can_create_prescriptions`; Secondary `Historial de recetas`; `Card` + `DetailGrid`; delete `ConfirmPanel` with Danger `Eliminar paciente` and confirm `Confirmar eliminación`; `ErrorText`. Keep delete API. | `Nueva receta` if allowed, else none |
| `PatientEdit` | `PageShell id="patients-page"`; `PageHeader` `Editar paciente` with Secondary `Cancelar`; `Card` + `FormGrid` same fields; `ErrorText`; `ActionRow` Primary `Guardar cambios`. Keep dispatch. | `Guardar cambios` |
| `Templates` | `PageShell id="templates-page"`; `PageHeader` `Plantillas` with Primary `Nueva plantilla`; `loaded`; `LoadingState`; `EmptyState` `Todavía no hay plantillas.`; `ListRow` title name, meta owner/current version, badge `TemplateStatusBadge`; actions Secondary `Editar`, `Vista previa` when not archived. | `Nueva plantilla` |
| `TemplateNew` | `PageShell id="templates-page"`; `PageHeader` `Nueva plantilla` with Secondary `Cancelar`; `Card` + `FormGrid` with one `TextField` `Nombre`; `HintText` `Tipo: {owner}`; `ErrorText`; `ActionRow` Primary `Crear plantilla`. Keep create dispatch/redirect. | `Crear plantilla` |
| `TemplateEdit` | `PageShell id="templates-page"`; `PageHeader` title model.name, subtitle owner/status, Secondary `Vista previa`, `Volver`; archived Warning `Notice`; `TemplateDocumentForm`; `ActionRow` Primary `Publicar versión` disabled until `saved_draft`, Secondary `Recargar`, Danger `Archivar plantilla`; archive `ConfirmPanel` message `La plantilla archivada no admite cambios.` confirm `Confirmar archivo`, cancel `Cancelar`; `ErrorText`. Keep dispatch bodies and generation keys. | `Publicar versión` |
| `TemplatePreview` | `PageShell id="templates-page"`; `PageHeader` `Vista previa A4` subtitle `210 × 297 mm con valores de ejemplo.` with Secondary `Volver a plantillas`, `Abrir editor`; controls `Card` + `FormGrid` with `SelectField` version and `TextField` number margin 5..40; warnings `Notice` Warning `Variables no válidas`; `.ui-preview-canvas`; keep `A4Preview` args. | None |
| `Prescriptions` | `PageShell id="prescriptions-page"`; `PageHeader` `Recetas` with Primary `Nueva receta` when `can_create_prescriptions`; `FilterBar` SearchField `Paciente`, SelectField `Estado` same options, TextField date `Fecha`; `loaded`; `LoadingState`; `EmptyState` `Sin recetas`; `ListRow` title patient_name, meta status/date/No., badge `PrescriptionStatusBadge`, Secondary `Abrir`; pending-sync `Badge` Cream `Provisional · PENDIENTE DE SINCRONIZACIÓN`; failed rows in `SectionCard` `Errores de sincronización` with `ListRow` title operation meta `[detail]`. Keep `failed_rows` and desktop branch. | `Nueva receta` |
| `PrescriptionNew` | `PageShell id="prescriptions-page"`; `PageHeader` `Nueva receta` with Secondary `Cancelar`; `Card` + `FormGrid`; `SelectField` patient/template, `TextAreaField` diagnosis, `TextField` validity, `TextAreaField wide` instructions; `ErrorText`; `ActionRow` Primary `Crear borrador`. Keep published filter and API/desktop dispatch. | `Crear borrador` |
| `PrescriptionPreview` | `PageShell id="prescriptions-page"`; `PageHeader` title `Vista previa A4 — {patient_name}`, subtitle status/number/date, Secondary `Volver a la receta`; `ActionRow`: Primary `Firmar y emitir` when draft+can_sign (Secondary `Firmar reconociendo las advertencias` when ack needed), otherwise Primary `LinkAction` `Descargar PDF`, Secondary `LinkAction` `Imprimir`, Secondary desktop `PDF provisional (local)`, Secondary wasm `Compartir`; Warning `Notice` when provisional; `ErrorText`; `.ui-preview-canvas` with `forms::A4Preview`. Preserve offline sign and share branches. | One of `Firmar y emitir` / `Descargar PDF` |
| `PrescriptionDetail` | `PageShell id="prescriptions-page"`; `PageHeader` title patient_name, subtitle status/number/date, Secondary `Vista previa A4`, `Volver`; `RxBlockIndicator`; draft edit `Card` + `FormGrid`; `ActionRow`: when `can_sign`, Primary `Firmar receta` plus Secondary `Guardar borrador`; when the role is nursing, Primary `Guardar borrador`; otherwise Secondary `Guardar borrador` only; `SectionCard` `Medicamentos`: empty `EmptyState`; each item `div.ui-item-card` + `FormGrid` + `ActionRow` Secondary `Guardar medicamento`, Secondary `Subir`/`Bajar`, Danger `Eliminar`; Secondary `Agregar medicamento`; read mode `DetailGrid`; signed transition `ActionRow` Secondary `Reemplazar (corrección)`, Danger `Anular`, `ConfirmPanel` with `TextField` `Motivo de anulación` and Danger confirm. Keep every dispatch arg, reorder, and `LlmExplainLink`. | `Guardar borrador` or `Firmar receta` |
| `SettingsClinic` | `PageShell id="settings-page"`; `PageHeader` `Clínica`; `ErrorText`; `SectionCard` `Datos de la clínica` with fields and Primary `Guardar datos`; `SectionCard` `Logo` with optional `img.ui-asset-preview`, hint, `FileField`; `SectionCard` `Biblioteca de imágenes` with `ImageLibrary`; `SectionCard` `Exportación de datos` with retention/grace hints, explanation, Secondary `LinkAction` `/api/clinic/export` `Exportar datos (JSON)`. Keep upload/delete API bodies in named callbacks. Use the sync context to disable both upload fields while offline; the library also disables deletes. | `Guardar datos` |
| `SettingsUsers` | `PageShell id="settings-page"`; `PageHeader` `Usuarios y roles`; `ErrorText`; `SectionCard` `Personal` with `DataTable` columns `Persona`,`Correo`,`Alta`,`Estado`,`Rol`,`Acciones`; role `SelectField hide_label`, Secondary `Habilitar`, Danger `Deshabilitar`/`Quitar del equipo`; `SectionCard` `Dispositivos` with `DataTable` columns `Equipo`,`Plataforma`,`Versión`,`Última conexión`,`Estado`,`Acciones`, Danger `Revocar`. Keep role/disable/remove/revoke API closures and refresh counters. Empty tables show `EmptyState` `Sin usuarios` / `No hay usuarios para mostrar.` and `Sin dispositivos` / `No hay equipos registrados.` inside the cards. Access denied uses `Notice` Error. | None |
| `SettingsProfile` | `PageShell id="settings-page"`; `PageHeader` `Mi perfil` subtitle clinic/role; `SectionCard` `Datos del perfil` with `FormGrid` and Primary `Guardar`; `div.ui-columns` with `SectionCard` `Verificación en dos pasos` and `SectionCard` `Cambiar contraseña`; `TextField` secret/code/password; Secondary `Activar verificación`, `Confirmar activación`, `Cambiar contraseña`; Danger `Desactivar verificación`; messages `HintText`; `.ui-break` for secret/URI. Keep MFA/password calls. | `Guardar` |
| `Audit` | `PageShell id="audit-page"`; `PageHeader` `Auditoría`; access denied `Notice` Error; `FilterBar` with TextFields `Acción`,`Entidad`,`Buscar` and Primary `Filtrar`; `loaded` set after the fetch; `LoadingState`; `ErrorText`; empty `EmptyState` `Sin eventos` else `DataTable` columns `Fecha`,`Actor`,`Acción`,`Entidad`,`ID entidad`,`Metadatos`, `format_timestamp(row.at)` and `code.ui-mono` metadata. | `Filtrar` |

es-EC copy for touched auth/nav strings: `Iniciar sesión`, `Crear cuenta`, `Cerrar sesión`, `Correo`, `Contraseña`, `Nombre`, `Código de verificación`, `Verificar código`, `Bienvenido de nuevo`, `Crea tu cuenta`, `Completa todos los campos.`, `Procesando...`, `Panel`, `Configuración`, `Sin clínica asignada.`, `Rol`, `Clínica`, `Sesión iniciada`, `Cargando...`, `Redirigiendo...`, `No se pudo iniciar la sesión.`, `Completar perfil`. Keep existing Spanish copy elsewhere. Set the AuthPage empty-submit message to `Completa todos los campos.` and its no-session fallback to `No se pudo iniciar la sesión.`

- [ ] 17. Remove the old style system from `persona-sanus/assets/main.css`. Delete the selectors and IDs listed below after the last page no longer uses them. Keep `.a4-*` as the print projection; keep `#navbar` only as a semantic id on `header.ui-navbar`, never as a style hook. Replace page-ID width rules with `.ui-page`/`.ui-container`. Do not add comments. Audit:
`rg -n 'class: "(button|secondary|danger|badge|notice|error|hint|page-head|actions|form-grid|field|detail-grid|notes|patient-|template-|editor-section|settings-columns|break-anywhere|variable-row|medication-block|inline-block|preview-controls|prescription-filters|block-indicator|warning|medication-list|item-card|item-read|llm-link)' persona-sanus/src/main.rs persona-sanus/src/forms.rs` returns nothing.

  Deleted CSS selectors: `.button`, `.button.secondary`, `button.secondary`, `button.danger`, `.badge`, `.notice`, `.error`, `.hint`, `.page-head`, `.actions`, `.form-grid`, `.field`, `.field.wide`, `.detail-grid`, `.notes`, `.search-row`, `.patient-list`, `.patient-row`, `.patient-name`, `.patient-meta`, `.template-list`, `.template-row`, `.template-editor`, `.editor-section`, `.settings-columns`, `.break-anywhere`, `.variable-row`, `.medication-block`, `.inline-block`, `.preview-controls`, `.preview-scroll`, `.prescription-filters`, `.block-indicator`, `.warning`, `.medication-list`, `.item-card`, `.item-read`, `.llm-link`, `.clinic-logo`, `.asset-row`, `#echo`, `#hero`, `#links`, `#navbar`, `#patients-page`, `#templates-page`, `#settings-page`, `#prescriptions-page`, `#audit-page`, and every `#navbar ...` descendant rule. Replace the old global `button`, `input`, `textarea`, `select` component skins with `font: inherit; color: inherit;`. Keep the `body`, `p`, `a`, heading, and focus-visible base; keep the A4 block. No comments.

  Old-style audit:
`rg -n '(\.(button|badge|notice|error|hint|page-head|actions|form-grid|field|detail-grid|notes|search-row|patient-|template-|editor-section|settings-columns|break-anywhere|variable-row|medication-block|inline-block|preview-controls|preview-scroll|prescription-filters|block-indicator|warning|medication-list|item-card|item-read|llm-link|clinic-logo|asset-row)|button\.(secondary|danger)|#(navbar|echo|hero|links|patients-page|templates-page|settings-page|prescriptions-page|audit-page))' persona-sanus/assets/main.css`
returns nothing, including media-query copies.

- [ ] 18. Update source-of-truth docs and close the spec. Add `src/ui/` to the module map in `context/architecture.md` as a shared client module that owns design tokens and reusable UI primitives, and explicitly has no SQL, HTTP handlers, storage, or business rules. Add `/dev/ui` under `context/sitemap.md` §1.3 as a debug-only developer surface, not a product route. Add a short `Component library` section to `persona-sanus/README.md`: shared components live in `src/ui`, styles in `assets/main.css`, debug builds expose `/dev/ui`, release builds do not. Do not edit `context/PRD.md`, `context/Design.md`, or billing status. When all testing criteria pass, move the spec to `specs/done/15-O-ui-component-library.md`; if a divergence remains, record it in section 4 and leave it in `specs/`.

## CSS contract

The component tables define the markup structure and variants. The following are the only shared declarations needed to make those classes render consistently; page-specific one-offs are not allowed.

| Selector | Values |
|---|---|
| `.ui-page` | `padding: 48px 24px 80px;` |
| `.ui-container` | `max-width: 1200px; margin-inline: auto;` `.ui-container--narrow` 720px; `.ui-container--reading` 960px |
| `.ui-stack` | `display: flex; flex-direction: column; gap: 24px;` |
| `.ui-page-head` | flex, space-between, align start, wrap, gap 16, margin-bottom 0 |
| `.ui-page-title` | font mint 500 22px/1.22 Ink, margin 0 |
| `.ui-page-subtitle` | mint 400 14px/1.5 Mist, margin 4px 0 0 |
| `.ui-page-head-actions` | flex, wrap, align center, gap 12, margin-top 0 |
| `.ui-actions` | flex, align center, wrap, gap 12, margin-top 16 |
| `.ui-card` | background Paper, border 1px hairline, radius 24, padding 24 |
| `.ui-card-title` | mint 500 20px/1.25 Ink, margin 0 0 16px |
| `.ui-card-description` | mint 400 14px/1.5 Mist, margin -8px 0 16px |
| `.ui-button` | inline-flex center, gap 8, border 1px transparent, radius 8, padding 12px 24px, mint 500 16px/1.22, nowrap, no shadow, text-decoration none, cursor pointer |
| `.ui-button--primary` | Deep Teal bg, Paper text; hover Bright Teal bg with Ink text |
| `.ui-button--secondary` | transparent bg, Mist border/text; hover Deep Teal border/text |
| `.ui-button--ghost` | transparent, no border, Ink 400 text, padding 8px 16px; hover Deep Teal text |
| `.ui-button--danger` | transparent, Ember Ring border, Plum text; hover Blossom bg, Plum border |
| `.ui-button:disabled` | opacity .55, cursor not-allowed; hover colours inherit |
| `.ui-button--block` | width 100%, justify center |
| `.ui-field` | flex column, gap 4, min-width 0; `.ui-field--wide` grid-column 1/-1 |
| `.ui-field-label` | mint 500 14px/1.5 Ink |
| `.ui-input`, `.ui-textarea`, `.ui-select` | width 100%, Paper bg, 1px hairline border, radius 8, padding 8px 12px, mint 400 16px/1.5 Ink, min-height 44px; textarea min-height 96px, resize vertical; select cursor pointer |
| controls focus | `border-color: Deep Teal;` and the step-3 focus utility; never `outline: none` without a ring |
| controls disabled | Cream bg, Mist text, opacity 1, cursor not-allowed |
| `.ui-input[type="color"]` | width 64px, height 44px, padding 4px |
| `.ui-input[type="file"]` | 14px Mist, padding 6px 8px; `::file-selector-button` / `::-webkit-file-upload-button` Paper, 1px Mist, radius 8, 6px 12px, mint 500 14px, margin-right 12px |
| field hint/error | hint 14px Mist; error 14px Plum, 4px top margin |
| `.ui-check` | flex, align start, gap 8, 16px/1.5 Ink; checkbox 16px, accent Deep Teal, 2px top margin; description 14px Mist; `--wide` grid-column 1/-1 |
| `.ui-badge` | inline-flex center, gap 4, padding 4px 12px, radius 9999, mint 500 13px/1.22; tones from the Badge table; transparent border for Mint/Plum |
| `.ui-pill` | Mint bg, Ink, mint 500 13px/1.22, padding 4px 12px, radius 9999, white-space nowrap |
| `.ui-notice` | flex, gap 12, align start, Paper/Cream/Mint/Blossom surface by tone, 1px hairline, radius 24, padding 20, text-align left; icon 20px; body 16px/1.5; title 500 |
| `.ui-error` | flex, gap 6, align center, Plum 14px/1.5, margin 8px 0 0 |
| `.ui-hint` | Mist 14px/1.5, margin 8px 0 0 |
| `.ui-empty` | text-align center, Cream bg, 1px hairline, radius 24, padding 32; title mint 500 18px/1.25; description 14px/1.5 Mist, max 48ch, centered |
| `.ui-loading` | flex, center, gap 8, 14px Mist, padding 16px |
| `.ui-spinner` | 16px circle, 2px hairline border, 2px Deep Teal top border, `animation: ui-spin 800ms linear infinite`; reduced motion disables |
| `.ui-confirm` | Cream bg, 1px hairline, radius 24, padding 20; 16px/1.5; actions flex end wrap gap 12 margin-top 16 |
| `.ui-list` | flex column, gap 12 |
| `.ui-row` | Paper, 1px hairline, radius 24, padding 16px 20px, flex space-between center gap 16; `.ui-row-body` min-width 0; `.ui-row-title` mint 500 18px/1.25 Ink margin 0; `.ui-row-meta` 14px/1.5 Mist margin 2px 0 0; `.ui-row-actions` flex wrap gap 12 |
| `.ui-table-wrap` | overflow-x auto, Paper, 1px hairline, radius 24 |
| `.ui-table` | width 100%, border-collapse collapse; th mint 500 14px/1.4 Mist, text-align left, padding 12px 16px, border-bottom 1px hairline; td 14px/1.5 Ink, padding 12px 16px, border-bottom hairline, vertical-align top; last row no border |
| `.ui-form-grid` | grid 2 cols, gap 16; one col on `max-width: 760px`; `--one` 1fr; `--inline` 1fr 1fr auto, align end (stacks on mobile) |
| `.ui-detail-grid` | grid 2 cols, gap 16; one col on `max-width: 760px` |
| `.ui-detail-grid dt` | 14px/1.5 Mist mint 400 |
| `.ui-detail-grid dd` | 16px/1.5 Ink margin 0; `.ui-detail-wide` grid-column 1/-1 |
| `.ui-columns` | grid 2 cols gap 24 align-start; one col on `max-width: 900px` |
| `.ui-filter-bar` | grid auto-fit minmax(180px, 1fr), gap 12, align-end, margin-bottom 24 |
| `.ui-variable-row` | grid 1fr auto, gap 12, align-end, margin-bottom 12; one col below 640 |
| `.ui-link-grid` | grid auto-fit minmax(220px, 1fr), gap 16 |
| `.ui-link-card` | Card surface; flex column gap 4; Ink text, no underline; hover Deep Teal text, Cream bg; 16px 20px padding |
| `.ui-navbar` | Paper bg, 1px hairline border-bottom, `min-height: 64px`; inner max 1200, margin auto, padding 8px 24px, grid 1fr auto 1fr, gap 16, align center; account justify-self end |
| `.ui-navbar-brand` | flex center gap 12, min-width 0 |
| `.ui-navbar-links` | flex center wrap gap 4 |
| `.ui-nav-link` | Ink 16px/1.22, padding 8px 12px, radius 8, no underline; hover Deep Teal text/Cream bg; `--active` Deep Teal 500; focus ring |
| `.ui-logo` | inline-flex center gap 8, Deep Teal, no underline; hover Bright Teal |
| `.ui-logo-word` | mint 500 22px/1.2, letter-spacing -.01em |
| `.ui-logo-mark` | 22px circle, Mint bg, Deep Teal icon, grid center |
| `.ui-navbar-account` | flex center justify end gap 12, min-width 0 |
| `.ui-navbar-user` | 14px Mist, max-width 180px, ellipsis, nowrap |
| `.ui-sync` | flex center gap 6, 14px Mist; `.ui-sync-dot` 8px circle; `.ui-sync--online` Mint, `.ui-sync--offline` Mist, `.ui-sync--pending` Cream, `.ui-sync--failed` Blossom dot colours; `.ui-sync-link` 14px Plum underline |
| `.ui-hero` | width min(calc(100% - 48px), 720px), margin auto, text-align center, display flex column align center; padding 80px 0 64px |
| `.ui-display` | grenette 400 clamp(46px, 7vw, 60px)/1.05 Ink, letter-spacing -.015em, balanced, margin 0 |
| `.ui-hero-sub` | 18px/1.5 Mist, max 600, margin 16px auto 0 |
| `.ui-hero .ui-actions` | width 100%, justify center, margin-top 32 |
| `.ui-hero .ui-trust` | margin-top 24px |
| `.ui-microcopy` | 14px/1.5 Mist, margin-top 16 |
| `.ui-band` | padding 80px 24px; `--paper` Paper, `--cream` Cream, `--mint` Mint, `--blossom` Blossom |
| `.ui-section-head` | max 640, margin 0 auto 40, text-align center |
| `.ui-section-title` | grenette 400 46px/1.15 Ink, balanced, margin 0; 34px below 760 |
| `.ui-section-sub` | 18px/1.5 Mist, max 560, margin 16 auto 0 |
| `.ui-steps` | grid repeat(3, minmax(0,1fr)), gap 24, align-start; below 900: 1fr, max-width 420px, margin-inline auto |
| `.ui-steps-note` | 14px/1.5 Mist, text-align center, margin 24px 0 0 |
| `.ui-step` | text-align center, min-width 0; `.ui-step-number` 28px circle, 1px Ink, mint 500 14px/1, margin 24px auto 12px; `.ui-step-title` mint 500 20px/1.25 Ink margin 0 0 8; `.ui-step-body` mint 400 16px/1.5 Mist |
| `.ui-a4-card` | position relative, radius 24, padding 24, display flex center, no border/shadow; accent Mint/Cream/Blossom/conic token; `.ui-a4-card .ui-verification-tag` absolute top 8px right 12px |
| `.ui-a4-sheet` | width 100%, aspect-ratio 210/297, Paper bg, radius 8, padding 18px, color Ink, mint, display flex column, gap 6px, overflow hidden, text-align left |
| A4 mock internals | kicker 10px 500 Mist uppercase tracking .08em; name 18px/1.25 500 Ink; row 11px/1.4 Mist, flex space-between, gap 8, border-top hairline, padding-top 6; med 13px/1.45 500 Ink; detail 11px/1.4 Mist; qr 56px, Ink colour, margin-top auto; ring 44px conic, white check |
| `.ui-verification-tag` | relative inline-flex, Plum bg, Paper text, mint 500 13px/1.2, padding 6px 12px, radius 9999; `::after` 6px triangle below center pointing down in Plum |
| `.ui-trust` | flex center wrap gap 12, mint 500 14px/1.4 Ink, text-align center; `.ui-trust-platforms` inline-flex gap 12 Mist |
| `.ui-trust-strip` | text-align center; caption 14px Mist margin 0 0 24; logos flex center wrap gap 24px 44px; text logo mint 500 18px/1.2 Ink; img max-height 28, width auto, grayscale, opacity .9 |
| `.ui-verify-ring` | 80px circle, padding 4px, conic gradient background, radius 9999, grid center; wraps the valid seal |
| `.ui-public` | min-height 100vh |
| `.ui-public-head` | max-width 1200px; margin-inline auto; padding 24px; display flex; space-between; center; gap 16; border-bottom hairline |
| `.ui-public-main` | max-width 720px; margin 48px auto 80px; padding 0 24px |
| `.ui-verify-inline` | max-width 560px; margin-inline auto; width 100% |
| `.ui-verify-card` | Paper, 1px hairline, radius 24, padding 32, text-align center |
| `.ui-verify-title` | grenette 400 46px/1.15 Ink margin 8 0; mobile 34 |
| `.ui-verify-copy` | 16px/1.5 Mist max 44ch margin 0 auto 24 |
| `.ui-verify-seal` | 72px circle, border 1px, grid center, margin 0 auto 16; valid Mint/Deep Teal; voided Blossom/Plum; superseded Cream/Ink; not found and unknown Paper/Mist with hairline |
| `.ui-verify-details` | text-align left; margin-top 24 |
| `.ui-code-form` | grid 1fr auto gap 12 align-end; one col below 640; `.ui-code-form .ui-field` margin 0 |
| `.ui-auth-shell` | max-width 440, margin 64px auto 0 |
| `.ui-auth-head` | text-align center, margin-bottom 24, padding-top 8 |
| `.ui-auth-title` | mint 500 26px/1.22 Ink, margin 0 0 4 |
| `.ui-auth-sub` | 14px/1.5 Mist, margin 0 |
| `.ui-segmented` | flex, gap 4, padding 4, 1px hairline, radius 9999, Paper; buttons flex 1, padding 8px 16px, radius 9999; `.is-active` Cream bg Ink text 500 |
| `.ui-preview-canvas` | overflow-x auto, padding 24, Cream bg, 1px hairline, radius 24 |
| `.ui-asset-preview` | display block, max-width 240px, max-height 96px, object-fit contain, margin-bottom 12px, padding 8px, 1px hairline, radius 8, Paper bg |
| `.ui-item-card` | Paper, 1px hairline, radius 24, padding 16px 20px, margin-bottom 12 |
| `.ui-llm` | flex, wrap, align baseline, gap 8, margin-top 8; link Deep Teal; hint margin 0 |
| `.ui-library` | flex column gap 16; row flex space-between center gap 16, padding 12px 0, border-top hairline; thumb 48px square radius 8 border hairline object-fit cover, grid center Mist bg |
| `.ui-quota` | flex column gap 6, margin-bottom 16; track 4px hairline bg radius 9999, fill Deep Teal height 100%, radius 9999, width from used/limit |
| `.ui-billing-plans` | grid 2 cols gap 16; one col below 760 |
| `.ui-plan` | Paper bg, 1px hairline border, radius 24, padding 24, flex column gap 12; selected state keeps the same hairline and is marked only by the Mint `Plan actual` badge, never a coloured border or shadow |
| `.ui-plan-price` | mint 500 26px/1.22 Ink |
| `.ui-plan-interval` | mint 400 14px/1.5 Mist |
| `.ui-plan-features` | margin 0, padding-left 18px, display grid, gap 4, 14px/1.5 Mist |
| `.ui-gallery` | display flex column gap 48; PageShell owns width, margin, and inline padding |
| `.ui-gallery-head` | text-align center; h1 mint 500 26px/1.22 Ink; p 14px Mist |
| `.ui-gallery-section` | display flex column gap 16; its `h2` is mint 500 20px/1.25 Ink |
| `.ui-gallery-grid` | grid auto-fit minmax(220px, 1fr), gap 16, align start |
| `.ui-gallery-frame` | Paper, 1px hairline, radius 24, padding 20 |
| `.ui-gallery-label` | mint 500 12px/1.5 Mist uppercase, letter-spacing .08em |
| `.ui-break` | word-break break-all |
| `.ui-notes` | white-space pre-wrap |
| `.ui-mono` | font-family ui-monospace, SFMono-Regular, Menlo, monospace; font-size 12px; overflow-wrap anywhere |
| `.ui-visually-hidden` | 1px square clip-path inset(50%) absolute overflow hidden white-space nowrap |
| Responsive | nav grid -> flex wrap at 980; links full-width second row; `.ui-navbar-user` hidden below 760; page-head stacks below 640; `.ui-steps` one col below 900; form/detail grid one col below 760; `.ui-columns` one below 900; hero padding 56px 0 48px below 640; band 64px 24px below 640 |
| Motion | only spinner; `@media (prefers-reduced-motion: reduce)` disables spinner animation; no card/button transitions needed |

No new literal colour outside `:root`; no pure black; no drop shadow except `--shadow-paper` on `.a4-page` and the focus halo, which is a ring, not elevation. No gradient except `--gradient-ember-ring`. Inputs, buttons, cards, and tags use the radius scale only.

## Critical files and anchors

These are hints; reread before editing.

| File | Anchor |
|---|---|
| `persona-sanus/src/main.rs` | `enum Route` near 40; `App` 1222; `Home` 1281; `Navbar` 1376; `AuthPage` 1461; `Dashboard` 1701; `Settings` 1760; `SettingsClinic` 1803; `SettingsUsers` 2119; `SettingsProfile` 2318; `Audit` 2633; `Patients` 2757; `PatientNew` 2953; `PatientDetail` 3127; `PatientEdit` 3357; `Templates` 3566; `TemplateNew` 3679; `TemplateEdit` 3787; `TemplatePreview` 4048; `Prescriptions` 4359; `PrescriptionNew` 4594; `PrescriptionPreview` 4938; `PrescriptionDetail` 5379; label helpers at the end |
| `persona-sanus/src/forms.rs` | `A4Preview` 463; `TemplateDocumentForm` 635; `SCHEMA_VERSION` 9; `TEMPLATE_VARIABLES` 11 |
| `persona-sanus/assets/main.css` | `:root` 1-58; legacy selectors 69-874; keep `.a4-*` 647-870 |
| `persona-sanus/src/api.rs` | `Me` 41; `Patient` 87; `Prescription` 146; `RxBlockStatus` 180; `UserRow` 511; `DeviceRow` 527; `AuditRow` 579; `AssetRow` 878; `AssetQuota` 894; `ClinicSettings` 908; `VerificationResult` 973 |
| `context/sitemap.md` | route tables §1.1-1.3; role matrix §2; conventions §5 |

## Verification

Run every command from the repo root unless it changes directory. Unit tests are not added. `persona-smoke` is the integration gate: it runs `cargo fmt --check`, `cargo check`, `cargo clippy -- -D warnings`, `dx build --release --fullstack`, then starts the release server and exercises auth, session, dashboard, signup, and signout against a throwaway SQLite file.

1. Static and both targets:
```
cd persona-sanus
cargo fmt --check
cargo check
cargo clippy -- -D warnings
cargo check --no-default-features --features desktop
```
Expected: exit 0 for all. The desktop command proves the shared controls compile without the web/server cheats.

2. Release integration:
```
caveman-blocks run persona-smoke --dir persona-sanus --port 8094
```
Expected: one JSON object with `"ok": true`, every value in `checks` true, and `"port": 8094`; exit 0. This covers the release binary and auth/session `/` `/login` `/dashboard` paths.

3. Debug gallery static proof:
```
set -euo pipefail
cd persona-sanus
dx build --fullstack
tmpdir="$(mktemp -d)"
DATABASE_URL="sqlite://${tmpdir}/ui.db" PORT=8095 IP=127.0.0.1 RUSTAUTH_SECRET="$(openssl rand -hex 32)" target/dx/persona-sanus/debug/web/server >"${tmpdir}/server.log" 2>&1 &
pid=$!
trap 'kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; rm -rf "$tmpdir"' EXIT
for _ in $(seq 1 100); do curl -fsS http://127.0.0.1:8095/health >/dev/null 2>&1 && break; sleep 0.2; done
curl -fsS http://127.0.0.1:8095/health >/dev/null
curl -fsS http://127.0.0.1:8095/dev/ui >"${tmpdir}/gallery.html"
for section in tokens layout buttons forms badges feedback data brand marketing status verification library billing; do
  grep -q "id=\"gallery-${section}\"" "${tmpdir}/gallery.html" || { echo "missing ${section}"; exit 1; }
done
curl -fsS http://127.0.0.1:8095/ >"${tmpdir}/home.html"
grep -q 'Recetas claras, verificables y listas para firmar.' "${tmpdir}/home.html"
test "$(grep -oE '<(a|button)[^>]*class="[^"]*ui-button--primary[^"]*"' "${tmpdir}/home.html" | wc -l | tr -d ' ')" = "1"
curl -fsS http://127.0.0.1:8095/login >"${tmpdir}/login.html"
test "$(grep -oE '<(a|button)[^>]*class="[^"]*ui-button--primary[^"]*"' "${tmpdir}/login.html" | wc -l | tr -d ' ')" = "1"
curl -fsS http://127.0.0.1:8095/verify/NO-EXISTE >"${tmpdir}/verify.html"
grep -q 'Verificar' "${tmpdir}/verify.html"
test "$(grep -oE '<(a|button)[^>]*class="[^"]*ui-button--primary[^"]*"' "${tmpdir}/verify.html" | wc -l | tr -d ' ')" = "1"
```
Expected: no output, exit 0. This is the concrete new-behaviour check: gallery section markers exist, Home has one primary action, login has one primary action, verify has its code action.

4. Release gallery absence:
```
set -euo pipefail
cd persona-sanus
tmpdir="$(mktemp -d)"
DATABASE_URL="sqlite://${tmpdir}/ui.db" PORT=8096 IP=127.0.0.1 RUSTAUTH_SECRET="$(openssl rand -hex 32)" target/dx/persona-sanus/release/web/server >"${tmpdir}/server.log" 2>&1 &
pid=$!
trap 'kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; rm -rf "$tmpdir"' EXIT
for _ in $(seq 1 100); do curl -fsS http://127.0.0.1:8096/health >/dev/null 2>&1 && break; sleep 0.2; done
curl -fsS http://127.0.0.1:8096/health >/dev/null
curl -fsS http://127.0.0.1:8096/dev/ui >"${tmpdir}/release-gallery.html"
grep -q 'solo existe en compilaciones de desarrollo' "${tmpdir}/release-gallery.html"
! grep -q 'id="gallery-' "${tmpdir}/release-gallery.html"
```
Expected: exit 0. This proves the developer surface is stripped.

5. Source audits:
```
rg -n '\b(button|input|select|textarea) \{' persona-sanus/src/main.rs persona-sanus/src/forms.rs
rg -n 'contenteditable|execCommand|innerHTML|WYSIWYG|rich-text' persona-sanus/src
rg -n -i 'card_token|card_holder|ctoken|authorization_code|card_number|\bcvv_number\b|\bpan_number\b' persona-sanus/src/ui
rg -n 'class: "(button|secondary|danger|badge|notice|error|hint|page-head|actions|form-grid|field|detail-grid|notes|patient-|template-|editor-section|settings-columns|break-anywhere|variable-row|medication-block|inline-block|preview-controls|prescription-filters|block-indicator|warning|medication-list|item-card|item-read|llm-link)' persona-sanus/src/main.rs persona-sanus/src/forms.rs
rg -n '(\.(button|badge|notice|error|hint|page-head|actions|form-grid|field|detail-grid|notes|search-row|patient-|template-|editor-section|settings-columns|break-anywhere|variable-row|medication-block|inline-block|preview-controls|preview-scroll|prescription-filters|block-indicator|warning|medication-list|item-card|item-read|llm-link|clinic-logo|asset-row)|button\.(secondary|danger)|#(navbar|echo|hero|links|patients-page|templates-page|settings-page|prescriptions-page|audit-page))' persona-sanus/assets/main.css
```
Expected: all five return no matches (exit 1). If a match is an SVG path attribute containing `input {`, inspect; only raw RSX form controls count.

6. Manual browser gate:
Run `cd persona-sanus && dx serve --fullstack --port 8097` and open:
- `http://127.0.0.1:8097/`: centered hero, serif 46-60px headline, one filled teal CTA, secondary outlined CTA, mint pill next to wordmark, 3-column steps on wide; entering a code navigates to `/verify/<code>`; clicking primary goes to login (signed out) or prescription creation/history per role (signed in).
- `http://127.0.0.1:8097/login`: sign in/sign up switch keyboard usable; all labels visible; empty fields show `Completa todos los campos.`; one filled submit; after signup, dashboard loads.
- `http://127.0.0.1:8097/dev/ui`: all sections; primary/secondary/ghost/danger; badge tones; field error; `$5,00 USD` / `$50,00 USD`; no card/token/PAN/CVV values; trust strip note; four verification result states; both RX states; no console errors.
- `http://127.0.0.1:8097/verify/NO-EXISTE`: `Receta no encontrada`; no patient, cédula, diagnosis, or medication labels. Use a valid code from a signed prescription to see `Receta auténtica` with only doctor, date, number.
- Authenticated: `/patients` search/empty/create/detail/edit; `/templates` edit/publish/preview; `/prescriptions` filters/preview/sign; `/settings`, `/settings/clinic`, `/settings/users`, `/settings/profile`, `/audit` loaded as an administrator. Confirm focused controls show the visible ring, rows stack below 640, the three steps stack below 900, and only tables/preview scroll horizontally. Also Tab through auth, gallery, patient form, template form, and the billing demo for visible focus.

7. Close the spec only when 1-6 pass, the source audits are clean, and all objectives are checked. Move `specs/15-O-ui-component-library.md` to `specs/done/`.

## Assumptions and contingencies

- Scope is fixed: full v1 adoption plus presentational v1.1 billing. The copy above is fixed; do not substitute synonyms. Do not add a billing route, a plan picker that calls an endpoint, a card form, or a `sync_locked` gate. The billing gallery is the only billing surface.
- The repo has no grenette/mint webfonts. Keep the existing fallback stacks; do not fetch or add fonts. If a local licensed font later lands, point `--font-grenette`/`--font-mint` at it; no component markup changes.
- No partner-clinic logos exist, so Home must not show `TrustLogoStrip`. If real marks arrive, pass them only from a data source; never bake branded names into marketing markup.
- Dioxus 0.7-specific fallbacks: if `NavigationTarget` as a component prop does not accept `Route` directly, change `ButtonLink.to`/`NavLink.to` to `Route` and keep the internal `Link`; if `Option<Element>` props break the props macro, make `badge`/children required and pass `rsx! {}` at call sites; if `FormEvent` cannot be a component callback, keep the file `<input>` in each page and share only `Field` + `.ui-file-input` styling. None of these changes the visual contract or the one-primary rule.
- If the debug server path is not `target/dx/persona-sanus/debug/web/server`, use `dx serve --fullstack --port 8095` for step 3 and keep the same curl assertions. If port 8095/8096/8097 is busy, change only the port and the matching URL.
- If browser verification finds a control not in the Design.md list but required by the existing page, use the nearest existing primitive (`Badge`, `Card`, `Button Secondary`) and do not add a new component file. The component set is closed after step 9.


