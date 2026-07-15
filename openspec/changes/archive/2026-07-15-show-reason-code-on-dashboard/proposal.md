## Why

The Arduino firmware already transmits a `reason-code` field in every JSON reading — codes like `RC1`, `RC3.1`, `RC5` that explain *why* the controller chose its current action (e.g., failsafe-min heating triggered, in-band with natural heating, failsafe-max cooling triggered). The Rust ingest pipeline already deserializes and stores this field. However, the dashboard does not display it, forcing operators to guess at the controller's decision logic when diagnosing unexpected heating or cooling behaviour.

## What Changes

- Add a `Reason` row to the reading `<dl>` in the status template (`partials/status.html`), displaying `{{ reading.reason_code }}` immediately after the existing `Action` row.
- Update the snapshot test for the status fragment to reflect the new markup.

## Capabilities

### New Capabilities

<!-- None: this change only adds a display field to an existing capability's template. -->

### Modified Capabilities

- `web-dashboard`: The dashboard reading display SHALL include the current `reason_code` value alongside `action`, so the operator can see the controller's decision rationale without consulting serial logs.

## Impact

- **Template:** `fermenter/templates/partials/status.html` — add one `<dt>/<dd>` pair.
- **Snapshot:** `fermenter/src/web/snapshots/fermenter__web__handlers__tests__status_fragment_with_reading_snapshot.snap` — auto-updated via `cargo insta accept` or `INSTA_UPDATE=always cargo test`.
- No backend, model, serial, or Redis changes required.
