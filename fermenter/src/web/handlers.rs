use std::sync::atomic::Ordering;

use axum::extract::State;
use axum::response::Html;
use axum::{Form, Json};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::error::Result;
use crate::model::Reading;
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

/// Context for `target_form.html`, shared by the `GET` (display) and `POST`
/// (re-render on validation failure, or confirm on success) handlers.
#[derive(Serialize)]
pub(crate) struct TargetFormContext {
    target: f64,
    error: Option<String>,
    confirmed: bool,
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
            confirmed: false,
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
) -> Result<Html<String>> {
    match validate_target(&form.target) {
        Ok(value) => {
            let previous = *state.target_tx.borrow();
            info!(previous, target = value, "target temperature accepted");
            if let Err(e) = state.target_tx.send(value) {
                warn!(error = %e, "no ingest receiver listening for target updates");
            }
            if let Err(e) = persist_target(state.store.as_ref(), value, &state.brew_id).await {
                warn!(error = %e, "failed to persist target temperature — continuing");
            }
            render(
                &state.env,
                "target_form.html",
                TargetFormContext {
                    target: value,
                    error: None,
                    confirmed: true,
                },
            )
        }
        Err(message) => {
            let current = *state.target_tx.borrow();
            render(
                &state.env,
                "target_form.html",
                TargetFormContext {
                    target: current,
                    error: Some(message),
                    confirmed: false,
                },
            )
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

    #[test]
    fn target_form_with_current_target_snapshot() {
        let env = build_environment();
        let ctx = TargetFormContext {
            target: 19.5,
            error: None,
            confirmed: false,
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
            confirmed: false,
        };
        let html = render(&env, "target_form.html", ctx).unwrap();
        insta::assert_snapshot!(html.0);
    }
}
