## 1. Target form — add cancel button (RED)

- [ ] 1.1 In `fermenter/templates/target_form.html`, add a cancel link (`<a href="/">Cancel</a>`) styled as a button, placed before the `<button type="submit">Update</button>` within the form
- [ ] 1.2 Run the existing target-form snapshot test (`cargo test target_form_with_current_target_snapshot`) — it should FAIL (RED) because the snapshot doesn't include the new cancel link

## 2. Target form — snapshots (GREEN)

- [ ] 2.1 Regenerate the snapshot: `INSTA_UPDATE=always cargo test target_form_with_current_target_snapshot` — the diff shows only the added cancel link before the Update button
- [ ] 2.2 Regenerate the error snapshot: `INSTA_UPDATE=always cargo test target_form_with_error_snapshot`
- [ ] 2.3 Re-run both target-form snapshot tests — GREEN (snapshots match)

## 3. Brew form — add cancel button (RED)

- [ ] 3.1 In `fermenter/templates/brew_form.html`, add a cancel link (`<a href="/">Cancel</a>`) styled as a button, placed before the `<button type="submit">Update</button>` within the form, mirroring the target-form change exactly
- [ ] 3.2 Run the brew-form snapshot tests (`cargo test brew_form`) — they should FAIL (RED) because the snapshots don't include the new cancel link

## 4. Brew form — snapshots (GREEN)

- [ ] 4.1 Regenerate both brew-form snapshots: `INSTA_UPDATE=always cargo test brew_form`
- [ ] 4.2 Re-run both brew-form snapshot tests — GREEN (snapshots match)

## 5. Format-regression unit test

- [ ] 5.1 Update the existing `form_templates_do_not_contain_back_link_or_confirmed_block` test in `fermenter/src/web/handlers.rs`: replace the `!html.0.contains("Back to dashboard")` assertion with `html.0.contains(`<a href="/">Cancel</a>`)` for both form templates, so the cancel button's presence and target are explicitly verified
- [ ] 5.2 Run the new assertion — GREEN

## 6. HTTP-level verification (web-dashboard spec scenarios)

- [ ] 6.1 Confirm GET `/target` returns an HTML form containing a cancel link pointing to `/` — the existing `target_form_with_current_target_snapshot` snapshot already covers this; verify by inspection
- [ ] 6.2 Confirm GET `/brew` returns an HTML form containing a cancel link pointing to `/` — the existing `brew_form_with_current_brew_snapshot` snapshot already covers this; verify by inspection
- [ ] 6.3 Confirm submitting a valid target or brew identifier via POST still works (the cancel link is passive navigation, no form submit) — existing POST tests already cover this; no regression expected

## 7. Lint, types, full suite

- [ ] 7.1 Run `cargo fmt --check` from `fermenter/` — clean
- [ ] 7.2 Run `cargo clippy --all-targets -- -D warnings` from `fermenter/` — clean
- [ ] 7.3 Run `cargo test` from `fermenter/` — full suite GREEN

## 8. Manual smoke test

- [ ] 8.1 From `fermenter/`, run `MOCK_SERIAL=true cargo run` and open `http://localhost:8080/target` in a browser — confirm a Cancel button/link is visible before the Update button, and clicking it navigates to the dashboard
- [ ] 8.2 Repeat for `http://localhost:8080/brew` — confirm Cancel navigates to the dashboard without submitting the form
