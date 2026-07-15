use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::routing::get;
use minijinja::Environment;
use tokio::sync::watch;
use tower_http::services::ServeDir;

use crate::model::Reading;
use crate::store::TimeStore;

pub mod handlers;
pub mod render;

/// Shared state for the web layer. `target_tx` (slice-4) sends the
/// operator-accepted desired target into the ingest side's reconcile check;
/// `brew_tx` (this slice) mirrors it exactly for the active brew identifier
/// (design.md decision 1) — both remain plain `watch` channels rather than
/// the `Command`/`mpsc` `AppState` `rewrite-plan.md` §5 sketches, per that
/// decision's rationale. `store` is shared so `/target` and `/brew` can both
/// persist accepted changes.
#[derive(Clone)]
pub struct AppState {
    pub latest: Arc<Mutex<Option<Reading>>>,
    pub env: Arc<Environment<'static>>,
    /// Set by the ingest task once it starts running; read by `/healthz`
    /// as a liveness signal (design.md decision 4).
    pub ingest_alive: Arc<AtomicBool>,
    /// Sends the operator-accepted desired target temperature to the
    /// ingest side's reconcile check (temperature-control capability).
    pub target_tx: watch::Sender<f64>,
    /// Sends the operator-accepted active brew identifier to the ingest
    /// side's per-reading tagging (brew-session capability).
    pub brew_tx: watch::Sender<String>,
    /// Shared store handle so `/target` and `/brew` can persist an accepted
    /// change (temperature-control / brew-session: "Persist and restore ...
    /// across restarts").
    pub store: Arc<dyn TimeStore>,
}

/// Build the Axum router for the dashboard, status fragment, target-setpoint
/// form, brew-identifier form, health check, and static assets. The
/// dashboard (`/`) and status fragment (`/status`) remain read-only;
/// `/target` (slice-4) and `/brew` (this slice) are the mutating routes
/// (web-dashboard spec: narrowed "read-only" requirement).
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(handlers::dashboard))
        .route("/status", get(handlers::status))
        .route(
            "/target",
            get(handlers::target_form).post(handlers::set_target),
        )
        .route("/brew", get(handlers::brew_form).post(handlers::set_brew))
        .route("/healthz", get(handlers::healthz))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state)
}

/// Build the MiniJinja `Environment` used to render templates.
///
/// Without the `embed` cargo feature (the default — plain `cargo run`/`cargo
/// test`): loads from the `templates/` directory at the crate root via a
/// path loader (design.md decision 2 of slice-3 — dev-mode only).
///
/// With `--features embed` (slice-7: deployment-packaging, used by the
/// release Docker image): templates are loaded eagerly from the bundle
/// baked into the binary at compile time by `build.rs`
/// (`minijinja_embed::embed_templates!`), so no `templates/` directory needs
/// to exist at runtime.
pub fn build_environment() -> Environment<'static> {
    let mut env = Environment::new();
    #[cfg(feature = "embed")]
    {
        minijinja_embed::load_templates!(env);
    }
    #[cfg(not(feature = "embed"))]
    {
        env.set_loader(minijinja::path_loader("templates"));
    }
    env
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Method, Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use super::*;

    fn test_state(reading: Option<Reading>) -> AppState {
        test_state_with_target(reading, 19.5)
    }

    fn test_state_with_target(reading: Option<Reading>, target: f64) -> AppState {
        test_state_with_target_and_brew(reading, target, "00-TEST-v00")
    }

    fn test_state_with_brew(reading: Option<Reading>, brew_id: &str) -> AppState {
        test_state_with_target_and_brew(reading, 19.5, brew_id)
    }

    fn test_state_with_target_and_brew(
        reading: Option<Reading>,
        target: f64,
        brew_id: &str,
    ) -> AppState {
        test_state_with_target_brew_and_store(
            reading,
            target,
            brew_id,
            Arc::new(crate::store::fake::FakeTimeStore::new()),
        )
    }

    fn test_state_with_target_brew_and_store(
        reading: Option<Reading>,
        target: f64,
        brew_id: &str,
        store: Arc<dyn TimeStore>,
    ) -> AppState {
        let (target_tx, _target_rx) = watch::channel(target);
        let (brew_tx, _brew_rx) = watch::channel(brew_id.to_string());
        AppState {
            latest: Arc::new(Mutex::new(reading)),
            env: Arc::new(build_environment()),
            ingest_alive: Arc::new(AtomicBool::new(true)),
            target_tx,
            brew_tx,
            store,
        }
    }

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

    async fn body_string(response: axum::response::Response) -> String {
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn status_returns_ok_with_latest_reading() {
        let router = build_router(test_state(Some(sample_reading())));

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_string(response).await;
        assert!(body.contains("18.7"));
        assert!(body.contains("Server time:"));
        assert!(body.contains("below-target"));
    }

    #[tokio::test]
    async fn status_returns_ok_with_no_reading() {
        let router = build_router(test_state(None));

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_string(response).await;
        assert!(body.contains("No reading available"));
        assert!(body.contains("Server time:"));
    }

    #[tokio::test]
    async fn dashboard_returns_ok_with_embedded_status_content() {
        let router = build_router(test_state(Some(sample_reading())));

        let response = router
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_string(response).await;
        assert!(body.contains("<html"));
        // Same current-state content the /status fragment renders.
        assert!(body.contains("18.7"));
        assert!(body.contains(r#"id="status""#));
        assert!(body.contains("Server time:"));
        assert!(body.contains("below-target"));
    }

    #[tokio::test]
    async fn no_route_accepts_mutating_methods() {
        let mutating = [Method::POST, Method::PUT, Method::PATCH, Method::DELETE];
        for path in ["/", "/status", "/healthz"] {
            for method in &mutating {
                let router = build_router(test_state(None));
                let response = router
                    .oneshot(
                        Request::builder()
                            .method(method.clone())
                            .uri(path)
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();
                assert_ne!(
                    response.status(),
                    StatusCode::OK,
                    "{method} {path} unexpectedly succeeded — dashboard routes must be read-only this slice"
                );
            }
        }
    }

    #[tokio::test]
    async fn healthz_reports_healthy_while_ingest_is_alive() {
        let router = build_router(test_state(Some(sample_reading())));

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_string(response).await;
        assert!(body.contains(r#""serial_connected":true"#));
    }

    #[tokio::test]
    async fn healthz_responds_before_any_reading_observed() {
        let router = build_router(test_state(None));

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn static_asset_is_served() {
        let router = build_router(test_state(None));

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/static/styles.css")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_string(response).await;
        assert!(body.contains("font-family"));
    }

    #[test]
    fn status_context_server_time_matches_dd_mmm_yyyy_hh_mm_ss() {
        // web-dashboard spec: the fragment includes a server-local-time
        // timestamp formatted `DD-MMM-YYYY HH:MM:SS`. Validated by hand
        // rather than a regex dep (design.md decision 5).
        let ctx = handlers::status_context(&test_state(None));
        let stamped = &ctx.server_time;
        let parts: Vec<&str> = stamped.split(' ').collect();
        assert_eq!(parts.len(), 2, "expected '<date> <time>', got {stamped}");
        let (date, time) = (parts[0], parts[1]);

        let date_parts: Vec<&str> = date.split('-').collect();
        assert_eq!(date_parts.len(), 3, "expected DD-MMM-YYYY, got {date}");
        let (day, month, year) = (date_parts[0], date_parts[1], date_parts[2]);
        assert_eq!(day.len(), 2, "day should be 2 digits, got {day}");
        assert!(
            day.chars().all(|c| c.is_ascii_digit()),
            "day should be digits, got {day}",
        );
        assert_eq!(month.len(), 3, "month should be 3 letters, got {month}");
        assert!(
            month.chars().next().is_some_and(|c| c.is_uppercase()),
            "month first letter should be uppercase, got {month}",
        );
        assert!(
            month.chars().skip(1).all(|c| c.is_lowercase()),
            "month remaining letters should be lowercase, got {month}",
        );
        assert_eq!(year.len(), 4, "year should be 4 digits, got {year}");
        assert!(
            year.chars().all(|c| c.is_ascii_digit()),
            "year should be digits, got {year}",
        );

        let time_parts: Vec<&str> = time.split(':').collect();
        assert_eq!(time_parts.len(), 3, "expected HH:MM:SS, got {time}");
        for tp in time_parts {
            assert_eq!(tp.len(), 2, "time component should be 2 digits, got {tp}");
            assert!(
                tp.chars().all(|c| c.is_ascii_digit()),
                "time component should be digits, got {tp}",
            );
        }
    }

    fn post_form(path: &str, body: &str) -> Request<Body> {
        Request::builder()
            .method(Method::POST)
            .uri(path)
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    #[tokio::test]
    async fn target_form_returns_ok_with_current_target() {
        let router = build_router(test_state_with_target(None, 19.5));

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/target")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_string(response).await;
        assert!(body.contains("19.5"));
    }

    #[tokio::test]
    async fn valid_post_target_updates_state_and_confirms() {
        let state = test_state_with_target(None, 19.5);
        let target_rx = state.target_tx.subscribe();
        let router = build_router(state);

        let response = router
            .oneshot(post_form("/target", "target=21.0"))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_string(response).await;
        assert!(body.contains("21"));
        assert_eq!(*target_rx.borrow(), 21.0);
    }

    #[tokio::test]
    async fn non_numeric_post_target_is_rejected_and_state_unchanged() {
        let state = test_state_with_target(None, 19.5);
        let target_rx = state.target_tx.subscribe();
        let router = build_router(state);

        let response = router
            .oneshot(post_form("/target", "target=not-a-number"))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_string(response).await;
        assert!(body.to_lowercase().contains("not a valid number"));
        assert_eq!(*target_rx.borrow(), 19.5);
    }

    #[tokio::test]
    async fn out_of_range_post_target_is_rejected_and_state_unchanged() {
        let state = test_state_with_target(None, 19.5);
        let target_rx = state.target_tx.subscribe();
        let router = build_router(state);

        let response = router
            .oneshot(post_form("/target", "target=100"))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_string(response).await;
        assert!(body.contains("must be between"));
        assert_eq!(*target_rx.borrow(), 19.5);
    }

    #[tokio::test]
    async fn post_target_is_a_deliberate_exception_to_the_read_only_guarantee() {
        // web-dashboard spec: the dashboard/status read-only requirement is
        // narrowed to those two routes; `/target` is explicitly permitted
        // to mutate state.
        let router = build_router(test_state_with_target(None, 19.5));

        let response = router
            .oneshot(post_form("/target", "target=20.0"))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn brew_form_returns_ok_with_current_brew_id() {
        let router = build_router(test_state_with_brew(None, "00-TEST-v00"));

        let response = router
            .oneshot(Request::builder().uri("/brew").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_string(response).await;
        assert!(body.contains("00-TEST-v00"));
    }

    #[tokio::test]
    async fn valid_post_brew_updates_state_and_confirms() {
        let state = test_state_with_brew(None, "00-TEST-v00");
        let brew_rx = state.brew_tx.subscribe();
        let router = build_router(state);

        let response = router
            .oneshot(post_form("/brew", "brew_id=01-IPA-v02"))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_string(response).await;
        assert!(body.contains("01-IPA-v02"));
        assert_eq!(*brew_rx.borrow(), "01-IPA-v02");
    }

    #[tokio::test]
    async fn valid_post_brew_is_reflected_by_subsequent_status_requests() {
        let state = test_state_with_brew(Some(sample_reading()), "00-TEST-v00");
        // Keep a receiver alive for the duration of the test — `watch::Sender::send`
        // silently no-ops (and leaves the value unchanged) once every receiver has
        // been dropped, which the test helpers' own internal receiver would be by
        // the time the router handles a request (see design.md's `watch::Sender`
        // risk note).
        let _brew_rx = state.brew_tx.subscribe();
        let router = build_router(state);

        router
            .clone()
            .oneshot(post_form("/brew", "brew_id=01-IPA-v02"))
            .await
            .unwrap();

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = body_string(response).await;
        assert!(body.contains("01-IPA-v02"));
    }

    #[tokio::test]
    async fn empty_post_brew_is_rejected_and_state_unchanged() {
        let state = test_state_with_brew(None, "00-TEST-v00");
        let brew_rx = state.brew_tx.subscribe();
        let router = build_router(state);

        let response = router
            .oneshot(post_form("/brew", "brew_id="))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_string(response).await;
        assert!(body.to_lowercase().contains("must not be empty"));
        assert_eq!(*brew_rx.borrow(), "00-TEST-v00");
    }

    #[tokio::test]
    async fn whitespace_only_post_brew_is_rejected_and_state_unchanged() {
        let state = test_state_with_brew(None, "00-TEST-v00");
        let brew_rx = state.brew_tx.subscribe();
        let router = build_router(state);

        let response = router
            .oneshot(post_form("/brew", "brew_id=%20%20%20"))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_string(response).await;
        assert!(body.to_lowercase().contains("must not be empty"));
        assert_eq!(*brew_rx.borrow(), "00-TEST-v00");
    }

    #[tokio::test]
    async fn switching_to_a_brew_with_existing_history_rehydrates_immediately() {
        // brew-session: "Switching to a brew with existing history shows
        // that history immediately".
        let store = crate::store::fake::FakeTimeStore::new();
        store.seed_reading(
            "01-IPA-v02",
            Reading {
                target: 18.0,
                average: 17.5,
                min: 17.0,
                max: 18.0,
                ambient: 19.0,
                action: "idle".to_string(),
                reason_code: String::new(),
                json_size: None,
            },
        );
        let state =
            test_state_with_target_brew_and_store(None, 19.5, "00-TEST-v00", Arc::new(store));
        let _brew_rx = state.brew_tx.subscribe();
        let router = build_router(state);

        router
            .clone()
            .oneshot(post_form("/brew", "brew_id=01-IPA-v02"))
            .await
            .unwrap();

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = body_string(response).await;
        assert!(body.contains("17.5"));
    }

    #[tokio::test]
    async fn switching_to_a_brew_with_no_history_shows_no_data_immediately() {
        // brew-session: "Switching to a brew with no history shows no data
        // immediately" — even though the prior brew had a reading displayed.
        let state = test_state_with_brew(Some(sample_reading()), "00-TEST-v00");
        let _brew_rx = state.brew_tx.subscribe();
        let router = build_router(state);

        router
            .clone()
            .oneshot(post_form("/brew", "brew_id=never-used-brew"))
            .await
            .unwrap();

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = body_string(response).await;
        assert!(body.contains("No reading available"));
    }
}
