## 1. Brew validation & persistence helpers (brew-session) — TDD

- [ ] 1.1 Create `src/brew_session.rs` module (mirrors
      `temperature_control.rs`'s shape per design.md decision 5); register it
      in `src/lib.rs`
- [ ] 1.2 Write unit tests (RED, `rstest` cases) for a pure validation
      function `validate_brew_id(input: &str) -> Result<String, String>`:
      non-empty input accepted and trimmed (e.g. `"  01-IPA-v02  "` ->
      `"01-IPA-v02"`); empty string rejected; whitespace-only string rejected
- [ ] 1.3 Implement `validate_brew_id` (GREEN for 1.2)
- [ ] 1.4 Write a test (RED, using `FakeTimeStore`) for
      `initial_brew_id(store, default) -> String`: returns the persisted
      `ControllerState.brew_id` when `load_state()` returns `Some`
- [ ] 1.5 Write a test (RED): `initial_brew_id` returns the passed-in
      `default` when `load_state()` returns `None`
- [ ] 1.6 Implement `initial_brew_id` (GREEN for 1.4/1.5)
- [ ] 1.7 Write a test (RED) confirming `persist_brew(store, brew_id,
      current_target_temp)` calls `store.save_state` with a
      `ControllerState` combining the new `brew_id` and the passed-in
      `current_target_temp`, asserted via `FakeTimeStore`
- [ ] 1.8 Implement `persist_brew` (GREEN for 1.7)

## 2. Ingest loop: dynamic brew tagging — TDD

- [ ] 2.1 Write a test (RED) in `ingest.rs`: given a `MockSerialSource` with
      scripted readings and a seeded `watch::Receiver<String>`, running
      `ingest_loop` results in the store's `write_reading` being called with
      the watch channel's brew identifier (assert via
      `FakeTimeStore::last_reading` under that identifier)
- [ ] 2.2 Write a test (RED): when the brew identifier changes between two
      sequential `ingest_loop` calls sharing one `watch` channel (matching
      the existing target-reconcile test pattern at
      `reconcile_reflects_the_desired_target_updated_between_reading_batches`),
      the reading processed after the change is persisted under the new
      identifier while the reading processed before it stays under the old
      one
- [ ] 2.3 Change `ingest_loop`'s signature: replace `brew_id: &str` with
      `brew_rx: &watch::Receiver<String>`; inside the loop, clone the
      current value (`brew_rx.borrow().clone()`) before the awaited
      `store.write_reading` call, per design.md decision 3 (GREEN for
      2.1/2.2)
- [ ] 2.4 Update all existing `ingest.rs` call sites (slice-1–4 tests) to
      pass a fixed `watch::channel("test-brew".to_string())`'s receiver
      instead of a `&str`, confirming they still pass unchanged otherwise

## 3. Web layer: AppState & handlers (web-dashboard, brew-session) — TDD

- [ ] 3.1 Replace `web::AppState.brew_id: String` with
      `brew_tx: watch::Sender<String>` (design.md decision 1); update the
      test-only `test_state`/`test_state_with_target` helpers in
      `web/mod.rs` to construct and seed a `watch::channel` for brew id
      (parallel to the existing `target_tx` seeding)
- [ ] 3.2 Update `StatusContext`/`status_context` (`web/handlers.rs`) to read
      `state.brew_tx.borrow().clone()` instead of the removed `brew_id`
      field
- [ ] 3.3 Create `templates/brew_form.html` (`{% extends "base.html" %}`),
      structurally mirroring `target_form.html`: current brew identifier
      pre-filled, a submit button, and an optional error message block
- [ ] 3.4 Add a link/control from `dashboard.html` to `GET /brew` (alongside
      the existing `/target` link)
- [ ] 3.5 Write an `insta` snapshot test (RED) rendering `brew_form.html`
      with a seeded current-brew context and no error
- [ ] 3.6 Write an `insta` snapshot test (RED) rendering `brew_form.html`
      with an error message present
- [ ] 3.7 Implement the `/brew` `GET` handler (`brew_form`): builds a
      context from `AppState.brew_tx.borrow()`'s current value, renders
      `brew_form.html` (GREEN for 3.5/3.6)
- [ ] 3.8 Write a `tower::oneshot` HTTP test (RED): `POST /brew` with a
      valid non-empty value returns 200, the response confirms the update,
      `AppState.brew_tx.borrow()` reflects the new value, and subsequent
      `/status`/`/` responses reflect the new brew identifier
- [ ] 3.9 Write a `tower::oneshot` HTTP test (RED): `POST /brew` with an
      empty value returns the redisplayed form with an error, and
      `brew_tx`'s value is unchanged
- [ ] 3.10 Write a `tower::oneshot` HTTP test (RED): `POST /brew` with a
      whitespace-only value returns the redisplayed form with an error, and
      `brew_tx`'s value is unchanged
- [ ] 3.11 Write a `tower::oneshot` HTTP test (RED) for design.md decision 4
      (rehydration on switch): seed `FakeTimeStore` with a persisted reading
      under a brew identifier different from the current one; `POST /brew`
      switching to that identifier results in a subsequent `GET /status`
      immediately reflecting the persisted reading, without any new reading
      having been ingested
- [ ] 3.12 Write a `tower::oneshot` HTTP test (RED): `POST /brew` switching
      to a brew identifier with no persisted history results in a subsequent
      `GET /status` immediately showing the no-data state, even if a reading
      was previously displayed for the prior brew
- [ ] 3.13 Implement the `/brew` `POST` handler (`set_brew`): parse the form
      value, call `validate_brew_id` (task 1), on success update `brew_tx`,
      call `persist_brew` sourcing `current_target_temp` from
      `*state.target_tx.borrow()` (design.md decision 2), call
      `ingest::rehydrate_latest_reading` for the new identifier and
      overwrite `state.latest` (design.md decision 4), then confirm; on
      failure re-render the form with the error without touching `brew_tx`,
      storage, or `state.latest` (GREEN for 3.8–3.12)
- [ ] 3.14 Wire `GET /brew` and `POST /brew` routes into `build_router`

## 4. Persistence symmetry for `/target` (resolves slice-4 open question)

- [ ] 4.1 Write a test (RED) confirming `set_target`'s handler now persists
      `ControllerState { target_temp: <new value>, brew_id:
      state.brew_tx.borrow().clone() }` — i.e. sources the paired `brew_id`
      from the live `brew_tx` channel rather than a removed static field
- [ ] 4.2 Update `web/handlers.rs`'s `set_target` handler's call to
      `persist_target` to pass `&state.brew_tx.borrow().clone()` in place of
      the removed `&state.brew_id` (GREEN for 4.1)
- [ ] 4.3 Confirm existing `/target` tests (slice-4) still pass unchanged
      given the new `AppState` shape

## 5. Main wiring

- [ ] 5.1 In `main.rs`, after building the store, call
      `brew_session::initial_brew_id` (task 1) to compute the seed brew
      identifier, and construct
      `let (brew_tx, brew_rx) = watch::channel(initial_brew_id)`
- [ ] 5.2 Update the startup call to `ingest::rehydrate_latest_reading` to
      use the resolved `initial_brew_id` instead of `config.default_brew_id`
      directly (design.md decision 5's correctness fix)
- [ ] 5.3 Pass `brew_rx` into the ingest task's call to `ingest_loop` (task
      2's new signature), replacing the `&config.default_brew_id` argument
- [ ] 5.4 Add `brew_tx` (cloned/wrapped as needed) to the `AppState`
      constructed for the web router, replacing the removed `brew_id` field
- [ ] 5.5 Confirm `MOCK_SERIAL=true` end-to-end: submit a new brew id via
      `POST /brew` mid-run and confirm the next ingested mock reading is
      persisted under the new identifier (verified by inspecting Redis keys
      or via a subsequent `last_reading` lookup)

## 6. Demo + verification

- [ ] 6.1 Run the binary with `MOCK_SERIAL=true` against a real `redis:8`
      Docker container; hit `GET /brew` and `POST /brew` via `curl`:
      submitting a new brew id returns a confirmation and a subsequent
      `GET /brew` reflects the new value
- [ ] 6.2 Confirm: `POST /brew` with an empty or whitespace-only value
      returns an error message; a following `GET /brew` still shows the
      previous value unchanged
- [ ] 6.3 Confirm rehydration: with the process running and a reading
      already displayed on the dashboard, `POST /brew` to a fresh
      (never-used) identifier and confirm `GET /status` immediately shows
      the no-data state; `POST /brew` back to the original identifier and
      confirm `GET /status` immediately shows that brew's last known
      reading again, without waiting for a new mock cycle
- [ ] 6.4 Kill and restart the process against the same Redis container:
      confirm `GET /brew` on the fresh process shows the most recently set
      brew identifier (not `DEFAULT_BREW_ID`), and the dashboard rehydrates
      that brew's last persisted reading on startup
- [ ] 6.5 Run `cargo fmt --check` and `cargo clippy --all-targets -- -D
      warnings` — confirm clean
- [ ] 6.6 Run the full `cargo test` suite (unit + `insta` snapshots +
      `testcontainers`-backed Redis integration tests) — confirm all passing;
      review and accept any new `.snap.new` snapshot files
- [ ] 6.7 Run `openspec validate slice-5-brew-session --strict` — confirm
      valid
