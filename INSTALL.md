# Installation Guide

Step-by-step setup for a fresh Raspberry Pi running the fermentation temperature controller stack (Rust app + Redis).

## Hardware required

- Raspberry Pi (any model with USB and a 64-bit CPU — Pi 3B+ or newer recommended)
- MicroSD card (16 GB or larger)
- Arduino Uno connected via USB
- DS18B20 temperature sensors and relay board wired to the Arduino

---

## 1. Flash and configure the Raspberry Pi

1. Download and flash **Raspberry Pi OS Lite (64-bit)** to the SD card using Raspberry Pi Imager.
   - In the imager's advanced options, pre-configure your hostname, SSH key, Wi-Fi credentials, and locale before flashing.

2. Boot the Pi and SSH in, then run:
   ```bash
   sudo apt update && sudo apt upgrade -y
   sudo apt install -y git
   ```

3. Enable the serial port:
   ```bash
   sudo raspi-config
   ```
   Navigate to **Interface Options → Serial Port**. Disable the login shell over serial, but **enable the serial port hardware**.

4. Set a fixed IP address if needed (edit `/etc/network/interfaces` or use your router's DHCP reservation).

---

## 2. Install Docker

Use the official convenience script:

```bash
curl -fsSL https://get.docker.com | sh
```

Then add your user to the `docker` group so you can run Docker without `sudo`:

```bash
sudo usermod -aG docker $USER
newgrp docker
```

Verify with:

```bash
docker run hello-world
```

---

## 3. Clone the repository

```bash
git clone <repo-url>
cd fermenter-temp-controller
```

---

## 4. Configure the application

The Rust app is configured entirely via environment variables (12-factor style), read by the `fermenter` container from `fermenter/.env`.

```bash
cp fermenter/.env.example fermenter/.env
```

Edit `fermenter/.env` for your setup:

```bash
SERIAL_PORT=/dev/ttyACM0     # confirm this after plugging in the Arduino (see step 5)
SERIAL_BAUD=115200           # matches the fixed Arduino firmware contract, leave as-is
MOCK_SERIAL=false            # real deployment: leave false
REDIS_URL=redis://redis:6379 # `redis` is the Compose service name, not localhost
TS_RETENTION_DAYS=7
DEFAULT_BREW_ID=my-first-brew
HTTP_PORT=8080
DEFAULT_TARGET_TEMP=20.0     # starting target temperature in °C, only used if no persisted state exists yet
RUST_LOG=info
```

`fermenter/.env` is gitignored and never leaves this machine. After editing it you must restart the `fermenter` container for changes to take effect (step 6) — no rebuild needed, since it's plain environment configuration.

---

## 5. Set up the Arduino firmware

### 5a. Install the Arduino IDE

The bundled IDE installer is included in the repo:

```bash
cd arduino/install
tar -xf arduino-1.8.19-linuxaarch64.tar.tar
sudo mv arduino-1.8.19 /opt/arduino
sudo /opt/arduino/install.sh
```

Add the `arduino` binary to your PATH:

```bash
echo 'export PATH=$PATH:/opt/arduino' >> ~/.bashrc
source ~/.bashrc
```

### 5b. Install required libraries

Unzip each bundled library into `~/Arduino/libraries`:

```bash
mkdir -p ~/Arduino/libraries
cd /path/to/fermenter-temp-controller/arduino/install
for zip in AUnit-*.zip ArduinoJson-*.zip DallasTemperature-*.zip OneWire-*.zip; do
    unzip -q "$zip" -d ~/Arduino/libraries/
done
```

### 5c. Identify the Arduino serial port

Plug the Arduino into the Pi via USB, then run:

```bash
ls /dev/ttyACM*
```

It will typically be `/dev/ttyACM0`. If it differs, update `SERIAL_PORT` in `fermenter/.env` accordingly (see step 4).

### 5d. Compile and upload the firmware

```bash
cd fermenter-temp-controller/arduino/TempController
./compile_arduino.sh   # verify it compiles cleanly
./upload_arduino.sh    # upload to the connected Arduino
```

Confirm the Arduino is running by checking its serial output:

```bash
cat /dev/ttyACM0
```

You should see JSON lines appear every 10 seconds.

---

## 6. Build and start the application

From the project root:

```bash
docker compose build
docker compose up -d
```

Check that both services (`fermenter` and `redis`) started cleanly:

```bash
docker compose logs -f
```

---

## 7. Verify everything is working

| Check | How |
|---|---|
| App reading Arduino | `docker compose logs fermenter` — look for `serial port opened` / reading-ingest log lines |
| Data writing to Redis | `docker compose logs fermenter` — no write-error/warn lines |
| Web dashboard | Open `http://<pi-ip>:8080` in a browser |
| Health check | `curl http://<pi-ip>:8080/healthz` — reports `serial_connected` |

---

## Ongoing operations

```bash
docker compose stop          # stop all services
docker compose start         # start again without rebuilding
docker compose up -d --build # rebuild images after code changes and restart
docker compose logs -f       # tail all logs
docker compose logs fermenter -f  # tail the app only
```

To change the target temperature or brew ID at runtime, use the web dashboard at `http://<pi-ip>:8080` rather than editing `fermenter/.env` — changes made through the dashboard take effect immediately without a restart.
</content>
