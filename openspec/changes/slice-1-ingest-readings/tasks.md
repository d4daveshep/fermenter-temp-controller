## 1. Project scaffolding

- [x] 1.1 Create the `fermenter` Rust binary crate (`Cargo.toml`, `src/main.rs`)
- [x] 1.2 Add dependencies: `tokio`, `serde`/`serde_json`, `async-trait`, `tracing`, `tracing-subscriber`, `thiserror`, `anyhow`, config crate (`figment` or `envy`); dev-dep: `rstest`
- [x] 1.3 Set up module skeleton: `config`, `model`, `error`, `serial` (with `mock` submodule)
- [x] 1.4 Verify the crate builds and `cargo test` runs (empty) green

## 2. Error model (operational-health)

- [x] 2.1 Define the typed `AppError` enum with `thiserror` and a `Result` alias
- [x] 2.2 Add error variants for parse failure and serial I/O failure

## 3. Reading model + validation (temperature-monitoring) — TDD

- [x] 3.1 Write `rstest` cases (RED): valid reading parses; missing required field rejected; `reason-code` defaults when absent; `json-size` accepted but ignored; malformed JSON rejected
- [x] 3.2 Implement `Reading` struct with `serde` derive + field renames to pass the tests (GREEN)
- [x] 3.3 Refactor; confirm all parse tests pass

## 4. Configuration (system-configuration) — TDD

- [x] 4.1 Write config-parse tests (RED): valid env parses into `Config`; missing/invalid required value yields an error; mock-serial toggle parsed
- [x] 4.2 Implement `Config` (serial port, baud, mock-serial toggle, log level) with env parsing + validation (GREEN)
- [x] 4.3 Wire fail-fast: on config error, log and exit non-zero in `main`

## 5. Serial transport abstraction (device-connection) — TDD

- [x] 5.1 Define the `SerialSource` async trait: `read_line` and `write_target`
- [x] 5.2 Write `MockSerialSource` tests (RED): emits scripted lines in order; records `write_target` calls; can inject a read error
- [x] 5.3 Implement `MockSerialSource` to satisfy the trait and tests (GREEN)
- [x] 5.4 Document/spec-note that the real `tokio-serial` impl and reconnect/backoff are deferred to slice-6 (requirement specified, not implemented here)

## 6. Ingest loop wiring (temperature-monitoring + operational-health)

- [x] 6.1 Write a test (RED) that runs the ingest loop against a `MockSerialSource` feeding valid + malformed lines, asserting: valid readings update current state, malformed lines are skipped (logged), loop continues
- [x] 6.2 Implement the ingest loop: read line → parse `Reading` → on Ok update latest-reading state + log; on Err log at `warn` and continue (GREEN)
- [x] 6.3 Initialise `tracing-subscriber` with config-driven level in `main`
- [x] 6.4 Wire `main`: load config (fail-fast) → build mock source (when `MOCK_SERIAL=true`) → run ingest loop, logging readings

## 7. Demo + verification

- [x] 7.1 Run the binary with `MOCK_SERIAL=true` and confirm readings are logged as structured events with no hardware
- [x] 7.2 Run `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` clean
- [x] 7.3 Confirm full `cargo test` suite passes
- [x] 7.4 Run `openspec validate slice-1-ingest-readings --strict`
