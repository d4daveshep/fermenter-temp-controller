## 1. Dashboard — replace links with buttons (RED)

- [ ] 1.1 In `fermenter/templates/dashboard.html`, replace the `<p><a href="/target">Set target temperature</a></p>` block with a `<form method="get" action="/target"><button type="submit">Set target temperature</button></form>`
- [ ] 1.2 In the same file, replace the `<p><a href="/brew">Change brew identifier</a></p>` block with a `<form method="get" action="/brew"><button type="submit">Change brew identifier</button></form>`
- [ ] 1.3 Run the existing `insta` snapshot test (`cargo test dashboard_renders`) — it should FAIL (RED) because the snapshot contains the old `<a>` tags, not `<button>` elements

## 2. Dashboard buttons — snapshots (GREEN)

- [ ] 2.1 Regenerate the dashboard snapshot: `INSTA_UPDATE=always cargo test dashboard_renders` — the diff shows `<form method="get" action="..."><button type="submit">` replacing the old `<a>` links
- [ ] 2.2 Run `cargo test dashboard_renders` — GREEN (snapshot matches)
- [ ] 2.3 Confirm the other snapshot tests (`cargo test target_form`) still pass unchanged at this point

## 3. Form templates — remove confirmation & back link (RED)

- [ ] 3.1 In `fermenter/templates/target_form.html`, remove the `{% if confirmed %}<p class="confirmed">...</p>{% endif %}` block and the `<p><a href="/">Back to dashboard</a></p>` link
- [ ] 3.2 In `fermenter/templates/brew_form.html`, remove the `{% if confirmed %}<p class="confirmed">...</p>{% endif %}` block and the `<p><a href="/">Back to dashboard</a></p>` link
- [ ] 3.3 Run `cargo test target_form` — should FAIL (RED) because snapshots still contain the removed markup; the handler tests that built context with `confirmed: true` will also need updating

## 4. Handlers — redirect on success (GREEN)

- [ ] 4.1 Remove the `confirmed` field from `TargetFormContext` struct in `fermenter/src/web/handlers.rs`
- [ ] 4.2 Remove the `confirmed` field from `BrewFormContext` struct in the same file
- [ ] 4.3 Change `set_target` return type from `Result<Html<String>>` to `Result<Response>` (`axum::response::Response`); on validation success return `Redirect::to("/")`; on validation failure render and return the form HTML as before
- [ ] 4.4 Change `set_brew` return type from `Result<Html<String>>` to `Result<Response>`; on validation success return `Redirect::to("/")`; on validation failure render and return the form HTML as before
- [ ] 4.5 Remove all `confirmed: true/false` field assignments from both handler bodies and the `target_form`/`brew_form` GET handlers
- [ ] 4.6 Add `use axum::response::{Redirect, Response, IntoResponse};` at top of handlers.rs (or adjust existing imports)
- [ ] 4.7 Regenerate all affected snapshots: `INSTA_UPDATE=always cargo test`, review with `cargo insta review` and accept (`cargo insta accept`)
- [ ] 4.8 Re-run `cargo test` — all tests GREEN

## 5. Format-regression unit test

- [ ] 5.1 Add a unit test in `fermenter/src/web/handlers.rs` (or `web/mod.rs`) that builds an `AppState`, calls `set_target` with a valid POST, and asserts the response is an HTTP 303 redirect with `Location: /`
- [ ] 5.2 Add a unit test that calls `set_target` with an invalid POST and asserts the response is HTML (200) containing the error message — confirms invalid submissions still re-render the form
- [ ] 5.3 Add a unit test that calls `set_brew` with a valid POST and asserts the response is an HTTP 303 redirect with `Location: /`
- [ ] 5.4 Add a unit test that calls `set_brew` with an invalid POST and asserts the response is HTML (200) containing the error message
- [ ] 5.5 Add a unit test that renders `dashboard.html` and asserts the response contains `<button` elements (not the old `<a href="/target">` or `<a href="/brew">`)
- [ ] 5.6 Run the new tests — all GREEN

## 6. HTTP-level verification (web-dashboard spec scenarios)

- [ ] 6.1 Confirm the existing `tower::oneshot` test for `POST /target` still passes and now returns a 303 redirect (status code assertion), not HTML with confirmation text
- [ ] 6.2 Confirm the existing `tower::oneshot` test for `POST /brew` still passes and now returns a 303 redirect, not HTML with confirmation text
- [ ] 6.3 Confirm existing GET tests for `/target` and `/brew` still pass and no longer contain "Back to dashboard" in their response bodies
- [ ] 6.4 Confirm the existing dashboard `GET /` test still passes and its response body contains `<button` elements for navigation

## 7. Lint, types, full suite

- [ ] 7.1 Run `cargo fmt --check` from `fermenter/` — clean
- [ ] 7.2 Run `cargo clippy --all-targets -- -D warnings` from `fermenter/` — clean
- [ ] 7.3 Run `cargo test` from `fermenter/` — full suite GREEN (unit + insta snapshots + HTTP oneshot; ignore `#[ignore]`'d serial hardware tests unless running on the Pi with an Arduino attached)

## 8. OpenSpec validation

- [ ] 8.1 Run `openspec validate ui-links-to-buttons --strict` — passes (confirms the MODIFIED delta against `web-dashboard` is well-formed, requirement names match exactly, scenarios use `####` headers)
- [ ] 8.2 Run `openspec status --change "ui-links-to-buttons"` — `isComplete: true` (all `applyRequires` artifacts done)

## 9. Manual smoke test

- [ ] 9.1 From `fermenter/`, run `MOCK_SERIAL=true cargo run` and open `http://localhost:8080/` in a browser
- [ ] 9.2 Confirm the dashboard shows `<button>` elements for "Set target temperature" and "Change brew identifier" — clicking each submits its `<form method="get">` and navigates to the respective form page
- [ ] 9.3 From the target form page, submit a valid temperature — confirm the browser redirects to the dashboard, showing the updated target value
- [ ] 9.4 From the brew form page, submit a valid brew identifier — confirm the browser redirects to the dashboard, showing the updated brew ID
- [ ] 9.5 Submit an invalid value in each form — confirm the form re-renders with an error message and no redirect
- [ ] 9.6 Confirm neither form page shows a "Back to dashboard" link or confirmation message
