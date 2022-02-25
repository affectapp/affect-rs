PORT=8080
ENVOY_ADMIN_PORT=9901
AFFECT_PORT=50051
CONFIG=`cat ./server/config.toml`

# docker build -t affect-server .
echo "Envoy admin: http://localhost:9901"
echo "grpc-web: http://localhost:50051"
echo "grpc: http://localhost:${PORT}"

docker run -it \
  -p ${PORT}:${PORT} \
  -p ${AFFECT_PORT}:${AFFECT_PORT} \
  -p ${ENVOY_ADMIN_PORT}:${ENVOY_ADMIN_PORT} \
  -e PORT \
  -e CONFIG \
  affect-server
