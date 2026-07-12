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
fn main() {
    if std::env::var_os("CARGO_FEATURE_EMBED").is_some() {
        minijinja_embed::embed_templates!("templates");
    }
}
