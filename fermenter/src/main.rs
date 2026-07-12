//! Thin binary entry point — all logic lives in the `fermenter` library
//! (`src/lib.rs`) so it's usable from `tests/` integration tests. Module
//! unit tests (config, model, ingest, serial, store) live in the library
//! and run via `cargo test --lib` (or plain `cargo test`, which also runs
//! this binary's own tests and the `tests/` integration suite).
//! `cargo test --bin fermenter` only runs this file's tests, which is
//! currently none — use `--lib` for unit tests.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tracing::info;

use fermenter::config::Config;
use fermenter::ingest;
use fermenter::model::Reading;
use fermenter::serial::mock::MockSerialSource;
use fermenter::store::redis::RedisTimeStore;
use fermenter::web::{self, AppState};

#[tokio::main]
async fn main() {
    // Load config — fail fast on error
    let config = Config::from_env().unwrap_or_else(|e| {
        eprintln!("Configuration error: {e}");
        std::process::exit(1);
    });

    // Initialise structured logging with config-driven level
    tracing_subscriber::fmt()
        .with_env_filter(&config.rust_log)
        .init();

    info!(
        serial_port = %config.serial_port,
        mock = config.mock_serial,
        redis_url = %config.redis_url,
        brew_id = %config.default_brew_id,
        "fermenter controller starting"
    );

    // Building the store never blocks on Redis being reachable — the
    // connection is established lazily, and write/read failures are logged
    // and skipped by the ingest loop rather than stopping startup (design.md
    // decision 5). Only invalid *config values* are fail-fast, handled above.
    //
    // The exit path below is reached only if `redis_url` fails to *parse* as
    // a URL (`redis::Client::open`) — a malformed config value, not a
    // connectivity problem. An unreachable-but-well-formed Redis URL never
    // hits this branch; it's handled by the lazy connection manager's
    // reconnect/backoff instead, per decision 5.
    let store = RedisTimeStore::connect(&config.redis_url, config.ts_retention_days)
        .unwrap_or_else(|e| {
            eprintln!("Redis configuration error: {e}");
            std::process::exit(1);
        });

    // Rehydrate current state from persisted storage, if any, before the
    // ingest loop starts (temperature-monitoring: rehydrate on start).
    let rehydrated = ingest::rehydrate_latest_reading(&store, &config.default_brew_id).await;
    let latest_reading: Arc<Mutex<Option<Reading>>> = Arc::new(Mutex::new(rehydrated));

    if !config.mock_serial {
        // Real serial source — deferred to slice-6
        eprintln!(
            "Real serial source not yet implemented. Set MOCK_SERIAL=true to run without hardware."
        );
        std::process::exit(1);
    }

    // Liveness flag for `/healthz` (operational-health): set once the
    // ingest task actually starts running, read by the web layer.
    let ingest_alive = Arc::new(AtomicBool::new(false));

    let app_state = AppState {
        latest: Arc::clone(&latest_reading),
        brew_id: config.default_brew_id.clone(),
        env: Arc::new(web::build_environment()),
        ingest_alive: Arc::clone(&ingest_alive),
    };
    let router = web::build_router(app_state);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", config.http_port))
        .await
        .unwrap_or_else(|e| {
            eprintln!("failed to bind HTTP port {}: {e}", config.http_port);
            std::process::exit(1);
        });
    info!(port = config.http_port, "web server listening");

    let lines = vec![
        Ok(r#"{"target":19.5,"average":18.2,"min":18.0,"max":18.4,"ambient":20.1,"action":"heating","reason-code":"below-target"}"#.to_string()),
        Ok(r#"{"target":19.5,"average":19.0,"min":18.9,"max":19.1,"ambient":20.0,"action":"idle"}"#.to_string()),
        Ok(r#"not valid json"#.to_string()),
        Ok(r#"{"target":19.5,"average":19.5,"min":19.4,"max":19.6,"ambient":20.0,"action":"idle","json-size":42}"#.to_string()),
    ];
    let mut source = MockSerialSource::new(lines);

    // Run the ingest loop and the web server concurrently in the same
    // process (rewrite-plan.md §2 single-binary decision). `axum::serve`
    // never completes on its own, so once the (currently finite, mock)
    // ingest feed is exhausted the server keeps the process alive rather
    // than letting it exit — the behaviour this slice adds over slice-1/2,
    // where the process exited once ingestion finished.
    let ingest_future = async {
        ingest_alive.store(true, Ordering::Relaxed);
        ingest::ingest_loop(
            &mut source,
            Arc::clone(&latest_reading),
            &store,
            &config.default_brew_id,
        )
        .await;
        info!("ingest loop finished (mock source exhausted) — web server keeps running");
    };

    let server_future = async { axum::serve(listener, router).await };
    let (_, server_result) = tokio::join!(ingest_future, server_future);
    if let Err(e) = server_result {
        eprintln!("web server error: {e}");
        std::process::exit(1);
    }
}
