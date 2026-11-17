# Spec 20 — Phase T: Shell, auth, home, and verification adoption

- Source: `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` steps 10–12; `context/Design.md` §§Components, Tokens, Surfaces, Elevation, Layout, Agent Prompt Guide; `context/sitemap.md` §§1.1–1.3, §2, §5; `context/PRD.md` FR-043, FR-050, D-06, D-21; `context/architecture.md` §3.
- Depends on: 15-O, 16-P, 17-Q, 18-R, 19-S.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Result carried forward: the navbar, `/login`, the Home landing, and `/verify/:code` render through the UI layer; the authenticated nav has exactly five guarded destinations and no Home text link; auth is es-EC copy with a keyboard-usable sign-in/sign-up switch and an MFA-only state; Home has one filled primary CTA and no partner strip; public verification shows only doctor, date, and prescription number and always offers another code.

## Before you start

Unit testing is not required; avoid it. Prioritize end-to-end and integration testing only.

Read the nearest `AGENTS.md` (target workspace, then parents up to repo root) before executing.

If reality diverges from this spec: STOP, append a divergence (section 4), and NEVER improvise outside the objectives.

This spec adopts existing pages. It does not change session, sign-in, MFA, verification, or role logic. Every signal, effect, API call, guard, route, and dispatch path stays, except the UI-only state named here (`loaded`, and the code form's local value). `allocating` is added later, in `23-W`.

## 1. Objectives

*The ordered work to implement — one item per step. Check `[x]` only after the step is implemented and verified.*

*Item format:* `<verb phrase>` — requirements: `<what must change / constraints>`; commands: `<exact command>`.

- [ ] Adopt the navbar and auth page — requirements: replace the `#navbar` internals with `header.ui-navbar` > `div.ui-navbar-inner`; brand group uses `LogoLockup` plus `PillBadge`; centre is `nav.ui-navbar-links` of `NavLink` with exactly five guarded destinations `Panel`, `Pacientes`, `Recetas`, `Plantillas`, `Configuración`; remove `Mi perfil`, `Usuarios`, and `Auditoría` from the bar; remove the old text `Home` link; right group uses `SyncStatusIndicator`, the optional `{failed} con error` link to the filtered prescriptions route, `.ui-navbar-user` email, and a Ghost `Button` `Cerrar sesión`; signed-out right uses a Ghost `ButtonLink` `Iniciar sesión`. Keep the router current-route `use_effect`, all contexts, every guard, and the sign-out handler. Rewrite `AuthPage` exactly as section 7 and the es-EC copy table, using a `PageShell id="auth-page"` shell and the no-native-submit rule; commands: `cd persona-sanus && cargo check`; `rg -n 'Welcome back|Create your account|Fill in every field\.|Please wait\.\.\.|The auth server did not return|Signed in|Sign in|Sign out|Sign up' persona-sanus/src/main.rs`; `rg -n 'NavLink \\{' persona-sanus/src/main.rs`.
- [ ] Rebuild Home as the landing — requirements: root `<div id="home-page">` inside the navbar layout, not `PageShell`, so bands stay full-bleed. Resolve the CTA before RSX in this order: no session -> `Route::AuthPage {}` label `Emitir receta`; session plus profile with `can_create_prescriptions()` -> `Route::PrescriptionNew { patient_id: None, template_id: None }` label `Emitir receta`; session plus profile with a role but no create permission -> `Route::Prescriptions { status: None }` label `Ver recetas`; session with no profile or no role -> `Route::SettingsProfile {}` label `Completar perfil`. Render the `Hero`, steps, and verification band with the fixed copy and links in section 7. Do not render `TrustLogoStrip`; commands: `cd persona-sanus && cargo check`; `rg -n 'TrustLogoStrip' persona-sanus/src/main.rs`; `rg -n 'Recetas claras, verificables y listas para firmar\.' persona-sanus/src/main.rs`.
- [ ] Adopt Verify to the public shell — requirements: root `div#verify-page.ui-public`; `.ui-public-head` holds `LogoLockup` and `PillBadge`; `.ui-public-main.ui-stack` holds a conditional region with only `LoadingState` `Verificando receta…`, Error `Notice` for fetch failure, and `VerificationResultCard`, then always a `SectionCard` `Verificar otra receta` with `VerificationCodeForm` default Primary and `TrustSecurityRow`. Keep the existing `requested` fetch signal. At the start of each fetch set `loaded` false, clear `message`, and clear the old result. On form submit, trim and uppercase, set `requested`, and call `navigator.replace(Route::Verify { code })`. Preserve the no-auth route and never render a patient name, cédula, diagnosis, or medication field; commands: `cd persona-sanus && cargo check`; `rg -n 'Verificando receta…|Verificar otra receta' persona-sanus/src/main.rs`; `rg -n 'Receta no encontrada' persona-sanus/src/ui/domain.rs`; `rg -n 'patient_name|patient_id|patient_birth|patient_sex|cedula|cédula|diagnosis' persona-sanus/src/ui/domain.rs`.

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No new crate, CSS framework, external stylesheet, webfont, CDN asset, icon package, or JavaScript bridge.
- No rich text, WYSIWYG, `contenteditable`, formatting toolbar, `execCommand`, HTML payload, or editor dependency. All text entry is plain controlled inputs, textareas, selects, and checks.
- No new product route, API handler, server function, Diesel model, table, migration, entitlement rule, or billing persistence. Do not add `/verify` as a new route; it exists. `/settings/billing` stays reserved with no surface.
- No change to authorization, session, device registration, sync, numbering, signing, PDF, QR, verification limits, or business validation. The router effect, role guards, fetch arguments, and sign-out/sign-in closures stay.
- No native `form` submit in `AuthPage`. Use the existing click closures; do not add `onsubmit`, `action`, or a submit button type.
- No extra navbar destination. The authenticated bar has exactly five destinations: Panel, Pacientes, Recetas, Plantillas, Configuración. Remove Mi perfil, Usuarios, and Auditoría from the bar; do not add a Home text link.
- No `TrustLogoStrip` on Home. No fake partner logos, no endorsement claim, and no branded clinic names in marketing markup.
- No patient name, cédula, diagnosis, medication, or photo on `/verify/:code` or in Home marketing copy. The verification card receives only `api::VerificationResult` and renders only doctor, date, and number.
- No more than one filled primary action in a viewport. Home has exactly one, Auth has one, and Verify has one. Secondary and Ghost actions are not filled primary.
- No pure black, drop shadow on UI cards or buttons, chromatic card or image border, or bold grenette headline.
- No second stylesheet and no CSS framework; extend `assets/main.css`.
- No unit tests and no comments in Rust, CSS, or RSX.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [ ] objective 1: `cd persona-sanus && cargo check` exits 0.
- [ ] objective 1: `cd persona-sanus && cargo check --no-default-features --features desktop` exits 0.
- [ ] objective 1: `rg -n 'Welcome back|Create your account|Fill in every field\.|Please wait\.\.\.|The auth server did not return|Signed in|Sign out|Sign up' persona-sanus/src/main.rs` returns no matches.
- [ ] objective 1: `rg -n 'NavLink \\{' persona-sanus/src/main.rs` returns 5 matches.
- [ ] objective 2: `rg -n 'TrustLogoStrip' persona-sanus/src/main.rs` returns no matches.
- [ ] objective 2: `rg -n 'Recetas claras, verificables y listas para firmar\.' persona-sanus/src/main.rs` returns 1 match.
- [ ] objective 3: `rg -n 'Verificando receta…|Verificar otra receta' persona-sanus/src/main.rs` returns the loading and repeat strings.
- [ ] objective 3: `rg -n 'Receta no encontrada' persona-sanus/src/ui/domain.rs` returns the not-found string.
- [ ] debug static proof:
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
! grep -qiE 'paciente|c[eé]dula|diagn[oó]stico|medicamento' "${tmpdir}/verify.html"
```
- [ ] Browser pass on `http://127.0.0.1:8097/` after `cd persona-sanus && dx serve --fullstack --port 8097`: centered hero, serif headline, one filled primary CTA, one outlined secondary `Verificar una receta` that scrolls to `#verificar`; three steps on wide and one column below 900; no trust logo strip; entering a code in the verification band navigates to `/verify/<code>`.
- [ ] Browser pass on `http://127.0.0.1:8097/login`: the two modes switch by keyboard, labels are visible, empty submit shows `Completa todos los campos.`, one filled submit exists, and the signed-in card shows `Sesión iniciada`.
- [ ] MFA browser pass: with an MFA ticket present, the page shows only `Código de verificación`, a Primary `Verificar código`, and a Secondary `Volver`; `Volver` clears the ticket and error; no email, password, or name field is visible.
- [ ] Browser pass on `http://127.0.0.1:8097/verify/NO-EXISTE`: `Receta no encontrada`; no patient, cédula, diagnosis, or medication label; the repeat form exists. With a valid signed code, the card shows only `Médico`, `Fecha de emisión`, and `No. de receta`.
- [ ] Home CTA branch pass: signed out -> login; can create -> new prescription with both ids absent; role without create -> prescription history; no profile or no role -> profile completion. No branch falls through.
- [ ] Login state pass: sign in and sign up switch, the incomplete-field error, and post-signup dashboard load.
- [ ] Focus pass: Tab through auth, Home, and `/verify`; every focused control shows the visible ring and no trapped focus.
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

The shared components exist after `19-S`, but the four release surfaces a visitor sees first still render legacy markup. This spec is the first page adoption and the reference implementation for `21-U` through `24-X`: it shows how to keep every guard and handler while replacing only the RSX shell and visual classes.

`Navbar` currently renders a `div#navbar` with a text Home link, Dashboard/Patients/Prescriptions/Templates/Settings plus Mi perfil/Usuarios/Auditoría, a raw sync string, the session email, and an English `Sign out`/`Sign in` button. `AuthPage` renders `div#echo`, English headings and placeholders, no labels, and a single button. `Home` is a plain hero with `PersonaSanus`, one English subtitle, and two text links. `Verify` renders `div#verify-page` with a raw `h1`, a detail grid, and a plain `button` labelled `Verificar`; it already has the `requested` signal but the effect does not reset `loaded`, `message`, or `result` at the start.

## 6. Reuse, do not rebuild

| Existing symbol | Decision |
|---|---|
| `forms::A4Preview` | Keep. Only `22-V` and `23-W` restyle its surrounding canvas. |
| `forms::TemplateDocumentForm` | Keep. `22-V` adopts the controls; this spec does not touch it. |
| `Navbar` | Keep the component, `use_effect` router probe, session/sync contexts, role guard methods, and sign-out/sign-in handlers. Replace markup and styles only. |
| `Route`, `App` | Keep. Do not add `/verify` or `/settings/billing`. `DevUi` already exists from `19-S`. |
| `page components` | Keep all data effects and dispatch. Add only UI state named in the objectives. |
| `forms::paginate_document`, `resolve_text`, document warnings | Keep unchanged. |
| `SyncStatus`, probe and counts | Consume the `18-R` moved type and `SyncStatusIndicator`. Do not move `probe_sync_status`. |
| `format_date`, label helpers | Consume the `15-O` exports. Do not duplicate. |
| Home `h1#hero`, `#links`, auth `#echo` CSS and IDs | Replace the markup and uses; legacy CSS removal is `25-Y`. Keep `#home-page` and `#auth-page` as the new roots. |
| `.a4-*` CSS and A4 geometry | Keep unchanged. |

## 7. Component surface

All page copy below is fixed. Keep existing Spanish copy elsewhere. Technical status strings, API values, and role names stay unchanged.

### Navbar

- Root: `header#navbar.ui-navbar` > `div.ui-navbar-inner`.
- Brand group: `div.ui-navbar-brand` with `LogoLockup` then `PillBadge` (default label `Receta verificable`).
- Centre: `nav.ui-navbar-links` with exactly five `NavLink`s and the existing guards:
  - `Panel` -> `Route::Dashboard {}` when a session is present.
  - `Pacientes` -> `Route::Patients {}` when `profile.can_read_patients()`.
  - `Recetas` -> `Route::Prescriptions { status: None }` when `profile.can_read_prescriptions()`.
  - `Plantillas` -> `Route::Templates {}` when `profile.can_manage_templates()`.
  - `Configuración` -> `Route::Settings {}` when a session is present.
- Remove the `Mi perfil`, `Usuarios`, and `Auditoría` links from the bar. Remove the old text `Home` link; the logo is the Home link from `LogoLockup`.
- Right group: `div.ui-navbar-account` with `SyncStatusIndicator { status }`; when `status.failed > 0`, a Link `class="ui-sync-link"` to `Route::Prescriptions { status: Some("failed".to_owned()) }` labelled `{status.failed} con error`; when signed in, `span.ui-navbar-user` with the email and a Ghost `Button` `Cerrar sesión` carrying the existing onclick closure. Signed out shows only a Ghost `ButtonLink` `Iniciar sesión` to `Route::AuthPage {}`.
- Keep `Outlet::<Route> {}` outside the header. Keep the `use_effect` that reads `router.current()` and calls `probe_sync_status`.

### AuthPage

- Signed-in branch: `PageShell id="auth-page"` > `PageHeader title="Sesión iniciada"` > `Card` > `DetailGrid` with `DetailItem label="Nombre"` and `DetailItem label="Correo"` from the session; then a Ghost `Button` `Cerrar sesión` with the existing sign-out closure.
- Signed-out branch: `PageShell id="auth-page"` > `div.ui-auth-shell` (max 440) > `div.ui-auth-head` with `h1.ui-auth-title` (`Bienvenido de nuevo` when `!sign_up_mode`, `Crea tu cuenta` otherwise). Then `.ui-segmented` with two Ghost `Button`s: `Iniciar sesión` active when `!sign_up_mode`, `Crear cuenta` active when `sign_up_mode`; clicking clears `error` and sets the mode. Then `Card` > `FormGrid columns: FormColumns::One`.
- Normal mode fields: `Nombre` only when `sign_up_mode`; `Correo` `input_type="email"` `autocomplete="email"`; `Contraseña` `input_type="password"`, `autocomplete="current-password"` when signing in and `"new-password"` when signing up. All labels visible. The field values and `oninput` handlers stay wired to the existing signals.
- If `mfa_ticket` is non-empty: hide the segmented control and the name/email/password fields; render only `Código de verificación` (value `mfa_code`), the error, and `ActionRow` with a Primary `Button` `Verificar código` calling the existing `verify_mfa` closure and a Secondary `Button` `Volver` that clears `mfa_ticket` and `error`. Do not add a native form submit.
- Normal mode error: render `ErrorText { text: error }` when `error` is non-empty. Primary `Button` label is `Procesando...` while `submitting`, else `Iniciar sesión`/`Crear cuenta`; clicking calls the existing `submit` closure. Set the empty-submit message to `Completa todos los campos.` and the no-session fallback to `No se pudo iniciar la sesión.`.
- Keep the existing sign-in/sign-up API calls, device registration, local session persistence, `actor`, `user`, and `checked` updates, and the Dashboard navigation.

### Home

- Root: `div#home-page`. Do not wrap the page in `PageShell`; the bands stay full-bleed. Keep the Home route under the navbar layout.
- Resolve the primary CTA before RSX in this exact order:
  1. no session -> `Route::AuthPage {}`, label `Emitir receta`;
  2. session with a profile where `can_create_prescriptions()` -> `Route::PrescriptionNew { patient_id: None, template_id: None }`, label `Emitir receta`;
  3. session with a profile with a role but no create permission -> `Route::Prescriptions { status: None }`, label `Ver recetas`;
  4. session with no profile or no role -> `Route::SettingsProfile {}`, label `Completar perfil`.
- `Hero` title `Recetas claras, verificables y listas para firmar.`; subtitle `PersonaSanus reúne pacientes, plantillas y recetas en un espacio sereno, pensado para el trabajo clínico en Ecuador.`.
- Inside `Hero`: `ActionRow` containing the Primary `ButtonLink` CTA (route/label above) and a Secondary `LinkAction` `Verificar una receta` with `href="#verificar"`, `target: None`, `rel: None`; then `p.ui-microcopy` `Diseñado para clínicas y consultorios · Datos protegidos`; then `TrustSecurityRow`.
- `SectionBand id="como-funciona"` with `SectionHeader title="Cómo funciona" subtitle="Del paciente a la receta firmada en tres pasos."`; `div.ui-steps` containing three `NumberedStepCard`s:
  1. number 1, title `Elige el paciente`, accent `Mint`, mock `Patient`, tag `None`;
  2. number 2, title `Completa la receta`, accent `Cream`, mock `Medication`, tag `None`;
  3. number 3, title `Firma y verifica`, accent `Blossom`, mock `Verification`, tag `Some("Verificable")`.
  Body copy: 1 `Selecciona a la persona y revisa su ficha antes de recetar.`; 2 `Completa diagnóstico, indicaciones y medicamentos en un formulario claro.`; 3 `Firma, descarga el PDF y comparte un código verificable.`. Then `p.ui-steps-note` `Vistas de ejemplo del formato A4.`.
- `SectionBand id="verificar"` with `tone: SurfaceTone::Mint`; inside it `div.ui-verify-inline` > centered `SectionHeader title="Verifica una receta"` subtitle `Ingresa el código impreso junto al QR. La página muestra únicamente el médico, la fecha y el número de la receta: nunca los datos del paciente.`; `Card` containing `VerificationCodeForm { button_variant: ButtonVariant::Secondary }` with `on_submit` that calls `navigator.push(Route::Verify { code })`; then `TrustSecurityRow`.
- Do not render `TrustLogoStrip`.

### Verify

- Root: `div#verify-page.ui-public`; `div.ui-public-head` with `LogoLockup` and `PillBadge`; `div.ui-public-main.ui-stack`.
- The conditional region is only: `LoadingState label="Verificando receta…"` while `!loaded`; `Notice tone="Error"` with the fetch `message` when `loaded` and result is none and message is non-empty; `VerificationResultCard { verification }` when result is `Some`. No other card or heading sits inside that conditional.
- Below the conditional, always render `SectionCard title="Verificar otra receta"` containing `VerificationCodeForm` (default Primary) and then `TrustSecurityRow`.
- Keep `use_signal(|| code.clone())` for `requested`, and the fetch effect. At the start of every fetch: `loaded.set(false)`, `message.set(String::new())`, `result.set(None)`. Success sets the result; error sets the message; both set loaded true in a finally-style block.
- `VerificationCodeForm.on_submit`: the component already trims and uppercases and rejects empty. Set `requested` to the submitted code and call `navigator.replace(Route::Verify { code })`. Do not push. This makes a re-typed code on the same page refetch.
- Keep the route outside the auth layout. No auth guard, no patient field, no medication field, no request for patient data.

### es-EC copy

`Iniciar sesión`, `Crear cuenta`, `Cerrar sesión`, `Correo`, `Contraseña`, `Nombre`, `Código de verificación`, `Verificar código`, `Bienvenido de nuevo`, `Crea tu cuenta`, `Completa todos los campos.`, `Procesando...`, `Panel`, `Configuración`, `Sin clínica asignada.`, `Rol`, `Clínica`, `Sesión iniciada`, `Cargando...`, `Redirigiendo...`, `No se pudo iniciar la sesión.`, `Completar perfil`.

Keep existing Spanish copy elsewhere. Set the AuthPage empty-submit message to `Completa todos los campos.` and its no-session fallback to `No se pudo iniciar la sesión.` exactly.

## 8. Page adoption

| Page | Adoption | Only primary |
|---|---|---|
| `Navbar` | `header#navbar.ui-navbar` > `.ui-navbar-inner`; brand group `LogoLockup` + `PillBadge`; centre five guarded `NavLink`s Panel/Pacientes/Recetas/Plantillas/Configuración; remove Mi perfil/Usuarios/Auditoría and the text Home link; right `SyncStatusIndicator`, failed `{n} con error` link, `.ui-navbar-user`, Ghost `Cerrar sesión`; signed-out Ghost `ButtonLink` `Iniciar sesión`; keep the router effect and handlers. | None |
| `AuthPage` | Signed-out `PageShell id="auth-page"` > `.ui-auth-shell` > `.ui-auth-head` + `h1.ui-auth-title`; segmented Ghost mode buttons; `Card` + `FormGrid One` fields; `ErrorText`; one Primary submit. MFA ticket shows only the code, Primary `Verificar código`, Secondary `Volver`. Signed-in `PageShell` + `PageHeader` `Sesión iniciada` + `Card` + `DetailGrid` + Ghost `Cerrar sesión`. | One: sign in/create, or verify code in MFA |
| `Home` | Root `div#home-page` inside navbar layout, not `PageShell`; branch CTA; `Hero`; `ActionRow`; microcopy; `TrustSecurityRow`; `SectionBand como-funciona`; `SectionHeader`; `.ui-steps` with three `NumberedStepCard`s; `.ui-steps-note`; Mint `SectionBand verificar`; `.ui-verify-inline`; `SectionHeader`; `Card` + Secondary `VerificationCodeForm`; `TrustSecurityRow`. No `TrustLogoStrip`. | `Emitir receta` or `Ver recetas` |
| `Verify` | Root `div#verify-page.ui-public`; `.ui-public-head` `LogoLockup` + `PillBadge`; `.ui-public-main.ui-stack`; conditional `LoadingState` / Error `Notice` / `VerificationResultCard`; always `SectionCard` `Verificar otra receta` + Primary `VerificationCodeForm` + `TrustSecurityRow`. | `Verificar` |

## 9. CSS contract

| Selector | Values |
|---|---|
| `.ui-navbar` | Paper background, 1px hairline border-bottom, min-height 64px |
| `.ui-navbar-inner` | max-width 1200px, margin auto, padding 8px 24px, grid `1fr auto 1fr`, gap 16, align center |
| `.ui-navbar-brand` | flex center gap 12, min-width 0 |
| `.ui-navbar-links` | flex center wrap gap 4 |
| `.ui-navbar-account` | flex center justify-self end gap 12, min-width 0 |
| `.ui-navbar-user` | 14px Mist, max-width 180px, ellipsis, nowrap |
| `.ui-public` | min-height 100vh |
| `.ui-public-head` | max-width 1200px, margin-inline auto, padding 24px, flex space-between center gap 16, border-bottom hairline |
| `.ui-public-main` | max-width 720px, margin 48px auto 80px, padding 0 24px |
| `.ui-verify-inline` | max-width 560px, margin-inline auto, width 100% |
| `.ui-auth-shell` | max-width 440px, margin 64px auto 0 |
| `.ui-auth-head` | text-align center, margin-bottom 24px, padding-top 8px |
| `.ui-auth-title` | mint 500 26px/1.22 Ink, margin 0 0 4px |
| `.ui-auth-sub` | 14px/1.5 Mist, margin 0 |
| `.ui-segmented` | flex, gap 4, padding 4, 1px hairline, radius 9999, Paper; buttons flex 1, padding 8px 16px, radius 9999; `.is-active` Cream background, Ink text, 500 |
| `.ui-sync-link` | 14px Plum underline; owned by `18-R` — `20-T` consumes it and must not redefine it |
| Responsive | nav grid to flex wrap at 980; links full-width second row; `.ui-navbar-user` hidden below 760; page-head stacks below 640; steps one column below 900; hero padding 56px 0 48px below 640; band 64px 24px below 640 |

No new literal colour outside `:root`; no pure black; no drop shadow; no gradient except `--gradient-ember-ring` (and the verification ring owned by `18-R`). Cards and tags use the radius scale only.

## 10. Critical files and anchors

These are hints; reread before editing.

| File | Anchor |
|---|---|
| `persona-sanus/src/main.rs` | `enum Route` 48-93; `Verify` 1297; `Navbar` 1376; `AuthPage` 1461; `Settings` 1761; `App` 1222; `Home` 1281 |
| `persona-sanus/src/ui/mod.rs` | `15-O`–`18-R` exports; the `19-S` gallery gate |
| `persona-sanus/src/forms.rs` | `A4Preview` 463; `TemplateDocumentForm` 635; keep unchanged |
| `persona-sanus/assets/main.css` | `:root` 1-58; append shell/auth/public declarations after the `19-S` blocks; do not delete legacy selectors until `25-Y` |
| `context/sitemap.md` | §1.1–1.3 route tables; §2 role matrix; §5 conventions |
| `context/PRD.md` | FR-043, FR-050, D-06, D-21 |
| `context/architecture.md` | §3 client modules |

## 11. Assumptions and contingencies

- Scope is fixed: full v1 adoption plus presentational v1.1 billing. Do not add a billing route, a plan picker that calls an endpoint, a card form, or a `sync_locked` gate.
- No partner-clinic logos exist. Home keeps `TrustLogoStrip` out; the gallery sample note is not a Home claim.
- The repository has no grenette or mint webfonts. Keep the existing fallback stacks; do not fetch or add fonts.
- Dioxus 0.7 same-value signal sets may not retrigger an effect. If re-typing the same code on `/verify/:code` does not refetch after `requested.set(code)` plus `navigator.replace`, add one private monotonically increasing fetch-ticket signal, increment it in `on_submit`, and read it inside the effect alongside `requested`. Do not duplicate the fetch. This is a behavior-only fallback and must be recorded as a divergence if it is needed.
- If `ButtonLink.to` or `NavLink.to` reject `Route` in the props macro, change the prop type to `Route` and keep the internal Dioxus `Link`; the visual contract does not change.
- If the debug server path is not `target/dx/persona-sanus/debug/web/server`, use `dx serve --fullstack --port 8095` for the static proof. If a port is busy, change only the port and the matching URL.
- If browser verification finds a control not in the Design.md list but required by an existing page, use the nearest primitive (`Badge`, `Card`, `Button Secondary`) and do not add a component file. The component set is closed.

---

*Housekeeping: once a spec is fully done with no open divergences, move its
file to `specs/done/` (same file name, e.g.
`specs/done/20-T-shell-auth-home-and-verify.md`).
Specs with open divergences stay in `specs/` next to `TEMPLATE.md` until the
tracking file is cleared.*
