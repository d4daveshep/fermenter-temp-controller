## Why

The project uses annotated SemVer Git tags (e.g., `v2.0.0`) for releases, created on the dev machine and shipped to the Raspberry Pi via `scripts/build_and_ship_image.sh`. Currently there is no way to see which version is running on the dashboard — operators must check Docker tags on the Pi manually. Displaying the Git tag as a subtitle under "Fermenter" provides immediate version visibility on the web UI.

## What Changes

- **New capability**: `version-display` — injects the Git tag (from the annotated SemVer tag at build time) into the web dashboard as a subtitle under the "Fermenter" title.
- **Modified capability**: `web-dashboard` — the base template (`base.html`) receives a new `version` context variable and renders it as a subtitle; the dashboard handler passes it through.

The Git tag is captured at compile time via a build script (similar to how `minijinja-embed` embeds templates) and baked into the binary — no runtime Git dependency on the Pi. The tag is also surfaced via a `FERMENTER_VERSION` environment variable in `compose.yaml` so local/dev builds without a tag can still display a version.

## Capabilities

### New Capabilities
- `version-display`: Provides the Git tag (or dev fallback) to the web layer for display on the dashboard.

### Modified Capabilities
- `web-dashboard`: Extends the base template context to include a `version` field; renders it as a subtitle under the "Fermenter" header.

## Impact

**Code changes:**
- `fermenter/build.rs` — capture `git describe --tags --exact-match` at build time (when `embed` feature is on), emit as `cargo:rustc-env=FERMENTER_VERSION=<tag>`.
- `fermenter/src/web/mod.rs` — read `FERMENTER_VERSION` env var (with dev fallback) and add to `AppState`/`build_environment` context.
- `fermenter/templates/base.html` — render `version` as a subtitle under `<h1>Fermenter</h1>`.
- `fermenter/src/web/handlers.rs` — pass `version` through `status_context` so `/status` fragment also shows it.
- `compose.yaml` — set `FERMENTER_IMAGE_TAG` as fallback env var for local dev builds.

**No changes to:**
- Arduino firmware / serial protocol
- Redis schema / store layer
- Temperature control / brew session logic
- Deployment script (`scripts/build_and_ship_image.sh`) — the tag is already available there via `IMAGE_TAG`

**Dependencies:** No new crates; uses existing `std::env` and build script capabilities.