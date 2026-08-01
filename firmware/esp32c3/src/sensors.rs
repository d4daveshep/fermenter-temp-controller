//! DS18B20 sensor identification by ROM address — see design.md Decision 3.

use core::convert::Infallible;

use esp_hal::delay::Delay;
use esp_hal::gpio::{DriveMode, Flex, InputConfig, OutputConfig};
use onecable::{FamilyCodeError, OneWire, OneWireError, ds18b20::DS18B20, rom_code::RomCode};

/// Documentation/cross-reference constant — see HARDWARE.md and
/// `examples/discover_sensors.rs`. The actual peripheral selection happens
/// via `peripherals.GPIO4` at construction time, not a runtime pin-number
/// lookup (same pattern as `relays::HEAT_RELAY_PIN`/`COOL_RELAY_PIN`).
pub const ONE_WIRE_PIN: u8 = 4;

/// Discovered on the production board via `examples/identify_sensors.rs`
/// (2026-08-01): `rom=820000073792F228` (valid CRC, family 0x28). Physically
/// identified by warming the sensor wired near the fermenter vessel and
/// watching this address's reading rise — see `firmware/README.md`.
pub const FERMENTER_SENSOR_ADDR: [u8; 8] = [0x28, 0xF2, 0x92, 0x37, 0x07, 0x00, 0x00, 0x82];

/// Discovered on the production board via `examples/identify_sensors.rs`
/// (2026-08-01): `rom=52000009B5F2F728` (valid CRC, family 0x28) — the other
/// of the two sensors found, by elimination the ambient one. See
/// `FERMENTER_SENSOR_ADDR`.
pub const AMBIENT_SENSOR_ADDR: [u8; 8] = [0x28, 0xF7, 0xF2, 0xB5, 0x09, 0x00, 0x00, 0x52];

/// A single sensor's read failed — either its ROM address constant isn't a
/// DS18B20 (family code != `0x28`, e.g. a still-unset placeholder address),
/// or the OneWire transaction itself failed (CRC mismatch, no presence
/// pulse, ...).
#[derive(Debug)]
pub enum SensorFault {
    NotADs18b20(FamilyCodeError),
    Bus(OneWireError<Infallible>),
}

/// Which of the two sensors failed to read — see design.md Decision 10: the
/// caller maps either variant to `Action::Error` (fail-safe, both relays
/// LOW) rather than propagating a bogus float.
#[derive(Debug)]
pub enum SensorError {
    Fermenter(SensorFault),
    Ambient(SensorFault),
}

/// Reads both DS18B20 sensors over a shared OneWire bus, identified by their
/// fixed ROM addresses (`FERMENTER_SENSOR_ADDR`/`AMBIENT_SENSOR_ADDR`) rather
/// than bus-scan position — see design.md Decision 3.
pub struct SensorReader<'a> {
    pin: Flex<'a>,
}

impl<'a> SensorReader<'a> {
    /// Configures `pin` as a true open-drain, bidirectional OneWire line
    /// (driven low to signal, released for the external 4.7k pull-up to
    /// bring high to listen) — same setup as `examples/discover_sensors.rs`.
    pub fn new(mut pin: Flex<'a>) -> Self {
        pin.apply_input_config(&InputConfig::default());
        pin.set_input_enable(true);
        pin.apply_output_config(&OutputConfig::default().with_drive_mode(DriveMode::OpenDrain));
        pin.set_output_enable(true);
        pin.set_high();
        Self { pin }
    }

    /// Triggers a conversion and reads back both sensors by their ROM
    /// address. Returns `(fermenter, ambient)` in degrees Celsius, or the
    /// first error encountered — never a silently-coerced bogus float (see
    /// design.md Decision 10).
    pub fn read(&mut self) -> Result<(f64, f64), SensorError> {
        let fermenter =
            Self::read_one(&mut self.pin, FERMENTER_SENSOR_ADDR).map_err(SensorError::Fermenter)?;
        let ambient =
            Self::read_one(&mut self.pin, AMBIENT_SENSOR_ADDR).map_err(SensorError::Ambient)?;
        Ok((fermenter, ambient))
    }

    fn read_one(pin: &mut Flex<'a>, addr: [u8; 8]) -> Result<f64, SensorFault> {
        let sensor =
            DS18B20::try_from(RomCode { bytes: addr }).map_err(SensorFault::NotADs18b20)?;
        let mut wire = OneWire::new(pin);
        let temp = sensor
            .read_temperature(&mut wire, &mut Delay::new(), &mut Delay::new())
            .map_err(SensorFault::Bus)?;
        Ok(temp.to_num::<f64>())
    }
}
