## 1. Template change (RED)

- [ ] 1.1 Add `<dt>Reason</dt><dd>{{ reading.reason_code }}</dd>` to
      `fermenter/templates/partials/status.html`, immediately after the
      `Action` row
- [ ] 1.2 Run the existing `insta` snapshot test (`cargo test
      status_fragment_with_reading_snapshot`) — it should FAIL (RED) because
      the snapshot doesn't include the new `Reason` row yet

## 2. Snapshot update (GREEN)

- [ ] 2.1 Regenerate the snapshot:
      `INSTA_UPDATE=always cargo test status_fragment_with_reading_snapshot`,
      then review with `cargo insta review` and accept (`cargo insta accept`)
      — the diff should show only the added `Reason: below-target` line
- [ ] 2.2 Re-run `cargo test status_fragment` — GREEN (snapshots match)
- [ ] 2.3 Confirm `status_fragment_without_reading_snapshot` still passes
      unchanged (no reading means no reason code display)

## 3. Format-regression unit test

- [ ] 3.1 Add a unit test in `fermenter/src/web/handlers.rs` that renders
      `partials/status.html` with `sample_reading()` (which already sets
      `reason_code: "below-target"`) and asserts the rendered HTML contains
      the string `"below-target"` and the label `"Reason"`
- [ ] 3.2 Run the new test — GREEN

## 4. HTTP-level verification (web-dashboard spec scenarios)

- [ ] 4.1 In the existing `status_returns_ok_with_latest_reading` test
      (`fermenter/src/web/mod.rs`), add an assertion that the response body
      contains `"below-target"` (the reason code from `sample_reading()`)
- [ ] 4.2 In the existing `dashboard_returns_ok_with_embedded_status_content`
      test, add the same assertion — the dashboard page embeds the same
      status content
- [ ] 4.3 Confirm `status_returns_ok_with_no_reading` still passes (no
      reason code in the no-data view)
- [ ] 4.4 Run the HTTP tests — GREEN

## 5. Lint, types, full suite

- [ ] 5.1 Run `cargo fmt --check` from `fermenter/` — clean
- [ ] 5.2 Run `cargo clippy --all-targets -- -D warnings` from `fermenter/`
      — clean
- [ ] 5.3 Run `cargo test` from `fermenter/` — full suite GREEN

## 6. OpenSpec validation

- [ ] 6.1 Run `openspec validate show-reason-code-on-dashboard --strict` —
      passes (confirms the MODIFIED delta against `web-dashboard` is
      well-formed, requirement names match exactly, scenarios use `####`
      headers)
- [ ] 6.2 Run `openspec status --change "show-reason-code-on-dashboard"` —
      `isComplete: true` (all `applyRequires` artifacts done)

## 7. Manual smoke test

- [ ] 7.1 From `fermenter/`, run `MOCK_SERIAL=true cargo run` and open
      `http://localhost:8080/` in a browser
- [ ] 7.2 Confirm the `Reason` row is visible next to `Action`, showing a
      reason code value (e.g. `RC2.1`, `RC4.1`) from the mock serial feed
- [ ] 7.3 Confirm the reason code changes as the mock feed cycles through
      different temperature states
