#!/bin/bash
# Builds and runs a containerized affect server proxied
# through envoy to support http1 (frontend/web requests).
export PORT=8080
export ENVOY_ADMIN_PORT=9901
export AFFECT_SERVER_PORT=50051
export CONFIG=`cat ./server/affect.toml`

# Build image.
docker build -t affect-server .
if [ $? -ne 0 ]; then
  exit 2
fi

# Run container interactively.
echo "Starting container, shortcut links:"
echo "Envoy admin: http://localhost:${ENVOY_ADMIN_PORT}"
echo "grpc-web: http://localhost:${AFFECT_SERVER_PORT}"
echo "grpc: http://localhost:${PORT}"
echo "Exit using Ctrl+C..."
docker run -it \
  -p ${PORT}:${PORT} \
  -p ${AFFECT_SERVER_PORT}:${AFFECT_SERVER_PORT} \
  -p ${ENVOY_ADMIN_PORT}:${ENVOY_ADMIN_PORT} \
  -e PORT \
  -e AFFECT_SERVER_PORT \
  -e ENVOY_ADMIN_PORT \
  -e CONFIG \
  affect-server
if [ $? -ne 0 ]; then
  exit 2
fi