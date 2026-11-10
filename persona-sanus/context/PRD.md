# PersonaSanus — PRD: Clinical Prescription Generator

| Field | Value |
|---|---|
| Product | PersonaSanus — prescription generator for doctors |
| Status | Draft v0.1 — 2026-11-10 |
| Platforms | Desktop (macOS, Windows) offline-first + universal Web (Chromium) — same experience |
| Current stack | Dioxus 0.7.1 (`router`, `fullstack`) — `persona-sanus/Cargo.toml` |
| Backend | SpaceTimeDB self-hosted with Docker (`docker compose`) on our own infrastructure |
| Market | Ecuador — Spanish (`es-EC`), Ley Orgánica de Protección de Datos Personales + applicable health regulations (validate with legal counsel) |
| v1 scope | Multi-user clinic: doctors + nursing/secretary staff, shared patients and templates; prescriptions only |
| Output | A4 PDF via Chromium print-to-PDF — download or direct print; QR verification |
| Editor | Rich-text editor over an A4 template, variables, per-doctor/per-clinic branding |

## 1. Problem and solution

Small/medium clinics in Ecuador produce prescriptions on paper, in Word, or from loose templates: inconsistent format, no synced history between consultation and front desk, and no way to verify authenticity.

**Solution:** a desktop+web application where each doctor works with their own templates (logo, letterhead, signature, layout) and dynamic variables, edits the prescription in a rich-text editor with a true A4 preview, issues a PDF ready to download/print with a verifiable QR code, and keeps everything synced in SpaceTimeDB — including offline work on the desktop.

## 2. Goals / non-goals

**v1 goals:**
- Issue a signable prescription in < 60 s from the consultation (template → patients/medications → PDF).
- Identical UX and data on desktop and web; desktop works without a network connection and syncs on reconnect.
- Every PDF is publicly verifiable by QR without exposing sensitive patient data.
- Reusable templates that preserve each doctor's and the clinic's personal identity.

**Non-goals v1:**
- No diagnosis support (DDX) / interaction checks, no official drug database (medications are free text — see §9), no full medical record, no sick leave/referrals/reports, no billing, no EHR/HL7-FHIR integration, no patient app, no qualified eIDAS-style signature (validate against current Ecuadorian regulations).
- No Linux desktop in v1 (macOS + Windows only).

## 3. Users and roles

| Role | Access |
|---|---|
| **Administrator** | Users, roles, clinic settings, master templates, access revocation, audit. |
| **Doctor** | Create/edit own prescriptions and patients, create and version own templates, download/print PDFs, view own history. |
| **Nursing** | Create prescription drafts, edit vitals/medication fields, cannot sign or publish. |
| **Secretary** | Manage patients, reprint/download issued prescriptions, scheduling, cannot edit signed clinical content. |
| **Public verifier** | No account: scans the QR → `#/verify/:code` page confirming authenticity plus minimal data (doctor, date, prescription number). |

Permissions are enforced in SpaceTimeDB reducers (never client-side only). Signed/published prescriptions are immutable: any correction creates a new version that references the previous one.

## 4. Main flows

1. **Issue prescription (online):** Login → search/create patient → choose template → rich-text editor (variables pre-loaded) → complete medications → A4 preview → *Sign and issue* → PDF generated (Chromium print-to-PDF) → Download / Print / Share options.
2. **Issue prescription (offline, desktop):** same flow against the local cache; the prescription queues in an outbox with `pending_sync` status; on reconnect, reducers upload it with an idempotent `client_id`. Conflicts: server wins for signed data; only an in-progress draft warns of a conflict and requires a choice.
3. **Personal template:** Doctor uploads logo/stamp → defines letterhead/footer (rich-text) → saves template → insertable variables (`{{patient.name}}`, `{{doctor.name}}`, `{{date}}`, `{{prescription.number}}`) → A4 preview → publish (new immutable version).
4. **Verification:** patient/printer scans the QR → web opens the verification page → sees "Receta auténtica — issued by Dr. …, date …, No. …" plus the hash. No PHI.
5. **Re-issue / download history:** search history → *Download PDF* (regenerates from the final versioned document, identical to the issued one). Only ≤ 1 month lives in the desktop local cache; the rest downloads on demand (current product decision).
6. **Explain medication:** next to each medication line, an *"Explain with LLM"* link opens an LLM page in a new tab with the medication name pre-filled (medication only, never patient data) + disclaimer.

## 5. Functional requirements

### 5.1 Patients
- FR-001: Patient CRUD (names, cédula, age/birth date, sex, phone, notes). Search by name and cédula.
- FR-002: Deduplication by cédula; warn on likely duplicate creation.
- FR-003: A patient's prescription history is visible only to authorized roles in the clinic.

### 5.2 Prescription
- FR-010: Create a prescription from patient + template; fields: date/time, diagnosis (text), general instructions (rich-text), medications (draggable/reorderable), validity/recommendations.
- FR-011: Each medication: name (free text), dose, route, frequency, duration, quantity, notes (all free text, no validation against an official database).
- FR-012: Per-medication "Explain with LLM" link (configurable via `LLM_URL_TEMPLATE`; default: generic chat). Opens with the drug name only; never with PHI. The app never sends data to any LLM by itself.
- FR-013: States: `draft → signed(published) → superseded/voided`. Signing locks editing. Voiding requires a reason and leaves an audit trail.
- FR-014: Sequential prescription number per clinic, unique and gap-free in one network, assigned by a reducer at signing time.

### 5.3 Templates and rich-text editor
- FR-020: Per-doctor templates (and clinic master templates): letterhead (logo, doctor/clinic details, colors) + body + footer (signature/stamp, QR, contact). Images uploaded to asset storage referenced by `asset_id`.
- FR-021: WYSIWYG rich-text editor: bold, italic, underline, lists, tables, alignment, headings, variable insertion, and medication blocks. Documents stored as versioned structured JSON (never unsanitized raw HTML).
- FR-022: True-scale A4 preview: 210×297 mm, configurable margins, page breaks, repeated header/footer, embedded typography.
- FR-023: Supported v1 variables: `patient.{name,national_id,age}`, `doctor.{name,specialty,national_id,contact}`, `clinic.{name,address,phone}`, `prescription.{number,date,diagnosis}`, `medications[]`. Unknown variables → pre-signing warning, never a literal `{{key}}` in the PDF.
- FR-024: Publishing a template creates an immutable version; already-issued prescriptions keep the version they used.

### 5.4 A4 PDF, download, print, QR
- FR-030: Single pipeline: editor-identical HTML/CSS rendering → Chromium `print-to-PDF` (`--print-to-pdf` / `page.pdf()`), unscaled A4, template background colors, embedded fonts. No server-side PDF library (current decision).
- FR-031: Output: Download `.pdf`, Print from the system viewer/PDF, Share (web).
- FR-032: **File name:** `{PatientName}-{receta}-{localized_date}-{DoctorName}.pdf`. Example: `Juan Perez-receta-10-11-2026-Dr Ana Ruiz.pdf`. Sanitize `/ \ : * ? " < > |`, collapse whitespace, cap at 120 characters without truncating key fields.
- FR-033: Every PDF carries a QR (per current decision) encoding `https://<domain>/verify/{code}` + an opaque per-version `code`; the QR contains no PHI (name, cédula, diagnosis, or medications).
- FR-034: PDF header/footer: prescription number, QR, verification code printed as readable text (for re-typing), and issue date/time.

### 5.5 Sync, offline and local retention (desktop)
- FR-040: Offline-first desktop: patients, published templates, and prescriptions (drafts + signed) are accessible offline after first login. Outbound operation queue with retries and visible status.
- FR-041: Real-time sync via SpaceTimeDB subscriptions (online mode); on reconnect, drain the outbox with idempotent reducers (`client_id`).
- FR-042: **Local retention:** only prescriptions from the last **30 days** are kept locally (configurable calendar month). On expiry they are evicted from the local cache (not deleted from the server); on-demand re-download with a *Download* button.
- FR-043: UX indicators: online / offline / syncing / pending uploads (count). Never hide data loss: if a signed operation could not be uploaded, the PDF carries a `PENDIENTE DE SINCRONIZACIÓN` marker and the QR activates only after sync.

### 5.6 Public verification
- FR-050: Public endpoint without authentication: code → status (valid / voided / superseded / not found) + doctor, date, prescription number (no PHI). Per-IP rate limit + verification logging (audit, without storing raw IPs beyond what the law allows).

## 6. Data model (SpaceTimeDB)

Tables (all with standard `id`, `created_at`, `updated_at`, `created_by`):

| Table | Key fields |
|---|---|
| `users` | `display_name`, `role`, `status`, `spacetime_identity` |
| `doctors` | `user_id`, `specialty`, `license_no`, `signature_asset_id`, `phone`, `email` |
| `clinics` | `name`, `address`, `phone`, `logo_asset_id`, `timezone`, `llm_url_template`, `retention_days=30` |
| `clinic_members` | `clinic_id`, `user_id`, `role` |
| `patients` | `clinic_id`, `full_name`, `national_id`, `birth_date`, `sex`, `phone`, `notes` |
| `templates` | `owner_type(doctor|clinic)`, `owner_id`, `name`, `current_version_id`, `status` |
| `template_versions` | `template_id`, `version_no`, `doc_json` (editor), `variables_spec`, `theme`, `status` |
| `prescriptions` | `patient_id`, `doctor_id`, `clinic_id`, `template_version_id`, `status`, `rx_number` (at signing), `diagnosis`, `instructions_json`, `signed_at`, `client_id` (idempotency), `supersedes_id` |
| `prescription_items` | `prescription_id`, `position`, `med_name`, `dose`, `route`, `frequency`, `duration`, `quantity`, `notes` |
| `verification_codes` | `prescription_id`, `code_hash`, `status`, `created_at` |
| `audit_log` | `actor_id`, `action`, `entity`, `entity_id`, `metadata_json`, `at` |

**Reducers (authorized mutations):** `register_user`, `create_patient`, `update_patient`, `create_template`, `publish_template_version`, `create_prescription(draft)`, `upsert_prescription_item`, `sign_prescription`, `void_prescription`, `create_verification_code`, `request_pdf_regeneration`, `evict_local_cache_row` (client-only), plus transactional `get_rx_number` at signing time.

**Reads:** subscriptions scoped by `clinic_id`/`doctor_id`/`patient_id` with role filters; PHI only flows to authorized roles. Sensitive-read audit: log access to patients/prescriptions (who/when/what) — a healthcare traceability requirement.

## 7. Technical architecture

| Layer | Decision |
|---|---|
| Client | Dioxus 0.7.1, single codebase for web + desktop (features `web` / `desktop`). Routes: `/`, `/patients`, `/prescriptions/:id`, `/templates`, `/verify/:code`. |
| State | Signals/context for session, role, sync status; `use_resource`/`use_server_future` for initial loads. Identical UI on desktop/web. |
| Editor | Content-editable component (embedded Tiptap/ProseMirror via JS interop) emitting JSON; imported HTML is sanitized; persisted as `doc_json`. |
| PDF | Chromium print-to-PDF from the 1:1 A4 print view; single shared HTML/CSS source for editor and printed output. Fallback: system print dialog. |
| Backend | SpaceTimeDB in Docker (compose: `spacetimedb` + Caddy/Nginx with TLS + persistent volume + encrypted daily backups). SpaceTimeDB identity + `users` table. |
| Sync | Rust SpaceTimeDB SDK on desktop; local cache (SQLite) + outbox; web always online-first (with a minimal queue if the network drops). |
| Assets | Images (logos/stamps) in private object storage; short-lived signed URLs. |
| LLM link | `clinics.llm_url_template`, e.g. `https://chat.openai.com/?q={med}` — filled only with `med_name`, explicit user click. |

> Note: SpaceTimeDB already syncs over websocket subscriptions; desktop "offline-first" is built on top (cache + outbox + idempotency), not native to SpaceTimeDB — this is the main implementation risk.

## 8. Non-functional requirements

- **Ecuador locale/format:** 100% `es-EC` UI; `DD-MM-AAAA` dates in filenames and UI (`es-EC`), 24-hour time, currency N/A in v1. The 10-digit cédula is the preferred identifier.
- **Performance:** create/edit a prescription < 2 s on the national network; PDF generated < 3 s; offline-first desktop is immediate (no network spinner to view recent history).
- **Availability:** basic HA SpaceTimeDB on a Docker host (restart policies, healthchecks); RPO ≤ 24 h (daily backup), RTO ≤ 4 h in v1 (no critical medical SLA).
- **Accessibility:** AA contrast, keyboard navigation in the editor and PDF viewer, visible focus.
- **Compatibility:** macOS 13+, Windows 10/11 (WebView2), recent Chromium browsers (Chrome/Edge). Standard A4 printers; detect and warn on non-A4 paper.
- **Traceability:** audit log with a minimum legal retention (define with counsel — initial suggestion 5 years, to confirm).

## 9. Security, privacy and compliance (Ecuador)

- **Reference framework:** Ley Orgánica de Protección de Datos Personales (2021) and its regulations, MSP/ACESS rules on clinical records. **Required action:** local legal review before production — this PRD is not a substitute for legal advice (e.g. retention periods, consent criteria, legitimacy of public verification).
- Minimization: PHI (patient, diagnosis, medications) never travels to an external LLM, into a QR, or into URLs. URLs use only an opaque `code`; never names/cédulas.
- Encryption: TLS 1.2+ in transit; at-rest encryption on disk/Docker volumes; encrypted desktop cache (OS disk encryption / vault).
- Access: MFA for doctors/administrators; expiring sessions; registered and revocable desktop devices.
- ARCO rights (access/rectification/cancellation/opposition): search, export, and deletion of patients through an audited flow; logical deletion on the server (hard delete only when no retention obligation applies).
- PHI in PDFs: downloaded PDFs are the doctor's responsibility (shared devices → warning + auto-cleanup on desktop after print/download).

## 10. Success metrics

- End-to-end issue time < 60 s (90th percentile).
- % of created prescriptions fully synced within < 5 s after reconnect.
- % of PDFs successfully verified on first attempt (readable QR) > 98%.
- Adoption: ≥ 5 active doctors/week within 3 months; > 80% of active doctors create a personal template.

## 11. v1 roadmap

- **M0 (foundation):** auth + roles, patient CRUD, SpaceTimeDB in Docker, CRUD reducers.
- **M1 (editor + PDF):** templates + JSON editor + variables + A4 preview + Chromium PDF + filename + QR.
- **M2 (offline):** desktop cache/outbox, 30-day retention, on-demand re-download, sync indicators.
- **M3 (hardening):** audit, MFA, backups, verification rate limiting, QA with real A4 printers.

## 12. Acceptance criteria (v1)

- [ ] Doctor creates a patient, picks their template, edits with rich-text, signs, and gets an A4 PDF with their logo and QR.
- [ ] The same flow works offline on desktop and syncs on reconnect with a unique prescription number.
- [ ] A PDF ≥ 31 days old is not in the local cache; it re-downloads on demand, identical to the issued one.
- [ ] Scanned QR shows "Receta auténtica" without exposing patient data.
- [ ] Medication shows an LLM link with the drug name only.
- [ ] Filename follows `{Patient}-receta-{dd-mm-aaaa}-{Doctor}.pdf` in `es-EC`.
- [ ] Secretary cannot edit a signed prescription; nursing cannot sign one; everything is audited.

## 13. Risks and open decisions

- Dioxus 0.7 integration (CSS/JS interop + Rust SpaceTimeDB SDK + desktop WebView) is non-trivial — mitigate with an editor spike before M1.
- "What you see is the PDF" fidelity: any editor↔Chromium mismatch is fixed in the print HTML/CSS, never with a second template.
- Free-text medication errors (spelling, dose): without an official database there is clinical risk — visible disclaimer and mandatory doctor verification; Phase 2: local catalog / CIMA-like database.
- SpaceTimeDB in Docker on our own infrastructure: backups, TLS, and updates become our responsibility (runbook required).
- Confirm with Ecuadorian counsel: prescription/audit retention, formal legal-validity requirements for the PDF/QR, and MSP/ACESS requirements for clinics.

---
*Note: the repository already contains the Dioxus + router + fullstack base — this PRD is implemented on top of it, not as a new project.*
