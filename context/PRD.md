# PersonaSanus — PRD: Clinical Prescription Generator

| Field | Value |
|---|---|
| Product | PersonaSanus — prescription generator for doctors |
| Status | Draft v0.8 — 2026-11-12 (§14 decision log; §15 remaining open questions) |
| Platforms | Desktop (macOS, Windows) offline-first + universal Web (Chromium) — same experience |
| Current stack | Dioxus 0.7.1 (`router`, `fullstack`) + axum 0.8 + Diesel 2.3 (`sqlite`, `r2d2`) + Argon2 0.6 — `persona-sanus/Cargo.toml`; v1 adds `printpdf` 0.12 (PDF renderer, planned M1, not yet a dependency) |
| Backend | Single-node SQLite + app-owned identity (Argon2 passwords, hashed opaque sessions), self-hosted with Docker Compose on our own infrastructure (`docker-compose.yml`) |
| Market | Ecuador — Spanish (`es-EC`), Ley Orgánica de Protección de Datos Personales + applicable health regulations (validate with legal counsel) |
| v1 scope | Multi-user clinic: doctors + nursing/secretary staff, shared patients and templates; prescriptions only |
| Billing | **Deferred to v1.1.** v1 is a free beta with no payment provider and no entitlement gate; the v1.1 subscription uses **PayPhone Tokenization** (card-on-file, Visa/Mastercard) — §5.7, `https://docs.payphone.app/tokenizacion` |
| Storage | Image library: 1 image per clinic in v1 (free beta), 10 MB (10,485,760 bytes) each; the clinic logo is a separate branding asset and does not consume the slot. v1.1 raises the library to 5 with billing |
| Output | A4 PDF rendered in Rust by `printpdf` 0.12 — linked into the server binary (web) and the desktop binary (offline); QR verification. No Chromium, no headless browser, no print-to-PDF dialog |
| Editor | Form-based editing over an A4 template (no rich text — D-21), variables, per-doctor/per-clinic branding |

## 1. Problem and solution

Small/medium clinics in Ecuador produce prescriptions on paper, in Word, or from loose templates: inconsistent format, no synced history between consultation and front desk, no way to verify authenticity, and no simple way to keep a verified history available across devices.

**Solution:** a desktop+web application where each doctor works with their own templates (logo, letterhead, layout) and dynamic variables, edits the prescription in a form with a true A4 preview, issues a PDF ready to print (the doctor signs the printed paper) with a verifiable QR code, and keeps everything in a single SQLite backend — with the desktop continuing to work offline (local cache + outbox) and syncing on reconnect. Server sync is free in v1; the $5/month or $50/year subscription (3-month trial) is designed for v1.1 on PayPhone Tokenization and never blocks clinical work. v1 includes a private image library of one 10 MB image per clinic for template branding; v1.1 raises it to five.

## 2. Goals / non-goals

**v1 goals:**
- Issue a signable prescription in < 60 s from the consultation (template → patients/medications → PDF).
- Identical UX and data on desktop and web; desktop works without a network connection and syncs on reconnect.
- Every PDF is publicly verifiable by QR without exposing sensitive patient data.
- Reusable templates that preserve each doctor's and the clinic's personal identity.
- Clinical work never depends on connectivity: the desktop drafts, signs with a preallocated number, and prints PDFs offline; sync resumes on reconnect.
- Private image storage: one 10 MB image per clinic in v1, reusable in templates (five in v1.1).

**Non-goals v1:**
- No diagnosis support (DDX) / interaction checks, no official drug database (medications are free text — see §9), no full medical record, no sick leave/referrals/reports, no clinical/medical billing, no insurance claims, no SRI electronic invoicing, no EHR/HL7-FHIR integration, no patient app, no qualified eIDAS-style signature (validate against current Ecuadorian regulations). Subscription billing and payment collection are designed for v1.1 (§5.7) and are not part of v1.
- No Linux desktop in v1 (macOS + Windows only).

## 3. Users and roles

| Role | Access |
|---|---|
| **Administrator** | Users, roles, clinic settings, master templates, image library, access revocation, audit, read-only clinic-wide patient/prescription history. |
| **Doctor** | Create/edit own prescriptions and patients, create and version own templates, upload images for own templates, print/save PDFs, view own history. |
| **Nursing** | Create/edit patients, create prescription drafts, edit medication fields, cannot sign or publish. |
| **Secretary** | Manage patients, reprint/save issued prescriptions, cannot edit signed clinical content. |
| **Public verifier** | No account: scans the QR → `/verify/:code` page confirming authenticity plus minimal data (doctor, date, prescription number). |

Permissions are enforced server-side in the authorized service layer (never client-side only). Each user belongs to exactly one clinic in v1; `clinic_members.role` is the only role authority (D-03). Signed/published prescriptions are immutable: any correction creates a new version that references the previous one. Administrator and Doctor can upload library images (Administrator clinic-wide; Doctor for own templates); Nursing and Secretary can only view images referenced by templates they can open. Image uploads follow the flat v1 quota (FR-080, one image per clinic). Signatures are never stored as images: the doctor signs the printed prescription on paper (D-06).

## 4. Main flows

1. **Issue prescription (online):** Login → search/create patient → choose template → fill the prescription form (variables pre-loaded) → complete medications → A4 preview → *Sign and issue* → the number is assigned from the device's preallocated block and a verification code is created → PDF rendered by `printpdf` (download or *Print*) → sign the printed paper.
2. **Issue prescription (offline, desktop):** same flow against the local cache; signing consumes the next number in the device's preallocated block (FR-014) and the prescription queues in an outbox with `pending_sync` status; the PDF is provisional (`PENDIENTE DE SINCRONIZACIÓN`, inactive QR). On reconnect, the sync API uploads it with an idempotent `client_id`, reconcile the block, and the final numbered PDF with an active QR is regenerated. If the block is exhausted, signing is blocked until the device reconnects. Conflicts: server wins for signed data; only an in-progress draft warns of a conflict and requires a choice.
3. **Personal template:** Doctor uploads a logo → fills in the letterhead/footer fields → saves template → insertable variables (`{{patient.name}}`, `{{doctor.name}}`, `{{date}}`, `{{prescription.number}}`) → A4 preview → publish (new immutable version). No signature or stamp images (D-06).
4. **Verification:** patient/printer scans the QR → web opens the verification page → sees "Receta auténtica — issued by Dr. …, date …, No. …" plus the status. No PHI.
5. **Re-issue / download history:** search history → *Download PDF* / *Print* (regenerated from the final versioned document by the same `printpdf` renderer, same content as issued; not guaranteed byte-identical if the renderer version changed). Only ≤ 30 days of server-backed rows live in the desktop local cache; the rest downloads on demand (current product decision). Unsynced rows are never evicted.
6. **Explain medication:** next to each medication line, an *"Explain with LLM"* link opens an LLM page in a new tab with the medication name pre-filled (medication only, never patient data) + disclaimer.
7. **Subscribe / renew (v1.1, not in v1):** Administrator opens *Settings → Subscription* → sees trial/status → picks $5 monthly or $50 yearly → gives explicit consent to store a card token → is redirected to the PayPhone payment button/box → pays with Visa or Mastercard → PayPhone returns the `ctoken` (card token) and cardholder data to the response URL → the server confirms the charge, stores the token, and activates the plan. Later periods are charged server-side with the stored `ctoken` — no per-period approval. The full design is retained in §5.7 and ships in v1.1.
8. **Upload images:** Administrator or Doctor opens *Clinic settings* or a template → uploads one library image (10 MB; logo is separate) → server validates the format, strips metadata, checks the quota, and stores the object in the private MinIO bucket → inserts the image into a template → publishing pins it to that immutable version. Upload is online-only. The five-image, state-based behavior belongs to v1.1 (§5.8).

## 5. Functional requirements

### 5.1 Patients
- FR-001: Patient CRUD (names, cédula, age/birth date, sex, phone, notes). Search by name and cédula.
- FR-002: Deduplication by cédula; warn on likely duplicate creation.
- FR-003: A patient's prescription history is visible only to authorized roles in the clinic.

### 5.2 Prescription
- FR-010: Create a prescription from patient + template; fields: date/time, diagnosis (text), general instructions (text), medications (draggable/reorderable), validity/recommendations.
- FR-011: Each medication: name (free text), dose, route, frequency, duration, quantity, notes (all free text, no validation against an official database).
- FR-012: Per-medication "Explain with LLM" link (per clinic via `clinics.llm_url_template`; default: generic chat). Opens with the drug name only; never with PHI. The app never sends data to any LLM by itself.
- FR-013: States: `draft → signed(published) → superseded/voided`. Signing locks editing and is allowed offline against a preallocated number (FR-014); an offline-signed document is provisional (no server copy, inactive QR) until sync reconciles it. Voiding requires a reason and leaves an audit trail. A correction creates a new prescription row with its own number from a block and its own verification code; the replaced row keeps its number and becomes `superseded`.
- FR-014: Prescription number per clinic, unique and monotonically increasing across the network, assigned from preallocated per-device blocks. A device reserves a block (default 50 numbers; configurable) in a `BEGIN IMMEDIATE` transaction that advances `rx_counters.last_rx_number`, and signs offline by consuming the next number in its active block. A web client registers a `devices` row with `platform='web'` before its first allocation and uses the same block flow. Numbers are **not contiguous**: the unused tail of a block is burned and recorded in `rx_blocks`, never reused, so gaps are expected and legitimate (D-02). On sync the server validates that the number belongs to a block issued to that device and is unused, rejects duplicates via the unique index, and records the consumption. A device with an exhausted block cannot sign until it reconnects and refills; the UI shows the remaining count and warns below 10.

### 5.3 Templates and form-based editing
- FR-020: Per-doctor templates (and clinic master templates): letterhead (logo, doctor/clinic details, colors) + body + footer (printed doctor name/license, a blank space where the doctor signs on paper, QR, contact). Images uploaded to asset storage referenced by `asset_id`. Clinics can insert the library image(s) up to their v1 quota (FR-080); publishing pins every referenced asset to the version. No signature or stamp images are stored (D-06).
- FR-021: Form-based editing (plain Dioxus forms shared by web, macOS, and Windows — no WYSIWYG editor, no rich text, D-21): letterhead fields (logo, doctor/clinic details, colors), body text, footer fields, variable insertion, image slots, and medication rows. The form emits schema-checked structured JSON persisted as `doc_json` (never raw HTML).
- FR-022: True-scale A4 preview: 210×297 mm, configurable margins, page breaks, repeated header/footer, embedded typography. The preview is a browser projection of the same structured document; the `printpdf` render (FR-030) is authoritative for the issued PDF.
- FR-023: Supported v1 variables: `patient.{name,national_id,age}`, `doctor.{name,specialty,national_id,contact}`, `clinic.{name,address,phone}`, `prescription.{number,date,diagnosis}`, `medications[]`. Unknown variables → pre-signing warning, never a literal `{{key}}` in the PDF.
- FR-024: Publishing a template creates an immutable version; already-issued prescriptions keep the version they used.

### 5.4 A4 PDF, download, print, QR
- FR-030: Single pipeline: `printpdf` 0.12 renders the PDF in Rust from the pinned structured document (`template_versions.doc_json`) plus the variable set, page by page at A4 (210×297 mm) with explicit placement ops — text, shapes, embedded TTF fonts, JPEG/PNG images, and the QR generated by `qrcode-generator` 6.1.2 as an embedded PNG. The renderer is one shared module linked into both the `server` build (web: `GET /api/prescriptions/:id/pdf`) and the `desktop` build (local rendering, so PDF generation works offline — D-01). No Chromium, no headless browser, no print-to-PDF dialog, and never printpdf's experimental HTML layout (`from_html`) for issued output.
- FR-031: Output: *Save as PDF* downloads the rendered bytes with `Content-Disposition: attachment`; *Print* hands the same bytes to the system viewer/print dialog; *Share* uses the Web Share API on web. Generation runs only on an explicit user action — never as a silent background download or write.
- FR-032: **File name:** `{PatientName}-{receta}-{localized_date}-{DoctorName}.pdf`. Example: `Juan Perez-receta-10-11-2026-Dr Ana Ruiz.pdf`. Sanitize `/ \ : * ? " < > |`, collapse whitespace, cap at 120 characters without truncating key fields; applied to the `Content-Disposition` filename (and to the title of the opened PDF document).
- FR-033: Every **final** PDF carries a QR encoding `https://<domain>/verify/{code}` + an opaque per-version `code`; the QR contains no PHI (name, cédula, diagnosis, or medications). A provisional offline PDF shows a placeholder QR area plus `PENDIENTE DE SINCRONIZACIÓN` and no scannable code; the active QR appears only after sync reconciles the number and assigns the verification code (FR-014, FR-043).
- FR-034: PDF header/footer: prescription number, QR, verification code printed as readable text (for re-typing), issue date/time, and a printed signature line for the doctor to sign on paper.

### 5.5 Sync, offline and local retention (desktop)
- FR-040: Offline-first desktop: patients, published templates, and prescriptions (drafts + signed) are accessible offline after first login. Outbound operation queue with retries and visible status. The desktop cache relies on OS disk encryption (FileVault/BitLocker) as a deployment prerequisite; no app-level database encryption in v1 (D-07).
- FR-041: Real-time sync via SSE served by the existing axum router, with a 30-second polling fallback when SSE is unavailable (no second socket server). On reconnect, drain the outbox oldest-first with idempotent REST mutations (`client_id`), reconcile preallocated `rx_blocks` (FR-014), and refresh the cached number block and metadata.
- FR-042: **Local retention:** only server-backed prescriptions from the last **30 days** are kept locally (configurable in days via `clinics.retention_days`). On expiry they are evicted from the local cache (not deleted from the server); on-demand regeneration with a *Download PDF* / *Print* button. Rows with **no server copy** (created offline, still in the outbox, or never synced) are never evicted — they are the only copy.
- FR-043: UX indicators: online / offline / syncing / pending uploads (count) / provisional documents. Never hide data loss: an offline-signed prescription is provisional — its PDF carries `PENDIENTE DE SINCRONIZACIÓN`, no active QR, and no final number until sync reconciles it; after sync the final PDF is regenerated. Billing-driven states (`sync_locked`, `SINCRONIZACIÓN BLOQUEADA`) are v1.1 behavior (§5.7) and are not implemented in v1.

### 5.6 Public verification
- FR-050: Public endpoint without authentication: code → status (valid / voided / superseded / not found) + doctor, date, prescription number (no PHI). A provisional offline prescription has no verification code and therefore no public record until sync. Per-IP rate limit + verification logging (audit, without storing raw IPs beyond what the law allows).

### 5.7 Subscription, trial and payments (v1.1 — not in v1)

**Scope: v1.1.** v1 ships as a free beta with no billing tables, entitlement gates, jobs, or UI (D-04). The design below is retained unchanged so the schema, provider, and dunning decisions stay traceable; every requirement here (FR-060–FR-070) is v1.1 scope. `sync_locked`, `past_due`, `trialing`, and `canceled` do not exist in v1.

**Plans (USD, prices shown include IVA 15%):**

| Plan | Price | Billing | Entitlement |
|---|---|---|---|
| Free trial | $0 | 3 calendar months from clinic creation; one trial per clinic | Full online sync; server keeps the trial-period data; branding assets + 1 library image |
| Free (post-trial) | $0 | — | No online sync; app otherwise unaffected; server access limited to the original first 3 months; branding + 1 retained library image render; no uploads; 0 after cancellation |
| Monthly | $5.00 / month | First payment via PayPhone button; renewals charged with the stored `cardToken` | Online sync while active; unlimited server retention; 5-image library |
| Yearly | $50.00 / year (2 months free) | First payment via PayPhone button; renewals charged with the stored `cardToken` | Same as monthly, including the 5-image library; billed annually |

- **Billing unit is the clinic (tenant).** One subscription per clinic; the Administrator is the payer, their card becomes the stored payment method, and all members inherit the entitlement. Prices are tax-inclusive and the IVA rate lives in `tax_rates` (15% seeded): at 15%, $5.00 = $4.35 net + $0.65 IVA; $50.00 = $43.48 net + $6.52 IVA. The split is computed at checkout and snapshotted per transaction.
- **Trial:** starts when the clinic is created and ends after 3 calendar months. During the trial all online features work and the clinic can upload one library image (FR-080); branding assets remain available. On expiry without an active plan the clinic enters `locked` (online sync disabled, one image retained for rendering).
- **Payment status never gates clinical work.** Patients, prescriptions, templates, editing forms, A4 PDF/print, local history, QR codes of already-issued prescriptions, and public verification remain available regardless of billing state. Only online sync, new server writes, and library uploads beyond the free image are gated; branding renders until cancellation, and from `past_due` only one library image renders.
- **Web is online-only:** after the trial without a plan, the web app is read-only over retained data. Desktop continues to work fully offline with local data; new prescriptions stay local and their QR activates only after sync.
- **Free-tier server retention:** the first 3 months of the clinic's data (`clinics.server_retention_until`) stay on the server. Data created afterwards survives the 30-day export grace and becomes eligible for trimming (FR-069, only when the job is enabled). Local rows with no server copy are never evicted by the 30-day cache rule (FR-042).
- **Renewal and dunning:** a renewal is a server-initiated tokenized charge (`POST /api/transaction/web`) using the stored `ctoken`. A declined charge gets a 7-day `past_due` grace with sync, retries on days 0, +2, and +4, then `locked`. After a lapse, server data beyond the free window is kept for a 30-day export grace (`data_grace_ends_at`) before trimming; the first 3 months are never deleted and legal sign-off remains a production prerequisite (§9).
- **Plan changes:** monthly ↔ yearly at period end; no proration. The user's cancellation takes effect at period end (`cancel_at_period_end`), and sync continues until then. `canceled` is the terminal state for an immediate/support termination: it behaves like `locked` for sync entitlement and deletes image assets.
- **Provider: PayPhone Tokenization** (`https://docs.payphone.app/tokenizacion`). The first payment goes through the PayPhone payment button/box with a Visa or Mastercard; PayPhone returns `ctoken` (card token) plus cardholder data to the response URL. Renewals are server-initiated charges to `POST https://pay.payphonetodoesposible.com/api/transaction/web` using the stored `ctoken`, the AES-256-CBC-encrypted `cardHolder`, `documentId`, `phoneNumber`, `email`, `amount`, `clientTransactionId`, `currency=USD`, `storeId`, and the required `order` (`billTo` + `lineItems`). PayPhone states that tokenization is **not** an automatic recurring system — the app owns the per-period schedule. Only Visa and Mastercard are eligible; explicit cardholder consent is required before storing a token; tokenization must be authorized by PayPhone, which retains 10% of tokenized balances for 6 months.

- FR-060: Plan catalog is fixed in seed data: `monthly` = 500 cents, `yearly` = 5000 cents, both USD and tax-inclusive; `interval_months` 1 and 12. Plans store only the tax-inclusive amount; the IVA rate comes from the effective-dated `tax_rates` table (15% seeded).
- FR-061: A subscription row is created for every clinic with `trialing` status and `trial_ends_at` = clinic creation + 3 months. Retention boundaries stay on `clinics` (`server_retention_until`, `data_grace_ends_at`). Users cannot reset the trial.
- FR-062: Sync entitlement is enforced server-side in every authorized mutation: `trialing`, `active`, and `past_due` allow sync; `locked` and `canceled` return `402 sync_locked` with the upgrade URL. Reads of retained data remain allowed; auth and public verification are never gated.
- FR-063: Checkout (first charge): the server creates a `payment_transactions` row (UUID v7 `id` used as `clientTransactionId`), computes the tax-inclusive split from the effective `tax_rates` row, and redirects the Administrator to the PayPhone payment button/box with the plan amount, a `PersonaSanus <plan> — <clinic>` reference (never patient data), and an absolute HTTPS `responseUrl`. The Administrator pays with Visa or Mastercard and consents to token storage before leaving the app. PayPhone returns `id`, `clientTransactionID`, and `ctoken` to the response URL; the server confirms the transaction, stores the token (FR-071), and activates the plan. `PAYPHONE_TOKEN`, `PAYPHONE_STORE_ID`, and `PAYPHONE_CODING_PASSWORD` stay server-side.
- FR-064: Resolution: the `responseUrl` returns `id`, `clientTransactionID`, and `ctoken`; the server treats it as a hint and confirms the charge with PayPhone before changing entitlement, and a 60-second polling job covers a missing redirect. Tokenized renewals are confirmed from the `POST /api/transaction/web` response (`statusCode` `2` canceled, `3` approved, plus `status`, `transactionId`, `authorizationCode`). Replays are idempotent keyed by `payment_events.event_hash` and transaction status.
- FR-065: An approved payment activates/extends the subscription: during the trial the paid period starts at `trial_ends_at` (the remaining trial is not wasted); an early renewal starts at `current_period_end`; a lapsed/locked subscription starts now. Canceled/failed attempts are recorded and follow dunning. Duplicate callbacks and replayed polls are no-ops keyed by `payment_events.event_hash` and transaction status.
- FR-066: An erroneous approved charge is reversed through API Reverse the same day (PayPhone limit 23:59); later refunds are manual and audited. Cancelling a subscription revokes the stored `ctoken` (status `revoked`) and starts no further charges. Both paths set `payment_transactions.status='reversed'` and write `audit_log`.
- FR-067: Administrator billing page: current status, trial end, next charge, plan selection, saved card token status, consent record, last transactions, and cancel/reactivate. It must state that only Visa and Mastercard are eligible.
- FR-068: All billing transitions (checkout, approval, failure, lock, cancel, reverse, trim) are written to `audit_log`; raw provider payloads go to `payment_events`, never to logs with PHI.
- FR-069: Retention trimming runs by default in v1 (D-12): after `data_grace_ends_at` it trims server rows created after `server_retention_until`, never touching the first 3 months, with an export offered first. In v1.1 the same job is restricted to `locked`/`canceled` clinics and keys off `clinics.server_retention_until`. It deletes children before parents and never removes a row still referenced by a retained record; no encrypted archive is kept, and counsel sign-off remains a production prerequisite.
- FR-070: Billing failures never destroy local data: the outbox is not evicted, and the desktop shows `SINCRONIZACIÓN BLOQUEADA — renueva para sincronizar` with offline PDF/print still available.
- FR-071: Token storage: only after explicit consent, store the PayPhone `ctoken`, the returned cardholder data (`email`, `phoneNumber`, `document`, name encrypted AES-256-CBC without IV with `PAYPHONE_CODING_PASSWORD`), and `consented_at` in `payment_methods`. Never store PAN, CVV, or full card numbers; at most one active token per clinic; revoke on cancellation.
- FR-072: Renewals: the scheduler charges the stored token with `POST https://pay.payphonetodoesposible.com/api/transaction/web` (Bearer `PAYPHONE_TOKEN`) with `cardToken`, `cardHolder`, `documentId`, `phoneNumber`, `email`, `amount`, `amountWithoutTax`, `tax`, `clientTransactionId`, `currency=USD`, `storeId`, `optionalParameter`, and the required `order` (`billTo` + `lineItems`). `amount = amountWithoutTax + amountWithTax + tax + service + tip`; at 15%, monthly is `amount=500`, `amountWithoutTax=435`, `tax=65`; yearly is `amount=5000`, `amountWithoutTax=4348`, `tax=652`.

### 5.8 Images (1 in v1; state-based in v1.1)

Images in this section are template/branding content, not clinical attachments. Patient photos and signature/stamp images are out of scope; the doctor signs the printed prescription on paper.

- FR-080: **v1 quota is flat: 1 library image per clinic**, at most **10 MB (10,485,760 bytes; 10 MiB)**, stored in private object storage. The clinic logo is a branding asset and does not count (FR-084). Uploads require a network connection; the desktop caches existing images for offline rendering but never queues new image uploads. The upload that exceeds the quota returns `409 image_quota_exceeded`. v1.1 replaces this with the state-based quota (5 `active`/`past_due`, 1 `trialing`/`locked`, 0 `canceled`, `402 image_upload_requires_plan`).
- FR-081: Accepted formats are JPEG and PNG. WebP and SVG are rejected. The server verifies magic bytes, decodes the image, strips EXIF/GPS metadata, enforces the 10 MB cap while streaming, and stores `content_type`, `size_bytes`, `sha256`, `width`, and `height`.
- FR-082: The quota counts non-deleted `assets.kind='image'` rows for the clinic: **1 in v1**, 5 `active`/`past_due`, 1 `trialing`/`locked`, 0 `canceled` in v1.1. The logo (`kind='logo'`) is excluded. Deleting an image frees the slot only when no published template version or retained prescription depends on it.
- FR-083: Objects live in a private MinIO bucket under `clinics/{clinic_id}/images/{asset_id}`. There are no public object URLs; reads use signed URLs valid for 15 minutes. Object keys contain no names, cédulas, or other PHI.
- FR-084: Branding assets are clinic/doctor **logos only** (`kind='logo'`). They use the same storage and validation but do **not** count against the library quota. Signatures and stamps are not stored at all: the doctor signs the printed paper (D-06). Logos render in every state until the clinic is `canceled` (v1.1).
- FR-085: **v1:** an image referenced by a published template version or a retained prescription cannot be deleted until those records are archived or removed; there is no cancellation state. **v1.1:** from `past_due`/`locked`, only the logo and one library image render and the rest show placeholders while online; `canceled` purges **every** object — library images and logos — and renders placeholders while published-version pins stay intact (D-05, an accepted limitation: re-downloaded historical PDFs are not visually identical). Upload and delete write `audit_log` rows (`asset_upload`, `asset_delete`) with metadata only.

## 6. Data model (SQLite)

Tables follow the standard `id`, `created_at`, `updated_at`, `created_by` shape except `clinic_members` and `template_version_assets` (composite primary keys), `rx_counters` (`clinic_id` primary key), `audit_log` (`at`, no `created_at`/`updated_at`), and `payment_events` (no `updated_at`/`created_by`); the auth module owns the identity schema (`users`, `sessions`) and `profiles.auth_user_id` links the app profile to the identity:

| Table | Key fields |
|---|---|
| `profiles` | `display_name`, `status`, `auth_user_id` (role lives only in `clinic_members`) |
| `devices` | `profile_id`, `name`, `platform`, `app_version`, `last_seen_at`, `revoked_at` |
| `doctors` | `profile_id`, `specialty`, `license_no`, `national_id`, `phone`, `email` (no signature/stamp image — D-06) |
| `clinics` | `name`, `address`, `phone`, `logo_asset_id`, `timezone`, `llm_url_template`, `retention_days=30`, `server_retention_until` (creation + 3 months), `data_grace_ends_at` (v1: creation + 30 days; v1.1: set at lock) |
| `assets` | `clinic_id`, `kind(logo, image)`, `object_key`, `content_type`, `size_bytes`, `sha256`, `width`, `height`, `status(pending, ready)`, `deleted_at` |
| `clinic_members` | `clinic_id`, `profile_id`, `role` |
| `patients` | `clinic_id`, `full_name`, `national_id`, `birth_date`, `sex`, `phone`, `notes` |
| `templates` | `owner_type(doctor, clinic)`, `owner_id`, `name`, `current_version_id`, `status` |
| `template_versions` | `template_id`, `version_no`, `doc_json` (form), `variables_spec`, `theme`, `status` |
| `template_version_assets` | `template_version_id`, `asset_id` (pins images to published versions) |
| `prescriptions` | `patient_id`, `doctor_id` (NULL on a nursing draft; `created_by` is the author; required before signing), `clinic_id`, `template_version_id`, `status`, `rx_number` (from the device's block, at signing), `rx_block_id`, `validity`, `diagnosis`, `instructions`, `signed_at`, `voided_at`, `void_reason`, `client_id` (idempotency), `supersedes_id`, `synced_at` (server sets on sync; the local cache uses NULL = not yet uploaded) |
| `prescription_items` | `prescription_id`, `position`, `med_name`, `dose`, `route`, `frequency`, `duration`, `quantity`, `notes` |
| `verification_codes` | `prescription_id`, `code` (printed on the PDF; reprints reuse it), `code_hash`, `status`, `created_at` |
| `rx_counters` | `clinic_id`, `last_rx_number` (highest allocated, not consumed) |
| `rx_blocks` | `clinic_id`, `device_id`, `start_number`, `end_number`, `consumed`, `status(active, exhausted, revoked)`, `allocated_at` |
| `tax_rates` (v1.1) | `name`, `rate_bps`, `effective_from` |
| `plans` (v1.1) | `code(monthly, yearly)`, `amount_cents` (tax-inclusive), `currency`, `interval_months`, `active` |
| `payment_methods` (v1.1) | `clinic_id`, `provider`, `card_token`, `card_holder_enc` (AES-256-CBC), `holder_email`, `holder_phone`, `holder_document`, `status(active, revoked)`, `consented_at` |
| `billing_accounts` (v1.1) | `clinic_id`, `legal_name`, `tax_id`, `email`, `phone_number`, `country_code` |
| `subscriptions` (v1.1) | `clinic_id`, `plan_code`, `status`, `trial_ends_at`, `current_period_start/end`, `next_charge_at`, `grace_ends_at`, `cancel_at_period_end` |
| `payment_transactions` (v1.1) | `clinic_id`, `subscription_id`, `payment_method_id`, `plan_code`, `provider`, `provider_transaction_id`, `authorization_code`, `amount_cents`, `net_cents`, `tax_cents`, `tax_rate_bps`, `status`, `status_code`, `phone_number`, `period_start/end`, `requested_at`, `resolved_at` |
| `payment_events` (v1.1) | `event_hash` (idempotency), `transaction_id`, `payload_json`, `processed_at` |
| `audit_log` | `actor_id`, `action`, `entity`, `entity_id`, `metadata_json`, `at` |

`sessions.device_id` (nullable, auth-owned) links a session to `devices`; revoking a device invalidates its sessions.

v1 creates every table above except the seven marked (v1.1). Retention dates stay on `clinics` so the 3-month server rule exists without a subscription row (D-09).

**Mutations (authorized service functions):** `register_user`, `create_clinic`, `set_member_role`, `remove_member`, `update_profile`, `update_doctor`, `register_device`, `revoke_device`, `create_patient`, `update_patient`, `soft_delete_patient`, `create_template`, `create_template_version`, `publish_template_version`, `archive_template`, `create_prescription(draft)`, `update_prescription`, `upsert_prescription_item`, `sign_prescription`, `void_prescription`, `supersede_prescription`, `create_verification_code`, `request_pdf_regeneration`, `upload_asset`, `delete_asset`, `evict_local_cache_row` (client-only), plus the transactional number-block functions `allocate_rx_block` (server, `BEGIN IMMEDIATE`), `consume_rx_number` (client, offline, within an allocated block), and `reconcile_rx_block` (server, on sync). Billing mutations (`start_trial`, `create_checkout`, `resolve_payment`, `store_payment_method`, `revoke_payment_method`, `charge_subscription`, `schedule_renewal`, `lock_subscription`, `cancel_subscription`, `reverse_payment`, `trim_beyond_free_window`, `can_sync`, `can_upload_images`) are v1.1.

**Reads:** queries scoped by `clinic_id`/`doctor_id`/`patient_id` with role filters; PHI only flows to authorized roles. Sensitive-read audit: log access to patients/prescriptions (who/when/what) — a healthcare traceability requirement. Billing reads (v1.1) expose plan/status/transaction metadata only, never PHI. Image reads expose asset metadata and short-lived signed URLs only; object keys carry no PHI.

## 7. Technical architecture

| Layer | Decision |
|---|---|
| Client | Dioxus 0.7.1, single codebase for web + desktop (features `web` / `desktop`). Implemented routes: `/`, `/login`, `/dashboard` (client-side session check). Target v1 routes: `/patients*`, `/prescriptions*` (incl. `:id/preview`), `/templates*`, `/settings*`, `/audit`, `/verify/:code`. |
| State | Signals/context for session, role, sync status; `use_resource`/fetch over `/api/*` for initial loads. Identical UI on desktop/web. |
| Editor | Form-based editing (plain Dioxus forms shared by web, macOS, and Windows — no WYSIWYG, no rich text, D-21): letterhead/body/footer fields, variable insertion, image slots, medication rows; the form emits schema-checked structured JSON persisted as `doc_json`; raw HTML is never persisted or rendered. |
| PDF | `printpdf` 0.12 renders the PDF in Rust from the pinned `doc_json` layout model: A4 page ops, embedded TTF fonts, JPEG/PNG images, and the QR generated by `qrcode-generator` 6.1.2 and embedded as PNG. One shared renderer module in the server binary (`GET /api/prescriptions/:id/pdf`) and the desktop binary (local, offline). The A4 preview is the browser projection of the same model; no Chromium and no `from_html` (D-01). |
| Backend | Single fullstack process (axum + Dioxus serve) with embedded SQLite via Diesel (`app.db` locally; `/data/app.db` in the `persona-data` volume under Docker). App-owned identity (Argon2 passwords, hashed opaque sessions) mounted same-origin at `/api/auth/*` with `users`/`sessions` tables. Planned ops: Caddy/Nginx with TLS + encrypted daily backups of the SQLite file. |
| Billing (v1.1) | Not implemented in v1. Provider: **PayPhone Tokenization** (`https://docs.payphone.app/tokenizacion`). First charge via the PayPhone payment button/box (Visa/Mastercard) returns `ctoken`; renewals `POST https://pay.payphonetodoesposible.com/api/transaction/web` with Bearer `PAYPHONE_TOKEN`, `storeId`, `cardToken`, AES-256-CBC `cardHolder` (`PAYPHONE_CODING_PASSWORD`), and the required `order`. Amounts in integer cents from `plans`; IVA split from `tax_rates`, snapshotted per transaction. No PAN/CVV; tokenization requires PayPhone authorization and a 10%/6-month balance hold. |
| Jobs | In-process scheduler in the single fullstack node: expired-session purge, stale/pending asset cleanup, and retention trimming after `data_grace_ends_at` (on in v1, D-12). v1.1 adds trial expiry, renewal requests, payment polling, and dunning. Jobs are idempotent and restart-safe (state lives in SQLite). |
| Sync | Local cache (SQLite) + outbox; SSE from the existing axum router with a 30-second polling fallback; web online-first with a minimal queue. No entitlement gate in v1; `can_sync(clinic_id)` and `402 sync_locked` are v1.1. |
| Assets | Private MinIO bucket for template images. v1 quota: 1 image per clinic, 10 MB; the logo uses the same store but does not consume the quota, and no signature/stamp assets exist (D-06). Uploads are server-validated (JPEG/PNG only; WebP/SVG rejected; EXIF stripped), quota-checked, and served via 15-minute signed URLs. v1.1 raises the quota by state (5 `active`/`past_due`, 1 `trialing`/`locked`, 0 `canceled`) and renders placeholders when lapsed. |
| Distribution | Signed, notarized macOS DMG and signed Windows MSI. In-app auto-update with a signed manifest, staged rollout, and rollback to the previous version (D-08). |
| LLM link | `clinics.llm_url_template`, e.g. `https://chat.openai.com/?q={med}` — filled only with `med_name`, explicit user click. |

> Note: SQLite alone does not sync — desktop "offline-first" is built on top (cache + outbox + idempotency). Together with `printpdf` layout fidelity (manual page ops, no CSS engine) and preallocated numbering/reconciliation, these are the main implementation risks (§13).

## 8. Non-functional requirements

- **Ecuador locale/format:** 100% `es-EC` UI; `DD-MM-AAAA` dates in filenames and UI (`es-EC`), 24-hour time; clinical records carry no currency. Billing (v1.1) is USD (ISO 4217) displayed `es-EC`. The 10-digit cédula is the preferred identifier.
- **Performance:** create/edit a prescription < 2 s on the national network; the A4 preview renders in < 3 s, and the `printpdf` render completes in < 1 s locally (desktop, no network) or < 2 s server-side; offline-first desktop is immediate (no network spinner to view recent history). A number-block allocation completes in < 1 s when online.
- **Availability:** SQLite on a Docker host (restart policies, healthchecks); RPO ≤ 24 h (daily encrypted file backup), RTO ≤ 4 h in v1 (no critical medical SLA). v1.1 adds that a PayPhone outage only delays payment/renewal and never blocks clinical work.
- **Accessibility:** AA contrast, keyboard navigation in the editor and PDF viewer, visible focus.
- **Compatibility:** macOS 13+, Windows 10/11 (WebView2), recent Chromium browsers (Chrome/Edge). PDF output requires no browser print path: `printpdf` produces the file on both targets; printing uses the OS viewer on desktop. Standard A4 printers; warn when the selected paper is not A4. Auto-update requires a verified signed manifest and keeps the previous version for rollback.
- **Traceability:** audit log retained for 5 years, append-only; no purge before the window.

## 9. Security, privacy and compliance (Ecuador)

- **Reference framework:** Ley Orgánica de Protección de Datos Personales (2021) and its regulations, MSP/ACESS rules on clinical records. **Required action:** local legal review before production — this PRD is not a substitute for legal advice (e.g. retention periods, consent criteria, legitimacy of public verification).
- Minimization: PHI (patient, diagnosis, medications) never travels to an external LLM, into a QR, or into URLs. URLs use only an opaque `code`; never names/cédulas.
- Encryption: TLS 1.2+ in transit; at-rest encryption on disk/Docker volumes. The desktop cache relies on OS disk encryption (FileVault/BitLocker) as a deployment prerequisite — there is no app-level database encryption in v1 (D-07), so shared or unencrypted machines are unsupported.
- Access: MFA for doctors/administrators (M3); expiring sessions; registered and revocable devices (`devices` + `sessions.device_id`, desktop and web).
- ARCO rights (access/rectification/cancellation/opposition): search, export, and deletion of patients through an audited flow; logical deletion on the server (hard delete only when no retention obligation applies).
- PHI in PDFs: generated PDFs are the doctor's responsibility (shared devices → warning + auto-cleanup on desktop after print). The app stores no PDF bytes on the server; regeneration is from the immutable document.
- **Images:** uploads are template/branding content, not patient photos, and no signature/stamp images exist (D-06). JPEG/PNG only (WebP and SVG rejected); magic-byte validation, decode limits, EXIF/GPS stripping, 10 MB cap; v1 quota is 1 library image per clinic (logo excluded); private MinIO bucket with 15-minute signed URLs and no PHI in object keys. v1.1 adds the state-based quota and the `past_due`/`canceled` placeholder behavior; `canceled` purges every object, so re-downloaded historical PDFs are not visually identical (D-05).
- **Payment data (v1.1):** the app never receives or stores PAN, CVV, or full card numbers; PayPhone handles the instrument. Store the card token (`ctoken`), the cardholder data returned by PayPhone (email, phone, document, AES-encrypted name), transaction ids, status, and amounts. Treat the token as a bearer secret, record explicit consent before storage, and revoke it on cancellation.
- **Payment credentials (v1.1):** `PAYPHONE_TOKEN`, `PAYPHONE_STORE_ID`, and `PAYPHONE_CODING_PASSWORD` (the AES key used for `cardHolder`) live only in server-side env/secret storage, never shipped to the client; rotate on compromise. All PayPhone calls use HTTPS.
- **Callback integrity (v1.1):** the `responseUrl` and tokenized-charge responses are untrusted input; `clientTransactionId` is an opaque UUID and the server confirms every notification with PayPhone before changing entitlement. Replays are idempotent (`payment_events.event_hash`).
- **Server retention vs. health-record law (D-12):** every clinic keeps a 3-month server window (`server_retention_until` = creation + 3 months; v1 sets `data_grace_ends_at` = that instant + 30 days); at month 3 an export is offered, then data created beyond the window is trimmed after the grace. v1.1 sets `data_grace_ends_at` at lock time and clears it while the clinic is active. Enforcement is on in v1 (D-12): the trim job runs by default, so a free-beta clinic loses server records older than ~4 months unless it exports them. In v1.1, trimming applies only to `locked`/`canceled` clinics (FR-069); `active`/`trialing` retain indefinitely. Counsel sign-off on prescription retention remains a production prerequisite, and no encrypted archive is kept in v1 — add one before production if mandatory retention is confirmed.
- **Trial abuse (v1.1):** one trial per clinic, tied to clinic creation; monitor duplicate tax ids/phones and manual resets. This is a commercial control, not a security boundary.
- **Provisional offline documents:** an offline-signed prescription has no server copy, no verification code, and no active QR until sync; its PDF is a local artifact marked `PENDIENTE DE SINCRONIZACIÓN`. The outbox is never discarded, and the desktop warns before signing when the number block is exhausted.
- **Auto-update:** desktop updates ship only from our signed manifest over HTTPS; the signing key is server-side, the client verifies signature and version before applying, and the previous version is retained for rollback. An update never carries clinical data.

## 10. Success metrics

- End-to-end issue time < 60 s (90th percentile).
- % of created prescriptions fully synced within < 5 s after reconnect.
- % of PDFs successfully verified on first attempt (readable QR) > 98%.
- Adoption: ≥ 5 active doctors/week within 3 months; > 80% of active doctors create a personal template.
- ≥ 98% of first PDF renders produce a usable A4 document with a readable QR; ≥ 99.9% of offline-signed prescriptions reconcile without a numbering conflict.
- v1.1 metrics (deferred): trial → paid conversion ≥ 25%; involuntary churn < 5%; recovery within the grace ≥ 60%.

## 11. v1 roadmap

- **M0 (foundation):** auth + roles, patient CRUD, SQLite service layer, Docker Compose deployment.
- **M1 (forms + PDF):** templates + form editor + variables + A4 preview + `printpdf` A4 renderer + filename + QR + logo/one-image upload.
- **M2 (offline):** desktop cache/outbox (OS disk encryption prerequisite), preallocated number blocks, provisional PDFs, SSE + polling sync, 30-day cache retention, on-demand regeneration, sync indicators.
- **M3 (hardening):** audit, MFA, encrypted backups, signed installers + auto-update channel (D-08), QA with real A4 printers. Device registration/revocation and verification rate limiting land earlier (specs 04-D and 09-I).
- **M4 (beta release):** retention/export job (on in v1, D-12), session purge, orphan cleanup, runbook, beta telemetry.

Spec phases (`specs/01`–`13`) are the execution order and may deliver roadmap items earlier than their milestone label.

**v1.1 (billing, deferred):** `plans`/`tax_rates`/`subscriptions`/`payment_methods`/`payment_transactions`, trial, PayPhone Tokenization (payment-button first charge, `ctoken` storage/consent, `/api/transaction/web` renewals, gated on PayPhone authorization), renewal/dunning scheduler, `can_sync` gate, image library (5/1 tiers), `/settings/billing`, state-based rendering rules.

M0 progress (2026-11-11): Diesel-backed sign-up/sign-in/sign-out with `POST /api/auth/*`, the `/login` page and the client-checked `/dashboard`, and the Docker Compose deployment (app container, `persona-data` volume, `/health` healthcheck) are implemented. Roles and patient CRUD remain.

## 12. Acceptance criteria (v1)

- [ ] Doctor creates a patient, picks their template, fills the form, signs, and prints an A4 PDF with the clinic logo and an active QR.
- [ ] The same flow works offline on desktop using a preallocated number; on reconnect it syncs, is reconciled without duplication, and the final numbered PDF with an active QR is regenerated.
- [ ] Exhausting the device's number block offline blocks further signing with a visible warning and remaining count.
- [ ] Numbers are unique and monotonically increasing per clinic across devices; gaps from burned block tails are recorded in `rx_blocks` and never reused.
- [ ] A PDF ≥ 31 days old is not in the local cache; it regenerates on demand from the immutable version with the same content.
- [ ] Scanned QR shows "Receta auténtica" without exposing patient data; a provisional offline PDF carries no scannable QR.
- [ ] Medication shows an LLM link with the drug name only.
- [ ] The print title follows `{Patient}-receta-{dd-mm-aaaa}-{Doctor}.pdf` in `es-EC`.
- [ ] Secretary cannot edit a signed prescription; nursing cannot sign one; everything is audited.
- [ ] A clinic uploads one 10 MB image; a second upload and any file over 10 MB are rejected; the logo does not consume the slot; uploads are refused offline.
- [ ] An image referenced by a published template version or a retained prescription cannot be deleted.
- [ ] Server retention enforces the 3-month window: export is offered at month 3, and trimming runs after the 30-day grace for data created beyond the window.
- [ ] Auto-update applies only a manifest with a valid signature and rolls back on a bad signature or failed launch.

## 13. Risks

| Risk | Impact | Mitigation |
|---|---|---|
| `printpdf` layout fidelity (programmatic page ops, no CSS engine) | preview ≠ issued PDF; page-break, overflow, or font defects reach the patient | one renderer module; golden A4 fixture diffed per PR; fix layout in the renderer, never with a second template; pin the crate version |
| `printpdf` HTML layout is experimental and the API moves | template-driven layout that silently changes after a dependency bump | never call `from_html`; explicit ops only; pin `printpdf` 0.12 and re-run the golden fixture on upgrade |
| PDF delivery UX | wrong or missing filename; a download that looks like a silent background write | generation only on user action; `Content-Disposition` filename (FR-032); *Print* opens the system viewer; measure first-attempt success ≥ 98% |
| Preallocated numbering | gaps from burned blocks; exhaustion blocks signing; replayed duplicates | `BEGIN IMMEDIATE` allocation, server validation on reconcile, unique index, remaining-count UI, configurable block size, unused tails recorded in `rx_blocks` |
| Desktop auth transport | the client posts to absolute `http://127.0.0.1:8080` and `reqwest` has no cookie jar, so desktop sessions do not persist | REST under `/api/*` with a configurable origin + cookie jar + credentialed CORS (chosen — specs 02-B/04-D); blocking for M2 |
| Single-node SQLite concurrency | `SQLITE_BUSY` under the pool plus jobs | WAL, `busy_timeout`, retry policy for write transactions before M2 |
| Free-text medications | spelling/dose errors carry clinical risk | visible disclaimer + mandatory doctor verification; Phase 2 catalog |
| Self-hosted operations | backups, TLS, updates are our responsibility | runbook, encrypted daily file backup, RPO ≤ 24 h, RTO ≤ 4 h |
| 3-month server retention vs. clinical-record law | destroying records that must be kept, or keeping a window that contradicts the 5-year audit rule | export-before-trim; counsel sign-off before production; encrypted archive if mandatory retention is confirmed |
| Image cost/abuse | orphaned objects and decoder attacks | 1 × 10 MB cap in v1, magic-byte + decode limits, quota inside `BEGIN IMMEDIATE`, orphan cleanup, private bucket |
| Image privacy | EXIF/GPS or PHI in files | EXIF stripping, JPEG/PNG only, no PHI in object keys, 15-minute signed URLs |
| Auto-update supply chain | a rogue release could exfiltrate PHI | signed manifest, server-side key custody, staged rollout, rollback, previous version retained |
| Legal validity of PDF/QR | printed reprint may not satisfy MSP/ACESS expectations | doctor signs the printed paper; counsel review (§15 O-01) |
| Deferred billing (v1.1) | PayPhone tokenization requires prior authorization; 10% of tokenized balances is held 6 months; Visa/Mastercard only; token custody and consent; decline handling | apply for authorization before M4; store tokens as secrets with recorded consent; dunning on declines; the §5.7 design is retained |

## 14. Decision log

| ID | Date | Decision | Rationale | Consequence |
|---|---|---|---|---|
| D-01 | 2026-11-11 | PDF is rendered in Rust by `printpdf` 0.12, linked into both the server and desktop binaries; no Chromium, no print dialog, no PDF library on a separate service | one deterministic renderer for web and desktop; the desktop build renders offline with the same code; removes the WebView print-API dependency and the print-CSS fidelity gap | PDF is real file bytes, not a print-dialog artifact; filename via `Content-Disposition`; `GET /api/prescriptions/:id/pdf`; the wasm client never links the renderer; reprints reuse the pinned version and renderer build |
| D-02 | 2026-11-11 | Prescription numbers come from preallocated per-device blocks (default 50) | signing offline with a real number is impossible with server-assigned, gap-free numbering | gaps from burned block tails; exhaustion blocks signing; server reconciles on sync |
| D-03 | 2026-11-11 | One clinic per user; `clinic_members.role` is the only role authority | removes the clinic switcher and the `profiles.role` ambiguity from v1 | `profiles.role` removed; multi-clinic returns only if beta demands it |
| D-04 | 2026-11-11 | Billing is deferred to v1.1; v1 is a free beta | billing must not block launch; the provider also requires authorization | no billing tables, gates, jobs, or UI in v1; §5.7 retained as the v1.1 design |
| D-05 | 2026-11-11 | On cancellation (v1.1) every object is purged and reprints render placeholders | simplest storage lifecycle, explicitly chosen | historical reprints lose logo/images; recorded limitation in FR-085 |
| D-06 | 2026-11-11 | No signature or stamp images; the doctor signs the printed paper | signatures are physical, not digital | `assets.kind` is `logo` or `image`; footer carries a printed signature line |
| D-07 | 2026-11-11 | Desktop cache relies on OS disk encryption (FileVault/BitLocker) | avoids SQLCipher build and key-recovery complexity | shared or unencrypted machines are unsupported; documented prerequisite |
| D-08 | 2026-11-11 | Auto-update ships in v1 | a desktop app without updates strands beta users on fixed bugs | signed manifest, staged rollout, rollback; part of M3 |
| D-09 | 2026-11-11 | Keep the 3-month server window + 30-day export grace | user decision, reconciled with the counsel prerequisite in §9 | retention dates live on `clinics`; enforcement decided in D-12 |
| D-10 | 2026-11-11 | One library image per clinic in v1 | free beta; the 5-image library arrives with billing | logo is excluded; v1.1 restores the state-based quota |
| D-11 | 2026-11-11 | Sync uses SSE from the existing axum router with a 30-second polling fallback | request/response HTTP cannot push and a second socket server is unwanted | one server process; polling covers proxies that break SSE |
| D-12 | 2026-11-11 | Free-beta retention is enforced: 3-month window + 30-day export grace, then trim | user decision (resolves O-09) | the trim job runs by default in v1; export is offered first; counsel sign-off still required before production |
| D-13 | 2026-11-11 | Public verification is opaque-code only; no second factor | user decision (resolves O-02) | FR-050 unchanged; the QR/typed code reveal the minimal record (doctor, date, number) |
| D-14 | 2026-11-11 | Subscription provider is PayPhone Tokenization (`ctoken` + `POST /api/transaction/web`), replacing API Sale's one-off-per-period model | card-on-file removes the per-period approval; PayPhone still requires authorization | requires merchant authorization + 10%/6-month balance hold; Visa/Mastercard only; the app schedules renewals |
| D-15 | 2026-11-11 | Store the verification `code` in plaintext beside `code_hash` | the code is printed and QR-encoded on every issued PDF, so it is public by design; a hash alone makes reprints unable to reproduce the issued QR | `verification_codes.code` is required; `code_hash` remains the lookup key; reprints reuse the stored code |
| D-16 | 2026-11-11 | A supersede creates a new row with its own `rx_number` and verification code | keeps numbering and codes unambiguous per issued document and preserves `uq_prescriptions_rx` | the replaced row keeps its number and becomes `superseded`; its code reports `superseded` |
| D-17 | 2026-11-11 | `prescriptions.doctor_id` may be NULL on a nursing draft | nursing cannot sign, so the doctor is unknown until signing; `created_by` already records the author | a table CHECK requires a doctor for any non-draft row; signing sets `doctor_id` and `signed_at` |
| D-18 | 2026-11-11 | The Administrator has read-only clinic-wide patient/prescription access | matches the sitemap and lets the clinic owner audit clinical activity without granting edit rights | Administrator never signs or edits clinical content; every read is audited |
| D-19 | 2026-11-11 | `clinics` is the only retention authority in v1 and v1.1 | removes the duplicated `subscriptions` deadlines that left the v1 trim job undefined | v1 sets `data_grace_ends_at` at clinic creation; v1.1 sets it at lock and clears it while active; subscriptions keeps billing dates only |
| D-20 | 2026-11-11 | Web clients register a `devices` row (`platform='web'`) and allocate number blocks like desktop | `rx_blocks` requires a device, and web signing must follow the same uniqueness rules | `sessions.device_id` stays nullable until the first block allocation |
| D-21 | 2026-11-12 | No rich-text editor: prescriptions and templates are edited with plain forms on web, macOS, and Windows | the planned WYSIWYG editor (taino-edit) compiles only to wasm32 and cannot run in the native desktop binary; a native fallback would fork the UX the PRD promises to keep identical | taino-edit is removed from the stack; editing is form fields shared by all three targets; the form emits `doc_json`; no rich text in v1 |

## 15. Open questions

| ID | Question | Owner | Blocks |
|---|---|---|---|
| O-01 | Ecuadorian counsel sign-off: prescription retention, legal validity of the PDF/QR, and acceptance of a printed signature applied after printing | Legal | Production (not M0–M2) |
| O-03 | Which desktop path prints a `printpdf`-rendered PDF file: open it in the default viewer or call a native print shell? | Engineering spike | M1 desktop print |
| O-04 | Is a 50-number block the right size for typical consultation volume? | Product | Tune after beta telemetry; config change only |
| O-05 | PayPhone tokenization authorization (application + review) and acceptance of the 10%/6-month balance hold | Business | v1.1 |
| O-07 | Backup retention window, encryption-key custody, and a tested restore drill | Ops + Security | M3 |
| O-08 | Auto-update signing-key custody, rotation, and rollback drill | Security | M3 |

---
*Note: the repository already contains the Dioxus + router + fullstack base with working Diesel-backed authentication (sign-up/sign-in, `/login`, client-checked `/dashboard`) and a Docker Compose deployment — this PRD is implemented on top of it, not as a new project.*
