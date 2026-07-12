use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::routing::get;
use minijinja::Environment;
use tower_http::services::ServeDir;

use crate::model::Reading;

pub mod handlers;
pub mod render;

/// Shared state for the web layer. Deliberately minimal for this slice
/// (design.md decision 1): reuses the same `Arc<Mutex<Option<Reading>>>`
/// already passed to the ingest loop rather than building the full
/// `watch`/`mpsc`-channel `AppState` sketched in `rewrite-plan.md` §5, which
/// has no consumer until slice-4 introduces `SetTarget`.
#[derive(Clone)]
pub struct AppState {
    pub latest: Arc<Mutex<Option<Reading>>>,
    pub brew_id: String,
    pub env: Arc<Environment<'static>>,
    /// Set by the ingest task once it starts running; read by `/healthz`
    /// as a liveness signal (design.md decision 4).
    pub ingest_alive: Arc<AtomicBool>,
}

/// Build the Axum router for the dashboard, status fragment, health check,
/// and static assets. Read-only this slice — no mutating routes (web-dashboard
/// spec: "Dashboard rendering is read-only in this capability").
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(handlers::dashboard))
        .route("/status", get(handlers::status))
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
        AppState {
            latest: Arc::new(Mutex::new(reading)),
            brew_id: "00-TEST-v00".to_string(),
            env: Arc::new(build_environment()),
            ingest_alive: Arc::new(AtomicBool::new(true)),
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
}
