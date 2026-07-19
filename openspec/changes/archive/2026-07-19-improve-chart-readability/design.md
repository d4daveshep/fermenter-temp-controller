## Context

The dashboard temperature-history chart is rendered server-side by the `plotters` v0.3.7 SVG backend inside `TemperatureChart::svg()` in `fermenter/src/web/handlers.rs`. Currently the axis tick labels use plotters' default font size (~10px) and all drawn lines (series, legend swatches) use the default 1px stroke. This works but is hard to read on the fermentation fridge's display at a distance.

## Goals / Non-Goals

**Goals:**
- Increase axis tick-label font size from the plotters default to 14px.
- Increase plotted series line width from 1px to 2px for all three temperature series.
- Increase legend swatch line thickness from 1px to 2px.
- Increase the dashboard polling interval from 5 seconds to 10 seconds for both the status panel and the chart.

**Non-Goals:**
- Changing caption or axis-description font sizes.
- Changing chart dimensions, colors, grid styles, or layout.
- Changing time windows or other HTMX swap behavior beyond the polling interval.

## Decisions

### D1: Use `.x_label_style(("sans-serif", 14))` / `.y_label_style(("sans-serif", 14))` on the mesh configurator

**Choice:** Call `.x_label_style()` and `.y_label_style()` on the `MeshStyle` builder returned by `chart.configure_mesh()`, passing `("sans-serif", 14).into_font()`.

**Rationale:** `plotters` exposes these methods explicitly for per-axis tick-label styling. They integrate directly with the existing `configure_mesh()` chain without requiring a separate font-registry or layout recalculation. Font family "sans-serif" is already used for the caption and legend and keeps visual consistency.

**Alternatives considered:**
- A single `.label_style()` call — would style both axes identically, but explicit per-axis calls are clearer and leave room for future divergence.
- CSS-based font scaling (`#chart svg text { font-size: ... }`) — would affect every text element (caption, descriptions, legend) indiscriminately, which is not desired.

### D2: Use `ShapeStyle::from(color).stroke_width(2)` for all line drawing

**Choice:** Construct a `ShapeStyle` from each `RGBColor` and call `.stroke_width(2)` on it, passing the result to `LineSeries::new()` and `PathElement::new()`.

**Rationale:** `plotters` uses `ShapeStyle` as the single object representing fill, stroke, and width. `RGBColor` implements `Into<ShapeStyle>` with stroke-width 1. Calling `.stroke_width(2)` on the `ShapeStyle` bumps the width without affecting color or fill. This applies consistently to `LineSeries` (plot lines) and `PathElement` (legend swatches).

**Alternatives considered:**
- Hardcoding `2` as a named constant — unnecessary abstraction for a single value used in one method body.
- Different widths per series (e.g., thicker fermenter line) — out of scope; adds visual complexity without a clear use case.

### D3: Change HTMX polling triggers from `every 5s` to `every 10s`

**Choice:** Update the `hx-trigger` attribute on the status-panel `<div>` and the chart `<div>` in `dashboard.html` from `every 5s` to `every 10s`.

**Rationale:** A 5-second poll rate is unnecessarily aggressive for a fermentation temperature controller where readings change slowly. Doubling to 10 seconds halves server load (two GET requests per poll period) while still providing more-than-adequate UI responsiveness. The `hx-trigger` attribute is the single source of truth for HTMX polling; changing it in the template is the only change needed.

**Alternatives considered:**
- Configurable interval via Rust constant or env var — adds indirection for a value that is unlikely to need further tuning; if configurability is needed later it can be added then.
- Asymmetric rates (status faster than chart) — the two fragments are polled together on page load and serve related data; keeping them in sync avoids confusing discrepancies.

## Risks / Trade-offs

- [Risk] The tick-label size increase from ~10px to 14px adds ~4px per label, which may cause overlap on very narrow time windows (e.g., 5-minute window with 5 labels could push X-axis labels closer together). → **Mitigation:** The chart canvas is 800px wide with 5 labels, providing ample horizontal space; the 14px labels have been verified to fit at this density.
- [Risk] 2px stroke width on the fermenter line could visually obscure the ambient or target lines if they overlap. → **Mitigation:** All three series share the same 2px width, so overlap would affect the topmost series's visibility regardless of stroke width. Colors are distinct (blue, orange, teal), providing adequate differentiation.
