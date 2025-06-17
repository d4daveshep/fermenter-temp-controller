#!/bin/bash

# inital config of latest influxdb2 docker image
# image will stay running at port 8086

docker run -d -p 8086:8086 \
  -v $PWD/influx_data/data:/var/lib/influxdb2 \
  -v $PWD/influx_data/config:/etc/influxdb2 \
  influxdb:2.7-alpine
