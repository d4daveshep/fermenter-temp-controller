## Why

The dashboard surfaces the latest reading but gives the operator no indication
of how current the displayed state is — the page could be showing a reading
from seconds ago or from hours ago, and there is no on-screen clock to tell
them the server is still alive and polling. The status fragment already
auto-refreshes every 5s via HTMX, so a server-time stamp placed in that
fragment would tick alongside the readings with no extra plumbing, giving the
operator a constant, visible freshness signal.

## What Changes

- Add a local-server-time timestamp to the live-polled status fragment
  (`partials/status.html`), formatted `DD-MMM-YYYY HH:MM:SS`
  (e.g. `14-Jul-2026 14:30:45`). Because the fragment is re-fetched every 5s
  by the dashboard's existing `hx-get="/status" hx-trigger="every 5s"` poll,
  the displayed time stays current without any client-side scripting or new
  endpoint.
- The timestamp reflects the server's local wall clock at render time (not the
  reading's ingest time), so it advances on every poll even when readings
  themselves stop arriving — making a stalled ingest loop visibly
  distinguishable from a frozen browser tab.
- Remove the dormant `Last updated: {{ updated_at }}` block in
  `partials/status.html`. That branch has never rendered because
  `StatusContext` has no `updated_at` field; it is dead markup carried since
  slice-3. It is removed here rather than wired up, because `updated_at`
  semantics ("when the reading arrived") differ from this slice's goal
  ("server is alive and rendering now"), and leaving a second dormant branch
  alongside the new one would repeat the original mistake.
- No new HTTP routes, no changes to the serial contract, no changes to
  storage. Read-only dashboard behaviour is preserved.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `web-dashboard`: the "Render a current-state dashboard page" and "Serve a
  live-polled current-state fragment" requirements gain a scenario asserting
  the fragment includes a local-server-time timestamp in
  `DD-MMM-YYYY HH:MM:SS` format, refreshed on each poll. The dormant
  `updated_at` markup is removed.

## Impact

- **Dependencies**: add `chrono = { version = "0.4", features = ["clock"] }`
  to `fermenter/Cargo.toml` `[dependencies]`. `chrono` is already widely used
  in the Rust ecosystem and pulls no new C deps for the `clock` feature on the
  Pi's `linux/arm64` target. No other time crate (`time`, `humantime`) is
  already present in the tree, so `chrono` is introduced fresh.
- **Modified code**:
  - `fermenter/src/web/handlers.rs` — `StatusContext` gains a `server_time:
    String` field; `status_context()` populates it via
    `chrono::Local::now().format("%d-%b-%Y %H:%M:%S").to_string()`; the two
    `insta` snapshot test constructors (`status_fragment_with_reading_snapshot`,
    `status_fragment_without_reading_snapshot`) are updated to supply a
    fixed `server_time` so snapshots remain deterministic.
- **Modified templates**: `fermenter/templates/partials/status.html` — remove
  the `{% if updated_at %}` block; add
  `<p class="server-time">Server time: {{ server_time }}</p>` outside the
  `{% if reading %}` block so it renders even before any reading arrives.
- **Updated snapshots**: `fermenter/src/web/snapshots/
  fermenter__web__handlers__tests__status_fragment_with_reading_snapshot.snap`
  and `..._without_reading_snapshot.snap` regenerated via
  `INSTA_UPDATE=always cargo test` then `cargo insta accept`.
- **No impact** on the ingest loop, serial contract, Redis schema, target/brew
  forms, `/healthz`, or the Arduino firmware.
- **No impact** on deployment packaging beyond the new transitive dependency;
  the `linux/arm64` cross-compile path in `.github/workflows/rust.yml` and
  `scripts/build_and_ship_image.sh` is unaffected since `chrono` builds
  cleanly on that target.
