#![no_std]
#![no_main]

//! Exercises `sensors::SensorReader` end-to-end against the real
//! `FERMENTER_SENSOR_ADDR`/`AMBIENT_SENSOR_ADDR` constants, printing both
//! temperatures once per second — confirms OneWire timing, the pull-up
//! resistor, and the ROM address constants are correct through the same
//! code path the main control loop (task 16) will use. See tasks.md task
//! 13.2.
//!
//! ```bash
//! cargo run --release --example sensor_read_test
//! ```

use esp_backtrace as _;
use esp_hal::{delay::Delay, gpio::Flex};
use esp32c3::sensors::SensorReader;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal::main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let pin = Flex::new(peripherals.GPIO4);
    let mut reader = SensorReader::new(pin);
    let delay = Delay::new();

    esp_println::println!("Reading fermenter/ambient sensors every 1s via SensorReader...");

    loop {
        match reader.read() {
            Ok((fermenter, ambient)) => {
                esp_println::println!("fermenter={fermenter:.2}C ambient={ambient:.2}C")
            }
            Err(e) => esp_println::println!("read error: {e:?}"),
        }
        delay.delay_millis(1000);
    }
}
