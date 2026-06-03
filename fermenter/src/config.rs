use serde::Deserialize;

use crate::error::{AppError, Result};

#[derive(Debug, Deserialize)]
pub struct Config {
    /// Serial device path, e.g. /dev/ttyACM0
    #[serde(default = "default_serial_port")]
    pub serial_port: String,

    /// Serial baud rate (default 115200 to match Arduino firmware).
    /// Used by the real tokio-serial impl in slice-6.
    #[serde(default = "default_serial_baud")]
    #[allow(dead_code)]
    pub serial_baud: u32,

    /// When true, use the mock serial source instead of real hardware
    #[serde(default)]
    pub mock_serial: bool,

    /// Log level filter string, e.g. "info", "debug"
    #[serde(default = "default_log_level")]
    pub rust_log: String,
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

impl Config {
    /// Load configuration from environment variables. Returns an error if any
    /// required value is missing or cannot be parsed.
    pub fn from_env() -> Result<Self> {
        envy::from_env::<Config>().map_err(|e| AppError::Config(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Safety: tests run sequentially (single-threaded test harness for this
    // module) so mutating env vars is safe here.
    fn with_env<F: FnOnce()>(vars: &[(&str, &str)], f: F) {
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
        unsafe {
            std::env::remove_var("SERIAL_PORT");
            std::env::remove_var("SERIAL_BAUD");
            std::env::remove_var("MOCK_SERIAL");
            std::env::remove_var("RUST_LOG");
        }
        let config = Config::from_env().expect("should use defaults");
        assert_eq!(config.serial_port, "/dev/ttyACM0");
        assert_eq!(config.serial_baud, 115_200);
        assert!(!config.mock_serial);
        assert_eq!(config.rust_log, "info");
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
}
