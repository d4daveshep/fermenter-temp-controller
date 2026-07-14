## Context

The `web-dashboard` capability (slice-3) renders the latest controller state
via two routes: `GET /` (full page) and `GET /status` (HTMX-polled fragment,
re-fetched every 5s by `dashboard.html`'s `hx-get="/status" hx-trigger="every
5s"`). Both routes share `StatusContext` (`fermenter/src/web/handlers.rs:23`)
and render `partials/status.html`, so any field added to `StatusContext`
appears in both render paths with no branching.

The fragment template already contains a dormant
`{% if updated_at %}<p ...>Last updated: {{ updated_at }}</p>{% endif %}` block
that has never rendered, because `StatusContext` has no `updated_at` field.
This was carried from slice-3 as an unimplemented placeholder.

The app currently has no time/date dependency: `grep` for `chrono`, `time::`,
`DateTime`, `Local::now`, `humantime` across `fermenter/src/` returns only
`tokio::time::*` usages, which are monotonic durations, not wall-clock
timestamps. Adding wall-clock formatting introduces a new dependency.

## Goals / Non-Goals

**Goals:**

- Display a server-local wall-clock timestamp on the dashboard that the
  operator can use as a freshness/liveness signal — it should advance on each
  poll even when readings stop arriving.
- Keep the change inside the existing render path: no new route, no new
  channel, no client-side JS.
- Make the timestamp deterministic in tests so `insta` snapshots stay stable.

**Non-Goals:**

- Showing the reading's ingest time (a true "last updated at <ingest
  moment>"). That is a separate concern requiring a timestamp on `Reading`
  or a side-channel from the ingest loop, and is explicitly deferred.
- Client-side clock or timezone handling. The timestamp is server-local only.
- Removing or altering the HTMX poll interval (5s stays).
- Touching the target/brew forms, `/healthz`, or any non-dashboard route.

## Decisions

### Decision 1: `chrono` with `Local` offset, `clock` feature only

Add `chrono = { version = "0.4", features = ["clock"] }` to
`[dependencies]`. Render with
`chrono::Local::now().format("%d-%b-%Y %H:%M:%S").to_string()`.

**Why `chrono` over `time`:** `chrono::Local::now()` reads `/etc/localtime`
(or `TZ`) and formats via a `%d-%b-%Y`-style strftime string in one call,
which maps directly to the required `DD-MMM-YYYY HH:MM:SS` format. The `time`
crate requires the `local-offset` feature, which has a soundness caveat
(`time` documents that reading the local offset is unsound in multi-threaded
programs unless the process is single-threaded at first call). Since this app
is multi-threaded (Axum + ingest loop on Tokio), `chrono` avoids that footgun.

**Why `Local` over `Utc`:** The requirement says "local server time." The Pi
runs with `/etc/localtime` set to the operator's timezone (or `TZ` via the
12-factor env). `Local` respects that. `Utc` would force the operator to do
mental timezone arithmetic, defeating the "at-a-glance freshness" goal.

**Why the `clock` feature only:** Disabling `chrono`'s default features and
requesting only `clock` avoids pulling `chrono::serde` and the old `time`
dependency shim, keeping the dep footprint minimal. `clock` is what provides
`Local::now()`.

**Alternative considered — `std::time::SystemTime` + manual formatting:**
Rejected. `SystemTime` gives a duration since UNIX epoch; formatting it into
`DD-MMM-YYYY` by hand means re-implementing calendar math (leap years, month
lengths, month-name table). That is exactly what `chrono` exists to avoid,
and the codebase has no existing calendar helpers to reuse.

**Alternative considered — `humantime`:** Rejected. `humantime` formats
durations (`"3h 12m ago"`), not absolute wall-clock timestamps, and does not
do `%d-%b-%Y`-style formatting at all.

### Decision 2: `server_time: String` lives on `StatusContext`, computed in `status_context()`

`StatusContext` gains a `server_time: String` field, populated inside
`status_context()` (the shared helper both `GET /` and `GET /status` already
call). Both render paths get the field for free — no handler branching, no
second code path to drift.

The field is `String`, not `DateTime<Local>`, so the template renders it
verbatim with `{{ server_time }}` and no filter. MiniJinja can format
`DateTime` only via a custom filter, which would be more machinery for no
gain; the format string is fixed by the requirement, so formatting in Rust is
simpler and keeps the template trivial.

**Why compute in `status_context()` not in a middleware/extension:** The
timestamp is render-time, per-response, and only needed by these two routes.
A middleware that injects it into every request (including `/healthz`, static
assets, form POSTs) would be over-scoped and would couple non-dashboard
routes to `chrono` for no reason.

### Decision 3: Timestamp renders outside the `{% if reading %}` block

The new `<p class="server-time">Server time: {{ server_time }}</p>` is placed
*before* the `{% if reading %}` / `{% else %}` fork in
`partials/status.html`, so it renders both before any reading has arrived and
after one has. This gives the operator a liveness signal during the
post-startup window where no reading exists yet — exactly the case where a
"server is alive" indicator is most useful (otherwise the no-data state is
indistinguishable from a crashed server).

### Decision 4: Remove the dormant `updated_at` block, don't wire it

The `{% if updated_at %}` block is deleted rather than populated. Wiring it
to `Local::now()` would conflate "last reading arrived at" semantics
(implied by the label "Last updated") with "page rendered at" semantics
(this slice's goal). Leaving the dormant block in place alongside the new
`server_time` line would repeat the slice-3 mistake (a second never-rendering
branch). Removing it is a one-line deletion and makes the template honest.

### Decision 5: Snapshots use a fixed `server_time` string

The two existing `insta` snapshot tests (`status_fragment_with_reading_snapshot`,
`status_fragment_without_reading_snapshot`) construct `StatusContext` by hand
rather than via `status_context()`. They are updated to pass
`server_time: "14-Jul-2026 14:30:45"` (a fixed, format-correct string), so
the snapshots stay deterministic and also serve as a regression guard for the
exact format. A separate unit test asserts the live `status_context()` output
matches `^\d{2}-[A-Z][a-z]{2}-\d{4} \d{2}:\d{2}:\d{2}$` so a future format
drift is caught without making the snapshot tests time-dependent.

## Risks / Trade-offs

- **[Snapshot churn]** The two status-fragment snapshots change. → Mitigation:
  regenerate with `INSTA_UPDATE=always cargo test` then `cargo insta accept`,
  and review the diff in the PR (the only change should be the added
  `Server time:` line and the removed `Last updated:` block).
- **[Timezone misconfiguration on the Pi]** If the Pi's `/etc/localtime` is
  unset or UTC, the timestamp shows UTC and the operator might assume it's
  local. → Mitigation: this is a deployment concern, not a code concern; the
  `deployment-packaging` capability already bakes the image and the operator
  sets the Pi's timezone at install. No code change here. Documented as an
  open question if a future slice wants a `TZ` default in `.env.example`.
- **[Render-time vs ingest-time confusion]** An operator might read "Server
  time: 14:30:45" as "the reading is from 14:30:45." It is actually "the
  page rendered at 14:30:45." → Mitigation: the label is "Server time:", not
  "Last updated:", to avoid the conflation. The reading's own timestamp is a
  separate, deferred feature.
- **[Clock skew across polls]** Two consecutive `/status` polls 5s apart
  will produce timestamps ~5s apart, but sub-second jitter means they won't
  be exactly `:45` and `:50`. → Not a risk; the second-level format is
  intentional and sub-second precision would be noise on a dashboard.
- **[New transitive dependency on `chrono`'s `iana-time-zone` for `Local`]**
  `chrono::Local` needs to resolve the local offset, which on Linux reads
  `/etc/localtime` directly (no `iana-time-zone` crate needed on the Pi's
  glibc target). → Verified: the `clock` feature pulls `iana-time-zone` only
  for wasm/Windows targets; on `linux/arm64` glibc, `chrono` uses
  `/etc/localtime` natively. No extra dep lands on the Pi image.

## Migration Plan

- No data migration; the change is render-only and adds no persisted field.
- Deploy: rebuild the `fermenter:arm64` image via the existing
  `scripts/build_and_ship_image.sh` pipeline; `docker compose up -d` picks up
  the new image. No Redis schema change, no firmware change.
- Rollback: revert to the previous image. The `Server time:` line disappears
  and the dormant `Last updated:` block reappears (still non-rendering). No
  state to restore.

## Open Questions

- Should `.env.example` document a `TZ` default (e.g. `TZ=Etc/UTC`) so
  operators deploying to a fresh Pi get a predictable zone? Deferred to a
  follow-up; this slice just renders whatever the OS timezone is.
- Should the reading's ingest time eventually be shown too (a true "Last
  updated at <ingest moment>")? That would require timestamping `Reading` at
  ingest, which touches the serial contract's consumer side (not the wire
  format) and the Redis schema. Tracked as a future slice, not here.
