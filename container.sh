#!/bin/bash
# Builds and runs a containerized affect server.
export PORT=8080
export CONFIG=`cat ./server/affect.toml`

# Build image.
docker build -t affect-server .
if [ $? -ne 0 ]; then
  return
fi

# Run container interactively.
echo "Starting container: http://localhost:${PORT}"
echo "Exit using Ctrl+C..."
docker run -it \
  -p ${PORT}:${PORT} \
  -e PORT \
  -e CONFIG \
  affect-server
if [ $? -ne 0 ]; then
  return
fi