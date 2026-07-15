//! Thin binary entry point — all logic lives in the `fermenter` library
//! (`src/lib.rs`) so it's usable from `tests/` integration tests. Module
//! unit tests (config, model, ingest, serial, store) live in the library
//! and run via `cargo test --lib` (or plain `cargo test`, which also runs
//! this binary's own tests and the `tests/` integration suite).
//! `cargo test --bin fermenter` only runs this file's tests, which is
//! currently none — use `--lib` for unit tests.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tokio::sync::watch;
use tracing::info;

use fermenter::brew_session;
use fermenter::config::Config;
use fermenter::ingest;
use fermenter::model::Reading;
use fermenter::serial::SerialSource;
use fermenter::serial::arduino::ArduinoSerialSource;
use fermenter::serial::mock::MockSerialSource;
use fermenter::store::TimeStore;
use fermenter::store::redis::RedisTimeStore;
use fermenter::temperature_control;
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
    let store: Arc<dyn TimeStore> = Arc::new(
        RedisTimeStore::connect(&config.redis_url, config.ts_retention_days).unwrap_or_else(|e| {
            eprintln!("Redis configuration error: {e}");
            std::process::exit(1);
        }),
    );

    // Seed the active brew identifier from persisted controller state,
    // falling back to the configured default (brew-session: "Persist and
    // restore the brew identifier across restarts").
    let initial_brew_id =
        brew_session::initial_brew_id(store.as_ref(), &config.default_brew_id).await;
    let (brew_tx, brew_rx) = watch::channel(initial_brew_id.clone());

    // Rehydrate current state from persisted storage, if any, before the
    // ingest loop starts (temperature-monitoring: rehydrate on start). Uses
    // the resolved `initial_brew_id` — not the static `config.default_brew_id`
    // — so a persisted brew identifier that differs from the configured
    // default is honoured on startup (design.md decision 5's correctness
    // fix; previously this always rehydrated against `config.default_brew_id`
    // regardless of what had actually been persisted).
    let rehydrated = ingest::rehydrate_latest_reading(store.as_ref(), &initial_brew_id).await;
    let latest_reading: Arc<Mutex<Option<Reading>>> = Arc::new(Mutex::new(rehydrated));

    // Seed the desired target temperature from persisted controller state,
    // falling back to the configured default (temperature-control:
    // "Persist and restore the target temperature across restarts").
    let initial_target =
        temperature_control::initial_target(store.as_ref(), config.default_target_temp).await;
    let (target_tx, target_rx) = watch::channel(initial_target);

    // Liveness flag for `/healthz` (operational-health): set once the
    // ingest task actually starts running, read by the web layer.
    let ingest_alive = Arc::new(AtomicBool::new(false));

    let app_state = AppState {
        latest: Arc::clone(&latest_reading),
        env: Arc::new(web::build_environment()),
        ingest_alive: Arc::clone(&ingest_alive),
        target_tx,
        brew_tx,
        store: Arc::clone(&store),
    };
    let router = web::build_router(app_state);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", config.http_port))
        .await
        .unwrap_or_else(|e| {
            eprintln!("failed to bind HTTP port {}: {e}", config.http_port);
            std::process::exit(1);
        });
    info!(port = config.http_port, "web server listening");

    // A scripted, static demo feed — `MockSerialSource` has no notion of a
    // device that updates its own reported state in response to a written
    // target, so `reading.target` here always stays `19.5` regardless of
    // what an operator sets via `/target`; only the real
    // `ArduinoSerialSource` transport (slice-6) reports a changed target
    // back. Cycling this finite script indefinitely (rather than exhausting
    // once) keeps the ingest loop alive so target reconcile keeps running
    // and the dashboard's other fields visibly tick over during manual
    // testing.
    fn mock_lines() -> Vec<std::result::Result<String, String>> {
        vec![
            Ok(r#"{"target":19.5,"average":18.2,"min":18.0,"max":18.4,"ambient":20.1,"action":"heating","reason-code":"below-target"}"#.to_string()),
            Ok(r#"{"target":19.5,"average":19.0,"min":18.9,"max":19.1,"ambient":20.0,"action":"idle","reason-code":"RC3.1"}"#.to_string()),
            Ok(r#"not valid json"#.to_string()),
            Ok(r#"{"target":19.5,"average":19.5,"min":19.4,"max":19.6,"ambient":20.0,"action":"idle","reason-code":"RC1","json-size":42}"#.to_string()),
        ]
    }

    // Run the ingest loop and the web server concurrently in the same
    // process (rewrite-plan.md §2 single-binary decision). `axum::serve`
    // never completes on its own.
    //
    // `MOCK_SERIAL=true` (hardware-free dev/test): the scripted mock feed is
    // re-scripted and re-processed in a cycle (with a pause between cycles,
    // mimicking the Arduino's periodic reporting cadence) so the process
    // keeps ingesting — and reconciling the target — for the process's
    // lifetime rather than exhausting once at startup.
    //
    // `MOCK_SERIAL=false` (default, real hardware — slice-6):
    // `ArduinoSerialSource` opens `config.serial_port` at
    // `config.serial_baud` lazily on first use and reconnects with backoff
    // on any failure (see `serial::arduino`), so a single `ingest_loop` call
    // runs for the process's lifetime without an outer restart loop — it
    // never returns (real `read_line` errors are retried internally, never
    // the "mock source exhausted" sentinel `ingest_loop` treats as
    // terminal).
    let ingest_future = async {
        ingest_alive.store(true, Ordering::Relaxed);
        if config.mock_serial {
            loop {
                let mut source = MockSerialSource::new(mock_lines());
                ingest::ingest_loop(
                    &mut source,
                    Arc::clone(&latest_reading),
                    store.as_ref(),
                    &brew_rx,
                    &target_rx,
                )
                .await;
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        } else {
            let mut source: Box<dyn SerialSource> = Box::new(ArduinoSerialSource::new(
                config.serial_port.clone(),
                config.serial_baud,
            ));
            ingest::ingest_loop(
                source.as_mut(),
                Arc::clone(&latest_reading),
                store.as_ref(),
                &brew_rx,
                &target_rx,
            )
            .await;
        }
    };

    let server_future = async { axum::serve(listener, router).await };
    let (_, server_result) = tokio::join!(ingest_future, server_future);
    if let Err(e) = server_result {
        eprintln!("web server error: {e}");
        std::process::exit(1);
    }
}
