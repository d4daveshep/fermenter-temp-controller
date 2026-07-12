## 1. Template embedding (`embed` feature)

- [x] 1.1 Add `minijinja-embed` dependency to `fermenter/Cargo.toml`, gated
      behind a new `embed` feature (`[features] embed = ["dep:minijinja-embed"]`
      or equivalent).
      **Implemented:** `minijinja-embed = { version = "2.21.0", optional =
      true }` in `[dependencies]`, unconditional `minijinja-embed = "2.21.0"`
      in `[build-dependencies]` (so `build.rs` always compiles regardless of
      the feature), `embed = ["dep:minijinja-embed"]` in `[features]`.
- [x] 1.2 Add a `build.rs` (if required by `minijinja-embed`) that embeds the
      `templates/` directory at compile time.
      **Implemented:** `fermenter/build.rs` calls
      `minijinja_embed::embed_templates!("templates")` only when
      `CARGO_FEATURE_EMBED` is set (i.e. only when `--features embed` is
      passed); a no-op otherwise.
- [x] 1.3 Update `fermenter::web::build_environment()` to branch on
      `#[cfg(feature = "embed")]`: embedded loader when enabled, existing
      `path_loader("templates")` unchanged when disabled.
      **Implemented** in `src/web/mod.rs`: `#[cfg(feature = "embed")]` calls
      `minijinja_embed::load_templates!(env)`; `#[cfg(not(feature =
      "embed"))]` keeps the existing `path_loader("templates")` line
      unchanged.
- [x] 1.4 Add/extend a template smoke test that runs under `--features embed`
      (e.g. a second CI-style `cargo test --features embed`) confirming a
      known template renders identically with and without the feature.
      **Verified via the existing insta HTML-snapshot suite** rather than a
      new bespoke test: `cargo test --features embed` reuses the exact same
      recorded snapshots (`dashboard.html`, `partials/status.html`,
      `target_form.html`, `brew_form.html`) as the default build — all 87
      lib tests pass unchanged under both configurations, which is a
      stronger check than a single smoke test (byte-exact HTML equality per
      template, not just "renders without error").
- [x] 1.5 Confirm plain `cargo run`/`cargo test` (no `--features embed`)
      behavior is byte-for-byte unchanged from before this slice.
      **Verified:** default `cargo test` still shows 87 lib tests + 6 Redis
      integration tests passing, `cargo fmt --check` and
      `cargo clippy --all-targets -- -D warnings` both clean; also confirmed
      clean with `--features embed` added to clippy's invocation.

## 2. Dockerfile

- [x] 2.1 Write `fermenter/Dockerfile`, multi-stage:
  - [x] 2.1.1 Builder stage: `rust:1-bookworm`, cross-compile
        `cargo build --release --features embed` for `linux/arm64`.
  - [x] 2.1.2 Runtime stage: `debian:bookworm-slim`, copy only the compiled
        binary and `static/`.
      **Implemented** in `fermenter/Dockerfile` (relies on `buildx
      --platform linux/arm64` to select arm64 base images + QEMU emulation
      rather than an explicit cross target triple — see the file's header
      comment). Added `fermenter/.dockerignore` excluding `target/`,
      `tests/`, `.git`, `*.md` from the build context.
- [x] 2.2 Set up a local `docker buildx` builder with `linux/arm64` support
      (QEMU binfmt or equivalent) on the dev machine, documented in a
      comment or short README note for reproducibility.
      **Done:** `docker run --privileged --rm tonistiigi/binfmt --install
      arm64` registered the QEMU handler, then
      `docker buildx create --name fermenter-builder --driver
      docker-container --use` created a builder confirmed via `docker buildx
      ls` to support `linux/arm64` (alongside `linux/amd64`, `linux/386`).
- [x] 2.3 Build the image locally
      (`docker buildx build --platform linux/arm64 --features embed -t
      fermenter:arm64 fermenter/`) and confirm it succeeds.
      **Verified:** `docker buildx build --builder fermenter-builder
      --platform linux/arm64 -t fermenter:arm64 --load .` succeeded — full
      release build under QEMU emulation took ~12m15s (expected per
      design.md's documented emulation-overhead risk); `docker images`
      confirms `fermenter:arm64`, 104MB, and `docker inspect` confirms
      `Architecture: arm64`.
- [x] 2.4 Sanity-check the runtime image doesn't need a missing shared
      library (e.g. `docker run --rm --platform linux/arm64 fermenter:arm64
      --help` or equivalent under QEMU) — see design.md's open question on
      whether this becomes a permanent CI smoke test.
      **Verified:** `ldd /usr/local/bin/fermenter` inside the running arm64
      container resolves cleanly (only `libgcc_s`, `libm`, `libc`,
      `ld-linux-aarch64` — all present in `debian:bookworm-slim`, no
      `libudev`/`libssl` needed as design.md anticipated). Ran the binary
      itself under QEMU (no `--help` flag exists, so just started it): it
      logged startup, attempted Redis/serial connections (correctly failed
      since neither was available in the bare container), and exhibited the
      real capped-exponential backoff (500ms→...→30s) — a stronger check
      than a bare smoke test.

## 3. Compose orchestration

- [x] 3.1 Write `compose.yaml` (new stack, 2 containers):
  - [x] 3.1.1 `redis` service: `redis:8`, named volume, `--save 60 1`.
  - [x] 3.1.2 `fermenter` service: `build: ./fermenter`, `env_file: .env`,
        scoped `devices: ["/dev/ttyACM0:/dev/ttyACM0"]` (not `privileged`),
        `depends_on: [redis]`, `restart: on-failure`.
  - [x] 3.1.3 `healthcheck:` on the `fermenter` service calling `/healthz`.
      **Implemented** at `fermenter/compose.yaml` (self-contained alongside
      the Dockerfile, deliberately separate from the repo-root
      `docker-compose.yaml`/`.env` used by the old Python stack — no
      collision). Devices/ports/healthcheck use `${VAR:-default}`
      interpolation from `.env`. Healthcheck uses `curl` — discovered
      neither `wget` nor `curl` exist in `debian:bookworm-slim` by default,
      so added a small `apt-get install curl` layer to the Dockerfile's
      runtime stage (not in the original design sketch, but required for
      the healthcheck to function at all).
- [x] 3.2 Write/update a `.env.example` documenting the required variables
      (`SERIAL_PORT`, `SERIAL_BAUD`, `MOCK_SERIAL`, `REDIS_URL`,
      `TS_RETENTION_DAYS`, `DEFAULT_BREW_ID`, `HTTP_PORT`,
      `DEFAULT_TARGET_TEMP`, `RUST_LOG`) per `system-configuration`
      (already implemented — this just documents existing vars for the new
      deployment path).
      **Implemented** at `fermenter/.env.example`; also added `.env` to
      `fermenter/.gitignore` so a real `.env` copied from it is never
      committed.
- [x] 3.3 Bring the new stack up locally on the dev machine
      (`docker compose -f compose.yaml up`) against the spare Arduino;
      confirm the container reads/writes the real device and the dashboard
      is reachable, mirroring slice-6's manual verification but from inside
      a container this time.
  - **Note:** if the old Python `docker-compose.yaml` `controller`
    service (privileged) is running, it will hold the serial device open
    and block this container exactly as it blocked the bare binary during
    slice-6 — stop it first for this verification (see design.md risk).
      **Verified end-to-end:** `docker compose up -d` (native amd64 build,
      no emulation needed for local dev-machine testing — the arm64 image
      built separately via `buildx` in task 2.3 is the actual deployment
      artifact) brought up both containers; `fermenter` reported
      `(healthy)` in `docker compose ps` within the healthcheck's
      `start_period`. Logs confirmed the real, non-privileged, scoped
      device passthrough works: `serial port opened port=/dev/ttyACM0`,
      real readings ingested (`-127.0` fields, no sensors, as expected),
      and the startup target-reconcile wrote `<19.5>` to the real device.
      `GET /healthz` → `{"serial_connected":true}`; `GET /status` reflected
      the real reading; `GET /` (dashboard) → HTTP 200. Torn down cleanly
      with `docker compose down` afterward.

## 4. CI build job

- [x] 4.1 Add a CI job that runs `docker buildx build --platform linux/arm64
      --features embed` against `fermenter/Dockerfile` on push/PR, failing
      the build if it doesn't succeed.
      **Discovered no CI existed at all yet** (no `.github/workflows/` in
      the repo, and no prior slice created one, despite rewrite-plan.md
      §12.6 and this slice's own design.md describing fmt/clippy/test as
      "existing" jobs). Per your direction, created the full
      `.github/workflows/rust.yml` in one file: `fmt`, `clippy`, `test`
      (all newly added, matching §12.6's outline), plus the new
      `docker-build` job (`docker/setup-qemu-action` +
      `docker/setup-buildx-action` + `docker/build-push-action@v6`,
      `platforms: linux/arm64`, `push: false`).
- [x] 4.2 Confirm the job does not attempt to run the image or require any
      hardware, ARM64 runner, or attached Arduino (build-only, per
      `docs/rewrite-plan.md` §12.6 and design.md).
      **Confirmed:** `docker-build` job only builds (`push: false`, no
      `docker run` step); runs on a plain `ubuntu-latest` runner via QEMU
      emulation, no ARM64 runner or hardware needed.
- [x] 4.3 Confirm existing CI jobs (`cargo fmt --check`, `cargo clippy`,
      `cargo test`) are unaffected/still pass alongside the new job.
      **Verified all 4 jobs locally with `nektos/act`** (installed
      temporarily, removed after): `fmt` ✅, `clippy` ✅ (`-D warnings`
      clean), `test` ✅ (93 tests: 87 lib + 6 `testcontainers`-backed Redis
      integration tests passed, 3 hardware tests correctly `ignored`,
      confirming testcontainers works via the Docker socket on a
      GitHub-Actions-like runner), `docker-build` ✅ (arm64 cross-compile
      succeeded under `act`'s emulation, ~12m38s, matching the manual
      buildx timing from task 2.3).

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
