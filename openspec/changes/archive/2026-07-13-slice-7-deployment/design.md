## Context

Slices 1-6 built and proved the full controller — ingest, persistence
(Redis TS), dashboard (Axum + MiniJinja + HTMX), target control, brew
sessions, and a real `tokio-serial` transport verified against a physical
Arduino — but only as `cargo run`/`cargo test` on a dev machine:

- `fermenter/src/web/mod.rs::build_environment()` unconditionally uses
  `minijinja::path_loader("templates")`, which requires the `templates/`
  directory to exist relative to the process's working directory. There is
  no cargo feature flag at all yet in `fermenter/Cargo.toml`.
- `fermenter/static/` (currently just `styles.css`) is served via
  `ServeDir::new("static")`, also resolved relative to the working
  directory.
- No `Dockerfile` or `compose.yaml` exists for the Rust stack. The only
  Dockerfiles in the repo (`Dockerfile.controller`, `Dockerfile.web_apis`,
  `Dockerfile.pytests`) belong to the old Python/InfluxDB system and are
  explicitly out of scope (`AGENTS.md`: "ignore `src/`"; these are for the
  Python stack only).
- The old Python stack's `docker-compose.yaml` `controller` service runs
  `privileged: true` with `network_mode: host` to get at the serial device
  — this was directly observed during slice-6's manual verification, where
  it held `/dev/ttyACM0` open and blocked the new Rust binary until stopped.
  `rewrite-plan.md` §11 explicitly calls out scoped device passthrough
  (`devices: ["/dev/ttyACM0:/dev/ttyACM0"]`) as the intended improvement.
- `/healthz` (slice-3) reports only `serial_connected`; a `redis_connected`
  field was explicitly deferred in that slice's design doc pending a status
  accessor on `RedisTimeStore`'s `ConnectionManager`, and no later slice
  picked it up. Per `docs/openspec-rewrite-management.md` §7's slice-map
  matrix, `operational-health` is untouched (`·`) at slice-7 — this slice
  does not revisit that gap; it wires the healthcheck to what exists today.
- `redis:8` (used since slice-2) bundles Time Series and is 64-bit only —
  `rewrite-plan.md` §16 already notes this is "fine for a modern Pi"
  (Raspberry Pi OS 64-bit / aarch64).
- `fermenter/Cargo.toml` targets `edition = "2024"`, which requires a
  reasonably current `rustc` (1.85+) in whatever builder image is chosen.

## Goals / Non-Goals

**Goals:**
- A cargo `embed` feature that bakes MiniJinja templates into the binary at
  compile time (via `minijinja-embed`), so the release image needs no
  mounted `templates/` volume, while leaving the existing dev-mode
  `path_loader` behavior completely unchanged when the feature is off.
- A multi-stage `Dockerfile`: a builder stage that cross-compiles
  `cargo build --release --features embed` for `linux/arm64`, and a slim
  runtime stage that copies only the compiled binary and `static/`.
- A `compose.yaml` for the new 2-container stack (`fermenter` + `redis:8`),
  with scoped serial device passthrough (not `privileged`), config via
  `env_file`, and a `healthcheck:` calling `/healthz`.
- A CI job that builds (never runs) the cross-compiled image, so a broken
  Dockerfile/build is caught without needing ARM64 hardware or a real
  Arduino in CI.
- A verified deployment to the actual Raspberry Pi: image runs, readings
  ingest from the real production Arduino, dashboard updates, and
  `cargo test -- --ignored` passes on the Pi.

**Non-Goals:**
- No `redis_connected` addition to `/healthz` — explicitly out of scope per
  the slice-map matrix (`operational-health` stays untouched this slice);
  the Compose healthcheck uses the endpoint as it exists today
  (`serial_connected` only), which is an incomplete but non-regressive
  signal (the old Python system had no health endpoint at all).
- No deletion of the old Python stack (`controller/`, `web/`, old
  Dockerfiles, `docker-compose.yaml`, `pyproject.toml`) — that's the
  separate "cutover" step in `rewrite-plan.md` §15 Phase 3, gated on this
  slice being proven on the Pi first. Both stacks coexist after this slice
  lands.
- No changes to Arduino firmware, the serial contract, or any capability's
  runtime behavior — this slice is packaging and deployment only.
- No automated Pi deployment (e.g. Ansible/CD pipeline) — manual deploy
  (copy/build image on the Pi, or `docker buildx --push` to a registry the
  Pi pulls from) is sufficient for milestone 8; automating the deploy step
  itself is not called for by the plan.

## Decisions

- **`minijinja-embed` for template embedding, gated behind an `embed` cargo
  feature.** This crate exists on crates.io under exactly the name
  `rewrite-plan.md` §6 anticipated, and pairs naturally with the existing
  `minijinja::Environment`. `build_environment()` branches on
  `#[cfg(feature = "embed")]`: with the feature, load from the
  compile-time-embedded template source; without it, keep today's
  `path_loader("templates")` unchanged, so dev workflow (`cargo run`,
  `cargo test`) is unaffected. Alternative considered: plain
  `include_str!` per template, manually wired into a custom loader —
  rejected as more boilerplate than `minijinja-embed` for the same result,
  and less idiomatic given the plan already named this crate.
- **Multi-stage `Dockerfile`, builder + slim runtime.** Builder:
  `rust:1-bookworm` (tracks current stable, satisfying `edition = "2024"`),
  cross-compiling for `linux/arm64` via `docker buildx` (using QEMU
  emulation or a `cross`-style toolchain) on the dev machine — matching
  `rewrite-plan.md` §11's explicit "cross-compiled on dev machine" decision
  rather than building natively on the Pi. Runtime: `debian:bookworm-slim`,
  copying only the compiled binary and `static/` (templates are embedded,
  not copied — no `templates/` in the final image at all with the `embed`
  feature on). Alternative considered: distroless runtime image — left as
  an explicit option in `rewrite-plan.md` §11 itself ("or distroless");
  `debian:bookworm-slim` chosen for this slice for easier debugging
  (a shell exists for exec-ing in) with room to switch later.
- **`compose.yaml` mirrors the plan's example almost verbatim**
  (`rewrite-plan.md` §11): `redis:8` with a named volume and
  `--save 60 1`, `fermenter` built from the new `Dockerfile`, `env_file:
  .env`, scoped `devices:` passthrough, `depends_on: [redis]`,
  `restart: on-failure`. A `healthcheck:` stanza is added on the
  `fermenter` service (not in the plan's original snippet) calling
  `GET /healthz` — the natural place for it now that the endpoint exists
  (it didn't yet when `rewrite-plan.md` §11 was written).
- **CI build job only builds, never runs, the image** (mirrors
  `rewrite-plan.md` §12.6: "Build job: `docker buildx` cross-compile for
  `linux/arm64` with `--features embed` to verify the release image
  builds"). Running the image would need either QEMU-emulated execution
  (slow, and still not real hardware) or actual ARM64/Arduino hardware,
  neither available in CI — this mirrors why `tests/serial_hardware.rs`
  stays `#[ignore]`'d.
- **Pi deployment is manual, not automated in this slice.** Build the image
  via `buildx` on the dev machine (already cross-compiling for
  `linux/arm64`), transfer or push it, then `docker compose up` on the Pi.
  No CD pipeline — matches the plan's milestone 8 framing ("Pi deploy +
  verify: end-to-end on hardware") as a verification step, not a new
  automation capability.
- **No `redis_connected` in `/healthz` this slice** (see Non-Goals) —
  keeping this slice's spec delta scoped purely to the new
  `deployment-packaging` capability, matching
  `docs/openspec-rewrite-management.md`'s documented slice-map (§7:
  `operational-health` is `·` at s7).

## Risks / Trade-offs

- **[Risk] Cross-compiling Rust for `linux/arm64` from an x86_64 dev
  machine can be slow or fiddly** (QEMU emulation overhead, or needing the
  `cross` tool / a properly configured `buildx` builder with binfmt
  support) → **Mitigation:** documented as a one-time `buildx` builder setup
  step in tasks.md; CI job surfaces breakage early without needing to wait
  for a Pi deploy to discover it.
- **[Risk] `debian:bookworm-slim` may be missing a runtime dependency the
  binary needs** (e.g. `libudev`/`libssl` if any transitive dependency
  dynamically links against them — `tokio-serial`'s Linux backend uses
  `libudev`-independent syscalls, but this should be verified once the
  Dockerfile exists) → **Mitigation:** CI build job fails fast if the final
  image can't produce a working binary; a `docker run --rm <image>
  --help`-style smoke check (or just `/healthz` via the CI-built container
  briefly under QEMU) can catch missing shared libraries before Pi deploy.
- **[Risk] Device path differs between the dev machine (`/dev/ttyACM0`
  during slice-6) and the Pi** (could be `/dev/ttyACM0`, `/dev/ttyUSB0`, or
  a `by-id` symlink depending on what else is attached) → **Mitigation:**
  `SERIAL_PORT` is already an env var read via `.env` (`system-configuration`
  capability, already implemented); no code change needed, just correct
  `.env`/`compose.yaml` `devices:` values on the Pi at deploy time.
- **[Risk] Scoped device passthrough (non-privileged) might behave
  differently under permissions than the old `privileged: true` setup** — the
  container's user needs read/write on the passed-through device node
  → **Mitigation:** verified directly during slice-6 on the dev machine
  (non-containerized) that a `dialout`-group user can open the port; the
  container's runtime user/group needs equivalent access — call out as an
  explicit Pi-verification task rather than assuming it silently works.
- **[Risk] Running the new `fermenter` stack and the old privileged Python
  stack simultaneously on the same host could re-create the exact port
  contention discovered in slice-6** (the old stack grabs any
  `/dev/ttyACM0`-like device via `privileged: true`) → **Mitigation:**
  documented as an operational note for Pi deployment — stop the old stack
  before starting the new one for verification, exactly as slice-6 required
  on the dev machine; this is inherent to running both stacks
  simultaneously and resolves itself at cutover (`rewrite-plan.md` §15
  Phase 3) when the old stack is removed.
- **[Trade-off] `debian:bookworm-slim` over distroless** — larger image,
  easier debugging; acceptable per `rewrite-plan.md` §11 explicitly leaving
  this as an open choice ("or distroless").

## Migration Plan

Additive — introduces a new, independent 2-container stack (`compose.yaml`)
alongside the existing `docker-compose.yaml` Python stack; nothing about the
old stack changes or is removed in this slice. Rollback is simply not
running the new stack (`docker compose down` on the new file) and
continuing to run the old one, exactly as today. The eventual old-stack
removal is the separate, later cutover step (`rewrite-plan.md` §15 Phase 3),
explicitly gated on this slice being proven on the Pi first — not performed
here.

## Open Questions

- Registry vs. direct transfer for getting the cross-compiled image onto the
  Pi (`docker save`/`scp`/`docker load` vs. a real registry push/pull) —
  either works for milestone 8's verification purposes; left to whichever
  is more convenient at deploy time rather than mandated here.
- Whether to also add a `docker run --rm <image> <binary> --version`-style
  smoke test to CI (beyond "the build succeeds") to catch missing runtime
  shared libraries earlier — nice-to-have, not required by
  `rewrite-plan.md` §12.6's stated CI scope ("verify the release image
  builds"); left as a tasks.md stretch item rather than a hard requirement.
