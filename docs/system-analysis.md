# Fermentation Temperature Controller — System Analysis

> Technical analysis of the current codebase, prepared as a reference for a
> potential ground-up rewrite. Covers technical capabilities, software
> architecture, and the deployment model. The Arduino firmware (`arduino/`)
> is intentionally excluded; only the Python services are analysed, with the
> firmware treated as an external device over a serial link.

---

## 1. Overview

The system controls and monitors the temperature of a fermentation vessel
(e.g. for home brewing). It runs on a Raspberry Pi, talks to an Arduino over
USB serial, persists time-series temperature data to InfluxDB, and exposes a
small web UI for monitoring and runtime control.

The actual control logic (heating/cooling decisions) lives in the **Arduino
firmware**, not in the Python code. The Python side is responsible for:

- Relaying readings from the Arduino into a time-series database.
- Pushing a desired target temperature down to the Arduino.
- Presenting current state and accepting operator input via a web UI.

There are three runtime components plus the database:

| Component    | Role                                              | Tech                  |
|--------------|---------------------------------------------------|-----------------------|
| `controller` | Background service: serial ↔ DB ↔ control bridge  | Python, asyncio       |
| `web`        | FastAPI web UI and HTTP API                        | FastAPI, Uvicorn      |
| `influxdb`   | Time-series storage for temperature readings       | InfluxDB 2.7 (OSS)    |
| Arduino      | Sensor reading + heating/cooling control (external)| C++ firmware (excl.)  |

---

## 2. Technical Capabilities

### 2.1 Data acquisition (controller)
- Opens an async serial connection to the Arduino at **115200 baud**
  (`serial_asyncio`).
- Reads newline-delimited **JSON records** emitted by the Arduino (~every 10s
  per the install guide).
- Normalises integer fields to floats (`convert_dict_int_values_to_float`),
  skipping the `json-size` field.
- Extracts and caches key fields in memory: `target`, `average`, `min`, `max`,
  `ambient`, `action`, `reason-code`.
- Writes every received record to InfluxDB as a `temperature` measurement,
  tagged with `brew-id`, with all JSON keys mapped to fields and a UTC
  millisecond timestamp.

### 2.2 Control output (controller)
- Every 5 seconds, compares the configured `target_temp` against the last
  target reported by the Arduino. If they differ, writes the new target to the
  serial port wrapped in framing characters: `<19.5>`.
- This is the only "control" the Python layer exerts; the Arduino owns the
  actual heating/cooling regulation.

### 2.3 Runtime reconfiguration (controller via ZeroMQ)
- Listens on a ZeroMQ **PULL** socket for JSON command messages.
- Supported commands:
  - `{"new-target-temp": <float>}` — updates the in-memory target temperature.
  - `{"new-brew-id": <string>}` — updates the brew identifier (validated as a
    string).
- Malformed JSON or invalid value types raise errors and are logged.

### 2.4 Web UI & API (web)
HTTP endpoints exposed by `web/fastapi_app.py`:

| Method & Path                  | Purpose                                                    |
|--------------------------------|------------------------------------------------------------|
| `GET /`                        | HTML dashboard: latest reading, target, action, temps      |
| `GET /debug`                   | Returns the last DB record as JSON                         |
| `GET /update-target-temp/{t}`  | Sets target temp via path param; sends ZMQ message         |
| `GET /new-target-temp`         | HTML form to enter a new target temp                       |
| `POST /post-target-temp/`      | Form submit → ZMQ message → redirect to `/`                |
| `GET /new-brew-id`             | HTML form to enter a new brew ID                           |
| `POST /post-brew-id/`          | Form submit → ZMQ message → redirect to `/`                |

- Templates are server-rendered with **Jinja2** (`root.html`,
  `new_target_temp_form.html`, `new_brew_id_form.html`).
- Static assets: a single `styles.css`.
- The dashboard reads the **last record** from InfluxDB on each load (no live
  push/websocket; refresh is manual via a button).
- Runtime control messages from the web are delivered to the controller over
  ZeroMQ, **not** by direct call — the web process binds the ZMQ PUSH socket
  and the controller connects a PULL socket.

### 2.5 Configuration
- INI file parsed by `configparser`, wrapped in `ControllerConfig`
  (`controller/config.py`).
- Sections: `[fermenter]` (target_temp, brew_id), `[influxdb]`
  (url, auth_token, org, bucket), `[arduino]` (serial_port, baud_rate),
  `[zmq]` (url), `[general]` (timezone).
- `INFLUXDB_TOKEN` environment variable overrides the config file's
  `auth_token`.
- Config filename is selected via the `config_file` env var (default
  `config.ini`), read through a Pydantic v1 `BaseSettings` class (`EnvSettings`).
- URL validation for InfluxDB and ZMQ is currently **commented out**.

### 2.6 Logging
- Standard library `logging` to stdout, formatted with level/timestamp/message,
  level `DEBUG`. Configured in `ControllerConfig.configure_logging()`.

---

## 3. Software Architecture

### 3.1 Process and data-flow model

```
                +-------------------+
                |     Arduino       |
                |  (firmware, C++)  |
                +---------+---------+
                          | USB serial @115200
                          | JSON readings  ▲  target temp "<t>"
                          ▼                │
                +-------------------+      │
                |    controller     |──────┘
                |  (asyncio loops)  |
                |  - serial reader  |
                |  - target writer  |
                |  - zmq receiver   |
                +----+---------+----+
                     |         ▲
       write points  |         | ZMQ PULL (commands)
                     ▼         |
            +-----------+   +---+--------------------+
            | InfluxDB  |   |   ZeroMQ (PUSH→PULL)   |
            |  (OSS 2.7)|   +---+--------------------+
            +-----+-----+       ▲
                  ▲             | ZMQ PUSH (commands)
       query last |             |
       record     |        +----+-----+
                  +--------|   web    |
                           | FastAPI  |
                           +----+-----+
                                ▲ HTTP :8080
                                |
                           +----+-----+
                           | Browser  |
                           +----------+
```

### 3.2 Controller internals
`controller/temp_controller.py` (`TempController`) runs three concurrent
asyncio tasks on a single event loop (`run()`):

1. `read_temps_from_serial_and_write_to_database` — read loop, busy-spinning
   with `sleep(0)`, catches all exceptions and logs them.
2. `write_target_temp_to_serial_port_if_updated` — 5s polling loop comparing
   configured vs. reported target.
3. `receive_and_process_zmq_message` — awaits ZMQ messages and dispatches them.

Notable structural points:
- The first serial line is read and discarded on startup.
- Shared state (`config.target_temp`, `config.brew_id`) is mutated by the ZMQ
  receiver and read by the writer loop — coordination is via the single event
  loop, not explicit locking.
- Errors in the read loop are swallowed (logged but never propagated), so the
  loop continues regardless of malformed data.

### 3.3 Module breakdown (`controller/`)
| Module                     | Responsibility                                          |
|----------------------------|---------------------------------------------------------|
| `temp_controller.py`       | Orchestration; asyncio tasks; serial I/O; ZMQ dispatch  |
| `config.py`                | INI parsing, env settings, logging, custom `ConfigError`|
| `temperature_database.py`  | InfluxDB client, point creation, write, last-record read|
| `zmq_receiver.py`          | ZMQ PULL socket + poller (controller side)              |
| `zmq_sender.py`            | ZMQ PUSH socket (web side)                              |

### 3.4 Web internals (`web/`)
- `fastapi_app.py` constructs module-level singletons at import time:
  `ControllerConfig`, `TemperatureDatabase`, and `ZmqSender`.
- Reuses the controller's `config.py` and `temperature_database.py` modules —
  i.e. the web service depends directly on the `controller` package.
- Timezone for display is hard-coded to `Pacific/Auckland` in addition to the
  configured timezone.

### 3.5 Inter-service communication
- **Serial**: controller ↔ Arduino, JSON in / framed float out.
- **ZeroMQ PUSH/PULL**: web (PUSH, binds) → controller (PULL, connects), over
  `tcp://127.0.0.1:5555`. One-way fire-and-forget command channel.
- **InfluxDB HTTP API**: both controller (writes) and web (reads) talk to
  InfluxDB on `:8086`.
- **HTTP**: browser → web on `:8080`.

### 3.6 Architectural observations (relevant to a rewrite)
- **Shared coupling**: `web` imports `controller` modules directly; the two
  services are not cleanly separated despite running as separate processes.
- **No live updates**: dashboard is pull-on-load with a manual refresh button.
- **Indirect control path**: web → ZMQ → controller → serial is more
  machinery than the single-host deployment strictly requires; the ZMQ hop
  exists because the controller owns the serial port.
- **Mutable shared config object** doubles as runtime state store.
- **Broad exception handling** in the serial read loop hides failures.
- **Pydantic v1** is pinned (`>=1.0,<2.0`); a rewrite would likely move to v2.
- **Two sources of truth for target/brew-id**: the config object (controller)
  and the last InfluxDB record (web reads). These can drift.
- **Hard-coded values**: display timezone, ZMQ URL defaults, InfluxDB init
  credentials in compose.

---

## 4. Deployment Model

### 4.1 Target platform
- Raspberry Pi (Pi 3B+ or newer, 64-bit), Raspberry Pi OS Lite.
- Arduino Uno connected via USB (`/dev/ttyACM0` by default).
- Docker + Docker Compose orchestrate all services.

### 4.2 Containers
Three Compose services defined in `docker-compose.yaml`:

| Service     | Image / Build              | Command                                            | Ports / Network        |
|-------------|----------------------------|----------------------------------------------------|------------------------|
| `influxdb`  | `influxdb:2.7-alpine`      | (default)                                          | `8086:8086`            |
| `controller`| `Dockerfile.controller`    | `python3 controller/temp_controller.py`            | `network_mode: host`, `privileged`, `restart: on-failure` |
| `web`       | `Dockerfile.web_apis`      | `uvicorn fastapi_app:app --host 0.0.0.0 --port 8080`| `8080:8080`, `network_mode: host` |

- `controller` runs **privileged** with **host networking** to access the USB
  serial device and reach InfluxDB/ZMQ on localhost.
- `web` also uses host networking; `depends_on: controller`.
- `controller` `depends_on: influxdb`.

### 4.3 Images (Dockerfiles)
All three Dockerfiles use `python:alpine` and `pip install` from a
`requirements.txt`:

- **`Dockerfile.controller`**: installs `controller/requirements.txt`, copies
  `controller/*.py`, and bakes `config-test.ini` in as `config.ini`.
- **`Dockerfile.web_apis`**: installs `web/requirements.txt`, then **flattens**
  paths — `fastapi_app.py`, `templates/`, and `static/` are copied to the
  container root, and the `controller` package is copied alongside. This is why
  `fastapi_app.py` resolves `static` via `Path(__file__).parent.parent`.
  Running the web app outside this container layout breaks path resolution.
- **`Dockerfile.pytests`**: installs `tests/requirements.txt`, copies controller
  code + tests, and runs `pytest -v tests/test_config_file.py`.

> Note: configuration is **baked into the image at build time** (the INI is
> COPYed in). Changing config requires rebuilding the image — there is no
> volume-mounted config. Runtime changes to target temp / brew ID are made via
> the web UI (ZMQ), which is not persisted back to the config file.

### 4.4 Configuration & secrets
- `INFLUXDB_TOKEN` is supplied via a gitignored `.env` file and passed as an
  environment variable to all services and to InfluxDB's first-boot setup.
- InfluxDB init credentials (`my-user` / `my-password`, org `daveshep.net`,
  bucket `temp-test`, 1-week retention) are hard-coded in the compose file.

### 4.5 Persistence
- InfluxDB data is bind-mounted: `$PWD/influxdb/data:/var/lib/influxdb2`.
- Default retention is **1 week**.

### 4.6 Build / run scripts
| Script                            | Purpose                                              |
|-----------------------------------|------------------------------------------------------|
| `docker-compose.yaml`             | Primary orchestration (`docker compose up -d`)       |
| `build_and_run_web_docker_image.sh` | Build & run just the web image (host network)      |
| `build_run_docker_for_testing.sh` | Build & run the pytest image (privileged, host net)  |

### 4.7 Testing
- Test suite under `tests/` (pytest, pytest-asyncio). Heavy emphasis on config
  file validation (many `test_config_file_*.ini` fixtures), plus serial, DB,
  ZMQ sender/receiver, temp controller, and web API tests.
- Several tests require real hardware/services (a serial port at
  `/dev/ttyACM0`, a running InfluxDB), so they are integration-style rather
  than pure unit tests.
- The Docker test runner only executes `tests/test_config_file.py`.

---

## 5. Summary for a Rewrite

**What the system does, distilled:**
1. Read JSON temperature records from an Arduino over serial.
2. Store them in InfluxDB (time-series, tagged by brew).
3. Push a target temperature down to the Arduino.
4. Serve a simple dashboard + forms to view state and change target/brew at
   runtime.

**Constraints any rewrite must respect:**
- Single physical host (Pi) owning one USB serial device.
- Arduino firmware contract: newline-delimited JSON in, `<float>` framed target
  out, 115200 baud.
- InfluxDB as the data store (or a deliberate replacement decision).

**Candidate improvements to consider:**
- Collapse the web/controller split or formalise their boundary (the current
  direct import coupling + ZMQ hop is awkward for a single host).
- Replace pull-on-load dashboard with live updates (SSE/WebSocket).
- Move config to runtime-mounted files/env with a single source of truth for
  target temp and brew ID, persisted across restarts.
- Re-enable URL/value validation; tighten exception handling in the read loop.
- Modernise dependencies (Pydantic v2, current FastAPI).
- Separate unit tests from hardware/integration tests.
