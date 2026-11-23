# Spec 35 — Phase AI: Update channel and signed-installers close

- Source: `specs/divergences/13-M-signed-installers.md` and `specs/divergences/13-M-update-channel.md` (merged 2026-11-20; both trackers deleted at this spec's creation — the update-channel tracker defers its last two boxes to the signed-installer tracker explicitly, so the two files form one dependency chain and one spec; `specs/13-M-ops-and-release.md` objective 4 and its section 3 update criterion).
- Depends on: 34-AH.
- Repo root: `/Users/sergiobarba/projects/doctor-prescriptions`. Every path and command below is relative to it.
- Owned files: `persona-sanus/src/update.rs`, `persona-sanus/src/device.rs`, `persona-sanus/src/jobs.rs`, `context/architecture.md`, `.blocks/update-channel-verify.py`, `specs/13-M-ops-and-release.md`, `specs/35-AI-update-channel-and-signed-installers.md`
- Gate: `caveman-blocks run update-channel-verify --repo . --port 8092`
- Desktop-verify: yes
- Result carried forward: the ed25519 update channel is proven green on the current tree and, once the release process holds the certificates, a signed macOS DMG and Windows MSI apply and roll back end to end; until then this spec records the explicit OPEN deferral and 13-M objective 4 stays unchecked.

## Before you start

Unit testing is not required; avoid it. Prioritize end-to-end and integration testing only.

Read the nearest `AGENTS.md` (target workspace, then parents up to repo root) before executing.

If reality diverges from this spec: STOP, append a divergence (section 4), and NEVER improvise outside the objectives.

## 1. Objectives

*The ordered work to implement — one item per step. Check `[x]` only after the step is implemented and verified.*

*Item format:* `<verb phrase>` — requirements: `<what must change / constraints>`; commands: `<exact command>`.

- [ ] 1. Re-verify the update mechanism on the current tree — requirements: run the channel probe (rollout 50/0/100, empty-device refusal, device-id stability, HTTPS/loopback transport, SHA-256 digest, bad-signature refusal) and the desktop compile check, confirming the `version|rollout|url|sha256` canonical string is agreed between `update.rs` and the server signer in `jobs.rs`; any drift found is fixed within Owned files only; commands: `caveman-blocks run update-channel-verify --repo . --port 8092`
- [ ] 2. Obtain release credentials and build signed installers — requirements: hold the Apple Developer ID (Application) certificate with notarization credentials and the Windows code-signing certificate per `context/runbook.md` section 3, build the DMG/MSI in the release pipeline, sign and notarize them, and serve a signed manifest (`UPDATE_SIGNING_KEY` / `UPDATE_PUBLIC_KEY`, `GET /api/updates/manifest`); if credentials are unavailable, stop, append a section 4 OPEN divergence recording the evidence (zero codesigning identities, absent env vars and `.p12`/`.pfx`/`.cer` files), and check nothing; commands: `cd persona-sanus && cargo check --no-default-features --features desktop`
- [ ] 3. Prove staged apply and rollback with real installers — requirements: with the signed artifacts served, run the `context/runbook.md` section 6 desktop flow: a tampered manifest is refused and the previous version still runs, a valid staged manifest applies, and a failed launch rolls back to the previous version; the GUI pass records the observed window states (this is a desktop visual smoke only — the Gate must still exit 0); commands: `caveman-blocks run update-channel-verify --repo . --port 8092`
- [ ] 4. Close 13-M — requirements: only after objective 3 passes, check `specs/13-M-ops-and-release.md` objective 4, its section 3 update criterion, and its objective 5 criterion (whose remaining evidence spec 34-AH records), then move the spec to `specs/done/13-M-ops-and-release.md`; commands: `cd persona-sanus && cargo check --no-default-features --features desktop`

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No faking, self-signing, unsigned-with-note, or otherwise fabricated signed-installer proof; no checking any box of the merged trackers or of 13-M objective 4 while the credentials are absent — the OPEN deferral is the honest state (13-M section 2 rule applies unchanged).
- No editing `docker-compose.yml`, `.env.example`, `context/runbook.md`, `persona-sanus/src/server.rs`, `specs/26-Z-*`, `specs/31-AE-*`, `specs/33-AG-*`, `specs/34-AH-*`, `specs/36-AJ-*`, or anything outside Owned files. No clearing any 26-Z gate, no changing the manifest contract without the server/client agreement recorded in `context/architecture.md`.
- No new updater dependency (the mechanism is the in-house `update.rs` downloader); no unit tests; no comments in any code.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [ ] `caveman-blocks run update-channel-verify --repo . --port 8092` returns `{"ok": true}` with every `checks` value true; exit 0
- [ ] objective 2: signed and notarized DMG/MSI artifacts exist from the release pipeline (or the section 4 OPEN divergence records the credential evidence)
- [ ] objective 3: tampered-manifest refusal, staged apply, and failed-launch rollback observed against the real installers and recorded here
- [ ] objective 4: 13-M objective 4, its update criterion, and its objective 5 criterion are checked and the spec sits in `specs/done/`
- [ ] every objective in section 1 is checked

## 4. Divergences

*Process and tracking-file rules: see `AGENTS.md` (§ Spec-Driven Development). Summary: when reality diverges from the spec, append a numbered entry below; leave the matching testing-criteria `- [ ]` unchecked and note the divergence inline next to it. Open divergences owned by a later spec also get a tracking file in `specs/divergences/`.*

- [ ] every divergence found is appended below (or the section reads "None.")

*Entry format (one numbered item per divergence):*

1. <what diverged and its current state> — <why it could not be done as written, citing the No-go rule or spec that blocks it> (Verified <date>: <N> fix attempts — <attempt 1>, <attempt 2>, <attempt 3> — all fenced by <constraint>; <impact> is deferred to <later spec> by design, not a defect.)

### Divergences

None yet.
