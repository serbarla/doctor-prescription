# Spec 15 — Phase O: UI tokens, actions, and formatting

- Source: `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` steps 2–3; `context/Design.md` §§Components, Tokens, Surfaces, Elevation, Layout, Agent Prompt Guide; `context/architecture.md` §3.
- Depends on: 01-A–12-L and 14-N.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Result carried forward: `assets/main.css` has a valid token scale, border-box reset, and shared layout/text utilities; `persona-sanus/src/ui/` exposes `Icon`, `Button`, `ButtonLink`, `LinkAction`, `NavLink`, and every moved format helper; `main.rs` imports them explicitly and no longer defines them; both web and desktop targets compile.

## Before you start

Unit testing is not required; avoid it. Prioritize end-to-end and integration testing only.

Read the nearest `AGENTS.md` (target workspace, then parents up to repo root) before executing.

If reality diverges from this spec: STOP, append a divergence (section 4), and NEVER improvise outside the objectives.

This is the foundation spec for the split UI library work. Specs `16-P` through `25-Y` consume its exports. Spec `25-Y` removes the legacy CSS and records the docs. The open objective 4 in `13-M` (signed desktop installers) is non-blocking: no UI surface, route, or contract here depends on it.

## 1. Objectives

*The ordered work to implement — one item per step. Check `[x]` only after the step is implemented and verified.*

*Item format:* `<verb phrase>` — requirements: `<what must change / constraints>`; commands: `<exact command>`.

- [x] 1. Establish tokens and layout rhythm — requirements: in `persona-sanus/assets/main.css`, keep every existing token name. Add `--leading-caption: 1.5`, `--leading-body-sm: 1.5`, `--leading-body: 1.5`, `--leading-body-lg: 1.5`, `--leading-subheading: 1.25`, `--leading-heading-sm: 1.22`, `--leading-heading: 1.22`, `--leading-heading-lg: 1.15`, and `--leading-display: 1.05`. Keep `--font-weight-regular` and `--font-weight-medium`. Add `--content-display: 720px`, `--content-copy: 560px`, `--section-gap: 80px`, `--section-gap-compact: 64px`, `--element-gap: 8px`, `--element-gap-lg: 16px`, `--radius-pill: 9999px`, `--color-mint-halo: rgba(185, 255, 232, 0.55)`, `--shadow-paper: 0 1px 2px rgba(6, 29, 41, 0.06)`, and `--navbar-height: 64px`. Replace the invalid declarations `--section-gap: 64-80px` and `--element-gap: 8-16px`; never leave an invalid length. Add `*, *::before, *::after { box-sizing: border-box; }`. Add `.ui-page`, `.ui-container`, `.ui-container--narrow`, `.ui-container--reading`, `.ui-stack`, `.ui-visually-hidden`, `.ui-mono`, `.ui-break`, `.ui-notes`, `.ui-text-caption`, `.ui-text-body-sm`, `.ui-text-body`, `.ui-text-body-lg`, and `.ui-text-subheading` exactly as section 9 contracts. No pure black, no new stylesheet, and no CSS framework; commands: `cd persona-sanus && cargo check`; `rg -n -- '--section-gap:\s*64-80px|--element-gap:\s*8-16px' persona-sanus/assets/main.css`.
- [x] 2. Add icon and action primitives — requirements: create `persona-sanus/src/ui/mod.rs`, `icon.rs`, and `button.rs`, and add `mod ui;` to `persona-sanus/src/main.rs`. Implement `Icon`, `IconName`, `Button`, `ButtonVariant`, `ButtonLink`, `LinkAction`, and `NavLink` exactly as section 7 contracts. Use inline SVG path data only; `Cross` is the one filled icon. Never inline style and never JavaScript. Add the shared focus utility to `assets/main.css`; commands: `cd persona-sanus && cargo check`; `rg -n 'style:' persona-sanus/src/ui/button.rs persona-sanus/src/ui/icon.rs`.
- [x] 3. Extract formatting helpers — requirements: add `persona-sanus/src/ui/format.rs` and move the thirteen helpers named in section 7 from `main.rs` verbatim, changing only visibility so `ui/mod.rs` can re-export them. Replace their definitions and call sites in `main.rs` with one explicit `use crate::ui::{...}` import list that names every moved helper `main.rs` uses. Keep `valid_birth_date`, `llm_explain_url`, `new_prescription_item_form`, `probe_sync_status`, the desktop count helpers, and every dispatch or submit function in `main.rs`; commands: `cd persona-sanus && cargo check`; `rg -n '^fn (format_date|format_timestamp|last_seen_label|role_label|status_label|platform_label|device_name_label|sex_label|optional_date|prescription_status_label|template_status_label|version_status_label|template_owner_label)' persona-sanus/src/main.rs`.

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No new crate, CSS framework, external stylesheet, webfont, CDN asset, icon package, or JavaScript bridge.
- No rich text, WYSIWYG, `contenteditable`, formatting toolbar, `execCommand`, HTML payload, or editor dependency. All text entry is plain controlled inputs, textareas, selects, and checks.
- No new product route, API handler, server function, Diesel model, table, migration, entitlement rule, or billing persistence.
- No change to authorization, session, device registration, sync, numbering, signing, PDF, QR, verification limits, or business validation.
- No page markup adoption in this spec. Do not restyle or restructure `Home`, `Navbar`, `AuthPage`, `Verify`, or any settings or clinical page.
- No inline `style` attribute and no JavaScript dispatch in the new primitives. No `outline: none` without the focus ring.
- No pure black and no new literal colour outside `:root`. No drop shadow except in the focus halo and `--shadow-paper`; buttons and cards get none.
- No unit tests, and no comments in Rust, CSS, or RSX.
- No change to `.a4-*` print geometry, pagination, or page sizes.
- No second styling system. Extend `persona-sanus/assets/main.css` only.
- No patient name, cédula, diagnosis, medication, or photo in any sample used by this spec.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [x] `cd persona-sanus && cargo check` exits 0.
- [x] `cd persona-sanus && cargo check --no-default-features --features desktop` exits 0.
- [x] objective 1: `rg -n -- '--section-gap:\s*64-80px|--element-gap:\s*8-16px' persona-sanus/assets/main.css` returns no matches.
- [x] objective 1: `rg -n -- '--leading-(caption|body-sm|body|body-lg|subheading|heading-sm|heading|heading-lg|display):' persona-sanus/assets/main.css` returns 9 matches.
- [x] objective 1: `rg -n -- '--content-display:\s*720px|--content-copy:\s*560px|--section-gap:\s*80px|--section-gap-compact:\s*64px|--element-gap:\s*8px|--element-gap-lg:\s*16px|--radius-pill:\s*9999px|--color-mint-halo|--shadow-paper|--navbar-height:\s*64px' persona-sanus/assets/main.css` returns 10 matches.
- [x] objective 1: `rg -n 'box-sizing: border-box|^\.ui-(page|container|stack|visually-hidden|mono|break|notes|text-caption|text-body-sm|text-body|text-body-lg|text-subheading)' persona-sanus/assets/main.css` returns the reset and every utility.
- [x] objective 1: no pure black token appears in `persona-sanus/assets/main.css`.
- [x] objective 2: `rg -n '^mod ui;' persona-sanus/src/main.rs` returns 1 match.
- [x] objective 2: `rg -n 'style:' persona-sanus/src/ui/button.rs persona-sanus/src/ui/icon.rs` returns no matches.
- [x] objective 3: `rg -n '^fn (format_date|format_timestamp|last_seen_label|role_label|status_label|platform_label|device_name_label|sex_label|optional_date|prescription_status_label|template_status_label|version_status_label|template_owner_label)' persona-sanus/src/main.rs` returns no matches.
- [x] objective 3: `rg -n 'fn (format_date|format_timestamp|last_seen_label|role_label|status_label|platform_label|device_name_label|sex_label|optional_date|prescription_status_label|template_status_label|version_status_label|template_owner_label)' persona-sanus/src/ui/format.rs` returns 13 matches.
- [x] objective 3: `rg -n 'fn (valid_birth_date|llm_explain_url|new_prescription_item_form)' persona-sanus/src/main.rs` still returns 3 matches.
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

`persona-sanus/src/main.rs` is 6,177 lines and owns every page inline. Only two reusable render components exist: `forms::A4Preview` at `forms.rs:463` and `forms::TemplateDocumentForm` at `forms.rs:635`. `assets/main.css` has the Design.md tokens, but most components are one-off classes (`.button`, `.badge`, `.notice`, `.page-head`, `.editor-section`) or absent. Two spacing tokens are invalid CSS lengths (`64-80px`, `8-16px`).

This spec creates the shared vocabulary only. Full page adoption starts at spec `20-T`; the final cleanup at `25-Y` depends on every page having moved.

The split and ownership:

| Plan steps | Spec | Result |
|---|---|---|
| 2–3 | `15-O` | Tokens, layout/text utilities, icon and action primitives, format helpers |
| 4–5 | `16-P` | Form controls, feedback, cards, grids, tables, rows, bands |
| 6 | `17-Q` | Brand lockup, trust rows, hero and marketing blocks |
| 7–8 | `18-R` | Status, sync, RX, LLM, verification and image library; presentational billing |
| 9 | `19-S` | Debug-only `/dev/ui` gallery and route gate |
| 10–12 | `20-T` | Navbar, auth, Home, public verification |
| 13 | `21-U` | Dashboard, Settings hub, Patients and patient forms |
| 14 | `22-V` | Templates pages and `TemplateDocumentForm` |
| 15 | `23-W` | Prescriptions list, new, preview, detail |
| 16 | `24-X` | Settings clinic, users, profile, Audit |
| 17–18 | `25-Y` | Remove legacy CSS, update docs, close the split |

Execution order is `15-O` → `16-P` → `17-Q` → `18-R` → `19-S` → `20-T`, then the page specs in the order above. Run `cargo check` after every objective.

## 6. Reuse, do not rebuild

| Existing symbol | Decision |
|---|---|
| `forms::A4Preview` | Keep and reuse for the true-scale document projection. This spec does not touch it. |
| `forms::TemplateDocumentForm` | Keep as the template editor. Spec `22-V` restyles its internals; this spec does not recreate its document state or persistence. |
| `Navbar` | Keep the component, router current-route effect, session/sync contexts, role guards, and sign-out/sign-in handlers. Spec `20-T` replaces only its markup and styles. |
| `Route`, `App`, page components | Keep. Add only `DevUi` in `19-S`; do not add `/verify` or `/settings/billing`. |
| `forms::paginate_document`, `resolve_text`, document warnings | Keep unchanged. |
| `SyncStatus`, `probe_sync_status`, local count helpers | `SyncStatus` moves into `src/ui/domain.rs` in `18-R`; `probe_sync_status` and the `#[cfg(feature = "desktop")]` counts stay in `main.rs`. Reuse them from every page. |
| `format_date`, `format_timestamp`, status/role/platform/sex label helpers | Move verbatim here. Keep `valid_birth_date`, `llm_explain_url`, `new_prescription_item_form`, and all dispatch/submit functions in `main.rs`. |
| `.a4-*` CSS and A4 geometry | Keep. Only `25-Y` may swap the literal `#111111` for Ink and reduce the page shadow to `--shadow-paper`; never change mm sizes, pagination, or QR layout. |

New components live only in `persona-sanus/src/ui/`. There is no second styling system. Extend `assets/main.css`; do not add a CSS framework or another stylesheet.

## 7. Component surface

All symbols are re-exported from `src/ui/mod.rs`; `main.rs` and `forms.rs` import them explicitly. Every interactive control renders with `aria-*` where stated, a visible `:focus-visible` ring, and no drop shadow. `Button` and `LinkAction` both use `.ui-button`, so their geometry is identical. Fields marked `#[props(default)]` are optional; unmarked fields are required.

### Icons and actions

- `Button`: `variant: ButtonVariant = Primary`, `button_type: String = "button"`, `#[props(default)] disabled: bool`, `#[props(default)] loading: bool`, `#[props(default)] extra_class: String`, `onclick: EventHandler<MouseEvent> = no-op`, `children`. Renders `button.ui-button.ui-button--{variant}` with `type=button_type`, `disabled=disabled || loading`, and `aria-busy="true"` while loading. No shadow.
- `ButtonLink`: `#[props(into)] to: NavigationTarget`, `variant: ButtonVariant = Primary`, `active_class: Option<String>`, `#[props(default)] extra_class: String`, `children`. Renders Dioxus `Link` with `.ui-button` classes and passes the active class through. Used only for in-app navigation.
- `LinkAction`: `href: String`, `variant: ButtonVariant = Primary`, `target: Option<String>`, `rel: Option<String>`, `#[props(default)] extra_class: String`, `children`. Renders `<a class="ui-button ...">`; omits `target` and `rel` when `None`. Used for PDF, print, share, export, and external LLM links.
- `NavLink`: `#[props(into)] to: NavigationTarget`, `label: String`. Renders `Link.ui-nav-link` with `active_class: "ui-nav-link--active"`.
- `Icon`: `name: IconName`, `size: u16 = 20`, `#[props(default)] extra_class: String`. Renders a 24-unit `svg` with RSX attributes `view_box="0 0 24 24"`, `aria_hidden="true"`, `focusable="false"`, `stroke="currentColor"`, `stroke_width="1.5"`, `stroke_linecap="round"`, and `stroke_linejoin="round"`. The DOM output uses `viewBox` and `stroke-width`.
- `ButtonVariant`: `Primary`, `Secondary`, `Ghost`, `Danger`. Default `Primary`.
- `IconName`: `Cross`, `Shield`, `Command`, `Windows`, `Globe`, `Check`, `Close`, `Alert`, `Info`, `Qr`, `ArrowRight`, `Lock`, `Image`. The filled `Cross` uses `fill="currentColor"` and no stroke; every other icon is a line icon.

The focus utility is `outline: 1px solid var(--color-deep-teal); outline-offset: 2px; box-shadow: 0 0 0 3px var(--color-mint-halo);`. Apply it to every `:focus-visible` state on buttons, links, inputs, textareas, selects, checkboxes, and file controls. Never use `outline: none` without a visible replacement ring.

### Formatting helpers

Move verbatim from `main.rs`: `format_date`, `format_timestamp`, `last_seen_label`, `role_label`, `status_label`, `platform_label`, `device_name_label`, `sex_label`, `optional_date`, `prescription_status_label`, `template_status_label`, `version_status_label`, `template_owner_label`.

Keep in `main.rs`: `valid_birth_date`, `llm_explain_url`, `new_prescription_item_form`, every dispatch function, and every submit function.

## 8. Page adoption

None. This spec does not change page markup. It adds `mod ui;` and replaces helper call sites in `main.rs` with the explicit import list only. Page adoption belongs to `20-T` through `24-X`.

## 9. CSS contract

The component contracts define markup structure and variants. These are the only shared declarations this spec adds beyond the token layer; page-specific one-offs are not allowed.

| Selector | Values |
|---|---|
| `.ui-page` | `padding: 48px 24px 80px;` |
| `.ui-container` | `max-width: 1200px; margin-inline: auto;` |
| `.ui-container--narrow` | `max-width: 720px;` |
| `.ui-container--reading` | `max-width: 960px;` |
| `.ui-stack` | `display: flex; flex-direction: column; gap: 24px;` |
| `.ui-button` | inline-flex center, gap 8, border 1px transparent, radius 8, padding 12px 24px, mint 500 16px/1.22, nowrap, no shadow, text-decoration none, cursor pointer |
| `.ui-button--primary` | Deep Teal background, Paper text; hover Bright Teal background with Ink text |
| `.ui-button--secondary` | transparent background, Mist border and text; hover Deep Teal border and text |
| `.ui-button--ghost` | transparent, no border, Ink 400 text, padding 8px 16px; hover Deep Teal text |
| `.ui-button--danger` | transparent, Ember Ring border, Plum text; hover Blossom background, Plum border |
| `.ui-button:disabled` | opacity .55, cursor not-allowed; hover colours inherit |
| `.ui-button--block` | width 100%, justify center |
| `.ui-nav-link` | Ink 16px/1.22, padding 8px 12px, radius 8, no underline; hover Deep Teal text with Cream background; `--active` Deep Teal 500; focus ring |
| `.ui-visually-hidden` | `position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden; clip: rect(0, 0, 0, 0); clip-path: inset(50%); white-space: nowrap; border: 0;` |
| `.ui-mono` | `font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 12px; overflow-wrap: anywhere;` |
| `.ui-break` | `word-break: break-all;` |
| `.ui-notes` | `white-space: pre-wrap;` |
| `.ui-text-caption` | font-size and line-height from `--text-caption` and `--leading-caption`; weight inherited |
| `.ui-text-body-sm` | font-size and line-height from `--text-body-sm` and `--leading-body-sm`; weight inherited |
| `.ui-text-body` | font-size and line-height from `--text-body` and `--leading-body`; weight inherited |
| `.ui-text-body-lg` | font-size and line-height from `--text-body-lg` and `--leading-body-lg`; weight inherited |
| `.ui-text-subheading` | font-size and line-height from `--text-subheading` and `--leading-subheading`; weight inherited |

No new literal colour outside `:root`; no pure black; no drop shadow except `--shadow-paper` on `.a4-page` and the focus halo. Inputs, buttons, cards, and tags use the radius scale only.

## 10. Critical files and anchors

These are hints; reread before editing.

| File | Anchor |
|---|---|
| `persona-sanus/src/main.rs` | `enum Route` near 40; `App` 1222; `Home` 1281; `Navbar` 1376; `AuthPage` 1461; `Dashboard` 1701; label helpers at the end |
| `persona-sanus/assets/main.css` | `:root` 1-58; legacy selectors 69-874; keep `.a4-*` 647-870 |
| `context/Design.md` | §§Components, Tokens, Surfaces, Elevation, Layout, Agent Prompt Guide |
| `context/architecture.md` | §3 client modules |
| `context/sitemap.md` | route tables §1.1-1.3; role matrix §2; conventions §5 |

## 11. Assumptions and contingencies

- Scope is fixed: full v1 adoption plus presentational v1.1 billing. Do not add a billing route, a plan picker that calls an endpoint, a card form, or a `sync_locked` gate. The billing gallery in `19-S` is the only billing surface.
- The repo has no grenette or mint webfonts. Keep the existing fallback stacks; do not fetch or add fonts. If a local licensed font later lands, point `--font-grenette` or `--font-mint` at it; no component markup changes.
- No partner-clinic logos exist, so Home must not show `TrustLogoStrip`. If real marks arrive, pass them only from a data source; never bake branded names into marketing markup.
- Dioxus 0.7 fallbacks: if `NavigationTarget` as a component prop does not accept `Route` directly, change `ButtonLink.to` and `NavLink.to` to `Route` and keep the internal `Link`; if `Option<Element>` props break the props macro, make `badge` or children required and pass `rsx! {}` at call sites; if `FormEvent` cannot be a component callback, keep the file `<input>` in each page and share only `Field` and `.ui-file-input` styling. None of these changes the visual contract or the one-primary rule.
- If the debug server path is not `target/dx/persona-sanus/debug/web/server`, use `dx serve --fullstack --port 8097` for the manual gate and keep the same curl assertions. If port 8095, 8096, or 8097 is busy, change only the port and the matching URL.
- If browser verification finds a control not in the Design.md list but required by the existing page, use the nearest existing primitive (`Badge`, `Card`, `Button Secondary`) and do not add a new component file. The component set is closed after `19-S`.

---

*Housekeeping: once a spec is fully done with no open divergences, move its
file to `specs/done/` (same file name, e.g.
`specs/done/15-O-ui-tokens-actions-and-formatting.md`).
Specs with open divergences stay in `specs/` next to `TEMPLATE.md` until the
tracking file is cleared.*
