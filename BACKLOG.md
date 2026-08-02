# Backlog

## UI Enhancements

- Add a nicer theme to the dashboard via CSS

## Functionality

## Technical Architecture

- Swap from MiniJinja to Askama templating engine

## Code Improvements / Refactoring

## Observability

- Improve the logging:
  - At INFO level: add ambient temperature and reason code
  - At DEBUG level: log the raw JSON string

## Documentation

## Firmware

### Logic

- Refactor firmware/logic/src using EO principles (within the constraints of no_std)

### Device

- Control loop tick/telemetry cadence drifts from spec: `SensorReader::read()` blocks ~1.5s+ per tick (two sequential DS18B20 conversions at 750ms each), so the nominal 1s tick actually takes ~1.5-2s and telemetry lands every ~15-16s instead of every 10s. Either loosen the spec wording to match reality or move sensor reads off the tick path.
- Re-run `fermenter/`'s `#[ignore]`'d hardware reconnect tests (`fermenter/tests/serial_hardware.rs`, or a manual unplug/replug) specifically against the ESP32-C3 board — the reconnect-with-backoff path was never explicitly re-exercised against the new device (host code is unchanged so risk is low, but it hasn't been confirmed).
