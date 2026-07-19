## Context

The project uses annotated SemVer Git tags (`v2.0.0`, `v2.1.0`, etc.) for releases. The deploy script (`scripts/build_and_ship_image.sh`) extracts the tag at build time via `git describe --exact-match --tags` and uses it as the Docker image tag. Currently, the running container has no way to report its version to the web dashboard — operators must check `docker images` or `docker compose ps` on the Pi.

The dashboard is rendered via MiniJinja templates (`fermenter/templates/base.html` + `dashboard.html`). The web layer is built in `fermenter/src/web/` with shared `AppState` and per-request context builders. Templates are loaded from disk in dev mode (`cargo run`) and embedded in the binary at compile time when the `embed` feature is enabled (release Docker build via `build.rs` + `minijinja-embed`).

## Goals / Non-Goals

**Goals:**
- Capture the Git tag at compile time (release) or from env var (dev) and make it available to templates.
- Display the version as a subtitle under "Fermenter" in the header on all pages (`base.html`).
- Ensure the version appears in the HTMX-polled `/status` fragment so it stays current (though it won't change at runtime).
- No runtime Git dependency on the Pi; version is baked into the binary.

**Non-Goals:**
- Version API endpoint (not needed — version is static UI chrome).
- Displaying commit SHA, build date, or Rust version (out of scope).
- Changing the Arduino firmware or serial protocol.
- Adding a version API endpoint (not needed — version is static UI chrome).

## Decisions

### 1. Version Injection: Docker Build Arg + Build Script Fallback

**Decision:** The primary mechanism is a Docker build arg (`--build-arg FERMENTER_VERSION=<tag>`), passed by `compose.yaml` and `scripts/build_and_ship_image.sh`. Inside the Dockerfile it becomes an `ENV` visible to `build.rs`, which emits `cargo:rustc-env=FERMENTER_VERSION=<tag>` to bake the value into the binary at compile time. `build.rs` also falls back to `git describe --tags --exact-match` for native `cargo build --features embed` outside Docker. For dev builds (no `embed`), the binary reads `FERMENTER_VERSION` from the runtime environment, defaulting to `dev`.

**Rationale:**
- The Docker build context (`fermenter/`) does not contain `.git/`, so `git describe` inside `build.rs` cannot discover the tag during `docker build`. A build arg is the idiomatic Docker way to inject host-side metadata.
- `cargo:rustc-env` still makes the value available at compile time as `option_env!("FERMENTER_VERSION")` — zero runtime cost.
- Dev fallback via `std::env::var` keeps local `cargo run` working without Git tags.
- The deploy script already computes the tag and passes it as `--build-arg`.
- `compose.yaml` passes `FERMENTER_VERSION=dev` as the default build arg (no tag for local builds), matching the existing `dev` fallback.

**Alternatives considered:**
- **Runtime `git describe` in container:** Rejected — no Git in the minimal runtime image (debian:bookworm-slim), adds attack surface.
- **`vergen` crate:** Rejected — adds a build dependency for a one-liner; `build.rs` is already present and simple.
- **`build.rs` `git describe` only (no build arg):** Rejected — Docker build context lacks `.git/`, so this silently fails in all container builds.

### 2. Version Propagation: `AppState` → Template Context

**Decision:** Read `FERMENTER_VERSION` once at startup in `main.rs`, store in `AppState`, and add it as a global to the MiniJinja environment in `build_environment()` so `{{ version }}` is available in *all* templates automatically.

**Rationale:**
- `AppState` is already the central shared state for the web layer; adding a `version: String` field is idiomatic.
- MiniJinja globals are the cleanest way to inject a value into every template without modifying every context struct (`StatusContext`, `ChartContext`, `TargetFormContext`, `BrewFormContext`, `HealthResponse`).
- `build_environment()` is called once at startup; we can set the global there.

**Alternatives considered:**
- **MiniJinja globals only (no `AppState`):** Even simpler — set `env.add_global("version", version)` directly in `build_environment()`. But `main.rs` needs the version for logging/startup message anyway, so storing in `AppState` is useful.
- **Per-handler context injection:** More explicit but repetitive; every handler would need to add `version` to its context struct.

### 3. Template Rendering: Subtitle in `base.html`

**Decision:** In `templates/base.html`, render `<p class="version">{{ version }}</p>` under `<h1>Fermenter</h1>` inside `<header>`. Style with `.version { font-size: 0.85rem; color: var(--muted); margin-top: -0.5rem; margin-bottom: 1rem; }` in `static/styles.css`.

**Rationale:**
- `base.html` is the parent of all pages (dashboard, target form, brew form) — version appears everywhere automatically.
- Minimal HTML change; no HTMX fragment coordination needed.
- CSS variable `--muted` already exists in `styles.css` (used for secondary text).

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| `git describe` fails in Docker build (no `.git/` in context) | Build arg is primary; `git describe` is only a fallback for native builds outside Docker. |
| Version stale if container restarted without rebuild | Version is baked at compile time — always matches the binary. Correct behavior. |
| Dev build shows `dev` instead of meaningful version | Acceptable — local dev doesn't need precise version. Can set `FERMENTER_VERSION=local` in `.env` if desired. |
| `minijinja-embed` captures templates *before* build script runs | `build.rs` runs before codegen; `minijinja_embed::embed_templates!` runs at codegen time. Env var is set before template compilation. Order is correct. |

## Migration Plan

1. Implement `build.rs` version capture (feature-gated on `embed`).
2. Add `version` global to MiniJinja env in `build_environment()`.
3. Update `base.html` to render `{{ version }}` subtitle.
4. Add `.version` style to `static/styles.css`.
5. Update `compose.yaml` to pass `FERMENTER_IMAGE_TAG` as `FERMENTER_VERSION` (already set by deploy script).
6. Verify: `cargo run` shows `dev`; `docker buildx build --features embed -t fermenter:test . && docker run --rm fermenter:test` shows the tag.

## Open Questions

- Should the version also appear in `/healthz` JSON response? (Proposal: no — health check is for orchestration, not humans. Keep it out of band.)
- Should we also expose a `BUILD_DATE` or `RUST_VERSION`? (Proposal: out of scope; can be added later if needed.)