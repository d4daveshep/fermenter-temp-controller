# Managing the Rust Rewrite with OpenSpec

> How we use OpenSpec to drive the fermenter controller rewrite (see
> `docs/rewrite-plan.md`) and accrete a durable specification library as we go.
> The rewrite is built as fast vertical slices; each slice is one OpenSpec
> change whose spec deltas fold into the capability library on archive.

## 1. Capability library (the stable layer)

Capabilities are **behavior-named** — what the system does, in brewer/operator
terms. They deliberately do **not** mirror the Rust module layout
(`rewrite-plan.md` §3) or the build milestones (§13), so they survive refactors.
After archiving, these live in `openspec/specs/`:

| Capability | Scope |
|---|---|
| `device-connection` | Serial transport contract: baud, `<float>` framing, newline JSON, reconnect/backoff |
| `temperature-monitoring` | Ingest + validate Arduino readings; expose current state (latest/min/max/ambient) |
| `temperature-control` | Target setpoint; reconcile to device via the serial write contract |
| `reading-history` | Time-series persistence (Redis TS); retention; last/range queries |
| `brew-session` | Brew-id lifecycle; runtime relabel |
| `web-dashboard` | Pages, HTMX fragments, forms, live polling |
| `system-configuration` | Env-var config; validation; fail-fast on bad config |
| `operational-health` | `/healthz`; structured logging; typed error model |
| `deployment-packaging` | Docker image; device passthrough (not privileged); compose; ARM64 |

## 2. The mental model: proposals ≠ capabilities ≠ milestones

Three distinct axes that must not be conflated:

- **Milestones** (`rewrite-plan.md` §13) = *when* (execution order).
- **Capabilities** (§1 above) = *what the system does, forever* (durable specs).
- **Proposals / slices** = *units of change* (one OpenSpec change folder each).

```
   MILESTONES (when)     PROPOSALS / SLICES (change)      CAPABILITIES (what, forever)
   ─────────────────     ──────────────────────────       ────────────────────────────
   M1+M2  ───────────►  slice-1-ingest-readings  ──────►  device-connection
                                                          temperature-monitoring
                                                          system-configuration
                                                          operational-health (logging)
   M3+M4  ───────────►  slice-2-persist-readings ──────►  reading-history
                                                          (+mod temperature-monitoring)
   M5     ───────────►  slice-3-view-dashboard   ──────►  web-dashboard
                                                          (+add operational-health /healthz)
   M3/M5  ───────────►  slice-4-set-target       ──────►  temperature-control
                                                          (+mod web-dashboard, device-connection)
   M5     ───────────►  slice-5-brew-session     ──────►  brew-session
                                                          (+mod web-dashboard, reading-history)
   M6     ───────────►  slice-6-real-hardware    ──────►  (no spec deltas; fulfills device-connection)
   M7+M8  ───────────►  slice-7-deployment       ──────►  deployment-packaging
```

Key consequences:
1. A milestone can spawn multiple proposals.
2. A proposal targets one capability primarily but may amend others.
3. **Not every slice grows the library.** slice-6 swaps mock→real serial and is
   expected to carry zero spec deltas — the moment the `device-connection`
   contract proves itself by letting the implementation change underneath it.

## 3. Slice sequence (vertical, numbered)

7 slices. `*` marks the capability getting its first `ADDED` requirements.
Resilience (reconnect/backoff) is **specified up front** in slice-1 (serial) and
slice-2 (Redis) contracts, and merely implemented later.

| Slice | Thread (demoable) | Primary `ADDED` | `MODIFIED` amendments | Milestone |
|---|---|---|---|---|
| `slice-1-ingest-readings` | read (mock) → validate → log; contracts fixed | device-connection*, temperature-monitoring*, system-configuration*, operational-health* (logging) | — | M1+M2 |
| `slice-2-persist-readings` | + store to Redis, survive restart | reading-history* | temperature-monitoring (rehydrate on start) | M3+M4 |
| `slice-3-view-dashboard` | + see current state in browser, polling | web-dashboard* | operational-health (`/healthz`) | M5 |
| `slice-4-set-target` | + change target temp from UI → device | temperature-control* | web-dashboard (form), device-connection (write `<target>`) | M3/M5 |
| `slice-5-brew-session` | + relabel brew at runtime | brew-session* | web-dashboard (form), reading-history (keyed by brew-id) | M5 |
| `slice-6-real-hardware` | + swap mock → tokio-serial | — (impl + `#[ignore]` hardware tests) | device-connection (fulfills; expect no delta) | M6 |
| `slice-7-deployment` | + Docker, passthrough, ARM64, Pi verify | deployment-packaging* | — | M7+M8 |

slice-1 over-invests in specs relative to its modest demo: it fully fixes the
serial transport contract, the `Reading` validation rules, and the config
surface — the bedrock everything else amends.

## 4. Anatomy of a slice proposal

Each slice is one OpenSpec change folder. The `specs/` subfolder holds the
*delta*; archiving promotes it into top-level `openspec/specs/`.

```
openspec/changes/slice-2-persist-readings/
├── proposal.md     # why + what + non-goals
├── design.md       # TimeStore trait, Redis TS schema, reconnect (the M4 detail)
├── tasks.md        # TDD tasks: write store_redis.rs tests FIRST, then impl
└── specs/
    ├── reading-history/spec.md           # ## ADDED Requirements
    └── temperature-monitoring/spec.md    # ## MODIFIED Requirements (rehydrate on start)
```

## 5. Accretion discipline (keeps the library coherent)

- **Greenfield early, amend later:** first proposal per capability is mostly
  `## ADDED`; later slices mix `ADDED` (new capability) with `MODIFIED`
  (amend earlier).
- **The rule:** when a slice changes how an existing capability behaves, its
  proposal carries an explicit `MODIFIED` delta against that capability's spec —
  never a silent redefinition.
- **Archive folds deltas in:** archiving each slice merges its `specs/` deltas
  into the canonical `openspec/specs/`, which becomes the living "as-built"
  behavioral truth.

```
   early slices  = mostly ADDED (greenfield)
        │
        ▼
   later slices  = ADDED (new capability) + MODIFIED (amend earlier)
        │
        ▼
   archive each slice → deltas fold into openspec/specs/
        │
        ▼
   specs/ = living as-built behavioral truth
```

## 6. Conventions

- **Change naming:** numbered `slice-N-verb-noun` for legible ordering.
- **Proposal-first per slice:** on the `rewrite/rust` branch (see
  `rewrite-plan.md` §15), write proposal + design + tasks + spec deltas, then
  TDD the code. Specs lead implementation.
- **Validate + archive loop:** `openspec validate` before coding; archive each
  slice once implemented and verified.
- **Populate `openspec/config.yaml` `context:`** with the tech stack (Rust,
  Axum, Tokio, MiniJinja, Redis TS, HTMX), the TDD convention, the 9-capability
  list, and the slice-naming rule, so every generated artifact stays aligned.

## 7. Living slice-map matrix

Kept in sync as slices land. `A` = ADDED, `M` = MODIFIED, `·` = untouched.

| Capability \ Slice | s1 | s2 | s3 | s4 | s5 | s6 | s7 |
|---|---|---|---|---|---|---|---|
| device-connection | A | · | · | M | · | (fulfills) | · |
| temperature-monitoring | A | M | · | · | · | · | · |
| temperature-control | · | · | · | A | · | · | · |
| reading-history | · | A | · | · | M | · | · |
| brew-session | · | · | · | · | A | · | · |
| web-dashboard | · | · | A | M | M | · | · |
| system-configuration | A | · | · | · | · | · | · |
| operational-health | A | · | A | · | · | · | · |
| deployment-packaging | · | · | · | · | · | · | A |

## 8. Open / deferred items

- **Requirement-wording conventions** (SHALL phrasing, scenario format): let the
  first `openspec` change establish the house style rather than over-specify now.
- **Deployment verification** (old M8): lives as tasks in `slice-7`, not a spec
  delta.
- **Resilience implementation** attaches to `slice-6` work (the requirements
  themselves are specified in slice-1/slice-2 contracts).
```
