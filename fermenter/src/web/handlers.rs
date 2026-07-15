use std::sync::atomic::Ordering;

use axum::extract::State;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::{Form, Json};
use chrono::Local;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::brew_session::{persist_brew, validate_brew_id};
use crate::error::Result;
use crate::ingest;
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
    pub(crate) server_time: String,
}

pub(crate) fn status_context(state: &AppState) -> StatusContext {
    let reading = state.latest.lock().unwrap().clone();
    StatusContext {
        reading,
        brew_id: state.brew_tx.borrow().clone(),
        server_time: Local::now().format("%d-%b-%Y %H:%M:%S").to_string(),
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
            server_time: "14-Jul-2026 14:30:45".to_string(),
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
            server_time: "14-Jul-2026 14:30:45".to_string(),
        };
        let html = render(&env, "partials/status.html", ctx).unwrap();
        insta::assert_snapshot!(html.0);
    }

    #[test]
    fn dashboard_renders_snapshot() {
        let env = build_environment();
        let ctx = StatusContext {
            reading: Some(sample_reading()),
            brew_id: "00-TEST-v00".to_string(),
            server_time: "14-Jul-2026 14:30:45".to_string(),
        };
        let html = render(&env, "dashboard.html", ctx).unwrap();
        insta::assert_snapshot!(html.0);
    }

    #[test]
    fn status_fragment_includes_reason_code() {
        let env = build_environment();
        let ctx = StatusContext {
            reading: Some(sample_reading()),
            brew_id: "00-TEST-v00".to_string(),
            server_time: "14-Jul-2026 14:30:45".to_string(),
        };
        let html = render(&env, "partials/status.html", ctx).unwrap();
        assert!(html.0.contains("<dt>Reason</dt>"));
        assert!(html.0.contains("<dd>below-target</dd>"));
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
        let ctx = StatusContext {
            reading: Some(sample_reading()),
            brew_id: "00-TEST-v00".to_string(),
            server_time: "14-Jul-2026 14:30:45".to_string(),
        };
        let html = render(&env, "dashboard.html", ctx).unwrap();
        assert!(html.0.contains("<button"), "dashboard must use buttons");
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
            !html.0.contains("Back to dashboard"),
            "target form must not contain back link"
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
            !html.0.contains("Back to dashboard"),
            "brew form must not contain back link"
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
