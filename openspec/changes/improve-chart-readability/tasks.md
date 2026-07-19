## 1. Chart Tick-Label Font Size — TDD

- [ ] 1.1 Record current chart snapshot as RED baseline (confirm existing `chart_fragment_with_history_snapshot` test passes before changes)
- [ ] 1.2 Add `.x_label_style(("sans-serif", 14).into_font())` and `.y_label_style(("sans-serif", 14).into_font())` to `configure_mesh()` in `TemperatureChart::svg()`
- [ ] 1.3 Run `INSTA_UPDATE=always cargo test -- chart_fragment_with_history` and verify the updated snapshot shows 14px tick labels in the SVG output
- [ ] 1.4 Run `cargo test -- chart` — all chart-related tests GREEN

## 2. Series and Legend Line Weight — TDD

- [ ] 2.1 Confirm existing `chart_fragment_with_history_snapshot` test is GREEN with the updated tick-label snapshot
- [ ] 2.2 Replace `&fermenter`, `&ambient`, `&target` with `ShapeStyle::from(color).stroke_width(2)` in all three `LineSeries::new()` calls
- [ ] 2.3 Replace `fermenter`/`ambient`/`target` with `ShapeStyle::from(color).stroke_width(2)` in the three `.legend()` closures and the legend-drawing loop `PathElement`
- [ ] 2.4 Run `INSTA_UPDATE=always cargo test -- chart_fragment_with_history` and verify the updated snapshot shows `stroke-width="2"` on series paths and legend lines
- [ ] 2.5 Run `cargo test -- chart` — all chart-related tests GREEN

## 3. Dashboard Polling Interval — TDD

- [ ] 3.1 Confirm existing `dashboard_with_no_reading_snapshot` (or equivalent dashboard-page insta test) passes before changes
- [ ] 3.2 Change `hx-trigger="every 5s"` to `hx-trigger="every 10s"` on the status-panel `<div>` and the chart `<div>` in `dashboard.html`
- [ ] 3.3 Run `INSTA_UPDATE=always cargo test -- dashboard` (or the specific dashboard-page snapshot test) and verify the updated snapshot shows `every 10s` in the HTMX triggers
- [ ] 3.4 Run `cargo test` — full test suite GREEN

## 4. Lint, Types, and Full Suite

- [ ] 4.1 Run `cargo fmt --check` — no formatting issues
- [ ] 4.2 Run `cargo clippy --all-targets -- -D warnings` — no clippy warnings
- [ ] 4.3 Run `cargo test` — full test suite GREEN

## 5. OpenSpec Validation

- [ ] 5.1 Run `openspec validate --changes improve-chart-readability --strict` — no validation errors
- [ ] 5.2 Run `openspec status --change "improve-chart-readability"` — confirms isComplete: true
