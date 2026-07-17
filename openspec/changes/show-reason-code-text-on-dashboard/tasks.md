## Historical TDD note

The original completed checklist did not record either pre-implementation RED
run. Those executions cannot be verified or truthfully backfilled after the
implementation exists. The RED checkpoints below document the required process
for this change.

## 1. Reason-code mapping module (RED -> GREEN)

- [x] 1.1 Add table-driven `#[cfg(test)]` cases for every documented code, asserting its verbatim `TempControllerRules.md:27-58` text, plus `""`, `"RC_ERR"`, and unknown codes asserting `None`
Historical RED checkpoint: before implementing the lookup table, the new
mapping tests should have been run with `cargo test reason_code` and confirmed
to fail. This execution was not recorded and cannot be reconstructed now.
- [x] 1.3 Create `fermenter/src/reason_code.rs` with a source-of-truth comment pointing to `arduino/TempController/TempControllerRules.md:27-58`
- [x] 1.4 Add `pub fn reason_text(code: &str) -> Option<&'static str>` returning verbatim text for `RC1`, `RC6`, `RC5`, `RC10`, `RC2.1`-`RC4.3`, and `RC7.1`-`RC9.3`; return `None` for unrecognized, empty, and `RC_ERR` codes
- [x] 1.5 Register the module in `fermenter/src/main.rs` (or the crate's module declarations) and run `cargo test reason_code` to confirm GREEN

## 2. Plumb the description into the dashboard context

- [x] 2.1 Add `reason_text: Option<String>` to `StatusContext` in `fermenter/src/web/handlers.rs`
- [x] 2.2 In `status_context()`, derive the field from the latest reading with `reason_code::reason_text(&r.reason_code).map(str::to_owned)` and use `None` when no reading exists
- [x] 2.3 Confirm the shared `StatusContext` supplies the field to both the dashboard handler (`GET /`) and status handler (`GET /status`)

## 3. Render the description in the status template (RED)

- [x] 3.1 Edit `fermenter/templates/partials/status.html` so its existing Reason `<dd>` renders `{{ reading["reason-code"] }}{% if reason_text %} — {{ reason_text }}{% endif %}`
- [x] 3.2 Preserve bracket access for the hyphenated `reading["reason-code"]` key and use ordinary dot access for `reason_text`
Historical RED checkpoint: after the template change and before snapshot
acceptance, `cargo test status_fragment_with_reading_snapshot` should have
failed because the snapshot lacked the description. This execution was not
recorded and cannot be reconstructed now.

## 4. Snapshot update (GREEN)

- [x] 4.1 Search `fermenter/` for `"below-target"` to enumerate all fixture and assertion sites before updating them
- [x] 4.2 Update `sample_reading()` in `fermenter/src/web/handlers.rs` and `fermenter/src/web/mod.rs` to use `reason_code: "RC3.1".to_owned()`
- [x] 4.3 Update any asserted mock-serial `"below-target"` fixture discovered by the search
- [x] 4.4 Regenerate `dashboard_renders_snapshot.snap` and `status_fragment_with_reading_snapshot.snap` with `INSTA_UPDATE=always cargo test`
- [x] 4.5 Re-run the affected snapshot tests and confirm GREEN; confirm the no-reading snapshot remains unchanged
- [x] 4.6 Review the refreshed snapshots and confirm the Reason cell contains `RC3.1` followed by its mapped description

## 5. Format-regression unit tests

- [x] 5.1 Update `status_fragment_includes_reason_code` in `fermenter/src/web/handlers.rs` to render the status partial with `RC3.1` and assert both the Reason label/code and mapped description are present
- [x] 5.2 Add a rendered-template regression test for an unknown code, `""`, and `"RC_ERR"`, asserting each displays the raw code without a separator or fabricated description
- [x] 5.3 Run the format-regression unit tests and confirm GREEN

## 6. HTTP-level verification (web-dashboard spec scenarios)

- [x] 6.1 In `status_returns_ok_with_latest_reading`, assert the `GET /status` body contains both `RC3.1` and its mapped description
- [x] 6.2 In `dashboard_returns_ok_with_embedded_status_content`, assert the `GET /` body contains both `RC3.1` and its mapped description
- [x] 6.3 Add HTTP-level coverage that an unknown, empty, or `RC_ERR` code renders the raw code alone with no trailing separator or description
- [x] 6.4 Run the affected `tower::oneshot` tests and confirm GREEN

## 7. Lint, types, and full suite

- [x] 7.1 Run `cargo fmt --check` from `fermenter/` and fix formatting
- [x] 7.2 Run `cargo clippy --all-targets -- -D warnings` from `fermenter/` and fix lints
- [x] 7.3 Run `cargo test` from `fermenter/` and confirm the full suite is GREEN
- [x] 7.4 Run `cargo build --features embed` from `fermenter/` to verify the baked-template build path

## 8. OpenSpec validation and self-check

- [x] 8.1 Run `openspec validate show-reason-code-text-on-dashboard --strict`
- [x] 8.2 Run `openspec status --change "show-reason-code-text-on-dashboard"` and confirm `isComplete: true`
- [x] 8.3 Confirm the implementation satisfies both MODIFIED `web-dashboard` requirements: dashboard and `/status` show code plus description, while unknown/empty/`RC_ERR` show the code alone
- [x] 8.4 Confirm no serial-contract, `Reading` shape, Redis schema, API JSON shape, or firmware changes were made

## 9. Manual smoke test

- [x] 9.1 From `fermenter/`, run `MOCK_SERIAL=true cargo run` and request the dashboard in a browser or with a local HTTP client
- [x] 9.2 Confirm the dashboard Reason cell renders the mapped terminal `RC1` mock reading during the feed's pause; deterministic tests cover the `RC3.1` fixture
- [x] 9.3 Confirm an unknown/error/empty reason code, when injected through the test path, displays only the raw code with no dangling separator
