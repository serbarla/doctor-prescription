# persona-sanus

Dioxus fullstack app with app-owned email/password authentication in a single process, backed by a single SQLite database (`app.db`) through Diesel.

```
persona-sanus/
├─ assets/          # App assets
├─ migrations/      # Diesel migrations (embedded in the server binary)
├─ src/
│  ├─ main.rs       # Entry point: Dioxus UI + fullstack server `main`
│  ├─ auth.rs       # Browser client for the auth API
│  ├─ auth_server.rs# Argon2 password hashing, opaque sessions, auth handlers
│  ├─ db.rs         # Diesel connection pool, models and queries
│  ├─ schema.rs     # Diesel table definitions
│  └─ server.rs     # Server-only: axum router = Dioxus app + auth routes
├─ app.db           # The single SQLite database (created on first run)
├─ Cargo.toml       # Default features: web + server (fullstack)
└─ Dockerfile
```

Auth routes (`/api/auth/...`) and the app itself are served from one origin on `http://127.0.0.1:8080`. Sessions use an `HttpOnly`, `SameSite=Lax` cookie (`persona_session`); tokens are random, stored only as a SHA-256 hash, and expire after 30 days. The browser talks to the auth API same-origin, so no CORS setup is needed.

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

Environment variables (all optional in development):

| Variable | Default | Purpose |
|---|---|---|
| `DATABASE_URL` | `sqlite://app.db` | SQLite database path |
| `PERSONA_COOKIE_SECURE` | `false` | Set to `true` behind HTTPS so the session cookie gets `Secure` |

## Run with Docker Compose

From the repository root:

```bash
docker compose up --build
```

Starts the fullstack app on port 8080 with SQLite persisted in the `persona-data` volume (`/data/app.db`).

> Warning: set `PERSONA_COOKIE_SECURE=true` and terminate TLS in front of the container outside local development.
