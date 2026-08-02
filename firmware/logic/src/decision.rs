//! Ports `arduino/TempController/Decision.h`/`.cpp`.

/// The relay action the controller has decided on.
///
/// `NoAction` is the sentinel used before any decision has been made (or
/// after `Decision::clear()`); it must never be reported to the host or
/// applied to the relays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    NoAction,
    Rest,
    Heat,
    Cool,
    Error,
}

/// Pairs an `Action` with the reason code that produced it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Decision {
    action: Action,
    reason_code: &'static str,
}

impl Decision {
    pub fn new() -> Self {
        Self {
            action: Action::NoAction,
            reason_code: "",
        }
    }

    pub fn action(&self) -> Action {
        self.action
    }

    pub fn set_next_action(&mut self, action: Action) {
        self.action = action;
    }

    pub fn reason_code(&self) -> &'static str {
        self.reason_code
    }

    pub fn set_reason_code(&mut self, reason_code: &'static str) {
        self.reason_code = reason_code;
    }

    /// Matches `Decision::getActionText()` in the Arduino source, and the
    /// strings the host's `reason_code.rs` and templates expect.
    pub fn action_text(&self) -> &'static str {
        match self.action {
            Action::NoAction => "No Action",
            Action::Rest => "Rest",
            Action::Heat => "Heat",
            Action::Cool => "Cool",
            Action::Error => "Error",
        }
    }

    pub fn is_made(&self) -> bool {
        self.action != Action::NoAction
    }

    pub fn clear(&mut self) {
        self.action = Action::NoAction;
        self.reason_code = "";
    }
}

impl Default for Decision {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn stores_action_and_reason() {
        let mut decision = Decision::new();
        decision.set_next_action(Action::Heat);
        decision.set_reason_code("RC_TEST");

        assert_eq!(decision.action(), Action::Heat);
        assert_eq!(decision.reason_code(), "RC_TEST");
    }

    #[rstest]
    #[case(Action::NoAction, "No Action")]
    #[case(Action::Rest, "Rest")]
    #[case(Action::Heat, "Heat")]
    #[case(Action::Cool, "Cool")]
    #[case(Action::Error, "Error")]
    fn action_text_for_each_variant(#[case] action: Action, #[case] expected: &str) {
        let mut decision = Decision::new();
        decision.set_next_action(action);
        assert_eq!(decision.action_text(), expected);
    }

    #[test]
    fn is_made_returns_false_before_set() {
        let decision = Decision::new();
        assert!(!decision.is_made());
    }

    #[test]
    fn is_made_returns_true_after_set() {
        let mut decision = Decision::new();
        decision.set_next_action(Action::Rest);
        assert!(decision.is_made());
    }

    #[test]
    fn clear_resets_to_unmade() {
        let mut decision = Decision::new();
        decision.set_next_action(Action::Heat);
        decision.set_reason_code("RC1");

        decision.clear();

        assert!(!decision.is_made());
        assert_eq!(decision.action(), Action::NoAction);
        assert_eq!(decision.reason_code(), "");
    }
}
