# Spec 37 — Phase AK: Integrate rust-ui/icons icon library

- Source: User request (2026-11-23) to install https://github.com/rust-ui/ui/tree/main/crates/icons and replace the custom icon implementation in `persona-sanus`.
- Depends on: 15-O–25-Y (UI component library; established the current `Icon`/`IconName` contract in `persona-sanus/src/ui/`).
- Repo root: `/home/orca/projects/doctor-prescription`. Every path and command below is relative to it.
- Owned files: `persona-sanus/Cargo.toml`, `persona-sanus/Cargo.lock`, `persona-sanus/src/ui/icon.rs`, `persona-sanus/src/ui/mod.rs`, `persona-sanus/assets/main.css`, `context/architecture.md`, `context/Design.md`, `context/done-plans/UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md`.
- Gate: `cargo check --manifest-path persona-sanus/Cargo.toml`
- Desktop-verify: no
- Result carried forward: `persona-sanus` depends on the `icons` crate (Rust/UI, Lucide-based, Dioxus feature) for all icon rendering; the custom inline-SVG `Icon` implementation is replaced by a thin wrapper around the crate that preserves the existing `Icon`/`IconName` API; context files document the library and no longer forbid icon packages.

## Before you start

Unit testing is not required; avoid it. Prioritize end-to-end and integration testing only.

Read the nearest `AGENTS.md` (target workspace, then parents up to repo root) before executing.

If reality diverges from this spec: STOP, append a divergence (section 4), and NEVER improvise outside the objectives.

## 1. Objectives

*The ordered work to implement — one item per step. Check `[x]` only after the step is implemented and verified.*

*Item format:* `<verb phrase>` — requirements: `<what must change / constraints>`; commands: `<exact command>`.

- [x] 1. Add the `icons` crate dependency — requirements: add `icons = { version = "0.19", features = ["dioxus"] }` to `[dependencies]` in `persona-sanus/Cargo.toml`; the crate must be unconditional (shared UI code) and must compile with the existing `web`, `desktop`, and `server` feature sets; run `cargo fetch` to update `Cargo.lock`; commands: `cargo check --manifest-path persona-sanus/Cargo.toml && cargo check --manifest-path persona-sanus/Cargo.toml --features desktop && cargo check --manifest-path persona-sanus/Cargo.toml --no-default-features --features server`.
- [x] 2. Update `context/architecture.md` — requirements: add `icons 0.19` to the runtime dependency list in the Architecture header field and, in §2 (Compile-time split), note that `icons` is the sole icon library used by the shared `src/ui/` client layer; commands: `rg -n 'icons 0\.19' context/architecture.md`.
- [x] 3. Update `context/Design.md` — requirements: in the Imagery / Iconography section, state that icons are sourced from the `icons` crate (Rust/UI, Lucide-based) and rendered through `persona-sanus/src/ui/icon.rs`; keep the existing line-based, 1.5 px stroke, monochrome Ink/Deep Teal house rule; commands: `rg -n 'icons crate|rust-ui' context/Design.md`.
- [x] 4. Update `context/done-plans/UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md` — requirements: remove the blanket "No ... icon package" rule from Context and No-go; update the `Icon` component table to state it is a thin wrapper over `icons::DioxusIcon`/`IconType` that preserves `name`, `size`, and `extra_class`; commands: `rg -n 'icons::DioxusIcon|IconType' context/done-plans/UI_COMPONENT_LIBRARY_REUSABLE_UI_PLAN.md`.
- [x] 5. Replace `persona-sanus/src/ui/icon.rs` implementation — requirements: keep the public `Icon`/`IconName` API unchanged so callers need no edits; map each `IconName` variant to the corresponding `icons::IconType` per the mapping table in §5; render with `icons::DioxusIcon`; preserve the 20 px default size, 1.5 px stroke, `aria-hidden="true"`, `focusable="false"`, and the filled `Cross` treatment; wrap the icon in a span that carries `aria-hidden="true"` and sets `--icon-size` for CSS; commands: `cargo check --manifest-path persona-sanus/Cargo.toml`.
- [x] 6. Add icon styling to `persona-sanus/assets/main.css` — requirements: add `.ui-icon-wrap`, `.ui-icon`, and `.ui-icon--filled` rules; `.ui-icon` must size from the inherited `--icon-size` custom property (default 20 px), set `stroke-width: 1.5`, and use `currentColor`; `.ui-icon--filled` must set `fill: currentColor` and `stroke: none`; `.ui-icon-wrap` must be `display: inline-flex`; commands: `rg -n '\.ui-icon' persona-sanus/assets/main.css`.
- [x] 7. Verify no custom SVG paths remain — requirements: confirm no hardcoded SVG `d` path strings remain in `persona-sanus/src/ui/` and that `IconName` is only referenced inside `icon.rs` and at valid call sites; commands: `rg -n '\bd: "' persona-sanus/src/ui/ ; rg -n 'IconName' persona-sanus/src`.

## 2. No-go

*What this spec MUST NOT do, even if convenient. Never touch anything listed here.*

- No additional crate, CDN, icon library, webfont, CSS framework, or JS bridge beyond `icons`.
- No change to the public `Icon`/`IconName` API unless the crate makes it unavoidable (if so, record a divergence and update callers).
- No new icon shapes, meanings, or decorative marks; only replace the rendering source of icons already in use.
- No change to routes, API handlers, server functions, Diesel models/migrations, authorization, session, sync, PDF, QR, or business logic.
- No rich-text, WYSIWYG, or contenteditable editor (PRD D-21): all text entry stays plain Dioxus forms and inputs.
- No unit tests; no comments in Rust, CSS, or RSX.

## 3. Testing criteria

*Observable proof the spec is complete. Every box checked = spec done.*

- [x] `cargo check --manifest-path persona-sanus/Cargo.toml` passes (run from the repo root)
- [x] `dx build --release --fullstack` passes for `persona-sanus` (fullstack smoke)
- [x] objective 1: `grep -E '^icons = ' persona-sanus/Cargo.toml` shows the dependency with the `dioxus` feature
- [x] objective 5: `rg -n 'IconName' persona-sanus/src` shows only `persona-sanus/src/ui/icon.rs` and valid `IconName::` call sites
- [x] objective 7: `rg -n '\bd: "' persona-sanus/src/ui/` returns nothing.
- [x] every objective in section 1 is checked

## 4. Divergences

*Process and tracking-file rules: see `AGENTS.md` (§ Spec-Driven Development). Summary: when reality diverges from the spec, append a numbered entry below; leave the matching testing-criteria `- [ ]` unchecked and note the divergence inline next to it. Open divergences owned by a later spec also get a tracking file in `specs/divergences/`.*

- [x] every divergence found is appended below (or the section reads "None.")

*Entry format (one numbered item per divergence):*

1. <what diverged and its current state> — <why it could not be done as written, citing the No-go rule or spec that blocks it> (Verified <date>: <N> fix attempts — <attempt 1>, <attempt 2>, <attempt 3> — all fenced by <constraint>; <impact> is deferred to <later spec> by design, not a defect.)

### Divergences

1. Resolved: The original objective 7 verification command `rg -n 'd: "' persona-sanus/src/ui/` matched unrelated RSX attributes such as `aria_invalid`, `id`, and `kind`. The command now uses `\bd: "` to match the `d` attribute without matching those names. The corrected command returned no matches on 2026-11-27; no implementation change was required.

---

## 5. Icon mapping

`IconName` variant → `icons::IconType` used by the wrapper. This mapping keeps the public API stable.

| `IconName` | `icons::IconType` | Notes |
|---|---|---|
| `Cross` | `Cross` | Filled via `.ui-icon--filled` (`fill: currentColor; stroke: none`). The Lucide `Cross` shape is a plus-style medical cross, which matches the logo glyph semantic better than the previous X-shaped path. |
| `Shield` | `Shield` | — |
| `Command` | `Command` | macOS platform glyph in `TrustSecurityRow`. |
| `Windows` | `Monitor` | The `icons` crate has no Windows logo; `Monitor` represents desktop platforms generically in the trust row alongside `Command` and `Globe`. |
| `Globe` | `Globe` | Web platform glyph in `TrustSecurityRow`. |
| `Check` | `Check` | — |
| `Close` | `X` | — |
| `Alert` | `TriangleAlert` | Lucide alert-triangle. |
| `Info` | `Info` | — |
| `Qr` | `QrCode` | — |
| `ArrowRight` | `ArrowRight` | — |
| `Lock` | `Lock` | — |
| `Image` | `Image` | — |

If any mapping produces a visually wrong or missing icon, stop and record a divergence before changing the mapping or callers.

## 6. Implementation sketch

*Non-normative; the spec objectives are authoritative.*

`persona-sanus/src/ui/icon.rs` should look approximately like:

```rust
use dioxus::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconName {
    Cross,
    Shield,
    Command,
    Windows,
    Globe,
    Check,
    Close,
    Alert,
    Info,
    Qr,
    ArrowRight,
    Lock,
    Image,
}

impl IconName {
    fn to_icon_type(self) -> icons::IconType {
        match self {
            IconName::Cross => icons::IconType::Cross,
            IconName::Shield => icons::IconType::Shield,
            IconName::Command => icons::IconType::Command,
            IconName::Windows => icons::IconType::Monitor,
            IconName::Globe => icons::IconType::Globe,
            IconName::Check => icons::IconType::Check,
            IconName::Close => icons::IconType::X,
            IconName::Alert => icons::IconType::TriangleAlert,
            IconName::Info => icons::IconType::Info,
            IconName::Qr => icons::IconType::QrCode,
            IconName::ArrowRight => icons::IconType::ArrowRight,
            IconName::Lock => icons::IconType::Lock,
            IconName::Image => icons::IconType::Image,
        }
    }
}

#[component]
pub fn Icon(
    name: IconName,
    #[props(default = 20)] size: u16,
    #[props(default)] extra_class: String,
) -> Element {
    let filled = name == IconName::Cross;
    let icon_type = name.to_icon_type();
    let class = format!(
        "ui-icon{}{}",
        if filled { " ui-icon--filled" } else { "" },
        if extra_class.is_empty() { "".to_string() } else { format!(" {extra_class}") }
    );
    rsx! {
        span {
            class: "ui-icon-wrap",
            style: "--icon-size: {size}px",
            "aria-hidden": "true",
            "focusable": "false",
            icons::DioxusIcon { icon: icon_type, class: class }
        }
    }
}
```

CSS additions to `persona-sanus/assets/main.css`:

```css
.ui-icon-wrap {
  display: inline-flex;
  --icon-size: 20px;
}

.ui-icon {
  width: var(--icon-size);
  height: var(--icon-size);
  stroke-width: 1.5;
}

.ui-icon--filled {
  fill: currentColor;
  stroke: none;
}
```

*Housekeeping: once this spec is fully done with no open divergences, move its file to `specs/done/37-AK-icons-crate-integration.md`.*
