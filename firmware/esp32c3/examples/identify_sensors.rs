#![no_std]
#![no_main]

//! Discovers every DS18B20 on the OneWire bus, then loops printing each
//! one's live temperature, labeled by ROM address, once per second forever.
//! Run this when more than one sensor is wired and it's unclear which
//! physical sensor is which: warm one sensor (fingertip, warm water, ...)
//! and watch which address's reading rises, then record that mapping —
//! which ROM address is `FERMENTER_SENSOR_ADDR` vs `AMBIENT_SENSOR_ADDR` in
//! `sensors.rs` — see task 11/13 in tasks.md.
//!
//! ```bash
//! cargo run --release --example identify_sensors
//! ```

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{DriveMode, Flex, InputConfig, OutputConfig},
};
use heapless::Vec;
use onecable::{OneWire, ds18b20::DS18B20, rom_code::RomCode};

esp_bootloader_esp_idf::esp_app_desc!();

// Matches `ONE_WIRE_PIN` in HARDWARE.md / sensors.rs.
const ONE_WIRE_GPIO: u8 = 4;

#[esp_hal::main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let mut delay = Delay::new();

    // OneWire needs a true open-drain, bidirectional pin — see
    // examples/discover_sensors.rs for the same setup.
    let mut one_wire_pin = Flex::new(peripherals.GPIO4);
    one_wire_pin.apply_input_config(&InputConfig::default());
    one_wire_pin.set_input_enable(true);
    one_wire_pin
        .apply_output_config(&OutputConfig::default().with_drive_mode(DriveMode::OpenDrain));
    one_wire_pin.set_output_enable(true);
    one_wire_pin.set_high();

    esp_println::println!("Scanning OneWire bus on GPIO{ONE_WIRE_GPIO} for DS18B20 sensors...");

    let rom_codes: Vec<RomCode, 8> = {
        let mut wire = OneWire::new(&mut one_wire_pin);
        wire.search_rom_iter(&mut delay).collect()
    };

    if rom_codes.is_empty() {
        esp_println::println!("No devices found on the bus.");
    } else {
        esp_println::println!("Found {} device(s):", rom_codes.len());
        for (i, rom) in rom_codes.iter().enumerate() {
            esp_println::println!("  #{}: rom={rom} family=0x{:02X}", i + 1, rom.family_code());
        }
    }

    let sensors: Vec<DS18B20, 8> = rom_codes
        .iter()
        .filter_map(|rom| DS18B20::try_from(*rom).ok())
        .collect();

    esp_println::println!(
        "Reading temperatures every 1s — warm one sensor and watch which rom's reading \
         changes. Ctrl+C to stop watching."
    );

    loop {
        for (i, sensor) in sensors.iter().enumerate() {
            let mut wire = OneWire::new(&mut one_wire_pin);
            match sensor.read_temperature(&mut wire, &mut Delay::new(), &mut Delay::new()) {
                Ok(temp) => esp_println::println!(
                    "  rom={} temp={:.2}C",
                    rom_codes[i],
                    temp.to_num::<f64>()
                ),
                Err(e) => esp_println::println!("  rom={}: read error: {e:?}", rom_codes[i]),
            }
        }
        delay.delay_millis(1000);
    }
}
