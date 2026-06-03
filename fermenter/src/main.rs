mod config;
mod error;
mod ingest;
mod model;
mod serial;

use std::sync::{Arc, Mutex};

use tracing::info;

use crate::config::Config;
use crate::model::Reading;
use crate::serial::mock::MockSerialSource;

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
        "fermenter controller starting"
    );

    let latest_reading: Arc<Mutex<Option<Reading>>> = Arc::new(Mutex::new(None));

    if config.mock_serial {
        let lines = vec![
            Ok(r#"{"target":19.5,"average":18.2,"min":18.0,"max":18.4,"ambient":20.1,"action":"heating","reason-code":"below-target"}"#.to_string()),
            Ok(r#"{"target":19.5,"average":19.0,"min":18.9,"max":19.1,"ambient":20.0,"action":"idle"}"#.to_string()),
            Ok(r#"not valid json"#.to_string()),
            Ok(r#"{"target":19.5,"average":19.5,"min":19.4,"max":19.6,"ambient":20.0,"action":"idle","json-size":42}"#.to_string()),
        ];
        let mut source = MockSerialSource::new(lines);
        ingest::ingest_loop(&mut source, Arc::clone(&latest_reading)).await;
    } else {
        // Real serial source — deferred to slice-6
        eprintln!(
            "Real serial source not yet implemented. Set MOCK_SERIAL=true to run without hardware."
        );
        std::process::exit(1);
    }
}
