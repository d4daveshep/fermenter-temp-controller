# Redesign Plan: Fermenter Temperature Controller (Rust Rewrite)

> Ground-up rewrite of the fermentation temperature controller using Rust,
> HTMX, MiniJinja templating, and Redis Time Series (replacing InfluxDB).
> Deployment remains Docker images on a Raspberry Pi. The Arduino firmware is
> unchanged and treated as an external device over a serial link.

## 1. Decisions locked in

| Area | Decision |
|---|---|
| Topology | **Single Rust binary** — serial task + Axum web server, shared state via channels/`Arc` |
| Web framework | **Axum + Tokio** |
| Templating | **MiniJinja** (runtime, Jinja2-compatible, hot-reload in dev) |
| Frontend interactivity | **HTMX**, polling a fragment endpoint (replaces manual refresh) |
| Time-series store | **Redis 8** official image (Time Series built in), ARM64 multi-arch |
| State source of truth | **In-memory authoritative, mirrored to Redis** for durability across restarts |
| Config | **Env vars + `.env`** (12-factor) |
| Image build | **Multi-stage, cross-compiled on dev machine** for `linux/arm64`; tiny runtime image |
| Serial contract | **Strict serde struct**, same fields; log/reject malformed lines |
| Resilience | **Auto-reconnect (serial + Redis) with backoff; `tracing` structured logs; typed errors** |
| Dev/test | **Mock serial source** behind a trait for hardware-free dev and unit tests |
| UI scope | **Current state only** (latest reading, target, action, forms); charts deferred |

## 2. Target architecture

```
                +-------------------+
                |     Arduino       |  (firmware unchanged, external)
                +---------+---------+
                          | USB serial @115200
              JSON lines  |  ▲  "<target>" framed float
                          ▼  │
   ┌───────────────────────────────────────────────────────┐
   │            Single Rust binary (Tokio runtime)          │
   │                                                        │
   │  SerialTask  ──readings──►  mpsc  ──►  Ingest/State    │
   │     ▲                                    │   │         │
   │     └──── target updates (watch) ◄───────┘   │ writes  │
   │                                              ▼         │
   │  Axum web server ──► AppState (Arc) ──► RedisStore ────┼──► Redis 8
   │     ▲  HTTP :8080  (MiniJinja + HTMX)         ▲        │   (TimeSeries)
   └─────┼────────────────────────────────────────┼────────┘
         │ browser                                 │ persist target/brew + readings
```

**Why single binary:** removes ZeroMQ entirely. The web layer mutates
target/brew-id via a channel the serial task listens to; no separate process,
no dual source of truth. 2 containers total (app + Redis) vs. the old 3.

## 3. Crate & module layout

```
fermenter/
├── Cargo.toml
├── templates/                  # MiniJinja templates (loaded at runtime / embedded)
│   ├── base.html
│   ├── dashboard.html
│   ├── partials/
│   │   └── status.html         # HTMX-polled fragment ({% include %}-able)
│   ├── target_form.html
│   └── brew_form.html
├── static/
│   └── styles.css
└── src/
    ├── main.rs                 # wiring: config, build MiniJinja Environment, spawn tasks, run server
    ├── config.rs               # env parsing (figment/envy) -> Config
    ├── error.rs                # thiserror AppError, Result alias
    ├── model.rs                # Reading, Action, ControllerState, commands
    ├── serial/
    │   ├── mod.rs              # SerialSource trait + reconnect loop
    │   ├── arduino.rs          # tokio-serial impl (real hardware)
    │   └── mock.rs             # fake feed for dev/test
    ├── state.rs                # AppState (Arc), watch/mpsc channels
    ├── store/
    │   ├── mod.rs             # TimeStore trait
    │   └── redis.rs           # Redis TS impl (+ reconnect)
    └── web/
        ├── mod.rs             # Router construction + MiniJinja Environment in AppState
        ├── handlers.rs        # route handlers (render named templates with serde context)
        └── render.rs          # thin helper: render(env, name, ctx) -> Html<String>, maps errors
```

## 4. Key types & traits

```rust
// model.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reading {            // strict serde of Arduino JSON; Serialize for template context
    pub target: f64,
    pub average: f64,
    pub min: f64,
    pub max: f64,
    pub ambient: f64,
    pub action: String,
    #[serde(rename = "reason-code", default)]
    pub reason_code: String,
    #[serde(rename = "json-size", default)]
    pub json_size: Option<u32>, // ignored on write
}

#[derive(Clone, Serialize)]
pub struct ControllerState {    // authoritative runtime state; Serialize so it can feed templates
    pub target_temp: f64,
    pub brew_id: String,
    pub latest: Option<Reading>,
}

pub enum Command {              // web -> serial task
    SetTarget(f64),
    SetBrewId(String),
}
```

```rust
// serial/mod.rs
#[async_trait]
pub trait SerialSource: Send {
    async fn read_line(&mut self) -> Result<String>;
    async fn write_target(&mut self, target: f64) -> Result<()>;
}
// arduino.rs -> tokio_serial; mock.rs -> scripted lines for dev/tests
```

```rust
// store/mod.rs
#[async_trait]
pub trait TimeStore: Send + Sync {
    async fn write_reading(&self, brew_id: &str, r: &Reading) -> Result<()>;
    async fn last_reading(&self, brew_id: &str) -> Result<Option<Reading>>;
    async fn save_state(&self, s: &ControllerState) -> Result<()>;
    async fn load_state(&self) -> Result<Option<ControllerState>>;
}
```

## 5. Concurrency design (channels)

- **`watch::channel<f64>`** for target temp: web handler updates it; serial task
  observes and writes `<target>` to the port when it differs from the Arduino's
  reported target (event-driven equivalent of the old 5s reconcile).
- **`mpsc::channel<Command>`** for brew-id changes and any other commands
  web → serial/state task.
- **`AppState { state: Arc<RwLock<ControllerState>>, store: Arc<dyn TimeStore>, cmd_tx, target_tx, env }`**
  shared with Axum via the `State` extractor (`env` = MiniJinja `Environment`).
- Serial task loop: read line → deserialize → update `latest` in state →
  `store.write_reading` → check target watch.

## 6. Templating layer (MiniJinja)

- Build a single `minijinja::Environment` at startup, stored in `AppState`
  (wrapped in `Arc`).
- **Dev:** load templates from the `templates/` directory via
  `Environment::set_loader(path_loader(...))` so edits hot-reload without
  recompiling. Optionally use `minijinja-autoreload` for automatic cache
  invalidation.
- **Release:** embed templates into the binary with `minijinja-embed` (or
  `include_str!`) so the runtime image stays self-contained — no mounted
  template volume. A cargo feature flag (`embed`) switches between the two
  loaders.
- **Rendering:** handlers build a `serde`-serializable context (the
  `ControllerState`/`Reading` structs already derive `Serialize`, plus a small
  wrapper for page-specific extras like the formatted timestamp) and call a
  thin `render(&env, "dashboard.html", ctx)` helper that returns
  `axum::response::Html<String>` and maps `minijinja::Error` into `AppError`.
- **HTMX fragments are first-class:** `partials/status.html` is rendered by name
  directly for the `/status` poll endpoint and
  `{% include "partials/status.html" %}`-ed inside `dashboard.html` — no
  per-fragment Rust struct, no duplicated layout context. This is the main
  ergonomic win driving the MiniJinja choice.
- Template inheritance: `dashboard.html`, `target_form.html`, `brew_form.html`
  all `{% extends "base.html" %}`.

## 7. HTTP endpoints (Axum + HTMX)

| Method & Path | Purpose | Returns |
|---|---|---|
| `GET /` | Full dashboard page | renders `dashboard.html` |
| `GET /status` | HTMX-polled fragment (every ~5s) | renders `partials/status.html` |
| `GET /target` | Target temp form | `target_form.html` |
| `POST /target` | Update target → `watch` + persist | redirect / swapped fragment |
| `GET /brew` | Brew-id form | `brew_form.html` |
| `POST /brew` | Update brew-id → `mpsc` + persist | redirect / swapped fragment |
| `GET /healthz` | Liveness (serial + redis status) | JSON |
| `GET /static/*` | CSS | static file |

Dashboard panel: `hx-get="/status" hx-trigger="every 5s" hx-swap="innerHTML"`.

## 8. Redis key schema (Redis 8 Time Series)

- `TS.CREATE temp:{brew_id}:fermenter RETENTION <ms> LABELS brew {brew_id} field fermenter`
  (and `ambient`, `target`) — created lazily, ignore "already exists".
- `TS.ADD temp:{brew_id}:{field} <ts> <value>` per field per reading.
- Persisted control state: a plain hash `controller:state` →
  `{ target_temp, brew_id }`, loaded on startup, written on every change.
- Retention configurable via env (default 1 week, matching old InfluxDB).

## 9. Configuration (env vars)

```
SERIAL_PORT=/dev/ttyACM0
SERIAL_BAUD=115200
REDIS_URL=redis://localhost:6379
TS_RETENTION_DAYS=7
DEFAULT_TARGET_TEMP=19.5     # only used if no persisted state
DEFAULT_BREW_ID=00-TEST-v00
TIMEZONE=Pacific/Auckland
HTTP_PORT=8080
MOCK_SERIAL=false            # true => use mock source, no hardware
RUST_LOG=info
```

Parsed once at startup into a `Config` struct; on parse error, log and exit
non-zero. (Separately, the `embed` cargo feature decides whether templates are
loaded from disk or embedded — not an env var.)

## 10. Resilience & observability

- **Serial reconnect:** if the port errors/closes, log, back off (exponential,
  capped), reopen. Malformed JSON lines are logged at `warn` and skipped — not
  silently swallowed like the old loop.
- **Redis reconnect:** connection-manager style; writes retried with backoff; a
  failed write logs but doesn't crash the serial loop.
- **Logging:** `tracing` + `tracing-subscriber`, env-filtered, structured.
- **Errors:** `thiserror` for typed errors (`AppError`), `anyhow` at top-level
  wiring only. Template render failures map to `AppError` → 500 with a logged
  `tracing` error (the runtime-error tradeoff accepted by choosing MiniJinja),
  covered by template smoke tests.
- **`/healthz`** reports serial-connected and redis-connected booleans for a
  Compose healthcheck.

## 11. Deployment (Docker on ARM64)

**`Dockerfile` (multi-stage, cross-compiled):**
- Stage 1 (builder, `linux/arm64` via `docker buildx` / `cross` on the dev
  machine): `rust:1-bookworm`, `cargo build --release --features embed` so
  templates are baked into the binary.
- Stage 2 (runtime): `debian:bookworm-slim` (or distroless) — copy the single
  binary + `static/`. With `embed`, no `templates/` volume is needed; the image
  is self-contained.

**`compose.yaml`:**
```yaml
services:
  redis:
    image: redis:8                 # TimeSeries built in
    volumes: [ ./redis-data:/data ]
    command: redis-server --save 60 1
    ports: [ "6379:6379" ]

  fermenter:
    build: .
    env_file: .env
    devices: [ "/dev/ttyACM0:/dev/ttyACM0" ]   # device passthrough, not privileged
    depends_on: [ redis ]
    ports: [ "8080:8080" ]
    restart: on-failure
```
Improvement over old setup: pass the specific serial device instead of running
`privileged`/host-network; config via `env_file` instead of baking into the
image.

## 12. Testing strategy (TDD + integration)

### 12.1 Philosophy

TDD red→green→refactor throughout. The three external boundaries
(`SerialSource`, `TimeStore`, HTTP) are trait-abstracted so the bulk of
behaviour is testable without hardware or network. Classic pyramid weighting:
many fast unit tests, a solid integration layer, a few gated end-to-end tests.

```
   e2e (few, #[ignore], real Arduino + Redis)
   integration (some: real Redis via testcontainers; Axum oneshot)
   unit (many, fast, default; mock SerialSource + fake TimeStore)
```

### 12.2 Tooling

- `cargo test` (std harness) as the base.
- **testcontainers-rs** — spins up `redis:8` per integration run; requires
  Docker on the test host.
- **rstest** — parametrized cases and fixtures (esp. the many `Reading` serde
  cases).
- **insta** — snapshot tests for rendered HTML fragments (ideal for HTMX
  partials; review diffs with `cargo insta review`).
- **tower** (`ServiceExt::oneshot`) — drive the Axum `Router` without a real
  socket.

### 12.3 Test doubles (build first, milestones 2–3)

| Double | For | Behaviour |
|---|---|---|
| `MockSerialSource` | unit + http | scripted lines to emit; records `write_target` calls; can inject errors for reconnect/backoff tests |
| `FakeTimeStore` | http | in-memory map; deterministic, no network |
| `app_for_test(store, serial)` | http + full-stack | builds the real `Router` + `AppState` from injected doubles |

### 12.4 Layers and representative tests

**Unit (inline `#[cfg(test)]`, fast, default):**
- `Reading` serde via `rstest` cases: valid, missing required field, default
  `reason-code`, ignored `json-size`, malformed JSON.
- Target reconcile: writes when differing, no-op when equal (asserts on
  `MockSerialSource` records).
- State transitions: set-target signals `watch`, set-brew updates state, latest
  reading replaced.
- Template smoke/snapshot tests (in-process MiniJinja): dashboard, status
  fragment, forms render with representative context (insta snapshots). Recovers
  most of the compile-time safety given up vs. Askama.

**Integration (`tests/`, public API only):**
- `tests/store_redis.rs` (testcontainers `redis:8`): write→last roundtrip,
  idempotent `TS.CREATE` on restart, retention label applied, save/load
  controller state roundtrip, load returns `None` on empty DB.
- `tests/http_handlers.rs` (Axum oneshot + fake store + mock serial): `/`
  returns 200 + dashboard, `/status` returns fragment only, `POST /target`
  updates state + 303 redirect, `POST /brew` likewise, out-of-range target
  rejected, `/healthz` reports component status.
- `tests/full_stack.rs` (optional, router + real Redis): `POST /target` then
  `GET /status` reflects change end-to-end.

**End-to-end (gated):**
- `tests/serial_hardware.rs`, all `#[ignore]`: open + read real line, write
  target roundtrip. Run on the Pi via `cargo test -- --ignored`. Excluded from
  normal runs and CI.

### 12.5 Test layout

```
fermenter/
├── src/                          # inline #[cfg(test)] unit tests
└── tests/
    ├── common/mod.rs             # FakeTimeStore, MockSerialSource, app_for_test factory
    ├── store_redis.rs            # testcontainers Redis TS
    ├── http_handlers.rs          # Axum oneshot, fake store
    ├── full_stack.rs             # router + real Redis (optional)
    └── serial_hardware.rs        # #[ignore] e2e, real Arduino
```

Unit tests stay inline (`#[cfg(test)]`) next to the code (Rust idiom, access to
private items). Integration tests live in `tests/` and use only the public API,
which usefully pressures the crate's public surface to stay clean.

### 12.6 CI outline (GitHub Actions)

On every push/PR (Docker available on the runner for testcontainers):
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test` (unit + integration; testcontainers pulls `redis:8`)
- Hardware e2e excluded (they're `#[ignore]`'d; never run in CI).
- Build job: `docker buildx` cross-compile for `linux/arm64` with
  `--features embed` to verify the release image builds.

### 12.7 TDD workflow folded into the milestones

| Milestone | Write tests first | Then implement |
|---|---|---|
| 1 Skeleton | `Reading` serde rstest cases; config parse cases | `model.rs`, `config.rs` |
| 2 Serial | `MockSerialSource`; reconcile + reconnect/backoff tests | `SerialSource`, mock, reconnect loop |
| 3 State | state-transition + watch/mpsc signalling tests | `AppState`, channels |
| 4 Redis store | `tests/store_redis.rs` (testcontainers) roundtrips | `TimeStore` Redis impl |
| 5 Web | `tests/http_handlers.rs` + template insta snapshots | router, handlers, templates |
| 6 Real serial | (covered by `#[ignore]` hardware tests) | `arduino.rs` |
| 7 Dockerize | CI build job asserts image builds | Dockerfile, compose |
| 8 Pi deploy | run `cargo test -- --ignored` on Pi | verify on hardware |

## 13. Phased build order (milestones)

1. **Skeleton:** Cargo project, `Config` from env, `tracing`, `model.rs` with
   `Reading` serde + tests. *(no hardware)*
2. **Serial layer:** `SerialSource` trait, `mock.rs`, reconnect loop; drive
   state from mock feed.
3. **State + channels:** `AppState`, `watch`/`mpsc`, target reconcile against
   mock.
4. **Redis store:** `TimeStore` trait + Redis TS impl; write readings, save/load
   control state; reconnect.
5. **Web (MiniJinja + HTMX):** build `Environment` with path loader;
   `base.html`, dashboard, `/status` polling fragment, target & brew forms,
   `/healthz`; add template smoke tests.
6. **Real serial:** `arduino.rs` via `tokio-serial`; wire `MOCK_SERIAL` switch.
7. **Dockerize:** multi-stage cross-compile with `--features embed`;
   `compose.yaml`; device passthrough; healthcheck.
8. **Pi deploy + verify:** end-to-end on hardware; confirm readings flow and UI
   updates.

## 14. Key dependencies

`tokio`, `axum`, **`minijinja`** (+ optionally **`minijinja-autoreload`** for dev
hot-reload and **`minijinja-embed`** for release embedding), `tokio-serial`,
`serde`/`serde_json`, `redis` (async; TS via raw `TS.*` commands), `figment` or
`envy`, `tracing`/`tracing-subscriber`, `thiserror`/`anyhow`, `async-trait`.
Dev: `tower` (`ServiceExt::oneshot`), `testcontainers` (+ redis module),
`rstest` (parametrized cases/fixtures), `insta` (HTML snapshot tests). No
production-dependency changes from the testing strategy.

## 15. Git strategy

**Approach:** Long-lived `rewrite/rust` branch off `master`. `master` stays the
deployable Python system throughout; the Rust app is built on the branch and
lands via a merge commit at cutover. Shared `arduino/` history is preserved.
This fits the repo's existing feature-branch workflow and keeps the live
temperature controller deployable at every point until cutover.

### Phase 0 — Land the plan docs (on `master`)
1. On `master`, commit the two docs so they're visible to everyone:
   - `docs/system-analysis.md`
   - `docs/rewrite-plan.md`
   - Suggested message: `docs: add system analysis and Rust rewrite plan`
2. Push `master`.

### Phase 1 — Create the rewrite branch
3. `git checkout -b rewrite/rust` from the updated `master`.
4. Push and set upstream: `git push -u origin rewrite/rust`.
5. Branch carries the plan docs forward automatically (committed to `master`
   first).

### Phase 2 — Build on the branch (per the 8 milestones)
6. Develop the Rust app on `rewrite/rust`, one focused commit (or sub-PR into
   the branch) per milestone (M1 skeleton → M8 Pi verify).
7. **Leave `arduino/` untouched.** If firmware fixes are needed during the
   rewrite, make them on `master` and merge/cherry-pick into `rewrite/rust` so
   both stacks share the fix and history stays unified.
8. The Python tree stays in place on the branch until cutover, so old behaviour
   (serial contract, reconcile logic) can be cross-referenced while building.
9. Python is in maintenance so `master` barely moves — drift risk is low.
   Periodically `git merge master` into the branch to absorb `arduino/`/doc
   changes.

### Phase 3 — Cutover (when Rust is proven on the Pi)
10. On `rewrite/rust`, remove the obsolete Python stack in a dedicated commit:
    delete `controller/`, `web/`, `Dockerfile.controller`, `Dockerfile.web_apis`,
    `Dockerfile.pytests`, `pyproject.toml`, `uv.lock`, Python `tests/`, etc.;
    update `docker-compose.yaml` to the 2-container (app + Redis) layout; keep
    `arduino/`, `docs/`, and revised `INSTALL.md`/`README.md`.
11. **Tag the last Python state on `master` before merging** so it's
    recoverable:
    - `git tag -a v1-python <last-python-master-commit> -m "Final Python/InfluxDB system before Rust rewrite"`
    - `git push origin v1-python`
12. Open a PR `rewrite/rust → master`, review the (large) rewrite diff, and
    merge with a **merge commit** (not squash) to preserve granular history.
13. Optionally tag the new baseline `v2-rust`.
14. Delete `rewrite/rust` after merge.

### Guardrails
- `master` remains buildable/deployable until step 12 — critical for a live
  controller.
- The big "delete Python" diff happens once, deliberately, in the cutover PR.
- `v1-python` is the rollback safety net for the Pi.
- Per-milestone commits keep the rewrite history reviewable rather than one
  opaque squash.

## 16. Notes & open items

- MiniJinja chosen over Askama for beginner DX, true Jinja2 syntax, and
  fragment-friendly HTMX rendering; the runtime-error tradeoff is offset by
  template smoke tests. Switching later is low-cost since both consume `serde`
  context.
- The `redis` crate talks to Time Series via raw `TS.ADD`/`TS.RANGE` commands
  (no first-class typed API) — straightforward but worth knowing.
- Charts are explicitly deferred; the Redis TS schema already supports adding a
  range-query chart endpoint later with no data migration.
- `RedisTimeSeries` standalone images are deprecated; Redis 8+ includes Time
  Series built in (64-bit only — fine for a modern Pi).
- **Status (milestones 7+8 / `slice-7-deployment`, complete):** the Rust
  stack is now packaged (self-contained cross-compiled ARM64 Docker image,
  `embed`-baked templates, `fermenter/compose.yaml` with scoped device
  passthrough) and deployed to the Raspberry Pi — dashboard, ingestion, and
  target/brew round-trips all verified against the Pi's real, sensored
  Arduino. The old Python/InfluxDB stack (`docker-compose.yaml` at the repo
  root) is left in place, untouched, alongside it. The §15 Phase 3 cutover
  (removing the Python stack, tagging `v1-python`, merging `rewrite/rust` →
  `master`) is now available as the next step whenever desired — it is a
  deliberate, separate decision, not performed as part of this slice.
- **Status (§15 Phase 3 cutover, `cutover-to-rust`):** on this branch, the
  old Python/InfluxDB stack has been deleted, and `compose.yaml` (formerly
  `fermenter/compose.yaml`) is now the sole repo-root orchestration file —
  the 2-container Rust + Redis stack is the only deployable system
  described in the repository. The remaining steps in this section's
  numbered sequence — tagging `master` `v1-python`, merging
  `rewrite/rust → master` via a merge commit, tagging the result
  `v2-rust`, and deleting `rewrite/rust` — are the immediate next actions
  to actually complete the cutover.
