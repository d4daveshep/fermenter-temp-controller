# deployment-packaging

## Purpose

Packaging and orchestration of the Rust application as a self-contained,
cross-compiled Docker image, deployed via Compose alongside its Redis Time
Series dependency with scoped device passthrough, health checking, and CI
build verification — the deployment mechanism that replaces the old
`cargo run`-on-dev-machine workflow.

## Requirements

### Requirement: Self-contained cross-compiled release image

The system SHALL build as a multi-stage Docker image, cross-compiled for
`linux/arm64`, whose runtime stage contains a single self-contained binary
with no dependency on a mounted template directory.

#### Scenario: Templates are embedded at compile time

- **WHEN** the release binary is built with the `embed` cargo feature
- **THEN** MiniJinja templates are baked into the binary at compile time and
  no `templates/` directory needs to exist at runtime

#### Scenario: Runtime image contains only the binary and static assets

- **WHEN** the runtime stage of the Docker image is built
- **THEN** it contains only the compiled binary and the `static/` directory,
  copied from the builder stage, with no build toolchain or source code

#### Scenario: Dev builds are unaffected

- **WHEN** the binary is built without the `embed` feature (e.g. local
  `cargo run`/`cargo test`)
- **THEN** templates continue to load from the `templates/` directory at
  runtime, exactly as before this capability existed

### Requirement: Compose orchestration with scoped device passthrough

The system SHALL be orchestrated via Docker Compose alongside its Redis
Time Series dependency, granting the application container access to the
serial device by passing through only that specific device rather than
running the container privileged.

#### Scenario: Serial device is passed through, not privileged

- **WHEN** the Compose stack starts the application container
- **THEN** the container is granted access to the configured serial device
  path via a scoped `devices:` mapping, and the container does not run in
  privileged mode

#### Scenario: Configuration is supplied via env file

- **WHEN** the Compose stack starts the application container
- **THEN** its configuration (serial port, Redis URL, ports, etc.) is
  supplied via an env file rather than being baked into the image

#### Scenario: Application depends on Redis being orchestrated together

- **WHEN** the Compose stack is started
- **THEN** the Redis Time Series service starts as part of the same stack
  and the application container is configured to depend on it

### Requirement: Container health check

The system SHALL configure a Compose health check for the application
container using its existing liveness endpoint, so container orchestration
can detect an unhealthy application instance.

#### Scenario: Health check reflects application liveness

- **WHEN** the application container is running
- **THEN** Compose's health check calls the application's health check
  endpoint and reflects its reported status

### Requirement: CI build verification

The system SHALL verify in CI that the cross-compiled release image builds
successfully, without requiring ARM64 hardware or a physical Arduino to be
present in the CI environment.

#### Scenario: CI builds the cross-compiled image

- **WHEN** CI runs on a push or pull request
- **THEN** it cross-compiles the release Docker image for `linux/arm64` with
  the `embed` feature enabled and fails the build if the image fails to
  build

#### Scenario: CI does not require hardware

- **WHEN** the CI build verification runs
- **THEN** it only builds the image and does not run it against real
  hardware, a Raspberry Pi, or a physical Arduino
