# Divergence 05-E-taino-edit-wasm-only-desktop

Source spec: `specs/10-J-desktop-offline.md` divergence 1.
Blocked criteria: objective 5 (offline draft → sign in the desktop binary).

## What's broken

The desktop binary has no rich-text editor to draft prescriptions with.
taino-edit is wasm-only: its DOM bridge compiles only to wasm32, so the
`desktop` feature cannot depend on it.

## Why each fix was fenced

- Serve the wasm client inside the desktop webview: fenced until the
  webview strategy is resolved.
- Native fallback editor: fenced (no spec owns it).
- Upstream native adapter: no upstream target exists.

The dependency is installed target-gated per spec 05-E.

## How to clear

1. Later spec resolves the webview strategy.
2. Desktop build runs the full editor.
3. Re-run objective 5's flow and check the box.
4. Delete this file.
