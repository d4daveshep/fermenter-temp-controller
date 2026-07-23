# Hardware Reference: ESP32-C3 SuperMini Fermenter Controller

This document captures the physical build decisions for the ESP32-C3 SuperMini
firmware. It is the reference point for GPIO pin assignments, wiring
assumptions, and component choices. Decisions that are still open at spec time
are marked **TBD-at-bring-up** and resolved in task 12.1 of `tasks.md`.

---

## Board

**ESP32-C3 SuperMini Dev Board** (SKU: ESP32-C3-MINI-SLD-1PK, nznelectronics.co.nz)

| Property | Value |
|---|---|
| SoC | ESP32-C3 FN4, single-core RISC-V @ up to 160 MHz |
| Flash | 4 MB onboard |
| SRAM | 400 KB |
| Power input | USB-C (5V) or 5V pin (3.3–6V) |
| Logic voltage | 3.3V — all GPIO signals are 3.3V |
| Dimensions | 22.5 × 18 mm |
| USB | Native USB-Serial-JTAG via internal USB PHY (no bridge chip) |
| Onboard LED | Blue, GPIO8 (active LOW) |
| BOOT button | GPIO9 |
| RESET button | CHIP_PU |

### Exposed GPIO pins (16-pin header)

| Pin | GPIO | Notes |
|---|---|---|
| 1 | 5V | Power |
| 2 | GND | Ground |
| 3 | 3V3 | 3.3V out |
| 4 | GPIO0 | Safe — ADC1 Ch0, PWM |
| 5 | GPIO1 | Safe — ADC1 Ch1, PWM |
| 6 | GPIO2 | ⚠ Strapping pin — avoid for relay/sensor outputs |
| 7 | GPIO3 | Safe — ADC1 Ch3, PWM |
| 8 | GPIO4 | ⚠ JTAG TMS / Quad-SPI hold — use with care |
| 9 | GPIO5 | ⚠ JTAG TDI / Quad-SPI WP — use with care |
| 10 | GPIO6 | ⚠ JTAG TCK / SPI2 SCK — use with care |
| 11 | GPIO7 | ⚠ JTAG TDO / SPI2 SS — use with care |
| 12 | GPIO8 | ⚠ Strapping pin, onboard LED (active LOW) — avoid |
| 13 | GPIO9 | ⚠ Strapping pin, BOOT button — avoid |
| 14 | GPIO10 | Safe — PWM |
| 15 | GPIO21 | UART0 TX (header) |
| 16 | GPIO20 | UART0 RX (header) |

**Safest pins for new assignments (no system duties):** GPIO0, GPIO1, GPIO3, GPIO10.

---

## Signals required by this firmware

Three signals needed: one OneWire data bus, two relay outputs.

### GPIO pin assignments

**Status: TBD-at-bring-up** — to be confirmed and locked as named constants in
task 12.1 before first flash. Candidates from the safe set:

| Signal | Proposed GPIO | Notes |
|---|---|---|
| `ONE_WIRE_PIN` | GPIO3 | OneWire data bus (DS18B20 × 2) |
| `HEAT_RELAY_PIN` | GPIO0 | Heating relay output |
| `COOL_RELAY_PIN` | GPIO1 | Cooling relay output |

Rationale for these candidates: GPIO0, GPIO1, and GPIO3 are the safest general-
purpose pins with no JTAG, SPI, strapping, or LED duty. The assignments must be
verified against the actual relay module board's layout to avoid mechanical
interference on a shared PCB or breadboard. Record the final values as:

```rust
// firmware/esp32c3/src/relays.rs
const HEAT_RELAY_PIN: u8 = 0;
const COOL_RELAY_PIN: u8 = 1;

// firmware/esp32c3/src/sensors.rs
const ONE_WIRE_PIN: u8 = 3;
```

---

## DS18B20 Temperature Sensors

Two sensors: one measuring **fermenter temperature** (inside the fermenter),
one measuring **ambient temperature** (room/environment).

### Identification

Sensors are addressed by **64-bit ROM address**, not by bus-scan index
(see design.md Decision 3). ROM addresses must be discovered before the
production binary can be compiled — see task 11 in `tasks.md`.

```rust
// Placeholders — replace with values from the discovery example
const FERMENTER_SENSOR_ADDR: [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
const AMBIENT_SENSOR_ADDR:   [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
```

### Wiring

| Connection | Detail |
|---|---|
| Power mode | **External power** (3-wire: VDD, GND, DATA) — not parasitic mode |
| VDD | 3.3V from the board's 3V3 pin |
| GND | GND |
| DATA | `ONE_WIRE_PIN` (see above) |
| Pull-up resistor | **4.7 kΩ** between DATA and VDD |
| Bus topology | Both sensors on one shared data line (single pull-up resistor) |

**Why external power, not parasitic:** Parasitic (2-wire) mode is fragile
with multiple sensors on one bus and incompatible with some `ds18b20` crate
configurations. External power avoids conversion-timing workarounds entirely.

**DS18B20 conversion time:** ~750 ms at 12-bit resolution (the default). The
control loop tick is 1 second, giving sufficient margin.

---

## Relay Modules

The firmware drives two relay outputs — one for heating, one for cooling.

### Logic level assumption

**Active-HIGH**: the firmware drives the relay control pin HIGH to energise the
relay (close the circuit) and LOW to de-energise it (open the circuit). This
matches most widely-available relay modules where the input signal is HIGH-active.

> **Verify before first flash:** Some relay modules (particularly those with an
> optocoupler in-circuit) are active-LOW — HIGH de-energises, LOW energises.
> If your module is active-LOW, invert the `RelayController::set()` logic in
> `firmware/esp32c3/src/relays.rs`. This is a one-line change but must be
> confirmed against your specific relay module's datasheet or silkscreen.

### Voltage compatibility

The ESP32-C3 GPIO outputs are 3.3V. Most relay modules accept a 3.3V control
signal, but some require 5V to reliably trigger the optocoupler. Confirm the
relay module's minimum input voltage spec. If 5V input is required, a small
level-shifter (e.g. a BSS138 MOSFET or a dedicated level-shift IC) must be
added between the GPIO pin and the relay module input.

### Mutual exclusion

The firmware enforces that heating and cooling are never active simultaneously
(see `firmware-device` spec: Relay GPIO control requirement). The
`RelayController::set()` function is the single point of truth for relay state.

---

## USB-Serial-JTAG (host communication)

The USB-C port connects directly to the chip's internal USB-Serial-JTAG
peripheral (no GPIO pin number — internal USB PHY). On the Pi host, this
typically enumerates as `/dev/ttyACM0`.

**Baud rate:** 115200 (matching the fixed serial contract; see
`openspec/specs/device-connection/spec.md`).

**GPIO20/21 (UART0)** are the separate hardware serial port on the header pins.
They play no role in host communication for this firmware and are kept free for
future use.

---

## Two-board setup

| Board | Purpose |
|---|---|
| Test board | Development, bring-up, sensor discovery, validation against `fermenter/` with `MOCK_SERIAL=false` |
| Production board | Live controller — flashed with finalised binary after test validation |

Both boards are identical hardware. The same firmware binary (with the same
ROM address constants) can be flashed to both, provided the same two DS18B20
sensors are connected in the same role (fermenter vs. ambient). If different
sensors are used on each board, each board needs its own ROM address constants
compiled in — document which binary is for which board.

---

## Flashing

```bash
# From firmware/esp32c3/
cargo run --release          # flash + open serial monitor (espflash runner)

# Flash only, no monitor
espflash flash --release

# Monitor only (after flashing separately)
espflash monitor
```

If the board does not auto-enter flash mode, manually enter bootloader:
1. Hold BOOT (GPIO9)
2. Press and release RESET (CHIP_PU)
3. Release BOOT
