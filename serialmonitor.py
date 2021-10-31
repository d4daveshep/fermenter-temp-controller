# simple program to read from Arduino serial port

import argparse
import configparser
import glob
import json
import logging
import os
from datetime import datetime
from json import JSONDecodeError
from pathlib import Path

import pandas as pd
import serial
import time
from influxdb import InfluxDBClient
from influxdb.exceptions import InfluxDBClientError
from pytz import timezone
from serial import SerialException


def main(config_file):
    # get NZ timezone so we can localise the timestamps and deal with NZST/NZDT
    nztz = timezone("NZ")

    # open the serial port
    serial_port = get_serial_port()

    # read the  config file
    config = configparser.ConfigParser()
    config.read(config_file)

    # get the brew ID
    if config["fermenter"]["BrewID"]:
        brew_id = config["fermenter"]["BrewID"]
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

    # infinite loop to read data from serial port
    while True:
        logging.debug("--------------------")
        line = serial_port.readline()  # read serial line as bytes
        logging.debug(line.decode("utf-8"))




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
    logging.info("Sleeping for 30 sec to allow arduino to reboot after serial port was opened")
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
#        level=logging.INFO,
        level=logging.DEBUG,
        format="%(levelname)s: %(asctime)s: %(message)s")

    logging.info("")
    logging.info("===== Starting %s =====", __file__)
    logging.info("")

    args = parse_args()
    main(config_file=args.config_file)
