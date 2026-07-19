# version-display

## Purpose

Provides the running application's version identifier (derived from the annotated SemVer Git tag at release build time, or a fallback in development) to the web layer for display on the dashboard.

## Requirements

### Requirement: Expose version to templates

The system SHALL make a `version` string available to all MiniJinja templates as a global variable, so any template can render `{{ version }}` without per-handler context changes.

#### Scenario: Release build shows Git tag

- **WHEN** the binary is built with the `embed` cargo feature (release Docker image) and the current Git commit has an annotated SemVer tag (e.g., `v2.0.0`)
- **THEN** the `version` global in templates equals that tag

#### Scenario: Dev build shows fallback

- **WHEN** the binary is built without the `embed` feature (plain `cargo run` / `cargo test`)
- **THEN** the `version` global in templates equals the value of the `FERMENTER_VERSION` environment variable, or `dev` if unset

#### Scenario: Version is static for the process lifetime

- **WHEN** the application runs
- **THEN** the `version` value never changes during the process lifetime (it is captured at compile time or process start)

### Requirement: Version format

The version string SHALL be a plain SemVer tag (e.g., `v2.0.0`, `v2.1.0-rc.1`) or the fallback `dev`. No commit SHA, build date, or other metadata is included.