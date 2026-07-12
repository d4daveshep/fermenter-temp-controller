## 1. Template embedding (`embed` feature)

- [ ] 1.1 Add `minijinja-embed` dependency to `fermenter/Cargo.toml`, gated
      behind a new `embed` feature (`[features] embed = ["dep:minijinja-embed"]`
      or equivalent).
- [ ] 1.2 Add a `build.rs` (if required by `minijinja-embed`) that embeds the
      `templates/` directory at compile time.
- [ ] 1.3 Update `fermenter::web::build_environment()` to branch on
      `#[cfg(feature = "embed")]`: embedded loader when enabled, existing
      `path_loader("templates")` unchanged when disabled.
- [ ] 1.4 Add/extend a template smoke test that runs under `--features embed`
      (e.g. a second CI-style `cargo test --features embed`) confirming a
      known template renders identically with and without the feature.
- [ ] 1.5 Confirm plain `cargo run`/`cargo test` (no `--features embed`)
      behavior is byte-for-byte unchanged from before this slice.

## 2. Dockerfile

- [ ] 2.1 Write `fermenter/Dockerfile`, multi-stage:
  - [ ] 2.1.1 Builder stage: `rust:1-bookworm`, cross-compile
        `cargo build --release --features embed` for `linux/arm64`.
  - [ ] 2.1.2 Runtime stage: `debian:bookworm-slim`, copy only the compiled
        binary and `static/`.
- [ ] 2.2 Set up a local `docker buildx` builder with `linux/arm64` support
      (QEMU binfmt or equivalent) on the dev machine, documented in a
      comment or short README note for reproducibility.
- [ ] 2.3 Build the image locally
      (`docker buildx build --platform linux/arm64 --features embed -t
      fermenter:arm64 fermenter/`) and confirm it succeeds.
- [ ] 2.4 Sanity-check the runtime image doesn't need a missing shared
      library (e.g. `docker run --rm --platform linux/arm64 fermenter:arm64
      --help` or equivalent under QEMU) — see design.md's open question on
      whether this becomes a permanent CI smoke test.

## 3. Compose orchestration

- [ ] 3.1 Write `compose.yaml` (new stack, 2 containers):
  - [ ] 3.1.1 `redis` service: `redis:8`, named volume, `--save 60 1`.
  - [ ] 3.1.2 `fermenter` service: `build: ./fermenter`, `env_file: .env`,
        scoped `devices: ["/dev/ttyACM0:/dev/ttyACM0"]` (not `privileged`),
        `depends_on: [redis]`, `restart: on-failure`.
  - [ ] 3.1.3 `healthcheck:` on the `fermenter` service calling `/healthz`.
- [ ] 3.2 Write/update a `.env.example` documenting the required variables
      (`SERIAL_PORT`, `SERIAL_BAUD`, `MOCK_SERIAL`, `REDIS_URL`,
      `TS_RETENTION_DAYS`, `DEFAULT_BREW_ID`, `HTTP_PORT`,
      `DEFAULT_TARGET_TEMP`, `RUST_LOG`) per `system-configuration`
      (already implemented — this just documents existing vars for the new
      deployment path).
- [ ] 3.3 Bring the new stack up locally on the dev machine
      (`docker compose -f compose.yaml up`) against the spare Arduino;
      confirm the container reads/writes the real device and the dashboard
      is reachable, mirroring slice-6's manual verification but from inside
      a container this time.
  - **Note:** if the old Python `docker-compose.yaml` `controller`
    service (privileged) is running, it will hold the serial device open
    and block this container exactly as it blocked the bare binary during
    slice-6 — stop it first for this verification (see design.md risk).

## 4. CI build job

- [ ] 4.1 Add a CI job that runs `docker buildx build --platform linux/arm64
      --features embed` against `fermenter/Dockerfile` on push/PR, failing
      the build if it doesn't succeed.
- [ ] 4.2 Confirm the job does not attempt to run the image or require any
      hardware, ARM64 runner, or attached Arduino (build-only, per
      `docs/rewrite-plan.md` §12.6 and design.md).
- [ ] 4.3 Confirm existing CI jobs (`cargo fmt --check`, `cargo clippy`,
      `cargo test`) are unaffected/still pass alongside the new job.

## 5. Pi deployment + verification (milestone 8)

- [ ] 5.1 Get the cross-compiled image onto the Raspberry Pi (transfer via
      `docker save`/`scp`/`docker load`, or push/pull through a registry —
      whichever is more convenient; see design.md open question).
- [ ] 5.2 Configure `.env` on the Pi (`SERIAL_PORT` etc. matching the Pi's
      actual device path — may differ from the dev machine's
      `/dev/ttyACM0`, see design.md risk).
- [ ] 5.3 Stop the old Python stack on the Pi if it's running (same
      port-contention consideration as the dev machine, see design.md).
- [ ] 5.4 `docker compose up` the new stack on the Pi; confirm the
      `fermenter` container reports healthy (`/healthz` via the Compose
      `healthcheck:`).
- [ ] 5.5 Confirm end-to-end on real hardware: readings from the Pi's
      attached (sensored) Arduino flow into the dashboard, target/brew
      forms round-trip through the real device, values are physically
      plausible (unlike slice-6's sensorless dev-machine verification).
- [ ] 5.6 Run `cargo test -- --ignored` on the Pi against its attached
      Arduino, confirming the slice-6 hardware tests also pass on the
      actual target hardware (per `docs/rewrite-plan.md` §12.7's milestone
      8 row: "run `cargo test -- --ignored` on Pi").

## 6. Spec & docs

- [ ] 6.1 `openspec validate slice-7-deployment` passes.
- [ ] 6.2 Archive the change once 1-5 are complete and verified, folding the
      new `deployment-packaging` capability into `openspec/specs/`.
- [ ] 6.3 Note in the repo (e.g. `docs/rewrite-plan.md` or a follow-up) that
      the Rust stack is now deployable, and that the old Python stack
      removal (cutover, `rewrite-plan.md` §15 Phase 3) is the next
      available step whenever desired — not performed as part of this
      slice.
