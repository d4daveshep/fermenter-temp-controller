//! Ports `arduino/TempController/ControllerActionRules.h`/`.cpp`.
//!
//! See `openspec/changes/rust-firmware-esp32c3/design.md` Decision 9 and
//! `HARDWARE.md` for where the production constants below come from.

use crate::decision::{Action, Decision};

/// `target ± TARGET_RANGE` is the target band; `target ± 2 × TARGET_RANGE`
/// is the failsafe band. Ported verbatim from the Arduino's
/// `defaultRange = 0.3`.
pub const TARGET_RANGE: f64 = 0.3;

/// The firmware's own startup default, used only until the host writes its
/// own persisted/configured target. Ported verbatim from the Arduino's
/// `defaultTargetTemp = 20.0`.
pub const DEFAULT_TARGET_TEMP: f64 = 20.0;

/// How far below the target-range minimum cooling is allowed to coast before
/// switching off, to avoid overshooting the bottom of the range. Ported
/// verbatim from the Arduino's `coolingOverrunAdjustment = 0.2`.
const COOLING_OVERRUN_ADJUSTMENT: f64 = 0.2;

/// Which direction the fermenter temperature will drift without active
/// intervention, based on how ambient compares to the fermenter's actual
/// temperature. Target temperature is irrelevant to this classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NaturalDrift {
    NaturalHeating,
    NaturalCooling,
}

/// `ambient >= actual` implies natural heating (the fermenter will warm
/// toward ambient without intervention); `ambient < actual` implies natural
/// cooling.
pub fn get_natural_drift(ambient: f64, actual: f64) -> NaturalDrift {
    if ambient < actual {
        NaturalDrift::NaturalCooling
    } else {
        NaturalDrift::NaturalHeating
    }
}

/// Decides the next relay action from the current action, ambient
/// temperature, and fermenter temperature, per the RC1-RC10 decision rules.
pub struct ControllerActionRules {
    target: f64,
    range: f64,
    decision: Decision,
}

impl ControllerActionRules {
    pub fn new(target_temp: f64, range: f64) -> Self {
        Self {
            target: target_temp,
            range,
            decision: Decision::new(),
        }
    }

    pub fn get_target_temp(&self) -> f64 {
        self.target
    }

    pub fn set_target_temp(&mut self, new_target_temp: f64) {
        self.target = new_target_temp;
    }

    pub fn get_failsafe_min(&self) -> f64 {
        self.target - self.range * 2.0
    }

    pub fn get_failsafe_max(&self) -> f64 {
        self.target + self.range * 2.0
    }

    pub fn get_target_range_min(&self) -> f64 {
        self.target - self.range
    }

    pub fn get_target_range_max(&self) -> f64 {
        self.target + self.range
    }

    pub fn get_stop_cooling_temp(&self) -> f64 {
        self.get_target_range_min() + COOLING_OVERRUN_ADJUSTMENT
    }

    fn is_temp_in_target_range(&self, temp: f64) -> bool {
        self.get_target_range_min() <= temp && temp <= self.get_target_range_max()
    }

    fn is_temp_below_target_range(&self, temp: f64) -> bool {
        temp < self.get_target_range_min()
    }

    fn is_temp_above_target_range(&self, temp: f64) -> bool {
        temp > self.get_target_range_max()
    }

    fn is_temp_below_failsafe(&self, temp: f64) -> bool {
        temp < self.get_failsafe_min()
    }

    fn is_temp_above_failsafe(&self, temp: f64) -> bool {
        temp > self.get_failsafe_max()
    }

    pub fn get_decision(&self) -> Decision {
        self.decision
    }

    pub fn reset_decision(&mut self) {
        self.decision.clear();
    }

    /// RC1 — HEAT because we've tripped the failsafe minimum, regardless of
    /// ambient conditions or current action.
    fn check_failsafe_min(&mut self, actual_temp: f64) {
        if !self.decision.is_made() && self.is_temp_below_failsafe(actual_temp) {
            self.decision.set_next_action(Action::Heat);
            self.decision.set_reason_code("RC1");
        }
    }

    /// RC5 — COOL because we've tripped the failsafe maximum, regardless of
    /// ambient conditions or current action.
    fn check_failsafe_max(&mut self, actual_temp: f64) {
        if !self.decision.is_made() && self.is_temp_above_failsafe(actual_temp) {
            self.decision.set_next_action(Action::Cool);
            self.decision.set_reason_code("RC5");
        }
    }

    /// RC2.2 / RC7.2 — stop cooling early (or switch to heating) rather than
    /// overrun the bottom of the target range.
    fn check_cooling_overrun(&mut self, current_action: Action, actual: f64, drift: NaturalDrift) {
        if !self.decision.is_made()
            && current_action == Action::Cool
            && actual < self.get_stop_cooling_temp()
        {
            match drift {
                NaturalDrift::NaturalHeating => {
                    self.decision.set_next_action(Action::Rest);
                    self.decision.set_reason_code("RC2.2");
                }
                NaturalDrift::NaturalCooling => {
                    self.decision.set_next_action(Action::Heat);
                    self.decision.set_reason_code("RC7.2");
                }
            }
        }
    }

    /// RC2.1, RC2.3, RC7.1, RC7.3.
    fn decide_when_below_range(&mut self, current_action: Action, drift: NaturalDrift) {
        if self.decision.is_made() {
            return;
        }
        match (drift, current_action) {
            (NaturalDrift::NaturalHeating, Action::Rest) => {
                self.decision.set_next_action(Action::Rest);
                self.decision.set_reason_code("RC2.1");
            }
            (NaturalDrift::NaturalHeating, Action::Heat) => {
                self.decision.set_next_action(Action::Rest);
                self.decision.set_reason_code("RC2.3");
            }
            (NaturalDrift::NaturalCooling, Action::Rest) => {
                self.decision.set_next_action(Action::Heat);
                self.decision.set_reason_code("RC7.1");
            }
            (NaturalDrift::NaturalCooling, Action::Heat) => {
                self.decision.set_next_action(Action::Heat);
                self.decision.set_reason_code("RC7.3");
            }
            _ => {
                self.decision.set_next_action(Action::Error);
                self.decision.set_reason_code("RC_ERR");
            }
        }
    }

    /// RC3.1, RC3.2, RC3.3, RC8.1, RC8.2, RC8.3.
    fn decide_when_in_range(&mut self, current_action: Action, drift: NaturalDrift) {
        if self.decision.is_made() {
            return;
        }
        match (drift, current_action) {
            (NaturalDrift::NaturalHeating, Action::Rest) => {
                self.decision.set_next_action(Action::Rest);
                self.decision.set_reason_code("RC3.1");
            }
            (NaturalDrift::NaturalHeating, Action::Cool) => {
                self.decision.set_next_action(Action::Cool);
                self.decision.set_reason_code("RC3.2");
            }
            (NaturalDrift::NaturalHeating, Action::Heat) => {
                self.decision.set_next_action(Action::Rest);
                self.decision.set_reason_code("RC3.3");
            }
            (NaturalDrift::NaturalCooling, Action::Rest) => {
                self.decision.set_next_action(Action::Rest);
                self.decision.set_reason_code("RC8.1");
            }
            (NaturalDrift::NaturalCooling, Action::Cool) => {
                self.decision.set_next_action(Action::Rest);
                self.decision.set_reason_code("RC8.2");
            }
            (NaturalDrift::NaturalCooling, Action::Heat) => {
                self.decision.set_next_action(Action::Heat);
                self.decision.set_reason_code("RC8.3");
            }
            _ => {
                self.decision.set_next_action(Action::Error);
                self.decision.set_reason_code("RC_ERR");
            }
        }
    }

    /// RC4.1, RC4.2, RC4.3, RC9.1, RC9.2, RC9.3.
    fn decide_when_above_range(&mut self, current_action: Action, drift: NaturalDrift) {
        if self.decision.is_made() {
            return;
        }
        match (drift, current_action) {
            (NaturalDrift::NaturalHeating, Action::Rest) => {
                self.decision.set_next_action(Action::Cool);
                self.decision.set_reason_code("RC4.1");
            }
            (NaturalDrift::NaturalHeating, Action::Cool) => {
                self.decision.set_next_action(Action::Cool);
                self.decision.set_reason_code("RC4.2");
            }
            (NaturalDrift::NaturalHeating, Action::Heat) => {
                self.decision.set_next_action(Action::Cool);
                self.decision.set_reason_code("RC4.3");
            }
            (NaturalDrift::NaturalCooling, Action::Rest) => {
                self.decision.set_next_action(Action::Rest);
                self.decision.set_reason_code("RC9.1");
            }
            (NaturalDrift::NaturalCooling, Action::Cool) => {
                self.decision.set_next_action(Action::Cool);
                self.decision.set_reason_code("RC9.2");
            }
            (NaturalDrift::NaturalCooling, Action::Heat) => {
                self.decision.set_next_action(Action::Rest);
                self.decision.set_reason_code("RC9.3");
            }
            _ => {
                self.decision.set_next_action(Action::Error);
                self.decision.set_reason_code("RC_ERR");
            }
        }
    }

    /// Orchestrates the full decision: failsafe bounds first, then cooling
    /// overrun, then the target-range rules — mirroring
    /// `ControllerActionRules::makeActionDecision` in the Arduino source.
    pub fn make_action_decision(
        &mut self,
        current_action: Action,
        ambient: f64,
        actual: f64,
    ) -> Decision {
        self.decision.clear();

        self.check_failsafe_min(actual);
        self.check_failsafe_max(actual);

        let drift = get_natural_drift(ambient, actual);
        self.check_cooling_overrun(current_action, actual, drift);

        if self.is_temp_below_target_range(actual) {
            self.decide_when_below_range(current_action, drift);
        }
        if self.is_temp_in_target_range(actual) {
            self.decide_when_in_range(current_action, drift);
        }
        if self.is_temp_above_target_range(actual) {
            self.decide_when_above_range(current_action, drift);
        }

        self.decision
    }
}

impl Default for ControllerActionRules {
    /// Production defaults (design.md Decision 9) — tests construct with
    /// explicit `target`/`range` values instead, mirroring the Arduino test
    /// suite this was ported from.
    fn default() -> Self {
        Self::new(DEFAULT_TARGET_TEMP, TARGET_RANGE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    // --- Natural drift classification (task 3) ---

    #[test]
    fn natural_heating_when_ambient_equals_fermenter() {
        assert_eq!(get_natural_drift(18.0, 18.0), NaturalDrift::NaturalHeating);
    }

    #[test]
    fn natural_heating_when_ambient_above_fermenter() {
        assert_eq!(get_natural_drift(18.0, 16.0), NaturalDrift::NaturalHeating);
    }

    #[test]
    fn natural_cooling_when_ambient_below_fermenter() {
        assert_eq!(get_natural_drift(18.0, 20.0), NaturalDrift::NaturalCooling);
    }

    // --- Failsafe boundary enforcement (task 4) ---
    //
    // target = 18.0, range = 0.5 => target range [17.5, 18.5],
    // failsafe < 17.0 or > 19.0 (Test_ControllerActionRules.cpp WhatToDoNext,
    // tests 1, 5, 6, 10).

    #[rstest]
    #[case(Action::Rest, 22.0)] // ambient high
    #[case(Action::Cool, 22.0)]
    #[case(Action::Heat, 22.0)]
    #[case(Action::Rest, 14.0)] // ambient low
    #[case(Action::Cool, 14.0)]
    #[case(Action::Heat, 14.0)]
    fn below_failsafe_forces_heat_regardless_of_action_and_ambient(
        #[case] current_action: Action,
        #[case] ambient: f64,
    ) {
        let mut controller = ControllerActionRules::new(18.0, 0.5);
        let decision = controller.make_action_decision(current_action, ambient, 16.5);
        assert_eq!(decision.action(), Action::Heat);
        assert_eq!(decision.reason_code(), "RC1");
    }

    #[rstest]
    #[case(Action::Rest, 22.0)]
    #[case(Action::Cool, 22.0)]
    #[case(Action::Heat, 22.0)]
    #[case(Action::Rest, 14.0)]
    #[case(Action::Cool, 14.0)]
    #[case(Action::Heat, 14.0)]
    fn above_failsafe_forces_cool_regardless_of_action_and_ambient(
        #[case] current_action: Action,
        #[case] ambient: f64,
    ) {
        let mut controller = ControllerActionRules::new(18.0, 0.5);
        let decision = controller.make_action_decision(current_action, ambient, 19.5);
        assert_eq!(decision.action(), Action::Cool);
        assert_eq!(decision.reason_code(), "RC5");
    }

    // --- Cooling overrun adjustment (task 5) ---

    #[test]
    fn stop_cooling_temp_is_target_range_min_plus_adjustment() {
        // Ported from AdjustmentForCoolingOverrun: target=18.0, range=0.5.
        let controller = ControllerActionRules::new(18.0, 0.5);
        let expected = 18.0 - 0.5 + 0.2;
        assert_eq!(controller.get_stop_cooling_temp(), expected);
    }

    #[test]
    fn cooling_overrun_stops_cooling_with_natural_heating_rc2_2() {
        let mut controller = ControllerActionRules::new(18.0, 0.5);
        let adjusted_stop_cooling_temp = controller.get_stop_cooling_temp() - 0.1;
        let decision =
            controller.make_action_decision(Action::Cool, 22.0, adjusted_stop_cooling_temp);
        assert_eq!(decision.action(), Action::Rest);
        assert_eq!(decision.reason_code(), "RC2.2");
    }

    #[test]
    fn cooling_overrun_switches_to_heating_with_natural_cooling_rc7_2() {
        let mut controller = ControllerActionRules::new(18.0, 0.5);
        let adjusted_stop_cooling_temp = controller.get_stop_cooling_temp() - 0.1;
        let decision =
            controller.make_action_decision(Action::Cool, 14.0, adjusted_stop_cooling_temp);
        assert_eq!(decision.action(), Action::Heat);
        assert_eq!(decision.reason_code(), "RC7.2");
    }

    // --- Temperature range decision rules (task 6) ---
    //
    // Same target=18.0/range=0.5 fixture as WhatToDoNext; belowTargetRange
    // = 17.4, withinTargetRange = 18.0, aboveTargetRange = 18.6.

    #[rstest]
    #[case(22.0, Action::Rest, Action::Rest, "RC2.1")]
    #[case(22.0, Action::Heat, Action::Rest, "RC2.3")]
    #[case(14.0, Action::Rest, Action::Heat, "RC7.1")]
    #[case(14.0, Action::Heat, Action::Heat, "RC7.3")]
    fn below_range_decisions(
        #[case] ambient: f64,
        #[case] current_action: Action,
        #[case] expected_action: Action,
        #[case] expected_rc: &str,
    ) {
        let mut controller = ControllerActionRules::new(18.0, 0.5);
        let decision = controller.make_action_decision(current_action, ambient, 17.4);
        assert_eq!(decision.action(), expected_action);
        assert_eq!(decision.reason_code(), expected_rc);
    }

    #[rstest]
    #[case(22.0, Action::Rest, Action::Rest, "RC3.1")]
    #[case(22.0, Action::Cool, Action::Cool, "RC3.2")]
    #[case(22.0, Action::Heat, Action::Rest, "RC3.3")]
    #[case(14.0, Action::Rest, Action::Rest, "RC8.1")]
    #[case(14.0, Action::Cool, Action::Rest, "RC8.2")]
    #[case(14.0, Action::Heat, Action::Heat, "RC8.3")]
    fn in_range_decisions(
        #[case] ambient: f64,
        #[case] current_action: Action,
        #[case] expected_action: Action,
        #[case] expected_rc: &str,
    ) {
        let mut controller = ControllerActionRules::new(18.0, 0.5);
        let decision = controller.make_action_decision(current_action, ambient, 18.0);
        assert_eq!(decision.action(), expected_action);
        assert_eq!(decision.reason_code(), expected_rc);
    }

    #[rstest]
    #[case(22.0, Action::Rest, Action::Cool, "RC4.1")]
    #[case(22.0, Action::Cool, Action::Cool, "RC4.2")]
    #[case(22.0, Action::Heat, Action::Cool, "RC4.3")]
    #[case(14.0, Action::Rest, Action::Rest, "RC9.1")]
    #[case(14.0, Action::Cool, Action::Cool, "RC9.2")]
    #[case(14.0, Action::Heat, Action::Rest, "RC9.3")]
    fn above_range_decisions(
        #[case] ambient: f64,
        #[case] current_action: Action,
        #[case] expected_action: Action,
        #[case] expected_rc: &str,
    ) {
        let mut controller = ControllerActionRules::new(18.0, 0.5);
        let decision = controller.make_action_decision(current_action, ambient, 18.6);
        assert_eq!(decision.action(), expected_action);
        assert_eq!(decision.reason_code(), expected_rc);
    }

    // --- Target temperature mutability (task 7) ---

    #[test]
    fn updated_target_temp_is_saved() {
        let mut controller = ControllerActionRules::new(6.0, 0.3);
        controller.set_target_temp(7.0);
        assert_eq!(controller.get_target_temp(), 7.0);
    }

    #[test]
    fn updated_target_temp_is_used_by_next_decision() {
        let mut controller = ControllerActionRules::new(18.0, 0.5);

        // 16.5 is below the failsafe min for target=18.0 (17.0) -> RC1.
        let decision = controller.make_action_decision(Action::Rest, 22.0, 16.5);
        assert_eq!(decision.action(), Action::Heat);
        assert_eq!(decision.reason_code(), "RC1");

        // Lowering the target to 16.0 puts the same 16.5 actual reading
        // inside the new target range [15.5, 16.5] instead of below the new
        // failsafe min (15.0) -> RC3.1/Rest, not RC1/Heat.
        controller.set_target_temp(16.0);
        let decision = controller.make_action_decision(Action::Rest, 22.0, 16.5);
        assert_eq!(decision.reason_code(), "RC3.1");
        assert_eq!(decision.action(), Action::Rest);
    }
}
