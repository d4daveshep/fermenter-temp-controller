## 1. Temperature History Storage -- TDD

- [ ] 1.1 Write `FakeTimeStore` tests (RED) for a timestamped, bounded history query: complete average fermenter, ambient, and target samples are returned within the requested range; empty ranges return no samples; and data from another brew is excluded.
- [ ] 1.2 Add timestamped history types and the bounded range-query method to `TimeStore`; extend `FakeTimeStore` with timestamped fixtures and query behavior (GREEN for 1.1).
- [ ] 1.3 Write Redis-backed integration tests (RED) verifying that each returned sample contains all three series, aggregation bounds a long-range result, and history remains isolated by brew identifier.
- [ ] 1.4 Implement aligned `TS.RANGE ... AGGREGATION avg <bucket-ms>` queries in `RedisTimeStore`, omitting incomplete timestamp buckets (GREEN for 1.3).

## 2. Chart Model and SVG Rendering -- TDD

- [ ] 2.1 Write unit tests (RED) for `ChartWindow`: the nine supported values parse to their documented durations and aggregation intervals; missing or invalid values use the 15-minute default.
- [ ] 2.2 Implement the closed `ChartWindow` enum and its duration, display-label, and aggregation-interval behavior (GREEN for 2.1).
- [ ] 2.3 Write renderer tests (RED) using representative timestamped samples: the SVG labels axes `Time` and `Temperature`, renders three distinct stable series colors, maps those colors to the correct names in its legend, and Y-axis bounds include all values.
- [ ] 2.4 Write renderer tests (RED) for a constant-valued history producing a non-zero Y-axis span and an empty history producing the explicit no-history fragment.
- [ ] 2.5 Build the chart view model and server-side inline SVG renderer, including data-derived Y-axis margins, constant-range expansion, local-time X-axis labels, lines, legend, and no-history markup (GREEN for 2.3 and 2.4).

## 3. Chart Route and Dashboard Integration -- TDD

- [ ] 3.1 Write `tower::oneshot` HTTP tests (RED) for `GET /chart`: a selected supported window returns the active brew's chart, invalid and missing windows return the 15-minute chart, an empty selected window returns no-history markup, and a non-GET request cannot mutate state.
- [ ] 3.2 Add the read-only `GET /chart` handler and route, querying the active brew and rendering the chart fragment (GREEN for 3.1).
- [ ] 3.3 Write `insta` snapshot tests (RED) for the chart fragment with history and without history, and a dashboard snapshot/assertion covering the window selector plus HTMX initial-load, selection-change, and periodic requests that include the selected window.
- [ ] 3.4 Add the chart fragment template, dashboard window selector, HTMX attributes, and chart styles while preserving the current-status polling behavior (GREEN for 3.3).
- [ ] 3.5 Review and accept intentional `insta` snapshot updates.

## 4. Verification

- [ ] 4.1 Run `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` from `fermenter/`.
- [ ] 4.2 Run `cargo test` from `fermenter/`, including the Redis testcontainers integration tests.
- [ ] 4.3 Run `openspec validate dashboard-temperature-chart --strict`.
