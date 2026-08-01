#![cfg_attr(not(test), no_std)]

pub mod controller_action_rules;
pub mod decision;
pub mod protocol;
pub mod temperature_readings;

pub use controller_action_rules::{
    ControllerActionRules, DEFAULT_TARGET_TEMP, NaturalDrift, TARGET_RANGE, get_natural_drift,
};
pub use decision::{Action, Decision};
pub use protocol::parse_target_frame;
pub use temperature_readings::TemperatureReadings;
