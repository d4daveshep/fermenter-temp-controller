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

### Exposed GPIO pins (22-pin header)

**Correction:** an earlier draft of this document described a 16-pin header
(2 rows of 8). The board actually purchased for this build
(nznelectronics.co.nz, "ESP32-C3 SuperMini Dev Board – USB-C") has a
**22-pin header — 11 pins per side**, confirmed against the vendor's product
photo (two loose 11-pin header strips, one per edge, matching 11 castellated
pads per edge on the board itself). The GPIO set itself is unchanged from
what was previously documented — this was a header-pin-count/table error, not
a GPIO error.

**Confirmed from the product photo and vendor spec table:**

| Fact | Value | Confidence |
|---|---|---|
| Total header pins | 22 (11 per side) | High — counted directly from product photo |
| Total GPIOs broken out | 13: GPIO0–10, GPIO20, GPIO21 | High — consistent across vendor spec ("11× GPIO... 2× UART") and every independent reference checked |
| Far-end pad, one side | GPIO0 | High — legible silkscreen label in product photo |
| Far-end pad, other side | GPIO21 | High — legible silkscreen label in product photo |
| Strapping pins | GPIO2, GPIO8, GPIO9 | High — Espressif ESP32-C3 technical reference; unaffected by the pin-count correction |
| Onboard LED | Blue, GPIO8, active LOW | High — matches vendor spec table |
| BOOT button | GPIO9 | High — matches vendor spec table |
| Power pins (5V, GND, 3V3) | Cluster at the end of the header nearest the USB-C connector | Medium — legible in the product photo but not fully confirmed pin-by-pin |
| Exact left/right pin-by-pin order for the remaining ~9 non-anchor pins (GPIO1–10, GPIO20, and the power pins on each side) | Not yet confirmed | **Low — still open** (the 3 pins this firmware actually uses are confirmed; see below) |

**Status:** task 12.1's silkscreen read confirmed `ONE_WIRE_PIN` (GPIO4),
`HEAT_RELAY_PIN` (GPIO0), and `COOL_RELAY_PIN` (GPIO1) are correctly wired —
verified indirectly by every real-hardware test since (sensor reads, relay
clicks, the full control loop). The remaining ~9 unused header pins'
left/right silkscreen order was never read/recorded, since nothing in this
project needs them; still genuinely open if a future feature needs one of
them.

**Safest pins for new assignments (no system duties):** GPIO0, GPIO1, GPIO3, GPIO10.
GPIO4 (JTAG TMS) is also usable — it's not one of the three strapping pins
(GPIO2, GPIO8, GPIO9) and this project has no JTAG debugger connected, so its
JTAG duty is never exercised. See `ONE_WIRE_PIN` below, which uses it.

---

## Signals required by this firmware

Three signals needed: one OneWire data bus, two relay outputs.

### GPIO pin assignments

**Status:** All three pins are now **confirmed** on real hardware.
`ONE_WIRE_PIN` (GPIO4) matches the test board's original wiring.
`HEAT_RELAY_PIN`/`COOL_RELAY_PIN` (GPIO0/GPIO1) were locked in task 12.1 and
verified in task 12.3 — first via multimeter with no relays attached
(confirmed correct HIGH/LOW logic per action), then re-confirmed with real
relays wired up (both clicked in the correct Rest/Heat/Cool/Rest order), and
a third time after the production-board swap (task 17.2).

| Signal | GPIO | Notes |
|---|---|---|
| `ONE_WIRE_PIN` | GPIO4 | OneWire data bus (DS18B20 × 2). JTAG TMS, but safe — see note above; confirmed by real hardware, both on the test board and the production board |
| `HEAT_RELAY_PIN` | GPIO0 | Heating relay output — confirmed wired and verified (multimeter + real relay clicks) |
| `COOL_RELAY_PIN` | GPIO1 | Cooling relay output — confirmed wired and verified (multimeter + real relay clicks) |

Rationale: GPIO0 and GPIO1 are the safest general-purpose pins with no JTAG,
SPI, strapping, or LED duty. Record the final values as:

```rust
// firmware/esp32c3/src/relays.rs
const HEAT_RELAY_PIN: u8 = 0;
const COOL_RELAY_PIN: u8 = 1;

// firmware/esp32c3/src/sensors.rs
const ONE_WIRE_PIN: u8 = 4;
```

---

## Control loop tuning constants

Ported verbatim from `arduino/TempController/TempController.ino` and
`ControllerActionRules.cpp` (see design.md Decision 9). These are not
hardware-specific, but like the GPIO pins above they are load-bearing values
that must be locked in as named constants rather than re-derived or guessed
during implementation:

```rust
// firmware/logic/src/controller_action_rules.rs
const TARGET_RANGE: f64 = 0.3;                    // target ± 0.3, failsafe ± 0.6
const COOLING_OVERRUN_ADJUSTMENT: f64 = 0.2;
const DEFAULT_TARGET_TEMP: f64 = 20.0;            // overwritten once the host writes its own target

// firmware/esp32c3/src/main.rs
const FERMENTER_EMA_WINDOW: u32 = 60;
const AMBIENT_EMA_WINDOW: u32 = 10;
```

---

## DS18B20 Temperature Sensors

Two sensors: one measuring **fermenter temperature** (inside the fermenter),
one measuring **ambient temperature** (room/environment).

### Identification

Sensors are addressed by **64-bit ROM address**, not by bus-scan index
(see design.md Decision 3). ROM addresses were discovered on the production
board via `examples/identify_sensors.rs` (task 11) and physically identified
by warming the fermenter sensor and watching which address's live reading
rose — see `firmware/README.md`.

```rust
// firmware/esp32c3/src/sensors.rs — real, hardware-confirmed values
const FERMENTER_SENSOR_ADDR: [u8; 8] = [0x28, 0xF2, 0x92, 0x37, 0x07, 0x00, 0x00, 0x82];
const AMBIENT_SENSOR_ADDR:   [u8; 8] = [0x28, 0xF7, 0xF2, 0xB5, 0x09, 0x00, 0x00, 0x52];
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
with multiple sensors on one bus and complicates the conversion-timing
sequence the `onecable` driver crate expects (design.md Decision 12).
External power avoids those workarounds entirely.

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

**Status:** the production sensors and ROM addresses (see "Identification"
above) were discovered and confirmed directly on the board that was then
deployed — relays and sensors verified on real hardware (task 12.3, 13.2),
soak-tested for 10+ minutes (task 17.1), then physically swapped in for the
Arduino Uno and re-validated in place (task 17.2). It is now the live
controller.

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
