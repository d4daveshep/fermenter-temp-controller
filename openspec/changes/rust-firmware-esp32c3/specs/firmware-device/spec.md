# firmware-device

## Purpose

The ESP32-C3 embedded binary that implements the physical fermenter controller:
reading DS18B20 temperature sensors over a OneWire bus, driving heat and cool
relay GPIO outputs, and communicating with the Raspberry Pi host over
USB-Serial-JTAG at 115200 baud using the fixed serial contract (JSON telemetry
out, `<float>`-framed target in).

## Requirements

### Requirement: DS18B20 sensor identification by ROM address

The system SHALL identify each DS18B20 temperature sensor by its unique 64-bit
ROM address (not by bus-scan index), so that sensor roles (`fermenter` and
`ambient`) remain correct across sensor replacement, rewiring, or bus-scan
order changes.

#### Scenario: Sensors are addressed by compile-time ROM constants

- **WHEN** the firmware reads temperatures from the OneWire bus
- **THEN** it requests conversion from the sensor matching
  `FERMENTER_SENSOR_ADDR` for the fermenter reading and `AMBIENT_SENSOR_ADDR`
  for the ambient reading, ignoring bus-scan order

#### Scenario: Sensor discovery helper identifies ROM addresses

- **WHEN** a sensor-discovery binary (or example) is flashed to the board
- **THEN** it prints the 64-bit ROM addresses of all DS18B20 sensors found on
  the bus to USB-Serial-JTAG output, allowing the operator to assign
  `FERMENTER_SENSOR_ADDR` and `AMBIENT_SENSOR_ADDR` constants before
  compiling the production binary

### Requirement: Relay GPIO control

The system SHALL drive two GPIO output pins — one for the heating relay and one
for the cooling relay — according to the active `Action`, with mutual exclusion
(heating and cooling SHALL NOT be active simultaneously).

#### Scenario: Rest action disables both relays

- **WHEN** the active action is `Rest`
- **THEN** both the heat relay pin and the cool relay pin are driven LOW

#### Scenario: Heat action enables heat relay only

- **WHEN** the active action is `Heat`
- **THEN** the heat relay pin is driven HIGH and the cool relay pin is driven LOW

#### Scenario: Cool action enables cool relay only

- **WHEN** the active action is `Cool`
- **THEN** the heat relay pin is driven LOW and the cool relay pin is driven HIGH

#### Scenario: Error action disables both relays (fail-safe)

- **WHEN** the active action is `Error`
- **THEN** both relay pins are driven LOW (safe default — no uncontrolled
  heating or cooling)

### Requirement: Main control loop cadence

The system SHALL execute the control loop on a 1-second tick: poll for an
incoming `<target>` frame, read both temperature sensors, run
`make_action_decision`, update relay GPIO outputs, and emit JSON telemetry to
the host every 10 seconds.

#### Scenario: Loop runs once per second

- **WHEN** the Embassy tick timer fires
- **THEN** the system performs one complete loop cycle (serial poll, sensor
  read, decision, relay update) and waits for the next tick before repeating

#### Scenario: JSON telemetry is emitted every 10 seconds

- **WHEN** 10 loop ticks have elapsed since the last telemetry emission
- **THEN** one JSON telemetry line is written to USB-Serial-JTAG output

#### Scenario: Target temperature update takes effect within one loop cycle

- **WHEN** a valid `<target>` frame is received during the serial poll step
- **THEN** the new target temperature is used for the `make_action_decision`
  call in the same loop cycle

### Requirement: Target temperature frame parsing

The system SHALL parse incoming bytes on the USB-Serial-JTAG port looking for
`<value>`-framed floating-point numbers, returning `Some(f64)` when a complete
valid frame is received and `None` when no frame is present. The value `0.0`
SHALL be a valid target temperature.

#### Scenario: Valid frame is parsed to Some(value)

- **WHEN** the received bytes contain a complete `<19.5>` frame
- **THEN** the parser returns `Some(19.5)`

#### Scenario: Frame containing 0.0 is parsed to Some(0.0)

- **WHEN** the received bytes contain a `<0.0>` frame
- **THEN** the parser returns `Some(0.0)`, not `None` — zero is a valid
  lager temperature

#### Scenario: No frame present returns None

- **WHEN** no complete `<...>` frame is present in the received bytes
- **THEN** the parser returns `None` and the target temperature is unchanged

#### Scenario: Malformed or incomplete frame is ignored

- **WHEN** the received bytes contain a partial or malformed frame (e.g.
  `<abc>`, `<19.5` without closing `>`)
- **THEN** the parser returns `None` and no target update occurs

### Requirement: JSON telemetry output format

The system SHALL emit telemetry as a single newline-terminated JSON object
containing the fields required by the host's `Reading` struct: `target`,
`average`, `min`, `max`, `ambient`, `action`, and `reason-code`.

#### Scenario: Emitted JSON contains all required fields

- **WHEN** a telemetry line is emitted
- **THEN** it is a valid JSON object with keys `target`, `average`, `min`,
  `max`, `ambient`, `action`, and `reason-code`, terminated with a newline

#### Scenario: Action field matches host-expected strings

- **WHEN** the active action is `Rest`, `Heat`, `Cool`, or `Error`
- **THEN** the `action` field in the JSON is `"Rest"`, `"Heat"`, `"Cool"`,
  or `"Error"` respectively — matching the strings the host's templates and
  `reason_code.rs` expect

#### Scenario: Reason code field matches RC codes exactly

- **WHEN** the decision has a reason code (e.g. `"RC3.1"`)
- **THEN** the `reason-code` field in the emitted JSON is that exact string,
  byte-for-byte, so the host's `reason_code::reason_text` match works correctly

#### Scenario: Telemetry is emitted at 115200 baud over USB-Serial-JTAG

- **WHEN** a telemetry line is emitted
- **THEN** it is sent over the native USB-Serial-JTAG peripheral at 115200 baud,
  matching the fixed serial contract the host expects

### Requirement: Startup initialisation

The system SHALL initialise the EMA with the first real sensor reading rather
than starting from `0.0`, so the host receives a reasonable average from the
first telemetry emission.

#### Scenario: EMA seeded from first sensor reading on startup

- **WHEN** the device boots and completes its first successful sensor read
- **THEN** the EMA for both fermenter and ambient trackers is seeded with those
  first values before any telemetry is emitted

### Requirement: GPIO pin assignments avoid strapping pins

The system SHALL assign relay output and OneWire data pins to GPIO numbers that
are safe to drive during boot (i.e. not GPIO2, GPIO8, or GPIO9, which are
strapping pins on the ESP32-C3).

#### Scenario: No relay or sensor pin overlaps with a strapping pin

- **WHEN** the firmware is compiled
- **THEN** `HEAT_RELAY_PIN`, `COOL_RELAY_PIN`, and `ONE_WIRE_PIN` are all
  constants whose values exclude GPIO2, GPIO8, and GPIO9
