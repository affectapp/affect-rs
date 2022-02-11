# Cacheable-ish builder step (just dependencies).
FROM rust:1.58 AS builder
WORKDIR /deps
RUN cargo new --lib --name=affect-server ./server
COPY ./Cargo.toml ./
COPY ./Cargo.lock ./
COPY ./server/Cargo.toml ./server
RUN rustup component add rustfmt
RUN cargo build --release
RUN rm -rf **/src
# Build with project srcs.
WORKDIR /builder
COPY ./ ./
RUN cp -rf /deps/* ./
RUN cargo build --release

# App
FROM debian:buster-slim
WORKDIR /app
COPY --from=builder /builder/target/release/affect-server .
RUN apt-get update && apt-get install -y --no-install-recommends libssl-dev ca-certificates
CMD ["/app/affect-server"]