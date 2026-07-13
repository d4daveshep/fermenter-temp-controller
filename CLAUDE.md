# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Arduino + Raspberry Pi fermentation temperature controller. The Arduino reads temperature sensors and controls heating/cooling relays; a single Rust binary on the Raspberry Pi reads the Arduino over serial, persists readings to Redis Time Series, and serves an Axum + MiniJinja + HTMX web dashboard. This replaced an earlier Python/InfluxDB implementation at cutover — see `docs/rewrite-plan.md` for the full rewrite history and design.

## Repository Boundaries

- **`fermenter/`** — the Rust application: source of truth for all runtime behavior.
- **`src/`** — untracked artifact left over from the pre-rewrite Python stack (`__pycache__`, old `.egg-info`). Ignore it; it is not source code.
- **`arduino/TempController/`** — the Arduino firmware (C++), unchanged by the rewrite.
- **`docs/`** — planning history for the rewrite (`rewrite-plan.md`, `system-analysis.md`, `openspec-rewrite-management.md`); background, not live operational instructions.
- **`openspec/`** — drives ongoing feature work: proposals, specs, and archived changes documenting the capability library.

## Running the Application

```bash
cp fermenter/.env.example fermenter/.env   # edit SERIAL_PORT etc. for this host
docker compose up                          # from the repo root: starts fermenter + redis
```

For local development without Docker:

```bash
cd fermenter
cargo run
```

## Running Tests

```bash
cd fermenter
cargo test                    # unit + integration (spins up redis:8 via testcontainers; needs Docker)
cargo test -- --ignored       # also run hardware tests (needs a real Arduino attached)
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

CI (`.github/workflows/rust.yml`) runs `fmt`, `clippy`, `test`, and a build-only cross-compiled `docker buildx build --platform linux/arm64 --features embed` job. Hardware tests are `#[ignore]`'d and never run in CI.

## Architecture

### Rust application (`fermenter/`)

- **`src/serial/`** — `SerialSource` trait; `arduino.rs` (real `tokio-serial` transport) and `mock.rs` (hardware-free dev/test feed).
- **`src/store/`** — `TimeStore` trait; Redis Time Series implementation (`redis.rs`). In-memory state is authoritative; Redis is a durability mirror.
- **`src/web/`** — Axum router, handlers, MiniJinja rendering. Templates live in `fermenter/templates/`, loaded from disk in dev builds and baked into the binary at compile time with `--features embed` (used for the release Docker image).
- **`src/model.rs`** — `Reading`, `ControllerState`, and related types shared across serial, store, and web layers.
- **`src/ingest.rs`**, **`src/temperature_control.rs`**, **`src/brew_session.rs`** — ingest loop, target-temperature reconcile, and brew-id relabeling logic.
- **`src/config.rs`** — env-var configuration (`envy`), fail-fast on invalid config.

### Serial contract (fixed, shared with firmware)

Newline-delimited JSON readings at 115200 baud (`instant`, `average`, `min`, `max`, `target`, `ambient`, `action`, `reason-code`); the app writes the target temperature back as a `<float>` framed string (e.g. `<20.5>`). This contract must not change without a corresponding Arduino firmware change.

### Arduino firmware (`arduino/TempController/`)

C++ sketch targeting an Arduino Mega/Uno with:
- **`TempController.ino`** — main sketch; toggles between unit-test mode (`#define _DO_UNIT_TESTING`) and production mode.
- **Libraries** bundled as zips in `arduino/install/`: AUnit, ArduinoJson, DallasTemperature, OneWire.

**Build / upload Arduino firmware** (uses legacy `arduino` CLI, not `arduino-cli`):
```bash
cd arduino/TempController
./compile_arduino.sh    # arduino --verify TempController.ino
./upload_arduino.sh
```

### Configuration (env vars)

Recognised variables (all have in-code defaults except where noted), documented in `fermenter/.env.example`:

```
SERIAL_PORT=/dev/ttyACM0
SERIAL_BAUD=115200
MOCK_SERIAL=false
REDIS_URL=redis://redis:6379
TS_RETENTION_DAYS=7
DEFAULT_BREW_ID=00-TEST-v00
HTTP_PORT=8080
DEFAULT_TARGET_TEMP=19.5
RUST_LOG=info
```
</content>
