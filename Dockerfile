# syntax=docker/dockerfile:1
FROM rust:slim-buster as builder

WORKDIR /oprish

COPY Cargo.lock Cargo.toml ./
COPY ./src ./src

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/oprish/target \
    cargo build --release
# Other image cannot access the target folder.
RUN --mount=type=cache,target=/oprish/target \
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
