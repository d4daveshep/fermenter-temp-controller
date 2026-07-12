use std::sync::atomic::Ordering;

use axum::Json;
use axum::extract::State;
use axum::response::Html;
use serde::Serialize;

use crate::error::Result;
use crate::model::Reading;

use super::AppState;
use super::render::render;

/// Context shared by the full dashboard page and the `/status` fragment, so
/// both render the same current-state markup from the same data
/// (web-dashboard spec: "Fragment content matches the dashboard's embedded
/// status block").
#[derive(Serialize)]
pub(crate) struct StatusContext {
    reading: Option<Reading>,
    brew_id: String,
}

fn status_context(state: &AppState) -> StatusContext {
    let reading = state.latest.lock().unwrap().clone();
    StatusContext {
        reading,
        brew_id: state.brew_id.clone(),
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

    fn sample_reading() -> Reading {
        Reading {
            target: 19.5,
            average: 18.7,
            min: 18.0,
            max: 19.0,
            ambient: 20.1,
            action: "heating".to_string(),
            reason_code: "below-target".to_string(),
            json_size: None,
        }
    }

    #[test]
    fn status_fragment_with_reading_snapshot() {
        let env = build_environment();
        let ctx = StatusContext {
            reading: Some(sample_reading()),
            brew_id: "00-TEST-v00".to_string(),
        };
        let html = render(&env, "partials/status.html", ctx).unwrap();
        insta::assert_snapshot!(html.0);
    }

    #[test]
    fn status_fragment_without_reading_snapshot() {
        let env = build_environment();
        let ctx = StatusContext {
            reading: None,
            brew_id: "00-TEST-v00".to_string(),
        };
        let html = render(&env, "partials/status.html", ctx).unwrap();
        insta::assert_snapshot!(html.0);
    }
}
