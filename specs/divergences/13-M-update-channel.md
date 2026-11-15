# Divergence tracking: 13-M update channel

Source spec: `specs/13-M-ops-and-release.md`, objective 4 (auto-update). Blocks: objective 4 — "consuming the signed manifest at `UPDATE_MANIFEST_URL` over HTTPS with staged rollout … the client verifies signature and version before applying"; the testing criterion `objective 4: an update manifest with a bad signature is refused and the previous version still runs; a valid staged manifest applies and can roll back`; and `context/PRD.md:263` (auto-update acceptance).
Resolving spec: none — requires an update-remediation spec. `13-M-signed-installers.md` covers only the missing Apple/Windows signing credentials; it does not cover these code-level gaps and must not be cleared first.

Severity: P1/P2 — staged rollout is not staged at all. Update transport can accept an unauthenticated artifact over plain HTTP while the signed manifest still verifies.

## What's broken

1. **Empty rollout identity.** `persona-sanus/src/update.rs:122` reads `PERSONA_DEVICE_ID`; nothing in this repository sets it. `run_check` therefore passes `""` to `evaluate`, and `in_rollout` hashes empty input. Reproduced 2026-11-15 with Python: `SHA256("")[0] % 100 = 27`. Any rollout of 1-27% excludes every install; 28-99% includes every install. Only 0% and 100% behave as written. “Staged rollout” is publish/no-publish.
2. **No HTTPS enforcement.** The manifest URL comes from `UPDATE_MANIFEST_URL` (`update.rs:116-124`), and the advertised artifact URL is fetched at `update.rs:91-93` without a scheme check. An operator can configure `http://`; a network attacker can then replace the binary because the signature covers `version|rollout|url`, not the artifact bytes. A valid manifest pointing at an HTTP URL is a valid attack.
3. **No artifact digest.** The manifest carries no SHA-256 of the installer. TLS is the only integrity check on the download, and the code does not require TLS.

## Why each fix was fenced

- The 2026-11-15 pass was report-only. The required fixes are code changes in `update.rs` and need an update-manifest/signing compatibility review.
- The device-id fix needs a two-installation E2E test, not a unit test; no signed installers or two desktop environments exist here yet, and the repository rule is E2E only.
- Adding an artifact hash changes the canonical string signed by the server. Existing dev clients and the signer must agree on a format version before the change; that is a release-contract decision.
- No credential blocks the device-id or scheme check; the missing certificates block only the final staged-apply/rollback proof.

Verified 2026-11-15: 0 fix attempts — static reading of `update.rs`, `jobs.rs`, `runbook.md`, and the existing tracker, plus one throwaway Python reproduction of the empty-input bucket. All repair is fenced by the spec process and the missing signed-installer prerequisites, not by tooling.

## How to clear it

- [ ] Device id: keep `PERSONA_DEVICE_ID` only as a test override, then fall back to `crate::device::device_id()`. Never pass an empty string into `evaluate`.
- [ ] Transport: parse `UPDATE_MANIFEST_URL` and the manifest's `url`; require `https` in release and non-debug builds. Allow `http` only for explicit loopback development (`127.0.0.1`, `::1`, `localhost`) and only via an explicit debug/insecure flag. Reject everything else before any network call or filesystem write.
- [ ] Stronger integrity: add a SHA-256 of the installer to the signed manifest and check it after download and before `.new`/`.prev` staging. This changes `canonical_manifest`; coordinate server signer and client rollout, or defer as a separate OPEN item rather than claiming binary integrity.
- [ ] Verify rollout: with two persisted device ids whose buckets differ, set `UPDATE_MANIFEST_ROLLOUT=50`; exactly one stages and one reports `NotInRollout` (repeat with 0 → none, 100 → both). The device id must be stable across relaunches.
- [ ] Verify transport: set `UPDATE_MANIFEST_URL=http://…` in a release build and assert refusal before fetch; set the signed manifest `url` to HTTP and assert refusal; use a loopback debug build to confirm the explicit dev exception still stages.
- [ ] Run the signed-installer end-to-end checks from `13-M-signed-installers.md` (bad signature refused, previous version runs, valid stage applies and rolls back).
- [ ] Update `context/runbook.md` §6 and the manifest contract in `context/architecture.md`; then mark 13-M objective 4 and its criterion `[x]`, and delete this file only after the installer tracker also clears.
