## 1. Config

- [x] 1.1 Write a config-parse test (RED) for `DEFAULT_TARGET_TEMP` (with a
      default, e.g. `19.5`) in `config.rs`
- [x] 1.2 Extend `Config` with `default_target_temp: f64` to pass the test
      (GREEN)

## 2. Target validation & reconcile (temperature-control) — TDD

- [x] 2.1 Write unit tests (RED, `rstest` cases) for a pure validation
      function, e.g. `validate_target(input: &str) -> Result<f64, String>`:
      valid numeric string within range accepted; non-numeric rejected;
      below-range rejected; above-range rejected; boundary values (exactly
      the min/max) accepted
- [x] 2.2 Implement `validate_target` (GREEN), with the allowed range as a
      named `const` (e.g. `TARGET_MIN: f64 = 0.0`, `TARGET_MAX: f64 = 40.0`)
- [x] 2.3 Write unit tests (RED) for a pure reconcile function, e.g.
      `target_to_write(desired: f64, reported: f64) -> Option<f64>`:
      returns `Some(desired)` when `desired != reported`; returns `None`
      when equal
- [x] 2.4 Implement `target_to_write` (GREEN)

## 3. Ingest loop reconcile wiring — TDD

- [x] 3.1 Write a test (RED) in `ingest.rs`: given a `MockSerialSource` whose
      scripted `Reading`s report a `target` different from a seeded
      `watch::Receiver<f64>` value, running `ingest_loop` results in
      `source.written_targets` containing the desired value
- [x] 3.2 Write a test (RED): when the `watch::Receiver`'s value matches the
      reading's reported `target`, `ingest_loop` results in no entries in
      `written_targets`
- [x] 3.3 Write a test (RED): when the watch value changes partway through
      (updated between two readings), the write occurs only for the
      reading(s) processed after the change — implemented as two sequential
      `ingest_loop` calls sharing one `watch` channel (see test), since
      `MockSerialSource`'s instant resolution gives no real interleaving
      point within a single call for the test driver to change the value
      mid-loop
- [x] 3.4 Extend `ingest_loop`'s signature to accept a
      `target_rx: &watch::Receiver<f64>`; after each successfully parsed
      reading, borrow the current value, compare via `target_to_write`, and
      call `source.write_target(..)` when `Some` (GREEN for 3.1–3.3)
- [x] 3.5 Confirm existing `ingest.rs` tests (slice-1/2) still pass with the
      new parameter (update call sites to pass a fixed/no-op
      `watch::channel` where reconcile isn't under test)

## 4. Persistence wiring (target_temp) — TDD

- [x] 4.1 Write a test (RED, using `FakeTimeStore`) confirming a helper that
      seeds the initial target — e.g. `initial_target(store, default) ->
      f64` — returns the persisted `ControllerState.target_temp` when
      `load_state()` returns `Some`
- [x] 4.2 Write a test (RED): the same helper returns the configured
      `default_target_temp` when `load_state()` returns `None`
- [x] 4.3 Implement the `initial_target` helper (GREEN for 4.1/4.2), placed
      in the new `temperature_control.rs` module (design.md's alternative
      module-naming choice) alongside `validate_target`/`target_to_write`,
      plus a `persist_target` helper wrapping `save_state` for task 4.4/4.5
      to call
- [x] 4.4 Write a test (RED) confirming that accepting a valid target (task
      2) triggers a `store.save_state` call with a `ControllerState`
      combining the new `target_temp` and the configured `brew_id`, using
      `FakeTimeStore` to assert the persisted value — covered by
      `persist_target_saves_target_temp_with_brew_id` in
      `temperature_control.rs`
- [x] 4.5 Wire the save-state call into the target-acceptance path (GREEN
      for 4.4) — the `POST /target` handler (task 5.10) calls
      `persist_target`

## 5. Web layer: AppState & handlers (web-dashboard, temperature-control) — TDD

- [x] 5.1 Extend `web::AppState` with `target_tx: watch::Sender<f64>` (plus a
      `store: Arc<dyn TimeStore>` handle, needed for the `/target` handler to
      persist accepted changes — not explicitly listed in this task but
      required for 4.4/4.5's wiring)
- [x] 5.2 Create `templates/target_form.html` (`{% extends "base.html" %}`):
      a form showing the current target temperature, an input field, a
      submit button, and an optional error message block
- [x] 5.3 Add a link/control from `dashboard.html` to `GET /target`
- [x] 5.4 Write an `insta` snapshot test (RED) rendering `target_form.html`
      with a seeded current-target context and no error
- [x] 5.5 Write an `insta` snapshot test (RED) rendering `target_form.html`
      with an error message present
- [x] 5.6 Implement the `/target` `GET` handler: builds a context from
      `AppState.target_tx.borrow()`'s current value, renders
      `target_form.html` (GREEN for 5.4/5.5)
- [x] 5.7 Write a `tower::oneshot` HTTP test (RED): `POST /target` with a
      valid numeric form value returns 200, the response confirms the
      update, and `AppState.target_tx.borrow()` reflects the new value
- [x] 5.8 Write a `tower::oneshot` HTTP test (RED): `POST /target` with a
      non-numeric value returns the redisplayed form with an error, and
      `target_tx`'s value is unchanged
- [x] 5.9 Write a `tower::oneshot` HTTP test (RED): `POST /target` with an
      out-of-range numeric value returns the redisplayed form with an error,
      and `target_tx`'s value is unchanged
- [x] 5.10 Implement the `/target` `POST` handler: parse the form value,
      call `validate_target` (task 2), on success update `target_tx` and
      call `store.save_state` (task 4), on failure re-render the form with
      the error — without touching `target_tx` or storage (GREEN for
      5.7–5.9). Follow-up from manual testing: the handler initially logged
      nothing on accepting a valid change, making a submitted target
      invisible in server logs until the next reconcile check (if any);
      added `info!(previous, target, "target temperature accepted")` on the
      success path for observability, matching the ingest loop's existing
      logging conventions.
- [x] 5.11 Wire `GET /target` and `POST /target` routes into `build_router`

## 6. Read-only guarantee update (web-dashboard)

- [x] 6.1 Update the existing "no mutating routes" test (from slice-3) so it
      asserts only against the dashboard (`/`) and status (`/status`) routes,
      not `/target` — confirm it still passes given the narrowed spec
      requirement (the test's path list already only covered `/`, `/status`,
      `/healthz`, never `/target`, so no change was needed; confirmed still
      green with the new route registered)
- [x] 6.2 Write a test (RED) confirming `POST /target` with a valid value
      returns a non-error status (i.e. mutating IS permitted on this route,
      the deliberate exception carved out by the modified web-dashboard
      spec) — GREEN once task 5's handler exists
      (`post_target_is_a_deliberate_exception_to_the_read_only_guarantee`)

## 7. Main wiring

- [x] 7.1 In `main.rs`, after building/rehydrating the store, call the
      `initial_target` helper (task 4) to compute the seed value, and
      construct `let (target_tx, target_rx) = watch::channel(initial_target)`
      (the store is now held as `Arc<dyn TimeStore>` so it can be shared
      between the ingest task and `AppState`)
- [x] 7.2 Pass `target_rx` into the ingest task's call to `ingest_loop`
      (task 3's new signature)
- [x] 7.3 Add `target_tx` (cloned/wrapped as needed) to the `AppState`
      constructed for the web router
- [x] 7.4 Confirm `MOCK_SERIAL=true` end-to-end: initially verified only via
      a full restart (persisted `21.0` differing from the mock's static
      `19.5` produced `target temperature needs updating — writing to
      device` on every reading). Manual testing then surfaced that a
      **live**, mid-process `POST /target` produced no visible reconcile
      log at all, because `main.rs`'s demo mock feed was a one-shot,
      4-line script that exhausted at startup — the ingest loop never ran
      again for the rest of the process's life. Fixed: `main.rs` now cycles
      the scripted feed indefinitely (fresh `MockSerialSource` per cycle,
      5s pause between cycles) so the ingest loop — and target reconcile —
      keeps running for the process's lifetime. Re-verified live (no
      restart): `POST /target` with `22.0` immediately logged `target
      temperature accepted previous=19.5 target=22.0`, and the very next
      mock cycle logged `target temperature needs updating — writing to
      device desired=22.0 reported=19.5` (repeating every cycle thereafter,
      since the static script's `reading.target` never itself changes to
      `22.0` — `MockSerialSource` doesn't simulate a device that updates
      its own reported state in response to a write; only slice-6's real
      `arduino.rs` transport will).

## 8. Demo + verification

- [x] 8.1 Ran the binary with `MOCK_SERIAL=true` against a real `redis:8`
      Docker container, hit `GET /target` and `POST /target` via `curl`
      (equivalent to a browser form submission): submitting
      `target=21.0` returned "Target temperature updated to 21.0°." and a
      subsequent `GET /target` reflected `value="21.0"`. (The dashboard's
      `/status`/`/` display the latest *reading's* self-reported `target`
      field, which stays at the mock feed's static `19.5` until a reading
      with a different reported value arrives — expected per design.md
      decision 2, not a bug.)
- [x] 8.2 Confirmed: `POST /target` with `target=abc` returned
      "'abc' is not a valid number"; with `target=100` returned "target must
      be between 0 and 40, got 100"; a following `GET /target` still showed
      `value="21.0"` (unchanged) after both
- [x] 8.3 Killed and restarted the process against the same Redis container:
      `GET /target` on the fresh process showed `value="21.0"` (the
      previously persisted value), not the `19.5` `DEFAULT_TARGET_TEMP` —
      confirmed via log line `current state rehydrated from storage` is a
      separate (reading-history) log; the target restoration itself was
      confirmed via the `GET /target` response and the reconcile log lines
      described in 7.4
- [x] 8.4 Ran `cargo fmt --check` and `cargo clippy --all-targets -- -D
      warnings` — clean (one `cargo fmt` pass was needed to reformat a few
      lines before `--check` passed)
- [x] 8.5 Ran the full `cargo test` suite: 66 unit tests (incl. 4 new `insta`
      snapshots and 9 new HTTP oneshot tests) + 6 `testcontainers`-backed
      Redis integration tests, all passing. `cargo-insta` CLI wasn't
      installed, so the two new snapshots' `.snap.new` files were reviewed
      manually and accepted (renamed, stripped the `assertion_line`
      metadata field to match the existing snapshot format) rather than via
      `cargo insta review`
- [x] 8.6 Ran `openspec validate slice-4-set-target --strict` — valid
