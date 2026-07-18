## Why

The dashboard shows only the current controller reading, so an operator cannot
see whether a fermentation temperature is stable or trending away from its
target. The application already persists the relevant time-series data and can
use it to provide this operational history without a separate monitoring
system.

## What Changes

- Add a dashboard temperature-history chart for the active brew, displaying
  average fermenter, ambient, and target temperatures together over time.
- Let an operator choose one fixed chart window: 15 minutes, 1 hour, 3 hours,
  6 hours, 12 hours, 24 hours, 3 days, 7 days, or 14 days.
- Serve the chart as server-rendered SVG through a read-only, HTMX-polled
  dashboard fragment; selecting a new window refreshes the chart without a
  full page reload.
- Add bounded time-series range retrieval so the server can query the three
  persisted temperature series for the active brew and selected window.
- Scale the chart's Y axis from the temperatures returned for the selected
  window, label the time and temperature axes, use a distinct color for each
  series, and show an explicit no-history state when no readings exist.
- Include a legend that identifies the average fermenter, ambient, and target
  series by their chart colors.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `reading-history`: retrieve bounded, timestamped temperature history for a
  brew.
- `web-dashboard`: display and periodically refresh a selectable,
  server-rendered temperature-history chart.

## Impact

Changes affect the `TimeStore` abstraction and Redis implementation, the fake
store used by tests, Axum routes and handlers, and dashboard templates and
snapshots. The server will add SVG chart rendering support; the browser will
continue to use HTMX and will not require a client-side charting library.
