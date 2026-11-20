# persona-sanus

Persona Sanus: Dioxus fullstack prescription app (Ecuador, `es-EC`) with app-owned email/password authentication in a single process, backed by a single SQLite database (`app.db`) through Diesel, plus a desktop offline client.

```
persona-sanus/
├─ assets/          # App assets
├─ migrations/      # Diesel migrations (embedded in the server binary)
├─ src/
│  ├─ main.rs          # Entry point: Dioxus UI + fullstack server `main`, routes
│  ├─ api.rs           # Typed REST client for the app pages
│  ├─ api_server.rs    # `/api/*` handlers (patients, templates, prescriptions, assets, clinic, settings, devices, audit, rx)
│  ├─ assets.rs        # S3/MinIO client, image validation, signed URLs
│  ├─ auth.rs          # Browser client for the auth API
│  ├─ auth_server.rs   # Argon2 password hashing, opaque sessions, MFA, auth handlers
│  ├─ db.rs            # Diesel connection pool, models and queries
│  ├─ desktop_local.rs # Desktop local SQLite cache + outbox
│  ├─ dev_db.rs        # Debug-only `/dev/db` SQLite browser
│  ├─ device.rs        # Device registration and revocation
│  ├─ forms.rs         # Template form components (`doc_json`)
│  ├─ http.rs          # Base-URL split and shared transport errors
│  ├─ jobs.rs          # In-process scheduler: purge, cleanup, retention trim, daily backup, update manifest
│  ├─ pdf.rs           # A4 `printpdf` renderer (server + desktop)
│  ├─ pdf_server.rs    # `GET /api/prescriptions/:id/pdf`
│  ├─ schema.rs        # Diesel table definitions
│  ├─ server.rs        # Server-only: axum router = Dioxus app + API routes, CORS, security headers
│  ├─ service.rs       # Authorized service layer (roles, clinic scoping, audit)
│  ├─ sync_server.rs   # `/api/sync/*`: outbox drain + SSE push
│  ├─ ui/              # Shared design-token UI primitives
│  ├─ update.rs        # Desktop update check, manifest verification, staged apply + rollback
│  └─ verify_server.rs # Public `/api/verify/:code`, per-IP limiter
├─ app.db           # The single SQLite database (created on first run)
├─ Cargo.toml       # Default features: web + server (fullstack)
└─ Dockerfile
```

Auth routes (`/api/auth/...`) and the app itself are served from one origin on `http://127.0.0.1:8080`. Sessions use an `HttpOnly`, `SameSite=Lax` cookie (`persona_session`); tokens are random, stored only as a SHA-256 hash, and expire after 30 days. Same-origin use needs no CORS setup; to serve another web origin, set `PERSONA_WEB_ORIGINS` to enable credentialed CORS for it.

Diesel applies the embedded migrations at startup. The first migration replaces the previous RustAuth schema: its `users`, `sessions`, `accounts`, and `verifications` tables are dropped and recreated, so accounts created before this change must sign up again.

## Run locally

```bash
dx serve --fullstack
```

Open `http://127.0.0.1:8080`, sign up at `/login`, then visit the protected `/dashboard` route.

Debug builds also mount a read-only SQLite browser at `http://127.0.0.1:8080/dev/db` (`src/dev_db.rs`). It lists the live schema, paginates every table, shows the DDL, and runs read-only SQL against `DATABASE_URL`.

| Surface | Purpose |
|---|---|
| `/dev/db` | Browser UI |
| `/dev/db/api/tables` | Tables and views from `sqlite_master` |
| `/dev/db/api/table?name=…&limit=…&offset=…` | Paginated table rows and column metadata (max 500 rows) |
| `/dev/db/api/query?sql=…` | Read-only SQL console (max 500 rows, one statement) |

Guards:

- Only compiled with `debug_assertions`; `dx serve --release --fullstack` and release binaries return 404.
- Loopback peer addresses only; non-loopback connections return 403.
- SQLite is opened with `SQLITE_OPEN_READONLY`.
- Writes are rejected via `sqlite3_stmt_readonly`; multiple statements are rejected.
- GET only; no write endpoint is mounted.

## Component library

Shared components live in `src/ui/` (design-token contract and reusable `.ui-*` primitives), and the styles live in `assets/main.css`. Debug builds expose the component gallery at `/dev/ui`; release builds do not expose the gallery — `/dev/ui` answers with the development-only notice.

## Operational surfaces

Product and operations documentation lives in the repository root: the runbook is [context/runbook.md](../context/runbook.md) and the full configuration reference is [context/architecture.md](../context/architecture.md) §14. Highlights:

- **MFA**: TOTP enrolment/verification under `/api/auth/mfa/*` (spec 12-L), gated in `src/auth_server.rs`.
- **Devices**: registration and revocation via `/api/devices*`, enforced through `sessions.device_id`.
- **`/dev/db`**: debug-only SQLite browser (see above); never shipped in release.
- **Backups**: encrypted daily `VACUUM INTO` snapshot plus MinIO `mc mirror`, scheduled by `src/jobs.rs`; failures are recorded in `audit_log` (`backup_daily`). Restore, key custody, and known backup gaps: [context/runbook.md](../context/runbook.md) §2–§3.
- **Retention**: export-first trim after `data_grace_ends_at` (on in v1, `RETENTION_TRIM_ENABLED=true`), export at `/settings/clinic`; prerequisites in [context/runbook.md](../context/runbook.md) §5.
- **Sync**: `/api/sync/state` and `/api/sync/events` (SSE) drain the desktop outbox with idempotent `client_id`.
- **Public verification**: `/verify/:code` (UI) and `/api/verify/:code`, rate-limited per IP, no PHI.
- **Auto-update**: desktop installs check the ed25519-signed manifest from `GET /api/updates/manifest` (`UPDATE_MANIFEST_URL`); staging, rollout, and rollback semantics in [context/runbook.md](../context/runbook.md) §6.

Environment variables (all optional in development; the full table with scopes and notes is §14 of [context/architecture.md](../context/architecture.md)):

| Variable | Default | Purpose |
|---|---|---|
| `DATABASE_URL` | `sqlite://app.db` | SQLite database path |
| `PERSONA_COOKIE_SECURE` | `false` | Set to `true` behind HTTPS so the session cookie gets `Secure` |
| `IP` / `PORT` | `127.0.0.1` / `8080` | Bind address (Compose and the image set `IP=0.0.0.0`) |
| `PUBLIC_BASE_URL` | `http://127.0.0.1:8080` fallback | Absolute base for QR/verification links; must be overridden in production |
| `PERSONA_WEB_ORIGINS` | `` (same-origin only) | Comma-separated web origins allowed by credentialed CORS |
| `PERSONA_SERVER_ORIGIN` | `http://127.0.0.1:8080` | Native (desktop) API origin; the wasm client uses relative paths |
| `ASSETS_BUCKET` / `ASSETS_ENDPOINT` / `ASSETS_REGION` | — / — / `us-east-1` | Object storage (MinIO) for clinic images |
| `ASSETS_ACCESS_KEY_ID` / `ASSETS_SECRET_ACCESS_KEY` | — | Object storage credentials; never shipped to clients |
| `ASSETS_SIGNED_URL_TTL_SECONDS` / `ASSETS_PUBLIC_ENDPOINT` | `900` / `ASSETS_ENDPOINT` | Signed GET URL lifetime; browser-reachable endpoint base |
| `BACKUP_DIR` / `BACKUP_AGE_RECIPIENT` / `BACKUP_INTERVAL_SECONDS` | `./backups` / — / `86400` | Backup target dir; `age1…` recipient required for backups |
| `JOBS_INTERVAL_SECONDS` / `RETENTION_TRIM_ENABLED` | `3600` / `true` | Scheduler cadence; retention trim switch |
| `AUTH_RATE_LIMIT_PER_MINUTE` / `VERIFY_RATE_LIMIT_PER_MINUTE` | `20` / `10` | Per-IP rate limits for auth and public verification |
| `UPDATE_MANIFEST_URL` / `UPDATE_PUBLIC_KEY` / `UPDATE_SIGNING_KEY` | — | Desktop auto-update client/server keys |
| `UPDATE_MANIFEST_VERSION` / `UPDATE_MANIFEST_DOWNLOAD_URL` / `UPDATE_MANIFEST_ROLLOUT` | — / — / `0` | Served update manifest fields; rollout `0` is the kill switch |
| `PERSONA_DEVICE_ID` / `PERSONA_LOCAL_DATA_DIR` | — / app-data dir | Desktop rollout-bucket identity; local cache location |

Object storage (`ASSETS_*`) is required for image features (clinic library, template images, logo). The backup recipient (`BACKUP_AGE_RECIPIENT`) is required for backups: without it the daily backup job skips with an `audit_log` row.

## Run with Docker Compose

From the repository root:

```bash
docker compose up --build
```

Starts the fullstack app on port 8080 with SQLite persisted in the `persona-data` volume (`/data/app.db`). MinIO ports bind to `127.0.0.1` only. The image also carries a standalone bind contract (`IP=0.0.0.0`, `EXPOSE 8080`), so `docker run -p 8080:8080 <image>` reaches `/health` without Compose env.

Production: copy the root `.env.example` to `.env`, set `PERSONA_ENV=production` with an HTTPS `PUBLIC_BASE_URL`, `PERSONA_COOKIE_SECURE=true`, and a browser-reachable `ASSETS_PUBLIC_ENDPOINT`, then terminate TLS in front of the app and route the public host through the proxy. The release server refuses to start when those values are missing, loopback, or insecure under `PERSONA_ENV=production`; the localhost defaults only apply to local development.
