## 1. Dependencies & config

- [ ] 1.1 Add `axum` to `[dependencies]`, and `tower` + `insta` to
      `[dev-dependencies]` in `Cargo.toml`
- [ ] 1.2 Add `minijinja` to `[dependencies]`
- [ ] 1.3 Write a config-parse test (RED) for `HTTP_PORT` (with a default,
      e.g. 8080) in `config.rs`
- [ ] 1.4 Extend `Config` with `http_port` to pass the test (GREEN)

## 2. Templates & static assets

- [ ] 2.1 Create `templates/base.html` (minimal layout: `<html>`/`<head>`
      with the stylesheet link, `{% block content %}`)
- [ ] 2.2 Create `templates/partials/status.html`: renders brew id + latest
      reading fields (average/min/max/ambient/target/action), or a "no
      reading yet" message when the reading is absent
- [ ] 2.3 Create `templates/dashboard.html`: `{% extends "base.html" %}`,
      `{% include "partials/status.html" %}` inside an element with
      `hx-get="/status" hx-trigger="every 5s" hx-swap="innerHTML"`
- [ ] 2.4 Create `static/styles.css` (minimal styling, can be sparse)

## 3. Web module scaffolding (web-dashboard)

- [ ] 3.1 Create `web/mod.rs`: `AppState { latest: Arc<Mutex<Option<Reading>>>,
      brew_id: String, env: Arc<minijinja::Environment<'static>> }`; a
      `build_router(state: AppState) -> Router` function wiring routes to
      handlers plus a `/static` `ServeDir`
- [ ] 3.2 Create `web/render.rs`: `render(env, name, ctx) -> Result<Html<String>,
      AppError>` helper wrapping `Environment::get_template` +
      `Template::render`, mapping `minijinja::Error` into a typed `AppError`
      variant (extend `error.rs` with an `AppError::Render(String)` variant
      and its `IntoResponse` mapping to a 500)
- [ ] 3.3 Add `pub mod web;` to `lib.rs`

## 4. Dashboard & status routes (web-dashboard) — TDD

- [ ] 4.1 Write an `insta` snapshot test (RED) rendering
      `partials/status.html` with a seeded `Reading` context — captures the
      expected fragment markup
- [ ] 4.2 Write an `insta` snapshot test (RED) rendering `partials/status.html`
      with no reading (`None`) — captures the "no data yet" markup
- [ ] 4.3 Implement the `/status` handler (`GET`): builds the status context
      from `AppState.latest` + `brew_id`, calls `render(..., "partials/status.html", ctx)`
      (GREEN for 4.1/4.2)
- [ ] 4.4 Write a `tower::oneshot` HTTP test (RED): `GET /status` against a
      router built with a seeded `AppState` returns 200 and body containing
      the latest reading's average value
- [ ] 4.5 Write a `tower::oneshot` HTTP test (RED): `GET /status` with no
      reading seeded returns 200 without erroring
- [ ] 4.6 Write a `tower::oneshot` HTTP test (RED): `GET /` returns 200 and
      body containing both the dashboard shell and the current-state content
      (same content as `/status` for the same state)
- [ ] 4.7 Implement the `/` handler (`GET`): builds the same context shape as
      `/status`, renders `dashboard.html` (GREEN for 4.4–4.6)
- [ ] 4.8 Write a `tower::oneshot` HTTP test (RED): `GET /static/styles.css`
      returns 200 with the stylesheet contents
- [ ] 4.9 Wire `ServeDir` for `/static` in `build_router` (GREEN for 4.8)

## 5. Read-only guarantee (web-dashboard)

- [ ] 5.1 Write a test (RED) enumerating the router's registered routes (or
      asserting directly against `build_router`'s method table) confirming no
      route under this slice accepts `POST`/`PUT`/`PATCH`/`DELETE`
- [ ] 5.2 Confirm the assertion holds given the routes implemented in Task 4
      (GREEN — no implementation change expected; this is a guardrail against
      scope creep into slice-4/5 territory)

## 6. Health check endpoint (operational-health) — TDD

- [ ] 6.1 Write a `tower::oneshot` HTTP test (RED): `GET /healthz` returns
      200 with a JSON body indicating the ingestion component is healthy,
      using an `AppState` where an "ingest alive" signal is set
- [ ] 6.2 Write a `tower::oneshot` HTTP test (RED): `GET /healthz` still
      returns 200 when no reading has ever been observed (liveness, not data
      presence)
- [ ] 6.3 Add an `Arc<AtomicBool>` (or equivalent) "ingest alive" flag to
      `AppState`, set by the ingest task at loop start; implement the
      `/healthz` handler returning `{"serial_connected": bool}` as JSON
      (GREEN for 6.1/6.2)

## 7. Main wiring

- [ ] 7.1 Restructure `main.rs` to build `web::AppState` from the same
      `Arc<Mutex<Option<Reading>>>` already passed to `ingest_loop`, the
      configured `default_brew_id`, and a `minijinja::Environment` built with
      `path_loader("templates")`
- [ ] 7.2 Spawn the ingest loop and the Axum server (bound to
      `0.0.0.0:{http_port}`) concurrently (e.g. `tokio::join!` or
      `tokio::spawn` + `axum::serve`), so both run for the lifetime of the
      process instead of the process exiting after ingestion completes
- [ ] 7.3 Confirm `MOCK_SERIAL=true` behaviour still works end-to-end: the
      mock feed's readings appear at `/status` while the process keeps running
      afterward instead of exiting (mock source exhaustion no longer ends the
      process)

## 8. Demo + verification

- [ ] 8.1 Run the binary with `MOCK_SERIAL=true`, open `http://localhost:8080/`
      in a browser, confirm the dashboard shows the mock readings and updates
      automatically without a manual refresh
- [ ] 8.2 Confirm `GET /healthz` responds while the server is running
- [ ] 8.3 Run `cargo fmt --check` and `cargo clippy --all-targets -- -D
      warnings` clean
- [ ] 8.4 Run the full `cargo test` suite (unit + `insta` snapshots + HTTP
      oneshot tests); review any new snapshots with `cargo insta review`
      before accepting
- [ ] 8.5 Run `openspec validate slice-3-view-dashboard --strict`
