## Why

The Rust rewrite needs a foundation: a running binary that connects to the
Arduino, reads and validates temperature readings, and observes them. This first
vertical slice establishes the three most stable, foundational contracts — the
serial transport, the reading data model, and the configuration surface — so
every later slice amends behaviour rather than redefining bedrock. It is
demoable (readings flow in from a mock source and are logged) without requiring
hardware.

## What Changes

- Introduce a single Rust binary (Tokio runtime) that ingests temperature
  readings from a serial source and logs them as structured events.
- Define the **serial transport contract**: newline-delimited JSON readings at
  115200 baud, with a `SerialSource` trait abstracting the transport so a mock
  feed can drive the system without an Arduino.
- Define the **`Reading` data model** and its strict validation: required
  fields parsed, malformed lines logged and skipped (not silently swallowed as
  in the old system), the legacy `json-size` field ignored.
- Define the **configuration surface**: 12-factor env vars parsed once at
  startup into a typed `Config`, failing fast on invalid config.
- Establish **structured logging and a typed error model** as the observability
  baseline.
- Specify (but defer implementing) **serial auto-reconnect with exponential
  backoff** as part of the transport contract, so the resilience requirement is
  fixed up front.

Out of scope for this slice: persistence (Redis), the web UI, target-temp
control, brew-id lifecycle, real hardware wiring, and Docker packaging. Those
arrive in later slices.

## Capabilities

### New Capabilities
- `device-connection`: The serial transport contract — baud rate, newline-JSON
  framing, the `SerialSource` abstraction, and reconnect/backoff behaviour.
- `temperature-monitoring`: Ingesting serial lines into validated `Reading`
  values and exposing the latest observed reading as current state.
- `system-configuration`: Loading and validating 12-factor environment
  configuration at startup with fail-fast semantics.
- `operational-health`: The structured-logging and typed-error baseline used
  across the application.

### Modified Capabilities
<!-- None: this is the first slice; the spec library is empty. -->

## Impact

- **New crate**: a Rust binary (`fermenter`) with modules `config`, `model`,
  `error`, and `serial` (trait + mock impl); the real `tokio-serial` impl is
  deferred to a later slice.
- **Dependencies introduced**: `tokio`, `serde`/`serde_json`, `async-trait`,
  `tracing`/`tracing-subscriber`, `thiserror`/`anyhow`, a config crate
  (`figment` or `envy`); dev: `rstest`.
- **No impact** on the existing Python system, which remains the deployable
  stack on `master` until cutover.
- **Contract dependency**: the serial JSON shape and baud rate are fixed by the
  unchanged Arduino firmware.
