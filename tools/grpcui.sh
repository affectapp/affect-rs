#!/bin/bash
export AFFECT_SERVER_PORT=${AFFECT_SERVER_PORT:-50051}
grpcui -plaintext \
  -bind localhost \
  -port 30031 \
  "localhost:${AFFECT_SERVER_PORT}"