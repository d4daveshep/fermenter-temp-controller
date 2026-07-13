## 1. Pre-flight checks

- [ ] 1.1 Confirm current branch is `rewrite/rust` and working tree is
      clean (`git status`).
- [ ] 1.2 Confirm `master` hasn't diverged meaningfully since the branch
      point (`git log master`, `git merge-base rewrite/rust master`); if it
      has moved (e.g. `arduino/` or doc fixes), merge `master` into
      `rewrite/rust` first so the eventual PR is a clean merge.
- [ ] 1.3 Do a final grep across tracked files for references to
      `config/`, `data/`, `docker-compose.yaml`, `INFLUXDB_TOKEN`, or
      `fermenter/compose.yaml` outside the files already identified for
      deletion/update, to catch anything design.md's inventory missed.

## 2. Promote `compose.yaml` to the repo root

- [ ] 2.1 `git mv fermenter/compose.yaml compose.yaml`.
- [ ] 2.2 Update the moved file's `build:` context from `.` to
      `fermenter/` (and any other relative paths that assumed the file
      lived inside `fermenter/`).
- [ ] 2.3 Rewrite the file's header comment: remove the
      slice-7-era "coexisting alongside the old Python stack" language;
      describe it as the sole repo-root orchestration entry point.
- [ ] 2.4 Update `fermenter/.env.example` references (or move/rename as
      needed) so the root `compose.yaml`'s `env_file:` points at the
      right location relative to the new root position.
- [ ] 2.5 Validate: `docker compose config` from the repo root against
      the new `compose.yaml` renders without error; run
      `docker compose build` too if Docker is available in this
      environment.

## 3. Delete the Python/InfluxDB stack

- [ ] 3.1 `git rm -r controller/ web/`.
- [ ] 3.2 `git rm Dockerfile.controller Dockerfile.web_apis
      Dockerfile.pytests`.
- [ ] 3.3 `git rm pyproject.toml uv.lock .python-version`.
- [ ] 3.4 `git rm -r tests/` (Python test suite) and remove
      `.pytest_cache/` from the working tree (add to cleanup if not
      already gitignored).
- [ ] 3.5 `git rm -r influxdb/ influx_data/`.
- [ ] 3.6 `git rm -r archive/` (legacy pre-`controller`/`web`-split
      scripts).
- [ ] 3.7 `git rm build_and_run_web_docker_image.sh
      build_run_docker_for_testing.sh`.
- [ ] 3.8 `git rm docker-compose.yaml` (old root compose file, superseded
      by task group 2's new root `compose.yaml`).
- [ ] 3.9 `git rm -r config/ data/` (empty Python-container bind-mount
      directories).
- [ ] 3.10 Confirm the untracked `src/` cache is left alone (not part of
      this tracked deletion, per `AGENTS.md`).
- [ ] 3.11 Confirm `.venv/` removal/handling is consistent with
      `.gitignore` (it should already be untracked; verify no
      accidental tracked files remain under it).

## 4. Update docs for Rust-only deployment

- [ ] 4.1 Rewrite `INSTALL.md` to describe only the Rust/Redis Compose
      deployment path (drop Python/InfluxDB setup steps, `uv`/`pip`
      install steps, `INFLUXDB_TOKEN` env var, etc.); reference the new
      repo-root `compose.yaml` and `fermenter/.env.example`.
- [ ] 4.2 Rewrite `README.md` to reflect the Rust stack as the only
      system.
- [ ] 4.3 Add a closing status note to `docs/rewrite-plan.md` §16
      recording that the Phase 3 cutover is complete (date, PR/merge
      commit reference once known).
- [ ] 4.4 Leave `docs/rewrite-plan.md` and
      `docs/openspec-rewrite-management.md` otherwise intact as historical
      record (per design.md decision — do not delete or rewrite their
      substance).

## 5. Verify the Rust stack still builds/tests clean after the move

- [ ] 5.1 `cargo fmt --check`, `cargo clippy --all-targets -- -D
      warnings`, `cargo test` inside `fermenter/` — confirm unaffected by
      the repo-root file moves/deletions.
- [ ] 5.2 Confirm `.github/workflows/rust.yml` jobs still resolve their
      paths correctly (they should be untouched, but verify no job
      referenced anything now-deleted).

## 6. Commit and open the cutover PR

- [ ] 6.1 Commit task groups 2-4 together as one dedicated "cutover:
      remove Python/InfluxDB stack, promote compose.yaml to repo root"
      commit on `rewrite/rust` (per design.md's delete-then-merge
      decision).
- [ ] 6.2 Push `rewrite/rust`.
- [ ] 6.3 On `master`: tag the current tip `v1-python` (`git tag -a
      v1-python <sha> -m "Final Python/InfluxDB system before Rust
      rewrite"`); push the tag (`git push origin v1-python`).
- [ ] 6.4 Open a PR `rewrite/rust → master`.
- [ ] 6.5 Review the full diff (not just the latest commit) — confirm it
      matches the intended deletions/moves and nothing unrelated slipped
      in.

## 7. Merge and finalize

- [ ] 7.1 Merge the PR using a **merge commit** (not squash, not rebase).
- [ ] 7.2 Tag the resulting `master` HEAD `v2-rust`; push the tag.
- [ ] 7.3 Delete the `rewrite/rust` branch, locally and on `origin`.
- [ ] 7.4 Confirm `master` now builds/tests clean standalone (fresh
      clone or `git clean`, then repeat task 5.1's checks) and that the
      repo-root `compose.yaml` is the only compose file present.

## 8. Spec & change housekeeping

- [ ] 8.1 `openspec validate cutover-to-rust` passes.
- [ ] 8.2 Archive this change once tasks 1-7 are complete and verified,
      folding the `deployment-packaging` MODIFIED delta into
      `openspec/specs/`.
</content>
