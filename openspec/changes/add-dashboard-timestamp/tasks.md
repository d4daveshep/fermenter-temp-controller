## 1. Dependency

- [ ] 1.1 Add `chrono = { version = "0.4", features = ["clock"] }` to
      `fermenter/Cargo.toml` `[dependencies]` (disable default features to
      avoid pulling in serde/old-time shim; keep only `clock`)
- [ ] 1.2 Run `cargo build` to confirm `chrono` resolves and compiles on the
      host, then `cargo build --target aarch64-unknown-linux-gnu` (or the
      cross job) to confirm `linux/arm64` still builds with `chrono::Local`
      reading `/etc/localtime` natively (no `iana-time-zone` crate on glibc)

## 2. Template — remove dormant block, add server-time line (RED)

- [ ] 2.1 In `fermenter/templates/partials/status.html`, delete the
      `{% if updated_at %}<p class="updated-at">Last updated: {{ updated_at }}</p>{% endif %}` block
- [ ] 2.2 In the same file, add
      `<p class="server-time">Server time: {{ server_time }}</p>` *before*
      the `{% if reading %}` block, so it renders both with and without a
      reading (design.md decision 3)
- [ ] 2.3 Run the existing `insta` snapshot tests (`cargo test
      status_fragment`) — they should FAIL (RED) because `StatusContext` has
      no `server_time` field yet, so MiniJinja renders an empty string and
      the snapshot diff shows the new/removed lines

## 3. Handler — extend StatusContext (GREEN)

- [ ] 3.1 In `fermenter/src/web/handlers.rs`, add a `server_time: String`
      field to `StatusContext` (design.md decision 2)
- [ ] 3.2 In `status_context()`, populate it via
      `chrono::Local::now().format("%d-%b-%Y %H:%M:%S").to_string()`
- [ ] 3.3 Add `use chrono::Local;` (or `chrono::Local`) import at the top of
      `handlers.rs`
- [ ] 3.4 Update the two snapshot test constructors
      (`status_fragment_with_reading_snapshot`,
      `status_fragment_without_reading_snapshot`) to pass a fixed
      `server_time: "14-Jul-2026 14:30:45".to_string()` so snapshots are
      deterministic (design.md decision 5)
- [ ] 3.5 Regenerate snapshots: `INSTA_UPDATE=always cargo test
      status_fragment`, then review with `cargo insta review` and accept
      (`cargo insta accept`) — the diff should show only the added
      `Server time:` line and the removed `Last updated:` block
- [ ] 3.6 Re-run `cargo test status_fragment` — GREEN (snapshots match)

## 4. Format-regression unit test

- [ ] 4.1 Add a unit test in `fermenter/src/web/handlers.rs` (or
      `web/mod.rs`) that calls `status_context()` against a stub `AppState`
      and asserts the `server_time` field matches
      `^\d{2}-[A-Z][a-z]{2}-\d{4} \d{2}:\d{2}:\d{2}$` via `regex` *or* a
      hand-rolled parser — prefer hand-rolled to avoid adding a `regex` dep;
      check the format is exactly `DD-MMM-YYYY HH:MM:SS` (e.g. split on
      `' '`, then on `-`, assert 2-digit day, 3-letter month title-case,
      4-digit year, and `HH:MM:SS` with 2-digit each)
- [ ] 4.2 Run the new test — GREEN

## 5. HTTP-level verification (web-dashboard spec scenarios)

- [ ] 5.1 Confirm the existing `tower::oneshot` test for `GET /status` with a
      seeded reading still passes and the response body now contains
      `Server time:` (assert the substring is present)
- [ ] 5.2 Confirm the existing `tower::oneshot` test for `GET /status` with
      no reading still passes and the response body contains `Server time:`
      (covers spec scenario "Fragment shows server time even with no
      reading")
- [ ] 5.3 Confirm the existing `tower::oneshot` test for `GET /` still
      passes and the response body contains `Server time:` (covers spec
      scenario "Dashboard displays the server's local time")

## 6. Lint, types, full suite

- [ ] 6.1 Run `cargo fmt --check` from `fermenter/` — clean
- [ ] 6.2 Run `cargo clippy --all-targets -- -D warnings` from `fermenter/`
      — clean (watch for `chrono`-related lints, e.g. `unwrap_used` if any)
- [ ] 6.3 Run `cargo test` from `fermenter/` — full suite GREEN (unit +
      insta snapshots + HTTP oneshot; ignore `#[ignore]`'d serial hardware
      tests unless running on the Pi with an Arduino attached)

## 7. OpenSpec validation

- [ ] 7.1 Run `openspec validate add-dashboard-timestamp --strict` — passes
      (confirms the MODIFIED delta against `web-dashboard` is well-formed,
      requirement names match exactly, scenarios use `####` headers)
- [ ] 7.2 Run `openspec status --change "add-dashboard-timestamp"` —
      `isComplete: true` (all `applyRequires` artifacts done)

## 8. Manual smoke test

- [ ] 8.1 From `fermenter/`, run `MOCK_SERIAL=true cargo run` and open
      `http://localhost:8080/` in a browser
- [ ] 8.2 Confirm the `Server time:` line is visible, formatted
      `DD-MMM-YYYY HH:MM:SS`, and that it advances roughly every 5s as the
      HTMX `/status` poll fires — without a manual page refresh
- [ ] 8.3 Confirm the `Last updated:` line is gone
- [ ] 8.4 Stop the mock feed early (or let it exhaust) and confirm the
      `Server time:` line keeps ticking even after readings stop arriving
      (the liveness signal works independent of new data)
