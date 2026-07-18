## Context

Redis already stores separate `fermenter` (average), `ambient`, and `target`
time series for each brew, but `TimeStore` exposes only the most recent full
reading. The dashboard renders current state and polls `/status` every five
seconds; it has no historical query or chart surface. This change adds a
read-only history path while preserving the established Axum, HTMX, MiniJinja,
and Redis TimeSeries architecture.

## Goals / Non-Goals

**Goals:**

- Retrieve the three persisted temperature series for the active brew over one
  supported fixed time window.
- Render a self-contained, labeled SVG chart on the server, with time on the X
  axis and a data-derived temperature range on the Y axis.
- Refresh the selected chart through HTMX without a full page reload.
- Bound response size and rendering work for long windows.
- Keep storage and HTTP behavior testable through `FakeTimeStore` and router
  tests without Redis or a browser.

**Non-Goals:**

- Client-side chart libraries, JavaScript chart calculation, image files, or
  interactive pan and zoom.
- Arbitrary date ranges, custom intervals, comparison between brews, or
  changing the active brew from the chart.
- Altering the firmware serial protocol or persisted series schema.

## Decisions

### 1. Serve server-rendered inline SVG from a chart fragment endpoint

Add a read-only `GET /chart?window=<window>` handler that queries the active
brew and renders a MiniJinja chart fragment containing inline SVG. The
dashboard includes a window selector and an HTMX target that requests this
endpoint on initial load, on window change, and at the existing five-second
poll interval.

Inline SVG keeps the chart server-prepared as requested, works with HTMX's
HTML replacement, remains sharp on different displays, and can include
accessible text labels without a separate asset endpoint. A PNG endpoint was
rejected because it needs raster rendering and provides less useful markup;
client-side charting was rejected because it would move graph preparation to
the browser.

### 2. Model windows as a closed server-side enum

Use a `ChartWindow` enum to accept only `5m`, `15m`, `1h`, `3h`, `6h`, `12h`,
`24h`, `3d`, `7d`, and `14d`. The enum provides its duration, display label,
and an aggregation bucket size. The 5-minute window uses 15-second buckets so
newly started test servers show graphable data promptly. Missing or invalid
query values resolve to the default 15-minute window, ensuring polling cannot
create unbounded storage queries.

An arbitrary duration query parameter was rejected because it makes query
cost and the UI contract unbounded.

### 3. Extend TimeStore with an aligned, bounded history query

Add timestamped chart-point types and a `temperature_history` method to
`TimeStore`, accepting a brew ID, start timestamp, end timestamp, and bucket
duration. `RedisTimeStore` queries each of the three known series with
`TS.RANGE ... ALIGN 0 AGGREGATION avg <bucket-ms>`, aligns returned buckets by
timestamp, and returns only complete points. Epoch-aligned buckets stay stable
when the dashboard polls a rolling time window. The selected buckets cap each
query to a practical chart-width-sized number of points across every window.

The fake store retains timestamped chart samples and applies the same bounds,
so tests can validate handler behavior without Redis. Returning raw values was
rejected because long windows can create very large SVG responses and exceed
the chart's useful visual resolution.

### 4. Use Plotters to render the server-side SVG

Add `plotters` with only its SVG backend enabled. The chart handler derives the
Y range from all three values, applies a visual margin, and expands a constant
range so the coordinate range remains valid. It passes the selected window's
start and end timestamps and the calculated Y range to Plotters'
`ChartBuilder`.

Render into an in-memory string using `SVGBackend::with_string`, then return
that string through the existing chart fragment. Configure the Plotters mesh
with labeled X/Y axes, readable server-local timestamp formatting, and major
gridlines; draw the average fermenter, ambient, and target `LineSeries` with
stable colors. Use Plotters' chrono coordinate support so key points align to
calendar boundaries, selecting minute, quarter-hour, half-hour, or hourly
intervals appropriate to the selected time window. Reserve a lower drawing
area beneath the X axis and render the legend box at its right edge, outside
the plotting area.

Plotters replaces the hand-built coordinates, tick selection, SVG markup, and
legend layout. It preserves server-side SVG and avoids a browser dependency.
Keeping the custom renderer was rejected after live data showed it did not
provide the mature plotting behavior expected from a conventional charting
tool. A fixed temperature range remains rejected because it would obscure
small but operationally important variation.

### 5. No data is a normal chart response

When the active brew has no complete samples in a selected period, `/chart`
returns a successful fragment with an explicit no-history message and retains
the window selector. Store errors continue through the application error path
rather than being represented as no data, so persistence faults remain
observable.

## Risks / Trade-offs

- [Risk] Redis's three series can have slightly different write timestamps,
  leaving no complete raw point at a timestamp -> aggregation buckets and
  timestamp alignment produce stable chart samples; incomplete buckets are
  omitted.
- [Risk] Five-second chart polling adds Redis reads and SVG rendering -> the
  closed windows and aggregation cap point counts; it is acceptable for the
  single-operator dashboard.
- [Risk] A constant temperature range produces zero-height geometry -> expand
  the calculated range before mapping points to SVG coordinates.
- [Risk] Server-local time axis labels can differ across deployments -> labels
  intentionally follow the dashboard's existing server-local-time convention.
- [Risk] Plotters adds a rendering dependency and increases the binary size ->
  enable only its SVG backend and verify the release image build before
  deployment.

## Migration Plan

Deploy the application with the new read-only route and templates. Existing
Redis series require no migration because their keys and values already match
the query model. Rolling back removes the route and dashboard markup; retained
series stay compatible with the prior application.

## Open Questions

None.
