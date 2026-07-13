## Why

All 7 planned vertical slices are implemented, archived, and folded into the
capability library (`openspec/specs/`), and slice-7-deployment's own
verification proved the Rust stack end-to-end on the Raspberry Pi against
the real production Arduino (readings ingesting, dashboard updating,
target/brew round-trips working, `cargo test -- --ignored` passing on the
Pi). Per `docs/rewrite-plan.md` §16's status note, the only remaining step
is the §15 Phase 3 cutover itself — removing the old Python/InfluxDB stack,
merging `rewrite/rust` into `master`, and making the Rust stack the sole
deployable system — which that plan explicitly calls "a deliberate, separate
decision, not performed as part of [slice-7]." This proposal is that
decision: with the Rust stack proven, continuing to carry the obsolete
Python stack only adds drift risk and confusion about which system is
authoritative.

## What Changes

- **BREAKING**: Delete the entire Python/InfluxDB stack: `controller/`,
  `web/`, `Dockerfile.controller`, `Dockerfile.web_apis`,
  `Dockerfile.pytests`, `pyproject.toml`, `uv.lock`, `.python-version`,
  Python `tests/` (and `.pytest_cache/`), `influxdb/`, `influx_data/`,
  `build_and_run_web_docker_image.sh`, `build_run_docker_for_testing.sh`,
  the empty `config/`/`data/` runtime-mount directories, and the legacy
  `archive/` folder of pre-`controller`/`web`-split scripts. The untracked
  `src/` cache noted in `AGENTS.md` is left for normal `.gitignore`
  cleanup, not a tracked deletion.
- Promote `fermenter/compose.yaml` to the repo-root `compose.yaml`,
  replacing the old root `docker-compose.yaml` (which is deleted), so the
  2-container stack (`fermenter` + `redis:8`) becomes the one and only
  orchestration entry point at the repo root instead of living
  side-by-side with the Python stack's compose file. Update the file's
  header comment (currently describing coexistence with the old stack) to
  reflect that it is now the sole stack.
- Update `INSTALL.md`/`README.md` to describe only the Rust/Redis
  deployment path (drop Python/InfluxDB setup steps, `.env`
  `INFLUXDB_TOKEN`, etc.).
- Tag the last Python `master` commit as `v1-python` before merging, per
  `rewrite-plan.md` §15 step 11, as the rollback safety net.
- Open and merge a PR `rewrite/rust → master` using a **merge commit** (not
  squash), preserving the full per-milestone/per-slice commit history on
  `master`.
- Tag the new post-merge `master` baseline `v2-rust`.
- Delete the `rewrite/rust` branch (local and remote) after the merge.
- No Arduino firmware changes, and no behavioral changes to the Rust
  application itself — this is a repository/deployment cutover, not a
  feature slice.

## Capabilities

### New Capabilities

(none — no new runtime behavior is introduced)

### Modified Capabilities

- `deployment-packaging`: the canonical Compose orchestration moves from
  `fermenter/compose.yaml` (deliberately coexisting alongside the old
  `docker-compose.yaml` since slice-7) to the repo-root `compose.yaml`,
  becoming the sole deployment mechanism for the project now that the
  Python/InfluxDB stack it used to coexist with is deleted.

## Impact

- **Code removed:** `controller/`, `web/`, `Dockerfile.controller`,
  `Dockerfile.web_apis`, `Dockerfile.pytests`, `pyproject.toml`, `uv.lock`,
  Python `tests/`, `influxdb/`, `influx_data/`, `archive/`,
  `build_and_run_web_docker_image.sh`, `build_run_docker_for_testing.sh`,
  root `docker-compose.yaml`, empty `config/`/`data/` directories.
- **Code moved:** `fermenter/compose.yaml` → repo-root `compose.yaml`
  (`build: context:` updated from `.` to `fermenter/`; `fermenter/.env`
  usage updated accordingly).
- **Docs:** `INSTALL.md`, `README.md` rewritten for the Rust-only stack;
  `docs/rewrite-plan.md` §16 gets a closing status note that cutover is
  complete.
- **Git:** new tags `v1-python` (pre-cutover `master`) and `v2-rust`
  (post-cutover `master`); `rewrite/rust` branch deleted after merge.
- **CI:** existing `.github/workflows/rust.yml` jobs (`fmt`, `clippy`,
  `test`, `docker-build`) are unaffected — they already only touch
  `fermenter/`. No Python CI existed to remove.
- **Deployment:** the Raspberry Pi already runs the Rust stack
  (slice-7's verification); this change only removes the now-dormant
  Python stack from the repository and from `master`, it does not change
  what's running on the Pi.
</content>
