#!/bin/bash
export PORT=${PORT:-8080}
grpcui -plaintext \
  -bind localhost \
  -port 30031 \
  "localhost:${PORT}"
