## Context

This is the first slice of the Rust rewrite (see `docs/rewrite-plan.md` and
`docs/openspec-rewrite-management.md`). The old Python system reads
newline-delimited JSON from an Arduino over serial at 115200 baud and silently
swallows malformed data. The rewrite keeps the serial contract (fixed by
unchanged firmware) but introduces strict validation, structured logging, and
trait-abstracted boundaries for testability.

This slice deliberately over-invests in stable contracts relative to its modest
demo output (readings logged, not yet stored or displayed). The `SerialSource`
trait, the `Reading` model, and the `Config` surface are bedrock that later
slices amend rather than redefine.

## Goals / Non-Goals

**Goals:**
- A Tokio binary that ingests readings from a `SerialSource` and logs them.
- A `SerialSource` trait with a mock implementation enabling hardware-free dev
  and TDD.
- Strict `serde` deserialization of `Reading`; malformed lines logged and
  skipped, never crashing the loop.
- 12-factor `Config` loaded once at startup, failing fast on invalid input.
- A `tracing`-based structured logging baseline and a `thiserror` typed error
  model.
- The serial reconnect/backoff requirement specified now (implemented when the
  real transport lands).

**Non-Goals:**
- The real `tokio-serial` implementation (deferred to slice-6).
- Persistence, web UI, target control, brew-id lifecycle, Docker (later slices).
- Holding more than the latest reading in memory (history is slice-2).

## Decisions

**1. `SerialSource` as an async trait, line-oriented.**
`async fn read_line(&mut self) -> Result<String>` and
`async fn write_target(&mut self, f64) -> Result<()>`. The write method is part
of the contract now (firmware framing is fixed) but is unused until slice-4.
Rationale: defining the full transport contract up front means slice-4/6 add no
new trait surface. Alternative (read-only trait now, extend later) rejected — it
would churn the most-stable interface.

**2. Mock-first implementation.**
Ship only `MockSerialSource` in this slice (scripted lines, recorded writes,
injectable errors). Rationale: enables the entire TDD cycle and a demoable run
with `MOCK_SERIAL=true` and no Arduino. The real impl is a drop-in later.

**3. Strict `Reading` with lenient unknowns.**
Required numeric fields (`target`, `average`, `min`, `max`, `ambient`) and
`action` must be present; `reason-code` defaults when absent; `json-size` is
accepted but ignored. A line that fails to parse is logged at `warn` and
skipped. Rationale: fixes the old silent-swallow bug while tolerating the legacy
`json-size` field and minor firmware additions.

**4. Fail-fast config.**
Parse env into a typed `Config` once at startup; on any invalid/missing required
value, log a clear error and exit non-zero. Rationale: a misconfigured
controller should refuse to start rather than run in a degraded, surprising
state.

**5. Reconnect specified, not yet implemented.**
The `device-connection` spec states the reconnect/backoff requirement now; the
mock doesn't need it, so implementation waits for the real transport.
Rationale: keeps the contract complete and stable; avoids re-opening the spec
in slice-6.

## Risks / Trade-offs

- **[Mock-only transport may mask real-serial quirks]** → The contract is
  defined to match firmware behaviour (baud, framing, newline JSON); slice-6
  adds `#[ignore]`'d hardware tests to validate the contract held.
- **[Specifying reconnect without implementing it]** → Risk of the spec drifting
  from reality. Mitigation: slice-6 must implement to the existing requirement,
  and verification checks it.
- **[Over-engineering the first slice]** → Accepted deliberately: these three
  contracts are the highest-churn-cost interfaces, so front-loading their design
  is the cheapest place to get them right.
- **[Config crate choice (`figment` vs `envy`)]** → Low risk; both parse env to
  a struct. Defer final pick to implementation; the spec only requires the
  behaviour (typed, validated, fail-fast), not the crate.
