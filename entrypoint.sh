#!/bin/bash
# Entrypoint for docker container. Not intended to be run outside
# of docker environment (see Dockerfile).

# Start server in background.
./affect-server 2>&1 &
echo "Affect PID: $!"

# Wait up to N seconds for affect to come online before
# starting envoy proxy.
elapsed=0
while ! nc -z localhost ${AFFECT_SERVER_PORT}; do
  sleep 1
  elapsed=$((counter + 1))
  if [[ "$counter" -gt 10 ]]; then
    echo "Affect server never came online!"
    exit 2
  fi
done

# Substitute env variables to produce final envoy config.
# Then run envoy proxy.
envsubst < ./envoy.yaml > ./envoy-final.yaml
./envoy --config-path ./envoy-final.yaml --service-cluster backend-proxy 2>&1 &
echo "Envoy PID: $!"

# On kill/ctrl+c, also kill background processes.
trap "$(jobs -p | xargs -r kill)" SIGHUP SIGINT SIGTERM
wait