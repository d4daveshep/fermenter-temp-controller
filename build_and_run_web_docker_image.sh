#!/bin/bash

IMAGE_NAME="web_apis"

# Docker image build
docker build -f ./Dockerfile.web_apis -t ${IMAGE_NAME} .

# Docker run
docker run --network="host" ${IMAGE_NAME}

