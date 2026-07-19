## 1. Build Script — Capture Git Tag at Compile Time

- [ ] 1.1 Modify `fermenter/build.rs` to run `git describe --tags --exact-match` when `CARGO_FEATURE_EMBED` is set, and emit `cargo:rustc-env=FERMENTER_VERSION=<tag>`
- [ ] 1.2 Verify `cargo build --features embed` sets the env var (check with `cargo build --features embed -v 2>&1 | grep FERMENTER_VERSION`)

## 2. App State & MiniJinja Globals

- [ ] 2.1 Add `version: String` field to `AppState` in `fermenter/src/web/mod.rs`
- [ ] 2.2 In `main.rs`, read `FERMENTER_VERSION` at startup (with `"dev"` fallback) and pass it when constructing `AppState`
- [ ] 2.3 In `build_environment()`, add the version as a MiniJinja global: `env.add_global("version", version)`

## 3. Templates & Styles

- [ ] 3.1 Update `fermenter/templates/base.html` — add `<p class="version">{{ version }}</p>` under `<h1>Fermenter</h1>` in `<header>`
- [ ] 3.2 Add `.version` style to `fermenter/static/styles.css`: `font-size: 0.85rem; color: var(--muted); margin-top: -0.5rem; margin-bottom: 1rem;`

## 4. Compose & Deploy Integration

- [ ] 4.1 Update `compose.yaml` fermenter service to pass `FERMENTER_VERSION=${FERMENTER_IMAGE_TAG}` (already set by deploy script)
- [ ] 4.2 Verify deploy script still works: `./scripts/build_and_ship_image.sh pi@<host>` and `FERMENTER_IMAGE_TAG=v2.0.0 docker compose up -d` shows `v2.0.0` on dashboard

## 5. Tests & Snapshots

- [ ] 5.1 Update `fermenter/src/web/mod.rs` test helpers (`test_state`, etc.) to include a version string
- [ ] 5.2 Run `INSTA_UPDATE=always cargo test` to update HTML snapshots for dashboard, status fragment, target form, brew form, chart fragment
- [ ] 5.3 Verify `cargo test` passes (all snapshot tests green)
- [ ] 5.4 Verify `cargo test --features embed` passes (release build path)

## 6. Dev Build Verification

- [ ] 6.1 Run `cargo run` locally — dashboard should show `dev` as version
- [ ] 6.2 Run `FERMENTER_VERSION=local cargo run` — dashboard should show `local`