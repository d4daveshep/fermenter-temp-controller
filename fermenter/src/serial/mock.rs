use async_trait::async_trait;

use crate::error::{AppError, Result};
use crate::serial::SerialSource;

// NOTE: The real tokio-serial implementation and reconnect/backoff are
// deferred to slice-6. The device-connection spec captures these requirements
// so they are locked in for that slice without changing the trait surface.

/// A mock serial source for testing and hardware-free development.
///
/// Feeds scripted lines in order and records any `write_target` calls.
/// An `Err` entry in `lines` causes `read_line` to return a `SerialError`.
pub struct MockSerialSource {
    lines: Vec<std::result::Result<String, String>>,
    index: usize,
    #[allow(dead_code)]
    pub written_targets: Vec<f64>,
}

impl MockSerialSource {
    /// Create a mock that emits the given lines in order.
    pub fn new(lines: Vec<std::result::Result<String, String>>) -> Self {
        Self {
            lines,
            index: 0,
            written_targets: Vec::new(),
        }
    }
}

#[async_trait]
impl SerialSource for MockSerialSource {
    async fn read_line(&mut self) -> Result<String> {
        match self.lines.get(self.index) {
            Some(Ok(line)) => {
                self.index += 1;
                Ok(line.clone())
            }
            Some(Err(msg)) => {
                self.index += 1;
                Err(AppError::Serial(msg.clone()))
            }
            None => Err(AppError::Serial("mock source exhausted".to_string())),
        }
    }

    async fn write_target(&mut self, target: f64) -> Result<()> {
        self.written_targets.push(target);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn emits_scripted_lines_in_order() {
        let mut mock =
            MockSerialSource::new(vec![Ok("line one".to_string()), Ok("line two".to_string())]);
        assert_eq!(mock.read_line().await.unwrap(), "line one");
        assert_eq!(mock.read_line().await.unwrap(), "line two");
    }

    #[tokio::test]
    async fn records_write_target_calls() {
        let mut mock = MockSerialSource::new(vec![]);
        mock.write_target(19.5).await.unwrap();
        mock.write_target(20.0).await.unwrap();
        assert_eq!(mock.written_targets, vec![19.5, 20.0]);
    }

    #[tokio::test]
    async fn injected_error_is_returned() {
        let mut mock = MockSerialSource::new(vec![
            Ok("ok line".to_string()),
            Err("injected serial error".to_string()),
        ]);
        assert!(mock.read_line().await.is_ok());
        let err = mock.read_line().await.unwrap_err();
        assert!(matches!(err, AppError::Serial(_)));
    }

    #[tokio::test]
    async fn exhausted_source_returns_error() {
        let mut mock = MockSerialSource::new(vec![]);
        let err = mock.read_line().await.unwrap_err();
        assert!(matches!(err, AppError::Serial(_)));
    }
}
