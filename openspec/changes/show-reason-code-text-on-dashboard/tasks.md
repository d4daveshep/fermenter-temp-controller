## 1. Reason-code mapping module (red→green)

- [x] 1.1 Create `fermenter/src/reason_code.rs` with a top-of-file comment pointing to `arduino/TempController/TempControllerRules.md:27-58` as the source of truth
- [x] 1.2 Add `pub fn reason_text(code: &str) -> Option<&'static str>` returning `Some(<verbatim text>)` for every documented code: `RC1`, `RC6`, `RC5`, `RC10`, and `RC2.1`–`RC4.3`, `RC7.1`–`RC9.3` (text copied verbatim from `TempControllerRules.md:27-58`)
- [x] 1.3 Ensure `reason_text("")`, `reason_text("RC_ERR")`, and any unrecognized code (e.g. `"RC_FOO"`) all return `None`
- [x] 1.4 Add `#[cfg(test)] mod tests` with an assertion for every documented code (verbatim text), plus `""`, `"RC_ERR"`, and an unknown code asserting `None`
- [x] 1.5 Register the module in `fermenter/src/main.rs` (or wherever the crate's `mod` declarations live) and run `cargo test reason_code` to confirm green

## 2. Plumb the description into the dashboard context

- [x] 2.1 Add `pub reason_text: Option<String>` to `StatusContext` in `fermenter/src/web/handlers.rs:23-28`
- [x] 2.2 In `status_context()` (`handlers.rs:30-37`), set `reason_text` from the latest reading: `reason_code::reason_text(&r.reason_code).map(str::to_owned)` when a reading exists, `None` otherwise
- [x] 2.3 Confirm both the `dashboard` handler (`GET /`) and the `status` handler (`GET /status`) receive the new field via the shared `StatusContext`

## 3. Render the description in the status template

- [x] 3.1 Edit `fermenter/templates/partials/status.html:7` to:
  `<dt>Reason</dt><dd>{{ reading["reason-code"] }}{% if reason_text %} — {{ reason_text }}{% endif %}</dd>`
- [x] 3.2 Verify the bracket syntax `reading["reason-code"]` is preserved (hyphenated key) and that `reason_text` uses plain dot access

## 4. Update test fixtures to a real firmware code

- [x] 4.1 `grep -rn "below-target" fermenter/` to enumerate every fixture/assertion site before editing
- [x] 4.2 Update `sample_reading()` in `fermenter/src/web/handlers.rs:225` to use `reason_code: "RC3.1".to_owned()`
- [x] 4.3 Update `sample_reading()` in `fermenter/src/web/mod.rs:145` to use `reason_code: "RC3.1".to_owned()`
- [x] 4.4 Update any mock-serial reason-code fixture (e.g. `fermenter/src/main.rs:122-125`) if it asserts `"below-target"` — confirm via the grep in 4.1
- [x] 4.5 Update the literal assertion in `status_fragment_includes_reason_code` (`handlers.rs:266-277`) to assert both `"RC3.1"` and the mapped description text
- [x] 4.6 Update the HTTP-body assertions in `fermenter/src/web/mod.rs:173, 212` to assert `"RC3.1"` (and the description where appropriate)

## 5. Snapshot refresh and full verification

- [x] 5.1 Run `INSTA_UPDATE=always cargo test` from `fermenter/` to refresh `dashboard_renders_snapshot.snap` and `status_fragment_with_reading_snapshot.snap`
- [x] 5.2 Re-run `cargo test` from `fermenter/` and confirm all tests green (including the no-reading snapshot, which should be unaffected)
- [x] 5.3 Run `cargo fmt --check` from `fermenter/` and fix any formatting
- [x] 5.4 Run `cargo clippy --all-targets -- -D warnings` from `fermenter/` and fix any lints
- [x] 5.5 Visually diff the two refreshed snapshots to confirm the `Reason` cell now reads `RC3.1 — REST->REST because we are in the target range.  There is natural heating so expect temperature to rise` (or equivalent) and that the no-reading snapshot is unchanged

## 6. Spec sync and self-check

- [x] 6.1 Confirm the implementation satisfies both MODIFIED requirements in `specs/web-dashboard/spec.md` (dashboard page and `/status` fragment show code + description; unknown/empty/`RC_ERR` shows code alone)
- [x] 6.2 Confirm no changes were made to the serial contract, `Reading` struct shape, Redis schema, API JSON shape, or any firmware file
- [x] 6.3 Confirm the change works under both `cargo run` (disk templates) and `--features embed` (no new runtime file deps introduced)
