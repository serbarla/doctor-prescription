# Spec 41-AO — Extract the application header module

- Source: `context/screens/done/Header.md` §§1–8; `context/screens/INDEX.md` §§1.1, 1.3 and 2.1; `context/architecture.md` §§3 and 7; approved dedicated-module extraction decision.
- Depends on: `20-T`, `40-AN`.
- Project root: the `doctor-prescriptions` repository root. All paths and commands below are relative to it.
- Owned files: `persona-sanus/src/main.rs`, `persona-sanus/src/navbar.rs`, `context/screens/done/Header.md`, `context/screens/INDEX.md`, `context/architecture.md`.
- Gate: `cargo check --manifest-path persona-sanus/Cargo.toml && cargo check --manifest-path persona-sanus/Cargo.toml --no-default-features --features desktop`.
- User-interface verification: required — exercise the actual routed header with signed-out, unassigned and four staff-role sessions on an isolated local database; record route, responsive, keyboard and accessibility evidence using section 3.
- Result carried forward: Navbar has one implementation in `src/navbar.rs`, routed by the existing layout, with unchanged shell DOM, permissions and side effects.
- Scope-change approver: the requesting repository owner.

## 1. Problem and decision

- Problem: the application header implementation is embedded in the large `main.rs`, making its session-aware shell responsibilities harder to locate independently of page bodies. The screen document contains stale source locations and planned-component descriptions; those are not permission to redesign the shipped shell.
- Decision: extract the intact `Navbar` into a dedicated sibling application module, not `src/ui/`, which architecture §3 reserves for reusable primitives without storage or business rules. Keep its name, markup, styles, routes, side effects and error behavior. This is an execution specification only; its generation does not execute the refactor.
- Open questions: None.

Both dependencies are completed: `specs/done/20-T-shell-auth-home-and-verify.md` supplies the adopted shell and public surfaces; `specs/done/40-AN-home-accessibility-and-flows.md` records the current Home/verification behavior. Execute this spec before `42-AP`: both own the extraction seam in `main.rs` and factual module references in common documentation. Do not add dependencies on unrelated unfinished product or release specs. Queue ownership conflicts are scheduling constraints, not authorization to edit those specs.

## 2. Objectives and scope

- [x] 1. Extract the single application layout component — requirements: move the complete `Navbar` component from `persona-sanus/src/main.rs` into new `persona-sanus/src/navbar.rs` as `#[component] pub(crate) fn Navbar() -> Element`. The inspected source inventory has no equivalent application module. Add `mod navbar;` and `use crate::navbar::Navbar;` in `main.rs`, delete the old definition, and keep `#[layout(Navbar)]` resolving to the moved component. `Header.md` is a frozen documentation filename, not a component rename. Restrict the extraction edits in `main.rs` to the moved component and its module/import seam. Update only factual component/module references caused by extraction in `Header.md`, `INDEX.md` and `architecture.md`; do not rewrite their screen contracts or repair unrelated historical prose.
- [x] 2. Establish explicit imports without compatibility layers — requirements: in `navbar.rs` use `dioxus::prelude::*`, `dioxus::router::router`, `crate::{api::Me, auth, auth::SessionUser, clear_persisted_session, probe_sync_status, Route}`, and `crate::ui::{LogoLockup, PillBadge, NavLink, Button, ButtonLink, ButtonVariant, SyncStatus, SyncStatusIndicator}`. Remove root imports of `NavLink`, `SyncStatusIndicator` and `router` when their consumers move; retain root `Navigator`, used elsewhere. Reuse existing primitives and root helpers. No `crate::*`, re-export aliases, forwarding components or moved helpers.
- [x] 3. Preserve hooks, sync probing and logout behavior intact — requirements: retain all session/profile/sync context and signal reads in the same hook order. Keep `router.current()` inside the existing route-tracking effect, the same spawned `probe_sync_status().await` and the assignment to the same sync signal. Keep logout ordering exactly: await `auth::sign_out()` while ignoring its result as today, call `clear_persisted_session()`, clear actor, clear user, then `navigator.push(Route::Home {})`. Local clearing must still happen after a network failure. No new error policy, polling, provider, prop or missing-state policy.
- [x] 4. Preserve header rendering and authorization — requirements: keep `header#navbar.ui-navbar`, every class, attribute, string, guard and route. Always render the brand (`LogoLockup` and `PillBadge`). Signed out renders no nav element and one Ghost `Iniciar sesión` to `/login`. Any session gets `Panel` and `Configuración`; reuse `can_read_patients()`, `can_read_prescriptions()` and `can_manage_templates()` for the other links. Keep order and targets: `Panel` → `/dashboard`, `Pacientes` → `/patients`, `Recetas` → `/prescriptions` with `status: None`, `Plantillas` → `/templates`, `Configuración` → `/settings`. Administrator/doctor get five links; nursing/secretary four without `Plantillas`; missing profile or null role gets two. Preserve session email and Ghost `Cerrar sesión`.
- [x] 5. Preserve sync details and layout integration — requirements: reuse `SyncStatusIndicator` unchanged, including offline → failed → pending → online label precedence (`Sin conexión`, `Error de sincronización`, `Sincronizando`, `En línea`). Preserve `· 1 pendiente`, `· {n} pendientes` and `{n} con error`, the existing count-link classes and literal status targets `pending_sync` and `failed`. Both links remain independently conditional on their respective positive counts. Retain `Outlet::<Route> {}` after the header and the complete `Route` enum/layout order: Verify and DevUi before the layout marker, Home, AuthPage and subsequent routes inside it. No CSS changes.

The repository owner approved one scope exception: update `update_patient_offline` in `main.rs` to pass `medical_record_no` and `allergies` to existing local persistence and include them in the returned `api::Patient`. No other helper changes are in scope.

### Out of scope / no-go

- No product, route, role, permission, copy, styling, layout-graph, PHI, legal or billing expansion. Preserve the implementation rather than making it conform to historical screen prose. No sixth nav destination, new chrome control, sticky positioning or conformance redesign.
- No dependency, provider, storage, API, server, session-helper, sync-helper or primitive changes. Application header policy must not move into `src/ui/`. Do not duplicate the component, introduce a shim or add code comments.
- Do not edit unowned files or repair unrelated existing UI/CSS/documentation failures. `main.rs` ownership does not authorize editing page bodies, the Route enum, root helpers or context providers. Documentation ownership permits factual component/module-reference changes only.
- Existing missing-profile and sign-out error handling is part of the preserved contract. An intact move needs no new fallback or missing-state policy. No broad test suite, release build or new test scaffolding is required for unchanged component bodies.

## 3. Acceptance and verification

Run the gate once after the final code edit, covering default web/server and desktop compilation. If a correction requires another edit, rerun the same gate after that final correction. Do not run checks between edits. Required UI observation below is additional proof of the unchanged surface, not a redesign opportunity.

Prerequisites: Cargo, `dx`, a browser, free port 8091 and a fresh temporary database. From the repository root launch:

```sh
CHROME_DB_DIR="$(mktemp -d)" &&
(cd persona-sanus && DATABASE_URL="sqlite://$CHROME_DB_DIR/chrome.db" PERSONA_ENV=development PERSONA_COOKIE_SECURE=false dx serve --fullstack --addr 127.0.0.1 --port 8091)
```

After the local service responds, open a fresh tab at `http://127.0.0.1:8091/`. Never use a production database or borrowed user tab. Use real UI interactions and read-only DOM/accessibility inspection, not injected controls or session state. Stop the service after observation and clean up only the temporary proof fixtures created for this task.

- [x] Objectives 1–2: module seam — LSP resolves the layout marker in `main.rs:72` to `navbar.rs:10`. Its only references are the definition, the root import, and the layout marker. The move keeps the component body intact and changes only the module seam and factual documentation references.
- [x] Objectives 4–5: signed-out routes and Outlet — `/` and `/login` each showed one `header#navbar.ui-navbar`, both brand elements, a Ghost `Iniciar sesión` linking to `/login`, and no nav element. `/verify/PS-TEST` showed the verification result and `/dev/ui` showed the component gallery, both without the navbar.
- [x] Objective 4: session/role matrix — on fresh SQLite, created both synthetic accounts through `Crear cuenta`, then changed the staff role in `/settings/users` and signed back in after each change. Administrator and Doctor showed `Panel` (`/dashboard`), `Pacientes` (`/patients`), `Recetas` (`/prescriptions?`, an empty query), `Plantillas` (`/templates`), and `Configuración` (`/settings`) in order. Nursing and Secretary omitted `Plantillas`; the unassigned session showed only `Panel` and `Configuración`. The unchanged guards give a missing profile the same two-link fallback; that state was not injected.
- [x] Objectives 3–5: navigation, sync and logout preservation — verified against the shipped behavior accepted by the repository owner. `Recetas` reaches `/prescriptions` without an active class; the visible keyboard focus outline remains 1 px Deep Teal.

Browser evidence for navigation and responsive behavior: `Panel`, `Pacientes`, and `Configuración` gained `ui-nav-link--active` on their routes. `Recetas` reached `/prescriptions` but did not gain the active class; see Divergences. The session email and `En línea` persisted across those navigations. The accessibility tree showed banner, navigation, labeled links, and status; the DOM retained `role="status"`, `aria-live="polite"`, and `aria-hidden="true"` on the dot. `Cerrar sesión` returned to `/` with the signed-out header. The moved source retains the probe, logout order, and both independent count conditions, labels, and targets. Nonzero desktop outbox counts were not exercised.
- [x] Gate: `cargo check --manifest-path persona-sanus/Cargo.toml && cargo check --manifest-path persona-sanus/Cargo.toml --no-default-features --features desktop`. Both configurations passed after the approved fix.
- [x] User-interface verification: observed and captured the administrator header at 1440, 980, and 375 px. It used one grid row at 1440 px, wrapped at 980 px, and hid the email at 375 px without horizontal header overflow. Two Tabs from page load focused `Panel` with a visible 1 px Deep Teal outline at each width. The role, route, sync, active-link, and logout evidence is recorded above. Screenshot IDs (Orca temporary captures): `5e80605c-11b9-49b3-b7ab-80e5f3ffee6c`, `96d10cdd-9f59-44b9-b1c4-d5c8b9af9caf`, `f26cf094-404e-4491-8b82-63754ce7c992`.
- [x] Every objective in section 2 is checked; the gate and required UI evidence are complete.

## 4. Execution and divergences

Execute in a fresh, focused task context with a hard ceiling of **500,000 loaded tokens**. This is a maximum per execution task, not a target spec size or provider configuration change. Read applicable project instructions, this spec, relevant completed dependency results, `context/screens/done/Header.md`, screen-index authority/naming rules, architecture module/shell rules, `persona-sanus/Cargo.toml` and only relevant source/component regions. For Header this includes `Route`, root imports, the complete `Navbar`, the `Me` guards, relevant UI component contracts, root `probe_sync_status` and persisted-session/logout helpers and their cfg branches. Do not load all of `main.rs`, all specs or previous transcripts.

Immediately before moving the symbol, run LSP references for `Navbar` and resolve the actual layout consumer. Current source hints are `main.rs:1596–1696` for the component and `:70` for the layout marker; line numbers are hints only. Reread the actual source region before editing, and resolve helpers with LSP rather than guessing their locations. If `navbar.rs` exists at execution time, reuse it only if it already owns the same component/contract; otherwise record an ownership divergence before touching it. Never overwrite an unrelated module or invent an alternate name.

If reality differs or necessary work exceeds ownership, record the current state, evidence and attempts actually made, impact, blocker, owner, and required decision under Divergences. Stop affected work and leave affected acceptance criteria unchecked with the divergence noted beside them; unaffected objectives may continue within scope. The requesting repository owner must decide before scope or ownership changes. Preserve existing error behavior rather than introducing an unapproved policy. Incomplete dependencies block affected objectives unless the requesting repository owner explicitly decides otherwise.

Open divergences block completion until resolved or an authorized scope change updates objectives and criteria. Follow project divergence tracking-file rules for approved deferrals, including an owner or later spec; required unowned tracking work needs approval, not an implicit ownership expansion. Archive the completed spec under `specs/done/` only when all applicable criteria, the gate, UI verification and objectives pass with no open divergences. Do not execute either extraction spec during specification generation.

### Divergences

1. **Resolved — desktop gate:** The desktop check originally failed with `E0063` because `update_patient_offline` omitted `medical_record_no` and `allergies` from its `api::Patient` initializer. With repository owner approval, the helper now passes both form fields to the existing local update and outbox path and returns them as optional fields. Both compilation gates passed.
2. **Resolved — existing route and focus behavior:** The repository owner accepted the observed `Recetas` active state and keyboard outline as shipped. No route or CSS change was made.

**Verification limit:** Compilation covered the offline update; the desktop patient flow was not exercised at runtime.
