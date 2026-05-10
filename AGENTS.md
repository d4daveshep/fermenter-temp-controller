# OpenCode Agent Instructions

## Repository Boundaries & Source of Truth
- **Ignore the `src/` directory.** It is an untracked artifact containing only `__pycache__` and old `.egg-info` files. The actual source code lives in `controller/` (the IoT background service), `web/` (the FastAPI frontend), and `arduino/` (firmware).
- **Arduino Firmware:** The `arduino/TempController/` directory contains the C++ firmware for the Arduino. It uses the older `arduino` CLI (not `arduino-cli`) for building and uploading via `compile_arduino.sh` and `upload_arduino.sh`. Dependencies are bundled as zips in `arduino/install/` (like AUnit, ArduinoJson, DallasTemperature).
- **Ignore `tests/component/` and `tests/unit/`.** These are untracked cache directories. The active test suite is directly under `tests/`.
- **Dependencies:** The `pyproject.toml` (and `uv.lock`) is **incomplete**. It misses `fastapi`, `uvicorn`, and `pytest` dependencies. The true source of truth for dependencies are `controller/requirements.txt`, `web/requirements.txt`, and `tests/requirements.txt`.

## Architecture & Quirks
- **Pydantic v1:** `controller/config.py` uses Pydantic v1 (`from pydantic import BaseSettings`). Do NOT use Pydantic v2 conventions (`pydantic-settings`) unless explicitly migrating the codebase.
- **Docker Path Flattening:** The `Dockerfile.web_apis` copies `web/fastapi_app.py`, `web/templates/`, and `web/static/` to the *same root directory* inside the container (`/`). 
  - Because of this, `web/fastapi_app.py` resolves paths assuming it is in the project root alongside `static/` (e.g., `Path(__file__).parent.parent.absolute() / "static"`).
  - If you run tests or the web API locally (outside Docker), this path resolution will throw a `RuntimeError: Directory .../static does not exist`. Do not "fix" this path logic unless requested, as it is required for the Docker deployment.

## Execution & Testing
- The primary way to run the application is via Docker Compose: `docker-compose up` orchestrates `controller`, `web`, and `influxdb`.
- **Running tests locally:** 
  - If running tests outside Docker, you must manually install missing web/test dependencies: `uv pip install fastapi httpx pytest-asyncio uvicorn jinja2 python-multipart`.
  - Local `pytest` runs will fail on `test_web_api.py` due to the local vs. Docker path flattening mentioned above.
- **Running tests via Docker:** 
  - Use `./build_run_docker_for_testing.sh` to run tests exactly as the project intended.
  - *Note:* The Docker test script only tests `tests/test_config_file.py` and relies on `tests/requirements.txt`. It may encounter missing dependency errors (like `pydantic`) if the test environment diverges from the controller environment.
  - Some URL validation tests fail on `master` because URL validation is currently commented out in `controller/config.py`. Do not proactively fix this unless requested.