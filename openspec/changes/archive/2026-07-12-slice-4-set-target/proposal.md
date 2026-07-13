## Why

Slice-3 gave the controller a read-only dashboard, but an operator still can't
change anything from it — the target temperature is baked into mock data and
there is no path from the browser to the Arduino. This is the rewrite's first
write path end-to-end (UI → in-memory state → device), unlocking the
`temperature-control` capability and the `watch`-channel plumbing
`rewrite-plan.md` §5 sketches for target reconcile, without which the
controller can't do the one thing a fermentation controller is *for*:
holding a chosen temperature.

## What Changes

- Add a `watch::channel<f64>` for the target temperature (`rewrite-plan.md`
  §5): the web layer sends on it when an operator submits a new target; the
  ingest/serial side observes it and writes `<target>` to the device when it
  differs from the Arduino's last-reported target — replacing the old
  Python system's periodic reconcile with an event-driven push.
- Add `GET /target` (form showing the current target) and `POST /target`
  (validate + apply a new target) routes to the web layer. This is the
  dashboard's first mutating route, so the existing web-dashboard requirement
  "no mutating routes are exposed" is narrowed to apply only to the dashboard
  and status-fragment routes, not the whole capability.
- Wire `ControllerState`/`TimeStore::save_state`/`load_state` (already defined
  in `model.rs`/`store/mod.rs`, unused since slice-2) into `main.rs`: persist
  the target temperature on every change and rehydrate it on startup, so a
  restart doesn't silently reset the setpoint to the env-var default.
- Extend `MockSerialSource` usage/tests so a target write can be asserted
  (the trait already has `write_target`; this slice is its first caller).
- Basic validation: reject out-of-range or non-numeric target submissions
  with a re-rendered form and an error message, rather than a panic or a
  silently-accepted bad value.

## Capabilities

### New Capabilities

- `temperature-control`: Target setpoint capability — accepting a new target
  from the web layer, holding it as authoritative in-memory state, and
  reconciling it to the device over the serial write contract when it
  differs from what the device last reported.

### Modified Capabilities

- `web-dashboard`: adds a target-setpoint form (`GET`/`POST /target`) and
  narrows the existing "dashboard rendering is read-only" requirement so it
  applies specifically to the dashboard page and status fragment, not the
  whole capability.
- `device-connection`: the `<target>` write framing scenario (already
  specified in slice-1) gets its first real caller; adds a requirement that
  the system writes the target only when it differs from the device's most
  recently reported value, to match `rewrite-plan.md` §5's event-driven
  reconcile.

## Impact

- **Modified modules**: `web/mod.rs` (`AppState` grows a
  `target_tx: watch::Sender<f64>`), `web/handlers.rs` (new `target_form` and
  `set_target` handlers), `ingest.rs` (reconcile check against the `watch`
  receiver after each reading), `main.rs` (build the channel, seed it from
  rehydrated/persisted state, wire it into both the ingest task and
  `AppState`).
- **New templates**: `templates/target_form.html` (`{% extends "base.html"
  %}`), extending `dashboard.html` with a link/control to reach it.
- **No new dependencies** — `tokio::sync::watch` is already part of the
  `tokio` "full" feature already in `Cargo.toml`.
- **No impact** on the Redis schema beyond exercising the already-defined but
  previously-unused `save_state`/`load_state` `TimeStore` methods.
- **No impact** on the Python system, which remains deployable on `master`
  until cutover.
