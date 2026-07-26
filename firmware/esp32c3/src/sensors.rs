//! DS18B20 sensor identification by ROM address — see design.md Decision 3.

/// Discovered on the test board via `examples/discover_sensors.rs`
/// (2026-07-26): the only sensor currently wired, on `ONE_WIRE_PIN` = GPIO4.
/// Tentatively assigned the fermenter role — reassign once it's clear which
/// physical sensor this is and a second sensor is wired for the ambient role.
pub const FERMENTER_SENSOR_ADDR: [u8; 8] = [0x28, 0xF9, 0xC9, 0x4D, 0x07, 0x00, 0x00, 0x2C];

/// Placeholder — the test board only has one DS18B20 wired so far. Replace
/// once a second sensor is connected and re-run `discover_sensors`.
pub const AMBIENT_SENSOR_ADDR: [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
