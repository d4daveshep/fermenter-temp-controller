## Why

Slice-1 only ever holds the single latest reading in memory — a restart loses
all history and even the last known state. The rewrite needs durable
time-series storage (replacing InfluxDB) so readings survive restarts and a
history exists for later querying, matching the old system's core value
without its silent-swallow failure modes. This slice is demoable: readings
flow in, land in Redis, and the latest state survives a process restart.

## What Changes

- Introduce a `TimeStore` trait abstracting time-series persistence, with a
  Redis 8 Time Series implementation (`redis` crate, raw `TS.*` commands — no
  first-class typed API exists).
- On every successfully parsed reading, write it to the store tagged by the
  current brew identifier, in addition to updating in-memory state.
- Persist minimal controller state (`target_temp`, `brew_id`) as a Redis hash
  so it can be reloaded on startup.
- On startup, rehydrate current state from the store when prior data exists:
  load the last persisted controller state and the most recent reading for
  that brew, instead of starting from an empty/`None` state.
- Extend the configuration surface with `REDIS_URL`, `TS_RETENTION_DAYS`, and
  `DEFAULT_BREW_ID`, parsed and validated the same fail-fast way as the
  existing serial/log config.
- Implement Redis connection resilience now (unlike the serial transport,
  where reconnect is deferred to slice-6): auto-reconnect with backoff; a
  failed write is logged and skipped, never crashing the ingest loop.
- Time-series creation is idempotent and retention-bounded: series are
  created lazily on first write with a configured retention period, and
  re-creation attempts on an existing series are tolerated, not treated as
  errors.

Out of scope for this slice: the web UI, writing target-temp changes back to
the device (`temperature-control`), runtime brew-id relabeling
(`brew-session`), and Docker packaging. Those arrive in later slices. The
brew identifier is a static, config-supplied value this slice — it does not
yet change at runtime.

## Capabilities

### New Capabilities

- `reading-history`: Time-series persistence of readings and controller state
  to Redis — write-through on ingest, retention-bounded series creation,
  most-recent-reading lookup, save/load of controller state, and Redis
  connection resilience (reconnect + backoff).

### Modified Capabilities

- `temperature-monitoring`: the "Expose latest reading as current state"
  requirement is amended so that, on startup, current state is rehydrated
  from persisted storage when prior data exists, rather than always starting
  from `None`.
- `system-configuration`: the "Load configuration from environment"
  requirement is amended to add the Redis URL, time-series retention period,
  and default brew identifier to the set of configuration values loaded and
  validated at startup.

## Impact

- **New module**: `store` (trait `TimeStore` + `store/redis.rs` implementation
  using raw `TS.*` commands via the `redis` crate).
- **Modified modules**: `config.rs` (new fields + validation), `ingest.rs`
  (write-through to the store per reading), `main.rs` (build the store,
  rehydrate state on startup).
- **Dependencies introduced**: `redis` (async, `tokio-comp` feature); dev:
  `testcontainers` (+ redis module) for integration tests against a real
  `redis:8`.
- **No impact** on the existing Python system, which remains the deployable
  stack on `master` until cutover.
- **Contract dependency**: the Redis key/series schema introduced here
  (`temp:{brew_id}:{field}`, `controller:state` hash) becomes bedrock for
  `temperature-control` (slice-4) and `brew-session` (slice-5), which will
  read and extend it rather than redefine it.
