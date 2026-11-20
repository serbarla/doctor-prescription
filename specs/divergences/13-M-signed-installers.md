# Divergence tracking: 13-M signed installers

Source spec: `specs/13-M-ops-and-release.md`, §4 divergence 1. Blocks: objective 4 (`cargo check --no-default-features --features desktop` passes for the in-app mechanism, but no signed installer is produced or applied) and the testing criteria `objective 4: an update manifest with a bad signature is refused and the previous version still runs; a valid staged manifest applies and can roll back` and `every objective in section 1 is checked`.
Resolving spec: none — installer signing lives in the release process outside any numbered spec; the release pipeline must hold the certificates.

## What's broken

Signed, notarized macOS DMG and signed Windows MSI installers do not exist yet, and the staged-apply/rollback path has never been exercised against a real installer. The in-app mechanism is implemented (`persona-sanus/src/update.rs`, `GET /api/updates/manifest`, ed25519 signature + version verification before staging, staged rollout, `.prev` rollback) but code without signed installers is not a shippable update channel.

## Why each fix was fenced

- No Apple Developer ID certificate in this environment: cannot sign/notarize. (Verified 2026-11-13: 0 fix attempts — credential absent, fenced by spec 13-M §2 No-go.)
- No Windows code-signing certificate in this environment: cannot sign MSI. Same fence.
- Faking the gate (self-signed, unsigned with a note) is forbidden by the spec's No-go section.

## How to clear it

- [ ] Obtain Apple Developer ID (Application) certificate + notarization credentials and a Windows code-signing certificate; store per `context/runbook.md` §3.
- [ ] Build DMG/MSI in the release pipeline, sign and notarize them.
- [ ] Serve a signed manifest (`UPDATE_SIGNING_KEY` / `UPDATE_PUBLIC_KEY`, `GET /api/updates/manifest`) pointing at the installer.
- [ ] Re-verify: `cd persona-sanus && cargo check --no-default-features --features desktop`, plus the desktop end-to-end flow from `context/runbook.md` §6 — a tampered manifest is refused and the previous version still runs; a valid staged manifest applies and rolls back on failed launch.
- [ ] Mark spec 13-M objective 4 and its testing criterion `[x]`, then delete this file.

## Re-verification 2026-11-20

1 try used (of 10). All checklist items remain blocked by the external fence — no credential was invented, self-signed, or faked.

- Evidence: `security find-identity -v -p codesigning` → `0 valid identities found`; no `UPDATE_SIGNING_KEY`/`UPDATE_PUBLIC_KEY`/`APPLE_*`/`WINDOWS*` env vars; no `.p12`/`.pfx`/`.cer` files in repo root/worktree.
- Non-fenced verification: `cd persona-sanus && cargo check --no-default-features --features desktop` → EXIT:0 (2026-11-20, 4 pre-existing warnings) — confirms the in-app mechanism still compiles; does not clear the checklist.
- Remaining: obtain Apple Developer ID (Application) certificate + notarization credentials and a Windows code-signing certificate per `context/runbook.md` §3; then build/sign/notarize DMG/MSI, serve a signed manifest, re-verify the tampered/valid staged apply + rollback end-to-end flow, and only then check spec 13-M objective 4 and delete this file.

## Deferral decision 2026-11-20

User approved keeping this divergence OPEN: no certificate work is possible in this environment, all checklist boxes above stay unchecked, and this tracker is kept in place. No spec 13-M box is checked and no installer is signed. This divergence clears only when the release process obtains the Apple Developer ID certificate plus notarization credentials and the Windows code-signing certificate per `context/runbook.md` §3, then completes the checklist above.
