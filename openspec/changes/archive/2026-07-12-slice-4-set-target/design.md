## Context

Slices 1–3 built the ingest loop, `Reading`/`ControllerState` models, the
`TimeStore` trait (including `save_state`/`load_state`, defined since slice-2
but never called), and a read-only Axum dashboard. `web::AppState` today
(`src/web/mod.rs`) is deliberately minimal — just the shared
`Arc<Mutex<Option<Reading>>>`, `brew_id`, MiniJinja `Environment`, and an
`ingest_alive` flag — per slice-3's design.md decision 1, which explicitly
deferred `watch`/`mpsc` channels until a handler actually needs to *send* a
command into the ingest side. This slice is that command: `SetTarget`.

`SerialSource::write_target` (`src/serial/mod.rs`) and `MockSerialSource`'s
`written_targets` recorder have existed since slice-1 with no caller. The
Redis `TimeStore::save_state`/`load_state` methods have existed since
slice-2, exercised only by their own unit tests. This slice is the first real
consumer of both.

`temperature-control` is a brand-new capability (`## ADDED`).
`web-dashboard` gains its first mutating route, which requires narrowing
(not deleting) its existing "read-only" requirement — the dashboard and
`/status` fragment stay read-only; `/target` is explicitly a new,
separate surface. `device-connection`'s existing `<target>` write-framing
scenario (specified but unexercised since slice-1) gets a real caller plus a
new "write only when changed" requirement.

## Goals / Non-Goals

**Goals:**

- A `tokio::sync::watch::channel<f64>` carrying the desired target
  temperature, per `rewrite-plan.md` §5: the web layer's `POST /target`
  handler sends a new value; the ingest side holds a cloned `Receiver` and
  compares it against each incoming `Reading`'s own `target` field (the
  Arduino's self-reported current target).
- Reconcile-on-reading: after each successfully parsed `Reading`, if the
  current watch value differs from `reading.target`, call
  `source.write_target(desired)`. This piggybacks the check on the existing
  per-reading loop iteration rather than adding a second concurrent listener
  or a `tokio::select!` over `Receiver::changed()` — see Decision 2.
- `GET /target` (form pre-filled with the current target) and
  `POST /target` (validate, apply, persist, re-render) added to the web
  layer, using the same `render`/`AppError` plumbing slice-3 established.
- Wire `ControllerState`/`save_state`/`load_state` into `main.rs`: on
  startup, load persisted state and seed the watch channel's initial value
  from it (falling back to `DEFAULT_TARGET_TEMP` config if nothing
  persisted); on every accepted `POST /target`, persist the new
  `ControllerState`.
- Validation: a submitted target must parse as a finite `f64` within a fixed
  sane range; invalid submissions re-render the form with an error and do
  **not** touch the watch channel or storage.

**Non-Goals:**

- No brew-id mutation or its own channel — that's slice-5 (`brew-session`).
  `ControllerState.brew_id` is populated from `config.default_brew_id`
  every time this slice saves state; it is not itself editable here.
- No real serial hardware — target reconcile is exercised against
  `MockSerialSource.written_targets`, per the existing test-double pattern.
  Real `tokio-serial` wiring is slice-6.
- No dedicated timer/background reconcile task independent of the reading
  cadence. `rewrite-plan.md` §5 calls this "event-driven" relative to the old
  system's 5s poll, but a genuinely instantaneous push (reconcile the instant
  `POST /target` lands, not waiting for the next reading) is deferred — see
  Decision 2's trade-off.
- No configurable target range (min/max) via env var — the range is a fixed
  constant this slice; making it configurable is a low-risk follow-up, not
  required by any current requirement.

## Decisions

**1. Grow `web::AppState` with a `target_tx: watch::Sender<f64>` rather than
introducing a new command-channel abstraction.**
`rewrite-plan.md` §4 sketches a `Command` enum (`SetTarget`, `SetBrewId`)
over an `mpsc::channel`. Building the full enum now, when this slice only
ever sends one variant, would speculatively couple this slice to slice-5's
`SetBrewId` before it exists. Instead: `AppState` gets a plain
`target_tx: watch::Sender<f64>`; the ingest side is constructed with the
paired `watch::Receiver<f64>`. `watch` (not `mpsc`) is deliberate — the
consumer only ever cares about the *latest* desired value, never a queue of
historical ones, which is exactly `watch`'s semantics and avoids ingest
falling behind on stale intermediate values if an operator submits several
times in quick succession. Alternative (introduce the `Command` enum +
`mpsc` now for both target and future brew-id) rejected as premature — slice-5
can introduce the enum when it has a second variant to justify it, folding
`target_tx` into it at that point if that turns out to be the better shape.

**2. Reconcile happens inline in `ingest_loop`, once per reading, not via a
concurrent `tokio::select!` on `Receiver::changed()`.**
Two designs were considered:
  - (a) `tokio::select! { line = source.read_line() => ..., _ = target_rx.changed() => ... }`
    — reacts to a target change immediately, independent of when the next
    reading arrives.
  - (b) After each reading is parsed, borrow the current watch value and
    compare it to `reading.target`; write if different.

(b) is chosen. Rationale: the Arduino already reports its own current target
on every reading (`reading.target`), so reconciliation is naturally
reading-paced — real hardware readings arrive every few seconds
(`system-analysis.md` §2), which is comparable to the old 5s poll this
replaces, so (a)'s sub-reading-interval responsiveness is not a requirement
anything currently demands. (b) also sidesteps a real hazard with the
finite, self-exhausting `MockSerialSource` used by `main.rs`'s demo scenario:
under (a), once `read_line()` permanently returns the "exhausted" error and
the loop `break`s, nothing is left listening on `target_rx` at all — a
worse regression than (b)'s reading-paced reconcile. (b) keeps the same
single-loop structure slice-1–3 already established. Trade-off accepted:
if no reading ever arrives (e.g., device genuinely silent), a target change
submitted via the UI won't reach the device until the next reading is
processed — acceptable since a silent device has no working write path
either way at that point.

**3. `ControllerState` is saved/loaded as a whole, but this slice only ever
originates the `target_temp` half of it.**
`ControllerState { target_temp, brew_id }` is the persisted unit
(`model.rs`, unchanged). This slice's `save_state` calls always pair the new
`target_temp` with `config.default_brew_id` — there is no other brew_id
source yet. On startup, `load_state()`'s `target_temp` (if present) seeds
the watch channel; a mismatched persisted `brew_id` vs. the configured one is
not reconciled or even compared this slice — brew-id ownership is entirely
slice-5's concern. Alternative (split persistence into two independent keys,
one per field) rejected — `ControllerState` is already the shared shape
slice-2 defined for exactly this pair, and splitting it now would be
speculative given slice-5 needs the combined roundtrip anyway.

**4. Fixed validation range: reject targets outside `0.0..=40.0` (°C) or
non-finite values; anything else accepted.**
The old Python system (`web/fastapi_app.py`, `controller/temp_controller.py`)
never validated the target beyond "parses as a float" — any value, including
absurd or negative ones, was accepted and pushed to the device. This slice
tightens that: FastAPI's own `float` path/form coercion already rejects
non-numeric input at the framework layer; on top of that, this slice adds an
explicit fermentation-sane range (0–40°C covers everything from a cold lager
to a hot sour) as a deliberate safety improvement, not a requirement carried
over from the old system. The bound is a `const` in `web/handlers.rs`, not an
env var — making it configurable is a low-risk future add, not needed now.
Alternative (no range check, matching old-system parity exactly) rejected —
free to reintroduce if the fixed bound proves wrong for some fermentation
style, but shipping no safety bound at all when this slice is already adding
new validation UX (re-rendered form + error) is not a good default.

**5. `POST /target` handler validates, updates `target_tx`, persists via
`store.save_state`, then re-renders `target_form.html` with a success or
error message — no redirect.**
Slice-3 didn't have any `POST` route to set a pattern for. A redirect (PRG
pattern) was considered — rejected for now because HTMX-driven form
submission (`hx-post="/target"`) more naturally swaps the response HTML
directly into the page than following a redirect, and there is no risk of
duplicate-submit-on-refresh with HTMX's swap model the way there is with a
plain HTML form post. Non-HTMX plain-form fallback (progressive enhancement)
is not addressed this slice — the dashboard already requires JS (HTMX) for
its polling, so this is consistent with slice-3's existing baseline.

## Risks / Trade-offs

- **[Risk]** Reconcile-on-reading (Decision 2) means a target change is
  invisible to the device until the next reading arrives, not instantly →
  **Mitigation**: matches real hardware's reading cadence; documented
  as an accepted trade-off, revisit only if a future requirement demands
  sub-reading responsiveness.
- **[Risk]** `ControllerState.brew_id` is always written as
  `config.default_brew_id` this slice, so if slice-5 later persists a
  different brew_id concurrently, save-state calls from this slice's
  `/target` handler could clobber it back to the config default →
  **Mitigation**: called out explicitly as a slice-5 integration concern in
  Open Questions below; slice-5 will need to source the brew_id from
  authoritative runtime state (not `config.default_brew_id`) when this
  slice's handler saves state, or merge the two save-paths.
- **[Trade-off]** No configurable validation range — accepted as a fixed
  constant; a real brewer needing e.g. sous-vide-adjacent temperatures above
  40°C would need a code change. Documented as a known, low-risk limitation.
- **[Risk]** `watch::Sender` requires a live `Receiver` to avoid the sender
  observing "channel closed" — since `AppState`'s `target_tx` and the ingest
  task's `target_rx` are both constructed once in `main.rs` and held for the
  process lifetime, this isn't expected to occur in practice, but the
  `.send()` call's `Result` is still handled (logged, not panicked) rather
  than unwrapped, in case the ingest task ever exits first.

## Open Questions

- When slice-5 (`brew-session`) adds brew-id mutation, should it and this
  slice's `/target` handler share a single `save_state` call path (so
  neither can clobber the other's field), or does slice-5 need to read the
  latest `ControllerState` before writing to merge in the other field?
  Current lean: slice-5 should read-modify-write (load current state, apply
  the brew-id change, save) rather than duplicate slice-4's shape — tracked
  as a slice-5 design concern, not resolved here.
- Is `0.0..=40.0`°C the right fixed validation bound for all fermentation
  styles this controller might ever run? Current lean: yes for now: revisit
  if a real use case needs wider range, at which point making it configurable
  via env var is the likely fix.
