## Context

Slices 1–2 built the ingest loop, `Reading`/`ControllerState` models, the
`TimeStore` trait + Redis implementation, and config/error/logging
foundations — but the only way to observe state today is `RUST_LOG=info`
output. `main.rs` runs a single `ingest_loop` to completion and exits; there
is no long-running server. This slice adds the first HTTP surface
(`rewrite-plan.md` §6–7) and must decide how the new Axum server shares state
with the existing ingest task without over-building the channel
infrastructure `rewrite-plan.md` §5 describes for slices 4–5 (target
reconcile, brew relabel), which don't exist yet.

`web-dashboard` is a brand-new capability (`## ADDED`); `operational-health`
already has structured-logging and typed-error requirements from slice-1 and
gains a `/healthz` requirement here (`## MODIFIED`, additive — no existing
scenario changes).

## Goals / Non-Goals

**Goals:**

- Axum HTTP server running concurrently with the existing ingest loop in one
  process (`tokio::join!` or `tokio::spawn`), per the single-binary decision
  in `rewrite-plan.md` §1–2.
- `AppState` shared between the ingest task and the web layer: the
  `Arc<Mutex<Option<Reading>>>` already threaded through `main.rs`/`ingest.rs`,
  plus an `Arc<minijinja::Environment<'static>>` and the brew id needed for
  display.
- `minijinja::Environment` built once at startup from a `templates/`
  directory (path loader — dev-only ergonomics; embedding is a slice-7
  deployment concern per `rewrite-plan.md` §6, not tackled here).
- Routes: `GET /` (full page), `GET /status` (HTMX fragment), `GET /healthz`
  (JSON liveness), `GET /static/*` (CSS).
- `/status` fragment is `{% include %}`-able from `dashboard.html` and
  independently renderable by name for the polling endpoint — no duplicated
  template or per-fragment Rust struct (`rewrite-plan.md` §6's stated
  ergonomic win).
- Template and handler tests: `insta` snapshot tests for rendered HTML with
  representative context (with-reading and no-reading-yet cases); `tower`
  oneshot HTTP tests against a real `Router` built from injected state
  (no real ingest loop or Redis needed).

**Non-Goals:**

- No forms, no `POST` routes, no mutation of target temp or brew id from the
  browser — that's `temperature-control` (slice-4) and `brew-session`
  (slice-5). The dashboard is strictly read-only this slice.
- No `watch`/`mpsc` command channels (`rewrite-plan.md` §5) — nothing yet
  needs to push a value *into* the ingest task from the web side. Introducing
  them now would be speculative; slice-4 adds the first command
  (`SetTarget`) and is the natural place for that channel to appear.
- No template embedding/`minijinja-embed` cargo feature — deferred to
  slice-7 (Dockerize), where the runtime image needs to be self-contained.
  Dev/this-slice loads from disk.
- No chart/history endpoints — `rewrite-plan.md` explicitly defers charts;
  `/status` only ever shows current state, not a range query.
- Redis-connected status in `/healthz` — no live handle to the Redis
  connection-manager's state is exposed outside `store/redis.rs` today.
  Reporting only `serial_connected`-equivalent liveness (the ingest task is
  alive) this slice; extending `/healthz` with a real Redis check is a
  natural, low-risk follow-up but not required to satisfy this slice's
  `operational-health` delta (see Open Questions).

## Decisions

**1. Reuse the existing `Arc<Mutex<Option<Reading>>>`; no new `AppState`
struct with channels yet.**
`rewrite-plan.md` §4/§5 sketches a full `ControllerState`/`AppState` with
`watch`/`mpsc` channels for target/brew commands. Building that now, before
any handler needs to *send* a command, would add unused plumbing. Instead,
`web::AppState` this slice is a small, honest struct:
```rust
pub struct AppState {
    pub latest: Arc<Mutex<Option<Reading>>>,
    pub brew_id: String,
    pub env: Arc<minijinja::Environment<'static>>,
}
```
`main.rs` constructs one `AppState`, clones the `Arc<Mutex<..>>` already
passed to `ingest_loop`, and passes `AppState` (wrapped for Axum's `State`
extractor) into the router. Alternative (build the full `rewrite-plan.md`
§5 `AppState` + channels now) rejected — YAGNI; slice-4 will grow this
struct when `SetTarget` needs a `watch::channel`, and that's a small,
well-scoped addition against real requirements rather than speculative
plumbing.

**2. `minijinja::Environment` with a path loader, dev-mode only.**
`Environment::set_loader(path_loader("templates"))` gives Jinja2-compatible
hot-reload during development, matching `rewrite-plan.md` §6's stated
dev/release split. The `embed` cargo feature (baking templates into the
binary via `minijinja-embed`/`include_str!`) is explicitly a slice-7 concern
tied to the Docker image; building it now would couple this slice to
deployment packaging prematurely. Risk: running the compiled binary from a
directory without a sibling `templates/` folder fails at render time — an
accepted, documented constraint of dev-mode loading (mirrors the existing
`web/fastapi_app.py` path-flattening quirk called out in `AGENTS.md`, which
this rewrite deliberately does *not* inherit long-term, only defers fixing).

**3. `/status` is the single source of template truth for current-state
markup.**
`dashboard.html` renders `{% include "partials/status.html" %}`; the `/status`
handler renders `partials/status.html` directly by name. Both consume the
same `serde` context shape (a small `StatusContext { reading: Option<Reading>,
brew_id: String, updated_at: Option<String> }|`). Alternative (separate
"full page" and "fragment" markup, kept manually in sync) rejected — exactly
the duplication `rewrite-plan.md` §6 calls out MiniJinja/HTMX as solving.

**4. `/healthz` reports `serial_connected` only this slice (not
`redis_connected`).**
`operational-health`'s new requirement needs a shippable, testable liveness
check now. The ingest task's aliveness is observable today (does the shared
`Arc<Mutex<Option<Reading>>>` exist / is the task still running — tracked via
a simple `Arc<AtomicBool>` set by the ingest task, or, more simply, whether at
least one reading has ever landed). Wiring a live Redis health probe would
require exposing internal state from `store/redis.rs`'s connection manager
that doesn't exist yet and isn't needed for this slice's read-only dashboard.
This is a deliberate scope cut, not a regression (the old Python system had
no health endpoint at all) — see Open Questions.

**5. HTTP test strategy: `tower::ServiceExt::oneshot` against a `Router`
built from a fake/seeded `AppState`, no real ingest loop.**
Mirrors slice-2's `FakeTimeStore` pattern: build the smallest fake needed at
the boundary (`Arc<Mutex<Option<Reading>>>` is already trivially fake-able —
just construct one directly with a seeded `Reading` or `None`) rather than
introducing a new trait for "current state provider." No new abstraction is
justified since there's exactly one implementation.

## Risks / Trade-offs

- **[Risk]** Path-loaded templates mean the binary is not standalone in dev
  → **Mitigation**: documented now, fixed in slice-7 with the `embed`
  feature; not a regression since nothing standalone exists yet.
- **[Risk]** `/healthz` without a real Redis check could report "healthy"
  while persistence is silently failing (reading-history's own resilience
  requirement already logs+continues on write failure, so this isn't a new
  failure mode, just an incomplete health signal) → **Mitigation**: scope cut
  documented explicitly in the proposal/spec; tracked as a follow-up for
  slice-4/5 or a dedicated hardening slice once a Redis-status accessor
  exists.
- **[Risk]** Sharing `Arc<Mutex<Option<Reading>>>` directly with Axum
  handlers (a std `Mutex` held briefly across a lock/clone/drop, never across
  an `.await`) is fine under Tokio, but it's easy to accidentally hold the
  lock across an `await` point later and deadlock the runtime →
  **Mitigation**: handlers clone the `Option<Reading>` out and drop the lock
  immediately, matching the existing pattern already used in `ingest.rs`.
- **[Trade-off]** No channels yet means slice-4 will need to touch this
  slice's `AppState` again to add a `watch::Sender<f64>` — accepted as the
  right amount of incrementalism per Decision 1.

## Open Questions

- Should `/healthz` gain a `redis_connected` field in this slice, or is
  deferring it to a later slice (once `store/redis.rs` exposes a status
  accessor) acceptable? Current plan: defer, and note it as a named gap in
  the spec's scenario set rather than silently omitting it.
- Template location for the shipped/dev binary: `templates/` relative to
  `Cargo.toml` (works with `cargo run`) — confirm this is also fine for the
  slice-7 Docker builder stage layout in `rewrite-plan.md` §11, or whether
  the path needs to be configurable via env now to avoid another migration
  later. Leaning toward: hardcode `"templates"` relative path this slice;
  revisit only if slice-7 needs it configurable.
