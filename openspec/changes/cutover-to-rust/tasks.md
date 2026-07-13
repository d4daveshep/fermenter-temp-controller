## 1. Pre-flight checks

- [x] 1.1 Confirm current branch is `rewrite/rust` and working tree is
      clean (`git status`).
- [x] 1.2 Confirm `master` hasn't diverged meaningfully since the branch
      point (`git log master`, `git merge-base rewrite/rust master`); if it
      has moved (e.g. `arduino/` or doc fixes), merge `master` into
      `rewrite/rust` first so the eventual PR is a clean merge.
      **Confirmed:** `master` is 0 commits ahead of the merge-base;
      `rewrite/rust` is 28 commits ahead. No merge needed.
- [x] 1.3 Do a final grep across tracked files for references to
      `config/`, `data/`, `docker-compose.yaml`, `INFLUXDB_TOKEN`, or
      `fermenter/compose.yaml` outside the files already identified for
      deletion/update, to catch anything design.md's inventory missed.
      **Found additional items** not in design.md's original inventory:
      root `.env.example` (Python-only `INFLUXDB_TOKEN`), a stale
      `controller/influxdb/` line in `.gitignore`, and `AGENTS.md`/
      `CLAUDE.md` (both describe the Python stack as current). Added as
      tasks 3.12, 3.13, 4.5 below.

## 2. Promote `compose.yaml` to the repo root

- [x] 2.1 `git mv fermenter/compose.yaml compose.yaml`.
- [x] 2.2 Update the moved file's `build:` context from `.` to
      `fermenter/` (and any other relative paths that assumed the file
      lived inside `fermenter/`).
- [x] 2.3 Rewrite the file's header comment: remove the
      slice-7-era "coexisting alongside the old Python stack" language;
      describe it as the sole repo-root orchestration entry point.
- [x] 2.4 Update `fermenter/.env.example` references (or move/rename as
      needed) so the root `compose.yaml`'s `env_file:` points at the
      right location relative to the new root position.
      **Updated:** `env_file: fermenter/.env`; `fermenter/.env.example`
      header comment simplified to drop the old-stack-coexistence
      wording.
- [x] 2.5 Validate: `docker compose config` from the repo root against
      the new `compose.yaml` renders without error; run
      `docker compose build` too if Docker is available in this
      environment.
      **Verified:** `docker compose config` (with a temporary
      `fermenter/.env` copied from the example) renders cleanly, with
      `build.context` correctly resolving to `fermenter/`. (Skipped a full
      `docker compose build` — cross-compiling isn't needed to validate
      the config; the `docker-build` CI job already covers the actual
      image build, unaffected by this file move.)

## 3. Delete the Python/InfluxDB stack

- [x] 3.1 `git rm -r controller/ web/`.
- [x] 3.2 `git rm Dockerfile.controller Dockerfile.web_apis
      Dockerfile.pytests`.
- [x] 3.3 `git rm pyproject.toml uv.lock .python-version`.
      **Note:** `uv.lock` was untracked (gitignored) despite existing on
      disk — removed directly with `rm`, not `git rm`.
- [x] 3.4 `git rm -r tests/` (Python test suite) and remove
      `.pytest_cache/` from the working tree (add to cleanup if not
      already gitignored).
- [x] 3.5 `git rm -r influxdb/ influx_data/`.
      **Note:** `influx_data/` was untracked runtime data (`config`/`data`
      subdirs, no tracked files) — removed directly with `rm -rf`, not
      `git rm`.
- [x] 3.6 `git rm -r archive/` (legacy pre-`controller`/`web`-split
      scripts).
- [x] 3.7 `git rm build_and_run_web_docker_image.sh
      build_run_docker_for_testing.sh`.
- [x] 3.8 `git rm docker-compose.yaml` (old root compose file, superseded
      by task group 2's new root `compose.yaml`).
- [x] 3.9 `git rm -r config/ data/` (empty Python-container bind-mount
      directories).
      **Note:** both were empty and untracked (git never tracks empty
      dirs) — removed with `rmdir`, not `git rm`.
- [x] 3.10 Confirm the untracked `src/` cache is left alone (not part of
      this tracked deletion, per `AGENTS.md`).
      **Confirmed:** `git ls-files src/` returns 0 tracked files; left
      untouched.
- [x] 3.11 Confirm `.venv/` removal/handling is consistent with
      `.gitignore` (it should already be untracked; verify no
      accidental tracked files remain under it).
      **Confirmed** 0 tracked files under `.venv/`; removed the directory
      from disk as a dead Python artifact.
- [x] 3.12 (found via task 1.3 grep) `git rm .env.example` at the repo
      root — it only ever documented the Python stack's `INFLUXDB_TOKEN`;
      the Rust stack's env vars are documented in `fermenter/.env.example`
      (which moves with `compose.yaml`, see task group 2).
- [x] 3.13 (found via task 1.3 grep) Remove the stale
      `controller/influxdb/` line from `.gitignore` (references a
      directory deleted in this task group); leave other still-relevant
      ignore patterns as-is.
      **Note:** removing this ignore line unmasked a pre-existing,
      root-owned, empty leftover directory pair
      (`controller/influxdb/{config,data}`, stale Docker bind-mount
      artifacts predating this change) that couldn't be `rm`-ed without
      `sudo`. Left in place — git never tracked them (empty dirs) so this
      doesn't affect the commit; flagged for the repo owner to
      `sudo rm -rf controller/` manually at their convenience.

## 4. Update docs for Rust-only deployment

- [x] 4.1 Rewrite `INSTALL.md` to describe only the Rust/Redis Compose
      deployment path (drop Python/InfluxDB setup steps, `uv`/`pip`
      install steps, `INFLUXDB_TOKEN` env var, etc.); reference the new
      repo-root `compose.yaml` and `fermenter/.env.example`.
- [x] 4.2 Rewrite `README.md` to reflect the Rust stack as the only
      system.
- [x] 4.3 Add a closing status note to `docs/rewrite-plan.md` §16
      recording that the Phase 3 cutover is complete (date, PR/merge
      commit reference once known).
      **Note:** phrased to describe only what's true at this commit
      (Python stack deleted, compose.yaml promoted on this branch) since
      the merge/tag/branch-delete steps (task groups 6-7) haven't
      happened yet at this point in the sequence — avoids asserting
      completion of steps not yet taken.
- [x] 4.4 Leave `docs/rewrite-plan.md` and
      `docs/openspec-rewrite-management.md` otherwise intact as historical
      record (per design.md decision — do not delete or rewrite their
      substance). Leave `docs/system-analysis.md` intact too, for the same
      reason — it documents the pre-rewrite Python system as history, not
      current operational guidance.
- [x] 4.5 (found via task 1.3 grep) Rewrite `AGENTS.md` and `CLAUDE.md`:
      both currently describe `controller/`/`web/` as "the actual source
      code" and document Python-specific quirks (Pydantic v1, Docker path
      flattening, `docker-compose.yaml`/`build_run_docker_for_testing.sh`)
      as live guidance for agents working in this repo. Update both to
      describe the Rust stack (`fermenter/`) as the source of truth,
      dropping the now-obsolete Python sections.

## 5. Verify the Rust stack still builds/tests clean after the move

- [x] 5.1 `cargo fmt --check`, `cargo clippy --all-targets -- -D
      warnings`, `cargo test` inside `fermenter/` — confirm unaffected by
      the repo-root file moves/deletions.
      **Verified:** `cargo fmt --check` clean, `cargo clippy --all-targets
      -- -D warnings` clean, `cargo test` — 87 lib tests + 6
      `testcontainers`-backed Redis integration tests passed, 3 hardware
      tests correctly `ignored`.
- [x] 5.2 Confirm `.github/workflows/rust.yml` jobs still resolve their
      paths correctly (they should be untouched, but verify no job
      referenced anything now-deleted).
      **Confirmed:** all four jobs use `working-directory: fermenter` /
      `context: fermenter` — entirely self-contained within `fermenter/`,
      unaffected by any repo-root deletions or the `compose.yaml` move.

## 6. Commit and open the cutover PR

- [x] 6.1 Commit task groups 2-4 together as one dedicated "cutover:
      remove Python/InfluxDB stack, promote compose.yaml to repo root"
      commit on `rewrite/rust` (per design.md's delete-then-merge
      decision).
      **Committed:** `043bed0`.
- [x] 6.2 Push `rewrite/rust`.
- [x] 6.3 On `master`: tag the current tip `v1-python` (`git tag -a
      v1-python <sha> -m "Final Python/InfluxDB system before Rust
      rewrite"`); push the tag (`git push origin v1-python`).
      **Done:** tagged `4bc862a` (confirmed matching `origin/master`
      before tagging) and pushed.
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
