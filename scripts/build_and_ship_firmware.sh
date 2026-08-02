#!/usr/bin/env bash
#
# Builds the ESP32-C3 firmware release binary (firmware/esp32c3, target
# riscv32imc-unknown-none-elf) on this machine and scp's it to a Raspberry
# Pi that has the board attached over USB. Assumes firmware/logic's tests
# and firmware/esp32c3's build have already passed here and in CI (see
# .github/workflows/rust.yml's firmware-logic/firmware-esp32c3 jobs) — this
# script does not run tests itself.
#
# Mirrors build_and_ship_image.sh's build-here/ship/manual-step-there shape:
# it deliberately stops short of flashing anything on the Pi itself — that
# stays a deliberate, manual step (same "no automated Pi deployment" stance
# as the docker image script), printed at the end as a copy-paste next
# step. Reflashing resets the board and briefly interrupts live
# fermentation control, so it shouldn't happen unattended.
#
# Usage:
#   ./scripts/build_and_ship_firmware.sh pi@raspberrypi.local
#   PI_HOST=pi@raspberrypi.local ./scripts/build_and_ship_firmware.sh
#
# Optional environment overrides:
#   REMOTE_PATH   (default: ~/)
#   OUTPUT_DIR    (default: dist/, gitignored)
#
# Prerequisites:
#   - This machine: the embedded Rust toolchain set up per
#     firmware/README.md (`rustup target add riscv32imc-unknown-none-elf` —
#     no `espup` needed for this RISC-V target).
#   - The Pi: `espflash` installed (`cargo install espflash`) and the board
#     attached over USB.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FIRMWARE_DIR="$REPO_ROOT/firmware/esp32c3"

REMOTE_PATH="${REMOTE_PATH:-~/}"
OUTPUT_DIR="${OUTPUT_DIR:-dist}"

PI_HOST="${1:-${PI_HOST:-}}"
if [[ -z "$PI_HOST" ]]; then
  echo "Usage: $0 <user@pi-host>  (or set PI_HOST env var)" >&2
  exit 1
fi

VERSION_LABEL="$(git -C "$REPO_ROOT" describe --always --dirty)"
ARTIFACT_NAME="esp32c3-${VERSION_LABEL}"

# No --features here, deliberately: `debug-log` must stay off for any
# binary that will run connected to a live `fermenter/` host — its
# esp_println output shares the same USB-Serial-JTAG stream as the JSON
# telemetry protocol (see firmware/README.md, design.md Decision 7).
echo "==> Building firmware/esp32c3 release binary (${VERSION_LABEL})"
( cd "$FIRMWARE_DIR" && cargo build --release )

BIN_PATH="$REPO_ROOT/firmware/target/riscv32imc-unknown-none-elf/release/esp32c3"

mkdir -p "$REPO_ROOT/$OUTPUT_DIR"
ARTIFACT_PATH="$REPO_ROOT/$OUTPUT_DIR/$ARTIFACT_NAME"
cp "$BIN_PATH" "$ARTIFACT_PATH"
ls -lh "$ARTIFACT_PATH"

echo "==> Copying ${ARTIFACT_NAME} to ${PI_HOST}:${REMOTE_PATH}"
scp "$ARTIFACT_PATH" "${PI_HOST}:${REMOTE_PATH}"

cat <<EOF

==> Done. On the Pi, run:

    espflash flash --non-interactive ${REMOTE_PATH%/}/${ARTIFACT_NAME}

This resets the board and briefly interrupts fermentation control while it
reflashes and reboots — confirm that's acceptable before running it. The
board resumes its normal control loop automatically once booted; no
restart needed on the fermenter/ side (the serial contract is unchanged).
EOF
