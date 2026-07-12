## Why

The controller currently has no way to observe its state except log lines —
slices 1–2 ingest and persist readings, but nothing renders them anywhere a
person can look. The old Python system's whole reason for a `web` service was
exactly this: a browsable dashboard. This slice gives the Rust rewrite the
same visible, demoable payoff — open a browser and see the latest reading —
using the target stack's web layer (Axum + MiniJinja + HTMX) for the first
time, with live polling replacing the old manual-refresh page.

## What Changes

- Add an Axum HTTP server, run alongside the existing ingest loop in the same
  process (single binary, per `rewrite-plan.md` §2), sharing current state via
  an `Arc<Mutex<Option<Reading>>>` already established in slice-1/2 — no new
  channel plumbing yet (target/brew commands arrive with slices 4–5).
- Build a `minijinja::Environment` at startup (path loader for dev; template
  directory shipped alongside the binary) and store it in shared app state for
  handlers to render named templates against a `serde` context.
- `GET /` — full dashboard page (`dashboard.html`, extending `base.html`):
  latest reading (average/min/max/ambient/target/action), brew id, and a
  "last updated" indication.
- `GET /status` — an HTMX-polled fragment (`partials/status.html`) rendering
  just the current-state block, included by `dashboard.html` and re-fetched
  every ~5s (`hx-get="/status" hx-trigger="every 5s"`) so the page updates
  without a manual refresh — the direct improvement over the old pull-on-load
  UI (`system-analysis.md` §3.6).
- `GET /static/*` — serve `styles.css` and any other static assets.
- Extend `/healthz` (new endpoint) to report liveness plus a serial-connected
  boolean, satisfying the `operational-health` amendment; Redis-connected
  status is deferred since no shared handle to the store's live connection
  state exists yet outside the ingest task (tracked as a follow-up, not a
  regression — the old system had no health endpoint at all).
- No new inbound mutation: this slice is read-only in the browser. Forms to
  change target temp or brew id are explicitly out of scope (slice-4/5).

## Capabilities

### New Capabilities

- `web-dashboard`: Server-rendered dashboard and HTMX-polled status fragment
  showing current controller state (latest reading, brew id) over HTTP, plus
  static asset serving.

### Modified Capabilities

- `operational-health`: adds a liveness/health-check requirement (`/healthz`)
  to the existing structured-logging and typed-error requirements.

## Impact

- **New module**: `web` (`web/mod.rs` — router + `AppState`; `web/handlers.rs`
  — route handlers; `web/render.rs` — thin MiniJinja render helper mapping
  errors to `AppError`).
- **New templates/static assets**: `templates/base.html`, `dashboard.html`,
  `partials/status.html`; `static/styles.css`.
- **Modified modules**: `main.rs` (spawn the Axum server alongside the ingest
  task instead of only running ingest); `error.rs` (a `Render`/`Http` error
  variant if MiniJinja render failures need distinct typed handling);
  `config.rs` (add `HTTP_PORT`, defaulting per `rewrite-plan.md` §9).
- **Dependencies introduced**: `axum`, `minijinja`, `tower` (dev-dependency,
  `ServiceExt::oneshot` for HTTP tests), `insta` (dev-dependency, HTML
  snapshot tests for templates/fragments).
- **No impact** on the ingest/store code paths from slice-1/2 beyond sharing
  the existing `Arc<Mutex<Option<Reading>>>` with the new web layer; no
  changes to the Redis schema or serial contract.
- **No impact** on the Python system, which remains deployable on `master`
  until cutover.
