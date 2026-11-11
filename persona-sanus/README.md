# persona-sanus

Dioxus fullstack app + RustAuth authentication in a single process, backed by a single SQLite database (`app.db`).

```
persona-sanus/
├─ assets/          # App assets
├─ src/
│  ├─ main.rs       # Entry point: Dioxus UI + fullstack server `main`
│  ├─ auth.rs       # Browser client for the auth API
│  └─ server.rs     # Server-only: axum router = Dioxus app + RustAuth routes
├─ app.db           # The single SQLite database (created on first run)
├─ Cargo.toml       # Default features: web + server (fullstack)
└─ Dockerfile
```

Auth routes (`/api/auth/...`) and the app itself are served from one origin on `http://127.0.0.1:8080`. The browser talks to the auth API same-origin, so no CORS setup is needed.

## Run locally

```bash
dx serve --fullstack
```

Open `http://127.0.0.1:8080`, sign up at `/login`, then visit the protected `/dashboard` route.

Environment variables (all optional in development):

| Variable | Default | Purpose |
|---|---|---|
| `DATABASE_URL` | `sqlite://app.db` | SQLite database path |
| `RUSTAUTH_SECRET` | dev-only secret | Session signing secret (≥32 chars) |
| `RUSTAUTH_BASE_URL` | `http://127.0.0.1:8080/api/auth` | Public base URL of the auth API |
| `RUSTAUTH_AUTH_BASE_PATH` | `/api/auth` | Path the auth API is mounted at |

## Run with Docker Compose

From the repository root:

```bash
docker compose up --build
```

Starts the fullstack app on port 8080 with SQLite persisted in the `persona-data` volume (`/data/app.db`).

> Warning: the default `RUSTAUTH_SECRET` is a development secret only. Set a real secret of at least 32 characters via the `RUSTAUTH_SECRET` environment variable outside local development.
