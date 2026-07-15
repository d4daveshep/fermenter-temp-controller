## 1. Dashboard — replace links with buttons (RED)

- [x] 1.1 In `fermenter/templates/dashboard.html`, replace the `<p><a href="/target">Set target temperature</a></p>` block with a `<form method="get" action="/target"><button type="submit">Set target temperature</button></form>`
- [x] 1.2 In the same file, replace the `<p><a href="/brew">Change brew identifier</a></p>` block with a `<form method="get" action="/brew"><button type="submit">Change brew identifier</button></form>`
- [x] 1.3 Run `cargo test dashboard_renders_snapshot` — FAILED (RED) because the snapshot contains the old `<a>` tags, not `<form><button>` elements

## 2. Dashboard buttons — snapshots (GREEN)

- [x] 2.1 Regenerate the dashboard snapshot: `INSTA_UPDATE=always cargo test dashboard_renders_snapshot` — the diff shows `<form method="get" action="..."><button type="submit">` replacing the old `<a>` links
- [x] 2.2 Run `cargo test dashboard_renders_snapshot` — GREEN (snapshot matches)
- [x] 2.3 Confirm the other snapshot tests (`cargo test target_form`) still pass unchanged at this point

## 3. Form templates — remove confirmation & back link (RED)

- [x] 3.1 In `fermenter/templates/target_form.html`, remove the `{% if confirmed %}<p class="confirmed">...</p>{% endif %}` block and the `<p><a href="/">Back to dashboard</a></p>` link
- [x] 3.2 In `fermenter/templates/brew_form.html`, remove the `{% if confirmed %}<p class="confirmed">...</p>{% endif %}` block and the `<p><a href="/">Back to dashboard</a></p>` link
- [x] 3.3 Run `cargo test -- target_form brew_form` — 4 snapshot tests FAILED (RED) because snapshots still contain the removed markup

## 4. Handlers — redirect on success (GREEN)

- [x] 4.1 Remove the `confirmed` field from `TargetFormContext` struct in `fermenter/src/web/handlers.rs`
- [x] 4.2 Remove the `confirmed` field from `BrewFormContext` struct in the same file
- [x] 4.3 Change `set_target` return type from `Result<Html<String>>` to `Result<Response>` (`axum::response::Response`); on validation success return `Redirect::to("/")`; on validation failure render and return the form HTML as before
- [x] 4.4 Change `set_brew` return type from `Result<Html<String>>` to `Result<Response>`; on validation success return `Redirect::to("/")`; on validation failure render and return the form HTML as before
- [x] 4.5 Remove all `confirmed: true/false` field assignments from both handler bodies and the `target_form`/`brew_form` GET handlers
- [x] 4.6 Add `use axum::response::{Redirect, Response, IntoResponse};` at top of handlers.rs (or adjust existing imports)
- [x] 4.7 Regenerate all affected snapshots: `INSTA_UPDATE=always cargo test`, review with `cargo insta review` and accept (`cargo insta accept`)
- [x] 4.8 Re-run `cargo test` — all tests GREEN

## 5. Format-regression unit test

- [x] 5.1 Add a unit test in `fermenter/src/web/handlers.rs` (or `web/mod.rs`) that builds an `AppState`, calls `set_target` with a valid POST, and asserts the response is an HTTP 303 redirect with `Location: /` (coverage: existing `valid_post_target_updates_state_and_redirects`)
- [x] 5.2 Add a unit test that calls `set_target` with an invalid POST and asserts the response is HTML (200) containing the error message — confirms invalid submissions still re-render the form (coverage: `non_numeric_post_target_is_rejected_and_state_unchanged`)
- [x] 5.3 Add a unit test that calls `set_brew` with a valid POST and asserts the response is an HTTP 303 redirect with `Location: /` (coverage: `valid_post_brew_updates_state_and_redirects`)
- [x] 5.4 Add a unit test that calls `set_brew` with an invalid POST and asserts the response is HTML (200) containing the error message (coverage: `empty_post_brew_is_rejected_and_state_unchanged`)
- [x] 5.5 Add a unit test that renders `dashboard.html` and asserts the response contains `<button` elements (not the old `<a href="/target">` or `<a href="/brew">`) — added `dashboard_uses_buttons_not_links_for_navigation` and `form_templates_do_not_contain_back_link_or_confirmed_block` in handlers.rs
- [x] 5.6 Run the new tests — all GREEN

## 6. HTTP-level verification (web-dashboard spec scenarios)

- [x] 6.1 Confirm the existing `tower::oneshot` test for `POST /target` still passes and now returns a 303 redirect (status code assertion), not HTML with confirmation text
- [x] 6.2 Confirm the existing `tower::oneshot` test for `POST /brew` still passes and now returns a 303 redirect, not HTML with confirmation text
- [x] 6.3 Confirm existing GET tests for `/target` and `/brew` still pass and no longer contain "Back to dashboard" in their response bodies — verified via snapshot tests which no longer show back-link or confirmed blocks
- [x] 6.4 Confirm the existing dashboard `GET /` test still passes and its response body contains `<button` elements for navigation

## 7. Lint, types, full suite

- [x] 7.1 Run `cargo fmt --check` from `fermenter/` — clean
- [x] 7.2 Run `cargo clippy --all-targets -- -D warnings` from `fermenter/` — clean
- [x] 7.3 Run `cargo test` from `fermenter/` — full suite GREEN (92 unit + insta snapshots + HTTP oneshot + 6 Redis integration; 3 hardware tests ignored)

## 8. OpenSpec validation

- [ ] 8.1 Run `openspec validate ui-links-to-buttons --strict` — passes (confirms the MODIFIED delta against `web-dashboard` is well-formed, requirement names match exactly, scenarios use `####` headers)
- [ ] 8.2 Run `openspec status --change "ui-links-to-buttons"` — `isComplete: true` (all `applyRequires` artifacts done)

## 9. Manual smoke test

- [x] 9.1 From `fermenter/`, run `MOCK_SERIAL=true cargo run` and open `http://localhost:8080/` in a browser — app starts, serves pages
- [x] 9.2 Confirm the dashboard shows `<button>` elements for "Set target temperature" and "Change brew identifier" — clicking each submits its `<form method="get">` and navigates to the respective form page — verified via curl: dashboard contains `<form method="get" action="/target"><button type="submit">`
- [x] 9.3 From the target form page, submit a valid temperature — confirm the browser redirects to the dashboard, showing the updated target value — verified: POST returns 303 with Location: /
- [x] 9.4 From the brew form page, submit a valid brew identifier — confirmed same 303 redirect pattern
- [x] 9.5 Submit an invalid value in each form — confirm the form re-renders with an error message and no redirect — verified: POST invalid returns 200
- [x] 9.6 Confirm neither form page shows a "Back to dashboard" link or confirmation message — verified: 0 occurrences of "Back to dashboard" in GET /target and GET /brew
