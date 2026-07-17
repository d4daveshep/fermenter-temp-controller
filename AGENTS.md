# OpenCode Agent Instructions

## Repository Boundaries & Source of Truth
- **The Rust application lives in `fermenter/`.** This is the single Tokio-based binary (serial ingest + Axum web server) that replaced the old Python/InfluxDB stack at cutover (see `docs/rewrite-plan.md`).
- **Arduino Firmware:** The `arduino/TempController/` directory contains the C++ firmware for the Arduino. It uses the older `arduino` CLI (not `arduino-cli`) for building and uploading via `compile_arduino.sh` and `upload_arduino.sh`. Dependencies are bundled as zips in `arduino/install/` (like AUnit, ArduinoJson, DallasTemperature). The firmware and serial contract are unchanged by the rewrite.
- **`docs/`** contains the rewrite's planning history (`rewrite-plan.md`, `system-analysis.md`, `openspec-rewrite-management.md`) — useful background, not live operational instructions.
- **OpenSpec** (`openspec/`) drives ongoing feature work: proposals, specs, and archived changes documenting the capability library. Change folders are named `slice-N-verb-noun`; a slice that changes an existing capability carries a MODIFIED delta against that capability's spec, never a silent redefinition (see `openspec/config.yaml`).

## Architecture & Quirks
- **Templating:** MiniJinja templates live in `fermenter/templates/`, loaded from disk in dev builds (`cargo run`/`cargo test`) and baked into the binary at compile time when built with `--features embed` (used for the release Docker image). Don't assume templates are always on-disk relative to the binary.
- **Serial contract is fixed:** newline-delimited JSON readings at 115200 baud; the app writes the target temperature back as a `<float>` framed string. This is shared with the Arduino firmware and must not change without a corresponding firmware change.
- **State store:** Redis 8 Time Series (`fermenter/src/store/`), not a SQL/InfluxDB store. In-memory state is authoritative; Redis is a durability mirror.

## Execution & Testing
- **Local dev:** `cargo run` / `cargo test` from `fermenter/` (requires a local Redis for integration tests, or use `cargo test` which spins up `redis:8` via `testcontainers` — needs Docker on the test host).
- **Full stack:** `docker compose up` from the repo root orchestrates `fermenter` + `redis` via the repo-root `compose.yaml`. Copy `fermenter/.env.example` to `fermenter/.env` first.
- **CI:** `.github/workflows/rust.yml` runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and a build-only cross-compiled `docker buildx build --platform linux/arm64` job.
- **Hardware tests:** `fermenter/tests/serial_hardware.rs` is `#[ignore]`'d by default — only run with `cargo test -- --ignored` on a machine with a real Arduino attached.
- **Snapshot tests:** `fermenter/src/web/handlers.rs` uses `insta::assert_snapshot!` for HTML fragments (snapshots in `fermenter/src/web/snapshots/`). After changing templates, update snapshots with `INSTA_UPDATE=always cargo test` or `cargo insta accept`.

## Deployment
- **Cross-compile + ship to Pi:** From an annotated SemVer Git tag, `scripts/build_and_ship_image.sh pi@<host>` builds `fermenter:<tag>` via QEMU (`docker buildx build --platform linux/arm64`), saves it to a tarball, and scp's it to the Pi. Then on the Pi: `gunzip -c fermenter-<tag>.tar.gz | docker load && FERMENTER_IMAGE_TAG=<tag> docker compose up -d` — Compose's `image:` + `build:` picks up the loaded image without rebuilding.
</content>
