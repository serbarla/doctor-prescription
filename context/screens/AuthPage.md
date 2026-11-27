# PersonaSanus — AuthPage (`/login`)

| Field | Value |
|---|---|
| Source | `context/sitemap.md` §1.1 (`/login` — all staff pre-auth; sign in / sign up), §2 (`/login` for Administrator, Doctor, Nursing, Secretary; not the public verifier), §3 flow 1 (`/login` → `/dashboard`), §4 (shipped auth module; the sitemap leaves MFA and device registration at M3, shipped under specs 12-L and 04-D), §5 (locale; no PHI in URLs); `context/PRD.md` §3 (roles; server-side authorization), §9 (MFA for doctors/administrators; registered and revocable devices), §11 M3, D-04, D-06, D-21; `context/Design.md` Primary Action Button, Outlined Secondary Button, Ghost Nav Button, Form Input and Textarea, Spacing & Shapes, Elevation, Layout, Imagery; `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` step 10, page-adoption shared rules, `PageShell`, `PageHeader`, `Card`, `FormGrid`, `DetailGrid`, `DetailItem`, `ActionRow`, `TextField`, `Button`, `ButtonLink`, `ErrorText`; `context/screens/INDEX.md` §2.2 |
| Component | `AuthPage` — `persona-sanus/src/main.rs:1462` (body 1462–1698). Composes planned primitives from `persona-sanus/src/ui/` (planned, not shipped): `layout::PageShell`, `layout::PageHeader`, `layout::Card`, `layout::FormGrid`, `layout::DetailGrid`, `layout::DetailItem`, `layout::ActionRow`, `form::TextField`, `button::Button`, `button::ButtonLink`, `feedback::ErrorText` |
| Route | `/login` (`Route::AuthPage {}`), declared `persona-sanus/src/main.rs:54-55`. Public: no session guard, no role guard, no query params, no path param. Signed-out submit posts to the auth API; a successful sign-in or MFA verification sets the session and pushes `/dashboard` (`Route::Dashboard {}`). `Panel` pushes `/dashboard`; `Cerrar sesión` posts sign-out, clears persisted state, and pushes `/` (`Route::Home {}`). No `next` parameter, no hash form |
| Layout | Inside `#[layout(Navbar)]`, opened at `persona-sanus/src/main.rs:51` and never closed. `PageShell id="auth-page"` on every branch. Signed-out: `.ui-auth-shell` (max 440, centered vertically, 64px top margin) holding `.ui-auth-head`, `.ui-segmented`, and one `Card`. Signed-in: `PageHeader` plus one `Card`. All bands Paper; no tinted band, no full-bleed region, no sidebar. The navbar band renders above the page and is owned by `context/screens/done/Header.md` |
| Locale | `es-EC` on every string. `receta`, `cédula` keep Ecuador Spanish spelling. Dates elsewhere use `DD-MM-AAAA`; times use 24-hour. This screen displays no dates or times |
| Status | ✅ Spec complete (sections 1–7). Route status: ✅ — sitemap §1.1 ships `/login` for all staff pre-auth ("Sign in / sign up (Diesel-backed auth module)") and ships the MFA step and device registration/revocation (MFA ✅, device registration/revocation ✅ per spec 04-D and sitemap §4). Shipped: email/password sign-in and sign-up, the MFA challenge, silent device registration after session success, the signed-in identity card, and sign-out. Shipped: §1–§7 presentation via adopted `PageShell`/`PageHeader`/`Card`/`FormGrid`/`TextField`/`ErrorText`/`ActionRow` primitives; no new route |

## 1. Purpose

`AuthPage` is the staff entrance. It does four jobs, in order:

1. Establish a session from an email and password so staff can reach `/dashboard`.
2. Offer account creation with a display name, email, and password; the account is created with no clinic or role choice here.
3. When the server answers `mfaRequired`, stop credential entry and collect the TOTP code before admitting the session.
4. When a session already exists, show who is signed in and offer `Panel` to `/dashboard` and `Cerrar sesión` to `/`.

It never renders patient or clinical content: no patient name, cédula, diagnosis, medication, photo, prescription number, or template. The only identity shown is the staff member's own account name and email.

It never renders billing, price, trial, plan, or `sync_locked` copy (D-04); v1 is a free beta. It never renders signature or stamp imagery and never implies a digital signature; the doctor signs the printed paper (D-06). It never renders a rich-text editor, formatting toolbar, or `contenteditable` surface; every value is a plain input (D-21).

It never renders password reset, email verification, SSO or social sign-in, role or clinic selection, a remember-me control, terms acceptance, invitation acceptance, MFA enrolment, a device list, device revocation, account deletion, or audit data. It never queues an auth attempt for later: offline sign-in is not a screen step.

## 2. Structure

| Order | Region | id / class | Band tone | Contents |
|---|---|---|---|---|
| — | Navbar (layout) | `#navbar` · `header.ui-navbar` | Paper | Owned by `context/screens/done/Header.md` §2: brand, role-guarded links, sync indicator, session email, ghost sign-out or sign-in |
| 1 | Page shell | `#auth-page` · `.ui-page` > `.ui-container.ui-stack` | Paper | `PageShell` root for both branches. 48px top, 80px bottom, 24px inline; 1200px container; 24px stack gap |
| 2 | Signed-out stage | `.ui-auth-shell` | Paper | Only when `user()` is `None`. Max 440px, centered, 64px top margin, one column. Holds 2.1 and either 2.2–2.3 or 2.4 |
| 2.1 | Auth head | `.ui-auth-head` · `h1.ui-auth-title` | Paper | Centered `h1`; padding-top 8px, 24px bottom margin. No subtitle: `.ui-auth-sub` is not emitted |
| 2.2 | Mode switch | `div.ui-segmented` | Paper | Sign-in only, no ticket. Two Ghost `Button`s, `Iniciar sesión` and `Crear cuenta`; active carries `is-active` (Cream, Ink, 500) |
| 2.3 | Credentials card | `.ui-card` > `.ui-form-grid.ui-form-grid--one` | Paper | Sign-in or sign-up form. `Nombre` only in sign-up; `Correo`; `Contraseña`; `ErrorText` when non-empty; one Primary submit |
| 2.4 | MFA card | `.ui-card` > `.ui-form-grid.ui-form-grid--one` | Paper | Only while `mfa_ticket` is non-empty. Replaces 2.2 and 2.3: `Código de verificación`, `ErrorText`, Primary `Verificar código`, Secondary `Volver` |
| 3 | Signed-in stage | `#auth-page` > `PageHeader` + `.ui-card` | Paper | Only when `user()` is `Some`. Replaces 2. No form, no segmented control, no error surface |
| 3.1 | Account header | `.ui-page-head` · `h1.ui-page-title` | Paper | `Sesión iniciada`; no subtitle, no header actions |
| 3.2 | Account card | `.ui-card` > `DetailGrid` + `ActionRow` | Paper | `Nombre` and `Correo` items; Secondary `ButtonLink` `Panel`; Ghost `Button` `Cerrar sesión` |
| 4 | Public footer | `<footer id="ui-footer">` | Paper | Signed-out branch only; appended by `context/screens/done/Footer.md` §2, which owns its structure and copy. The navbar and the page do not restate it |

There is no `SectionCard`, so the page has no `h2`. The navbar is the only banner and navigation landmark. No region uses a chromatic border or a shadow.

### 2.1 Session check

The page reads `user()`; `App` fills that context from `auth::fetch_session()` on web and from the persisted local session on desktop when the server is unreachable. While the check is in flight `user()` is `None`, so the signed-out credential stage paints first with sign-in active. No spinner, skeleton, or error appears for this interval. If a session resolves, the same route re-renders as the signed-in stage; the URL does not change.

Only the stage changes. The navbar, the 440px column, and the single-`h1` rule do not.

### 2.2 Signed-out, sign-in mode

`sign_up_mode` is false and `mfa_ticket` is empty. The heading is `Bienvenido de nuevo`; `Nombre` is absent; `Correo` and `Contraseña` are the only fields. The Primary label is `Iniciar sesión`. The switch keeps both options visible with sign-in active.

Submitting trims the email and sends the password unchanged. The client checks only that email and password are non-empty; it does not validate email shape, password length, or password strength. On failure the form stays populated, the error renders below the fields, and no session context is set.

### 2.3 Signed-out, sign-up mode

`sign_up_mode` is true. The heading becomes `Crea tu cuenta`, `Nombre` appears above `Correo`, and the Primary label becomes `Crear cuenta`. All three fields are required by the client's non-empty check. Switching modes clears the error and keeps typed values; it does not call the server.

The password is opaque: never trimmed, normalized, or length-checked here. The server's eight-character minimum is the only length rule, and a shorter value returns as an ErrorText message after submit. Sign-up success lands on `/dashboard` by the same path as sign-in; no clinic or role is chosen here.

### 2.4 MFA challenge

A sign-in response with `mfaRequired` true and a non-empty `mfaTicket` enters this state. The mode switch and the three credential fields unmount; the heading stays `Bienvenido de nuevo`; the MFA card shows only `Código de verificación`, Primary `Verificar código`, Secondary `Volver`, and the ErrorText when present.

`Verificar código` is the sole Primary while the ticket exists; the credential submit is not in the DOM. An empty submit produces a client ErrorText and no request. A wrong code produces the server's invalid-code message, leaves the ticket and typed code in place, and keeps the same Primary, so the user can correct and retry. An expired five-minute ticket produces the same unauthorized message; the page holds no timer and cannot distinguish expiry from a wrong code. `Volver` clears the ticket and returns to sign-in credentials; it does not call the server and does not enter the signed-in stage.

The challenge never infers MFA from role, email, or a remembered flag. Only the server response opens this state. The route, navbar, heading, and one-Primary rule do not change.

### 2.5 Signed-in

`user()` is `Some`. The page shows `Sesión iniciada`, the account `Nombre`, the account `Correo`, a Secondary `Panel` link to `/dashboard`, and a Ghost `Cerrar sesión`. The profile context `actor` may be `None`; this stage does not read it and shows no clinic or role. The credential form, mode switch, MFA card, and busy label are absent. There is no Primary.

`Cerrar sesión` ignores the server result, clears the desktop persisted session, clears `user` and `actor`, and pushes `/`. A failed network sign-out still ends the local session; the server session expires on its own schedule.

### 2.6 Web and desktop surfaces

Both targets render the same markup and strings. Web keeps the session in the `persona_session` cookie: `HttpOnly`, `SameSite=Lax`, `Path=/`, 30-day `Max-Age` (`auth_server.rs:26`, `:894-897`). Desktop also persists a copy through `persist_local_session` and restores it through `restore_local_session`; `App` uses that copy only when `/api/auth/get-session` cannot be reached. This page never reads or writes browser storage.

Desktop offline without a persisted session: the form renders, submit fails at the transport, and the fallback ErrorText appears. Offline with a persisted session: the signed-in card renders, `Panel` reaches the dashboard cache, and sign-out clears the local copy. No offline path authenticates credentials, and no auth attempt enters the outbox. Device registration is attempted only after an online session result; failure is ignored.

### 2.7 Busy, validation, errors

`submitting` is true from the start of a submit or verification until its handler resolves. The active Primary is disabled, carries `aria-busy="true"`, and reads `Procesando...`; `Volver` and both mode buttons are disabled too. Field values stay visible. There is no spinner and no second action.

The client gate is a trimmed non-empty check on every field required by the current mode. An empty submit sets `error` and makes no request. The MFA empty check also makes no request. All other validation, including the password minimum, account uniqueness, and code correctness, is the server's.

Errors are one `ErrorText` line inside the active card, after the fields and before the actions. It has `role="alert"`, the Alert icon, and Plum text, never a toast, browser dialog, page title, URL parameter, or second inline error. When `error` is empty nothing renders. An auth failure does not change the navbar bullets, the route, or any session context.

## 3. Copy text

All strings are `es-EC`, sentence case, punctuation included. Chrome copy is by reference only: navbar strings belong to `context/screens/done/Header.md` §3, footer strings to `context/screens/done/Footer.md` §3. The signed-out stage hosts the public footer band; its copy is owned there and not repeated here.

### 3.1 Headings

| Element | Copy |
|---|---|
| Signed-out `h1`, sign-in mode | `Bienvenido de nuevo` |
| Signed-out `h1`, sign-up mode | `Crea tu cuenta` |
| Signed-in `h1` | `Sesión iniciada` |
| Subtitle, all states | none rendered |

### 3.2 Mode switch and credentials

| Element | Copy |
|---|---|
| Sign-in mode option; credential submit in sign-in mode | `Iniciar sesión` |
| Sign-up mode option; credential submit in sign-up mode | `Crear cuenta` |
| `Nombre` field label, sign-up only | `Nombre` |
| `Correo` field label, both modes | `Correo` |
| `Contraseña` field label, both modes | `Contraseña` |
| Access-mode group `aria-label` | `Tipo de acceso` |

The fields have no placeholders. The password has no reveal control and no confirmation field.

### 3.3 MFA challenge

| Element | Copy |
|---|---|
| `Código de verificación` field label | `Código de verificación` |
| Primary submit | `Verificar código` |
| Secondary action | `Volver` |

### 3.4 Busy labels

| Element | Copy |
|---|---|
| Credential submit while `submitting` | `Procesando...` |
| MFA verify while `submitting` | `Procesando...` |

`Procesando...` is one string used by both Primary buttons; the credential submit and the MFA verify never render together. No other control changes label while busy.

### 3.5 Signed-in account

| Element | Copy |
|---|---|
| Detail label for the account name | `Nombre` |
| Detail label for the account email | `Correo` |
| Secondary `ButtonLink` to `/dashboard` | `Panel` |
| Ghost sign-out `Button` | `Cerrar sesión` |

### 3.6 Errors

| Element | Copy | Source |
|---|---|---|
| Empty credential submit | `Completa todos los campos.` | AuthPage client check |
| Empty MFA code, client; also the server empty-code guard | `Ingresa el código de verificación.` | Client check, or `auth_server::mfa_verify` |
| Invalid or expired MFA ticket | `El código de verificación no es válido o ha expirado.` | `auth_server::mfa_verify` unauthorized sentence |
| Wrong email or password | `Correo o contraseña incorrectos.` | `auth_server::sign_in` unauthorized sentence |
| Existing account for the email | `Ya existe una cuenta con este correo.` | `auth_server::sign_up` unique-email conflict |
| Malformed email on sign-up | `Ingresa un correo válido.` | `auth_server::sign_up` validation |
| Empty display name on sign-up | `Ingresa tu nombre.` | `auth_server::sign_up` validation |
| Password under eight characters | `La contraseña debe tener al menos 8 caracteres.` | `auth_server::sign_up` validation |
| Too many auth POSTs from one address | `Demasiadas solicitudes. Inténtalo de nuevo en un minuto.` | `AuthError::RateLimited` |
| Successful auth without a session; unreachable server; unreadable response; internal error | `No se pudo iniciar la sesión.` | AuthPage fallback, or the auth/http failure sentence |

The three client-authored strings are `Completa todos los campos.`, `Ingresa el código de verificación.`, and `No se pudo iniciar la sesión.`. The rest render verbatim from the returned `message`; AuthPage never matches, rewrites, or prefixes them. A rate-limited response is 429 with `code: "rate_limited"`; every other auth failure keeps its own status.

### 3.7 Programmatic strings

| Element | Copy | Note |
|---|---|---|
| Busy-state accessible name | `Procesando...` | The visible label; `aria-busy="true"` adds the state, no hidden suffix |
| Error announcement | — | `ErrorText` owns the visible sentence and `role="alert"`; no duplicate live region |
| Visually hidden text | — | None on this page; every control has a visible label |
| Field `aria-label` | — | None; the visible labels wrap their controls |

## 4. Displayed information

### 4.1 What the screen shows

| Element | Source | Value shown |
|---|---|---|
| Stage | context `Signal<Option<SessionUser>>` `user`; `user()` is the render snapshot | Signed-out stage when `None`, signed-in stage when `Some` |
| Mode | local `Signal<bool>` `sign_up_mode`, default false | Sign-in or sign-up; heading, `Nombre`, and submit label follow it |
| Signed-out heading | `sign_up_mode` | `Bienvenido de nuevo` or `Crea tu cuenta` |
| Display name field | local `Signal<String>` `name` | Typed text, sign-up only; trimmed for the request; not reused in the signed-in card |
| Email field | local `Signal<String>` `email` | Typed text, trimmed for the request; the server lowercases it |
| Password field | local `Signal<String>` `password` | Masked typed text only; never copied, formatted, or shown |
| MFA challenge | local `Signal<String>` `mfa_ticket` and `mfa_code`; set from `auth::is_mfa_required` and `auth::mfa_ticket` | Challenge card while the ticket is non-empty; otherwise the credential card |
| Error | local `Signal<String>` `error` | The active §3.6 sentence; empty renders no ErrorText |
| Busy | local `Signal<bool>` `submitting` | Disables the Primary and Secondary; Primary label becomes `Procesando...` |
| Account name | context `user.name`, originally `SessionUser.name` from `auth::fetch_session` / `auth::parse_session_user` | Staff display name in the signed-in card |
| Account email | context `user.email`, same source | Staff email in the signed-in card; account identity, not patient contact |
| Profile | context `Signal<Option<Me>>` `actor`, from `api::fetch_me()` | Not displayed; cleared at sign-out. Clinic and role are absent |
| Checked flag | context `Signal<bool>` `checked` | Not displayed; set true after the handler resolves; does not gate the signed-out paint |
| Device row | `api::register_current_device()` — posts `deviceId`, `platform`, `name`, `appVersion` to `/api/devices/register` | Not displayed; called silently after a session exists; failure ignored |
| Panel target | `Route::Dashboard {}` | `/dashboard` |
| Sign-out result | `auth::sign_out()`, then `clear_persisted_session()`, `actor.set(None)`, `user.set(None)`, `Route::Home {}` | `/`; the page leaves `/login` |
| Session lifetime | server `SESSION_DAYS = 30`; cookie `HttpOnly`, `SameSite=Lax`, `Path=/` | Not displayed; browser-managed |
| MFA ticket lifetime | server `MFA_TICKET_MINUTES = 5` (`auth_server.rs:27`) | Not displayed; server-authoritative |
| Auth POST ceiling | `AuthRateLimiter`, default 20 per minute per IP (`AUTH_RATE_LIMIT_PER_MINUTE` overrides) | Not displayed; 429 returns the rate-limit sentence |

### 4.2 Hard rules

- Server-side authorization is the boundary, never this page. `AuthPage` establishes a session; it does not decide what a role may read, write, or sign. The MFA gate lives in `auth_server::sign_in` and applies to `Administrator` and `Doctor` with MFA enabled; the client cannot skip, force, or infer it from an email, a role, or a stored flag.
- The signed-in card is an identity surface, not a permission surface. It reads `SessionUser.name` and `SessionUser.email` only. It does not read `Me.role`, `Me.clinic_name`, or any patient field, and it never shows `Sin clínica asignada.`, `Rol`, or `Clínica`.
- Absent `Me` or absent clinic data is legal. The signed-in card renders no clinic line; `Panel` stays available and `/dashboard` owns the explanation. Do not add profile-completion or clinic-selection controls here.
- Client validation is presence only, after trimming name and email; the password is tested for non-empty but never trimmed. Email shape, password minimum 8, account uniqueness, MFA correctness, and rate limits are server rules. Do not add a client regex, strength meter, confirmation field, or different password length.
- One filled Primary per viewport. It is the credential submit (`Iniciar sesión` or `Crear cuenta`) in the credential state, or `Verificar código` in the MFA state; the two states never coexist, so the count is never two. The signed-in stage has no Primary: `Panel` is Secondary and `Cerrar sesión` is Ghost. The mode switch is Ghost only.
- The MFA challenge replaces the credential form. While a ticket exists, hide the mode switch, `Nombre`, `Correo`, `Contraseña`, and the credential submit. The only secondary action is `Volver`, which clears the ticket and error without a server call.
- Server truth for MFA: mounting the challenge happens only from `mfaRequired` true plus a non-empty ticket. Invalid and expired codes share the server's unauthorized response; the page adds no countdown, attempt limit, resend, or "expired" branch of its own. After `Volver`, the user re-authenticates normally.
- Passwords never appear in copy, URL, query, logs, analytics, `aria-label`, hidden text, or a prefilled value. The input is `type="password"`; autocomplete is `current-password` for sign-in and `new-password` for sign-up. Browser password managers are allowed. No show-password control ships.
- Email is account identity only. It is not patient contact data and must not be presented as PHI; no patient name, cédula, diagnosis, medication, prescription number, clinic record, or clinical free text enters this page.
- No PHI in the URL: `/login` carries no query string and no path parameter, and the page adds none. The MFA ticket stays in memory, never in the address bar or browser storage.
- Redirect only after a session object exists. The handler parses the response, refreshes `auth::fetch_session()`, and prefers the fresh value, otherwise the parsed one. If both are absent, it stays on `/login`, shows the §3.6 fallback, and sets no session. The destination is fixed `/dashboard`; no `next` parameter exists, so the page cannot be used as an open redirect.
- Silent device registration is not a screen step. After a session exists, the page calls `api::register_current_device()`, ignores its result and error, then fetches `Me`. Registration never gates navigation, changes copy, exposes a retry, or appears in the error region. It runs on web and desktop; the endpoint requires the new session's actor.
- The page never persists a session itself. Web persistence is the server's `HttpOnly` cookie; desktop persistence is `persist_local_session` and its restore/clear helpers. No token, ticket, password, or email is written to `localStorage` or `sessionStorage` by `AuthPage`.
- Sign-out is local-first: the remote call is best-effort, the persisted copy and both contexts are cleared regardless, and the destination is `/`. A sign-out failure never keeps the user on `/login` or claims the session is still valid.
- No billing, price, trial, plan, payment, or `sync_locked` surface (D-04); no signature or stamp image, ink mark, or signing affordance (D-06); no rich text, `contenteditable`, markdown editor, or formatting toolbar (D-21). No new endpoint, table, column, role, or route.
- One column at 440px and at mobile widths. No horizontal scroll at 320px. No focus trap and no modal.

### 4.3 Dynamic states

| State | Screen behaviour |
|---|---|
| Session check before contexts resolve | Signed-out credential stage with sign-in active. No spinner, loading copy, or error. If a session resolves, the stage swaps to the signed-in card on the same route |
| Signed-out, sign-in mode | Heading `Bienvenido de nuevo`; `Correo` and `Contraseña`; Primary `Iniciar sesión`; option `Crear cuenta` available. One Primary |
| Signed-out, sign-up mode | Heading `Crea tu cuenta`; `Nombre`, `Correo`, `Contraseña`; Primary `Crear cuenta`; option `Iniciar sesión` available. One Primary |
| Mode switch | Error clears; typed values remain; `Nombre` hides or appears without server work; focus stays on the pressed mode button. No navigation |
| Empty credential submit | Client stops before the request; ErrorText `Completa todos los campos.`; `submitting` stays false; values stay |
| Wrong credentials | Server unauthorized; `submitting` false; form and mode preserved; ErrorText `Correo o contraseña incorrectos.`; no session context; no navigation |
| Sign-up validation failure | Server message in ErrorText shown, for example the email, name, or eight-character password sentence; values stay; no account and no session |
| Existing account on sign-up | Server conflict sentence in ErrorText; stays in sign-up with the email available for a different address or a switch to sign-in; no session |
| Server or transport failure | ErrorText `No se pudo iniciar la sesión.`; form remains; retry is a new explicit submit. No automatic retry, no queued attempt |
| Rate limited | 429 sentence `Demasiadas solicitudes. Inténtalo de nuevo en un minuto.` in ErrorText; no countdown; after the server window a new submit is allowed. Default ceiling 20 auth POSTs per minute per IP |
| MFA required | Credential fields submit, mode switch, and credential submit unmount; heading stays `Bienvenido de nuevo`; code field, Primary `Verificar código`, Secondary `Volver`. No error is seeded |
| MFA code empty | Client stops before the request; ErrorText `Ingresa el código de verificación.`; ticket stays; Primary remains the sole filled action |
| MFA invalid | Server unauthorized sentence in ErrorText; `submitting` false; ticket and typed code stay for correction; `Verificar código` remains the sole Primary; `Volver` remains available |
| MFA expired | Same as invalid: the five-minute ticket is gone or refused and the server's unauthorized sentence appears. No timer and no distinct expired screen. Retrying unchanged code keeps failing; `Volver` restarts credentials |
| MFA success | Session parsed and refreshed, silent device registration attempted, profile fetched, contexts set, `submitting` false, route pushed to `/dashboard`. No success banner here |
| Auth success, no session | Stays on `/login`; ErrorText `No se pudo iniciar la sesión.`; no signed-in card; no navigation |
| Submit busy | Active Primary disabled, `aria-busy="true"`, label `Procesando...`; mode switch and Secondary disabled; a second click is a no-op; one Primary holds |
| Signed-in, profile present or absent | `Sesión iniciada`, account `Nombre`, account `Correo`, Secondary `Panel`, Ghost `Cerrar sesión`; no clinic or role line; no Primary. Absent `Me` changes nothing here |
| `Panel` click | Pushes `/dashboard`; the auth page unmounts |
| `Cerrar sesión` click | Remote sign-out attempted and ignored; persisted session cleared; contexts cleared; router pushes `/`; navbar becomes signed-out |
| Desktop offline, persisted session | Signed-in card from the local session; no credential attempt; `Panel` reaches the dashboard cache; sign-out clears locally and goes `/`; this page owns no sync indicator |
| Desktop offline, no session | Credential form; submit fails at the transport; ErrorText `No se pudo iniciar la sesión.`; no session is fabricated and nothing is queued |
| Web server unreachable | Credential form; submit fails; fallback ErrorText; retry only by a new submit. The route never claims a session it did not receive |
| Any staff role, signed out | Administrator, Doctor, Nursing, and Secretary see the same page; role is unknown before auth. The public verifier has no account and is out of scope. No role-dependent block renders |
| `pending_sync` or `failed` rows | Not applicable. Auth is not a queued document; this page renders no outbox, sync count, or failed-row list |

## 5. Style

Every value is a `context/Design.md` token declared in `persona-sanus/assets/main.css` `:root`. No new colour, font, icon set, CDN, or webfont.

### 5.1 Color

| Use | Value | Token |
|---|---|---|
| Page canvas, card, input, segmented surfaces | `#ffffff` | `--color-paper` |
| Headings, detail values, input text, active segment text | `#061d29` | `--color-ink` |
| Labels, placeholder, Secondary text and border, detail labels | `#425d6d` | `--color-mist` |
| Card, input, segmented, hairline separators | `#e5e7eb` | `--color-hairline` |
| Primary fill; focus outline | `#006e75` | `--color-deep-teal` |
| Primary hover fill, Ink text | `#0b978e` | `--color-bright-teal` |
| Active segment fill | `#fffded` | `--color-cream` |
| Error text and Alert icon | `#4d0037` | `--color-plum` |
| Focus halo at 55% | `rgba(185,255,232,0.55)` | `--color-mint-glass` |

Never `#000000`. Never a chromatic border: the only border colours are Hairline and Deep Teal on a focused control. The Primary is the only filled teal surface. The active segment is Cream with Ink text, not Deep Teal.

Measured contrast (WCAG AA body needs ≥ 4.5): Ink on Paper 17.28; Ink on Cream 16.89; Mist on Paper 6.96; Paper on Deep Teal 6.02; Ink on Bright Teal 4.80; Plum on Paper 15.32. Deep Teal on Paper is 6.02, above the 3:1 non-text threshold. The Mint halo is a companion, never the only focus signal.

### 5.2 Typography

| Element | Family | Weight | Size / line-height |
|---|---|---|---|
| Signed-out `h1` | `--font-mint` | 500 | 26px / 1.22 (`--text-heading`), Ink, margin 0  kiss0 4 |
| Signed-in `h1` | `--font-mint` | 500 | 22px / 1.22 (`--text-heading-sm`), Ink, margin 0 |
| Mode-switch labels | `--font-mint` | 400; active 500 | 16px / 1.22 (`--text-body`), Ink |
| Field labels | `--font-mint` | 500 | 14px / 1.5 (`--text-body-sm`), Ink |
| Field values and placeholders | `--font-mint` | 400 | 16px / 1.5, Ink; placeholder Mist |
| Primary and Secondary button labels | `--font-mint` | 500 | 16px / 1.22, Paper / Mist |
| Ghost labels (`Cerrar sesión`) | `--font-mint` | 400 | 16px / 1.22, Ink |
| Detail labels (`Nombre`, `Correo`) | `--font-mint` | 400 | 14px / 1.5, Mist |
| Detail values | `--font-mint` | 400 | 16px / 1.5, Ink |
| Error line | `--font-mint` | 400 | 14px / 1.5, Plum |

`--font-grenette` is display-only at 46px and above; it never appears here, including on the `h1`. No weight above 500.

### 5.3 Geometry

- Base unit 4px; radii: buttons/inputs 8px, cards 24px, pills and segmented 9999px.
- `PageShell`: 48px top, 80px bottom, 24px inline; `.ui-container` max 1200px centered; `.ui-stack` gap 24px.
- `.ui-auth-shell`: max-width 440px, `margin: 64px auto 0`; single column; 24px gap between head and card. Under 520px the 24px page inline padding is the only side padding, so card padding stays 24px and fields stay one column.
- `.ui-auth-head`: centered, padding-top 8px, margin-bottom 24px; no subtitle block.
- `.ui-segmented`: flex, full shell width, 4px gap, 4px padding, 1px Hairline, 9999px radius, Paper; each Ghost button flex 1, min-height 44px, 8px 16px padding, 9999px radius; active Cream + Ink 500.
- Card: 1px Hairline, 24px radius, 24px padding, Paper, flat, no shadow. Form stack gap 16px.
- Field: label above control, 4px gap; control min-height 44px, 8px 12px padding, 8px radius, 1px Hairline. Focus adds the shared ring without changing height.
- Error line: 8px top margin, 6px icon gap, 16px icon; above the action row; never shifts card width.
- `ActionRow`: 16px top margin, 12px gap, wraps lists. Credential submit full-width via `.ui-button--block`, min-height 44px, 12px 24px padding, radius 8. MFA keeps `Verificar código` first and `Volver` second; both min-height 44px and wrap on narrow screens.
- Signed-in: `PageHeader` 22px title, no subtitle blockethyl; card 24px padding; `DetailGrid` two columns with 16px row and column gap, one column at 760px; `ActionRow` 16px top, 12px gap.
- Elevation is flat. Hairline only. No drop shadow, gradient card, chromatic border, or decorative rule. `--shadow-paper` belongs to the A4 sheet and is not used here.
- No full-bleed exception. The navbar is the only full-bleed band and belongs to `Header.md`.

### 5.4 Iconography and imagery

Only the `ErrorText` Alert glyph is an icon: line-based, 1.5px stroke, 16px, Plum, `aria-hidden`, paired with the visible sentence. The Primary and Ghost buttons carry no icon. There is no eye or reveal icon, mail or lock glyph, provider logo, avatar, illustration, photograph, QR, signature or stamp image, or document mock. The logo lockup lives in the navbar, not here. The empty space above the card is the only decoration.

## 6. Accessibility

- Landmarks: the navbar band is the page's banner and navigation landmark, owned by `Header.md`. `PageShell` renders a plain `div#auth-page`; this page adds no second `header`, no `nav`, and no `main`. The single `h1` is the screen-reader entry point, and all focusable content stays in DOM order inside the shell. No off-screen, `aria-hidden`, or duplicate form content exists.
- Heading order: exactly one `h1` per state. Signed-out: `Bienvenido de nuevo` or `Crea tu cuenta`. Signed-in: `Sesión iniciada`. No `h2` appears because no `SectionCard` is used; do not add a card title heading. The MFA card keeps the sign-in `h1` and adds no second heading.
- Tab order, credentials: navbar controls (by `Header.md`) → `Iniciar sesión` mode → `Crear cuenta` mode → `Nombre` when sign-up → `Correo` → `Contraseña` → Primary submit. ErrorText is not focusable; its `role="alert"` announces it.
- Tab order, MFA: navbar controls → code field → `Verificar código` Primary → `Volver` Secondary. The mode switch and credential fields are not in the DOM, so they are not tabbable.
- Tab order, signed-in: navbar controls → `Panel` → `Cerrar sesión`. Detail values are text, not controls. No Primary exists.
- Focus ring on every button and input: `outline: 1px solid var(--color-deep-teal); outline-offset: 2px; box-shadow: 0 0 0 3px rgba(185, 255, 232, 0.55)`; never `outline: none` without it. Focused inputs also set the Deep Teal border. The ring is not clipped by the card or the segmented rail.
- Labels: every input is wrapped by its visible `label`, so `Nombre`, `Correo`, `Contraseña`, and `Código de verificación` are the accessible names. No placeholder is used as a label, no `aria-label` duplicates a visible label, and no hidden text is required. The access-mode group carries `aria-label="Tipo de acceso"`; its buttons are named by visible text.
- State: the active mode is Cream plus weight 500 plus the changed `h1` and field set, not colour alone. The busy Primary is `disabled`, `aria-busy="true"`, and visibly `Procesando...`; no separate spinner announcement is needed. The error is Plum plus Alert icon plus sentence with `role="alert"`; colour is never the only signal.
- Errors: one `ErrorText` region carries `role="alert"`, announced on failure. No field-level `aria-describedby` is added because no hint or per-field error exists on this screen; the message is not attached to a single input. Focus is not trapped; the form is not a dialog. A mode switch clears the old error so a stale alert is not re-announced.
- MFA: the code field is a visible text input labeled `Código de verificación` with `autocomplete="one-time-code"`; it is not a password field and does not hide digits. The invalid message is announced once. The page does not auto-submit, count down, or move focus when the ticket expires. After `Volver`, focus returns to the credential form in DOM order; the mode switch is available.
- Contrast: all body text uses Ink or Mist on Paper or Cream at the ratios in §5.1, at or above 4.5. Plum on Paper is 15.32. Paper on Deep Teal is 6.02. Ink on Bright Teal is 4.80. The focus outline and border are non-text indicators above 3:1 against Paper. No state relies on a hue change alone.
- Touch and keyboard: every control is a native `button` or `input`, reachable by Tab, operable by Enter/Space (buttons) or typing queried, with a minimum 44px height and 44px effective target. The segmented options have 44px min-height and none is hidden on mobile. There is no hover-only action and no native form submit that could reload the page.
- Reduced motion: no page animation, spinner, parallax, or auto-transition. The only motion is any global Button colour transition; `prefers-reduced-motion: reduce` disables it without changing state, and no information depends on it.
- Zoom and reflow: at 200% and a 320px viewport the shell, fields, MFA actions, and signed-in detail list remain in one column with no horizontal scroll; button labels wrap rather than truncate. Text is not in images.

## 7. Implementation notes

Shipped: `AuthPage` at `persona-sanus/src/main.rs:1585`, inside `#[layout(Navbar)]` (`main.rs:66`). Both branches are `PageShell id="auth-page"`; there is no `div#echo` root.

- Signed-in early return (`main.rs:1600-1633`): `PageHeader` `Sesión iniciada` + `Card` > `DetailGrid` (`Nombre` / `Correo`) + `ActionRow` (Secondary `Panel` → `/dashboard`, Ghost `Cerrar sesión` → sign-out sequence ignored-result + `clear_persisted_session()` + `actor`/`user` cleared + push `/`). No Primary, no footer.
- Signed-out stage (`main.rs:1772-1934`): `.ui-auth-shell` > `.ui-auth-head` (`h1.ui-auth-title` `Bienvenido de nuevo` / `Crea tu cuenta` at `main.rs:1776`), `div.ui-segmented` (Ghost `Iniciar sesión` / `Crear cuenta`, `is-active` on the current mode), `Card` > `FormGrid columns: One` with labeled `TextField`s (`Nombre` sign-up only, `Correo` `input_type="email"`, `Contraseña` with `password_autocomplete` per mode), `ErrorText` when `error()` is non-empty, and one Primary `Button` (`loading: submitting()`, `Procesando...` while busy, else `Crear cuenta` / `Iniciar sesión`).
- MFA branch (`main.rs:1870-1929`): while `mfa_ticket()` is non-empty the mode switch and credential fields unmount; the same shell shows `TextField` `Código de verificación` (`autocomplete="one-time-code"`), `ErrorText`, Primary `Verificar código` (`verify_mfa`), and Secondary `Volver` (clears ticket + error, no request). `submit` / `verify_mfa` keep the behavior in §2.7/§4: trimmed non-empty client gate (`Completa todos los campos.` / `Ingresa el código de verificación.`), server text rendered verbatim in `ErrorText`, session parse → `fetch_session` refresh → silent `register_current_device()` → `fetch_me()` → desktop persist → contexts set → push `/dashboard` only when a session exists, else `No se pudo iniciar la sesión.`
- Footer: `Footer { surface: FooterSurface::Auth }` at `main.rs:1932`, last child of the signed-out `PageShell` (never the signed-in branch); navbar and footer copy are owned by `Header.md` §3 / `Footer.md` §3 and are not restated here.

Verification, in order:

- `cd persona-sanus && cargo check` passes.
- `dx build --release --fullstack` succeeds.
- `curl -fsS http://127.0.0.1:8080/login` contains `Bienvenido de nuevo`, `Iniciar sesión`, `Correo`, and `Contraseña`, contains exactly one `ui-button--primary`, and contains no `Welcome back`, `Sign in`, `Sign up`, `Email`, or `Password`.
- `caveman-blocks run persona-smoke --dir persona-sanus --port 8091` returns `"ok": true`; its `login`, `signup`, `session`, `signout`, `session_after_signout`, and `signin` stages are true.
- Manual browser check with MFA enrolled: a Doctor or Administrator sign-in swaps credentials for the challenge with `Verificar código` as the only filled button; an empty code shows `Ingresa el código de verificación.`; a wrong code shows `El código de verificación no es válido o ha expirado.` and leaves the field retryable; `Volver` restores credentials; a valid code reaches `/dashboard`. The smoke script does not cover MFA.
- Manual signed-in check on `/login`: `Sesión iniciada`, name, email, `Panel`, `Cerrar sesión`, no filled Primary; `Cerrar sesión` lands on `/` and a subsequent `/login` shows the signed-out form.
- Desktop offline with and without a persisted session: the signed-in card and the transport-error form behave as §4.3; device registration shows no UI and never blocks the dashboard push.
- `caveman-blocks run markdown-table-check --path context/screens/done/AuthPage.md --strict` passes.

- Do not add: password reset or a "forgot password" link; email verification or resend; SSO, OAuth, magic link, or social sign-in; role, clinic, or invitation selection; a remember-me checkbox; terms, privacy, or consent acceptance; a show-password/reveal toggle; a password confirmation field; a strength meter; a client-side email regex or eight-character check; a native form submit; an MFA countdown, attempt limit, resend, or local expiry decision; a device list, device name, registration status, or retry control; a profile-completion prompt; a second filled Primary; a link to `/settings/billing`; billing, price, trial, plan, or `sync_locked` copy; a signature, stamp, or ink image; a rich-text or `contenteditable` surface; a patient field, PHI, or patient example; a new route, endpoint, table, column, role, or permission; storing the ticket, email, or password in browser storage or the URL; auto-redirecting a signed-in visitor away from `/login`.

## 8. Spec references

| Spec | Connection |
|---|---|
| `specs/done/15-O-ui-tokens-actions-and-formatting.md` | `15-O` supplies the tokens, `Button`, and format helpers the auth forms use. |
| `specs/done/16-P-ui-forms-feedback-and-layout.md` | `16-P` supplies `PageShell`, `PageHeader`, `Card`, `FormGrid`, `TextField`, `DetailGrid`, `ErrorText`, `HintText`, and `LoadingState` the sign-in, sign-up, and MFA forms render. |
| `specs/done/20-T-shell-auth-home-and-verify.md` | `20-T` adopts `AuthPage`: `PageShell id="auth-page"`, keyboard-usable sign-in/sign-up switch, es-EC copy, MFA-only challenge state with one Primary `Verificar código`. |
| `specs/done/25-Y-ui-legacy-css-cleanup-and-docs.md` | `25-Y` removes the legacy auth-page selectors so the adopted markup runs only on `.ui-*` classes. |
