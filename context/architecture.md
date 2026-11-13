# PersonaSanus — Architecture

| Field | Value |
|---|---|
| Source | `context/PRD.md` (Draft v0.8 — 2026-11-12) §5.4, §5.8, §7–§9, `context/database-design.md`, `context/sitemap.md` |
| Baseline | Working tree 2026-11-11 — M0 auth implemented on the Dioxus fullstack base |
| Runtime | Dioxus 0.7.1 requirement, locked 0.7.10 (`router`, `fullstack`) · axum 0.8 · Diesel 2.3 (`sqlite`, `r2d2`) · Argon2 0.6 · `qrcode-generator` 6.1.2 (`std`, `qr`, `image`) for QR PNG output · `printpdf` 0.12.9 (`default-features = false`, `text_layout`, `jpeg`, `png`) |
| Topology | One fullstack node: server binary + wasm bundle; one SQLite file on local disk |
| Datastore | `app.db` locally · `/data/app.db` in Docker (`persona-data` volume) |
| Public surface | `:8080` — Dioxus pages, `/api/auth/*`, `/health`; `⬜ /verify/:code`, ✅ `/api/prescriptions/:id/pdf`, ✅ `/api/prescriptions/:id/preview` |
| Build | `dx build --release --fullstack` → `target/dx/persona-sanus/release/web/{server,public}` |
| Status key | ✅ Implemented · 🟡 Partial · ⬜ Planned |

This document records how the PRD, sitemap, and database design are realized in the `persona-sanus` crate, and what is missing before v1. It adds no product scope.

## 1. System shape

The server process is the unit of deployment. There is no separate API service, message broker, or second clinical datastore. Every component that must be transactional with clinical data runs in the same process and writes the same SQLite file (PRD §7, DB §1).

| Area | Status | Evidence |
|---|---|---|
| Fullstack shell: SSR, hydration, assets, `/` `/login` `/dashboard` | ✅ | `src/main.rs`, `src/server.rs` |
| App-owned auth: sign-up, sign-in, get-session, sign-out | ✅ | `src/auth_server.rs`, `src/db.rs` |
| Session cookie, SHA-256 token storage, 30-day expiry | ✅ | `src/auth_server.rs` |
| SQLite pool + embedded migrations applied at boot | ✅ | `src/db.rs` |
| Auth schema (`users`, `sessions`) | ✅ | `migrations/20261111000000_create_auth` |
| App schema (`profiles`, `devices`, `assets`, clinical tables, `audit_log`; billing is v1.1) | ✅ | `migrations/20261112000000_create_app_schema` |
| Roles, clinic membership, server-side authorization | ✅ | `src/service.rs`, `clinic_members` |
| Patients | ✅ | `src/service.rs`, `/api/patients*` |
| Templates, versions, publish/archive | ✅ | `src/service.rs`, `/api/templates*` |
| Prescriptions, items, verification | ✅ | PRD §5.4–§5.6 |
| Templates forms and inputs (forms on web + desktop, online and offline, D-21) | ✅ | `src/forms.rs` |
| A4 PDF + QR | ✅ | `src/pdf.rs`, `src/pdf_server.rs`, PRD §5.4 |
| Desktop cache + outbox + sync | ⬜ | PRD §5.5 |
| Billing + scheduler (v1.1) | ⬜ | PRD §5.7, D-04 |
| Image library (1 image in v1; MinIO, 15-min signed URLs) | ✅ | `src/assets.rs`, `/api/assets*` |
| Branding asset upload (logo only) | ✅ | `/api/clinic/logo`, `/settings/clinic` |
| Audit, MFA, rate limiting, backups | ⬜ | PRD §9, §11 M3 |

```mermaid
flowchart LR
    subgraph clients["Clients"]
        WEB["Web · Chromium<br/>wasm bundle"]
        DESK["Desktop · macOS/Windows<br/>WebView · 🟡"]
    end

    subgraph node["persona-sanus node :8080"]
        ROUTER["axum Router"]
        SSR["Dioxus SSR + hydration<br/>App · Router"]
        AUTH["/api/auth/*<br/>Argon2 · opaque sessions"]
        HEALTH["/health"]
        SVC["Authorized service layer<br/>⬜ M0"]
        JOBS["In-process scheduler<br/>⬜ M3"]
        POOL["Diesel r2d2 pool<br/>max_size = 8"]
    end

    DB[("SQLite<br/>app.db")]
    OBJ[["MinIO private bucket<br/>images · 15-min signed URLs<br/>⬜ M1"]]
    PAY["PayPhone Tokenization<br/>⬜ v1.1"]

    WEB -->|HTTPS| ROUTER
    DESK -->|HTTPS| ROUTER
    ROUTER --> SSR
    ROUTER --> AUTH
    ROUTER --> HEALTH
    ROUTER --> SVC
    AUTH --> POOL
    SVC --> POOL
    JOBS --> POOL
    POOL --> DB
    SVC -. "upload / signed URLs" .-> OBJ
    JOBS -. "collection + status" .-> PAY
    JOBS -. "orphan cleanup" .-> OBJ
```

## 2. Compile-time split (Cargo features)

| Feature | Pulls in | Produces | Used by |
|---|---|---|---|
| `default = ["web", "server"]` | both below | fullstack dev/prod build | `dx serve`, Docker |
| `web` | `dioxus/web`, `dioxus-motion` (`web`) | wasm client | browser |
| `server` | `dioxus/server`, `axum`, `tokio` (`macros`, `rt-multi-thread`, `net`), `diesel` (`sqlite`, `r2d2`), `diesel_migrations`, `libsqlite3-sys` (`bundled`), `dioxus-motion` (`dioxus`, no `web`), `argon2`, `sha2`, `hex`, `uuid` (`v4`, `v7`), `chrono` (`clock`), `anyhow`, `qrcode-generator`, `printpdf` (`default-features = false`, `text_layout`, `jpeg`, `png`) | native server binary | node |
| `desktop` | `dioxus/desktop`, `dioxus-motion` (`desktop`), `qrcode-generator`, `printpdf` (`default-features = false`, `text_layout`, `jpeg`, `png`); ⬜ M2 adds `diesel` (`sqlite`, `r2d2`) + `libsqlite3-sys` (`bundled`) + `reqwest` (`cookies`) for the local cache and sync | native WebView app | macOS/Windows |
| `mobile` | `dioxus/mobile` | declared, not v1 | — |

`reqwest` and `serde_json` are unconditional and compile on both sides.

Rules:

- Server modules are gated in `src/main.rs`: `auth_server`, `db`, `schema`, `server`, and `pdf_server` compile under `feature = "server"`; `pdf` compiles under `server` or `desktop`. The non-server `main` launches `dioxus::launch(App)`; the server `main` is `#[tokio::main]`.
- `printpdf` never enters the `web` feature: the wasm bundle must not carry a PDF engine. `default-features = false` drops printpdf's `html` layout (azul CSS/XML stack); `text_layout` is still required because font embedding goes through it, and `jpeg`/`png` add the image decoders used for logos, library images, and the QR PNG. Never call `PdfDocument::from_html`.
- `qrcode-generator` 6.1.2 ([upstream](https://github.com/magiclen/qrcode-generator), MIT) is the only QR encoder. It is optional, enabled by `server` and `desktop`, and uses `default-features = false` plus `std`/`qr`/`image` so the renderer can call `Renderer::to_png_vec()`. It never enters `web`: QR symbols are generated where the PDF is rendered and embedded as PNG; the payload is `{PUBLIC_BASE_URL}/verify/{code}` with no PHI. Do not add a second QR crate or hand-roll the matrix.
- `dioxus-motion` 0.4 ([upstream](https://github.com/wheregmis/dioxus-motion), MIT) is the typed, interruptible animation crate for Dioxus UI (springs, tweens, keyframes, presence, route transitions). It is an optional dependency that gets exactly one platform feature per build: `web` on wasm, `desktop` on the WebView app, `dioxus` alone for SSR. Its default `web` feature would drag `wasm-bindgen`/`web-sys` into the native server binary, hence `default-features = false`. Use the crates.io release, not the repository `main` branch, which carries unreleased breaking API changes.
- Text entry is plain Dioxus forms and inputs only (PRD D-21) — no WYSIWYG editor, no rich text, no editor dependency, no JS bridge, no contenteditable, and no target gate: single-line inputs, multi-line textareas, selects, and check controls carry plain-text values only (no formatting), and the same `src/forms.rs` components compile on `web`, `desktop`, and server-side render, so the desktop binary does the same plain-text entry offline against the local cache/outbox as the web does online. The forms emit schema-checked structured JSON (`doc_json`) and never produce raw HTML.
- Diesel, Argon2, and axum types must never appear in shared signatures. `SessionUser` is a plain struct in `src/auth.rs` precisely so the client can hold session state without pulling in the server stack.
- `libsqlite3-sys` uses `bundled`: SQLite is compiled with the crate, so the runtime image needs no system `libsqlite3`.
- `uuid` `v7` supplies sortable entity ids; `v4` supplies session-token entropy (DB §1).
- The Dockerfile consumes the fixed Dioxus output path; changing the build profile or crate layout requires updating it.

## 3. Module map

| Module | Side | Owns | Must not |
|---|---|---|---|
| `src/main.rs` | shared | `Route` enum, `App`, `Navbar`, `AuthPage`, `Dashboard`, server entry point | contain SQL, crypto, or HTTP handlers |
| `src/auth.rs` | client | auth HTTP calls, `SessionUser`, response parsing | import server modules |
| `src/forms.rs` | client (shared) | template form component, A4 preview projection reused by prescriptions, `doc_json` (de)serialization and validation, variable/medication/image fields | contain server code; persist or emit raw HTML |
| `src/auth_server.rs` | server | handlers, validation, Argon2, tokens, cookies, `with_conn` | build UI or HTML |
| `src/db.rs` | server | pool, embedded migrations, SQL, models | emit HTTP responses |
| `src/assets.rs` | server | S3/SigV4 client, private-bucket readiness, image decode/EXIF strip, presigned GET URLs | build UI; hold a database transaction across network I/O |
| `src/pdf.rs` | server + desktop | `printpdf` renderer: layout model → page ops, fonts, images, QR PNG via `qrcode-generator`, A4 page boxes; pure, deterministic, no I/O | SQL, HTTP, UI, `from_html` |
| `src/pdf_server.rs` | server | `GET /api/prescriptions/:id/pdf`, authorization, `Content-Disposition` filename, load pinned version + variables | layout or drawing logic |
| `src/schema.rs` | server | Diesel `table!` definitions | drift from migrations |
| `src/server.rs` | server | axum composition, `AppState`, `/health` | query the database directly |
| `migrations/` | server | schema history | be edited after release |

Dependency direction: UI/`auth` → HTTP; `server` → `auth_server` → `db` → `schema`. No upward imports. Extend this seam rather than adding ad-hoc pools, routers, or SQL from handlers: `server::router` wires state, `auth_server` speaks HTTP, `db` is the only place with SQL.

Planned server components keep the same split rather than new top-level patterns:

| Component | Responsibility | Gate |
|---|---|---|
| Authorized service layer | mutations/reads with session, role, clinic membership (entitlement is v1.1) | every app write |
| Sync API | idempotent outbox drain, cache reconciliation, push channel | v1: none; v1.1: `can_sync` |
| Billing (v1.1) | PayPhone tokenization client (`ctoken` capture, `/api/transaction/web` charges), consent, entitlement transitions | administrator |
| Jobs | v1: retention (on in v1, D-12), session purge, asset cleanup; v1.1: trial, renewal, polling, dunning | server-only, single node |
| Assets | image library 1 per clinic in v1 (5/1/0 in v1.1), logo excluded, validation, quota, 15-min signed URLs | role + quota (v1); clinic state (v1.1) |
| PDF renderer | `printpdf` A4 rendering from the pinned `doc_json` + variables, shared by server and desktop | server + desktop |

## 4. Startup and request lifecycle

Boot order:

1. `dioxus::cli_config::fullstack_address_or_localhost()` resolves the bind address. Defaults are `127.0.0.1:8080`; `IP` and `PORT` override them (Compose sets `IP=0.0.0.0`).
2. `server::router` reads `PERSONA_COOKIE_SECURE` (`1` or case-insensitive `true`).
3. `db::connect` reads `DATABASE_URL` (default `sqlite://app.db`), creates the parent directory, and builds an r2d2 pool with `max_size = 8` and a connection customizer that sets the SQLite pragmas on every acquire.
4. Embedded migrations run on a pooled connection before any request is served; a migration failure aborts startup.
5. `AppState { pool, cookie_secure }` is built; auth routes take the state; `/health` reads `PRAGMA foreign_keys` and `PRAGMA journal_mode` from a pooled connection and returns `{"ok":true,"foreign_keys":true,"journal_mode":"wal"}`; the Dioxus app is mounted with `ServeConfig::new()`.
6. `axum::serve` uses `into_make_service_with_connect_info::<SocketAddr>()`, preserving the peer address for future rate limiting and audit.

Per request:

- axum extracts the payload; the handler validates it.
- All request database work goes through `with_conn`: `spawn_blocking` → `pool.get()` → closure. The pool customizer has already applied the pragmas on acquire, so every path gets foreign keys, WAL, the busy timeout, and `synchronous = NORMAL`. Password hashing and verification use separate `spawn_blocking` tasks because they do not need a connection.
- `AuthError` maps validation to `400`, duplicate email to `409`, bad credentials to `401`, and everything else to `500` with `{"message": …}`.
- `get_session` is a read that writes: it deletes expired sessions before looking up the token. Under sustained traffic that serializes a write per session check; move purging to the scheduler once sync traffic lands.

## 5. Identity, sessions, auth API

| Method | Path | Credential | Request | Response |
|---|---|---|---|---|
| `POST` | `/api/auth/sign-up/email` | none | `name`, `email`, `password` | `200 {token, user}` + `Set-Cookie`; `400` validation; `409` duplicate |
| `POST` | `/api/auth/sign-in/email` | none | `email`, `password` | `200 {token, user}` + `Set-Cookie`; `401` |
| `GET` | `/api/auth/get-session` | `persona_session` cookie | — | `200 {session, user}` or `null` |
| `POST` | `/api/auth/sign-out` | `persona_session` cookie | — | `200 {success:true}` + expired cookie |
| `GET` | `/health` | none | — | `200 {"ok":true,"foreign_keys":true,"journal_mode":"wal"}` |

```mermaid
sequenceDiagram
    participant U as Browser / WebView
    participant H as axum / auth_server
    participant P as Diesel pool
    participant D as SQLite

    U->>H: POST /api/auth/sign-up/email
    H->>H: validate, Argon2 hash in spawn_blocking
    H->>P: transaction
    P->>D: SELECT users by email
    P->>D: INSERT users, INSERT sessions
    P-->>H: commit
    H-->>U: 200 token + user, Set-Cookie persona_session
    U->>H: GET /api/auth/get-session with cookie
    H->>P: DELETE expired sessions
    P->>D: SELECT sessions by token_hash, SELECT users by id
    H-->>U: 200 session + user
```

Rules:

- Entity ids are UUID v7 text. Email is trimmed and lowercased before storage; the column is `UNIQUE COLLATE NOCASE`.
- Passwords require at least 8 characters and are hashed with Argon2 defaults. `user_json` omits `password_hash`; the hash never leaves the server.
- A session token is two concatenated UUID v4 values (64 hex characters). Only `SHA-256(token)` is stored, under a unique index, so a database leak yields no usable token.
- Sessions expire after 30 days. The cookie is `persona_session; Path=/; HttpOnly; SameSite=Lax; Max-Age=2592000`, plus `Secure` when `PERSONA_COOKIE_SECURE` is true.
- Sign-out deletes only the presented session. There is no all-device revocation, session rotation on password change, email verification, password reset, MFA, lockout, or rate limiting (M0; M3 in PRD §11).

Security findings in the current implementation:

| Finding | Evidence | Impact | Action |
|---|---|---|---|
| Token returned in the JSON body as well as the cookie | `session_response` | an XSS can read a credential that `HttpOnly` otherwise protects | closed: `session_response` returns the user only and sets the `HttpOnly` cookie; the desktop client reads the token from `Set-Cookie` |
| Account enumeration | sign-up returns `409`; sign-in skips Argon2 for unknown emails | email discovery and timing signal | uniform sign-in errors plus a dummy hash and rate limiting (spec 12-L); sign-up keeps `409` — enumeration is inherent in account creation (accepted residual) |
| No CSRF token or `Origin` check | cookie auth with JSON `POST` | `SameSite=Lax` mitigates, but lacks defense in depth | closed: `server::origin_guard` rejects state-changing `Origin` headers outside `PERSONA_WEB_ORIGINS`, `x-forwarded-host`/`Host`, or the `dx serve` proxy's loopback devserver origin (spec 12-L) |
| No security headers or CORS layer | router composition | no CSP/HSTS surface; a browser-served second origin would fail without CORS | closed: CSP, `nosniff`, `Referrer-Policy`, `X-Frame-Options`, optional HSTS, and credentialed CORS (spec 12-L) |

## 6. Data access contract

Storage conventions follow DB §1: UUID v7 `TEXT` ids, RFC3339 UTC timestamps, `YYYY-MM-DD` dates, `TEXT` + `CHECK` enums, integer cents for USD, and booleans as `INTEGER 0/1`.

Connection rules:

1. **Foreign keys.** SQLite requires `PRAGMA foreign_keys = ON` per connection. `db::ConnectionPragmas`, an r2d2 `CustomizeConnection`, sets it on every acquire, so no query path can bypass it; `/health` exposes the effective value through the pool.
2. **Concurrency.** The customizer sets `journal_mode = WAL`, `busy_timeout = 5000`, and `synchronous = NORMAL`. `db::write_transaction` (deferred `BEGIN`) and `db::immediate_transaction` (raw `BEGIN IMMEDIATE`) retry `database is locked` with bounded attempts; allocation and quota paths must use `immediate_transaction`, never their own `BEGIN IMMEDIATE` or retry loop.
3. **Blocking work.** Diesel calls and Argon2 run in `spawn_blocking`. Never hold a pooled connection or a Dioxus signal lock across `.await`; `clippy.toml` already rejects the signal-lock case.
4. **Transactions.** Multi-row invariants belong in one transaction: sign-up (user + session), signing (rx block, prescription, items, verification code, audit), payment resolution (event + transaction + subscription, v1.1), and retention trim. Sign-up and prescription signing use the helper today. Use the `db` helpers instead of Diesel `transaction()` when the transaction must take the write lock up front.
5. **Migrations.** Append-only and embedded. Planned order: `create_auth` ✅ → app schema ✅ → `sessions.device_id` ✅ (spec 04-D) → billing ⬜ v1.1 (DB §8). Update `src/schema.rs` in the same change as each migration; startup applies pending migrations before serving.
6. **Scoping.** Clinic and role filters belong in the service layer, never in client-supplied query parameters. The dashboard and route guards are not an authorization boundary.
7. **Assets.** Uploads reserve a `pending` row and count it against the v1 quota (1 library image per clinic) in a `BEGIN IMMEDIATE` transaction, then upload and mark it `ready`; failures delete the row and object. Deletion checks `template_version_assets` and retained prescriptions before touching the object. The 5/1/0 state quota and the cancellation purge are v1.1 (PRD §5.8, D-05).

The connection pragmas live in the r2d2 customizer in `db.rs`; `database-design.md` matches that.

## 7. Client and route shell

`App` provides two context signals: `Option<SessionUser>` and a `checked` flag. A single `use_effect` calls `fetch_session` after mount. `Navbar` reads the session for sign-out; `Dashboard` redirects to `/login` with `navigator.replace` once the check completes and no session exists.

| Route | Layout | Guard | Status |
|---|---|---|---|
| `/` | `Navbar` | none | ✅ |
| `/login` | `Navbar` | none; signed-in users see their session | ✅ |
| `/dashboard` | `Navbar` | client-side redirect when signed out | ✅ |
| `/patients*` | `Navbar` | role- and session-dependent (sitemap §1.2) | ✅ (CRUD + prescription history) |
| `/templates*` | `Navbar` | doctor or administrator (sitemap §1.2) | ✅ |
| `/settings*` | `Navbar` | role- and session-dependent (sitemap §1.2) | ✅ (clinic image library in spec 06-F) |
| `/prescriptions*` | `Navbar` | role- and session-dependent (sitemap §1.2) | ✅ (forms and inputs, items, sign, void, supersede, A4 preview, PDF) |
| `/audit`, `/verify/:code` | target shell | role- and session-dependent (sitemap §1.2) | ⬜ |

Rules and current limits:

- The `#[layout(Navbar)]` on `Home` is never closed with `#[end_layout]`, so `Navbar` wraps every route, including `/` and `/login`. The sitemap separates public routes from the authenticated shell; add `#[end_layout]` and a second layout group when the shell gains nav sections and sync indicators.
- The redirect is client-side only. SSR still renders the page shell for an unauthenticated request, and any future PHI load must be authorized in the service layer behind the REST handler. Never rely on the redirect for data protection (PRD §3).
- The session fetch is a `use_effect`, so the first SSR paint is signed-out and flips after hydration. Data pages load through the authorized `/api/*` endpoints (`use_resource` or SSR-side fetch) so the first render carries the authorized result instead of an unauthenticated stub.
- UI code must copy signal values before awaiting. Holding `WriteLock` or `GenerationalRef` across `.await` is flagged by the `clippy.toml` configuration.
- No JavaScript interop: text entry is plain Dioxus forms and inputs (PRD D-21) — no WYSIWYG editor, no rich text, no bridge, no contenteditable. Single-line inputs, multi-line textareas, selects, and check controls hold plain-text values only, online and offline. The forms emit schema-checked structured JSON (`doc_json`) and raw HTML is never persisted or rendered.

Auth transport is resolved in spec 04-D:

| Runtime | Behavior | Result |
|---|---|---|
| Web on the same host and port | relative `/api/*` base; `fetch_credentials_include` | works on the development machine and behind TLS |
| Web behind TLS or a domain | relative base | same-origin requests preserve the cookie |
| Desktop (native) | `reqwest` cookie store; `PERSONA_SERVER_ORIGIN` | the session cookie is stored and sent, so auth persists |
| Remote desktop | configurable origin | calls the configured server, not a loopback constant |

Decision: application data is exposed as REST under `/api/*` with the session cookie (chosen — specs 02-B/04-D). The client splits the base URL (relative on wasm, configurable origin + cookie jar on native), the session cookie stays `SameSite=Lax`, and the server allows credentialed CORS for the web origins listed in `PERSONA_WEB_ORIGINS` (native `reqwest` is not a browser and needs no CORS). The server stays authoritative: every read and write is authorized in the service layer. This must be in place before shipping desktop or terminating TLS.

## 8. Target domains and their contracts

| Domain | Server contract | Stores | Gate | Status | PRD |
|---|---|---|---|---|---|
| Profile and membership | session → profile, role, clinic membership, doctor details | `users`, `sessions`, `profiles`, `clinic_members`, `doctors` | session | 🟡 identity only | §3, DB §2 |
| Patients | CRUD, cédula dedupe, soft delete | `patients` | role + clinic | ✅ | FR-001–003 |
| Templates | draft, publish immutable version, pin assets | `templates`, `template_versions`, `template_version_assets`, `assets` | doctor owner or administrator | 🟡 forms and inputs + publish; asset pinning spec 06-F | FR-020–024 |
| Images | upload/delete, v1 quota (1), 15-min signed URLs, template pinning, placeholder rendering (v1.1) | `assets`, `template_version_assets` | role + quota | ⬜ | FR-080–085 |
| Prescriptions | draft→signed→superseded/voided, ordered items, block-allocated number | `prescriptions`, `prescription_items`, `rx_counters`, `rx_blocks` | doctor signs; nursing drafts; secretary reads/reprints; administrator reads clinic-wide | ✅ | FR-010–014 |
| Verification | opaque code → public status, no PHI | `verification_codes` | public, rate limited | ⬜ | FR-050 |
| Audit | append-only actor/action/entity and sensitive reads | `audit_log` | administrator read | ⬜ | PRD §6 |
| Sync | idempotent mutations, block reconciliation, push | client cache + outbox | v1: none; v1.1: `can_sync` | ⬜ | FR-040–043 |
| Billing (v1.1) | plan, trial, subscription, transactions, events | billing tables (not in v1) | administrator | ⬜ v1.1 | FR-060–070 |

`users` is auth-owned. The app profile is `profiles`, linked by `auth_user_id`, because the two names collide in PRD §6 (DB §1). UI labels such as "Users" map to profiles plus `clinic_members`, not to the auth table.

## 9. Offline sync contract

```mermaid
flowchart TD
    A["Draft created offline<br/>local cache"] --> B["Sign locally<br/>consume rx_number from active block"]
    B --> C["outbox row pending<br/>payload + client_id + rx_block_id"]
    C --> D{"online?"}
    D -- no --> C
    D -- yes --> E["server function<br/>client_id"]
    E --> F{"number belongs to a block<br/>issued to this device and unused?"}
    F -- no --> G["reject: duplicate or foreign number<br/>outbox kept, user notified"]
    F -- yes --> H["one transaction:<br/>prescription · items · verification_code<br/>rx_block consumed · audit"]
    H --> I["ack: id, rx_number, signed_at, code"]
    I --> J["cache synced_at set<br/>QR active · final PDF"]
    G -. fix / retry .-> E
```

Required server behavior:

- Every outbox row carries the client-generated id. A replay must return the existing record, not duplicate it.
- `prescriptions.client_id` is the only modeled idempotency key (DB §5). Patient create and template publish have no equivalent column, so a replay duplicates rows; an update is naturally idempotent when it carries the full state. Add a `client_id` (or request id) before M2, or make the server derive a stable key from the outbox payload.
- Drain oldest-first with `attempts`, `next_retry_at`, and a visible failure state. Never discard the outbox; a rejected row stays visible until resolved.
- Offline signing consumes the next number in the device's active `rx_blocks` row; on sync the server validates block ownership and non-reuse before accepting the write. A web client registers a `devices` row with `platform='web'` and allocates a block the same way. Burned block tails remain gaps (D-02).
- v1 has no entitlement gate. v1.1 re-evaluates `can_sync` on every mutation; reads of retained data stay allowed, and `locked`/`canceled` reject writes with `402 sync_locked`.
- Server wins for signed data. Only an in-progress draft prompts for a conflict choice.
- Cache eviction applies only to server-backed rows older than `clinics.retention_days` (default 30). Rows with no server copy are never evicted.
- `pending_sync` is derived from the outbox, never a server column. Its PDF carries `PENDIENTE DE SINCRONIZACIÓN` and no active QR until the upload is reconciled.

Sync pushes over SSE from the existing axum router, with a 30-second polling fallback (PRD D-11). Do not add a second socket server.

## 10. Images and storage (1 image in v1; state-based in v1.1)

In v1 the library holds one 10 MB image per clinic; the clinic logo is a separate branding asset and does not count. The v1.1 billing entitlement restores the state-based library below: 5 images while `active` or `past_due`, 1 while `trialing` or `locked`, and 0 while `canceled`.

| Entitlement | Library uploads | Rendering |
|---|---|---|
| `trialing` | 1 | 1 library image + branding |
| `active` | 5 | all + branding |
| `past_due` within grace | 5 | 1 library image + branding; the rest show a placeholder |
| `locked` | 0 | 1 retained library image + branding |
| `canceled` | 0 | placeholders only; assets are soft-deleted |

| Rule | Value | Enforced at |
|---|---|---|
| Images per clinic | v1: 1; v1.1: 5 for `active`/`past_due`, 1 for `trialing`/`locked`, 0 for `canceled` (non-deleted `kind='image'` rows) | service transaction (count + insert) |
| Size per image | 10 MB (10,485,760 bytes) | stream cap + DB CHECK |
| Formats | JPEG, PNG (WebP and SVG rejected) | magic-byte sniff + decoder |
| Branding assets | logo only, excluded from the quota (no signature/stamp — D-06) | service |
| Storage key | `clinics/{clinic_id}/images/{asset_id}` | upload service |
| Access | private MinIO bucket; 15-minute signed GET URLs | bucket policy + signer |

```mermaid
sequenceDiagram
    participant U as Client
    participant S as Server
    participant D as SQLite
    participant B as MinIO bucket

    U->>S: upload image, at most 10 MB
    S->>S: session, role, quota check (v1: 1)
    S->>D: BEGIN IMMEDIATE, count image assets
    alt quota full
        S-->>U: 409 image_quota_exceeded
    else slot available
        S->>D: insert asset pending
        S->>S: sniff MIME, decode, strip EXIF, sha256
        S->>B: PUT object
        S->>D: mark ready
        S-->>U: 201 asset_id + signed GET URL
    end
```

Rules:

- v1 allows 1 library image per clinic; `active`/`past_due` (5), `trialing`/`locked` (1), and `canceled` (0) are the v1.1 gate. `locked`/`canceled` never accept uploads, and from `past_due` only the logo and one image render (v1.1).
- Uploads are server-proxied in v1: the 10 MiB cap, magic-byte MIME check, image decode, and EXIF/GPS strip happen before the object is marked `ready`. No presigned browser PUT.
- Reserve a `pending` row in a `BEGIN IMMEDIATE` transaction before the PUT, so the state quota cannot be raced. A failed upload deletes the row and object; a cleanup job removes `pending` rows older than one hour.
- The MinIO bucket is private, served only through 15-minute signed GET URLs. Object keys and URLs carry no PHI.
- Publishing a template writes `template_version_assets`. Deleting an image is blocked while any published version or retained prescription references it; delete the object last. The `canceled` soft-delete and object purge are v1.1 and leave pins intact with placeholder rendering (D-05).
- Offline rendering caches images pinned by published templates, verifies SHA-256, and caps each cached asset at its 10 MiB upload cap (logo + library ≤ 20 MiB in v1; 60 MiB at the v1.1 five-image quota). Never evict an image still referenced by a cached template.
- Uploads are online-only in v1: the desktop caches existing images but does not queue image files in the outbox.
- Upload and delete (including the cancellation purge) write `audit_log` rows (`asset_upload`, `asset_delete`) with metadata only.

## 11. PDF, QR, verification

Pipeline:

1. `GET /api/prescriptions/:id/pdf`: authorize the session, role, and clinic membership. `draft` is rejected (`422`); signed, superseded, and voided documents render. Provisional (`pending_sync`) documents exist only in the desktop cache (no server row), so the route never serves them; the desktop renders them locally.
2. Load the pinned `template_versions.doc_json`, the pinned `template_version_assets`, the variable set (patient, doctor, clinic, prescription, medications), and the stored plaintext `verification_codes.code` (reprints reuse it, so the issued QR stays valid). Never the current template (FR-024).
3. `pdf::render` builds a `PdfDocument` and one `PdfPage` per A4 sheet (210×297 mm) with explicit ops: embedded TTF text runs, rules, image XObjects, and the QR PNG. Page breaks, repeated header/footer, and margins are computed by the renderer — there is no CSS engine.
4. The QR encodes `https://<domain>/verify/{code}`, generated by `qrcode-generator` (`qr::Encoder` → `Renderer::to_png_vec()`), and is embedded as a PNG XObject; the payload carries no PHI (FR-033). A provisional document draws a placeholder frame plus `PENDIENTE DE SINCRONIZACIÓN` and no scannable code.
5. `save(&PdfSaveOptions::default(), &mut warnings)` returns the bytes. Web: the handler sends `application/pdf` with `Content-Disposition: attachment; filename="{Patient}-receta-{DD-MM-AAAA}-{Doctor}.pdf"` (FR-032). Desktop: the same `pdf::render` call runs in-process and writes the file locally — the route is not needed offline.
6. Regeneration uses the pinned version and the same renderer build, so a reprint reproduces the issued content (not guaranteed byte-identical across renderer upgrades).

Constraints:

- One layout model (`doc_json`) for the forms, the preview, and the PDF. The preview is the browser projection; the PDF render is authoritative. Fix fidelity in `src/pdf.rs`, never with a second template.
- `src/pdf.rs` is pure: no SQL, no filesystem, no clock beyond the passed issue timestamp. Input is the layout model + pinned asset bytes + variables, so desktop and server produce the same document and the golden fixture stays stable.
- The runtime image (`debian:bookworm-slim`) gets no Chromium and no host fonts: the renderer embeds TTF bytes (`include_bytes!` or bundled assets), so output does not depend on the host's font set.
- PDFs are never stored. There is no `pdf_blob` column; the route streams a fresh render from immutable data.
- Rendering is CPU-bound: call it inside `spawn_blocking`, never on the reactor.
- The public verification page will be unauthenticated and rate-limited. Its response is limited to status, doctor, date, and prescription number; "not found" means no matching `code_hash` row.

## 12. Billing and entitlement (v1.1 — not in v1)

**Not implemented in v1.** v1 ships with no billing tables, entitlement gate, or scheduler; retention dates live on `clinics` (`server_retention_until`, `data_grace_ends_at`). The design below is the v1.1 plan (PRD D-04).

- One subscription per clinic, created with the clinic. The trial is set at creation; retention boundaries live only on `clinics` and are not user-editable (FR-061).
- Plan amounts are seeded tax-inclusive; the IVA rate is effective-dated in `tax_rates`, and each transaction snapshots `tax_rate_bps`, `net_cents`, and `tax_cents` (DB §8). Later price or IVA changes never rewrite history.
- Checkout (first charge) inserts a `payment_transactions` row whose UUID v7 is the PayPhone `clientTransactionId`, then redirects to the PayPhone payment button/box (Visa/Mastercard) with the plan amount, reference, and an HTTPS `responseUrl`. PayPhone returns `id`, `clientTransactionID`, and `ctoken`; the server confirms the charge and stores the token after consent (`payment_methods`). Renewals POST `/api/transaction/web` with the stored `ctoken` and an AES-256-CBC `cardHolder`. PayPhone states tokenization is not automatic recurring — the app schedules charges.
- `can_sync` is derived from status plus time, not a mutable flag. It allows `trialing` before `trial_ends_at` and `active`/`past_due` before the grace deadline. `locked` and `canceled` deny writes with `402 sync_locked`.
- Payment state never gates clinical work: patients, templates, editing forms, PDF/print, local history, already-issued QR verification, auth, and public verification remain available.
- One in-flight pending transaction per subscription. Renewals are tokenized charges; a decline gets a 7-day `past_due` grace with retries on days 0, +2, and +4 before `locked`. Only Visa and Mastercard are eligible, and explicit consent is required before storing the token.
- After a lapse, data beyond the free window is exportable for 30 days before the trim. The trim job runs by default (D-12) and never deletes the first 3 months; counsel sign-off remains a production prerequisite (PRD §9).
- The image library is state-based: `active` uploads 5 and renders all; `past_due` uploads 5 but renders 1; `trialing` uploads 1; `locked` uploads none but renders 1; `canceled` uploads none and soft-deletes assets.
- v1.1 checkout work is gated on PayPhone authorizing the merchant for tokenization (application + review) and accepting the 10%/6-month balance hold.
- Same-day API Reverse is automated; later refunds are manual by the Administrator and audited (`payment_transactions.status='reversed'`).

```mermaid
sequenceDiagram
    participant ADM as Administrator
    participant UI as Settings / billing
    participant S as Server
    participant PP as PayPhone

    ADM->>UI: select plan, accept token storage
    UI->>S: start checkout
    S->>S: insert pending transaction, id = clientTransactionId
    S-->>UI: redirect to PayPhone payment button/box
    UI->>PP: pay with Visa/Mastercard
    PP-->>S: responseUrl with id, clientTransactionID, ctoken
    S->>PP: confirm charge
    PP-->>S: statusCode 3 approved + cardholder data
    S->>S: store payment_method + payment_events + subscription active
    S-->>UI: entitlement active, outbox drains
    Note over S,PP: Renewals POST /api/transaction/web with the stored ctoken
```

## 13. Jobs and single-node operations

| Job | Trigger | Writes | Idempotency | Status |
|---|---|---|---|---|
| Asset cleanup | periodic | delete `pending` assets older than one hour and their objects | age + object key | ⬜ |
| Trial expiry (v1.1) | scheduler | `subscriptions` trialing→locked, audit | compare timestamps | ⬜ v1.1 |
| Renewal request (v1.1) | `next_charge_at` | pending transaction + tokenized charge (`/api/transaction/web`) | one pending per subscription | ⬜ v1.1 |
| Payment confirmation (v1.1) | 60 s while pending | `payment_events`, activation | event hash + status | ⬜ v1.1 |
| Dunning (v1.1) | days 0/+2/+4, then lock | `past_due` → `locked` | status transitions | ⬜ v1.1 |
| Retention trim | after `data_grace_ends_at` | delete beyond the 3-month window | recompute deadline | ⬜ (on in v1, D-12) |
| Session purge | periodic | delete expired sessions | by `expires_at` | 🟡 lazy on get-session |
| Backup | daily | encrypted SQLite copy | date-stamped | ⬜ ops |

Single-node invariant: in-process scheduler + SQLite + a filesystem volume means exactly one writer replica. Two replicas would split the database and could double-charge a clinic. Horizontal scaling requires an external database and job locking, which is out of v1 scope. Compose restarts the container (`unless-stopped`) and probes `/health` every 5 seconds; TLS must terminate outside the container and is not shipped in Compose.

## 14. Configuration

| Variable | Default | Scope | Notes |
|---|---|---|---|
| `DATABASE_URL` | `sqlite://app.db` | server | Compose uses `sqlite:///data/app.db` |
| `PERSONA_COOKIE_SECURE` | `false` | server | `1` or `true`; enable behind TLS |
| `IP` | `127.0.0.1` | server | Compose sets `0.0.0.0` |
| `PORT` | `8080` | server | Compose maps `8080:8080` |
| `PUBLIC_BASE_URL` | — | server | ⬜ absolute base for QR/verification links, e.g. `https://rx.example.com` (specs 08-H, 09-I) |
| `PAYPHONE_TOKEN` | — | server | ⬜ v1.1 secret, never shipped to clients |
| `PAYPHONE_STORE_ID` | — | server | ⬜ v1.1 |
| `PAYPHONE_CODING_PASSWORD` | — | server | ⬜ v1.1 AES-256-CBC key for `cardHolder` |
| `RETENTION_TRIM_ENABLED` | `true` | server | enforced in v1 (D-12); counsel sign-off before production |
| `UPDATE_MANIFEST_URL` | — | desktop | signed auto-update manifest (D-08) |
| `ASSETS_BUCKET` | — | server | ✅ private bucket name |
| `ASSETS_ENDPOINT` | — | server | ✅ S3-compatible endpoint |
| `ASSETS_REGION` | `us-east-1` | server | ✅ |
| `ASSETS_ACCESS_KEY_ID` | — | server | ✅ secret, never shipped |
| `ASSETS_SECRET_ACCESS_KEY` | — | server | ✅ secret, never shipped |
| `ASSETS_SIGNED_URL_TTL_SECONDS` | `900` | server | ✅ 15-minute GET URLs |
| `ASSETS_PUBLIC_ENDPOINT` | `ASSETS_ENDPOINT` | server | ✅ browser-reachable base for signed URLs (Compose: host-published MinIO port, not `http://minio:9000`) |
| `PERSONA_WEB_ORIGINS` | `` (same-origin only) | server | ✅ comma-separated web origins allowed by credentialed CORS (spec 04-D); the same-origin fallback compares the request `Origin` against `x-forwarded-host`/`Host` (proxy-aware, so a TLS terminator that rewrites `Host` still passes), and a request proxied by `dx serve` may carry a loopback `Origin` on `DIOXUS_DEVSERVER_PORT` because the dev proxy rewrites `Host` to the backend's ephemeral port |
| `PERSONA_SERVER_ORIGIN` | `http://127.0.0.1:8080` | client | native API origin; the wasm client always uses relative paths (spec 04-D) |
| `VERIFY_RATE_LIMIT_PER_MINUTE` | `10` | server | ⬜ public verification per-IP limit (spec 09-I) |
| `AUTH_RATE_LIMIT_PER_MINUTE` | `20` | server | ⬜ auth-endpoint per-IP limit (spec 12-L) |


MinIO runs as a Compose service; `ASSETS_ENDPOINT` points at it and the bucket stays private. The LLM URL is per clinic in `clinics.llm_url_template` (DB §5), not a process environment variable. PRD FR-012 agrees (`clinics.llm_url_template`); seed it from configuration if an environment default is wanted.

## 15. Security boundaries

| Boundary | Control | Status |
|---|---|---|
| Image upload | magic-byte JPEG/PNG, decode limits, EXIF strip, private MinIO bucket, 15-min signed URLs, v1 quota 1 × 10 MB (v1.1: 5/1/0) | ✅ spec 06-F |
| Browser ↔ node | TLS outside the container; `Secure` cookie flag | 🟡 flag configurable, TLS is external and absent from Compose |
| Session | `HttpOnly`, `SameSite=Lax`, SHA-256 token hash | ✅ |
| CSRF | `SameSite=Lax` + JSON handlers + `Origin` guard on `POST`/`PUT`/`PATCH`/`DELETE` (`PERSONA_WEB_ORIGINS`, forwarded `Host`, or the `dx serve` loopback devserver origin) | ✅ |
| Authorization | server-side clinic/role checks in the service layer | ⬜ |
| PHI minimization | no PHI in URLs, QR, logs, or provider payloads | 🟡 design only |
| PHI at rest | SQLite file and desktop cache; OS disk encryption required (D-07) | ⬜ ops |
| Secrets | `PAYPHONE_TOKEN`, `PAYPHONE_STORE_ID`, `PAYPHONE_CODING_PASSWORD` (v1.1) | ⬜ |
| Card token | `ctoken` stored after consent; AES-256-CBC `cardHolder`; no PAN/CVV; revoked on cancel (v1.1) | ⬜ |
| Audit | sensitive reads; billing transitions are v1.1 | ⬜ |
| Abuse controls | rate limits on auth and `/verify/:code`; PayPhone polling is v1.1 | ⬜ |
| Device control and MFA | `devices` registered and revocable through `sessions.device_id`; MFA in M3 | 🟡 |

## 16. Invariants

| Invariant | Enforced at |
|---|---|
| Images: v1 1 per clinic; v1.1 5 `active`/`past_due`, 1 `trialing`/`locked`, 0 `canceled`; ≤10 MiB each; logo excluded | service + DB CHECK (✅) |
| Assets pinned by published versions are never deleted while referenced; the v1.1 cancellation purge soft-deletes and leaves pins | service + `template_version_assets` (✅) |
| One node, one SQLite file; the desktop cache is a replica, not an authority | deployment + `db.rs` |
| The server authorizes; client guards are UX only | service layer (⬜) |
| Replayed client mutations are no-ops | `client_id` / request keys (⬜) |
| Signed prescriptions are immutable; only void/supersede transition them | service layer (✅) |
| A supersede gets a new `rx_number` and verification code; the replaced row keeps its own and becomes `superseded` | transaction + `uq_prescriptions_rx` (✅) |
| The printed/QR verification code is recoverable for reprints (`code` + `code_hash`) | schema (⬜) |
| A nursing draft may lack a doctor; any non-draft row requires one | table CHECK + service layer (⬜) |
| `rx_number` is unique and monotonically increasing per clinic, assigned from a server-issued block and validated on sync; burned tails are recorded gaps | transaction + partial unique index + `rx_blocks` (⬜) |
| Money is integer cents; transactions snapshot the plan amounts (v1.1) | schema + service layer (⬜) |
| PHI never appears in a URL, QR, log, or provider payload | service/output layer (⬜) |
| The outbox is never discarded; only server-backed cache rows expire | desktop cache (⬜) |
| Foreign keys are enabled per connection; blocking work stays off the reactor | pool customizer ✅ |
| Migrations are append-only, embedded, and applied before serving | `db.rs` ✅ |

## 17. Risks and open decisions

| # | Risk | Evidence | Severity | Action |
|---|---|---|---|---|
| 1 | ~~Auth client targets loopback and does not carry cookies on desktop~~ resolved | `src/http.rs`, `PERSONA_SERVER_ORIGIN` | — | relative base on wasm, configurable native origin, cookie jar, and credentialed CORS (spec 04-D) |
| 2 | No server-side authorization; only a client redirect | `Dashboard` | P0 | authorize every read/write in the service layer before PHI pages |
| 3 | SQLite concurrency was unconfigured | ~~no WAL or busy timeout~~ resolved | — | pool customizer sets WAL/busy timeout/synchronous; `db` write helpers retry `database is locked` |
| 4 | Sync push transport | ~~undefined~~ resolved | — | SSE + 30 s polling fallback (PRD D-11); no second socket server |
| 5 | Idempotency exists only for prescriptions | `prescriptions.client_id` | P1 | add keys for patients/templates before sync |
| 6 | PDF rendering location | ~~undecided~~ resolved | — | `printpdf` in the server and desktop binaries (PRD D-01); shared `src/pdf.rs`; no `from_html` |
| 7 | Scheduler and SQLite assume one replica | in-process jobs | P1 | keep one node; document the scale-out prerequisites |
| 8 | Retention trim conflicts with health-record law | PRD §9 | P1 | enforced in v1 (D-12); export before trim; counsel sign-off before production |
| 9 | Account enumeration and no auth rate limiting | auth handlers | P2 | uniform errors, dummy hash, limits in M3 |
| 10 | No backups or observability | no job/backup code, no tracing layer | P2 | daily encrypted backup, structured logs, metrics |
| 11 | Destructive initial migration drops legacy auth tables | `20261111000000_create_auth/up.sql` | P2 | document that pre-existing accounts must sign up again |
| 12 | `users` name collision resolved as `profiles` | DB §1 | P1 | apply the rename in the first app-schema migration |
| 13 | Image quota race or orphaned objects | upload path not built | P1 | `BEGIN IMMEDIATE`, pending cleanup, bucket lifecycle |
| 14 | Malicious/oversized image or EXIF/GPS leakage | upload path not built | P1 | magic-byte/decode limits, EXIF strip, reject WebP/SVG, 10 MiB cap |
| 15 | Bucket credentials or public object exposure | no storage config yet | P1 | private MinIO bucket, least privilege, 15-minute signed URLs |
| 16 | Image deletion or cancellation breaks an issued PDF | template pinning not implemented | P2 | reference checks in v1; v1.1 cancellation accepts placeholders (PRD D-05) |
| 17 | IVA changes misstate historical charges (v1.1) | `tax_rates` effective-dated | P2 | snapshot `tax_rate_bps`, `net_cents`, and `tax_cents` per transaction |
| 18 | Device revocation not enforceable until `sessions.device_id` lands | `devices` modeled, auth schema unchanged | P1 | add the nullable column in the auth migration before M3 |
| 19 | Preallocated numbering gaps or block exhaustion | `rx_blocks` not built | P1 | block allocation, server validation on sync, remaining-count UI before signing |
| 20 | `printpdf` layout fidelity with manual page ops (no CSS engine), and an upstream crate whose HTML layout is experimental | no renderer module yet | P1 | golden A4 fixture diffed per PR; explicit ops only, never `from_html`; pin `printpdf` 0.12 and re-run the fixture on upgrade |
| 21 | Auto-update key compromise or bad release | no updater yet | P1 | signed manifest, key custody, staged rollout, rollback (PRD D-08) |
| 22 | PayPhone tokenization gated on merchant authorization; 10% of tokenized balances held 6 months; Visa/Mastercard only; `ctoken` is a bearer secret | `docs.payphone.app/tokenizacion` | P1 | apply before M4; consent + encrypted `cardHolder`; revoke on cancel; dunning on declines |

## 18. Verification

Repository policy is end-to-end and integration testing; unit tests are not required (AGENTS.md). Run these from the repository root; they use the existing blocks:

| Change | Gate |
|---|---|
| Any Rust or UI change | `persona-smoke` |
| Compose build, boot, health, auth persistence | `persona-docker-verify` |
| Auth endpoints and public pages | `persona-probe --flow full` |
| Server reachability | `wait-health` |
| Schema design or migration SQL | `sqlite-schema-check` |
| Image upload, quota, signed URL | `asset-probe` |
| PDF route and A4 output | end-to-end probe on `GET /api/prescriptions/:id/pdf`: `200`, `application/pdf`, `%PDF-` magic, `MediaBox` floats within ±1 pt of 595.28×841.89 (A4 in points), sanitized `Content-Disposition`, QR decodes to `/verify/`; promote to a block when implemented |

```bash
python3 .blocks/persona-smoke.py --dir persona-sanus --port 8091
python3 .blocks/persona-docker-verify.py --dir . --port 8080
python3 .blocks/persona-probe.py --base http://127.0.0.1:8080 --flow full
python3 .blocks/asset-probe.py --base http://127.0.0.1:8080
python3 .blocks/wait-health.py --url http://127.0.0.1:8080/health --expect 200 --timeout 60
python3 .blocks/sqlite-schema-check.py --path context/database-design.md --query "select name from sqlite_master where type = 'table' order by name"
```

`sqlite-schema-check` executes the DDL block in memory, so it validates the design before a migration exists; it does not inspect the live `app.db`.

## 19. Source-of-truth precedence and drift

When documents disagree, use this order:

| Topic | Authoritative source |
|---|---|
| Product behavior, roles, functional requirements | `PRD.md` |
| Tables, ids, retention, billing state | `database-design.md` |
| Routes and per-route roles | `sitemap.md` |
| What exists today | code + migrations |

Known drift to reconcile in implementation:

1. The sitemap places `/` and `/login` outside the authenticated shell; the current `Route` enum applies `Navbar` to all routes.
2. The `/dashboard` guard is client-side only, with no server middleware; server-side authorization is the service layer's job (spec 02-B).
3. The database design describes the full app and billing schema; only the auth migration exists, and billing is v1.1.
4. The PRD decision log (§14) supersedes earlier statements in this document and in the DB/sitemap about PDF rendering, billing scope, numbering, roles, image quota, verification codes, asset kinds, and the editing model (D-21); where they differ, the PRD decision log wins.
5. The v1.1 provider is PayPhone **Tokenization** (`ctoken` + `/api/transaction/web`), not API Sale; PRD §5.7/FR-063–072 and DB §1/§5 are authoritative.
