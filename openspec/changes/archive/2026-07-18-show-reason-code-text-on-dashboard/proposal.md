## Why

The dashboard already shows the raw `reason-code` (e.g. `RC3.1`) thanks to the prior `show-reason-code-on-dashboard` slice, but that opaque string is meaningless without cross-referencing `arduino/TempController/TempControllerRules.md`. When an operator sees unexpected heating or cooling, they have to leave the dashboard and look up the code in firmware docs to understand the controller's rationale. Adding the human-readable description inline closes that loop entirely in the UI. This is the explicitly-tracked follow-up from the prior slice (recorded in `openspec/changes/archive/2026-07-15-show-reason-code-on-dashboard/design.md` and `BACKLOG.md:5`).

## What Changes

- Introduce a static Rust mapping from each documented reason code (`RC1`, `RC2.1` … `RC9.3`, `RC5`, `RC10`) to its human-readable description, sourced from `arduino/TempController/TempControllerRules.md:27-58`.
- Compute the description for the latest reading in the dashboard handler and pass it to the template as a new `reason_text` field on `StatusContext`.
- Render the description in the existing `Reason` row of `partials/status.html`, in the same `<dd>` as the raw code, formatted `RC3.1 — <description>`.
- When the code is unknown, empty, or `RC_ERR`, show only the raw code (no trailing text) — the operator still sees the code for debugging, and the gap is an honest signal that the mapping needs updating.
- Update the Rust test fixtures that currently emit the non-firmware string `"below-target"` to emit a real firmware code (`RC3.1`), so tests exercise the real mapping.
- Update affected snapshot and assertion tests to reflect the new markup and the real code.

## Capabilities

### New Capabilities

<!-- None: this change extends an existing capability's display behavior. -->

### Modified Capabilities

- `web-dashboard`: The dashboard and live-polled `/status` fragment SHALL additionally display a human-readable description of the current reason code, derived from a static code-to-text mapping, immediately alongside the raw code value. The description SHALL be omitted (leaving only the raw code) when the code is unrecognized, empty, or an error sentinel.

## Impact

- **New code:** `fermenter/src/reason_code.rs` — a static lookup `fn reason_text(code: &str) -> Option<&'static str>` plus unit tests covering every documented code, `RC_ERR`, the empty string, and an unknown code.
- **Handler:** `fermenter/src/web/handlers.rs` — add `reason_text: Option<String>` to `StatusContext` and populate it in `status_context()`.
- **Template:** `fermenter/templates/partials/status.html:7` — extend the `Reason` `<dd>` to append ` — {{ reason_text }}` when present.
- **Tests:** `fermenter/src/web/handlers.rs` and `fermenter/src/web/mod.rs` — update `sample_reading()` fixtures and assertions from `"below-target"` to `"RC3.1"` plus its description.
- **Snapshots:** `dashboard_renders_snapshot.snap` and `status_fragment_with_reading_snapshot.snap` — refreshed via `INSTA_UPDATE=always cargo test`.
- **No impact** on the serial contract, `Reading` struct, Redis schema, firmware, or any API endpoint shape. The mapping is Rust-side only; the raw code remains the source of truth on the wire.
