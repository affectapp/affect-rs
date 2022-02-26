#!/bin/bash
# Builds and runs a containerized affect server proxied
# through envoy to support http1 (frontend/web requests).
export PORT=8080
export ENVOY_ADMIN_PORT=9901
export AFFECT_PORT=50051
export CONFIG=`cat ./server/affect.toml`

# Build image.
docker build -t affect-server .

# Run container interactively.
echo "Starting container, shortcut links:"
echo "Envoy admin: http://localhost:${ENVOY_ADMIN_PORT}"
echo "grpc-web: http://localhost:${AFFECT_PORT}"
echo "grpc: http://localhost:${PORT}"
echo "Exit using Ctrl+C..."
docker run -it \
  -p ${PORT}:${PORT} \
  -p ${AFFECT_PORT}:${AFFECT_PORT} \
  -p ${ENVOY_ADMIN_PORT}:${ENVOY_ADMIN_PORT} \
  -e PORT \
  -e CONFIG \
  affect-server
