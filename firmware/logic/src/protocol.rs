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
}
