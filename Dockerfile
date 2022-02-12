# Cacheable-ish builder steps (just dependencies).
FROM rust:1.58 AS planner
WORKDIR /builder
RUN rustup component add rustfmt
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM rust:1.58 AS cacher
WORKDIR /cacher
RUN rustup component add rustfmt
RUN cargo install cargo-chef
COPY --from=planner /builder/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:1.58 AS builder
WORKDIR /builder
COPY . .
COPY --from=cacher /cacher/target ./target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
RUN rustup component add rustfmt
RUN cargo build --release

# App
FROM debian:buster-slim
WORKDIR /app
COPY --from=builder /builder/target/release/affect-server .
RUN apt-get update && apt-get install -y --no-install-recommends libssl-dev ca-certificates
CMD ["/app/affect-server"]