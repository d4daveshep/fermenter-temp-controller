#!/bin/bash

IMAGE_NAME="web_apis"

# Docker build
docker build -f ./Dockerfile.fastapi -t ${IMAGE_NAME} .

# Docker run
docker run --network="host" ${IMAGE_NAME}

