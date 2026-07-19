## Why

The dashboard temperature-history chart uses the `plotters` crate's default font size for axis tick labels (~10px) and the default 1px stroke for plotted series and legend lines. On the fermentation fridge's display these details are hard to read at a glance. Increasing tick-label size and line weight will make the chart more legible for day-to-day monitoring.

## What Changes

- Increase Y-axis and X-axis tick-label font size from the plotters default (~10px) to 14px in the server-rendered SVG chart.
- Increase plotted series line width from the default 1px to 2px for all three temperature series (average fermenter, ambient, target).
- Increase legend swatch line thickness from the default 1px to 2px so the legend matches the plotting area.
- Increase the dashboard polling interval from 5 seconds to 10 seconds for both the status panel and the temperature-history chart, reducing redundant server load.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `web-dashboard`: the "Display a selectable temperature-history chart" requirement now specifies 14px axis tick labels and 2px series/legend stroke width. The "Serve a live-polled current-state fragment" and "Select and refresh fixed chart windows" requirements now specify a 10-second polling interval.

## Impact

- `fermenter/src/web/handlers.rs` — `TemperatureChart::svg()` tick-label style and `ShapeStyle::stroke_width()` calls.
- `fermenter/templates/dashboard.html` — HTMX polling trigger changed from `every 5s` to `every 10s` on the status and chart elements.
- `fermenter/src/web/snapshots/*` — insta snapshots will need updating after tick-label and line-style changes.
