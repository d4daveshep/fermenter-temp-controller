#![no_std]
#![no_main]

//! Scans the OneWire bus on `ONE_WIRE_PIN` and prints the ROM address of
//! every DS18B20 found, then halts. Run once per board during bring-up to
//! populate `FERMENTER_SENSOR_ADDR`/`AMBIENT_SENSOR_ADDR` in `sensors.rs` —
//! see design.md Decision 3 and `firmware/README.md`.
//!
//! ```bash
//! cargo run --release --example discover_sensors --features debug-log
//! ```

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{DriveMode, Flex, InputConfig, OutputConfig},
};
use onecable::OneWire;

esp_bootloader_esp_idf::esp_app_desc!();

// Matches `ONE_WIRE_PIN` in HARDWARE.md / task 12.1.
const ONE_WIRE_GPIO: u8 = 4;

#[esp_hal::main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let mut delay = Delay::new();

    // OneWire needs a true open-drain, bidirectional pin: driven low to
    // signal, released (external 4.7k pull-up brings it high) to listen.
    let mut one_wire_pin = Flex::new(peripherals.GPIO4);
    one_wire_pin.apply_input_config(&InputConfig::default());
    one_wire_pin.set_input_enable(true);
    one_wire_pin
        .apply_output_config(&OutputConfig::default().with_drive_mode(DriveMode::OpenDrain));
    one_wire_pin.set_output_enable(true);
    one_wire_pin.set_high();

    esp_println::println!("Scanning OneWire bus on GPIO{ONE_WIRE_GPIO} for DS18B20 sensors...");

    let mut wire = OneWire::new(&mut one_wire_pin);
    let mut count = 0u32;
    for rom_code in wire.search_rom_iter(&mut delay) {
        count += 1;
        esp_println::println!(
            "Found device #{count}: family=0x{:02X} rom={rom_code} valid_crc={}",
            rom_code.family_code(),
            rom_code.validate_crc(),
        );
    }

    if count == 0 {
        esp_println::println!("No devices found on the bus.");
    } else {
        esp_println::println!("Scan complete: {count} device(s) found.");
    }

    loop {
        delay.delay_millis(1000);
    }
}
