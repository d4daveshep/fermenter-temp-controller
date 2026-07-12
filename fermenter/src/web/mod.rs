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

/// Shared state for the web layer. Grew by exactly one field this slice
/// (design.md decision 1): a `target_tx: watch::Sender<f64>` for the web
/// layer to push a new desired target into the ingest side, plus a shared
/// `store` handle so the `/target` handler can persist accepted changes —
/// still short of the full `Command`/`mpsc` `AppState` `rewrite-plan.md` §5
/// sketches, which slice-5's `SetBrewId` may justify building.
#[derive(Clone)]
pub struct AppState {
    pub latest: Arc<Mutex<Option<Reading>>>,
    pub brew_id: String,
    pub env: Arc<Environment<'static>>,
    /// Set by the ingest task once it starts running; read by `/healthz`
    /// as a liveness signal (design.md decision 4).
    pub ingest_alive: Arc<AtomicBool>,
    /// Sends the operator-accepted desired target temperature to the
    /// ingest side's reconcile check (temperature-control capability).
    pub target_tx: watch::Sender<f64>,
    /// Shared store handle so `/target` can persist an accepted change
    /// (temperature-control: "Persist and restore the target temperature").
    pub store: Arc<dyn TimeStore>,
}

/// Build the Axum router for the dashboard, status fragment, target-setpoint
/// form, health check, and static assets. The dashboard (`/`) and status
/// fragment (`/status`) remain read-only; `/target` is this slice's first
/// mutating route (web-dashboard spec: narrowed "read-only" requirement).
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(handlers::dashboard))
        .route("/status", get(handlers::status))
        .route(
            "/target",
            get(handlers::target_form).post(handlers::set_target),
        )
        .route("/healthz", get(handlers::healthz))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state)
}

/// Build the MiniJinja `Environment` used to render templates, loading from
/// the `templates/` directory at the crate root via a path loader
/// (design.md decision 2 — dev-mode only; embedding is a slice-7 concern).
pub fn build_environment() -> Environment<'static> {
    let mut env = Environment::new();
    env.set_loader(minijinja::path_loader("templates"));
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
        let (target_tx, _target_rx) = watch::channel(target);
        AppState {
            latest: Arc::new(Mutex::new(reading)),
            brew_id: "00-TEST-v00".to_string(),
            env: Arc::new(build_environment()),
            ingest_alive: Arc::new(AtomicBool::new(true)),
            target_tx,
            store: Arc::new(crate::store::fake::FakeTimeStore::new()),
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
}
