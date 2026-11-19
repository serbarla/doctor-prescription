# Divergence tracking: done-spec checkbox integrity

Source specs: `specs/done/11-K-sync.md`, `10-J-desktop-offline.md`, `22-V-ui-templates.md`, `23-W-ui-prescriptions.md`, `24-X-ui-settings-detail-and-audit.md`; housekeeping rule in `specs/TEMPLATE.md` (done = every box checked, §4 reads "None." or lists divergences, then move to `specs/done/`).
Resolving spec: none — bookkeeping fix only; no product code. The specs stay in `specs/done/`; this tracker records that their completion state is not machine-readable as claimed.

Severity: P3 — human readers see finished specs; checkbox scanners (including `.blocks/spec-queue.py` and plain `grep '^- \[ \]'`) see unfinished or invisible work, so any future automation over `specs/done/` misclassifies them.

## What's broken

- `specs/done/11-K-sync.md:61`: `- [ ] every divergence found is appended below` is unchecked, while a duplicate of the same line at `:69` is `- [x]`. It is the only unchecked box in the 24 done specs (verified 2026-11-19: every other done spec is fully checked under the same count).
- `specs/done/10-J-desktop-offline.md:26-30,47-51,62`: every checkbox is written ` - [x]` with a leading space. A `^- \[` count returns 0/0 for the file — its objectives and criteria are invisible to column-0 parsers, unlike all other specs.
- `specs/done/22-V-ui-templates.md:84`, `23-W-ui-prescriptions.md:87`, `24-X-ui-settings-detail-and-audit.md:87`: §4 reads "None recorded yet." instead of the template's terminal "None.", leaving it ambiguous whether divergences were still expected after the move to `done/`.
- All 24 done specs (and `specs/13-M-ops-and-release.md`) predate the `Owned files` / `Gate` / `Desktop-verify` header fields added by 26-Z objective 1. This is by design (26-Z: legacy specs default to whole-spec ownership), but nothing in the done files records that default, so the scanner reports `whole_spec: true, owned_files: ["*"]` for work that is already complete.

## Why each fix was fenced

- This audit is report-only; editing done specs rewrites the historical completion record and should be done as one deliberate bookkeeping pass, not drive-by.
- The 10-J reformat touches every objective line; doing it without re-reading the file risks altering the resolved-divergence annotations appended to those lines.

Verified 2026-11-19: 0 fix attempts — per-file checkbox counts over `specs/done/*.md`, `grep` for the §4 divergence lines and "None" variants, header-field scan for `Owned files`. Fenced by the audit's report-only scope.

## How to clear it

- [ ] `11-K-sync.md`: check or remove the duplicate unchecked line 61 so exactly one divergence-attestation line remains, checked.
- [ ] `10-J-desktop-offline.md`: normalize ` - [x]` to `- [x]` at column 0 without changing any text after the checkbox; re-count to full checked.
- [ ] `22-V`, `23-W`, `24-X`: replace "None recorded yet." with "None." in §4.
- [ ] Verify: `grep -c '^- \[ \]' specs/done/*.md` returns 0 for every file and `python3 .blocks/spec-queue.py --specs-dir specs` is unchanged for the open queue.
- [ ] Delete this file after the counts verify.
