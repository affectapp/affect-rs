# Dockerfile for running an instance of the Affect server.
# Expects environment variables to be present:
# - PORT: The port to receive traffic within the container.
# - CONFIG or CONFIG_PATH: The affect config path to affect config toml file. 

# Sets up caching/building affect.
FROM rust:1.58 AS planner
WORKDIR /builder
RUN rustup component add rustfmt
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Caches affect dependencies.
FROM rust:1.58 AS cacher
WORKDIR /cacher
RUN rustup component add rustfmt
RUN cargo install cargo-chef
COPY --from=planner /builder/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Builds affect.
FROM rust:1.58 AS builder
WORKDIR /builder
COPY . .
COPY --from=cacher /cacher/target ./target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
RUN rustup component add rustfmt
RUN cargo build --release

# Contains the built envoy binary.
FROM envoyproxy/envoy:v1.12.2 AS envoy

# Runtime for the server and envoy.
FROM debian:buster-slim
ENV RUST_LOG=debug
WORKDIR /app
COPY --from=builder /builder/target/release/affect-server .
COPY --from=envoy /usr/local/bin/envoy .
COPY ./envoy.yaml ./
RUN apt-get update && \
  apt-get install -y --no-install-recommends libssl-dev ca-certificates gettext-base
# Start envoy in the background (with env variables in config), and
# affect in the foreground.
ENTRYPOINT envsubst < ./envoy.yaml > ./envoy-final.yaml && \
  ./envoy --config-path ./envoy-final.yaml --service-cluster backend-proxy & \
  ./affect-server