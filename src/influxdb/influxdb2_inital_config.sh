#!/bin/bash

mkdir -p $PWD/influx_data/config
mkdir -p $PWD/influx_data/data

# inital config of latest influxdb2 docker image
# image will stay running at port 8086

docker run -d -p 8086:8086 \
  -v $PWD/data:/var/lib/influxdb2 \
  -v $PWD/config:/etc/influxdb2 \
  -e DOCKER_INFLUXDB_INIT_MODE=setup \
  -e DOCKER_INFLUXDB_INIT_USERNAME=my-user \
  -e DOCKER_INFLUXDB_INIT_PASSWORD=my-password \
  -e DOCKER_INFLUXDB_INIT_ORG=daveshep.net \
  -e DOCKER_INFLUXDB_INIT_BUCKET=temp-test \
  -e DOCKER_INFLUXDB_INIT_RETENTION=1w \
  -e DOCKER_INFLUXDB_INIT_ADMIN_TOKEN=my-super-secret-auth-token \
  influxdb:2.7-alpine
