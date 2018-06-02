#!/bin/bash
DIR=/home/pi/development/fermenter-temp-controller
SCRIPT=tempcontroller.py
CONFIG_FILE=config.txt

# run the above script if it's not running
pgrep -f $SCRIPT || nohup python3 $DIR/$SCRIPT $DIR/$CONFIG_FILE



