## 1. Config

- [ ] 1.1 Write a config-parse test (RED) for `DEFAULT_TARGET_TEMP` (with a
      default, e.g. `19.5`) in `config.rs`
- [ ] 1.2 Extend `Config` with `default_target_temp: f64` to pass the test
      (GREEN)

## 2. Target validation & reconcile (temperature-control) — TDD

- [ ] 2.1 Write unit tests (RED, `rstest` cases) for a pure validation
      function, e.g. `validate_target(input: &str) -> Result<f64, String>`:
      valid numeric string within range accepted; non-numeric rejected;
      below-range rejected; above-range rejected; boundary values (exactly
      the min/max) accepted
- [ ] 2.2 Implement `validate_target` (GREEN), with the allowed range as a
      named `const` (e.g. `TARGET_MIN: f64 = 0.0`, `TARGET_MAX: f64 = 40.0`)
- [ ] 2.3 Write unit tests (RED) for a pure reconcile function, e.g.
      `target_to_write(desired: f64, reported: f64) -> Option<f64>`:
      returns `Some(desired)` when `desired != reported`; returns `None`
      when equal
- [ ] 2.4 Implement `target_to_write` (GREEN)

## 3. Ingest loop reconcile wiring — TDD

- [ ] 3.1 Write a test (RED) in `ingest.rs`: given a `MockSerialSource` whose
      scripted `Reading`s report a `target` different from a seeded
      `watch::Receiver<f64>` value, running `ingest_loop` results in
      `source.written_targets` containing the desired value
- [ ] 3.2 Write a test (RED): when the `watch::Receiver`'s value matches the
      reading's reported `target`, `ingest_loop` results in no entries in
      `written_targets`
- [ ] 3.3 Write a test (RED): when the watch value changes partway through
      (updated between two readings), the write occurs only for the
      reading(s) processed after the change
- [ ] 3.4 Extend `ingest_loop`'s signature to accept a
      `target_rx: &watch::Receiver<f64>`; after each successfully parsed
      reading, borrow the current value, compare via `target_to_write`, and
      call `source.write_target(..)` when `Some` (GREEN for 3.1–3.3)
- [ ] 3.5 Confirm existing `ingest.rs` tests (slice-1/2) still pass with the
      new parameter (update call sites to pass a fixed/no-op
      `watch::channel` where reconcile isn't under test)

## 4. Persistence wiring (target_temp) — TDD

- [ ] 4.1 Write a test (RED, using `FakeTimeStore`) confirming a helper that
      seeds the initial target — e.g. `initial_target(store, default) ->
      f64` — returns the persisted `ControllerState.target_temp` when
      `load_state()` returns `Some`
- [ ] 4.2 Write a test (RED): the same helper returns the configured
      `default_target_temp` when `load_state()` returns `None`
- [ ] 4.3 Implement the `initial_target` helper (GREEN for 4.1/4.2), placed
      alongside `rehydrate_latest_reading` in `ingest.rs` (or a new
      `temperature_control.rs` module — see design.md's module-naming
      choice) for symmetry with the existing rehydration pattern
- [ ] 4.4 Write a test (RED) confirming that accepting a valid target (task
      2) triggers a `store.save_state` call with a `ControllerState`
      combining the new `target_temp` and the configured `brew_id`, using
      `FakeTimeStore` to assert the persisted value
- [ ] 4.5 Wire the save-state call into the target-acceptance path (GREEN
      for 4.4) — see task 5 for where this path lives (the `POST /target`
      handler)

## 5. Web layer: AppState & handlers (web-dashboard, temperature-control) — TDD

- [ ] 5.1 Extend `web::AppState` with `target_tx: watch::Sender<f64>`
- [ ] 5.2 Create `templates/target_form.html` (`{% extends "base.html" %}`):
      a form showing the current target temperature, an input field, a
      submit button, and an optional error message block
- [ ] 5.3 Add a link/control from `dashboard.html` to `GET /target`
- [ ] 5.4 Write an `insta` snapshot test (RED) rendering `target_form.html`
      with a seeded current-target context and no error
- [ ] 5.5 Write an `insta` snapshot test (RED) rendering `target_form.html`
      with an error message present
- [ ] 5.6 Implement the `/target` `GET` handler: builds a context from
      `AppState.target_tx.borrow()`'s current value, renders
      `target_form.html` (GREEN for 5.4/5.5)
- [ ] 5.7 Write a `tower::oneshot` HTTP test (RED): `POST /target` with a
      valid numeric form value returns 200, the response confirms the
      update, and `AppState.target_tx.borrow()` reflects the new value
- [ ] 5.8 Write a `tower::oneshot` HTTP test (RED): `POST /target` with a
      non-numeric value returns the redisplayed form with an error, and
      `target_tx`'s value is unchanged
- [ ] 5.9 Write a `tower::oneshot` HTTP test (RED): `POST /target` with an
      out-of-range numeric value returns the redisplayed form with an error,
      and `target_tx`'s value is unchanged
- [ ] 5.10 Implement the `/target` `POST` handler: parse the form value,
      call `validate_target` (task 2), on success update `target_tx` and
      call `store.save_state` (task 4), on failure re-render the form with
      the error — without touching `target_tx` or storage (GREEN for
      5.7–5.9)
- [ ] 5.11 Wire `GET /target` and `POST /target` routes into `build_router`

## 6. Read-only guarantee update (web-dashboard)

- [ ] 6.1 Update the existing "no mutating routes" test (from slice-3) so it
      asserts only against the dashboard (`/`) and status (`/status`) routes,
      not `/target` — confirm it still passes given the narrowed spec
      requirement
- [ ] 6.2 Write a test (RED) confirming `POST /target` with a valid value
      returns a non-error status (i.e. mutating IS permitted on this route,
      the deliberate exception carved out by the modified web-dashboard
      spec) — GREEN once task 5's handler exists

## 7. Main wiring

- [ ] 7.1 In `main.rs`, after building/rehydrating the store, call the
      `initial_target` helper (task 4) to compute the seed value, and
      construct `let (target_tx, target_rx) = watch::channel(initial_target)`
- [ ] 7.2 Pass `target_rx` into the ingest task's call to `ingest_loop`
      (task 3's new signature)
- [ ] 7.3 Add `target_tx` (cloned/wrapped as needed) to the `AppState`
      constructed for the web router
- [ ] 7.4 Confirm `MOCK_SERIAL=true` end-to-end: submitting a new target via
      `POST /target` results in the next processed mock reading (if its
      reported `target` differs) recording a `write_target` call — verified
      via a log line or a full-stack integration test, since
      `MockSerialSource` in `main.rs` isn't directly inspectable from an
      HTTP test without extending the demo wiring (acceptable to verify via
      `RUST_LOG=debug` observation for the manual demo step in task 8)

## 8. Demo + verification

- [ ] 8.1 Run the binary with `MOCK_SERIAL=true`, open
      `http://localhost:8080/target`, submit a new target temperature,
      confirm the form confirms the update and the dashboard's displayed
      target reflects the new persisted value on next poll
- [ ] 8.2 Confirm submitting a non-numeric or out-of-range value redisplays
      the form with an error and does not change the displayed target
- [ ] 8.3 Restart the process (with `MOCK_SERIAL=true` and Redis running)
      and confirm the target temperature set in 8.1 is restored on startup
      rather than reset to `DEFAULT_TARGET_TEMP`
- [ ] 8.4 Run `cargo fmt --check` and `cargo clippy --all-targets -- -D
      warnings` clean
- [ ] 8.5 Run the full `cargo test` suite (unit + `insta` snapshots + HTTP
      oneshot tests); review any new snapshots with `cargo insta review`
      before accepting
- [ ] 8.6 Run `openspec validate slice-4-set-target --strict`
