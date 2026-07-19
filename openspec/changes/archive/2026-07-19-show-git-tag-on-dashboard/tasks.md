## 1. Build Script — Capture Git Tag at Compile Time

- [x] 1.1 Modify `fermenter/build.rs` to run `git describe --tags --exact-match` when `CARGO_FEATURE_EMBED` is set, and emit `cargo:rustc-env=FERMENTER_VERSION=<tag>`
- [x] 1.2 Verify `cargo build --features embed` sets the env var (check with `cargo build --features embed -v 2>&1 | grep FERMENTER_VERSION`)

## 2. App State & MiniJinja Globals

- [x] 2.1 Add `version: String` field to `AppState` in `fermenter/src/web/mod.rs`
- [x] 2.2 In `main.rs`, read `FERMENTER_VERSION` at startup (with `"dev"` fallback) and pass it when constructing `AppState`
- [x] 2.3 In `build_environment()`, add the version as a MiniJinja global: `env.add_global("version", version)`

## 3. Templates & Styles

- [x] 3.1 Update `fermenter/templates/base.html` — add `<p class="version">{{ version }}</p>` under `<h1>Fermenter</h1>` in `<header>`
- [x] 3.2 Add `.version` style to `fermenter/static/styles.css`

## 4. Compose & Deploy Integration

- [x] 4.1 Update `compose.yaml` to pass `FERMENTER_VERSION` as a Docker build arg; update `Dockerfile` with `ARG`/`ENV`; update `build_and_ship_image.sh` with `--build-arg`
- [x] 4.2 Verify deploy: `./scripts/build_and_ship_image.sh pi@<host>` and `FERMENTER_IMAGE_TAG=v2.1.0 docker compose up -d` shows `v2.1.0` on dashboard (manual — verified on Pi) ✓

## 5. Tests & Snapshots

- [x] 5.1 Update `fermenter/src/web/mod.rs` test helpers (`test_state`, etc.) to include a version string
- [x] 5.2 Run `INSTA_UPDATE=always cargo test` to update HTML snapshots for dashboard, status fragment, target form, brew form, chart fragment
- [x] 5.3 Verify `cargo test` passes (all snapshot tests green)
- [x] 5.4 Verify `cargo build --features embed` passes (release build path)

## 6. Dev Build Verification

- [ ] 6.1 Run `cargo run` locally — dashboard should show `dev` as version (manual — requires browser)
- [ ] 6.2 Run `FERMENTER_VERSION=local cargo run` — dashboard should show `local` (manual — requires browser)