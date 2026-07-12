## Why

Slices 1-6 built and proved the entire controller — ingest, persistence,
dashboard, target control, brew sessions, and now real hardware — but it only
runs via `cargo run` on a dev machine with templates loaded from disk
(`minijinja::path_loader("templates")`) and no container image. Milestones
7+8 of `docs/rewrite-plan.md` call for packaging the binary into a
self-contained, cross-compiled ARM64 Docker image and deploying it to the
Raspberry Pi via Compose with proper device passthrough — the step that
actually makes this rewrite deployable in place of the current Python
system. This is the last of the 7 planned vertical slices
(`docs/openspec-rewrite-management.md` §3).

## What Changes

- Add a cargo `embed` feature: when enabled, MiniJinja templates are baked
  into the binary at compile time via `minijinja-embed` instead of loaded
  from a `templates/` directory at runtime, so the release image is fully
  self-contained (no mounted template volume). The existing dev-mode
  `path_loader` behavior (slice-3) is unchanged when the feature is off.
- Add a multi-stage `Dockerfile` for the Rust app: a builder stage
  cross-compiling `--release --features embed` for `linux/arm64` (via
  `docker buildx`/`cross`), and a slim runtime stage (`debian:bookworm-slim`)
  copying just the compiled binary and `static/` (templates are embedded,
  not copied).
- Add `compose.yaml` for the new 2-container stack (`fermenter` + `redis:8`,
  Time Series built in) — replacing the old 3-container Python/InfluxDB
  setup (`docker-compose.yaml`, `Dockerfile.controller`,
  `Dockerfile.web_apis`) for this stack, though the old files are left in
  place until the eventual cutover (`rewrite-plan.md` §15, a separate,
  later step — not part of this slice). Device passthrough is scoped to the
  specific serial device (`devices: ["/dev/ttyACM0:/dev/ttyACM0"]`) rather
  than the old system's `privileged: true` / `network_mode: host`.
- Wire the existing `/healthz` endpoint (slice-3, currently
  `serial_connected` only — `redis_connected` was explicitly deferred and
  stays deferred here too; see design.md) into a Compose `healthcheck:`
  stanza.
- Add a CI build job asserting the cross-compiled release image builds
  (`docker buildx build --platform linux/arm64 --features embed`), per
  `docs/rewrite-plan.md` §12.6. No hardware involved — this never runs the
  image, only builds it.
- Deploy the built image to the Raspberry Pi and verify end-to-end:
  readings flowing, dashboard updating, `cargo test -- --ignored` passing
  against the Pi's attached Arduino (milestone 8).
- No Arduino firmware changes. No changes to any other capability's runtime
  behavior — this slice is packaging and deployment only.

## Capabilities

### New Capabilities

- `deployment-packaging`: the Docker image build (multi-stage, cross-compiled
  ARM64, embedded templates), Compose orchestration (device passthrough,
  healthcheck), and the CI build-verification job. First `ADDED` requirements
  for this capability, matching the slice-map matrix in
  `docs/openspec-rewrite-management.md` §7 (`deployment-packaging` is
  untouched — `·` — through slices 1-6, `A` only at slice-7).

### Modified Capabilities

(none — per the same slice-map matrix, `operational-health` is explicitly
`·` (untouched) at slice-7. `/healthz` gaining a `redis_connected` field
remains an open, unscheduled item first flagged in slice-3's design doc, not
picked up here; this slice only adds a Compose `healthcheck:` that calls the
endpoint as it exists today.)

## Impact

- **Code:** `fermenter/Cargo.toml` gains an `embed` feature + `minijinja-embed`
  dependency (build-time template embedding, gated behind the feature so dev
  builds are unaffected); `fermenter/src/web/mod.rs`'s
  `build_environment()` branches on the feature; new `fermenter/Dockerfile`;
  new `fermenter/compose.yaml` (or repo-root `compose.yaml` — see
  design.md).
- **CI:** new build-only job cross-compiling for `linux/arm64`; no existing
  jobs change.
- **Hardware:** requires the Raspberry Pi (milestone 8 verification) with
  the production Arduino attached — out of scope for slices 1-6, which only
  needed a dev-machine-attached Arduino (slice-6).
- **Deployment:** introduces the new 2-container stack alongside (not
  replacing) the existing `docker-compose.yaml` Python stack; cutover
  (deleting the Python stack) is an explicit, separate, later step per
  `rewrite-plan.md` §15 Phase 3 — not part of this slice.
</content>
