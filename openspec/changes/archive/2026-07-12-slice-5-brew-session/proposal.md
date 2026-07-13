## Why

Slice-4 gave the controller a write path for target temperature but
deliberately left `brew_id` static — sourced once from `DEFAULT_BREW_ID` at
startup and never editable, per that slice's non-goals. A real brewer starts
a new batch far more often than they restart the process, and today the only
way to relabel is to restart with a different env var, silently losing the
in-memory link to the just-finished brew's history. This slice closes that
gap: an operator can relabel the active brew at runtime from the dashboard,
readings ingested afterward are tagged and queryable under the new
identifier, and the previous brew's persisted history stays intact and
retrievable under its own id — unlocking the `brew-session` capability
`rewrite-plan.md` §4's `Command::SetBrewId` variant anticipated.

## What Changes

- Add `GET /brew` (form showing the current brew identifier) and `POST /brew`
  (validate + apply a new brew identifier) routes to the web layer, mirroring
  slice-4's `/target` form.
- Grow `web::AppState`: replace the static `brew_id: String` field with a
  `brew_tx: watch::Sender<String>` (paired with a `watch::Receiver<String>`
  held by the ingest side), matching slice-4's `target_tx` shape rather than
  introducing the `Command`/`mpsc` abstraction `rewrite-plan.md` §4 sketches
  — see design.md decision 1.
- `ingest_loop` reads the current brew identifier from the `watch` channel
  once per reading (replacing its fixed `brew_id: &str` parameter), so every
  reading persisted after a runtime switch is tagged with the newly active
  identifier.
- On an accepted brew change, immediately rehydrate the in-memory current
  state from the new brew's most recently persisted reading (or an explicit
  no-data state), reusing `ingest::rehydrate_latest_reading` — the same
  function startup already calls — so the dashboard never shows the
  previous brew's reading mislabeled under the new identifier while waiting
  for the next serial line.
- Persist the brew identifier via the existing `ControllerState`
  save/load roundtrip, resolving slice-4's design.md Open Question about the
  two mutating handlers (`/target`, `/brew`) clobbering each other's field:
  each handler now sources the *other* field's current value from its own
  live `watch` channel at persist time (see design.md decision 2), rather
  than a stale static default or a read-modify-write through the store.
- Basic validation: reject an empty or whitespace-only brew identifier with
  a re-rendered form and an error message, tightening the old Python system's
  "any string is accepted" behaviour (`controller/temp_controller.py`), which
  itself is the same style of deliberate safety tightening slice-4 applied to
  target-temperature validation.

## Capabilities

### New Capabilities

- `brew-session`: Brew-identifier lifecycle — accepting an operator-submitted
  relabel at runtime, holding it as the current, authoritative identifier
  that tags subsequent readings and persisted state, persisting/restoring it
  across restarts, and rehydrating current state to the newly active brew's
  history immediately on switch.

### Modified Capabilities

- `web-dashboard`: adds a brew-identifier form (`GET`/`POST /brew`),
  following the same pattern the target-setpoint form established; the
  existing "dashboard rendering is read-only" requirement already exempts
  mutating routes generically and needs no further narrowing.
- `reading-history`: clarifies that persisted readings are tagged with
  whichever brew identifier is active *at write time*, which can change
  mid-run — readings persisted before a switch remain queryable under their
  original identifier, unaffected by a later switch.

## Impact

- **Modified modules**: `web/mod.rs` (`AppState.brew_id: String` →
  `brew_tx: watch::Sender<String>`), `web/handlers.rs` (new `brew_form` and
  `set_brew` handlers; `StatusContext` reads `brew_tx` instead of the removed
  field), `ingest.rs` (`ingest_loop` takes `brew_rx: &watch::Receiver<String>`
  instead of `brew_id: &str`), `main.rs` (build the channel, seed it from
  rehydrated/persisted state, wire it into both the ingest task and
  `AppState`), `temperature_control.rs` (caller of `persist_target` now
  sources the paired `brew_id` from the live channel instead of a static
  field — no signature change).
- **New module**: `brew_session.rs` — pure `validate_brew_id`,
  `initial_brew_id`, and `persist_brew` helpers, mirroring
  `temperature_control.rs`'s shape.
- **New template**: `templates/brew_form.html` (`{% extends "base.html" %}`),
  extending `dashboard.html` with a link/control to reach it.
- **No new dependencies** — reuses `tokio::sync::watch`, already in use since
  slice-4.
- **No impact** on the Redis schema — `write_reading`/`last_reading` already
  take `brew_id` as a parameter (slice-2); no per-brew data migration needed.
- **No impact** on the Python system, which remains deployable on `master`
  until cutover.
