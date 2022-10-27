#!/bin/bash

IMAGE_NAME="test_fermenter_controller"

#To build docker image
docker build -f ./Dockerfile.pytests -t ${IMAGE_NAME} .

#To run docker container in host network mode
docker run --privileged --network="host" ${IMAGE_NAME}