## 1. Dependencies & scaffolding

- [x] 1.1 Add `tokio-serial` to `Cargo.toml`.
- [x] 1.2 Create `src/serial/arduino.rs` module, `mod arduino;` in `src/serial/mod.rs`.

## 2. Hardware tests FIRST (RED)

- [x] 2.1 Write `tests/serial_hardware.rs`, all `#[ignore]`, reading
      `SERIAL_PORT`/`SERIAL_BAUD` from the environment (default
      `/dev/ttyACM0` @ `115200` if unset):
  - [x] 2.1.1 `#[ignore] fn opens_real_port()` — open the configured port,
        assert success.
  - [x] 2.1.2 `#[ignore] fn reads_a_real_line()` — open the port, call
        `read_line()`, assert the returned string is non-empty, ends where
        expected (no embedded newline), and parses as JSON (don't assert on
        field values — no sensors attached, so `instant`/`average`/`min`/
        `max`/`ambient` may read `-127`-ish per `DEVICE_DISCONNECTED_C`).
  - [x] 2.1.3 `#[ignore] fn write_target_roundtrips()` — call
        `write_target(x)`, then poll `read_line()` a few times until a
        parsed reading reports `target == x`, with a timeout so the test
        fails cleanly instead of hanging if the firmware doesn't ack.
  - [x] 2.1.4 ~~Confirm all three fail to compile/run against a stub (RED)
        before `arduino.rs` exists.~~ **Deviation:** a real Arduino was
        physically attached to this dev machine this session
        (`/dev/ttyACM0`), so `arduino.rs` and the hardware tests were
        developed together and verified directly against real hardware
        rather than staging a strict RED phase first. See task 3.2 for the
        actual (GREEN) run.

## 3. `ArduinoSerialSource` implementation (GREEN)

- [x] 3.1 Implement `SerialSource` for `ArduinoSerialSource` using
      `tokio-serial`: open the port at construction (or lazily on first
      use), `read_line` reads until `\n` and returns the line as UTF-8
      (matching the mock's contract), `write_target` writes the value
      framed as `<value>` per the fixed firmware contract
      (`openspec/specs/device-connection/spec.md`).
- [x] 3.2 Run `cargo test -- --ignored` against the spare Arduino (USB to
      dev machine, no sensors required) and get 2.1.1-2.1.3 to GREEN.
      **Verified:** `cargo test --test serial_hardware -- --ignored
      --test-threads=1` → `3 passed; 0 failed` against the real, sensorless
      Arduino Uno at `/dev/ttyACM0`. Note: the genuine Uno resets on DTR
      toggle whenever the port opens, so the first line after any (re)connect
      can take ~12-13s (setup delays + firmware's 10s print cycle) —
      `READ_TIMEOUT` was bumped from 15s to 20s to avoid flaking on that.

## 4. Reconnect / backoff

- [x] 4.1 Implement exponential backoff (capped) around port-open failures
      and mid-read I/O errors, per the `device-connection` "Serial
      connection resilience" requirement.
      **Implemented** in `ArduinoSerialSource::ensure_connected`/`read_line`:
      500ms initial, doubling, capped at 30s; reset to 500ms on any
      successful open/read.
- [x] 4.2 Add an `#[ignore]`'d test that unplugs (or the human tester
      manually disconnects) the Arduino mid-test and confirms the log shows
      increasing backoff delays and eventual reconnect once replugged.
      (Manual step noted in test doc-comment since USB unplug can't be
      automated without extra hardware.)
      **Deviation:** attempted to automate this via USB sysfs unbind/bind
      (`/sys/bus/usb/drivers/usb/{unbind,bind}`) to simulate an unplug
      without physically touching the cable, but that path is root-only and
      this session has no passwordless `sudo` — didn't prompt for a sudo
      password to do it. Instead added a **deterministic, hardware-free unit
      test** (`src/serial/arduino.rs::tests::backoff_doubles_on_repeated_open_failure_then_caps`,
      using tokio's paused virtual clock) that proves the backoff-doubling
      mechanism itself (500ms → 1s → 2s → 4s → ... capped at 30s) against a
      nonexistent port — this covers the "port never opens" reconnect path
      deterministically and instantly (0.00s). The true hardware-in-the-loop
      "unplug the real Arduino mid-stream and watch it reconnect on replug"
      scenario remains a **manual verification step for you** — see task
      6.4 below.

## 5. Wire `MOCK_SERIAL`

- [x] 5.1 In `main.rs`, branch on `MOCK_SERIAL`: `true` → existing
      `MockSerialSource`, `false` → new `ArduinoSerialSource` built from
      `SERIAL_PORT`/`SERIAL_BAUD`.
- [x] 5.2 Confirm `cargo test` (default, no `--ignored`) still passes
      unchanged — hardware tests stay excluded from the default run.
      **Verified:** 87 lib tests + 6 Redis integration tests pass; the 3
      `serial_hardware` tests show as `ignored`, not run.

## 6. Manual end-to-end verification (dev machine)

- [x] 6.1 Plug the spare Arduino (latest firmware, no sensors) into the dev
      machine via USB; note the assigned device path
      (`/dev/serial/by-id/...` preferred over `/dev/ttyACM0` if multiple
      USB-serial devices are present).
      **Verified:** already attached this session at `/dev/ttyACM0`
      (`/dev/serial/by-id/usb-Arduino__www.arduino.cc__0043_...`), genuine
      Arduino Uno R3, no sensors.
- [x] 6.2 Run `cargo test -- --ignored` and confirm all `serial_hardware.rs`
      tests pass. **Verified** (see task 3.2 log).
- [x] 6.3 Run the binary with `MOCK_SERIAL=false` pointed at the Arduino;
      confirm readings ingest (values will show `~-127` for temperature
      fields — expected without sensors) and a target write via the
      dashboard round-trips into a subsequent reading's `target` field.
      **Verified end-to-end:** ran the release binary against
      `redis://localhost:16379` (a scratch `redis:8` container) with
      `MOCK_SERIAL=false`, `SERIAL_PORT=/dev/ttyACM0`. Readings ingested
      continuously (`average`/`min`/`max`/`ambient` all `-127.0`, as
      expected without sensors). `POST /target` with `22.7` via curl was
      accepted, written to the real device (`target temperature needs
      updating — writing to device desired=22.7 reported=19.5`), and the
      very next reading confirmed `target=22.7`, reflected in `GET
      /status`. Also observed the *startup* reconcile working unprompted:
      config's default target (19.5) differed from the device's own
      power-on default (20.0), so it wrote `<19.5>` immediately and the
      following reading confirmed it.
      **Note:** discovered mid-verification that the pre-existing
      `docker-compose` production Python `controller` container (privileged,
      `network_mode: host`) already had `/dev/ttyACM0` open, blocking the
      Rust binary (`Device or resource busy`) and triggering real
      reconnect/backoff (2s→4s→8s→16s→30s, capped — matching the design).
      Per your direction, stopped that container
      (`docker stop fermenter-temp-controller-controller-1`) for this
      verification and **left it stopped** (not restarted) since you said
      you didn't need it back.
- [x] 6.4 Unplug the Arduino while the binary is running; confirm
      reconnect/backoff logs appear and ingestion resumes cleanly on
      replug.
      **Deviation:** true physical unplug/replug requires touching the
      cable, which I can't do as an agent. I attempted an OS-level
      simulation (USB sysfs unbind/bind) but that's root-only and this
      session has no passwordless `sudo` — didn't prompt you for a sudo
      password to do it (see task 4.2). **Partial substitute observed
      live:** the serial-port-contention incident above (previous bullet)
      incidentally exercised the exact same code path as an unplug would —
      repeated open failures with logged, doubling, capped backoff
      (2s→4s→8s→16s→30s) — until the port became available again, at which
      point ingestion resumed cleanly with no restart needed. This is
      consistent with, but not identical to, a genuine physical
      unplug/replug. **Manual step performed by you:** physically unplugged
      and replugged the Arduino while the binary ran; confirmed good —
      reconnect/backoff behaved as expected (log showed the doubling
      backoff on disconnect, then a clean reconnect and resumed ingestion
      on replug).

## 7. Spec & docs

- [x] 7.1 `openspec validate slice-6-real-hardware` passes.
      **Verified**, along with `cargo fmt --check` and
      `cargo clippy --all-targets -- -D warnings` (both clean) and the full
      default `cargo test` (93 passed, 3 ignored, 0 failed) per
      `docs/rewrite-plan.md` §12.6's CI outline.
- [ ] 7.2 Archive the change once 2-6 are complete and verified, folding the
      `device-connection` scenario additions into
      `openspec/specs/device-connection/spec.md`.
</content>
