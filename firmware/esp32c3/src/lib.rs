#![no_std]

//! Shared modules between `main.rs` and `examples/*.rs` — Cargo
//! auto-detects this `lib.rs` alongside `main.rs`'s `[[bin]]` target and
//! links the binary and every example against it automatically.

pub mod relays;
pub mod sensors;
