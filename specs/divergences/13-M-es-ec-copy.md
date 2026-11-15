# Divergence tracking: 13-M es-EC copy

Source spec: `specs/13-M-ops-and-release.md`, objective 5 (walk the `context/PRD.md` §12 acceptance checklist and record evidence). Also blocks `context/PRD.md:202` ("100% `es-EC` UI") and the `es-EC` label criteria in specs 02-B, 03-C, 04-D, 05-E, 07-G, and 12-L.
Resolving spec: none — requires a UI-copy remediation spec. 13-M stays in `specs/`.

Severity: P2 — the app is mostly Spanish, but the first screen, the entire auth/dashboard shell, and many error paths render English. PRD §8 says 100%, so this is a release-visible nonconformance, not a typo.

## What's broken

A broad literal scan on 2026-11-15 counted 92 candidate English string literals; the displayed ones include:

- `persona-sanus/src/main.rs` — 26. Home tagline and links `1287`, `1290-1291`; Navbar `1397`, `1399`, `1451`, `1454`; signed-in panel `1481`, `1494`; empty-field error `1511`; auth fallback errors `1562`, `1618`; auth headings/buttons/placeholders `1633`, `1641`, `1648`, `1653`, `1660`, `1666`, `1689`, `1691`, `1693`; dashboard `1717`, `1726`, `1747`, `1754`. Examples: `"Your prescriptions and health records, kept organized."`, `"Sign in"`, `"Sign out"`, `"Sign up"`, `"Signed in"`, `"Create your account"`, `"Welcome back"`, `"Name"`, `"Email"`, `"Password"`, `"Please wait..."`, `"Loading..."`, `"Dashboard"`, `"Redirecting to sign in..."`, `"Fill in every field."`, `"The auth server did not return an active session."`.
- `persona-sanus/src/auth_server.rs` — 15: `177`, `300`, `304`, `308`, `373`, `392`, `477`, `487`, `490`, `501`, `512`, `543`, `550`, `601`, `650`. Examples: `"An account with this email already exists."`, `"Enter a valid email address."`, `"Enter your name."`, `"Password must be at least 8 characters."`, `"Invalid email or password."`, `"Enter the MFA code."`, `"Invalid MFA code."`, `"Sign in required."`, `"MFA is available for Doctor and Administrator roles."`, `"Enroll MFA first."`.
- `persona-sanus/src/service.rs` — 42: e.g. `69`, `76`, `85`, `94`, `97`, `257`, `402`, `430-460`, `540-563`, `611-737`, `776-812`, `871-892`. Examples: `"No clinic membership."`, `"Administrator role required."`, `"A clinic role is required."`, `"The administrator can only read patients."`, `"Profile disabled."`, `"Invalid device id."`, `"Unknown platform."`, `"Device name is too long."`, `"Device already registered."`, `"This device has been revoked."`, `"Device not found."`, `"Unknown role."`, `"Profile not found."`, `"Member not found."`, `"Enter a display name."`, `"Doctor role required."`, `"Enter the patient name."`, `"Enter a valid birth date."`.
- `persona-sanus/src/http.rs` — 8: `88`, `114`, `121`, `139`, `142`, `148`, `155`, `157`. Client-visible network errors: `"Could not reach the server: …"`, `"Could not read the server response: …"`, `"Request failed (…)"`.
- `persona-sanus/src/api_server.rs:517` — `"Sign in required."`.

The internal audit `action` strings (`sign_in_failed`, `asset_cleanup`, etc.) and protocol `code` values are machine values and are out of scope; only the UI/API displayed copy changes.

## Why each fix was fenced

- Translation is a product-copy decision: several imperative errors need one consistent Ecuadorian voice (`Ingresa…`, `Escribe…`, `Completa…`, `No se pudo…`). Rewriting them individually during a report-only pass would create mixed terminology again.
- Many strings originate in specs verified by browser spot checks of their own page, not a whole-app copy audit. No single spec owns the shared Navbar/auth shell, so no checkbox was reopened yet.
- The change spans login, MFA, patients, templates, prescriptions, audit, assets, and network fallbacks. It needs one inventory and a review, not drive-by edits.
- No unit tests: verification must be browser/E2E, and `cargo`/`dx` are not on this shell's PATH.

Verified 2026-11-15: 0 fix attempts — `rg` literal inventory over `main.rs`, `auth_server.rs`, `service.rs`, `http.rs`, and `api_server.rs`; representative lines read in context. Fenced by copy-consistency and spec-process constraints, not by tooling.

## How to clear it

- [ ] Pick and record one es-EC glossary before editing: `Sign in` → `Iniciar sesión`; `Sign up` → `Crear cuenta`; `Sign out` → `Cerrar sesión`; `Dashboard` → `Panel`; `Home` → `Inicio`; `Email` → `Correo`; `Password` → `Contraseña`; `Name` → `Nombre`; `Loading...` → `Cargando...`; `Please wait...` → `Espera...`; `Welcome back` → `Bienvenido de nuevo`; `Create your account` → `Crea tu cuenta`; `Fill in every field.` → `Completa todos los campos.`.
- [ ] Translate `main.rs` Home/Navbar/AuthPage/Dashboard: `1287`, `1290-1291`, `1397-1399`, `1451-1454`, `1481-1494`, `1511`, `1562`, `1618`, `1633-1693`, `1717-1754`.
- [ ] Translate `auth_server.rs` validation/auth/MFA errors (`177`, `300`, `304`, `308`, `373`, `392`, `477`, `487-512`, `543`, `550`, `601-604`, `650`, `688`) and `service.rs` domain/role/patient/doctor/device errors (`69-97`, `257`, `402`, `430-460`, `540-563`, `611-737`, `776-812`, `871-892`).
- [ ] Translate `http.rs` network fallbacks (`88`, `114`, `121`, `139`, `142`, `148`, `155`, `157`) and `api_server.rs:517`.
- [ ] Keep HTTP status codes and JSON `code`/`action` identifiers unchanged. Do not translate audit action names, `status` enum values, filenames, or `not found` protocol values.
- [ ] Audit: rerun the broad literal scan and assert 0 UI hits; accept-listed leftovers must be machine values. Record the command in the remediation spec evidence, not as a unit test.
- [ ] Browser E2E: signed-out Home → navbar labels; `/login` sign-in and sign-up; wrong email/password; short password; MFA challenge and wrong code; signed-in Dashboard; Navbar sign-out; `/audit`; `/patients/new`; `/templates/new`; `/prescriptions/new`; network-offline desktop message; 403 role message; 413 upload; 429 rate-limit. Every visible message must be es-EC; no machine codes change.
- [ ] Mark 13-M objective 5 (`every context/PRD.md §12 acceptance criterion has recorded evidence`) `[x]` only after the copy audit is attached, then delete this file.
