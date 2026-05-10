# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Arduino + Raspberry Pi fermentation temperature controller. The Arduino reads temperature sensors and controls heating/cooling relays; a Raspberry Pi service reads the Arduino over serial, writes data to InfluxDB, and serves a FastAPI web UI.

## Repository Boundaries

- **`src/`** — untracked artifact (`__pycache__`, old `.egg-info`). Ignore it; it is not source code.
- **`tests/component/` and `tests/unit/`** — untracked cache directories. The active test suite lives directly under `tests/`.
- **`pyproject.toml`** — incomplete: it is missing `fastapi`, `uvicorn`, and `pytest`. The authoritative dependency lists are `controller/requirements.txt`, `web/requirements.txt`, and `tests/requirements.txt`.

## Running the Application

The primary runtime is Docker Compose:

```bash
docker-compose up          # starts influxdb, controller, and web services
```

Individual Docker builds:

```bash
./build_and_run_web_docker_image.sh   # build and run the web service only
```

## Running Tests

**Preferred (via Docker, matches CI):**
```bash
./build_run_docker_for_testing.sh     # builds Dockerfile.pytests and runs pytest
```
The Docker test run only executes `tests/test_config_file.py` (see `Dockerfile.pytests`).

**Locally (outside Docker):**
```bash
uv pip install fastapi httpx pytest-asyncio uvicorn jinja2 python-multipart
python -m pytest tests/                        # all tests
python -m pytest tests/test_config_file.py     # single test file
python -m pytest tests/test_config_file.py::TestClassName::test_method  # single test
```

Local runs of `test_web_api.py` **will fail** due to the Docker path-flattening issue described below. Do not attempt to fix this unless explicitly requested.

URL validation tests fail on `master` because URL validation is intentionally commented out in `controller/config.py`. Do not proactively restore it.

## Architecture

### Python services (Raspberry Pi)

- **`controller/temp_controller.py`** — asyncio service that reads JSON from the Arduino over serial (`serial_asyncio`), writes temperature records to InfluxDB, and listens for ZMQ PULL messages to update `target_temp` or `brew_id` at runtime.
- **`controller/config.py`** — parses an INI config file (`config.ini` by default, overridden by `$config_file` env var). Uses **Pydantic v1** (`from pydantic import BaseSettings`). Do NOT use Pydantic v2 / `pydantic-settings` conventions.
- **`controller/temperature_database.py`** — InfluxDB v2 client wrapper.
- **`controller/zmq_receiver.py`** / **`controller/zmq_sender.py`** — ZMQ PUSH/PULL pair. The web API (`ZmqSender`) pushes commands; the controller (`ZmqReceiver`) pulls them. This is the IPC channel between the two services.
- **`web/fastapi_app.py`** — FastAPI app with Jinja2 templates. Displays live temperature data and provides forms to update `target_temp` and `brew_id`.

### Docker path flattening (web service)

`Dockerfile.web_apis` copies `web/fastapi_app.py`, `web/templates/`, and `web/static/` all into the container root. As a result, `fastapi_app.py` resolves the `static/` directory via `Path(__file__).parent.parent.absolute() / "static"` — which works inside the container but raises `RuntimeError: Directory .../static does not exist` when run locally. Do not change this path logic.

### Arduino firmware (`arduino/TempController/`)

C++ sketch targeting an Arduino Mega/Uno with:
- **`TempController.ino`** — main sketch; toggles between unit-test mode (`#define _DO_UNIT_TESTING`) and production mode.
- **Serial protocol** — Arduino emits JSON every 10 s (`instant`, `average`, `min`, `max`, `target`, `ambient`, `action`, `reason-code`). The Pi writes a float wrapped in `<` `>` markers (e.g. `<20.5>`) to update the target temperature.
- **Libraries** bundled as zips in `arduino/install/`: AUnit, ArduinoJson, DallasTemperature, OneWire.

**Build / upload Arduino firmware** (uses legacy `arduino` CLI, not `arduino-cli`):
```bash
cd arduino/TempController
./compile_arduino.sh    # arduino --verify TempController.ino
./upload_arduino.sh
```

### Config file format

Required INI sections and keys for `ControllerConfig`:

```ini
[general]
timezone = Pacific/Auckland   # optional, defaults to Pacific/Auckland

[fermenter]
target_temp = 20.0
brew_id = my-brew

[influxdb]
url = http://localhost:8086
auth_token = <token>
org = <org>
bucket = <bucket>

[arduino]
serial_port = /dev/ttyACM0

[zmq]
url = tcp://127.0.0.1:5555
```
