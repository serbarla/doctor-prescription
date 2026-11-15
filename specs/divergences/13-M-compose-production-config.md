# Divergence tracking: 13-M compose production config

Source spec: `specs/13-M-ops-and-release.md`, objective 3 (operational backups; "TLS termination and secret handling documented as external prerequisites") and objective 5 (acceptance evidence). Also blocks the deployment half of `specs/done/08-H-pdf-renderer-and-preview.md` objective 2 (QR encodes `{PUBLIC_BASE_URL}/verify/{code}`) and `specs/done/09-I-public-verification.md` objective 2 (the page matches that target).
Resolving spec: none — requires a deployment-config remediation spec. 13-M stays in `specs/`.

Severity: P1 — with the committed compose file, every generated QR points at `127.0.0.1:8080`, browser image URLs point at the browser's own loopback, the session cookie cannot get `Secure`, and MinIO is published with checked-in credentials. The deployment's image and QR surfaces work only on the machine that runs it.

## What's broken

- `docker-compose.yml:10` hardcodes `PERSONA_COOKIE_SECURE: "false"`. `README.md:67` and `context/runbook.md:17` tell the operator to set it true behind TLS, but compose does not allow `${PERSONA_COOKIE_SECURE}` substitution for this value. The server will never set `Secure` on `persona_session`, and `server::security_headers` will never emit HSTS from this compose file.
- `docker-compose.yml:13` hardcodes `ASSETS_PUBLIC_ENDPOINT: http://127.0.0.1:9000`. The server signs every image URL against that host (`src/assets.rs:128-145`). A browser on another machine resolves 127.0.0.1 to itself, so clinical images fail to load even though MinIO and the app are healthy.
- No `PUBLIC_BASE_URL` is set. `persona-sanus/src/pdf_server.rs:164-170` falls back to `http://127.0.0.1:8080`, and that value becomes the QR payload at lines 34 and 78. A PDF printed or shared from the deployment carries a QR that opens nothing on the verifier's phone.
- `docker-compose.yml:44-48` checks in `persona/persona-secret` and publishes MinIO 9000/9001 on every host interface. That is a reasonable local-dev default, but the same file is the runbook's deploy command; there is no production guard, no required `.env`, and no reverse-proxy topology in the docs.
- The objective-5 acceptance walk is recorded as complete, but the scanned-QR acceptance line cannot hold for a non-local deployment of this compose file.

## Why each fix was fenced

- The public base URL, TLS proxy network, and MinIO credentials are deployment inputs that do not exist in this checkout. Inventing a domain or committing production secrets would be worse than the current localhost defaults.
- These values intentionally default to local dev so `docker compose up --build` works on one machine. A fix needs a dev/prod split (explicit `${…}` defaults plus a production fail-closed check) that has not been specified.
- The 2026-11-15 pass was report-only and ran no container; changing the bind/port contract without testing the compose network could break image upload and PDF rendering.

Verified 2026-11-15: 0 fix attempts — static inspection of `docker-compose.yml`, `server.rs`, `assets.rs`, `pdf_server.rs`, README, runbook, architecture, and the 08-H/09-I specs. Fenced by missing deployment values and the spec process.

## How to clear it

- [ ] Parameterize with dev defaults: `PERSONA_COOKIE_SECURE: ${PERSONA_COOKIE_SECURE:-false}`, `ASSETS_PUBLIC_ENDPOINT: ${ASSETS_PUBLIC_ENDPOINT:-http://127.0.0.1:9000}`, and `PUBLIC_BASE_URL: ${PUBLIC_BASE_URL:-http://127.0.0.1:8080}`.
- [ ] Add a `.env.example` with the production values to change: HTTPS `PUBLIC_BASE_URL`, `PERSONA_COOKIE_SECURE=true`, the browser-reachable `ASSETS_PUBLIC_ENDPOINT`, `PERSONA_WEB_ORIGINS` only if cross-origin, plus MinIO credentials and the backup age recipient. Do not commit a real `.env`.
- [ ] For release/non-debug startup, fail closed with a clear error if `PUBLIC_BASE_URL` is unset or is `http://127.0.0.1:8080`, if the cookie is not secure, or if `ASSETS_PUBLIC_ENDPOINT` is loopback while the app is not. Keep the localhost fallback only for `dx serve`/debug.
- [ ] Keep MinIO ports on loopback in the default compose (`127.0.0.1:9000:9000`, `127.0.0.1:9001:9001`) and document the production reverse-proxy/TLS path. Do not publish the admin console broadly.
- [ ] Verify: `docker compose config` shows the substituted values; from a second machine on the LAN, sign in and confirm `Set-Cookie` contains `Secure; HttpOnly; SameSite=Lax`, an uploaded image renders, and a generated PDF's QR decodes to `https://<PUBLIC_BASE_URL>/verify/<code>`.
- [ ] Run `python3 .blocks/persona-docker-verify.py --dir . --port 8080`, `python3 .blocks/pdf-probe.py --base http://127.0.0.1:8080`, and `python3 .blocks/persona-probe.py --base http://127.0.0.1:8080 --flow verify`; then re-check 13-M objectives 3 and 5 and the 08-H/09-I evidence, and delete this file.
