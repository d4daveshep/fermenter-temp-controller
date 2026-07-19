use std::sync::atomic::Ordering;

use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::{Form, Json};
use chrono::{DateTime, Duration, Local, Utc};
use plotters::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::brew_session::{persist_brew, validate_brew_id};
use crate::error::{AppError, Result};
use crate::ingest;
use crate::model::Reading;
use crate::reason_code;
use crate::store::TemperatureSample;
use crate::temperature_control::{persist_target, validate_target};

use super::AppState;
use super::render::render;

/// Context shared by the full dashboard page and the `/status` fragment, so
/// both render the same current-state markup from the same data
/// (web-dashboard spec: "Fragment content matches the dashboard's embedded
/// status block").
#[derive(Serialize)]
pub(crate) struct StatusContext {
    reading: Option<Reading>,
    reason_text: Option<String>,
    brew_id: String,
    pub(crate) server_time: String,
    chart_windows: Vec<ChartWindowOption>,
}

pub(crate) fn status_context(state: &AppState) -> StatusContext {
    let reading = state.latest.lock().unwrap().clone();
    let reason_text = reading
        .as_ref()
        .and_then(|r| reason_code::reason_text(&r.reason_code))
        .map(str::to_owned);
    StatusContext {
        reading,
        reason_text,
        brew_id: state.brew_tx.borrow().clone(),
        server_time: Local::now().format("%d-%b-%Y %H:%M:%S").to_string(),
        chart_windows: ChartWindow::ALL
            .into_iter()
            .map(ChartWindowOption::from)
            .collect(),
    }
}

/// `GET /` — full dashboard page.
pub(crate) async fn dashboard(State(state): State<AppState>) -> Result<Html<String>> {
    render(&state.env, "dashboard.html", status_context(&state))
}

/// `GET /status` — HTMX-polled current-state fragment.
pub(crate) async fn status(State(state): State<AppState>) -> Result<Html<String>> {
    render(&state.env, "partials/status.html", status_context(&state))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ChartWindow {
    FiveMinutes,
    FifteenMinutes,
    OneHour,
    ThreeHours,
    SixHours,
    TwelveHours,
    TwentyFourHours,
    ThreeDays,
    SevenDays,
    FourteenDays,
}

impl ChartWindow {
    const ALL: [Self; 10] = [
        Self::FiveMinutes,
        Self::FifteenMinutes,
        Self::OneHour,
        Self::ThreeHours,
        Self::SixHours,
        Self::TwelveHours,
        Self::TwentyFourHours,
        Self::ThreeDays,
        Self::SevenDays,
        Self::FourteenDays,
    ];

    fn from_query(value: Option<&str>) -> Self {
        match value {
            Some("5m") => Self::FiveMinutes,
            Some("1h") => Self::OneHour,
            Some("3h") => Self::ThreeHours,
            Some("6h") => Self::SixHours,
            Some("12h") => Self::TwelveHours,
            Some("24h") => Self::TwentyFourHours,
            Some("3d") => Self::ThreeDays,
            Some("7d") => Self::SevenDays,
            Some("14d") => Self::FourteenDays,
            _ => Self::FifteenMinutes,
        }
    }

    fn duration(self) -> Duration {
        match self {
            Self::FiveMinutes => Duration::minutes(5),
            Self::FifteenMinutes => Duration::minutes(15),
            Self::OneHour => Duration::hours(1),
            Self::ThreeHours => Duration::hours(3),
            Self::SixHours => Duration::hours(6),
            Self::TwelveHours => Duration::hours(12),
            Self::TwentyFourHours => Duration::hours(24),
            Self::ThreeDays => Duration::days(3),
            Self::SevenDays => Duration::days(7),
            Self::FourteenDays => Duration::days(14),
        }
    }

    fn bucket(self) -> Duration {
        match self {
            Self::FiveMinutes => Duration::seconds(15),
            Self::FifteenMinutes => Duration::minutes(1),
            Self::OneHour => Duration::minutes(2),
            Self::ThreeHours => Duration::minutes(5),
            Self::SixHours => Duration::minutes(10),
            Self::TwelveHours => Duration::minutes(15),
            Self::TwentyFourHours => Duration::minutes(30),
            Self::ThreeDays => Duration::hours(1),
            Self::SevenDays => Duration::hours(2),
            Self::FourteenDays => Duration::hours(4),
        }
    }

    fn value(self) -> &'static str {
        match self {
            Self::FiveMinutes => "5m",
            Self::FifteenMinutes => "15m",
            Self::OneHour => "1h",
            Self::ThreeHours => "3h",
            Self::SixHours => "6h",
            Self::TwelveHours => "12h",
            Self::TwentyFourHours => "24h",
            Self::ThreeDays => "3d",
            Self::SevenDays => "7d",
            Self::FourteenDays => "14d",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::FiveMinutes => "Last 5 minutes",
            Self::FifteenMinutes => "Last 15 minutes",
            Self::OneHour => "Last 1 hour",
            Self::ThreeHours => "Last 3 hours",
            Self::SixHours => "Last 6 hours",
            Self::TwelveHours => "Last 12 hours",
            Self::TwentyFourHours => "Last 24 hours",
            Self::ThreeDays => "Last 3 days",
            Self::SevenDays => "Last 7 days",
            Self::FourteenDays => "Last 14 days",
        }
    }
}

#[derive(Serialize)]
struct ChartWindowOption {
    value: &'static str,
    label: &'static str,
}

impl From<ChartWindow> for ChartWindowOption {
    fn from(window: ChartWindow) -> Self {
        Self {
            value: window.value(),
            label: window.label(),
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct ChartQuery {
    window: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct ChartContext {
    svg: String,
}

struct TemperatureChart<'a> {
    samples: &'a [TemperatureSample],
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

impl<'a> TemperatureChart<'a> {
    fn new(samples: &'a [TemperatureSample], start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self {
            samples,
            start,
            end,
        }
    }

    fn svg(&self) -> Result<String> {
        if self.samples.is_empty() {
            return Ok(
                "<p class=\"no-history\">No temperature history is available for this window.</p>"
                    .to_string(),
            );
        }

        let values = self
            .samples
            .iter()
            .flat_map(|sample| [sample.fermenter, sample.ambient, sample.target]);
        let (minimum, maximum) = values.fold((f64::INFINITY, f64::NEG_INFINITY), |range, value| {
            (range.0.min(value), range.1.max(value))
        });
        let spread = maximum - minimum;
        let margin = if spread == 0.0 {
            maximum.abs().mul_add(0.1, 1.0)
        } else {
            spread * 0.05
        };
        let y_min = minimum - margin;
        let y_max = maximum + margin;
        let mut svg = String::new();
        {
            let drawing = SVGBackend::with_string(&mut svg, (800, 550)).into_drawing_area();
            drawing.fill(&WHITE).map_err(Self::render_error)?;
            let (plot_area, legend_area) = drawing.split_vertically(455);

            let mut chart = ChartBuilder::on(&plot_area)
                .caption("Temperature history", ("sans-serif", 24))
                .margin(16)
                .x_label_area_size(52)
                .y_label_area_size(62)
                .build_cartesian_2d(self.start..self.end, y_min..y_max)
                .map_err(Self::render_error)?;

            chart
                .configure_mesh()
                .x_desc("Time")
                .y_desc("Temperature")
                .x_labels(5)
                .y_labels(5)
                .max_light_lines(4)
                .x_label_formatter(&|timestamp| {
                    timestamp
                        .with_timezone(&Local)
                        .format("%d %b %H:%M")
                        .to_string()
                })
                .y_label_formatter(&|temperature| format!("{temperature:.1}"))
                .x_label_style(("sans-serif", 14))
                .y_label_style(("sans-serif", 14))
                .light_line_style(RGBColor(220, 224, 230))
                .axis_style(BLACK.mix(0.7))
                .draw()
                .map_err(Self::render_error)?;

            let fermenter = ShapeStyle::from(RGBColor(0, 114, 178)).stroke_width(2);
            chart
                .draw_series(LineSeries::new(
                    self.samples
                        .iter()
                        .map(|sample| (sample.timestamp, sample.fermenter)),
                    fermenter,
                ))
                .map_err(Self::render_error)?
                .label("Average fermenter")
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], fermenter));

            let ambient = ShapeStyle::from(RGBColor(213, 94, 0)).stroke_width(2);
            chart
                .draw_series(LineSeries::new(
                    self.samples
                        .iter()
                        .map(|sample| (sample.timestamp, sample.ambient)),
                    ambient,
                ))
                .map_err(Self::render_error)?
                .label("Ambient")
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], ambient));

            let target = ShapeStyle::from(RGBColor(0, 158, 115)).stroke_width(2);
            chart
                .draw_series(LineSeries::new(
                    self.samples
                        .iter()
                        .map(|sample| (sample.timestamp, sample.target)),
                    target,
                ))
                .map_err(Self::render_error)?
                .label("Target")
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], target));

            let legend = Rectangle::new([(492, 12), (790, 82)], WHITE.mix(0.9).filled());
            legend_area.draw(&legend).map_err(Self::render_error)?;
            legend_area
                .draw(&Rectangle::new([(492, 12), (790, 82)], BLACK))
                .map_err(Self::render_error)?;
            for (index, (label, color)) in [
                ("Average fermenter", fermenter),
                ("Ambient", ambient),
                ("Target", target),
            ]
            .into_iter()
            .enumerate()
            {
                let y = 27 + (index as i32 * 20);
                legend_area
                    .draw(&PathElement::new(vec![(508, y), (532, y)], color))
                    .map_err(Self::render_error)?;
                legend_area
                    .draw(&Text::new(label, (542, y), ("sans-serif", 14).into_font()))
                    .map_err(Self::render_error)?;
            }
            drawing.present().map_err(Self::render_error)?;
        }
        Ok(svg)
    }

    fn render_error(error: impl std::fmt::Display) -> AppError {
        AppError::Render(error.to_string())
    }
}

/// `GET /chart` — HTMX-polled temperature-history fragment for the active brew.
pub(crate) async fn chart(
    State(state): State<AppState>,
    Query(query): Query<ChartQuery>,
) -> Result<Html<String>> {
    let window = ChartWindow::from_query(query.window.as_deref());
    let end = Utc::now();
    let start = end - window.duration();
    let brew_id = state.brew_tx.borrow().clone();
    let samples = state
        .store
        .temperature_history(&brew_id, start, end, window.bucket())
        .await?;
    render(
        &state.env,
        "partials/chart.html",
        ChartContext {
            svg: TemperatureChart::new(&samples, start, end).svg()?,
        },
    )
}

/// Context for `target_form.html`, shared by the `GET` (display) and `POST`
/// (re-render on validation failure) handlers.
#[derive(Serialize)]
pub(crate) struct TargetFormContext {
    target: f64,
    error: Option<String>,
}

/// `GET /target` — target-setpoint form pre-filled with the current target
/// (web-dashboard spec: "Form displays the current target temperature").
pub(crate) async fn target_form(State(state): State<AppState>) -> Result<Html<String>> {
    let current = *state.target_tx.borrow();
    render(
        &state.env,
        "target_form.html",
        TargetFormContext {
            target: current,
            error: None,
        },
    )
}

#[derive(Deserialize)]
pub(crate) struct TargetForm {
    target: String,
}

/// `POST /target` — validate and apply a new target temperature
/// (temperature-control spec: "Accept a new target temperature setpoint").
/// On success, updates the shared `watch` channel (read by the ingest
/// side's reconcile check) and persists the new `ControllerState`; a
/// persistence failure is logged and does not prevent confirming the
/// in-memory update, matching the ingest loop's existing
/// log-and-continue treatment of storage failures. On validation failure,
/// the form is redisplayed with an error and neither the watch channel nor
/// storage is touched.
pub(crate) async fn set_target(
    State(state): State<AppState>,
    Form(form): Form<TargetForm>,
) -> Result<Response> {
    match validate_target(&form.target) {
        Ok(value) => {
            let previous = *state.target_tx.borrow();
            info!(previous, target = value, "target temperature accepted");
            if let Err(e) = state.target_tx.send(value) {
                warn!(error = %e, "no ingest receiver listening for target updates");
            }
            let current_brew_id = state.brew_tx.borrow().clone();
            if let Err(e) = persist_target(state.store.as_ref(), value, &current_brew_id).await {
                warn!(error = %e, "failed to persist target temperature — continuing");
            }
            Ok(Redirect::to("/").into_response())
        }
        Err(message) => {
            let current = *state.target_tx.borrow();
            render(
                &state.env,
                "target_form.html",
                TargetFormContext {
                    target: current,
                    error: Some(message),
                },
            )
            .map(|h| h.into_response())
        }
    }
}

/// Context for `brew_form.html`, shared by the `GET` (display) and `POST`
/// (re-render on validation failure) handlers.
#[derive(Serialize)]
pub(crate) struct BrewFormContext {
    brew_id: String,
    error: Option<String>,
}

/// `GET /brew` — brew-identifier form pre-filled with the current brew
/// identifier (web-dashboard spec: "Form displays the current brew
/// identifier").
pub(crate) async fn brew_form(State(state): State<AppState>) -> Result<Html<String>> {
    let current = state.brew_tx.borrow().clone();
    render(
        &state.env,
        "brew_form.html",
        BrewFormContext {
            brew_id: current,
            error: None,
        },
    )
}

#[derive(Deserialize)]
pub(crate) struct BrewForm {
    brew_id: String,
}

/// `POST /brew` — validate and apply a new brew identifier
/// (brew-session spec: "Accept a new brew identifier at runtime"). On
/// success: updates the shared `watch` channel (read by the ingest side's
/// per-reading tagging), persists the new `ControllerState` sourcing the
/// paired `target_temp` from the live `target_tx` channel (design.md
/// decision 2), and immediately rehydrates `state.latest` from the new
/// brew's most recently persisted reading — or clears it to no-data if none
/// exists — so the dashboard never shows the previous brew's reading
/// mislabeled under the new identifier (design.md decision 4; brew-session:
/// "Rehydrate current state when the active brew changes"). A persistence
/// failure is logged and does not prevent confirming the in-memory update,
/// matching `/target`'s existing log-and-continue treatment. On validation
/// failure, the form is redisplayed with an error and neither the watch
/// channel, storage, nor current state is touched.
pub(crate) async fn set_brew(
    State(state): State<AppState>,
    Form(form): Form<BrewForm>,
) -> Result<Response> {
    match validate_brew_id(&form.brew_id) {
        Ok(value) => {
            let previous = state.brew_tx.borrow().clone();
            info!(previous, brew_id = %value, "brew identifier accepted");
            if let Err(e) = state.brew_tx.send(value.clone()) {
                warn!(error = %e, "no ingest receiver listening for brew updates");
            }

            let current_target_temp = *state.target_tx.borrow();
            if let Err(e) = persist_brew(state.store.as_ref(), &value, current_target_temp).await {
                warn!(error = %e, "failed to persist brew identifier — continuing");
            }

            let rehydrated = ingest::rehydrate_latest_reading(state.store.as_ref(), &value).await;
            *state.latest.lock().unwrap() = rehydrated;

            Ok(Redirect::to("/").into_response())
        }
        Err(message) => {
            let current = state.brew_tx.borrow().clone();
            render(
                &state.env,
                "brew_form.html",
                BrewFormContext {
                    brew_id: current,
                    error: Some(message),
                },
            )
            .map(|h| h.into_response())
        }
    }
}

#[derive(Serialize)]
pub(crate) struct HealthResponse {
    /// Whether the ingest task is alive and processing input. Named
    /// `serial_connected` to match the field `rewrite-plan.md` §7 uses for
    /// the eventual `/healthz` shape; a `redis_connected` field is
    /// deliberately deferred (design.md decision 4 / Open Questions).
    serial_connected: bool,
}

/// `GET /healthz` — liveness check (operational-health spec).
pub(crate) async fn healthz(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        serial_connected: state.ingest_alive.load(Ordering::Relaxed),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::web::build_environment;
    use chrono::TimeZone;

    fn sample_reading() -> Reading {
        Reading {
            target: 19.5,
            average: 18.7,
            min: 18.0,
            max: 19.0,
            ambient: 20.1,
            action: "heating".to_string(),
            reason_code: "RC3.1".to_string(),
            json_size: None,
        }
    }

    fn sample_context() -> StatusContext {
        let reading = sample_reading();
        let reason_text = crate::reason_code::reason_text(&reading.reason_code).map(str::to_owned);
        StatusContext {
            reading: Some(reading),
            reason_text,
            brew_id: "00-TEST-v00".to_string(),
            server_time: "14-Jul-2026 14:30:45".to_string(),
            chart_windows: ChartWindow::ALL
                .into_iter()
                .map(ChartWindowOption::from)
                .collect(),
        }
    }

    fn empty_context() -> StatusContext {
        StatusContext {
            reading: None,
            reason_text: None,
            brew_id: "00-TEST-v00".to_string(),
            server_time: "14-Jul-2026 14:30:45".to_string(),
            chart_windows: ChartWindow::ALL
                .into_iter()
                .map(ChartWindowOption::from)
                .collect(),
        }
    }

    fn chart_samples() -> Vec<TemperatureSample> {
        vec![
            TemperatureSample {
                timestamp: Utc.timestamp_millis_opt(1_000).unwrap(),
                fermenter: 18.0,
                ambient: 20.0,
                target: 19.0,
            },
            TemperatureSample {
                timestamp: Utc.timestamp_millis_opt(61_000).unwrap(),
                fermenter: 18.5,
                ambient: 20.5,
                target: 19.0,
            },
        ]
    }

    fn test_chart(samples: &[TemperatureSample]) -> TemperatureChart<'_> {
        TemperatureChart::new(
            samples,
            Utc.timestamp_millis_opt(0).unwrap(),
            Utc.timestamp_millis_opt(300_000).unwrap(),
        )
    }

    #[test]
    fn chart_windows_have_documented_durations_and_buckets() {
        let expected = [
            ("5m", Duration::minutes(5), Duration::seconds(15)),
            ("15m", Duration::minutes(15), Duration::minutes(1)),
            ("1h", Duration::hours(1), Duration::minutes(2)),
            ("3h", Duration::hours(3), Duration::minutes(5)),
            ("6h", Duration::hours(6), Duration::minutes(10)),
            ("12h", Duration::hours(12), Duration::minutes(15)),
            ("24h", Duration::hours(24), Duration::minutes(30)),
            ("3d", Duration::days(3), Duration::hours(1)),
            ("7d", Duration::days(7), Duration::hours(2)),
            ("14d", Duration::days(14), Duration::hours(4)),
        ];

        for (value, duration, bucket) in expected {
            let window = ChartWindow::from_query(Some(value));
            assert_eq!(window.value(), value);
            assert_eq!(window.duration(), duration);
            assert_eq!(window.bucket(), bucket);
            assert!(!window.label().is_empty());
        }
        assert_eq!(ChartWindow::from_query(None), ChartWindow::FifteenMinutes);
        assert_eq!(
            ChartWindow::from_query(Some("not-a-window")),
            ChartWindow::FifteenMinutes
        );
    }

    #[test]
    fn temperature_chart_renders_labeled_colored_series_and_bounds() {
        let svg = test_chart(&chart_samples()).svg().unwrap();

        assert!(svg.contains("Time\n</text>"));
        assert!(svg.contains("Temperature\n</text>"));
        for (color, name) in [
            ("#0072B2", "Average fermenter"),
            ("#D55E00", "Ambient"),
            ("#009E73", "Target"),
        ] {
            assert!(svg.contains(&format!("stroke=\"{color}\"")));
            assert!(svg.contains(name));
        }
        assert!(svg.contains("18.0\n</text>"));
        assert!(svg.contains("20.0\n</text>"));
    }

    #[test]
    fn temperature_chart_renders_a_plotting_grid() {
        let svg = test_chart(&chart_samples()).svg().unwrap();

        assert!(svg.matches("stroke=\"#DCE0E6\"").count() >= 4);
        assert!(svg.contains("01 Jan 12:03"));
    }

    #[test]
    fn temperature_chart_uses_plotters_paths_for_mesh_and_line_series() {
        let svg = test_chart(&chart_samples()).svg().unwrap();

        assert!(svg.contains("font-family=\"sans-serif\""));
        assert!(svg.contains("<polyline"));
        assert!(svg.contains("Average fermenter"));
        assert!(svg.contains("Ambient"));
        assert!(svg.contains("Target"));
    }

    #[test]
    fn temperature_chart_reserves_space_for_external_legend_and_calendar_ticks() {
        let svg = test_chart(&chart_samples()).svg().unwrap();

        assert!(svg.contains("<svg width=\"800\" height=\"550\""));
        assert!(svg.contains("01 Jan 12:01"));
        assert!(svg.contains("01 Jan 12:03"));
    }

    #[test]
    fn temperature_chart_expands_constant_range_and_reports_empty_history() {
        let sample = TemperatureSample {
            timestamp: Utc.timestamp_millis_opt(1_000).unwrap(),
            fermenter: 19.0,
            ambient: 19.0,
            target: 19.0,
        };
        let svg = test_chart(&[sample]).svg().unwrap();
        assert!(svg.contains("19.0\n</text>"));
        assert!(
            test_chart(&[])
                .svg()
                .unwrap()
                .contains("No temperature history")
        );
    }

    #[test]
    fn chart_fragment_with_history_snapshot() {
        let env = build_environment();
        let html = render(
            &env,
            "partials/chart.html",
            ChartContext {
                svg: test_chart(&chart_samples()).svg().unwrap(),
            },
        )
        .unwrap();
        insta::assert_snapshot!(html.0);
    }

    #[test]
    fn chart_fragment_without_history_snapshot() {
        let env = build_environment();
        let html = render(
            &env,
            "partials/chart.html",
            ChartContext {
                svg: test_chart(&[]).svg().unwrap(),
            },
        )
        .unwrap();
        insta::assert_snapshot!(html.0);
    }

    #[test]
    fn status_fragment_with_reading_snapshot() {
        let env = build_environment();
        let ctx = sample_context();
        let html = render(&env, "partials/status.html", ctx).unwrap();
        insta::assert_snapshot!(html.0);
    }

    #[test]
    fn status_fragment_without_reading_snapshot() {
        let env = build_environment();
        let ctx = empty_context();
        let html = render(&env, "partials/status.html", ctx).unwrap();
        insta::assert_snapshot!(html.0);
    }

    #[test]
    fn dashboard_renders_snapshot() {
        let env = build_environment();
        let ctx = sample_context();
        let html = render(&env, "dashboard.html", ctx).unwrap();
        insta::assert_snapshot!(html.0);
    }

    #[test]
    fn status_fragment_includes_reason_code() {
        let env = build_environment();
        let ctx = sample_context();
        let html = render(&env, "partials/status.html", ctx).unwrap();
        assert!(html.0.contains("<dt>Reason</dt>"));
        assert!(html.0.contains("<dd>RC3.1"));
        assert!(
            html.0
                .contains("REST-&gt;REST because we are in the target range")
        );
    }

    #[test]
    fn status_fragment_omits_reason_text_for_unmapped_codes() {
        let env = build_environment();

        for reason_code in ["RC_FOO", "", "RC_ERR"] {
            let mut ctx = sample_context();
            ctx.reading.as_mut().unwrap().reason_code = reason_code.to_string();
            ctx.reason_text = None;

            let html = render(&env, "partials/status.html", ctx).unwrap();
            assert!(
                html.0
                    .contains(&format!("<dt>Reason</dt><dd>{reason_code}</dd>")),
                "reason row did not contain only {reason_code:?}: {}",
                html.0
            );
        }
    }

    #[test]
    fn target_form_with_current_target_snapshot() {
        let env = build_environment();
        let ctx = TargetFormContext {
            target: 19.5,
            error: None,
        };
        let html = render(&env, "target_form.html", ctx).unwrap();
        insta::assert_snapshot!(html.0);
    }

    #[test]
    fn target_form_with_error_snapshot() {
        let env = build_environment();
        let ctx = TargetFormContext {
            target: 19.5,
            error: Some("'abc' is not a valid number".to_string()),
        };
        let html = render(&env, "target_form.html", ctx).unwrap();
        insta::assert_snapshot!(html.0);
    }

    #[test]
    fn brew_form_with_current_brew_snapshot() {
        let env = build_environment();
        let ctx = BrewFormContext {
            brew_id: "00-TEST-v00".to_string(),
            error: None,
        };
        let html = render(&env, "brew_form.html", ctx).unwrap();
        insta::assert_snapshot!(html.0);
    }

    #[test]
    fn dashboard_uses_buttons_not_links_for_navigation() {
        let env = build_environment();
        let ctx = sample_context();
        let html = render(&env, "dashboard.html", ctx).unwrap();
        assert!(html.0.contains("<button"), "dashboard must use buttons");
        assert!(html.0.contains(r#"hx-get="/chart""#));
        assert!(html.0.contains(r#"hx-trigger="load, every 10s""#));
        assert!(html.0.contains(r##"hx-include="#chart-window""##));
        assert!(
            !html.0.contains("<a href=\"/target\">"),
            "dashboard must not use <a> link for target navigation"
        );
        assert!(
            !html.0.contains("<a href=\"/brew\">"),
            "dashboard must not use <a> link for brew navigation"
        );
    }

    #[test]
    fn form_templates_do_not_contain_back_link_or_confirmed_block() {
        let env = build_environment();
        let html = render(
            &env,
            "target_form.html",
            TargetFormContext {
                target: 19.5,
                error: None,
            },
        )
        .unwrap();
        assert!(
            html.0.contains("<button type=\"submit\">Cancel</button>"),
            "target form must contain cancel button"
        );
        assert!(
            !html.0.contains(r#"class="confirmed""#),
            "target form must not contain confirmed block"
        );
        let html = render(
            &env,
            "brew_form.html",
            BrewFormContext {
                brew_id: "00-TEST-v00".to_string(),
                error: None,
            },
        )
        .unwrap();
        assert!(
            html.0.contains("<button type=\"submit\">Cancel</button>"),
            "brew form must contain cancel button"
        );
        assert!(
            !html.0.contains(r#"class="confirmed""#),
            "brew form must not contain confirmed block"
        );
    }

    #[test]
    fn brew_form_with_error_snapshot() {
        let env = build_environment();
        let ctx = BrewFormContext {
            brew_id: "00-TEST-v00".to_string(),
            error: Some("brew identifier must not be empty".to_string()),
        };
        let html = render(&env, "brew_form.html", ctx).unwrap();
        insta::assert_snapshot!(html.0);
    }
}
