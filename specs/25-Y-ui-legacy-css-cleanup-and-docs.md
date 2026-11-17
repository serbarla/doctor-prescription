# Spec 25 — Phase Y: Legacy CSS removal and docs close

- Source: `UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` steps 17–18; `context/architecture.md` §3; `context/sitemap.md` §1.3; `context/PRD.md` §12; `context/Design.md` §§Tokens, Surfaces, Elevation, Layout, Agent Prompt Guide; `persona-sanus/README.md`.
- Depends on: 15-O, 16-P, 17-Q, 18-R, 19-S, 20-T, 21-U, 22-V, 23-W, 24-X.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Result carried forward: `persona-sanus/assets/main.css` contains only the token layer, the `.ui-*` component/utility layer, the A4 print projection, and the debug gallery rules; every legacy selector and page-ID width hook is gone; `#navbar` is a semantic id only; the docs name `src/ui` and `/dev/ui`; the full static, release, debug gallery, source-audit, and manual browser gates pass.

## Before you start

Unit testing is not required; avoid it. Prioritize end-to-end and integration testing only.

Read the nearest `AGENTS.md` (target workspace, then parents up to repo root) before executing.

If reality diverges from this spec: STOP, append a divergence (section 4), and NEVER improvise outside the objectives.

This is the final consolidation gate for the split UI work. It must not add a new visual language, selector, page pattern, route, API, or dependency. It removes only what earlier specs replaced and records what now exists.

## 1. Objectives

*The ordered work to implement — one item per step. Check `[x]` only after the step is implemented and verified.*

*Item format:* `<verb phrase>` — requirements: `<what must change / constraints>`; commands: `<exact command>`.

- [ ] Remove the old style system — requirements: in `persona-sanus/assets/main.css`, delete every legacy selector and ID listed in section 8 after no release page emits it. Keep `.a4-*` as the print projection. Keep `#navbar` only as the semantic id on `header.ui-navbar`, never as a style hook. Replace every remaining page-ID width rule with `.ui-page` and `.ui-container`, including the narrow and reading variants. Reduce the `.a4-page` shadow to `var(--shadow-paper)` and replace its literal `#111111` with `var(--color-ink)`; do not change mm sizes, page boxes, pagination, QR layout, or print fonts. Do not add comments. Do not add a replacement page-specific selector; reuse the shared `.ui-*` layer. Keep the focus utility and the one-primary rule intact. Run the section 3 source audits against the section 8 list; commands: `cd persona-sanus && cargo fmt --check`; `cd persona-sanus && cargo check`; `cd persona-sanus && cargo clippy -- -D warnings`; `cd persona-sanus && cargo check --no-default-features --features desktop`.
- [ ] Update the source-of-truth docs and close the split — requirements: add a `src/ui/` row to the module map in `context/architecture.md` describing a shared client module that owns the design-token contract and reusable UI primitives, and explicitly has no SQL, HTTP handlers, storage, or business rules. Add `/dev/ui` under `context/sitemap.md` §1.3 as a debug-only developer surface, not a product route. Add a short `Component library` section to `persona-sanus/README.md`: shared components live in `src/ui`, styles live in `assets/main.css`, debug builds expose `/dev/ui`, release builds do not expose the gallery. Do not edit `context/PRD.md`, `context/Design.md`, or billing status. When section 3 passes, move this file and every split predecessor from `specs/` to `specs/done/`; if any divergence is open, leave the owning file in `specs/` and record its tracking file; commands: `rg -n 'src/ui/' context/architecture.md`; `rg -n '/dev/ui' context/sitemap.md`; `rg -n 'Component library|/dev/ui|src/ui' persona-sanus/README.md`.

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No new crate, CSS framework, external stylesheet, webfont, CDN asset, icon package, or JavaScript bridge.
- No rich text, WYSIWYG, `contenteditable`, formatting toolbar, `execCommand`, HTML payload, or editor dependency. All text entry is plain controlled inputs, textareas, selects, and checks.
- No new product route, API handler, server function, Diesel model, table, migration, entitlement rule, or billing persistence. `/settings/billing` stays reserved with no surface; `/dev/ui` stays debug-only.
- No change to authorization, session, device registration, sync, numbering, signing, PDF, QR, verification limits, or business validation.
- No new page-specific selector, one-off width rule, or second styling system. Do not add a selector just to keep the layout of an old class name; migrate the markup to the shared primitive instead.
- No deletion or geometry change to `.a4-*`: no mm size, page box, pagination, QR layout, print font, or `A4Preview` return changes.
- No release-visible gallery link and no unconditional `gallery`/`billing` module.
- No pure black, drop shadow on UI cards or buttons, chromatic card border, bold grenette headline, or gradient beyond the ember ring and verification conic ring.
- No patient data, card token, PAN, CVV, or card number in docs, gallery copy, or markup.
- No unit tests and no comments in Rust, CSS, or RSX.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [ ] Static and both targets pass:
```
cd persona-sanus
cargo fmt --check
cargo check
cargo clippy -- -D warnings
cargo check --no-default-features --features desktop
```
Expected: exit 0 for all. The desktop command proves the shared controls and adopted desktop branches compile without the web/server cheats.

- [ ] Release integration passes:
```
caveman-blocks run persona-smoke --dir persona-sanus --port 8094
```
Expected: one JSON object with `"ok": true`, every value in `checks` true, and `"port": 8094`; exit 0.

- [ ] Debug gallery static proof passes:
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
grep -q 'Logos de ejemplo; no son clientes reales.' "${tmpdir}/gallery.html"
grep -Fq -- '$5,00 USD' "${tmpdir}/gallery.html"
grep -Fq -- '$50,00 USD' "${tmpdir}/gallery.html"
grep -Fq -- '$43,48 USD' "${tmpdir}/gallery.html"
grep -Fq -- '$1.234,56 USD' "${tmpdir}/gallery.html"
grep -Fq -- '-$5,00 USD' "${tmpdir}/gallery.html"
! grep -qE 'card_token|ctoken|card_holder|authorization_code' "${tmpdir}/gallery.html"
curl -fsS http://127.0.0.1:8095/ >"${tmpdir}/home.html"
grep -q 'Recetas claras, verificables y listas para firmar.' "${tmpdir}/home.html"
test "$(grep -oE '<(a|button)[^>]*class="[^"]*ui-button--primary[^"]*"' "${tmpdir}/home.html" | wc -l | tr -d ' ')" = "1"
curl -fsS http://127.0.0.1:8095/login >"${tmpdir}/login.html"
test "$(grep -oE '<(a|button)[^>]*class="[^"]*ui-button--primary[^"]*"' "${tmpdir}/login.html" | wc -l | tr -d ' ')" = "1"
curl -fsS http://127.0.0.1:8095/verify/NO-EXISTE >"${tmpdir}/verify.html"
grep -q 'Verificar' "${tmpdir}/verify.html"
test "$(grep -oE '<(a|button)[^>]*class="[^"]*ui-button--primary[^"]*"' "${tmpdir}/verify.html" | wc -l | tr -d ' ')" = "1"
! grep -qiE 'paciente|patient|c[eé]dula|diagn[oó]stico|medicamento|medication' "${tmpdir}/verify.html"
```
Expected: no output, exit 0.

- [ ] Release gallery absence passes:
```
set -euo pipefail
cd persona-sanus
tmpdir="$(mktemp -d)"
DATABASE_URL="sqlite://${tmpdir}/ui.db" PORT=8096 IP=127.0.0.1 RUSTAUTH_SECRET="$(openssl rand -hex 32)" target/dx/persona-sanus/release/web/server >"${tmpdir}/server.log" 2>&1 &
pid=$!
trap 'kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; rm -rf "$tmpdir"' EXIT
for _ in $(seq 1 100); do curl -fsS http://127.0.0.1:8096/health >/dev/null 2>&1 && break; sleep 0.2; done
curl -fsS http://127.0.0.1:8096/health >/dev/null
curl -fsS http://127.0.0.1:8096/dev/ui >"${tmpdir}/release-gallery.html"
grep -q 'solo existe en compilaciones de desarrollo' "${tmpdir}/release-gallery.html"
! grep -q 'id="gallery-' "${tmpdir}/release-gallery.html"
```
Expected: exit 0.

- [ ] Source audits pass:
```
rg -n '\b(button|input|select|textarea) \{' persona-sanus/src/main.rs persona-sanus/src/forms.rs
rg -n 'contenteditable|execCommand|innerHTML|WYSIWYG|rich-text' persona-sanus/src
rg -n -i 'card_token|card_holder|ctoken|authorization_code|card_number|\bcvv_number\b|\bpan_number\b' persona-sanus/src/ui
rg -n 'class: "(button|secondary|danger|badge|notice|error|hint|page-head|actions|form-grid|field|detail-grid|notes|patient-|template-|editor-section|settings-columns|break-anywhere|variable-row|medication-block|inline-block|preview-controls|prescription-filters|block-indicator|warning|medication-list|item-card|item-read|llm-link)' persona-sanus/src/main.rs persona-sanus/src/forms.rs
rg -n '(\.(button|badge|notice|error|hint|page-head|actions|form-grid|field|detail-grid|notes|search-row|patient-|template-|editor-section|settings-columns|break-anywhere|variable-row|medication-block|inline-block|preview-controls|preview-scroll|prescription-filters|block-indicator|warning|medication-list|item-card|item-read|llm-link|clinic-logo|asset-row)|button\.(secondary|danger)|#(navbar|echo|hero|links|patients-page|templates-page|settings-page|prescriptions-page|audit-page))' persona-sanus/assets/main.css
```
Expected: all five return no matches (exit 1). The first command is checked from the repo root; if a match is an SVG path attribute containing `input {`, inspect; only raw RSX form controls count.

- [ ] Extra visual/source audits pass:
```
rg -n -P 'box-shadow:\s*(?!none\b|0 0 0 3px var\(--color-mint-halo\)|var\(--shadow-paper\))' persona-sanus/src/ui persona-sanus/assets/main.css
rg -n 'font-weight: *700|font-weight: *bold' persona-sanus/src/ui persona-sanus/assets/main.css
rg -n -i '#000000|#000\b|rgb\(0,\s*0,\s*0\)|color:\s*black' persona-sanus/assets/main.css persona-sanus/src/ui
rg -n 'animation:' persona-sanus/assets/main.css
rg -n '#111111|0 2px 12px rgba\(0, 0, 0, 0\.18\)' persona-sanus/assets/main.css
```
Expected: the first, third, and fifth return no matches; the second returns only `.a4-doctor`; the fourth returns only `.ui-spinner`. No `font-weight: 700` outside the A4 document projection.

- [ ] Docs checks pass:
```
rg -n 'src/ui/' context/architecture.md
rg -n '/dev/ui' context/sitemap.md
rg -n 'Component library|/dev/ui|src/ui' persona-sanus/README.md
```
Expected: architecture has the shared client module row with `no SQL, HTTP handlers, storage, or business rules`; sitemap §1.3 labels `/dev/ui` debug-only and not a product route; README has the `Component library` section with `src/ui`, `assets/main.css`, `/dev/ui`, and release absence. Diff review confirms `context/PRD.md` and `context/Design.md` were not edited, and no billing status or `/settings/billing` surface changed.

- [ ] Manual browser gate passes with a throwaway database:
```
cd persona-sanus
dx serve --fullstack --port 8097
```
Open:
- `http://127.0.0.1:8097/`: centered hero, serif 46–60px headline, one filled teal CTA, secondary outlined `Verificar una receta`, mint pill next to the wordmark, 3-column steps on wide; entering a code navigates to `/verify/<code>`; the primary CTA goes to login (signed out) or prescription creation/history/profile per the four branches.
- `http://127.0.0.1:8097/login`: sign-in/sign-up switch is keyboard usable; all labels visible; empty fields show `Completa todos los campos.`; one filled submit; after signup the dashboard loads; with an MFA ticket only the code field, Primary `Verificar código`, and Secondary `Volver` show.
- `http://127.0.0.1:8097/dev/ui`: all 13 sections; button variants; badge tones; field error; `$5,00 USD` / `$50,00 USD`; no card/token/PAN/CVV value; trust strip note; all five verification results; the None, inactive, active, and exhausted RX states; no console errors.
- `http://127.0.0.1:8097/verify/NO-EXISTE`: `Receta no encontrada`; no patient, cédula, diagnosis, or medication label. Use a valid signed code to see `Receta auténtica` with only doctor, date, and number.
- Authenticated: `/patients` search/empty/create/detail/edit; `/templates` edit/publish/preview; `/prescriptions` filters/preview/sign; `/settings`, `/settings/clinic`, `/settings/users`, `/settings/profile`, `/audit` loaded as an administrator. Confirm focused controls show the ring, rows stack below 640, the three steps stack below 900, and only tables or A4 previews scroll horizontally. Tab through auth, gallery, patient form, template form, and the billing demo for visible focus.

- [ ] Split closure passes:
```
for name in \
  15-O-ui-tokens-actions-and-formatting \
  16-P-ui-forms-feedback-and-layout \
  17-Q-ui-brand-and-marketing-components \
  18-R-ui-domain-and-billing \
  19-S-ui-gallery-and-dev-route \
  20-T-shell-auth-home-and-verify \
  21-U-ui-dashboard-settings-and-patients \
  22-V-ui-templates \
  23-W-ui-prescriptions \
  24-X-ui-settings-detail-and-audit \
  25-Y-ui-legacy-css-cleanup-and-docs; do
  test -f "specs/done/${name}.md" || { echo "missing ${name}"; exit 1; }
done
```
Expected: exit 0 after all 11 files move. If the command fails, the named file has an open divergence: leave it in `specs/`, create or link `specs/divergences/` naming the blocking criteria, keep this box unchecked, and record the divergence in section 4.

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

At `13-M` the style sheet is a mix of design tokens (`:root` 1–58 lines), one-off selectors (69–874), and the A4 print projection (647–870). The UI work moves every page to `.ui-*`; the A4 block remains the print contract. This spec removes the middle layer so `.ui-*` and `.a4-*` are the only style languages.

The docs currently describe `main.rs` as the UI owner and `src/forms.rs` as the form module. They do not name `src/ui/` or `/dev/ui`. After this spec they do, and they say the UI module contains no server code. `context/PRD.md` and `context/Design.md` are requirements and design sources, not implementation status; they stay untouched. Billing stays v1.1 and reserved; no doc here claims otherwise.

## 6. Reuse, do not rebuild

| Existing symbol | Decision |
|---|---|
| `forms::A4Preview` | Keep. Only its `.a4-page` shadow token and Ink literal change; no layout, pagination, or geometry logic. |
| `forms::TemplateDocumentForm` | Keep the adopted form from `22-V`; no new styles. |
| `Navbar` | Keep the adopted shell from `20-T`; no new styles. |
| `Route`, `App`, page components | Keep. This spec deletes CSS, not routes or markup. |
| `forms::paginate_document`, `resolve_text`, document warnings | Keep unchanged. |
| `SyncStatus`, `probe_sync_status`, local count helpers | Keep unchanged. |
| `.a4-*` selectors | Keep every geometry selector. Only the shadow token and Ink literal are allowed to change. |
| `.ui-*` selectors and tokens | Keep. This spec may not delete a shared class still used by every page. |
| `format_*`, label helpers | Keep in `src/ui/format.rs`. |
| Old page-ID width rules | Replace with `.ui-page`/`.ui-container`; do not preserve them under new names. |

## 7. Component surface

No new component and no changed prop. This spec removes CSS only. The final stylesheet has four regions: `:root` tokens, the border-box reset, the `.ui-*` component/utility layer, and the `.a4-*` print projection. Debug gallery rules are part of `.ui-*`.

## 8. Legacy selectors to remove

Delete every rule whose selector contains one of the class names or IDs below, including compound, descendant, pseudo-class, pseudo-element, attribute, and media-query forms. Do not leave an empty rule or re-create a deleted class under a new name.

```
.button
.button.secondary
button.secondary
button.danger
.badge
.notice
.error
.hint
.page-head
.actions
.form-grid
.field
.field.wide
.detail-grid
.notes
.search-row
.patient-list
.patient-row
.patient-name
.patient-meta
.template-list
.template-row
.template-editor
.editor-section
.settings-columns
.break-anywhere
.variable-row
.medication-block
.inline-block
.preview-controls
.preview-scroll
.prescription-filters
.block-indicator
.warning
.medication-list
.item-card
.item-read
.llm-link
.clinic-logo
.asset-row
```

```
#echo
#hero
#links
#navbar
#patients-page
#templates-page
#settings-page
#prescriptions-page
#audit-page
```

Keep `.a4-*`, including `.a4-page`, `.a4-header`, `.a4-body`, `.a4-footer`, `.a4-qr`, `.a4-warnings`, `.a4-logo-image`, `.a4-image`, `.a4-doctor`, `.a4-clinic`, `.a4-page-no`, and `.a4-qr-placeholder`. Keep the A4 mm sizes and layout rules. Change only:

```
.a4-page {
  background-color: var(--color-paper);
  color: var(--color-ink);
  box-shadow: var(--shadow-paper);
}
```

The literal `#111111` and the old `0 2px 12px rgba(0, 0, 0, 0.18)` shadow must not appear. The `.a4-doctor` bold weight stays; it is intentional document typography, not a grenette headline.

Do not add any new selector. If a page still needs a width rule, its markup is wrong: use `PageShell id=...`, which already provides `.ui-page` and `.ui-container`.

## 9. Critical files and anchors

These are hints; reread before editing.

| File | Anchor |
|---|---|
| `persona-sanus/assets/main.css` | `:root` 1–58; legacy selectors 69–874; keep `.a4-*` 647–870 |
| `persona-sanus/src/main.rs` | `enum Route` 48–93; `App` 1222; all adopted pages |
| `persona-sanus/src/forms.rs` | `A4Preview` 463; `TemplateDocumentForm` 635; no raw controls after `22-V` |
| `context/architecture.md` | §3 Module map table; §2 compile-time split; §7 client and route shell |
| `context/sitemap.md` | §1.1–§1.3 route tables; §2 role matrix; §5 conventions |
| `persona-sanus/README.md` | after the run section, before environment variables |
| `specs/TEMPLATE.md` | sections 1–4 and housekeeping |

## 10. Assumptions and contingencies

- The final audits are run from the repo root, except the `cargo` block which starts with `cd persona-sanus`.
- `rg` exit status 1 means "no matches"; that is success for the five deletion audits. Any printed line is failure.
- If a legacy selector is still used by a page, migrate that page in its owning spec instead of deleting the selector. Do not leave both systems.
- If the A4 page shadow is intentionally different from `--shadow-paper`, record a divergence and keep the literal; do not change print geometry silently.
- If the debug server path differs, use `dx serve --fullstack --port 8095` for the debug proof and keep the same curl assertions. If a port is busy, change only the port and matching URL.
- Manual browser gate is required because SSR HTML cannot prove keyboard switching, focus rings, layout stacking, or the absence of console errors.
- If a split predecessor has an open divergence, leave that predecessor in `specs/` and create/link its tracking file. The final move applies only to specs with every objective and criterion checked.
- Do not edit `context/PRD.md`, `context/Design.md`, or billing status to make this spec pass. If a doc conflict appears, record a divergence and stop.

---

*Housekeeping: once this spec is fully done with no open divergences, move it
to `specs/done/25-Y-ui-legacy-css-cleanup-and-docs.md`. The tracking-file rule is
the only exception: leave it in `specs/` until `specs/divergences/` is cleared.*
