//! DS18B20 sensor identification by ROM address — see design.md Decision 3.

/// Placeholder — the test board's discovered sensor
/// (`family=0x28 rom=2C0000074DC9F928`) will not be used on the production
/// board, so no address has been recorded yet. Run
/// `examples/discover_sensors.rs` against the production board once both
/// DS18B20s are wired up, then set this to the fermenter sensor's ROM
/// address.
pub const FERMENTER_SENSOR_ADDR: [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

/// Placeholder — see `FERMENTER_SENSOR_ADDR`. Run `discover_sensors` against
/// the production board and set this to the ambient sensor's ROM address.
pub const AMBIENT_SENSOR_ADDR: [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
