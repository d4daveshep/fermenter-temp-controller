## Context

`master` still contains the full Python/InfluxDB stack (`controller/`,
`web/`, `Dockerfile.controller`, `Dockerfile.web_apis`,
`Dockerfile.pytests`, `pyproject.toml`, `uv.lock`, Python `tests/`,
`influxdb/`, `influx_data/`, root `docker-compose.yaml`, plus the
long-dormant pre-split `archive/` scripts) untouched, exactly as
`rewrite-plan.md` §15 intended: "`master` stays the deployable Python
system throughout." All Rust development happened on `rewrite/rust`
(diverged from `master` at the start of the rewrite; `master` has barely
moved since — confirmed by `git log`, no commits on `master` since the
branch point). The 7 slices are fully merged into `rewrite/rust` and
archived in OpenSpec; slice-7's own verification proved the Rust stack on
the Pi with the real Arduino. `docs/rewrite-plan.md` §16 explicitly flags
Phase 3 cutover as "available as the next step whenever desired... a
deliberate, separate decision."

This is the Phase 3 cutover itself: delete the Python stack, merge
`rewrite/rust` → `master`, tag both the pre- and post-cutover states.
Because `rewrite-plan.md` §15 already specifies the git mechanics in
detail (branch names, tag names, merge-commit-not-squash), this design
focuses on sequencing, the file-deletion inventory, and the one piece of
active migration work: consolidating `fermenter/compose.yaml` to the repo
root.

## Goals / Non-Goals

**Goals:**
- Execute `rewrite-plan.md` §15 Phase 3 exactly: tag `v1-python`, merge
  `rewrite/rust → master` via merge commit, tag `v2-rust`, delete the
  branch.
- Remove every Python/InfluxDB-era file so `master` contains only the
  Rust stack, `arduino/`, `docs/`, and updated top-level docs.
- Make `compose.yaml` (repo root) the single, unambiguous way to run the
  full stack, replacing both the old root `docker-compose.yaml` and the
  slice-7-era `fermenter/compose.yaml`-coexisting-alongside-it setup.
- Keep the Raspberry Pi's already-running Rust deployment (from
  slice-7) undisturbed — this change updates the repository and
  `master`, not what's currently running on the Pi.

**Non-Goals:**
- No changes to the Rust application's runtime behavior, endpoints, or
  the `device-connection` serial contract — this is a deletion +
  git-history change, not a feature slice. (Confirmed against
  `openspec/specs/`: no capability other than `deployment-packaging`
  needs a delta.)
- No re-verification of the Rust stack on the Pi — slice-7 already did
  that, and this change doesn't touch the running deployment.
- No automation of the cutover itself (no script that does the
  delete+merge+tag in one command) — `rewrite-plan.md` §15 already
  frames this as a small number of deliberate, manually-run git commands,
  and that framing is kept as-is.
- Not reopening the deferred `redis_connected` `/healthz` gap
  (`operational-health`) or any other previously-deferred item — those
  stay out of scope exactly as they were for slice-7.

## Decisions

- **Delete-then-merge, not merge-then-delete.** The Python stack removal
  happens as a dedicated commit *on `rewrite/rust`* (per `rewrite-plan.md`
  §15 step 10) before the PR into `master`, rather than deleting the
  files directly on `master` after merging. This keeps the "big delete"
  diff reviewable as one deliberate commit inside the PR, and means the
  PR into `master` is exactly the diff that gets reviewed — no
  post-merge cleanup commit needed on `master` itself.
- **`compose.yaml` promotion, not duplication.** Rather than leaving
  `fermenter/compose.yaml` in place and adding a second root-level file,
  the file is `git mv`-ed to the repo root and its `build:` context
  updated from `.` to `fermenter/` (it now runs from the repo root, one
  level up from where it used to). This avoids two compose files
  describing the same stack post-cutover, which was only ever a
  transitional slice-7 accommodation for coexistence with the old Python
  `docker-compose.yaml` — a need that disappears the moment that file is
  deleted. Alternative considered: leave it at `fermenter/compose.yaml`
  permanently — rejected because every other top-level operational entry
  point in the repo (`INSTALL.md` instructions, `.env` conventions) is
  written assuming a repo-root compose file, matching how the old Python
  stack worked; moving it now avoids a permanent path inconsistency.
- **Tag `v1-python` on `master` before merging, `v2-rust` after.**
  Exactly per `rewrite-plan.md` §15 steps 11 and 13 — `v1-python` is the
  rollback safety net (documented requirement, not optional), `v2-rust`
  marks the new baseline for future reference. No alternative considered;
  this was already decided in the plan.
- **Merge commit, not squash, not rebase.** Per §15 step 12, preserving
  the full per-slice/per-milestone history on `master` rather than
  collapsing ~7 slices of work into one commit. This matches the
  project's stated reason for the long-lived-branch strategy in the
  first place (§15's opening rationale).
- **File deletion inventory decided by direct inspection, not solely the
  plan's illustrative list.** `rewrite-plan.md` §15 step 10 names
  `controller/`, `web/`, the three Python `Dockerfile.*`, `pyproject.toml`,
  `uv.lock`, and Python `tests/` explicitly, then says "etc." This design
  resolves that "etc." by inspecting the actual repo root: it also
  includes `influxdb/`, `influx_data/` (InfluxDB's own image/data dirs,
  not mentioned by name in the plan but unambiguously part of the
  Python/InfluxDB stack), the legacy pre-`controller`/`web`-split
  `archive/` folder (pre-dates the current split entirely, per its own
  `Pipfile`/`tempcontroller.py` contents), the two Python-stack-only
  helper scripts (`build_and_run_web_docker_image.sh`,
  `build_run_docker_for_testing.sh`), `.python-version`, `.pytest_cache/`,
  and the empty `config/`/`data/` directories (runtime bind-mount targets
  for the Python containers, empty in-tree). The untracked `src/` cache
  (`AGENTS.md`: "ignore `src/`... untracked artifact") is deliberately
  *not* part of this tracked deletion — it isn't tracked by git in the
  first place, so there's nothing to `git rm`; it's cleaned up by
  whatever already ignores/prunes it locally.
- **`docs/rewrite-plan.md` and `docs/openspec-rewrite-management.md` are
  kept, not deleted**, per §15 step 10's explicit "keep `arduino/`,
  `docs/`". They're historical record of how the rewrite was planned and
  executed; only `INSTALL.md`/`README.md` get rewritten (they're the
  living operational docs, not history).

## Risks / Trade-offs

- **[Risk] Irreversible-feeling deletion of the Python stack on
  `master`.** → Mitigated by the mandatory `v1-python` tag (pushed to
  `origin` before merge) — the entire pre-cutover state, Python stack
  included, remains checkoutable/revertable at that tag indefinitely.
- **[Risk] The repo-root `compose.yaml` move could silently break a
  path assumption baked into the Dockerfile or `.env.example` (e.g. a
  relative path that assumed `fermenter/` as the build context root).**
  → Mitigation: after the `git mv` + context-path edit, run
  `docker compose config` (validates/renders the merged config without
  starting anything) and, if Docker is available in this environment,
  `docker compose build` from the new root location before considering
  the task done.
- **[Risk] Deleting `tests/` (Python) also removes the currently-passing
  `tests/test_config_file.py` that `build_run_docker_for_testing.sh`
  exercises (per `AGENTS.md`), losing that CI-equivalent check.** →
  Accepted, not mitigated: that test suite only ever validated the
  Python `controller/config.py`, which is itself being deleted in the
  same commit — there is nothing left for it to test. The Rust stack's
  own `cargo test` (already running in `.github/workflows/rust.yml`) is
  the sole remaining automated check after cutover.
- **[Risk] Merging a long-lived branch with ~7 slices of history could
  produce merge conflicts against `master`** if `master` moved since the
  branch point. → Mitigation: per `rewrite-plan.md` §15 step 9, confirm
  via `git log master` / `git merge-base` that `master` hasn't diverged
  meaningfully (already spot-checked in this design's Context section);
  if it has moved, merge `master` into `rewrite/rust` first (as the plan
  already recommends doing periodically) so the final PR is a clean
  fast-forward-able merge.
- **[Trade-off] No automated rollback path beyond `git revert`/tag
  checkout.** Accepted per Non-Goals — this is a one-time, manually
  executed cutover, not a repeatable deployment operation needing
  tooling investment.

## Migration Plan

1. On `rewrite/rust`: `git mv fermenter/compose.yaml compose.yaml`, edit
   `build.context` from `.` to `fermenter/`, update the file's header
   comment (no longer describing coexistence), and update any relative
   `.env`/`Dockerfile` references implied by the new root location.
2. On `rewrite/rust`: delete the Python-stack file inventory (see
   Decisions) in one dedicated commit; update `INSTALL.md`/`README.md`
   for the Rust-only stack; add a closing status note to
   `docs/rewrite-plan.md` §16.
3. Validate: `docker compose config` (and `docker compose build` if
   Docker is available) from the repo root against the new
   `compose.yaml`; `cargo test`/`cargo clippy`/`cargo fmt --check` in
   `fermenter/` unaffected by the move.
4. Confirm `master` hasn't diverged (per Risks); merge `master` into
   `rewrite/rust` first if it has.
5. On `master`: `git tag -a v1-python <current master HEAD> -m "Final
   Python/InfluxDB system before Rust rewrite"`; `git push origin
   v1-python`.
6. Open PR `rewrite/rust → master`; review; merge with a **merge commit**
   (GitHub "Create a merge commit", not squash/rebase).
7. Tag the resulting `master` HEAD `v2-rust`; push the tag.
8. Delete `rewrite/rust` locally and on `origin`.
9. Archive this OpenSpec change, folding the `deployment-packaging`
   delta into `openspec/specs/`.

**Rollback:** `git checkout v1-python` (or reset a maintenance branch to
it) recovers the complete pre-cutover Python + Rust dual-stack state at
any time; the Pi's running Rust deployment is unaffected either way since
this migration never touches it.

## Open Questions

- Should `v1-python` be created *only* if it doesn't already effectively
  exist as the `origin/master` tip at cutover time, or always
  unconditionally right before merge regardless of how much time has
  passed since slice-7? This design assumes "always, right before
  merge," per §15's literal step ordering — flagged here in case the
  human operator prefers to tag it now (locking in today's exact state)
  rather than at merge time.
- Does the empty `config/`/`data/` directory deletion need any
  `.gitignore` cleanup afterward (in case something still references
  those paths for bind mounts on the Pi outside of Compose)? Not found
  in any tracked file during this design's inspection, but worth a final
  grep before the deletion commit lands.
</content>
