# simple program to read from Arduino serial port

import argparse
import configparser
import glob
import json
import logging
import os
from datetime import datetime
from pathlib import Path

import serial
import time
from influxdb import InfluxDBClient
from pytz import timezone
from serial import SerialException


def open_influxdb(dbname):
    client = InfluxDBClient("localhost", 8086)
    logging.debug("Database client opened")

    db_list = client.get_list_database()

    check_name = {"name": dbname}
    if check_name in db_list:
        client.switch_database(dbname)
        logging.info("Using database " + dbname)
    else:
        client.create_database(dbname)
        logging.info("Created database " + dbname)
        client.create_retention_policy("my_policy", "4w", "1", dbname, default=True)

    return client


def main(config_file):
    # get NZ timezone so we can localise the timestamps and deal with NZST/NZDT
    nztz = timezone("NZ")

    # open the serial port
    serial_port = get_serial_port()

    # read the  config file
    config = configparser.ConfigParser()
    config.read(config_file)

    # get the brew ID
    if config["fermenter"]["brewID"]:
        brew_id = config["fermenter"]["brewID"]
    else:
        brew_id = "no_brew_id"
    logging.info("Brew ID is: " + brew_id)

    # get the new target temp
    new_target = config["fermenter"]["TargetTemp"]
    logging.info("Target temperature being set to %s", str(new_target))
    new_target_str = '<' + str(new_target) + '>'

    # write target temp to serial port
    try:
        serial_port.write(new_target_str.encode())
    except SerialException:
        logging.warning("Couldn't write target temp to serial port")

    # open the influxdb database
    influxdb_client = open_influxdb(brew_id)

    # build dictionary for influxdb
    influxdb_data = {"measurement": "temperature", "tags": {"brew_id": brew_id}}

    # infinite loop to read data from serial port
    while True:
        logging.debug("--------------------")
        line = serial_port.readline()  # read serial line as bytes

        try:
            # convert serial line to string and load to JSON sequence
            fermenter_data = json.loads(line.decode("utf-8"))
            logging.debug("Fermemter data is: %s", fermenter_data)

            # get formatted localised timestamp
            stamp = nztz.localize(datetime.now()).isoformat()
            logging.debug("timestamp is %s", stamp)

            # populate the influxdb_data dict with relevant data from fermenter
            influxdb_data["time"] = stamp
            influxdb_data["fields"] = {"ambient_temp": fermenter_data["ambient"],
                                       "fermemter_temp": fermenter_data["avg"],
                                       "target_temp": fermenter_data["target"],
                                       "controller_action": str(fermenter_data["action"]).upper()}
            if "change" in fermenter_data.keys():
                influxdb_data["fields"]["change_action"] = str(fermenter_data["change"]).upper()

            target = fermenter_data["target"]

            # check if we need to update the target temp
            if round(float(target), 1) != round(float(new_target), 1):
                new_target_str = '<' + str(new_target) + '>'
                serial_port.write(new_target_str.encode())
                influxdb_data["fields"]["target_temp"] = new_target
                logging.info("Updated target temp to %s", str(new_target))

            logging.debug("Writing InfluxDB json: %s", json.dumps(influxdb_data))

            # write data to database as json
            influxdb_client.write_points(json.dumps(influxdb_data), database=brew_id)

        except json.decoder.JSONDecodeError as err:
            logging.debug(err)
        finally:
            influxdb_client.close()
            logging.debug("Database closed")


def get_serial_port():
    # find the serial port
    ttylist = glob.glob("/dev/ttyACM*")
    if len(ttylist) == 0:
        msg = "No Arduino devices found at /dev/ttyACM*"
        logging.critical(msg)
        raise SystemExit("*** ERROR *** " + msg)

    if len(ttylist) > 1:
        msg = "WARNING: Multiple Arduino devices found at /dev/ttyACM*, using first device"
        logging.warning(msg)

    tty = ttylist[0]
    logging.info("Using Arduino device at %s", tty)

    # open the serial port
    try:
        port = serial.Serial(tty, 9600)
    except SerialException:
        logging.error("Couldn't open serial port at %s", tty)
        raise SystemExit("*** ERROR *** Couldn't open serial port")

    # sleep for 30 secs to allow arduino to reboot after serial port open
    logging.debug("Sleeping for 30 sec to allow arduino to reboot after serial port was opened")
    time.sleep(30)

    return port


def parse_args():
    # set up the command line parser
    parser = argparse.ArgumentParser(description="Fermentation Temp Controller")

    # define the argument to set the config file
    parser.add_argument("config_file", help="specify the full location of the config file")

    # parse the arguments and check the file exists
    args = parser.parse_args()

    if args.config_file:
        config_file = args.config_file
        logging.info("Using config file '%s'", config_file)

        if not Path(config_file).exists():
            logging.critical("Config file '%s' not found", config_file)
            raise SystemExit("*** ERROR*** config file not found")

        return args


if __name__ == '__main__':
    # set up the logger
    dirname, filename = os.path.split(os.path.abspath(__file__))  # get the directory for the log file
    logging.basicConfig(
        filename=dirname + "/tempcontroller.log",
        level=logging.DEBUG,
        format="%(levelname)s: %(asctime)s: %(message)s")

    logging.info("")
    logging.info("===== Starting %s =====", __file__)
    logging.info("")

    args = parse_args()
    main(config_file=args.config_file)
