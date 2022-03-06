#!/bin/sh
# Entrypoint for docker container. Not intended to be run outside
# of docker environment (see Dockerfile).

# Start server in background.
./affect-server 2>&1 &
AFFECT_SERVER_PID=$!
echo "Affect PID: ${AFFECT_SERVER_PID}"
while ! nc -z localhost ${AFFECT_SERVER_PORT}; do
  sleep 1
done

# Substitute env variables to produce final envoy config.
envsubst < ./envoy.yaml > ./envoy-final.yaml

# Start envoy in background.
./envoy --config-path ./envoy-final.yaml --service-cluster backend-proxy 2>&1 &
ENVOY_PID=$!
echo "Envoy PID: ${ENVOY_PID}"

trap "jobs -p | xargs -r kill" INT TERMRM

while nc -z localhost ${AFFECT_SERVER_PORT}; do
  sleep 1
done