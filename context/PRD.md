# PersonaSanus — PRD: Clinical Prescription Generator

| Field | Value |
|---|---|
| Product | PersonaSanus — prescription generator for doctors |
| Status | Draft v0.4 — 2026-11-11 (§14 decision log; §15 remaining open questions) |
| Platforms | Desktop (macOS, Windows) offline-first + universal Web (Chromium) — same experience |
| Current stack | Dioxus 0.7.1 (`router`, `fullstack`) + axum 0.8 + Diesel 2.3 (`sqlite`, `r2d2`) + Argon2 0.6 — `persona-sanus/Cargo.toml` |
| Backend | Single-node SQLite + app-owned identity (Argon2 passwords, hashed opaque sessions), self-hosted with Docker Compose on our own infrastructure (`docker-compose.yml`) |
| Market | Ecuador — Spanish (`es-EC`), Ley Orgánica de Protección de Datos Personales + applicable health regulations (validate with legal counsel) |
| v1 scope | Multi-user clinic: doctors + nursing/secretary staff, shared patients and templates; prescriptions only |
| Billing | **Deferred to v1.1.** v1 is a free beta with no payment provider and no entitlement gate; the PayPhone design is retained in §5.7 for v1.1 |
| Storage | Image library: 1 image per clinic in v1 (free beta), 10 MB (10,485,760 bytes) each; the clinic logo is a separate branding asset and does not consume the slot. v1.1 raises the library to 5 with billing |
| Output | A4 PDF via client-side print-to-PDF (Chromium/WebView print dialog → Save as PDF); QR verification. No server-side PDF renderer |
| Editor | Rich-text editor over an A4 template, variables, per-doctor/per-clinic branding |

## 1. Problem and solution

Small/medium clinics in Ecuador produce prescriptions on paper, in Word, or from loose templates: inconsistent format, no synced history between consultation and front desk, no way to verify authenticity, and no simple way to keep a verified history available across devices.

**Solution:** a desktop+web application where each doctor works with their own templates (logo, letterhead, layout) and dynamic variables, edits the prescription in a rich-text editor with a true A4 preview, issues a PDF ready to print (the doctor signs the printed paper) with a verifiable QR code, and keeps everything in a single SQLite backend — with the desktop continuing to work offline (local cache + outbox) and syncing on reconnect. Server sync is free in v1; the $5/month or $50/year subscription (3-month trial) is designed for v1.1 and never blocks clinical work. v1 includes a private image library of one 10 MB image per clinic for template branding; v1.1 raises it to five.

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
| **Administrator** | Users, roles, clinic settings, master templates, image library, access revocation, audit. |
| **Doctor** | Create/edit own prescriptions and patients, create and version own templates, upload images for own templates, print/save PDFs, view own history. |
| **Nursing** | Create prescription drafts, edit vitals/medication fields, cannot sign or publish. |
| **Secretary** | Manage patients, reprint/save issued prescriptions, scheduling, cannot edit signed clinical content. |
| **Public verifier** | No account: scans the QR → `#/verify/:code` page confirming authenticity plus minimal data (doctor, date, prescription number). |

Permissions are enforced server-side in the authorized service layer (never client-side only). Each user belongs to exactly one clinic in v1; `clinic_members.role` is the only role authority (D-03). Signed/published prescriptions are immutable: any correction creates a new version that references the previous one. Administrator and Doctor can upload library images (Administrator clinic-wide; Doctor for own templates); Nursing and Secretary can only view images referenced by templates they can open. Image uploads follow the flat v1 quota (FR-080, one image per clinic). Signatures are never stored as images: the doctor signs the printed prescription on paper (D-06).

## 4. Main flows

1. **Issue prescription (online):** Login → search/create patient → choose template → rich-text editor (variables pre-loaded) → complete medications → A4 preview → *Sign and issue* → the number is assigned from the device's preallocated block and a verification code is created → PDF from the print dialog → sign the printed paper.
2. **Issue prescription (offline, desktop):** same flow against the local cache; signing consumes the next number in the device's preallocated block (FR-014) and the prescription queues in an outbox with `pending_sync` status; the PDF is provisional (`PENDIENTE DE SINCRONIZACIÓN`, inactive QR). On reconnect, server functions upload it with an idempotent `client_id`, reconcile the block, and the final numbered PDF with an active QR is regenerated. If the block is exhausted, signing is blocked until the device reconnects. Conflicts: server wins for signed data; only an in-progress draft warns of a conflict and requires a choice.
3. **Personal template:** Doctor uploads a logo → defines letterhead/footer (rich-text) → saves template → insertable variables (`{{patient.name}}`, `{{doctor.name}}`, `{{date}}`, `{{prescription.number}}`) → A4 preview → publish (new immutable version). No signature or stamp images (D-06).
4. **Verification:** patient/printer scans the QR → web opens the verification page → sees "Receta auténtica — issued by Dr. …, date …, No. …" plus the hash. No PHI.
5. **Re-issue / download history:** search history → *Print / Save as PDF* (regenerated from the final versioned document, same content as issued; not byte-identical because the client renders it). Only ≤ 30 days of server-backed rows live in the desktop local cache; the rest downloads on demand (current product decision). Unsynced rows are never evicted.
6. **Explain medication:** next to each medication line, an *"Explain with LLM"* link opens an LLM page in a new tab with the medication name pre-filled (medication only, never patient data) + disclaimer.
7. **Subscribe / renew (v1.1, not in v1):** Administrator opens *Settings → Subscription* → sees trial/status → picks $5 monthly or $50 yearly → enters the payer's PayPhone number → server sends the collection request → payer approves in the PayPhone app (5-minute window) → webhook + status check activate the plan → sync entitlement updates. The full design is retained in §5.7 and ships in v1.1.
8. **Upload images:** Administrator or Doctor opens *Clinic settings* or a template → uploads one library image (10 MB; logo is separate) → server validates the format, strips metadata, checks the quota, and stores the object in the private MinIO bucket → inserts the image into a template → publishing pins it to that immutable version. Upload is online-only. The five-image, state-based behavior belongs to v1.1 (§5.8).

## 5. Functional requirements

### 5.1 Patients
- FR-001: Patient CRUD (names, cédula, age/birth date, sex, phone, notes). Search by name and cédula.
- FR-002: Deduplication by cédula; warn on likely duplicate creation.
- FR-003: A patient's prescription history is visible only to authorized roles in the clinic.

### 5.2 Prescription
- FR-010: Create a prescription from patient + template; fields: date/time, diagnosis (text), general instructions (rich-text), medications (draggable/reorderable), validity/recommendations.
- FR-011: Each medication: name (free text), dose, route, frequency, duration, quantity, notes (all free text, no validation against an official database).
- FR-012: Per-medication "Explain with LLM" link (configurable via `LLM_URL_TEMPLATE`; default: generic chat). Opens with the drug name only; never with PHI. The app never sends data to any LLM by itself.
- FR-013: States: `draft → signed(published) → superseded/voided`. Signing locks editing and is allowed offline against a preallocated number (FR-014); an offline-signed document is provisional (no server copy, inactive QR) until sync reconciles it. Voiding requires a reason and leaves an audit trail.
- FR-014: Prescription number per clinic, unique and monotonically increasing across the network, assigned from preallocated per-device blocks. A device reserves a block (default 50 numbers; configurable) in a `BEGIN IMMEDIATE` transaction that advances `rx_counters.last_rx_number`, and signs offline by consuming the next number in its active block. Numbers are **not contiguous**: the unused tail of a block is burned and recorded in `rx_blocks`, never reused, so gaps are expected and legitimate (D-02). On sync the server validates that the number belongs to a block issued to that device and is unused, rejects duplicates via the unique index, and records the consumption. A device with an exhausted block cannot sign until it reconnects and refills; the UI shows the remaining count and warns below 10.

### 5.3 Templates and rich-text editor
- FR-020: Per-doctor templates (and clinic master templates): letterhead (logo, doctor/clinic details, colors) + body + footer (printed doctor name/license, a blank space where the doctor signs on paper, QR, contact). Images uploaded to asset storage referenced by `asset_id`. Clinics can insert the library image(s) up to their v1 quota (FR-080); publishing pins every referenced asset to the version. No signature or stamp images are stored (D-06).
- FR-021: WYSIWYG rich-text editor: bold, italic, underline, lists, tables, alignment, headings, variable insertion, image blocks, and medication blocks. Documents stored as versioned structured JSON (never unsanitized raw HTML).
- FR-022: True-scale A4 preview: 210×297 mm, configurable margins, page breaks, repeated header/footer, embedded typography.
- FR-023: Supported v1 variables: `patient.{name,national_id,age}`, `doctor.{name,specialty,national_id,contact}`, `clinic.{name,address,phone}`, `prescription.{number,date,diagnosis}`, `medications[]`. Unknown variables → pre-signing warning, never a literal `{{key}}` in the PDF.
- FR-024: Publishing a template creates an immutable version; already-issued prescriptions keep the version they used.

### 5.4 A4 PDF, download, print, QR
- FR-030: Single pipeline: the A4 print view (editor-identical HTML/CSS, `@page { size: A4; margin: 0 }`, unscaled, background colors, embedded fonts) is printed to PDF by the client — Chromium/WebView print dialog with *Save as PDF*. No server-side Chromium, no server-side PDF library: PDF generation must work offline on desktop (D-01). The default filename is set via `document.title` because browsers ignore `Content-Disposition` on print-to-PDF.
- FR-031: Output: *Print* and *Save as PDF* through the system dialog, plus *Share* on web. There is no silent background download; the UI must say "Imprimir / Guardar como PDF".
- FR-032: **File name:** `{PatientName}-{receta}-{localized_date}-{DoctorName}.pdf`. Example: `Juan Perez-receta-10-11-2026-Dr Ana Ruiz.pdf`. Sanitize `/ \ : * ? " < > |`, collapse whitespace, cap at 120 characters without truncating key fields; applied to the print view's `document.title`.
- FR-033: Every **final** PDF carries a QR encoding `https://<domain>/verify/{code}` + an opaque per-version `code`; the QR contains no PHI (name, cédula, diagnosis, or medications). A provisional offline PDF shows a placeholder QR area plus `PENDIENTE DE SINCRONIZACIÓN` and no scannable code; the active QR appears only after sync assigns the number and verification code (FR-014, FR-043).
- FR-034: PDF header/footer: prescription number, QR, verification code printed as readable text (for re-typing), issue date/time, and a printed signature line for the doctor to sign on paper.

### 5.5 Sync, offline and local retention (desktop)
- FR-040: Offline-first desktop: patients, published templates, and prescriptions (drafts + signed) are accessible offline after first login. Outbound operation queue with retries and visible status. The desktop cache relies on OS disk encryption (FileVault/BitLocker) as a deployment prerequisite; no app-level database encryption in v1 (D-07).
- FR-041: Real-time sync via SSE served by the existing axum router, with a 30-second polling fallback when SSE is unavailable (no second socket server). On reconnect, drain the outbox oldest-first with idempotent server functions (`client_id`), reconcile preallocated `rx_blocks` (FR-014), and refresh the cached number block and metadata.
- FR-042: **Local retention:** only server-backed prescriptions from the last **30 days** are kept locally (configurable calendar month). On expiry they are evicted from the local cache (not deleted from the server); on-demand regeneration with a *Print / Save as PDF* button. Rows with **no server copy** (created offline, still in the outbox, or never synced) are never evicted — they are the only copy.
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
| Monthly | $5.00 / month | Recurring PayPhone collection request each period | Online sync while active; unlimited server retention; 5-image library |
| Yearly | $50.00 / year (2 months free) | Recurring PayPhone collection request each period | Same as monthly, including the 5-image library; billed annually |

- **Billing unit is the clinic (tenant).** One subscription per clinic; the Administrator is the payer and all members inherit the entitlement. Prices are tax-inclusive and the IVA rate lives in `tax_rates` (15% seeded): at 15%, $5.00 = $4.35 net + $0.65 IVA; $50.00 = $43.48 net + $6.52 IVA. The split is computed at checkout and snapshotted per transaction.
- **Trial:** starts when the clinic is created and ends after 3 calendar months. During the trial all online features work and the clinic can upload one library image (FR-080); branding assets remain available. On expiry without an active plan the clinic enters `locked` (online sync disabled, one image retained for rendering).
- **Payment status never gates clinical work.** Patients, prescriptions, templates, rich-text editor, A4 PDF/print, local history, QR codes of already-issued prescriptions, and public verification remain available regardless of billing state. Only online sync, new server writes, and library uploads beyond the free image are gated; branding renders until cancellation, and from `past_due` only one library image renders.
- **Web is online-only:** after the trial without a plan, the web app is read-only over retained data. Desktop continues to work fully offline with local data; new prescriptions stay local and their QR activates only after sync.
- **Free-tier server retention:** the first 3 months of the clinic's data (`free_retention_until`) stay on the server. Data created afterwards survives the 30-day export grace and becomes eligible for trimming (FR-069, only when the job is enabled). Local rows with no server copy are never evicted by the 30-day cache rule (FR-042).
- **Renewal and dunning:** a renewal is a new PayPhone collection request to the payer's phone. Failures get a 7-day grace (`past_due`) with sync, retries on days 0, +2, and +4, then `locked`. After a lapse, server data beyond the free window is kept for a 30-day export grace (`data_grace_ends_at`) before trimming; the first 3 months are never deleted and legal sign-off remains a production prerequisite (§9).
- **Plan changes:** monthly ↔ yearly at period end; no proration. The user's cancellation takes effect at period end (`cancel_at_period_end`), and sync continues until then. `canceled` is the terminal state for an immediate/support termination: it behaves like `locked` for sync entitlement and deletes image assets.
- **PayPhone API Sale is one-off, not native recurring.** The documented API sends a payment request to a phone number registered in PayPhone Personal; the payer approves it in the PayPhone app within 5 minutes. "Recurring" is app-scheduled: the server creates a collection request each period. The billing UI must say so explicitly. v1.1 implementation is gated on PayPhone confirming this in writing.

- FR-060: Plan catalog is fixed in seed data: `monthly` = 500 cents, `yearly` = 5000 cents, both USD and tax-inclusive; `interval_months` 1 and 12. Plans store only the tax-inclusive amount; the IVA rate comes from the effective-dated `tax_rates` table (15% seeded).
- FR-061: A subscription row is created for every clinic with `trialing` status, `trial_ends_at` = clinic creation + 3 months and `free_retention_until` = the same instant. Users cannot reset the trial.
- FR-062: Sync entitlement is enforced server-side in every authorized mutation: `trialing`, `active`, and `past_due` allow sync; `locked` and `canceled` return `402 sync_locked` with the upgrade URL. Reads of retained data remain allowed; auth and public verification are never gated.
- FR-063: Checkout: the server creates a `payment_transactions` row (UUID v7 `id` used as PayPhone `clientTransactionId`) and POSTs to `https://pay.payphonetodoesposible.com/api/Sale` with `phoneNumber`, `countryCode=593`, `amount`, `amountWithTax`, `tax`, `currency=USD`, `reference` (`PersonaSanus <plan> — <clinic>`, never patient data), `storeId`, and an absolute HTTPS `responseUrl`. At the 15% rate, monthly is `amount=500`, `amountWithTax=435`, `tax=65`; yearly is `amount=5000`, `amountWithTax=4348`, `tax=652`. The server selects the current `tax_rates` row, computes the split at checkout, and snapshots `tax_rate_bps`, `net_cents`, and `tax_cents` on the transaction. Token and `storeId` stay server-side.
- FR-064: Resolution: PayPhone callback `/api/billing/payphone/callback` (query `id`, `clientTransactionID`) plus a 60-second polling job while pending. The callback is never trusted alone: the returned `id` must match the stored `provider_transaction_id` for that `clientTransactionId`, then the server confirms with `GET /api/Sale/{transactionId}` (≤ 30 calls/minute), stores `statusCode` (`1` pending, `2` canceled, `3` approved), and applies transitions idempotently.
- FR-065: An approved payment activates/extends the subscription: during the trial the paid period starts at `trial_ends_at` (the remaining trial is not wasted); an early renewal starts at `current_period_end`; a lapsed/locked subscription starts now. Canceled/failed attempts are recorded and follow dunning. Duplicate callbacks and replayed polls are no-ops keyed by `payment_events.event_hash` and transaction status.
- FR-066: Cancellation within 5 minutes of an unpaid request uses API Cancel; an erroneous approved charge is reversed automatically through API Reverse on the same day (PayPhone limit 23:59). Refunds outside that window are manual by the Administrator and audited; both paths set `payment_transactions.status='reversed'` and write `audit_log`.
- FR-067: Administrator billing page: current status, trial end, next charge, plan selection, payer phone, last transactions, cancel/reactivate, and explicit notice that each renewal needs approval in the PayPhone app.
- FR-068: All billing transitions (checkout, approval, failure, lock, cancel, reverse, trim) are written to `audit_log`; raw provider payloads go to `payment_events`, never to logs with PHI.
- FR-069: The retention job exists but runs only when `RETENTION_TRIM_ENABLED` is on (off until counsel, D-09): it trims server rows created after `free_retention_until` for `locked`/`canceled` clinics only after `data_grace_ends_at`, never touching the first 3 months. It deletes children before parents and never removes a row still referenced by a retained record. Export is offered before trimming; no encrypted archive is kept, and counsel sign-off remains a production prerequisite.
- FR-070: Billing failures never destroy local data: the outbox is not evicted, and the desktop shows `SINCRONIZACIÓN BLOQUEADA — renueva para sincronizar` with offline PDF/print still available.

### 5.8 Images (1 in v1; state-based in v1.1)

Images in this section are template/branding content, not clinical attachments. Patient photos and signature/stamp images are out of scope; the doctor signs the printed prescription on paper.

- FR-080: **v1 quota is flat: 1 library image per clinic**, at most **10 MB (10,485,760 bytes; 10 MiB)**, stored in private object storage. The clinic logo is a branding asset and does not count (FR-084). Uploads require a network connection; the desktop caches existing images for offline rendering but never queues new image uploads. The upload that exceeds the quota returns `409 image_quota_exceeded`. v1.1 replaces this with the state-based quota (5 `active`/`past_due`, 1 `trialing`/`locked`, 0 `canceled`, `402 image_upload_requires_plan`).
- FR-081: Accepted formats are JPEG and PNG. WebP and SVG are rejected. The server verifies magic bytes, decodes the image, strips EXIF/GPS metadata, enforces the 10 MB cap while streaming, and stores `content_type`, `size_bytes`, `sha256`, `width`, and `height`.
- FR-082: The quota counts non-deleted `assets.kind='image'` rows for the clinic: **1 in v1**, 5 `active`/`past_due`, 1 `trialing`/`locked`, 0 `canceled` in v1.1. The logo (`kind='logo'`) is excluded. Deleting an image frees the slot only when no published template version or retained prescription depends on it.
- FR-083: Objects live in a private MinIO bucket under `clinics/{clinic_id}/images/{asset_id}`. There are no public object URLs; reads use signed URLs valid for 15 minutes. Object keys contain no names, cédulas, or other PHI.
- FR-084: Branding assets are clinic/doctor **logos only** (`kind='logo'`). They use the same storage and validation but do **not** count against the library quota. Signatures and stamps are not stored at all: the doctor signs the printed paper (D-06). Logos render in every state until the clinic is `canceled` (v1.1).
- FR-085: **v1:** an image referenced by a published template version or a retained prescription cannot be deleted until those records are archived or removed; there is no cancellation state. **v1.1:** from `past_due`/`locked`, only the logo and one library image render and the rest show placeholders while online; `canceled` purges **every** object — library images and logos — and renders placeholders while published-version pins stay intact (D-05, an accepted limitation: re-downloaded historical PDFs are not visually identical). Upload and delete write `audit_log` rows (`asset_upload`, `asset_delete`) with metadata only.

## 6. Data model (SQLite)

Tables (all with standard `id`, `created_at`, `updated_at`, `created_by`); the auth module owns the identity schema (`users`, `sessions`) and `profiles.auth_user_id` links the app profile to the identity:

| Table | Key fields |
|---|---|
| `profiles` | `display_name`, `status`, `auth_user_id` (role lives only in `clinic_members`) |
| `devices` | `profile_id`, `name`, `platform`, `app_version`, `last_seen_at`, `revoked_at` |
| `doctors` | `user_id`, `specialty`, `license_no`, `phone`, `email` (no signature/stamp image — D-06) |
| `clinics` | `name`, `address`, `phone`, `logo_asset_id`, `timezone`, `llm_url_template`, `retention_days=30`, `server_retention_until` (creation + 3 months), `data_grace_ends_at` |
| `assets` | `clinic_id`, `kind(logo, image)`, `object_key`, `content_type`, `size_bytes`, `sha256`, `width`, `height`, `status(pending, ready)`, `deleted_at` |
| `clinic_members` | `clinic_id`, `user_id`, `role` |
| `patients` | `clinic_id`, `full_name`, `national_id`, `birth_date`, `sex`, `phone`, `notes` |
| `templates` | `owner_type(doctor, clinic)`, `owner_id`, `name`, `current_version_id`, `status` |
| `template_versions` | `template_id`, `version_no`, `doc_json` (editor), `variables_spec`, `theme`, `status` |
| `template_version_assets` | `template_version_id`, `asset_id` (pins images to published versions) |
| `prescriptions` | `patient_id`, `doctor_id`, `clinic_id`, `template_version_id`, `status`, `rx_number` (from the device's block, at signing), `rx_block_id`, `diagnosis`, `instructions_json`, `signed_at`, `voided_at`, `void_reason`, `client_id` (idempotency), `supersedes_id`, `synced_at` (null = provisional) |
| `prescription_items` | `prescription_id`, `position`, `med_name`, `dose`, `route`, `frequency`, `duration`, `quantity`, `notes` |
| `verification_codes` | `prescription_id`, `code_hash`, `status`, `created_at` |
| `rx_counters` | `clinic_id`, `last_rx_number` (highest allocated, not consumed) |
| `rx_blocks` | `clinic_id`, `device_id`, `start_number`, `end_number`, `consumed`, `status(active, exhausted, revoked)`, `allocated_at` |
| `tax_rates` (v1.1) | `name`, `rate_bps`, `effective_from` |
| `plans` (v1.1) | `code(monthly, yearly)`, `amount_cents` (tax-inclusive), `currency`, `interval_months`, `active` |
| `billing_accounts` (v1.1) | `clinic_id`, `legal_name`, `tax_id`, `email`, `phone_number`, `country_code` |
| `subscriptions` (v1.1) | `clinic_id`, `plan_code`, `status`, `trial_ends_at`, `free_retention_until`, `current_period_start/end`, `next_charge_at`, `grace_ends_at`, `data_grace_ends_at`, `cancel_at_period_end` |
| `payment_transactions` (v1.1) | `clinic_id`, `subscription_id`, `plan_code`, `provider`, `provider_transaction_id`, `amount_cents`, `net_cents`, `tax_cents`, `tax_rate_bps`, `status`, `status_code`, `phone_number`, `period_start/end`, `requested_at`, `resolved_at` |
| `payment_events` (v1.1) | `event_hash` (idempotency), `transaction_id`, `payload_json`, `processed_at` |
| `audit_log` | `actor_id`, `action`, `entity`, `entity_id`, `metadata_json`, `at` |

`sessions.device_id` (nullable, auth-owned) links a session to `devices`; revoking a device invalidates its sessions.

v1 creates every table above except the six marked (v1.1). Retention dates stay on `clinics` so the 3-month server rule exists without a subscription row (D-09).

**Mutations (authorized service functions):** `register_user`, `register_device`, `revoke_device`, `create_patient`, `update_patient`, `create_template`, `publish_template_version`, `create_prescription(draft)`, `upsert_prescription_item`, `sign_prescription`, `void_prescription`, `create_verification_code`, `request_pdf_regeneration`, `upload_asset`, `delete_asset`, `evict_local_cache_row` (client-only), plus the transactional number-block functions `allocate_rx_block` (server, `BEGIN IMMEDIATE`), `consume_rx_number` (client, offline, within an allocated block), and `reconcile_rx_block` (server, on sync). Billing mutations (`start_trial`, `create_checkout`, `resolve_payment`, `schedule_renewal`, `lock_subscription`, `cancel_subscription`, `reverse_payment`, `trim_beyond_free_window`, `can_sync`, `can_upload_images`) are v1.1.

**Reads:** queries scoped by `clinic_id`/`doctor_id`/`patient_id` with role filters; PHI only flows to authorized roles. Sensitive-read audit: log access to patients/prescriptions (who/when/what) — a healthcare traceability requirement. Billing reads (v1.1) expose plan/status/transaction metadata only, never PHI. Image reads expose asset metadata and short-lived signed URLs only; object keys carry no PHI.

## 7. Technical architecture

| Layer | Decision |
|---|---|
| Client | Dioxus 0.7.1, single codebase for web + desktop (features `web` / `desktop`). Implemented routes: `/`, `/login`, `/dashboard` (session-guarded). Target v1 routes: `/patients*`, `/prescriptions*` (incl. `:id/preview`), `/templates*`, `/settings*`, `/audit`, `/verify/:code`. |
| State | Signals/context for session, role, sync status; `use_resource`/`use_server_future` for initial loads. Identical UI on desktop/web. |
| Editor | Content-editable component (embedded Tiptap/ProseMirror via JS interop) emitting JSON; imported HTML is sanitized; persisted as `doc_json`. |
| PDF | Client-side print-to-PDF: the 1:1 A4 print view is the single HTML/CSS source for editor and output; `window.print()` / WebView print dialog → *Save as PDF*. No server-side Chromium and no PDF library, so PDFs work offline on desktop (D-01). |
| Backend | Single fullstack process (axum + Dioxus serve) with embedded SQLite via Diesel (`app.db` locally; `/data/app.db` in the `persona-data` volume under Docker). App-owned identity (Argon2 passwords, hashed opaque sessions) mounted same-origin at `/api/auth/*` with `users`/`sessions` tables. Planned ops: Caddy/Nginx with TLS + encrypted daily backups of the SQLite file. |
| Billing (v1.1) | Not implemented in v1. PayPhone API Sale: `POST https://pay.payphonetodoesposible.com/api/Sale` with Bearer token + `storeId` from server env (`PAYPHONE_TOKEN`, `PAYPHONE_STORE_ID`); status via `GET /api/Sale/{transactionId}` (≤ 30 calls/min); callback `/api/billing/payphone/callback` is treated as a hint and always re-verified. Requires a Payphone Business account and developer app credentials. Amounts in integer cents from `plans`, with the IVA split computed from `tax_rates` and snapshotted per transaction. No card data touches the app. Gated on written PayPhone confirmation. |
| Jobs | In-process scheduler in the single fullstack node: expired-session purge, stale/pending asset cleanup, and retention trimming after `data_grace_ends_at` only when `RETENTION_TRIM_ENABLED` is on (off until legal sign-off, D-09). v1.1 adds trial expiry, renewal requests, payment polling, and dunning. Jobs are idempotent and restart-safe (state lives in SQLite). |
| Sync | Local cache (SQLite) + outbox; SSE from the existing axum router with a 30-second polling fallback; web online-first with a minimal queue. No entitlement gate in v1; `can_sync(clinic_id)` and `402 sync_locked` are v1.1. |
| Assets | Private MinIO bucket for template images. v1 quota: 1 image per clinic, 10 MB; the logo uses the same store but does not consume the quota, and no signature/stamp assets exist (D-06). Uploads are server-validated (JPEG/PNG only; WebP/SVG rejected; EXIF stripped), quota-checked, and served via 15-minute signed URLs. v1.1 raises the quota by state (5 `active`/`past_due`, 1 `trialing`/`locked`, 0 `canceled`) and renders placeholders when lapsed. |
| Distribution | Signed, notarized macOS DMG and signed Windows MSI. In-app auto-update with a signed manifest, staged rollout, and rollback to the previous version (D-08). |
| LLM link | `clinics.llm_url_template`, e.g. `https://chat.openai.com/?q={med}` — filled only with `med_name`, explicit user click. |

> Note: SQLite alone does not sync — desktop "offline-first" is built on top (cache + outbox + idempotency). Together with client-side print fidelity (WebView print APIs) and preallocated numbering/reconciliation, these are the main implementation risks (§13).

## 8. Non-functional requirements

- **Ecuador locale/format:** 100% `es-EC` UI; `DD-MM-AAAA` dates in filenames and UI (`es-EC`), 24-hour time; clinical records carry no currency. Billing (v1.1) is USD (ISO 4217) displayed `es-EC`. The 10-digit cédula is the preferred identifier.
- **Performance:** create/edit a prescription < 2 s on the national network; the A4 print view renders in < 3 s and opens the print dialog with no server round-trip; offline-first desktop is immediate (no network spinner to view recent history). A number-block allocation completes in < 1 s when online.
- **Availability:** SQLite on a Docker host (restart policies, healthchecks); RPO ≤ 24 h (daily encrypted file backup), RTO ≤ 4 h in v1 (no critical medical SLA). v1.1 adds that a PayPhone outage only delays payment/renewal and never blocks clinical work.
- **Accessibility:** AA contrast, keyboard navigation in the editor and PDF viewer, visible focus.
- **Compatibility:** macOS 13+, Windows 10/11 (WebView2), recent Chromium browsers (Chrome/Edge). Print output requires the platform print-to-PDF path (WKWebView/WebView2/Chromium); standard A4 printers; warn when the selected paper is not A4. Auto-update requires a verified signed manifest and keeps the previous version for rollback.
- **Traceability:** audit log retained for 5 years, append-only; no purge before the window.

## 9. Security, privacy and compliance (Ecuador)

- **Reference framework:** Ley Orgánica de Protección de Datos Personales (2021) and its regulations, MSP/ACESS rules on clinical records. **Required action:** local legal review before production — this PRD is not a substitute for legal advice (e.g. retention periods, consent criteria, legitimacy of public verification).
- Minimization: PHI (patient, diagnosis, medications) never travels to an external LLM, into a QR, or into URLs. URLs use only an opaque `code`; never names/cédulas.
- Encryption: TLS 1.2+ in transit; at-rest encryption on disk/Docker volumes. The desktop cache relies on OS disk encryption (FileVault/BitLocker) as a deployment prerequisite — there is no app-level database encryption in v1 (D-07), so shared or unencrypted machines are unsupported.
- Access: MFA for doctors/administrators (M3); expiring sessions; registered and revocable desktop devices (`devices` + `sessions.device_id`).
- ARCO rights (access/rectification/cancellation/opposition): search, export, and deletion of patients through an audited flow; logical deletion on the server (hard delete only when no retention obligation applies).
- PHI in PDFs: generated PDFs are the doctor's responsibility (shared devices → warning + auto-cleanup on desktop after print). The app stores no PDF bytes on the server; regeneration is from the immutable document.
- **Images:** uploads are template/branding content, not patient photos, and no signature/stamp images exist (D-06). JPEG/PNG only (WebP and SVG rejected); magic-byte validation, decode limits, EXIF/GPS stripping, 10 MB cap; v1 quota is 1 library image per clinic (logo excluded); private MinIO bucket with 15-minute signed URLs and no PHI in object keys. v1.1 adds the state-based quota and the `past_due`/`canceled` placeholder behavior; `canceled` purges every object, so re-downloaded historical PDFs are not visually identical (D-05).
- **Payment data (v1.1):** the app never receives or stores PAN, CVV, or full card numbers; PayPhone handles the instrument. Store only transaction ids, status, amount, phone number, and the provider payload needed for reconciliation.
- **Payment credentials (v1.1):** `PAYPHONE_TOKEN` and `PAYPHONE_STORE_ID` live only in server-side env/secret storage, never shipped to the client; rotate on compromise. All PayPhone calls use HTTPS.
- **Callback integrity (v1.1):** `/api/billing/payphone/callback` is unauthenticated by design; `clientTransactionId` is an opaque UUID, and the server confirms every notification with an authenticated `GET /api/Sale/{transactionId}` before changing entitlement. Replays are idempotent (`payment_events.event_hash`).
- **Server retention vs. health-record law (D-09):** the policy window is 3 calendar months per clinic (`server_retention_until` = creation + 3 months), after which data is exportable during a 30-day grace (`data_grace_ends_at`) and then eligible for trimming. Enforcement is off by default (`RETENTION_TRIM_ENABLED=false`) because counsel sign-off on prescription retention remains a production prerequisite, and because in the free beta every clinic crosses that window (see O-09). No encrypted archive is kept in v1; add one before enabling the job if mandatory retention is confirmed.
- **Trial abuse (v1.1):** one trial per clinic, tied to clinic creation; monitor duplicate tax ids/phones and manual resets. This is a commercial control, not a security boundary.
- **Provisional offline documents:** an offline-signed prescription has no server copy, no verification code, and no active QR until sync; its PDF is a local artifact marked `PENDIENTE DE SINCRONIZACIÓN`. The outbox is never discarded, and the desktop warns before signing when the number block is exhausted.
- **Auto-update:** desktop updates ship only from our signed manifest over HTTPS; the signing key is server-side, the client verifies signature and version before applying, and the previous version is retained for rollback. An update never carries clinical data.

## 10. Success metrics

- End-to-end issue time < 60 s (90th percentile).
- % of created prescriptions fully synced within < 5 s after reconnect.
- % of PDFs successfully verified on first attempt (readable QR) > 98%.
- Adoption: ≥ 5 active doctors/week within 3 months; > 80% of active doctors create a personal template.
- ≥ 98% of first print attempts produce a usable A4 PDF with a readable QR; ≥ 99.9% of offline-signed prescriptions reconcile without a numbering conflict.
- v1.1 metrics (deferred): trial → paid conversion ≥ 25%; involuntary churn < 5%; recovery within the grace ≥ 60%.

## 11. v1 roadmap

- **M0 (foundation):** auth + roles, patient CRUD, SQLite service layer, Docker Compose deployment.
- **M1 (editor + PDF):** templates + JSON editor + variables + A4 preview + client print-to-PDF + filename + QR + logo/one-image upload.
- **M2 (offline):** desktop cache/outbox (OS disk encryption prerequisite), preallocated number blocks, provisional PDFs, SSE + polling sync, 30-day cache retention, on-demand regeneration, sync indicators.
- **M3 (hardening):** audit, device registration/revocation (`devices` table), MFA, encrypted backups, verification rate limiting, signed installers + auto-update channel (D-08), QA with real A4 printers.
- **M4 (beta release):** retention/export job behind `RETENTION_TRIM_ENABLED` (off until counsel), session purge, orphan cleanup, runbook, beta telemetry.

**v1.1 (billing, deferred):** `plans`/`tax_rates`/`subscriptions`/`payment_transactions`, trial, PayPhone API Sale checkout + callback + polling (gated on written PayPhone confirmation), renewal/dunning scheduler, `can_sync` gate, image library (5/1 tiers), `/settings/billing`, state-based rendering rules.

M0 progress (2026-11-11): Diesel-backed sign-up/sign-in/sign-out with `POST /api/auth/*`, the `/login` page and session-guarded `/dashboard`, and the Docker Compose deployment (app container, `persona-data` volume, `/health` healthcheck) are implemented. Roles and patient CRUD remain.

## 12. Acceptance criteria (v1)

- [ ] Doctor creates a patient, picks their template, edits with rich-text, signs, and prints an A4 PDF with the clinic logo and an active QR.
- [ ] The same flow works offline on desktop using a preallocated number; on reconnect it syncs, is reconciled without duplication, and the final numbered PDF with an active QR is regenerated.
- [ ] Exhausting the device's number block offline blocks further signing with a visible warning and remaining count.
- [ ] Numbers are unique and monotonically increasing per clinic across devices; gaps from burned block tails are recorded in `rx_blocks` and never reused.
- [ ] A PDF ≥ 31 days old is not in the local cache; it regenerates on demand from the immutable version with the same content (not byte-identical).
- [ ] Scanned QR shows "Receta auténtica" without exposing patient data; a provisional offline PDF carries no scannable QR.
- [ ] Medication shows an LLM link with the drug name only.
- [ ] The print title follows `{Patient}-receta-{dd-mm-aaaa}-{Doctor}.pdf` in `es-EC`.
- [ ] Secretary cannot edit a signed prescription; nursing cannot sign one; everything is audited.
- [ ] A clinic uploads one 10 MB image; a second upload and any file over 10 MB are rejected; the logo does not consume the slot; uploads are refused offline.
- [ ] An image referenced by a published template version or a retained prescription cannot be deleted.
- [ ] Server retention keeps the 3-month window; data beyond it is exportable during the 30-day grace; trimming does not run while `RETENTION_TRIM_ENABLED` is off.
- [ ] Auto-update applies only a manifest with a valid signature and rolls back on a bad signature or failed launch.

## 13. Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Client print-to-PDF fidelity and platform print APIs (WKWebView/WebView2 in Dioxus 0.7 desktop) | editor output ≠ printed page; no server renderer to fall back to | one print HTML/CSS source; spike the platform print call before M1; `window.print()` fallback; fix mismatches only in print CSS, never with a second template |
| Print dialog UX | no silent download; filename comes from `document.title`; the user can cancel | label actions *Imprimir / Guardar como PDF*; measure first-attempt success ≥ 98% |
| Preallocated numbering | gaps from burned blocks; exhaustion blocks signing; replayed duplicates | `BEGIN IMMEDIATE` allocation, server validation on reconcile, unique index, remaining-count UI, configurable block size, unused tails recorded in `rx_blocks` |
| Desktop auth transport | the client posts to absolute `http://127.0.0.1:8080` and `reqwest` has no cookie jar, so desktop sessions do not persist | move application data to Dioxus server functions (preferred) or add a configurable origin + cookie jar + credentialed CORS; blocking for M2 |
| Dioxus 0.7 rich-text interop | Tiptap/ProseMirror bridge and CSS isolation are non-trivial | editor spike before M1; sanitized structured JSON only, never raw HTML |
| Single-node SQLite concurrency | `SQLITE_BUSY` under the pool plus jobs | WAL, `busy_timeout`, retry policy for write transactions before M2 |
| Free-text medications | spelling/dose errors carry clinical risk | visible disclaimer + mandatory doctor verification; Phase 2 catalog |
| Self-hosted operations | backups, TLS, updates are our responsibility | runbook, encrypted daily file backup, RPO ≤ 24 h, RTO ≤ 4 h |
| 3-month server retention vs. clinical-record law | destroying records that must be kept, or keeping a window that contradicts the 5-year audit rule | `RETENTION_TRIM_ENABLED` off by default; counsel sign-off before enabling; encrypted archive if mandatory retention is confirmed |
| Image cost/abuse | orphaned objects and decoder attacks | 1 × 10 MB cap in v1, magic-byte + decode limits, quota inside `BEGIN IMMEDIATE`, orphan cleanup, private bucket |
| Image privacy | EXIF/GPS or PHI in files | EXIF stripping, JPEG/PNG only, no PHI in object keys, 15-minute signed URLs |
| Auto-update supply chain | a rogue release could exfiltrate PHI | signed manifest, server-side key custody, staged rollout, rollback, previous version retained |
| Legal validity of PDF/QR | printed reprint may not satisfy MSP/ACESS expectations | doctor signs the printed paper; counsel review (§15 O-01) |
| Deferred billing (v1.1) | PayPhone one-off model, 5-minute approval, dunning, trial abuse | written PayPhone confirmation required; the §5.7 design is retained unchanged |

## 14. Decision log

| ID | Date | Decision | Rationale | Consequence |
|---|---|---|---|---|
| D-01 | 2026-11-11 | PDF is produced client-side from the A4 print view; no server Chromium and no PDF library | offline desktop must print without a network; removing the renderer removes a deployment dependency | "Download" becomes print-dialog → *Save as PDF*; filename via `document.title`; "identical re-download" means same content, not bytes |
| D-02 | 2026-11-11 | Prescription numbers come from preallocated per-device blocks (default 50) | signing offline with a real number is impossible with server-assigned, gap-free numbering | gaps from burned block tails; exhaustion blocks signing; server reconciles on sync |
| D-03 | 2026-11-11 | One clinic per user; `clinic_members.role` is the only role authority | removes the clinic switcher and the `profiles.role` ambiguity from v1 | `profiles.role` removed; multi-clinic returns only if beta demands it |
| D-04 | 2026-11-11 | Billing is deferred to v1.1; v1 is a free beta | PayPhone's one-off model is unconfirmed and must not block launch | no billing tables, gates, jobs, or UI in v1; §5.7 retained as the v1.1 design |
| D-05 | 2026-11-11 | On cancellation (v1.1) every object is purged and reprints render placeholders | simplest storage lifecycle, explicitly chosen | historical reprints lose logo/images; recorded limitation in FR-085 |
| D-06 | 2026-11-11 | No signature or stamp images; the doctor signs the printed paper | signatures are physical, not digital | `assets.kind` is `logo` or `image`; footer carries a printed signature line |
| D-07 | 2026-11-11 | Desktop cache relies on OS disk encryption (FileVault/BitLocker) | avoids SQLCipher build and key-recovery complexity | shared or unencrypted machines are unsupported; documented prerequisite |
| D-08 | 2026-11-11 | Auto-update ships in v1 | a desktop app without updates strands beta users on fixed bugs | signed manifest, staged rollout, rollback; part of M3 |
| D-09 | 2026-11-11 | Keep the 3-month server window + 30-day export grace; trimming is flag-gated | user decision, reconciled with the counsel prerequisite in §9 | `RETENTION_TRIM_ENABLED` off by default; retention dates live on `clinics` |
| D-10 | 2026-11-11 | One library image per clinic in v1 | free beta; the 5-image library arrives with billing | logo is excluded; v1.1 restores the state-based quota |
| D-11 | 2026-11-11 | Sync uses SSE from the existing axum router with a 30-second polling fallback | Dioxus server functions are request/response and a second socket server is unwanted | one server process; polling covers proxies that break SSE |

## 15. Open questions

| ID | Question | Owner | Blocks |
|---|---|---|---|
| O-01 | Ecuadorian counsel sign-off: prescription retention, legal validity of the PDF/QR, and acceptance of a printed signature applied after printing | Legal | Production (not M0–M2) |
| O-02 | Does public verification need a second factor (e.g. last 4 digits of cédula), or is the opaque code alone acceptable? | Product + Legal | FR-050 hardening (M3) |
| O-03 | Does Dioxus 0.7 desktop expose a reliable print path on WKWebView and WebView2? | Engineering spike | M1 desktop print |
| O-04 | Is a 50-number block the right size for typical consultation volume? | Product | Tune after beta telemetry; config change only |
| O-05 | Written PayPhone confirmation of the one-off-per-period model | Business | v1.1 (M4) |
| O-06 | Do beta doctors need to work across multiple clinics? | Product | v1.1; would reverse D-03 |
| O-07 | Backup retention window, encryption-key custody, and a tested restore drill | Ops + Security | M3 |
| O-08 | Auto-update signing-key custody, rotation, and rollback drill | Security | M3 |
| O-09 | With billing deferred, should the 3-month server window apply to every free-beta clinic once trimming is enabled (it would remove records older than 3 months for all clinics), or stay dormant until v1.1 billing? | Product + Legal | Enabling `RETENTION_TRIM_ENABLED` |

---
*Note: the repository already contains the Dioxus + router + fullstack base with working Diesel-backed authentication (sign-up/sign-in, `/login`, session-guarded `/dashboard`) and a Docker Compose deployment — this PRD is implemented on top of it, not as a new project.*
