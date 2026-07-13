# Fermentation Temperature Controller

## About

This project contains the software for my Arduino and Raspberry Pi based fermentation temperature controller.

## Architecture

An Arduino reads temperature sensors and drives heating/cooling relays; a Raspberry Pi runs a single Rust binary (Axum + Tokio) that reads the Arduino over USB serial, persists readings to Redis Time Series, and serves a web dashboard (MiniJinja + HTMX) for monitoring and runtime control (target temperature, brew ID). See `docs/rewrite-plan.md` for the full design.

## Source code

- `fermenter/` — the Rust application (serial ingest, Redis persistence, web dashboard)
- `arduino/TempController/` — the Arduino firmware
- `docs/` — design docs, including the history of the Python → Rust rewrite

## Build process

`cargo build`/`cargo test` inside `fermenter/` for local development. See `INSTALL.md` for the full setup, and `fermenter/Dockerfile` for the cross-compiled ARM64 release image.

## Deployment

`docker compose up` from the repository root (see `INSTALL.md`).
</content>
