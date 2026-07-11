use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Reading {
    pub target: f64,
    pub average: f64,
    pub min: f64,
    pub max: f64,
    pub ambient: f64,
    pub action: String,
    #[serde(rename = "reason-code", default)]
    pub reason_code: String,
    /// Legacy field from firmware — accepted but not retained.
    #[serde(rename = "json-size", default)]
    pub json_size: Option<u32>,
}

/// Minimal persisted controller state — the values that don't originate from
/// the Arduino and so must be saved/restored independently of the reading
/// time series. `latest` is deliberately not carried here; it's reconstructed
/// on startup via `TimeStore::last_reading`, avoiding a redundant, possibly
/// stale copy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ControllerState {
    pub target_temp: f64,
    pub brew_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn valid_json() -> &'static str {
        r#"{"target":19.5,"average":18.2,"min":18.0,"max":18.4,"ambient":20.1,"action":"heating","reason-code":"below-target"}"#
    }

    #[test]
    fn valid_reading_parses_all_fields() {
        let r: Reading = serde_json::from_str(valid_json()).unwrap();
        assert_eq!(r.target, 19.5);
        assert_eq!(r.average, 18.2);
        assert_eq!(r.min, 18.0);
        assert_eq!(r.max, 18.4);
        assert_eq!(r.ambient, 20.1);
        assert_eq!(r.action, "heating");
        assert_eq!(r.reason_code, "below-target");
    }

    #[test]
    fn reason_code_defaults_when_absent() {
        let json = r#"{"target":19.5,"average":18.2,"min":18.0,"max":18.4,"ambient":20.1,"action":"idle"}"#;
        let r: Reading = serde_json::from_str(json).unwrap();
        assert_eq!(r.reason_code, "");
    }

    #[test]
    fn json_size_is_accepted_but_treated_as_optional() {
        let json = r#"{"target":19.5,"average":18.2,"min":18.0,"max":18.4,"ambient":20.1,"action":"idle","json-size":42}"#;
        let r: Reading = serde_json::from_str(json).unwrap();
        // json-size is parsed into the field but is not meaningful data
        assert_eq!(r.json_size, Some(42));
        assert_eq!(r.target, 19.5);
    }

    #[rstest]
    #[case("not json at all")]
    #[case(r#"{"target":19.5}"#)] // missing required fields
    #[case(r#"{}"#)]
    #[case("")]
    fn malformed_json_fails_to_parse(#[case] input: &str) {
        let result = serde_json::from_str::<Reading>(input);
        assert!(result.is_err(), "expected parse failure for: {input}");
    }
}
