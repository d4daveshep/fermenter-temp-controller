//! Ports the `<float>`-framed target-temperature parsing from
//! `readSerialWithStartEndMarkers()`/`getUpdatedTargetTemp()` in
//! `arduino/TempController/TempController.ino`, fixing the zero-target bug
//! along the way — see proposal.md's "Zero-target fix" and design.md
//! Decision 11.

/// Parses one `<float>`-framed target temperature out of `buf`, e.g.
/// `<19.5>` -> `Some(19.5)`.
///
/// Pure and stateless over a single buffer — see design.md Decision 11 for
/// why the persistent across-tick accumulator lives at the call site
/// (`esp32c3/src/main.rs`) instead of in here. Returns `None` if `buf` has
/// no start marker, an unterminated frame, or content that isn't a valid
/// float — including `0.0`, unlike the Arduino original this ports (see
/// `zero_frame_returns_some_zero` below).
pub fn parse_target_frame(buf: &[u8]) -> Option<f64> {
    let start = buf.iter().position(|&b| b == b'<')?;
    let end = buf[start..].iter().position(|&b| b == b'>')? + start;
    let inner = core::str::from_utf8(&buf[start + 1..end]).ok()?;
    inner.parse::<f64>().ok()
}

/// The fields the JSON telemetry line reports — the minimal set
/// `fermenter/src/model.rs`'s `Reading` struct deserialises (see design.md
/// Decision 5). `action`/`reason_code` are `&'static str` because that's
/// what `Decision::action_text()`/`reason_code()` (task 2) already produce.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct TelemetryData {
    pub target: f64,
    pub average: f64,
    pub min: f64,
    pub max: f64,
    pub ambient: f64,
    pub action: &'static str,
    #[serde(rename = "reason-code")]
    pub reason_code: &'static str,
}

/// Serialises `data` to a single compact JSON line, newline-terminated,
/// ready to write straight to the USB-Serial-JTAG telemetry stream — see
/// design.md Decision 5. 256 bytes gives comfortable headroom over the
/// worst-case measured payload (~130 bytes); serialisation only fails if
/// that budget is somehow exceeded, which no realistic field values do, so
/// a failure here falls back to an empty (still newline-terminated) line
/// rather than panicking the control loop over a telemetry hiccup.
pub fn format_telemetry(data: &TelemetryData) -> heapless::String<256> {
    let mut json: heapless::String<256> =
        serde_json_core::to_string(data).unwrap_or_else(|_| heapless::String::new());
    let _ = json.push('\n');
    json
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_frame_returns_some_value() {
        assert_eq!(parse_target_frame(b"<19.5>"), Some(19.5));
    }

    #[test]
    fn zero_frame_returns_some_zero() {
        // The Arduino firmware treats a parsed 0.0 as "no frame received"
        // (`if (newTarget != 0.0)`), silently dropping a legitimate 0.0
        // target. This is the behavioral fix from proposal.md: a valid
        // `<0.0>` frame must be distinguishable from no frame at all.
        assert_eq!(parse_target_frame(b"<0.0>"), Some(0.0));
    }

    #[test]
    fn no_frame_returns_none() {
        assert_eq!(parse_target_frame(b"no markers here"), None);
        assert_eq!(parse_target_frame(b""), None);
    }

    #[test]
    fn partial_frame_returns_none() {
        assert_eq!(parse_target_frame(b"<19.5"), None);
    }

    #[test]
    fn malformed_frame_returns_none() {
        assert_eq!(parse_target_frame(b"<abc>"), None);
    }

    fn sample_telemetry() -> TelemetryData {
        TelemetryData {
            target: 19.5,
            average: 18.2,
            min: 18.0,
            max: 18.4,
            ambient: 20.1,
            action: "Rest",
            reason_code: "RC3.1",
        }
    }

    #[test]
    fn contains_all_required_keys() {
        let json = format_telemetry(&sample_telemetry());
        for key in [
            "\"target\"",
            "\"average\"",
            "\"min\"",
            "\"max\"",
            "\"ambient\"",
            "\"action\"",
            "\"reason-code\"",
        ] {
            assert!(json.contains(key), "missing key {key} in {json:?}");
        }
    }

    #[test]
    fn action_string_matches_expected_value() {
        let mut data = sample_telemetry();
        data.action = "Heat";
        let json = format_telemetry(&data);
        assert!(json.contains("\"action\":\"Heat\""), "got {json:?}");
    }

    #[test]
    fn reason_code_is_byte_identical() {
        let mut data = sample_telemetry();
        data.reason_code = "RC7.2";
        let json = format_telemetry(&data);
        assert!(json.contains("\"reason-code\":\"RC7.2\""), "got {json:?}");
    }

    #[test]
    fn output_is_newline_terminated() {
        let json = format_telemetry(&sample_telemetry());
        assert!(json.ends_with('\n'), "got {json:?}");
    }
}
