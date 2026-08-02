#![no_std]
#![no_main]

//! Cycles the relay outputs Rest -> Heat -> Cool -> Rest once per second,
//! forever. Flash this once relays (or LEDs+resistors as a stand-in) are
//! wired to `HEAT_RELAY_PIN`/`COOL_RELAY_PIN` (see HARDWARE.md), and confirm
//! with a multimeter or LED that pin logic matches the spec — task 12.3 in
//! tasks.md.
//!
//! ```bash
//! cargo run --release --example relay_test
//! ```

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
};
use esp32c3::relays::RelayController;
use logic::Action;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal::main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();

    let heat = Output::new(peripherals.GPIO0, Level::Low, OutputConfig::default());
    let cool = Output::new(peripherals.GPIO1, Level::Low, OutputConfig::default());
    let mut relays = RelayController::new(heat, cool);

    esp_println::println!(
        "Cycling relays: Rest -> Heat -> Cool -> Rest, 1s per step. Ctrl+C to stop watching."
    );

    loop {
        for (action, label) in [
            (Action::Rest, "Rest  (heat LOW,  cool LOW)"),
            (Action::Heat, "Heat  (heat HIGH, cool LOW)"),
            (Action::Cool, "Cool  (heat LOW,  cool HIGH)"),
            (Action::Rest, "Rest  (heat LOW,  cool LOW)"),
        ] {
            esp_println::println!("-> {label}");
            relays.set(action);
            delay.delay_millis(1000);
        }
    }
}
