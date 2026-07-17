use crate::model::Reading;

/// Returns the human-readable description of a firmware reason code, or `None`
/// if the code is unrecognized, empty, or an error sentinel (`RC_ERR`).
///
/// Source of truth: `arduino/TempController/TempControllerRules.md:27-58`.
/// The descriptions below are copied verbatim from that table.
pub fn reason_text(code: &str) -> Option<&'static str> {
    match code {
        // Failsafe range exceeded
        "RC1" | "RC6" => Some(
            "HEAT because the temperature is below the failsafe minimum, regardless of ambient conditions",
        ),
        "RC5" | "RC10" => Some(
            "COOL because the temperature is above the failsafe maximum, regardless of ambient conditions",
        ),
        // With natural heating
        "RC2.1" => Some(
            "REST->REST because even though there is natural heating, the temperature is below the target range",
        ),
        "RC2.2" => Some(
            "COOL->REST because temperature is below target range and there is natural heating",
        ),
        "RC2.3" => Some(
            "HEAT->REST because temperature is below target range and there is natural heating",
        ),
        "RC3.1" => Some(
            "REST->REST because we are in the target range.  There is natural heating so expect temperature to rise",
        ),
        "RC3.2" => Some(
            "COOL->COOL because we are still within target range and we have natural heating (cooling overrun adjustment is used here)",
        ),
        "RC3.3" => Some(
            "HEAT->REST because we are in the target range.  There is natural heating so expect temperature to rise",
        ),
        "RC4.1" => Some(
            "REST->COOL because the temperature is above the target range and we have natural heating",
        ),
        "RC4.2" => Some(
            "COOL->COOL becuase the temperature is above the target range and we have natural heating",
        ),
        "RC4.3" => Some(
            "HEAT->COOL because the temperature is above target range and we have natural heating.  (adjust heating lag?)",
        ),
        // With natural cooling
        "RC7.1" => Some(
            "REST->HEAT because the temperature is below target range and there is natural cooling",
        ),
        "RC7.2" => Some(
            "COOL->HEAT because the temperature is below the target range and there is natural cooling (adjust cooling lag?)",
        ),
        "RC7.3" => Some(
            "HEAT->HEAT because the temperature is below target range and there is natural cooling",
        ),
        "RC8.1" => Some(
            "REST->REST because the temperature is in the target range.  There is natural cooling so expect temperature to fall",
        ),
        "RC8.2" => Some(
            "COOL->REST because the temperature is in the target range.  There is natural cooling so expect temperature to fall",
        ),
        "RC8.3" => Some(
            "HEAT->HEAT because the temperature is still within target range and there is natural cooling",
        ),
        "RC9.1" => Some(
            "REST->REST because the temperature is above target range and there is natural cooling",
        ),
        "RC9.2" => Some(
            "COOL->COOL because the temperature is above target range and we'll continue cooling",
        ),
        "RC9.3" => Some(
            "HEAT->REST because the temperature is above target range and there is natural cooling",
        ),
        _ => None,
    }
}

pub fn reading_reason_text(reading: &Reading) -> Option<&'static str> {
    reason_text(&reading.reason_code)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_some(code: &str, expected: &str) {
        let text = reason_text(code);
        assert!(text.is_some(), "expected Some for code {code:?}, got None");
        assert_eq!(text.unwrap(), expected, "mismatch for code {code:?}");
    }

    fn assert_none(code: &str) {
        assert!(
            reason_text(code).is_none(),
            "expected None for code {code:?}, got Some"
        );
    }

    #[test]
    fn failsafe_codes() {
        assert_some(
            "RC1",
            "HEAT because the temperature is below the failsafe minimum, regardless of ambient conditions",
        );
        assert_some(
            "RC6",
            "HEAT because the temperature is below the failsafe minimum, regardless of ambient conditions",
        );
        assert_some(
            "RC5",
            "COOL because the temperature is above the failsafe maximum, regardless of ambient conditions",
        );
        assert_some(
            "RC10",
            "COOL because the temperature is above the failsafe maximum, regardless of ambient conditions",
        );
    }

    #[test]
    fn natural_heating_codes() {
        assert_some(
            "RC2.1",
            "REST->REST because even though there is natural heating, the temperature is below the target range",
        );
        assert_some(
            "RC2.2",
            "COOL->REST because temperature is below target range and there is natural heating",
        );
        assert_some(
            "RC2.3",
            "HEAT->REST because temperature is below target range and there is natural heating",
        );
        assert_some(
            "RC3.1",
            "REST->REST because we are in the target range.  There is natural heating so expect temperature to rise",
        );
        assert_some(
            "RC3.2",
            "COOL->COOL because we are still within target range and we have natural heating (cooling overrun adjustment is used here)",
        );
        assert_some(
            "RC3.3",
            "HEAT->REST because we are in the target range.  There is natural heating so expect temperature to rise",
        );
        assert_some(
            "RC4.1",
            "REST->COOL because the temperature is above the target range and we have natural heating",
        );
        assert_some(
            "RC4.2",
            "COOL->COOL becuase the temperature is above the target range and we have natural heating",
        );
        assert_some(
            "RC4.3",
            "HEAT->COOL because the temperature is above target range and we have natural heating.  (adjust heating lag?)",
        );
    }

    #[test]
    fn natural_cooling_codes() {
        assert_some(
            "RC7.1",
            "REST->HEAT because the temperature is below target range and there is natural cooling",
        );
        assert_some(
            "RC7.2",
            "COOL->HEAT because the temperature is below the target range and there is natural cooling (adjust cooling lag?)",
        );
        assert_some(
            "RC7.3",
            "HEAT->HEAT because the temperature is below target range and there is natural cooling",
        );
        assert_some(
            "RC8.1",
            "REST->REST because the temperature is in the target range.  There is natural cooling so expect temperature to fall",
        );
        assert_some(
            "RC8.2",
            "COOL->REST because the temperature is in the target range.  There is natural cooling so expect temperature to fall",
        );
        assert_some(
            "RC8.3",
            "HEAT->HEAT because the temperature is still within target range and there is natural cooling",
        );
        assert_some(
            "RC9.1",
            "REST->REST because the temperature is above target range and there is natural cooling",
        );
        assert_some(
            "RC9.2",
            "COOL->COOL because the temperature is above target range and we'll continue cooling",
        );
        assert_some(
            "RC9.3",
            "HEAT->REST because the temperature is above target range and there is natural cooling",
        );
    }

    #[test]
    fn empty_and_error_sentinels() {
        assert_none("");
        assert_none("RC_ERR");
    }

    #[test]
    fn unknown_code() {
        assert_none("RC_FOO");
        assert_none("RC99.9");
        assert_none("garbage");
    }

    #[test]
    fn reading_reason_text_delegates_correctly() {
        let mut reading = Reading {
            target: 19.5,
            average: 18.7,
            min: 18.0,
            max: 19.0,
            ambient: 20.1,
            action: "heating".to_string(),
            reason_code: String::new(),
            json_size: None,
        };
        assert_eq!(reading_reason_text(&reading), None);

        reading.reason_code = "RC3.1".to_string();
        assert!(reading_reason_text(&reading).is_some());
        assert!(
            reading_reason_text(&reading)
                .unwrap()
                .contains("target range")
        );

        reading.reason_code = "RC_ERR".to_string();
        assert_eq!(reading_reason_text(&reading), None);
    }
}
