# Spec 48-AV — Doctor template image upload

- Source: `context/PRD.md` §3, flow 8, FR-020, FR-080–085, D-06; `context/screens/TemplateEdit.md` §§2.1–2.4; `context/screens/SettingsClinic.md`; `persona-sanus/src/forms.rs` (`TemplateDocumentForm`), `src/service.rs` (`upload_owner`, `upload_asset`, `publish_template_version`), `src/api_server.rs` (`upload_asset`), `src/assets.rs`.
- Depends on: 47-AU.
- Project root: `/Users/sergiobarba/projects/doctor-prescriptions`. All paths and commands below are relative to it.
- Owned files: `specs/48-AV-doctor-image-upload.md`, `persona-sanus/src/forms.rs`, `persona-sanus/src/main.rs`, `persona-sanus/src/api.rs`, `persona-sanus/src/service.rs`, `persona-sanus/src/api_server.rs`, `persona-sanus/src/db.rs`, `persona-sanus/assets/*.css`
- Gate: `cargo check --manifest-path persona-sanus/Cargo.toml --no-default-features --features desktop && cargo check --manifest-path persona-sanus/Cargo.toml --no-default-features --features web,server`
- Desktop-verify: yes
- User-interface verification: `required` — desktop template editor upload, selection and offline error workflows in §3; capture non-PHI screenshots and outcome evidence.
- Result carried forward: An authorized doctor can upload an image online from their own template editor and select it without bypassing clinic quota, validation or pinning.

## 1. Objectives and scope

- [ ] 1. Add a focused online image-library upload to the existing template editor for an owning doctor, reusing the current file input, API upload and feedback conventions. Do not grant a doctor edit access to a clinic master or another doctor's template; hide upload when editing is not authorized. After success refresh the library and make the returned ready image selectable in the existing image slot; preserve unsaved form values and do not silently publish. Describe the shared clinic slot and show quota, upload progress/failure and offline unavailability in `es-EC` without billing copy. A failure, cancelled picker or double click must not lose draft edits or fabricate an asset.
- [ ] 2. Preserve and enforce the actual server trust boundary: `service::upload_owner` already admits doctors for `kind=image` with a doctor profile and derives clinic/owner from the actor; reuse it and server-side template access, never trust client-provided owner/clinic. Ensure the editor flow cannot attach another clinic's image or use a non-ready/deleted asset; publishing validates and pins every referenced image as it does today. Preserve administrator clinic-library behavior and existing read rules; nursing/secretary and unauthenticated callers cannot upload. Do not broaden logo permissions as a shortcut.
- [ ] 3. Keep FR-080–085 intact: 1 non-deleted library image per clinic shared across owners, 10,485,760-byte streaming limit, decoded JPEG/PNG only, strip metadata, private object storage/signed reads, atomic concurrent quota enforcement, audit on success, no PHI in object keys. Logos do not consume the slot. The over-quota response remains `409 image_quota_exceeded`; pinned published versions or retained prescriptions prevent deletion, and only an actually deletable image frees the slot. Storage, validation or permission failures must not leave a usable orphan or falsely claim success.

### Out of scope / no-go

- No automated tests, offline image upload/queue, new role, public bucket, PHI image attachments, signature/stamp, rich text, billing or v1.1 image entitlement behavior. Existing local cached images continue to render offline; this spec does not implement image sync.
- Do not invent a doctor logo upload path; the existing logo service is administrator-only. If changing its product authorization becomes necessary, stop and request scope approval.

## 2. Problem and decision

- Problem: PRD allows doctors to upload images for their own templates and the service already supports doctor image ownership, but the template editor only selects existing assets; clinic settings is administrator-only.
- Decision: Recommended UI entry point in the existing image section, calling the existing server upload path. The server remains authoritative for identity, shared quota, format and publication pins; no independent doctor quota or parallel storage path is introduced.
- Open questions: None.

## 3. Acceptance and verification

Use a disposable clinic with two doctors, administrator, nursing and secretary and synthetic non-PHI PNG/JPEG files. Record response codes and screenshots without credentials or signed URLs.

- [ ] Desktop workflow: with empty library, doctor opens own editor, changes unsaved text, uploads a valid PNG and selects it; text stays unchanged, image appears in the slot, saving and publishing pin it. A second image upload by either doctor or administrator returns the quota error; a logo is not counted. Pinned image deletion is refused and published preview remains intact.
- [ ] Authorized API and UI workflow: doctor cannot upload via another doctor's or clinic-master editor; nursing, secretary and signed-out callers cannot upload directly, and cross-clinic or deleted/non-ready image IDs cannot be attached/published. Administrator clinic upload remains functional. Verify rejected operations create no usable image or successful upload audit.
- [ ] API workflow: submit invalid bytes disguised as PNG, WebP/SVG, empty and >10 MiB uploads → rejection; valid JPEG with synthetic metadata is decoded and stripped. Start two valid uploads concurrently for an empty slot → at most one succeeds and the other receives 409; after an unpinned image is deleted the slot can be reused. A storage outage leaves no ready orphan or false success. Check private object key contains only opaque IDs and signed access is temporary.
- [ ] Desktop workflow: disconnect before upload, attempt it, cancel a picker and try again after reconnect → clear non-success and no queued upload while offline, existing edits retained; online success follows ordinary authorization/quota checks.
- [ ] Gate: `cargo check --manifest-path persona-sanus/Cargo.toml --no-default-features --features desktop && cargo check --manifest-path persona-sanus/Cargo.toml --no-default-features --features web,server` exits 0.
- [ ] User-interface verification: perform the desktop workflows above; retain screenshots without PHI.
- [ ] Every objective in section 1 is checked.

## 4. Execution and divergences

Read project instructions first. If a dependency is incomplete or required work is outside owned files, record evidence, impact and affected criteria and ask the project owner for scope approval; do not improvise. Unresolved divergences block completion.

### Divergences

None.
