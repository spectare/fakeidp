#!/bin/bash

#NOTE: This assumes the docker run as found in the README, exposing on port 9090
curl -d @claim.json -H "Content-Type: application/json" -XPOST http://localhost:9090/token -v