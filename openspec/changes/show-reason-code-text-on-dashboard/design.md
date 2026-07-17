## Context

The prior slice `2026-07-15-show-reason-code-on-dashboard` added a `Reason` row to the dashboard that renders the raw `reason-code` string emitted by the Arduino firmware (`RC1`, `RC3.1`, `RC5`, `RC_ERR`, …). That string is opaque to operators: understanding `RC3.1` requires opening `arduino/TempController/TempControllerRules.md` and matching the code against a table. The Rust side treats `reason_code` as a free-form `String` (`fermenter/src/model.rs:11`) with no enum, no `Display` impl, and no text mapping — the value flows untouched from serial → model → Redis → template.

The canonical human-readable text for each code lives only in firmware docs: `arduino/TempController/TempControllerRules.md:27-58` is the authoritative table, with inline comments at each `setReasonCode(...)` call site in `ControllerActionRules.cpp` as a secondary source. There is no firmware-side `getReasonCodeText()` (the firmware has `Decision::getActionText()` at `arduino/TempController/Decision.cpp:13-28` as a precedent, but adding a firmware-side text API is out of scope and would risk the sacred serial contract).

This slice closes that gap entirely on the Rust side by introducing a static code-to-text mapping and rendering the text alongside the code on the dashboard. It is the explicitly tracked follow-up recorded in `openspec/changes/archive/2026-07-15-show-reason-code-on-dashboard/design.md` and `BACKLOG.md:5`.

Constraints inherited from the project:
- The serial contract is sacred (`openspec/config.yaml:22-24`); this change MUST NOT alter the JSON shape or the `reason-code` field semantics.
- MiniJinja templates load from disk in dev and are baked in via `--features embed` for the release image; any template change works in both modes with no new runtime file dependencies.
- Bracket syntax `{{ reading["reason-code"] }}` is required in templates because the field is `#[serde(rename = "reason-code")]` (hyphenated key — dot access does not work in MiniJinja).
- TDD red→green→refactor; insta snapshots guard HTML; CI runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.

## Goals / Non-Goals

**Goals:**
- Render a human-readable description next to the raw reason code in the dashboard `Reason` row and in the live-polled `/status` fragment, so operators can read the controller's rationale without leaving the UI.
- Source the descriptions from a single Rust-side static mapping that mirrors `TempControllerRules.md:27-58`.
- Handle unknown, empty, and error-sentinel codes gracefully by showing only the raw code (no fabricated text).
- Keep the change Rust-side only: no serial, model, Redis, or firmware changes.
- Keep the template logic-free; compute the text in the handler and pass it as context.

**Non-Goals:**
- Tooltip / popover UI for the description (a plain inline text suffix is sufficient).
- Persisting the description to Redis Time Series (the raw code remains the stored source of truth; text is derived at render time).
- Adding `getReasonCodeText()` to the Arduino firmware (would touch the sacred serial contract for no benefit — the mapping is a presentation concern).
- Internationalization / multiple languages.
- Auto-generating the Rust mapping from the firmware `.md` file at build time (the table is small and stable; a hand-maintained module with a unit test asserting coverage is simpler and avoids a build-time file dependency).
- Changing the `Reading` struct shape or the API/JSON representation of readings.

## Decisions

### D1: Mapping lives in a new `fermenter/src/reason_code.rs` module

**Choice:** A dedicated module exposing `pub fn reason_text(code: &str) -> Option<&'static str>`.

**Rationale:** Keeps the mapping isolated from the model and web layers, is independently unit-testable, and avoids bloating `model.rs` or `handlers.rs`. A free function returning `Option<&'static str>` is the smallest surface that satisfies the template's needs and makes the "unknown code" case explicit in the type system.

**Alternatives considered:**
- *Method on `Reading` (e.g. `Reading::reason_text(&self)`):* couples the mapping to the model and requires a `Reading` instance to call, which is awkward for the no-reading path and for unit-testing the mapping in isolation. Rejected.
- *Inline `match` in the handler:* scatters presentation logic in the web layer and is harder to test exhaustively. Rejected.
- *A `ReasonCode` enum replacing the `String` field:* would require a custom `Deserialize` impl that handles unknown codes without failing (the firmware may emit new codes), and would touch the model/serial boundary. Out of scope for a presentation-only change. Rejected.

### D2: Mapping returns `Option<&'static str>`; `None` covers empty, unknown, and `RC_ERR`

**Choice:** `reason_text("")` → `None`; `reason_text("RC_ERR")` → `None`; `reason_text("RC_FOO")` (any unrecognized code) → `None`. Every code documented in `TempControllerRules.md:27-58` returns `Some(...)` with the verbatim table text. `RC6` and `RC10` are included even though they are not currently emitted by `ControllerActionRules.cpp`, because the rules table lists them as aliases of `RC1`/`RC5` and including them costs nothing.

**Rationale:** The dashboard's behavior for the "no description available" case is then uniform: show the raw code alone. `RC_ERR` is a sentinel for an internal controller error; showing a fabricated "description" would be misleading, and the raw `RC_ERR` is itself meaningful to an operator. Mapping it to `None` means the UI shows `RC_ERR` with no trailing text — an honest signal that something is wrong rather than a soothing but invented sentence.

**Alternatives considered:**
- *Return `&'static str` with an "Unknown reason code" fallback:* rejected per the user's decision — the raw code alone is preferred so the operator sees the actual code for debugging and the absence of text is a visible prompt that the mapping needs updating.
- *Map `RC_ERR` to a custom error string:* couples the Rust side to a firmware sentinel's meaning and risks drift. Rejected.

### D3: Text is computed in `status_context()` and passed to the template as `reason_text: Option<String>`

**Choice:** Add a field `reason_text: Option<String>` to `StatusContext` (`fermenter/src/web/handlers.rs:23-28`). In `status_context()` (`handlers.rs:30-37`), compute it from the latest reading: when `reading` is `Some(r)`, set `reason_text = reason_code::reason_text(&r.reason_code).map(str::to_owned)`; when there is no reading, leave it `None`.

**Rationale:** Mirrors the existing pattern where `status_context()` assembles all display data and the template stays logic-free. Using `Option<String>` (rather than `Option<&str>`) keeps `StatusContext` owned and easy to pass into MiniJinja without lifetime juggling. Because `status_context()` feeds both the dashboard page and the `/status` fragment, both routes get the text for free and the "fragment matches dashboard" spec scenario continues to hold.

**Alternatives considered:**
- *Call `reason_text()` directly from the template via a method/global:* pushes logic into the template and requires registering a MiniJinja global/function. Rejected — the current architecture deliberately keeps templates logic-free.
- *Add a `reason_text` field to `Reading`:* would change the model and leak presentation into the data layer. Rejected.

### D4: Template renders `RC3.1 — <description>` in the existing `Reason` `<dd>`

**Choice:** `fermenter/templates/partials/status.html:7` becomes:
```
<dt>Reason</dt><dd>{{ reading["reason-code"] }}{% if reason_text %} — {{ reason_text }}{% endif %}</dd>
```
Bracket syntax is retained for `reading["reason-code"]` (hyphenated key). An em-dash (` — `, space-emdash-space) separates the code from the description.

**Rationale:** Keeps the raw code visible (preserves the prior slice's debug affordance and the existing spec requirement that the code be shown) while adding the human-readable text in the same cell. A single `<dd>` avoids adding a new row and keeps the reading `<dl>` compact. The `{% if reason_text %}` guard makes the no-description case render as just the raw code with no trailing dash.

**Alternatives considered:**
- *Replace the code with the text:* loses the debug affordance and weakens the existing spec requirement that the code be displayed. Rejected per the user's decision.
- *Add a new `<dt>Reason (description)</dt>` row:* doubles the vertical footprint of the reading block for a secondary detail. Rejected per the user's decision.

### D5: Update test fixtures from `"below-target"` to `"RC3.1"`

**Choice:** Change `sample_reading()` in `fermenter/src/web/handlers.rs:225` and `fermenter/src/web/mod.rs:145` to emit `reason_code: "RC3.1".to_owned()` (and update the mock serial line in `fermenter/src/main.rs:122-125` only if its reason code is also `"below-target"` and asserted on — to be confirmed at implementation time). Update the literal `"below-target"` assertions in `handlers.rs:266-277` and `mod.rs:173, 212` to assert both `"RC3.1"` and the mapped description text. Refresh the two affected snapshots via `INSTA_UPDATE=always cargo test`.

**Rationale:** `"below-target"` is not a real firmware code and would map to `None`, which would exercise only the fallback path and hide the text-rendering behavior in tests. Switching to `RC3.1` (a documented, currently-emitted code with a stable description) makes the tests assert the real mapping end-to-end.

**Alternatives considered:**
- *Add `"below-target"` as a synonym in the mapping:* pollutes the production mapping with a test-only string. Rejected per the user's decision.
- *Keep `"below-target"` and have tests assert only the fallback:* leaves the text-rendering path untested at the snapshot/HTTP level. Rejected.

### D6: Unit-test the mapping exhaustively, snapshot-test the rendering

**Choice:** `fermenter/src/reason_code.rs` contains `#[cfg(test)] mod tests` with a case for every documented code (asserting the verbatim text), plus cases for `""`, `"RC_ERR"`, and an unknown `"RC_FOO"` asserting `None`. The dashboard/fragment rendering is covered by the existing insta snapshots (refreshed) and the updated HTTP/unit assertions.

**Rationale:** Exhaustive unit coverage of the mapping catches drift between the Rust table and `TempControllerRules.md`; the snapshot tests catch template regressions. This matches the project's TDD and insta conventions.

## Risks / Trade-offs

- **[Mapping drift]** The Rust mapping and `TempControllerRules.md` can diverge if the firmware adds a code and the docs/Rust table are not updated in lockstep. → **Mitigation:** The mapping's unit tests enumerate every documented code; an unknown code renders as the raw code alone (a visible signal, not a crash). The mapping is small and changes rarely. Document the source-of-truth pointer (`TempControllerRules.md:27-58`) in a comment at the top of `reason_code.rs` so future editors know where to sync from.
- **[Firmware emits an undocumented code]** If a future firmware build emits a code not in the Rust table, the dashboard shows only the raw code with no description. → **Mitigation:** This is the chosen fallback behavior, not a failure. The operator still sees the code; the missing text is a prompt to update the mapping. No incorrect text is ever shown.
- **[Template change breaks embed builds]** A new template field that only works in dev (disk-loaded) but not in the baked-in release image would be a release blocker. → **Mitigation:** `reason_text` is plain string data on `StatusContext`, referenced with standard MiniJinja `{{ }}` syntax; no new template files, no runtime file deps. CI's build-only cross-compile job and the existing snapshot tests (which run against the same template loader) cover this. No new `--features embed` code path is introduced.
- **[Snapshot churn]** Two snapshots change. → **Mitigation:** Low risk; `INSTA_UPDATE=always cargo test` then `cargo test` to confirm green, per `AGENTS.md`. The snapshot diff itself is a reviewable artifact.
- **[Fixture churn across 3+ sites]** Updating `sample_reading()` in two files plus assertions could miss a site. → **Mitigation:** `grep` for `"below-target"` across `fermenter/` at implementation time to enumerate every occurrence before editing; the tasks list calls this out explicitly.
