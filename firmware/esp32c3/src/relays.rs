//! Relay GPIO control — see `specs/firmware-device/spec.md` "Relay GPIO
//! control" and HARDWARE.md's "Relay Modules" section (active-HIGH assumed;
//! invert here if a specific relay module turns out to be active-LOW).

use esp_hal::gpio::Output;
use logic::Action;

/// Documentation/cross-reference constants — see HARDWARE.md. The actual
/// peripheral selection happens via `peripherals.GPIO0`/`GPIO1` at
/// construction time (esp-hal pins are distinct types, not runtime indices).
pub const HEAT_RELAY_PIN: u8 = 0;
pub const COOL_RELAY_PIN: u8 = 1;

/// Owns the two relay GPIO outputs. The single point of truth for relay
/// state — heating and cooling are never driven high simultaneously, since
/// `set` always assigns both pins together from one `Action`.
pub struct RelayController<'d> {
    heat: Output<'d>,
    cool: Output<'d>,
}

impl<'d> RelayController<'d> {
    pub fn new(heat: Output<'d>, cool: Output<'d>) -> Self {
        Self { heat, cool }
    }

    /// Drives both relay pins per the active `Action`:
    /// `Rest`/`Error`/`NoAction` → both LOW, `Heat` → heat HIGH cool LOW,
    /// `Cool` → heat LOW cool HIGH.
    pub fn set(&mut self, action: Action) {
        match action {
            Action::Rest | Action::Error | Action::NoAction => {
                self.heat.set_low();
                self.cool.set_low();
            }
            Action::Heat => {
                self.heat.set_high();
                self.cool.set_low();
            }
            Action::Cool => {
                self.heat.set_low();
                self.cool.set_high();
            }
        }
    }
}
