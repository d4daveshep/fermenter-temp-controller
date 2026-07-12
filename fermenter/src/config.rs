use serde::Deserialize;

use crate::error::{AppError, Result};

#[derive(Debug, Deserialize)]
pub struct Config {
    /// Serial device path, e.g. /dev/ttyACM0
    #[serde(default = "default_serial_port")]
    pub serial_port: String,

    /// Serial baud rate (default 115200 to match Arduino firmware).
    /// Used by `ArduinoSerialSource`, the real `tokio-serial` transport.
    #[serde(default = "default_serial_baud")]
    pub serial_baud: u32,

    /// When true, use the mock serial source instead of real hardware
    #[serde(default)]
    pub mock_serial: bool,

    /// Log level filter string, e.g. "info", "debug"
    #[serde(default = "default_log_level")]
    pub rust_log: String,

    /// Time-series storage connection URL, e.g. redis://localhost:6379
    #[serde(default = "default_redis_url")]
    pub redis_url: String,

    /// Retention period (days) for time-series data, matching the old
    /// InfluxDB default.
    #[serde(default = "default_ts_retention_days")]
    pub ts_retention_days: u32,

    /// Brew identifier used to tag persisted readings/state. Static for this
    /// slice — runtime relabeling arrives with the brew-session capability.
    #[serde(default = "default_brew_id")]
    pub default_brew_id: String,

    /// TCP port the Axum web server binds to.
    #[serde(default = "default_http_port")]
    pub http_port: u16,

    /// Fallback target temperature (°C) used to seed the current target
    /// when no persisted `ControllerState` exists yet.
    #[serde(default = "default_target_temp")]
    pub default_target_temp: f64,
}

fn default_serial_port() -> String {
    "/dev/ttyACM0".to_string()
}

fn default_serial_baud() -> u32 {
    115_200
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_redis_url() -> String {
    "redis://localhost:6379".to_string()
}

fn default_ts_retention_days() -> u32 {
    7
}

fn default_brew_id() -> String {
    "00-TEST-v00".to_string()
}

fn default_http_port() -> u16 {
    8080
}

fn default_target_temp() -> f64 {
    19.5
}

impl Config {
    /// Load configuration from environment variables. Returns an error if any
    /// required value is missing or cannot be parsed.
    pub fn from_env() -> Result<Self> {
        envy::from_env::<Config>().map_err(|e| AppError::Config(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, OnceLock};

    use super::*;

    /// `Config::from_env` reads process-wide environment variables, and
    /// `cargo test` runs `#[test]` functions concurrently across threads
    /// within a single binary. Without serialising access, tests that set or
    /// clear `SERIAL_PORT`/`SERIAL_BAUD`/`MOCK_SERIAL`/`RUST_LOG` race each
    /// other and intermittently observe a sibling test's env state. This
    /// mutex ensures only one env-mutating test body runs at a time.
    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    const ALL_VARS: &[&str] = &[
        "SERIAL_PORT",
        "SERIAL_BAUD",
        "MOCK_SERIAL",
        "RUST_LOG",
        "REDIS_URL",
        "TS_RETENTION_DAYS",
        "DEFAULT_BREW_ID",
        "HTTP_PORT",
        "DEFAULT_TARGET_TEMP",
    ];

    // Safety: callers hold `env_lock()` for the duration of `f`, so no other
    // test thread can observe or mutate these vars concurrently.
    fn with_env<F: FnOnce()>(vars: &[(&str, &str)], f: F) {
        let _guard = env_lock().lock().unwrap_or_else(|e| e.into_inner());
        for (k, v) in vars {
            unsafe { std::env::set_var(k, v) };
        }
        f();
        for (k, _) in vars {
            unsafe { std::env::remove_var(k) };
        }
    }

    #[test]
    fn valid_env_parses_into_config() {
        with_env(
            &[
                ("SERIAL_PORT", "/dev/ttyUSB0"),
                ("SERIAL_BAUD", "9600"),
                ("MOCK_SERIAL", "true"),
                ("RUST_LOG", "debug"),
            ],
            || {
                let config = Config::from_env().expect("should parse valid env");
                assert_eq!(config.serial_port, "/dev/ttyUSB0");
                assert_eq!(config.serial_baud, 9600);
                assert!(config.mock_serial);
                assert_eq!(config.rust_log, "debug");
            },
        );
    }

    #[test]
    fn defaults_applied_when_optional_vars_absent() {
        let _guard = env_lock().lock().unwrap_or_else(|e| e.into_inner());
        for var in ALL_VARS {
            unsafe { std::env::remove_var(var) };
        }
        let config = Config::from_env().expect("should use defaults");
        assert_eq!(config.serial_port, "/dev/ttyACM0");
        assert_eq!(config.serial_baud, 115_200);
        assert!(!config.mock_serial);
        assert_eq!(config.rust_log, "info");
        assert_eq!(config.redis_url, "redis://localhost:6379");
        assert_eq!(config.ts_retention_days, 7);
        assert_eq!(config.default_brew_id, "00-TEST-v00");
        assert_eq!(config.http_port, 8080);
        assert_eq!(config.default_target_temp, 19.5);
    }

    #[test]
    fn invalid_baud_rate_yields_error() {
        with_env(&[("SERIAL_BAUD", "not_a_number")], || {
            let result = Config::from_env();
            assert!(result.is_err(), "expected error for invalid baud rate");
        });
    }

    #[test]
    fn mock_serial_toggle_parsed() {
        with_env(&[("MOCK_SERIAL", "true")], || {
            let config = Config::from_env().unwrap();
            assert!(config.mock_serial);
        });
        with_env(&[("MOCK_SERIAL", "false")], || {
            let config = Config::from_env().unwrap();
            assert!(!config.mock_serial);
        });
    }

    #[test]
    fn redis_config_parses_from_env() {
        with_env(
            &[
                ("REDIS_URL", "redis://redis-host:6380"),
                ("TS_RETENTION_DAYS", "14"),
                ("DEFAULT_BREW_ID", "01-IPA-v02"),
            ],
            || {
                let config = Config::from_env().expect("should parse valid env");
                assert_eq!(config.redis_url, "redis://redis-host:6380");
                assert_eq!(config.ts_retention_days, 14);
                assert_eq!(config.default_brew_id, "01-IPA-v02");
            },
        );
    }

    #[test]
    fn invalid_retention_days_yields_error() {
        with_env(&[("TS_RETENTION_DAYS", "not_a_number")], || {
            let result = Config::from_env();
            assert!(result.is_err(), "expected error for invalid retention days");
        });
    }

    #[test]
    fn http_port_parses_from_env() {
        with_env(&[("HTTP_PORT", "9090")], || {
            let config = Config::from_env().expect("should parse valid env");
            assert_eq!(config.http_port, 9090);
        });
    }

    #[test]
    fn invalid_http_port_yields_error() {
        with_env(&[("HTTP_PORT", "not_a_number")], || {
            let result = Config::from_env();
            assert!(result.is_err(), "expected error for invalid http port");
        });
    }

    #[test]
    fn default_target_temp_parses_from_env() {
        with_env(&[("DEFAULT_TARGET_TEMP", "20.5")], || {
            let config = Config::from_env().expect("should parse valid env");
            assert_eq!(config.default_target_temp, 20.5);
        });
    }

    #[test]
    fn invalid_default_target_temp_yields_error() {
        with_env(&[("DEFAULT_TARGET_TEMP", "not_a_number")], || {
            let result = Config::from_env();
            assert!(
                result.is_err(),
                "expected error for invalid default target temp"
            );
        });
    }
}
