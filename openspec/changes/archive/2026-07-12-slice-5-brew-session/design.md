## Context

Slice-4 built the controller's first write path (`temperature-control`):
`web::AppState` grew a `target_tx: watch::Sender<f64>`, the ingest loop
reconciles against a paired `watch::Receiver<f64>` once per reading, and
`ControllerState { target_temp, brew_id }` round-trips through
`TimeStore::save_state`/`load_state`. That slice's design.md decision 1
explicitly deferred building `rewrite-plan.md` §4's `Command`/`mpsc`
abstraction, betting that `SetBrewId` (this slice) would either justify it or
confirm the simpler per-field `watch` channel shape. It also raised an open
question about whether `/target`'s persist call would clobber a concurrent
`/brew` change (or vice versa), tracked as unresolved.

Today, `brew_id` is a plain `String` field on `AppState` and a `&str`
parameter threaded through `ingest_loop`/`main.rs`, sourced once from
`config.default_brew_id` at startup and never changed. `write_reading` and
`last_reading` (`store/mod.rs`, `store/redis.rs`) already take `brew_id` as a
per-call parameter — persistence is already brew-scoped since slice-2 — so no
storage schema change is needed, only a way to change *which* brew_id the
running process currently uses.

## Goals / Non-Goals

**Goals:**

- A `tokio::sync::watch::channel<String>` carrying the active brew
  identifier, structurally identical to slice-4's `target_tx`/`target_rx`:
  the web layer's `POST /brew` handler sends a new value; the ingest side
  holds a cloned `Receiver` and reads the current value once per reading to
  tag persisted writes.
- `GET /brew` (form pre-filled with the current brew identifier) and
  `POST /brew` (validate, apply, persist, rehydrate, re-render) added to the
  web layer, reusing the `render`/`AppError` plumbing slice-3/4 established.
- On an accepted brew change, synchronously call
  `ingest::rehydrate_latest_reading` for the new identifier and overwrite the
  shared `Arc<Mutex<Option<Reading>>>` — the same rehydration startup already
  performs, now also triggered mid-run.
- Resolve slice-4's open persistence-clobber question (see Decision 2).
- Validation: a submitted brew identifier must be non-empty after trimming
  whitespace; invalid submissions re-render the form with an error and do
  **not** touch the watch channel, storage, or in-memory current state.

**Non-Goals:**

- No `Command`/`mpsc` enum — see Decision 1.
- No format constraints on the brew identifier beyond non-empty (no length
  cap, no character-set restriction). The old system
  (`controller/temp_controller.py`) only checked `type(brew_id) == str`;
  requiring non-empty is this slice's only tightening, matching slice-4's
  scope of adding one deliberate safety check, not a full redesign of what a
  valid brew id looks like.
- No UI for browsing or comparing historical brews' data (e.g. a brew picker
  showing past identifiers) — this slice only lets the operator relabel the
  *currently active* brew; charts and historical browsing remain deferred
  per `rewrite-plan.md`'s "UI scope" decision.
- No change to the Redis key schema or `TimeStore` trait — `write_reading`
  and `last_reading` already accept `brew_id` per call (slice-2).
- No de-duplication or validation against reusing a previous brew's
  identifier (e.g. switching back to an already-used id) — reusing an id
  simply resumes appending to that id's existing series, which is treated as
  acceptable, expected behaviour, not an error.

## Decisions

**1. Grow `web::AppState` with a `brew_tx: watch::Sender<String>` rather than
introducing the `Command`/`mpsc` abstraction, mirroring `target_tx` exactly.**
`rewrite-plan.md` §4 sketches `enum Command { SetTarget(f64), SetBrewId(String)
}` over an `mpsc::channel`, and slice-4 deferred it pending a second variant
to justify the enum. Now that `SetBrewId` exists, the question is whether it
finally justifies that abstraction. It does not: `brew_id`, like
`target_temp`, is a piece of *current state* the ingest side only ever needs
the latest value of — never a queue of historical relabel events to process
in order. `watch` remains the correct primitive for both, for the same
reason slice-4 chose it for target. Introducing `mpsc`/`Command` now would
add a layer of enum-matching indirection with no behavioural benefit over two
parallel, independently-testable `watch` channels. Alternative (build the
`Command` enum, fold both `target_tx` and `brew_tx` into one `mpsc` sender)
rejected: it would couple the two capabilities' channels together (a
`Command::SetTarget` and a `Command::SetBrewId` compete for the same queue
ordering guarantees neither needs) and would require both slice-4's
already-shipped `target_tx` call sites and this slice's new ones to be
migrated in the same change, widening this slice's blast radius for no
behavioural gain.

**2. `/target` and `/brew` each source the *other* field's current value
from its own live `watch` channel at persist time, rather than reading it
back through the store or clobbering it with a stale default.**
Slice-4's design.md left this as an explicit open question:
`persist_target(store, target_temp, brew_id)` always paired the new target
with whatever `brew_id` its caller passed in, and before this slice, that was
always the same static config-derived value — never a source of clobbering,
but only because nothing else could change concurrently. Now that `/brew`
also persists `ControllerState`, both handlers need the *other* half of the
pair to be the true current value, not stale. Because `AppState` already
holds both `target_tx` and `brew_tx` as the single source of truth for each
field, the simplest fix is: `set_target`'s handler persists
`ControllerState { target_temp: <new value>, brew_id: state.brew_tx.borrow().clone()
}`; symmetrically, `set_brew`'s handler persists
`ControllerState { target_temp: *state.target_tx.borrow(), brew_id: <new value> }`.
Neither handler needs to read the store back before writing — the live
channels already hold the authoritative current value for the field it isn't
changing. Alternative (read-modify-write: `load_state()`, apply the one
changed field, `save_state()`) rejected: it adds a network round-trip to
Redis on every mutation and introduces its own race (two concurrent
`POST`s could still interleave load/modify/save), whereas reading from the
in-process `watch` channel is synchronous, race-free (both channels are
independently authoritative for their own field), and requires no new
`TimeStore` method.

**3. Ingest reads the brew identifier from `brew_rx` once per reading via
`.borrow().clone()`, not held as a borrow across the `.await` points in the
loop body.**
`watch::Ref` (the guard returned by `.borrow()`) is not `Send` in general
usage across await points in the same way the existing `target_rx.borrow()`
call already avoids holding across awaits (it's read, copied into a `f64`,
and dropped before the next `.await`). The same pattern applies here:
`let brew_id = brew_rx.borrow().clone();` immediately clones into an owned
`String`, dropping the borrow guard before `store.write_reading(&brew_id,
&reading).await` and `source.write_target(..).await` run. This costs one
small string clone per reading — negligible compared to the network I/O
already happening in the same loop iteration.

**4. Brew switch triggers synchronous rehydration inline in the `POST
/brew` handler, not a background task or a signal consumed by the ingest
loop.**
Two designs were considered:
  - (a) The web handler alone calls `rehydrate_latest_reading` and
    overwrites `AppState.latest` directly — it already holds an `Arc<Mutex<
    Option<Reading>>>` clone and a `store` handle, both needed.
  - (b) The handler only sends on `brew_tx`; the ingest loop notices the
    change (via `brew_rx.has_changed()`) and performs the rehydration
    itself, then updates `latest`.

(a) is chosen. It requires no new plumbing (the handler already has every
dependency it needs) and takes effect immediately in the same request/response
cycle the operator is waiting on, rather than only becoming visible once the
ingest loop happens to notice the channel change on its own cadence (which,
per slice-4 decision 2, is reading-paced and could be seconds away — the
`/target` handler's underlying trade-off intentionally accepted for device
writes, but not appropriate for a UI-visible current-state field an operator
expects to update looking at the same page). (b) also duplicates the
rehydration call in two places conceptually (startup does it directly in
`main.rs`; this would add a second, loop-triggered call site) for no benefit.
Trade-off accepted: the handler briefly does two awaited I/O calls
(`persist_brew` then `rehydrate_latest_reading`) before responding, slightly
increasing `POST /brew`'s latency versus `/target`'s single persist call —
acceptable since this is a low-frequency, operator-initiated action, not a
per-reading hot path.

**5. `initial_brew_id` mirrors `initial_target`'s shape exactly: a pure
async helper in the new `brew_session.rs` module that calls
`store.load_state()` and returns the persisted `brew_id` if present,
otherwise a passed-in default.**
This means `main.rs` calls `store.load_state()` twice at startup — once via
`temperature_control::initial_target`, once via
`brew_session::initial_brew_id`. Alternative (a single combined
`initial_controller_state(store, default_target, default_brew) -> (f64,
String)` helper, one `load_state()` call) rejected: it would couple two
otherwise-independent, independently-unit-tested modules
(`temperature_control.rs`, `brew_session.rs`) into a single startup-only
function, for the sake of avoiding one extra in-process Redis read that only
happens once per process lifetime — not worth the coupling. The resolved
`initial_brew_id` value is also used to correct `main.rs`'s startup call to
`ingest::rehydrate_latest_reading`, which today always rehydrates against
`config.default_brew_id` even when a different brew was actually persisted
last — a latent gap this slice closes as part of wiring (tracked as a task,
not a spec change, since `temperature-monitoring`'s existing requirement text
already says "the current brew identifier", which was simply not being
resolved correctly before this slice gave `brew_id` a persisted, resolvable
value distinct from the static config default).

**6. Validation: reject empty or all-whitespace input; accept everything
else, trimmed.**
Mirrors slice-4 decision 4's approach of adding one deliberate, minimal
safety check beyond the old system's near-total absence of validation,
without over-specifying a format the project has no current requirement for.
`validate_brew_id` trims the input and returns the trimmed value on success,
so `"  01-IPA-v02  "` and `"01-IPA-v02"` are equivalent — consistent with
`validate_target`'s existing whitespace-trimming behaviour.

## Risks / Trade-offs

- **[Risk]** `POST /brew` now performs two sequential store calls
  (`persist_brew`, then `rehydrate_latest_reading`'s `last_reading`) before
  responding; if storage is slow or briefly unreachable, the request hangs
  longer than `/target`'s single call → **Mitigation**: both calls already go
  through the same reconnecting `ConnectionManager` `/target` relies on; a
  persistence failure is logged and does not block the response (matches
  `/target`'s existing log-and-continue treatment), and a rehydration
  failure falls back to `None` (no reading) exactly as `main.rs`'s startup
  path already does via `rehydrate_latest_reading`'s existing error handling
  — no new failure mode, just two call sites instead of one.
- **[Risk]** Two independent `watch` channels (`target_tx`, `brew_tx`) each
  authoritative for one field means there is a brief window between
  `set_target` and `set_brew` handlers running concurrently where each reads
  the other's channel at a slightly different instant — not a data race (each
  channel read is atomic and consistent), but the two persisted writes could
  theoretically apply in either order if requests overlap →
  **Mitigation**: both fields are eventually persisted correctly regardless
  of ordering (last write to each channel wins, and each handler always
  writes the *current* value of the field it isn't changing at the moment it
  persists) — no permanently incorrect state results, only a benign
  last-writer-wins ordering ambiguity under concurrent submissions, which is
  no worse than the old Python system's ZMQ-based single-writer config
  object had for the equivalent operation.
- **[Trade-off]** No dedicated "brew history browser" — switching back to a
  previously-used brew identifier silently resumes that id's existing time
  series rather than surfacing a warning or confirmation that data already
  exists under that name. Accepted as consistent with this slice's narrow
  scope (relabel only); a confirmation UX is a low-risk future addition if a
  real accidental-reuse incident occurs.

## Open Questions

- Should there be any limit on how frequently the brew identifier can be
  changed (e.g. debounce rapid accidental double-submits)? Current lean: no
  — `POST /target` has no such limit either, and a brew relabel is expected
  to be a rare, deliberate operator action; revisit only if manual testing
  surfaces an actual problem.
