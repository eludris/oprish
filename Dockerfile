# syntax=docker/dockerfile:1.4
FROM rust:slim-buster as builder

WORKDIR /oprish

# Remove docker's default of removing cache after use.
RUN rm -f /etc/apt/apt.conf.d/docker-clean; echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' > /etc/apt/apt.conf.d/keep-cache
ENV PACKAGES build-essential
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -yqq --no-install-recommends \
    $PACKAGES && rm -rf /var/lib/apt/lists/*

COPY Cargo.lock Cargo.toml ./
COPY ./src ./src

RUN --mount=type=cache,target=./target \
    cargo build --release
# Other image cannot access the target folder.
RUN --mount=type=cache,target=./target \
    cp ./target/release/oprish /usr/local/bin/oprish

FROM debian:buster-slim

COPY --from=builder /usr/local/bin/oprish /bin/oprish

# Don't forget to also publish these ports in the docker-compose.yml file.
ARG PORT=7159

EXPOSE $PORT
ENV ROCKET_ADDRESS 0.0.0.0
ENV ROCKET_PORT $PORT

ENV RUST_LOG debug

CMD ["/bin/oprish"]
