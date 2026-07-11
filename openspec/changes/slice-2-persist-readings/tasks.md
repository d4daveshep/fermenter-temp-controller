## 1. Configuration (system-configuration) — TDD

- [ ] 1.1 Write config-parse tests (RED): `REDIS_URL`, `TS_RETENTION_DAYS`
      (with default), `DEFAULT_BREW_ID` (with default) parsed into `Config`;
      invalid retention value yields an error
- [ ] 1.2 Extend `Config` with `redis_url`, `ts_retention_days`,
      `default_brew_id` fields to pass the tests (GREEN)
- [ ] 1.3 Confirm existing config tests still pass (no regressions on
      serial/log fields)

## 2. Dependencies

- [ ] 2.1 Add `redis` (async, `tokio-comp` feature) to `Cargo.toml`
- [ ] 2.2 Add dev-dependency `testcontainers` (+ redis module)

## 3. `TimeStore` trait + fakes (reading-history) — TDD

- [ ] 3.1 Define the `TimeStore` async trait in `store/mod.rs`:
      `write_reading`, `last_reading`, `save_state`, `load_state`
- [ ] 3.2 Define a minimal `ControllerState { target_temp, brew_id }` in
      `model.rs`
- [ ] 3.3 Write `FakeTimeStore` (in-memory) test double: records written
      readings per brew, returns the last one, round-trips saved state
- [ ] 3.4 Write `FakeTimeStore` unit tests (RED then GREEN): write→last
      roundtrip; last-reading on empty brew returns `None`; save/load state
      roundtrip; load with no prior save returns `None`

## 4. Redis Time Series implementation (reading-history) — TDD, testcontainers

- [ ] 4.1 Write `tests/store_redis.rs` (RED) using `testcontainers` to spin up
      `redis:8`: write→last roundtrip for a reading
- [ ] 4.2 Add cases: idempotent `TS.CREATE` on repeated writes (no error on
      already-exists); configured retention period is applied at creation
- [ ] 4.3 Add cases: save/load controller state roundtrip; `load_state`
      returns `None` on an empty database
- [ ] 4.4 Implement `store/redis.rs`: `redis::aio::ConnectionManager`-backed
      client; raw `TS.CREATE`/`TS.ADD`/`TS.GET` commands per
      `temp:{brew_id}:{field}` series (`fermenter` from `average`, `ambient`,
      `target`); `controller:state` hash for save/load (GREEN)
- [ ] 4.5 Refactor; confirm all `store_redis.rs` cases pass against real Redis

## 5. Write-through wiring (reading-history + temperature-monitoring)

- [ ] 5.1 Write a test (RED) that runs the ingest loop against a
      `MockSerialSource` + `FakeTimeStore`, asserting each valid reading is
      passed to `write_reading` tagged with the configured brew id
- [ ] 5.2 Implement: ingest loop writes each successfully parsed `Reading` to
      the `TimeStore` immediately after updating in-memory state (GREEN)
- [ ] 5.3 Write a test (RED): a `TimeStore` write failure is logged and the
      ingest loop continues to the next line rather than stopping
- [ ] 5.4 Implement: wrap `write_reading` calls so an `Err` is logged at
      `warn` and does not propagate/crash the loop (GREEN)

## 6. Startup rehydration (temperature-monitoring) — TDD

- [ ] 6.1 Write a test (RED): given a `FakeTimeStore` pre-populated with a
      persisted reading for the configured brew id, initialising current
      state loads that reading instead of starting empty
- [ ] 6.2 Write a test (RED): given a `FakeTimeStore` with no persisted data,
      current state starts as `None`, as in slice-1
- [ ] 6.3 Implement the rehydration step in `main.rs`: after building the
      store, call `last_reading(brew_id)` and seed `latest_reading` with the
      result before starting the ingest loop (GREEN)

## 7. Main wiring

- [ ] 7.1 Wire `main`: build the Redis `TimeStore` from config (`REDIS_URL`)
      alongside the existing mock/serial source selection
- [ ] 7.2 Rehydrate state (task 6.3) before spawning the ingest loop
- [ ] 7.3 Pass the `TimeStore` and configured `default_brew_id` into the
      ingest loop for write-through (task 5.2)
- [ ] 7.4 Confirm Redis connectivity issues at startup do not block the
      process from starting (only config *values* fail fast; connection
      resilience is runtime, per design.md decision 5)

## 8. Demo + verification

- [ ] 8.1 Run `docker run --rm -p 6379:6379 redis:8` locally (or via
      `docker-compose`), then run the binary with `MOCK_SERIAL=true` and
      confirm readings land in Redis (`redis-cli TS.RANGE
      temp:00-TEST-v00:fermenter - +`)
- [ ] 8.2 Restart the binary and confirm current state rehydrates from Redis
      instead of starting empty
- [ ] 8.3 Run `cargo fmt --check` and `cargo clippy --all-targets -- -D
      warnings` clean
- [ ] 8.4 Confirm full `cargo test` suite passes, including
      `tests/store_redis.rs` (requires Docker)
- [ ] 8.5 Run `openspec validate slice-2-persist-readings --strict`
