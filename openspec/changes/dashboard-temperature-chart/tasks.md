## 1. Temperature History Storage -- TDD

- [x] 1.1 Write `FakeTimeStore` tests (RED) for a timestamped, bounded history query: complete average fermenter, ambient, and target samples are returned within the requested range; empty ranges return no samples; and data from another brew is excluded.
- [x] 1.2 Add timestamped history types and the bounded range-query method to `TimeStore`; extend `FakeTimeStore` with timestamped fixtures and query behavior (GREEN for 1.1).
- [x] 1.3 Write Redis-backed integration tests (RED) verifying that each returned sample contains all three series, aggregation bounds a long-range result, and history remains isolated by brew identifier.
- [x] 1.4 Implement aligned `TS.RANGE ... AGGREGATION avg <bucket-ms>` queries in `RedisTimeStore`, omitting incomplete timestamp buckets (GREEN for 1.3).

## 2. Chart Model and SVG Rendering -- TDD

- [x] 2.1 Write unit tests (RED) for `ChartWindow`: the ten supported values parse to their documented durations and aggregation intervals; missing or invalid values use the 15-minute default.
- [x] 2.2 Implement the closed `ChartWindow` enum and its duration, display-label, and aggregation-interval behavior (GREEN for 2.1).
- [x] 2.3 Write renderer tests (RED) using representative timestamped samples: the SVG labels axes `Time` and `Temperature`, renders three distinct stable series colors, maps those colors to the correct names in its legend, and Y-axis bounds include all values.
- [x] 2.4 Write renderer tests (RED) for a constant-valued history producing a non-zero Y-axis span and an empty history producing the explicit no-history fragment.
- [x] 2.5 Build the chart view model and server-side inline SVG renderer, including data-derived Y-axis margins, constant-range expansion, local-time X-axis labels, lines, legend, and no-history markup (GREEN for 2.3 and 2.4).

## 3. Chart Route and Dashboard Integration -- TDD

- [x] 3.1 Write `tower::oneshot` HTTP tests (RED) for `GET /chart`: a selected supported window returns the active brew's chart, invalid and missing windows return the 15-minute chart, an empty selected window returns no-history markup, and a non-GET request cannot mutate state.
- [x] 3.2 Add the read-only `GET /chart` handler and route, querying the active brew and rendering the chart fragment (GREEN for 3.1).
- [x] 3.3 Write `insta` snapshot tests (RED) for the chart fragment with history and without history, and a dashboard snapshot/assertion covering the window selector plus HTMX initial-load, selection-change, and periodic requests that include the selected window.
- [x] 3.4 Add the chart fragment template, dashboard window selector, HTMX attributes, and chart styles while preserving the current-status polling behavior (GREEN for 3.3).
- [x] 3.5 Review and accept intentional `insta` snapshot updates.

## 4. Verification

- [x] 4.1 Run `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` from `fermenter/`.
- [x] 4.2 Run `cargo test` from `fermenter/`, including the Redis testcontainers integration tests.
- [x] 4.3 Run `openspec validate dashboard-temperature-chart --strict`.

## 5. Last Five Minutes Window -- TDD

- [x] 5.1 Extend the `ChartWindow` unit test (RED) for the `5m` value, then add the selectable 5-minute window with a 15-second aggregation interval (GREEN).

## 6. Stable Rolling-Window Aggregation -- TDD

- [x] 6.1 Add a Redis integration test (RED) showing that consecutive rolling history queries with the same samples must return the same aggregation buckets, then anchor `TS.RANGE` aggregation at the epoch (GREEN).

## 7. Conventional Time-Series Rendering -- TDD

- [x] 7.1 Add a renderer test (RED) for major plotting-grid lines and fixed-window X coordinates, then render five major ticks and gridlines per axis while mapping samples against the selected window (GREEN).

## 8. Plotters SVG Migration -- TDD

- [x] 8.1 Add `plotters` with only the SVG backend enabled; write a focused renderer test (RED) that expects Plotters-generated axes, mesh, three labeled line series, and a legend for representative temperature history.
- [x] 8.2 Replace the custom SVG coordinate, tick, grid, and legend generation with an in-memory `SVGBackend::with_string` + `ChartBuilder` renderer that preserves auto-scaled Y ranges, fixed selected-window X ranges, server-local time labels, and no-history output (GREEN for 8.1).
- [x] 8.3 Update chart styles and snapshots for Plotters SVG output, then run formatting, Clippy, the full test suite, and the embedded release build.
