# syntax=docker/dockerfile:1
#
# Multi-stage build for the fermenter controller (slice-7: deployment-packaging).
#
# Intended usage — cross-compiled on the dev machine, not built natively on
# the Raspberry Pi (rewrite-plan.md §11):
#
#   docker buildx build --platform linux/arm64 -t fermenter:arm64 .
#
# `--platform linux/arm64` makes buildx select the arm64 variant of every
# base image below and run both stages under QEMU user-mode emulation (via
# binfmt_misc), so `cargo build --release` here produces a genuine arm64
# binary without needing a separate cross-toolchain / target triple. This is
# slower than a native x86_64 build (QEMU emulation overhead) but requires no
# extra host setup beyond `docker buildx create --use` with binfmt support
# (see the repo's deployment docs / tasks.md task 2.2).
#
# The `embed` feature (added this slice) bakes `templates/` into the binary
# at compile time, so the runtime stage below needs no `templates/` volume —
# only the compiled binary and `static/` are copied into it.

# ---- Builder --------------------------------------------------------------
FROM rust:1-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock build.rs ./
COPY src ./src
COPY templates ./templates
COPY static ./static

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/app/target \
    cargo build --release --features embed && \
    cp target/release/fermenter /app/fermenter

# ---- Runtime ---------------------------------------------------------------
# debian:bookworm-slim over distroless (rewrite-plan.md §11 leaves this as an
# open choice): larger image, but keeps a shell available for exec-ing in
# during early deployment debugging (design.md decision, slice-7).
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# curl: neither wget nor curl exist in debian:bookworm-slim by default;
# curl is needed for compose.yaml's healthcheck (`GET /healthz`).
RUN apt-get update \
    && apt-get install -y --no-install-recommends curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/fermenter /usr/local/bin/fermenter
COPY --from=builder /app/static ./static

EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/fermenter"]
