# Spec 08 — Phase H: PDF renderer and A4 preview

- Source: `context/PRD.md` §5.4 (FR-030–034), D-01; `context/architecture.md` §2 (`printpdf` features), §11 (pipeline), §18 (PDF gate); `context/sitemap.md` §1.2 (`/prescriptions/:id/preview`), §1.3 (PDF actions).
- Depends on: 05-E, 06-F, 07-G.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Result carried forward: one shared pure `printpdf` renderer turns a pinned document + variables into an A4 PDF with QR and `es-EC` filename on the server route and (later) in the desktop binary, and `/prescriptions/:id/preview` shows the true-scale projection.

## Before you start

Unit testing is not required; avoid it. Prioritize end-to-end and integration
testing only.

Read the nearest `AGENTS.md` (target workspace, then parents up to repo root)
before executing.

If reality diverges from this spec: STOP, append a divergence (section 4), and
NEVER improvise outside the objectives.

## 1. Objectives

*The ordered work to implement — one item per step. Check `[x]` only after the
step is implemented and verified.*

*Item format:* `<verb phrase>` — requirements: `<what must change / constraints>`; commands: `<exact command>`.

- [ ] 1. Add the `printpdf` renderer module — requirements: `printpdf` pinned at an exact version with `default-features = false` and the `text_layout`/`jpeg`/`png` features on the `server` and `desktop` features only (never `web`) — verify the pinned release actually ships those feature names before adding the dependency; `src/pdf.rs` is pure (no SQL, no HTTP, no UI, no filesystem, no clock beyond the passed issue timestamp) and takes the pinned `doc_json` layout model (the structured document produced by spec 05-E's form) + pinned asset bytes + variables + a render mode; `PdfDocument::from_html` is never called; commands: `cargo check --manifest-path persona-sanus/Cargo.toml`
- [ ] 2. Render A4 output with QR — requirements: one `PdfPage` per 210×297 mm sheet with explicit ops: embedded TTF text (one named open-licensed font — Noto Sans or DejaVu Sans — vendored under `persona-sanus/assets/fonts/` with its license file; no host fonts), rules, JPEG/PNG XObjects, computed page breaks, repeated header/footer, configurable margins; header/footer carry the prescription number, the QR, the readable verification code (FR-034), and a printed signature line; the QR encodes `{PUBLIC_BASE_URL}/verify/{code}` (architecture §14) as an embedded PNG (`qrcode-generator` 6.1.2 → PNG via printpdf's `png` feature) and contains no PHI (FR-033); provisional mode draws `PENDIENTE DE SINCRONIZACIÓN`, a placeholder QR frame, and no scannable code; render calls run in `spawn_blocking`; commands: `cargo check --manifest-path persona-sanus/Cargo.toml`
- [ ] 3. Serve the PDF route — requirements: `src/pdf_server.rs` with `GET /api/prescriptions/:id/pdf` (session + role + clinic authorization; `draft` rejected `422`; signed/superseded/voided render from the pinned `template_versions.doc_json` + `template_version_assets` + items + stored `verification_codes.code`, never the current template); responds `application/pdf` with `Content-Disposition: attachment; filename="{Patient}-receta-{DD-MM-AAAA}-{Doctor}.pdf"` per FR-032 (sanitize `/ \ : * ? " < > |`, collapse whitespace, cap 120 chars) — the header must use `filename*=UTF-8''` percent-encoding (RFC 5987) with an ASCII `filename` fallback so names like `Pérez` survive — and the same sanitized title; PDF bytes are never stored; commands: `cargo check --manifest-path persona-sanus/Cargo.toml`
- [ ] 4. Build the preview route — requirements: `/prescriptions/:id/preview` shows the true-scale A4 browser projection of the same layout model (210×297 mm, margins, page breaks, repeated header/footer) — reusing the shared projection component from spec 05-E — with actions *Sign and issue* (from draft), *Download PDF*, *Print* (system viewer with the same bytes), *Share* (Web Share API on web); generation happens only on explicit user action; commands: `cd persona-sanus && dx build --release --fullstack`
- [ ] 5. Create `.blocks/pdf-probe` — requirements: new block (header per `.blocks` rules, indexed in `AGENTS.md` and `.blocks/INDEX.md`) composing `persona-probe --flow rx` (invoked as a subprocess, fixtures parsed from its JSON verdict) and then asserting on `GET /api/prescriptions/:id/pdf`: `200`, `application/pdf`, `%PDF-` magic, `MediaBox` floats within ±1 pt of 595.28×841.89 (A4 in points — no literal `[0 0 595 842]` string match), sanitized `Content-Disposition` matching `{Patient}-receta-{DD-MM-AAAA}-{Doctor}.pdf` (asserting the RFC 5987 `filename*`), and the QR bitmap decodes to the `/verify/{code}` URL; PDF parsing and QR decoding use a vendored pure-Python approach or a dependency declared in the block header (Pillow/pypdf are not installed); also asserts a `draft` returns `422`; commands: `python3 .blocks/pdf-probe.py --base http://127.0.0.1:8080`

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- Never call `from_html` or add a second template/layout engine; fidelity fixes belong in `src/pdf.rs`.
- No `printpdf` in the `web` feature; no Chromium, headless browser, or print-to-PDF dialog.
- No `/verify/:code` page (spec 09-I); no offline/desktop rendering (spec 10-J calls this module in-process).
- No PDF storage columns or files on the server.
- No unit tests; no comments in any code.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [ ] `python3 .blocks/persona-smoke.py --dir persona-sanus --port 8091` passes (fmt, check, clippy, release build, auth probe)
- [ ] objective 5: `python3 .blocks/pdf-probe.py --base http://127.0.0.1:8080` → all checks true (`200`, `%PDF-`, MediaBox floats within ±1 pt of 595.28×841.89, filename, QR payload, `422` on draft)
- [ ] objective 2: the rendered PDF embeds fonts (no host font dependency): `cd persona-sanus && dx build --release --fullstack` output serves an identical render from a container without fonts (persona-docker-verify stack)
- [ ] objective 4: `http://127.0.0.1:8080/prescriptions/:id/preview` in a browser → true-scale A4 projection matches the PDF content page-by-page for the same fixture (manual side-by-side; a golden A4 fixture checked into `persona-sanus/` guards regressions)
- [ ] objective 3: re-issuing from `/prescriptions` reproduces the same content from the pinned version (diff of extracted text is empty)
- [ ] every objective in section 1 is checked

## 4. Divergences

*Process and tracking-file rules: see `AGENTS.md` (§ Spec-Driven Development).
Summary: when reality diverges from the spec, append a numbered entry below;
leave the matching testing-criteria `- [ ]` unchecked and note the divergence
inline next to it. Open divergences owned by a later spec also get a tracking
file in `specs/divergences/`.*

- [ ] every divergence found is appended below (or the section reads "None.")

*Entry format (one numbered item per divergence):*

1. <what diverged and its current state> — <why it could not be done as written, citing the No-go rule or spec that blocks it> (Verified <date>: <N> fix attempts — <attempt 1>, <attempt 2>, <attempt 3> — all fenced by <constraint>; <impact> is deferred to <later spec> by design, not a defect.)

### Divergences

None.

---

*Housekeeping: once a spec is fully done with no open divergences, move its
file to `specs/done/` (e.g. `specs/done/08-H-pdf-renderer-and-preview.md`).
Specs with open divergences stay in `specs/` next to `TEMPLATE.md` until the
tracking file is cleared.*
