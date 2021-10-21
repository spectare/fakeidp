#!/bin/bash

#NOTE: This assumes the docker run as found in the README, exposing on port 9090
curl -GET http://localhost:9090/.well-known/openid-configuration