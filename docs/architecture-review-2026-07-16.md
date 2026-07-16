# Architecture Review — 2026-07-16

> Written as a learning guide for a Rust beginner working on this codebase.
> Scope: `fermenter/` (the Rust binary) only. Snapshot in time — line
> numbers/counts will drift as the code evolves; re-review periodically
> rather than treating this as permanently accurate.

## Bottom line

**The architecture is not overly complicated.** It's a small, conventional
Rust app: ~1,500 lines of production logic (out of ~3,290 total — the rest
is tests), 2 traits, 1 error enum, no actor framework, no `tokio::spawn`,
no macros beyond derives. The two things that will feel like "real Rust"
hurdles are `dyn Trait` + `#[async_trait]`, and the `tokio::join!`
(no-`spawn`) concurrency style — both covered below, and both worth
deliberately studying rather than trying to avoid.

## 1. Module structure

```
fermenter/src/
├── main.rs                  (184 ln)  binary entry point — wiring only
├── lib.rs                     (9 ln)  flat `pub mod` re-exports, no indirection
├── model.rs                  (75 ln)  Reading (Arduino JSON) + ControllerState
├── error.rs                  (40 ln)  single AppError enum (thiserror) + Axum IntoResponse
├── config.rs                (246 ln)  Config + env loading (~150 ln are tests)
├── ingest.rs                (499 ln)  serial→state→store pipeline (~350 ln are tests)
├── temperature_control.rs   (149 ln)  pure target-reconcile logic (~75 ln tests)
├── brew_session.rs          (110 ln)  pure brew-id validation/persist logic (~55 ln tests)
├── serial/
│   ├── mod.rs                (20 ln)  SerialSource trait (2 methods)
│   ├── arduino.rs           (208 ln)  real tokio-serial impl, backoff reconnect
│   └── mock.rs                (91 ln)  scripted test double
├── store/
│   ├── mod.rs                 (25 ln)  TimeStore trait (4 methods)
│   ├── redis.rs              (163 ln)  Redis Time Series impl (raw TS.* commands)
│   └── fake.rs                (109 ln)  in-memory test double, #[cfg(test)] only
└── web/
    ├── mod.rs                (614 ln)  AppState, router, build_environment (~530 ln tests)
    ├── handlers.rs           (381 ln)  7 Axum handlers (~170 ln tests/snapshots)
    └── render.rs               (18 ln)  one helper: MiniJinja render → Html<String>

fermenter/tests/
├── store_redis.rs           (166 ln)  testcontainers-backed Redis integration tests
└── serial_hardware.rs       (183 ln)  #[ignore]'d hardware-in-the-loop tests
```

No circular dependencies — clean DAG: `model`/`error` are leaves →
`config`/`serial`/`store` depend on those → `ingest`/`temperature_control`/
`brew_session` depend on those → `web` depends on everything → `main.rs`
wires it all together.

## 2. Data flow (serial → state → Redis → web)

1. **Source**: `SerialSource::read_line()` (`serial/mod.rs:15`) returns a raw
   line. Two impls: `ArduinoSerialSource` (real, `tokio-serial`) and
   `MockSerialSource` (scripted, for dev/test without hardware).
2. **Parse**: `ingest::ingest_loop` (`ingest.rs:57`) does
   `serde_json::from_str::<Reading>(&line)`. `Reading` (`model.rs:4`) mirrors
   the Arduino's fixed JSON contract.
3. **In-memory state**: on success, the loop locks
   `Arc<Mutex<Option<Reading>>>` (`ingest.rs:83`) and replaces its contents.
   Created once in `main.rs:80`, cloned into both the ingest loop and
   `AppState.latest` (`web/mod.rs:25`) — **shared mutex, not a channel**.
4. **Persistence**: same loop reads the current brew id off a
   `watch::Receiver<String>` (`ingest.rs:87`), then
   `store.write_reading(&brew_id, &reading)` — `TimeStore` trait
   (`store/mod.rs`), implemented by `RedisTimeStore` (`store/redis.rs:91`),
   which issues 3× `TS.ADD` (per-field series) plus a `SET` of the full JSON
   blob for lossless rehydration.
5. **Web UI**: Axum handlers (`web/handlers.rs`) read the same
   `Arc<Mutex<Option<Reading>>>` and pass it into a MiniJinja template
   context. HTMX polls `/status` every 5s to re-render a fragment.

Two `tokio::sync::watch` channels carry input the other direction:
- `target_tx`/`target_rx: watch<f64>` — web `/target` POST → ingest loop
  compares to the device-reported target → writes back to the Arduino if
  different.
- `brew_tx`/`brew_rx: watch<String>` — web `/brew` POST → tags subsequently
  persisted readings.

There is **no actor pattern and no mpsc channel** — the original plan
(`docs/rewrite-plan.md`) sketched a `SerialTask → mpsc → Ingest/State` split;
the actual implementation collapsed that into one function sharing a mutex
directly. Simpler than planned, not more complex.

## 3. Concurrency model

**The single most important thing to internalize**: there are **zero calls
to `tokio::spawn`** anywhere in the app. Everything is two futures polled
concurrently within the one `main` task, via `tokio::join!` (`main.rs:179`):

```rust
let ingest_future = async { ingest::ingest_loop(...).await };
let server_future = async { axum::serve(listener, router).await };
let (_, server_result) = tokio::join!(ingest_future, server_future);
```

So the concurrency vocabulary to learn is small:
- `Arc<Mutex<Option<Reading>>>` (std mutex, not tokio) — only ever locked for
  a short, non-`.await`-spanning critical section.
- `watch::channel` ×2 — "latest value wins" semantics. Note: `send()` on a
  channel with no live receivers silently no-ops — the codebase itself
  documents having been bitten by this (`web/mod.rs` test comments).
- `AtomicBool` (`ingest_alive`, `main.rs:91`) — trivial liveness flag for
  `/healthz`.
- `Arc<dyn TimeStore>` — shared trait-object handle cloned into both the
  ingest loop and `AppState`.

Axum spawns a task per HTTP connection internally — that's framework
plumbing, not something this app's code manages.

## 4. Web layer

- **Routing** (`web/mod.rs`, `build_router`): flat, 6 routes, no nested
  routers, only `tower_http::ServeDir` for `/static`.
- **Handlers** (`web/handlers.rs`): plain `async fn(State<AppState>, ...) ->
  Result<...>` using Axum's `State`/`Form`/`Json` extractors — idiomatic,
  nothing exotic.
- **Templating**: `render()` (`web/render.rs:10`) is a 15-line helper —
  `impl Serialize` context param, looks up a template by name, maps
  MiniJinja errors into `AppError::Render`.
- **`AppState`** (`web/mod.rs:24`): `#[derive(Clone)]` struct of
  `Arc`s/channel-halves — the standard Axum shared-state pattern.

Things a beginner will trip on here:
- `impl Serialize` as an argument type (`render.rs:10`) — an implicit
  generic, easy to miss as "this is generic."
- `Arc<dyn TimeStore>` inside `AppState` — needs "trait objects" explained
  once.
- Two `#[cfg(feature = "embed")]` branches in `build_environment`
  (`web/mod.rs`) for dev-mode disk templates vs. release-mode embedded
  templates — conditional compilation is not an intuitive idiom coming from
  most other languages.

## 5. Store layer

`TimeStore` (`store/mod.rs`) is a 4-method `#[async_trait]` trait:
`write_reading`, `last_reading`, `save_state`, `load_state`. Two impls:
- `RedisTimeStore` (`store/redis.rs`) — raw `redis::cmd("TS.CREATE"/"TS.ADD")`
  string commands (the `redis` crate has no typed Time Series API), plus
  plain `SET`/`GET`/`HSET` for the "latest reading" cache and controller
  state. Uses a lazy `ConnectionManager` so a Redis outage at boot doesn't
  block startup — only genuinely malformed config (a bad URL) is fail-fast.
- `FakeTimeStore` (`store/fake.rs`) — `HashMap`+`Mutex` test double, compiled
  only under `#[cfg(test)]`.

This is a textbook "port and adapters" pattern, small and not
over-engineered — but it is the one deliberate abstraction-for-testability
tradeoff in the app (see §7).

## 6. Error handling

One enum, `AppError` (`error.rs:8`, `thiserror`):
```rust
enum AppError { Parse(#[from] serde_json::Error), Serial(String), Config(String), Store(String), Render(String) }
```
Used consistently as `pub type Result<T> = std::result::Result<T, AppError>`
everywhere; implements Axum's `IntoResponse` so handlers just return
`Result<Html<String>>`. Non-fatal I/O errors (serial hiccup, a failed Redis
write) are logged via `tracing::warn!` and the loop continues — a
deliberate, documented design choice (comments cite specific "design.md
decision" numbers), not accidental inconsistency. Only config-parse failures
at startup are fail-fast.

**Update (this review)**: `anyhow` was listed as a dependency in
`Cargo.toml` and in `openspec/config.yaml`'s tech-stack blurb ("anyhow at
top-level wiring") but was never actually used anywhere in `src/` — it's
been removed as dead weight. `AppError`/`thiserror` is the sole error
mechanism end-to-end now, which is simpler to reason about than "two error
philosophies, one of them unused."

## 7. Testing

Idiomatic 3-tier pyramid, matching what was originally planned:
- Inline `#[cfg(test)] mod tests` in nearly every file — majority of several
  files' line counts (config.rs, ingest.rs, web/mod.rs are all >60% test).
- `rstest` for table-driven cases (e.g. valid/invalid target values).
- `tests/store_redis.rs` — real `redis:8` via `testcontainers` (needs
  Docker on the host).
- `tests/serial_hardware.rs` — `#[ignore]`'d, needs a physically attached
  Arduino, guarded by a `tokio::sync::Mutex` to serialize hardware access.
- `insta` snapshot tests for rendered HTML fragments
  (`src/web/snapshots/*.snap`).
- Router-level tests use `tower::ServiceExt::oneshot` to drive the full
  Axum `Router` without binding a real socket.

## 8. Advanced Rust concepts actually present (study these deliberately)

Ranked by how much they'll trip up a beginner:

1. **`#[async_trait]` on `SerialSource`/`TimeStore`** (`serial/mod.rs`,
   `store/mod.rs`) — needed because plain `async fn` in traits isn't fully
   object-safe for `dyn Trait` usage. Without it, `Arc<dyn TimeStore>` /
   `Box<dyn SerialSource>` wouldn't compile. This is genuinely the most
   "advanced Rust" concept in the app — but it's applied in exactly one
   shape everywhere, so learning it once covers the whole codebase's
   abstraction style.
2. **`dyn Trait` objects** for the two I/O boundaries — this *is* the app's
   dependency-injection mechanism (real Arduino/Redis in production,
   `MockSerialSource`/`FakeTimeStore` in tests).
3. **`impl Serialize` as an argument type** (`web/render.rs:10`) — implicit
   generics.
4. **`watch::channel` semantics** — "latest wins," and a silent no-op `send`
   when no receivers remain.
5. **Conditional compilation**, three flavors: `#[cfg(feature = "embed")]`,
   `#[cfg(test)] pub mod fake;`, and `build.rs` env-var gating. Individually
   simple, but three different mechanisms doing similar jobs.
6. **`unsafe { std::env::set_var/remove_var }`** in test helpers
   (`config.rs`) — became `unsafe` in newer Rust editions; surprising the
   first time you see it.

What's notably **absent** (and shouldn't be missed): no lifetimes beyond
implicit ones, no builder pattern, no extension traits, no hand-rolled
macros, no generic type parameters beyond the one `impl Serialize`.

## 9. The one architectural decision worth questioning

**Redis Time Series** (`store/redis.rs`) for 3 scalar values sampled every
~10s from one hobby fermenter, implemented via raw `TS.CREATE`/`TS.ADD`
string commands (the `redis` crate has no typed Time Series API). Per
`docs/rewrite-plan.md`, this was chosen mainly as a like-for-like InfluxDB
replacement plus Redis 8's convenient bundled image — not because any
range/chart queries exist yet (charts are explicitly deferred in the plan).
A plain Redis hash/list (or SQLite) would give equivalent durability with a
smaller, less exotic command surface. Not "wrong," just the one place where
"what the old stack did" drove the choice more than "what this app actually
queries." Worth revisiting if/when chart/range queries are ever built —
otherwise not worth disturbing today.

## 10. Recommendation

Don't restructure the module layout, concurrency model, or trait
boundaries — they're close to the simplest thing that works, and removing
the `TimeStore`/`SerialSource` traits would cost the hardware-free test
suite for a marginal readability gain. Focus study time on: `dyn Trait` +
`async_trait`, and the `tokio::join!`-without-`spawn` concurrency style.
Those two, once understood, unlock the rest of the codebase.

### Actioned from this review
- Removed the unused `anyhow` dependency from `fermenter/Cargo.toml` (dead
  weight, never referenced in `src/`) and corrected the stale mention in
  `openspec/config.yaml`. No OpenSpec change was needed — no capability
  behavior changed.
</content>
