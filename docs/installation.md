# Installation on new Raspberry Pi #
* download and flash Raspberry Pi OS to SDCard (>16GB)
  * choose **64-bit lite** version (no GUI) 
* boot and config RPi
  * enable SSH, serial port
  * fix wireless IP (/etc/network/interfaces)
* apply updates (sudo apt-get update, upgrade)
* install tools like aptitude, git
* install Arduino IDE v1.8+ from web e.g.
  * https://downloads.arduino.cc/arduino-1.8.19-linuxaarch64.tar.xzls
* install required Arduino libraries
  * OneWire, DallasTemperature, AUnit, ArduinoJSON (not beta)
  * these need unzipping under `~/Arduino/libraries`
* git clone fermenter-temp-controller repo
* test arduino compile
* test arduino upload (with Arduino plugged in)
* install Docker using convenience script
  * https://docs.docker.com/engine/install/debian/#install-using-the-convenience-script
* do post install steps
  * https://docs.docker.com/engine/install/linux-postinstall/
* test `docker run hello-world`
* in temp controller repo
  * do influxdb initial config script
  * test from different device on port 8086
  * stop influxdb container
  * run `docker compose build`
  * run `docker compose up -d`
  * check `docker compose logs -f`
