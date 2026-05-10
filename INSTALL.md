# Installation Guide

Step-by-step setup for a fresh Raspberry Pi running the full fermentation temperature controller stack.

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

### 4a. Set the InfluxDB token

Generate a secure random token and save it to `.env`:

```bash
echo "INFLUXDB_TOKEN=$(openssl rand -hex 32)" > .env
```

This token is used by both InfluxDB (on first boot) and the controller/web services. It is gitignored and never leaves this machine.

### 4b. Edit the application config

Open `controller/config-test.ini` and update the values for your setup:

```ini
[fermenter]
target_temp = 20.0        # starting target temperature in °C
brew_id = my-first-brew   # identifier recorded in the database

[influxdb]
url = http://localhost:8086
org = daveshep.net
bucket = temp-test

[arduino]
serial_port = /dev/ttyACM0   # confirm this after plugging in the Arduino (see step 5)

[general]
timezone = Pacific/Auckland   # IANA timezone name
```

Leave `auth_token` as-is — the token comes from the `INFLUXDB_TOKEN` env var set in step 4a.

After editing the config you must rebuild the Docker images for the change to take effect (step 6).

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

It will typically be `/dev/ttyACM0`. If it differs, update `serial_port` in `controller/config-test.ini` accordingly.

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

On first boot, InfluxDB automatically initialises itself using the token from `.env` — no manual database setup required.

Check that all three services started cleanly:

```bash
docker compose logs -f
```

---

## 7. Verify everything is working

| Check | How |
|---|---|
| Controller reading Arduino | `docker compose logs controller` — look for `read ... from serial` lines |
| Data writing to InfluxDB | `docker compose logs controller` — look for `Writing point to database` lines |
| Web UI | Open `http://<pi-ip>:8080` in a browser |
| InfluxDB UI | Open `http://<pi-ip>:8086` — log in with username `my-user`, password `my-password` |

---

## Ongoing operations

```bash
docker compose stop          # stop all services
docker compose start         # start again without rebuilding
docker compose up -d --build # rebuild images after config changes and restart
docker compose logs -f       # tail all logs
docker compose logs controller -f  # tail controller only
```

To change the target temperature or brew ID at runtime, use the web UI at `http://<pi-ip>:8080` rather than editing the config file — changes made through the UI take effect immediately without a restart.
