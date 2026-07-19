// Embeds `templates/` into the binary at compile time when the `embed`
// cargo feature is enabled (slice-7: deployment-packaging), so the release
// Docker image needs no mounted `templates/` volume. `minijinja-embed` is
// always a build-dependency (regardless of the feature) so this file always
// compiles; the actual embedding work — and the `cargo:rerun-if-changed`
// tracking it emits — only runs when the feature is on, via the
// `CARGO_FEATURE_EMBED` env var Cargo sets for enabled features.
//
// With the feature off (the default, e.g. plain `cargo run`/`cargo test`),
// this is a no-op: `web::build_environment()` falls back to the existing
// dev-mode `minijinja::path_loader("templates")`, unchanged from before this
// slice.
//
// When the `embed` feature is on (release build), also capture the Git tag
// via `git describe --tags --exact-match` and emit it as
// `cargo:rustc-env=FERMENTER_VERSION=<tag>` so the version is baked into the
// binary at compile time. This runs before template embedding so the version
// is available as a MiniJinja global.
fn main() {
    if std::env::var_os("CARGO_FEATURE_EMBED").is_some() {
        let version = std::env::var("FERMENTER_VERSION")
            .ok()
            .filter(|v| !v.is_empty() && v != "dev")
            .or_else(|| {
                std::process::Command::new("git")
                    .args(["describe", "--tags", "--exact-match"])
                    .current_dir(
                        std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string()),
                    )
                    .output()
                    .ok()
                    .filter(|o| o.status.success())
                    .and_then(|o| {
                        let v = String::from_utf8_lossy(&o.stdout).trim().to_string();
                        if v.is_empty() { None } else { Some(v) }
                    })
            });

        if let Some(version) = version {
            println!("cargo:rustc-env=FERMENTER_VERSION={version}");
        }

        minijinja_embed::embed_templates!("templates");
    }
}
