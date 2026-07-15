## 1. Target form — add cancel button (RED)

- [x] 1.1 In `fermenter/templates/target_form.html`, add a cancel button (`<form method="get" action="/"><button type="submit">Cancel</button></form>`) after the Update form, matching the dashboard's `<form method="get"><button>` navigation pattern
- [x] 1.2 Run the existing target-form snapshot test (`cargo test target_form_with_current_target_snapshot`) — it should FAIL (RED) because the snapshot doesn't include the new cancel button

## 2. Target form — snapshots (GREEN)

- [x] 2.1 Regenerate the snapshot: `INSTA_UPDATE=always cargo test target_form_with_current_target_snapshot` — the diff shows the cancel link replaced by a `<form method="get"><button>` element after the Update form
- [x] 2.2 Regenerate the error snapshot: `INSTA_UPDATE=always cargo test target_form_with_error_snapshot`
- [x] 2.3 Re-run both target-form snapshot tests — GREEN (snapshots match)

## 3. Brew form — add cancel button (RED)

- [x] 3.1 In `fermenter/templates/brew_form.html`, add a cancel button (`<form method="get" action="/"><button type="submit">Cancel</button></form>`) after the Update form, mirroring the target-form change exactly
- [x] 3.2 Run the brew-form snapshot tests (`cargo test brew_form`) — they should FAIL (RED) because the snapshots don't include the new cancel button

## 4. Brew form — snapshots (GREEN)

- [x] 4.1 Regenerate both brew-form snapshots: `INSTA_UPDATE=always cargo test brew_form`
- [x] 4.2 Re-run both brew-form snapshot tests — GREEN (snapshots match)

## 5. Format-regression unit test

- [x] 5.1 Update the existing `form_templates_do_not_contain_back_link_or_confirmed_block` test in `fermenter/src/web/handlers.rs`: replace the `!html.0.contains("Back to dashboard")` assertion with `html.0.contains(`<button type=\"submit\">Cancel</button>`)` for both form templates, so the cancel button's presence is explicitly verified
- [x] 5.2 Run the new assertion — GREEN

## 6. HTTP-level verification (web-dashboard spec scenarios)

- [x] 6.1 Confirm GET `/target` returns an HTML form containing a cancel button (`<button type="submit">Cancel</button>`) in a `<form method="get" action="/">` — the existing `target_form_with_current_target_snapshot` snapshot already covers this; verify by inspection
- [x] 6.2 Confirm GET `/brew` returns the same — the existing `brew_form_with_current_brew_snapshot` snapshot already covers this; verify by inspection
- [x] 6.3 Confirm submitting a valid target or brew identifier via POST still works (the cancel button is a separate form with GET, not a submit) — existing POST tests already cover this; no regression expected

## 7. Lint, types, full suite

- [x] 7.1 Run `cargo fmt --check` from `fermenter/` — clean
- [x] 7.2 Run `cargo clippy --all-targets -- -D warnings` from `fermenter/` — clean
- [x] 7.3 Run `cargo test` from `fermenter/` — full suite GREEN (92 unit + 6 Redis integration; 3 hardware ignored)
