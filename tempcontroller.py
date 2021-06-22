# simple program to read from Arduino serial port

import argparse
import configparser
import elasticsearch
import glob
import json
import logging
import os
import serial
import time
from datetime import datetime
from elasticsearch import Elasticsearch
from pathlib import Path
from pytz import timezone
from serial import SerialException

# use a local Elasticsearch instance
ES_HOST = "192.168.1.55"

# get NZ timezone so we can localise the timestamps and deal with NZST/NZDT
nztz = timezone("NZ")

# get the directory for the log file
dirname, filename = os.path.split(os.path.abspath(__file__))

# set up the logger
logging.basicConfig(
    filename=dirname + "/tempcontroller.log",
    level=logging.INFO,
    format="%(levelname)s: %(asctime)s: %(message)s")

logging.info("")
logging.info("===== Starting %s =====", __file__)
logging.info("")

### TEST CODE
###


# set up the command line parser
parser = argparse.ArgumentParser()

# define the argument to set the config file
parser.add_argument(
    "config_file", help="specify the full location of the config file")

# parse the arguments and check the file exists
args = parser.parse_args()

if args.config_file:
    config_file = args.config_file
    logging.info("Using config file '%s'", config_file)

    if not Path(config_file).exists():
        logging.critical("Config file '%s' not found", config_file)
        raise SystemExit("*** ERROR*** config file not found")

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
    ser = serial.Serial(tty, 9600)
except SerialException:
    logging.error("Couldn't open serial port at %s", tty)
    raise SystemExit("*** ERROR *** Couldn't open serial port")

# read the target temp from config file
config = configparser.ConfigParser()
config.read(config_file)
new_target = config["temperature"]["TargetTemp"]
logging.info("Target temperature being set to %s", str(new_target))

# sleep for 30 secs to allow arduino to reboot after serial port open
time.sleep(30)

new_target_str = '<' + str(new_target) + '>'

# write target temp to serial port
try:
    ser.write(new_target_str.encode())
except SerialException:
    logging.warning("Couldn't write target temp to serial port")

while True:
    line = ser.readline()  # read serial line as bytes

    try:
        # convert serial line to string and load to JSON sequence
        data = json.loads(line.decode("utf-8"))

        # get formatted localised timestamp
        stamp = nztz.localize(datetime.now()).strftime("%Y-%m-%dT%H:%M:%S%z")
        logging.debug("timestamp is %s", stamp)

        data["timestamp"] = stamp
        #    data['brewid'] = '12-AAA-02'
        data["brewid"] = '99-TEST-99'

        target = data["target"]
        logging.debug("Ardino target is %s and script target is %s", target, new_target)

        # check if we need to update the target temp
        if (round(float(target), 1) != round(float(new_target), 1)):
            new_target_str = '<' + str(new_target) + '>'
            ser.write(new_target_str.encode())
            logging.info("Updated target temp to %s", str(new_target))

        doc = data
        logging.debug("Indexing %s", json.dumps(doc))

        try:
            es = Elasticsearch(
                hosts=[{'host': ES_HOST, 'port': 9200}],
                #       use_ssl=True,
                #       verify_certs=True,
                connection_class=elasticsearch.connection.RequestsHttpConnection)

            # index the doc to elastic
            res = es.index(index="test-temp", doc_type="temp-reading", body=doc)
            logging.debug("Result is %s", json.dumps(res))
            logging.info("Indexed record: time=%s, temp=%s, target=%s, action=%s", data["timestamp"], data["avg"],
                         data["target"], data["action"])

        except elasticsearch.exceptions.ConnectionError as err:
            logging.critical("*** ConnectionError *** %s", err)
            raise (err)
    except json.decoder.JSONDecodeError as err:
        logging.debug(err)
