## Context

Slice-1 established the ingest loop, the `Reading` model, config, and typed
errors, but only ever holds the single latest reading in an
`Arc<Mutex<Option<Reading>>>` — nothing survives a restart. The old Python
system wrote every reading to InfluxDB; this slice replaces that with Redis 8
Time Series (built in, no separate `RedisTimeSeries` module needed) and adds
the persisted-state rehydration the old system never had cleanly (its state
lived only in the running process).

This slice deliberately fixes the Redis key/series schema and the
`TimeStore` trait shape now, since `temperature-control` (slice-4) and
`brew-session` (slice-5) will extend the same persisted-state hash rather
than redefine it.

## Goals / Non-Goals

**Goals:**

- A `TimeStore` trait with a Redis Time Series implementation: write a
  reading, fetch the most recent reading for a brew, and save/load minimal
  controller state (`target_temp`, `brew_id`).
- Write-through: every successfully parsed reading is persisted, tagged by
  the current brew identifier, immediately after in-memory state updates.
- Rehydration: on startup, load persisted controller state and the most
  recent reading (if any) so current state isn't `None` after a restart.
- Idempotent, retention-bounded series creation (`TS.CREATE ... RETENTION`,
  tolerating "already exists").
- Redis connection resilience implemented now (reconnect + backoff via a
  connection-manager style client) — unlike serial, which defers this to
  slice-6, because M4 explicitly calls for it and Redis client libraries make
  it comparatively cheap.
- Config gains `REDIS_URL`, `TS_RETENTION_DAYS`, `DEFAULT_BREW_ID`, validated
  the same fail-fast way as existing fields.

**Non-Goals:**

- Any web surface (`web-dashboard`, slice-3).
- Writing target-temp changes back to the Arduino, or reconciling against a
  device-reported target (`temperature-control`, slice-4).
- Runtime brew-id relabeling (`brew-session`, slice-5) — the brew id is a
  static, config-supplied value this slice.
- Range/history queries beyond "most recent" (no query API is exposed yet;
  only what rehydration needs).
- Docker packaging (slice-7).

## Decisions

**1. `TimeStore` as an async trait, minimal surface.**
`async fn write_reading(&self, brew_id: &str, r: &Reading) -> Result<()>`,
`async fn last_reading(&self, brew_id: &str) -> Result<Option<Reading>>`,
`async fn save_state(&self, s: &ControllerState) -> Result<()>`,
`async fn load_state(&self) -> Result<Option<ControllerState>>`. Matches
`rewrite-plan.md` §4. Rationale: mirrors the `SerialSource` pattern from
slice-1 — trait-abstract the boundary so ingest-loop tests use a
`FakeTimeStore` and need no Docker.

**2. Persisted `ControllerState` carries only `target_temp` + `brew_id`, not
`latest`.**
`rewrite-plan.md` §4 sketches `ControllerState { target_temp, brew_id,
latest }`, but persisting `latest` redundantly duplicates data already in the
time series. This slice's persisted hash (`controller:state`) holds only the
two fields that don't come from the Arduino; rehydration reconstructs
`latest` via `last_reading(brew_id)` instead. Alternative (persist all three
fields verbatim) rejected — redundant writes, and stale-`latest` risk if a
crash happens between writing the series point and the state hash.

**3. Raw `TS.*` commands, one series per field.**
The `redis` crate has no first-class Time Series API, so `TS.CREATE` /
`TS.ADD` / `TS.GET` are issued as raw commands. One series per
`(brew_id, field)` pair — `temp:{brew_id}:fermenter`, `temp:{brew_id}:ambient`,
`temp:{brew_id}:target` — labeled `brew={brew_id}`, `field={name}`, matching
`rewrite-plan.md` §8. Rationale: matches the documented schema so later
slices (querying history, charts) build on a stable layout; per-field series
keep `TS.RANGE` queries simple later.

**4. Lazy, idempotent series creation.**
On the first write for a `(brew_id, field)` pair, attempt `TS.CREATE ...
RETENTION <ms> LABELS ...`; a "key already exists" error is treated as
success, not a failure. Rationale: avoids a separate provisioning step and
supports brew-id changes (future slice) without pre-declaring series.

**5. Redis client: connection-manager style, reconnect + backoff now.**
Use `redis::aio::ConnectionManager` (or equivalent), which transparently
reconnects on transient errors. A failed write is logged at `warn` and
skipped — it must never crash the ingest loop, mirroring how slice-1 treats
malformed serial lines. Rationale: `rewrite-plan.md` §10 and §12.7 (M4) call
for reconnect to land with the store, unlike serial (M6). Startup does *not*
fail-fast on Redis being unreachable — only config *values* are fail-fast;
connectivity is handled by the resilience layer, consistent with treating
Redis like the serial device (external, can be down transiently).

**6. Config additions validated like existing fields.**
`REDIS_URL` (required, e.g. `redis://localhost:6379`), `TS_RETENTION_DAYS`
(default `7`, matching the old InfluxDB retention), `DEFAULT_BREW_ID`
(default `"00-TEST-v00"`, matching the old system's default). Parsed by the
same `envy`-based `Config::from_env()`; no new validation requirement beyond
"Fail fast on invalid configuration", which already covers any
missing/unparseable required value generically.

**7. Testing: `FakeTimeStore` for unit/ingest tests, `testcontainers` for
real-Redis integration.**
Unit tests (ingest loop, rehydration logic) run against an in-memory
`FakeTimeStore` — fast, no Docker. `tests/store_redis.rs` spins up a real
`redis:8` via `testcontainers` to verify actual `TS.*` roundtrips: write→last,
idempotent `TS.CREATE`, retention label applied, save/load state roundtrip,
and `load_state` returning `None` on an empty DB. Matches
`rewrite-plan.md` §12.3–§12.5.

## Risks / Trade-offs

- **[testcontainers requires Docker on the dev/test/CI host]** → Unit-level
  behaviour (ingest write-through, rehydration branching) is fully covered by
  `FakeTimeStore` without Docker; only the `store_redis.rs` integration
  suite needs it, matching `rewrite-plan.md` §12.6's CI assumption that
  Docker is available on the runner.
- **[Raw `TS.*` command strings are not type-checked]** → Mitigated by the
  `store_redis.rs` integration tests asserting real roundtrips against
  `redis:8`, not just mocked command construction.
- **[Persisted-state schema (`controller:state` hash, per-field series naming)
  becomes a dependency for slice-4/slice-5]** → Documented explicitly here and
  in the proposal's Impact section so those slices amend, not redefine, the
  schema.
- **[Write-through failures could mask real Redis outages if only logged]** →
  Accepted trade-off for this slice: the ingest loop's job is to keep reading
  the serial port regardless of storage health, matching the "never crash the
  loop" philosophy from slice-1. `/healthz` reporting Redis status is
  deferred to slice-3, where it becomes externally observable.
- **[Default brew id is static this slice]** → No risk to correctness (it's
  just a config value), but flagged as a Non-Goal so it isn't mistaken for
  `brew-session` functionality landing early.
