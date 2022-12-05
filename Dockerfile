FROM rust:slim-buster as builder

RUN USER=root cargo new --bin oprish
WORKDIR /oprish

RUN apt-get update && apt-get install -y build-essential

COPY Cargo.lock Cargo.toml ./

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/oprish*
RUN cargo build --release


FROM debian:buster-slim

COPY --from=builder /oprish/target/release/oprish /bin/oprish

# Don't forget to also publish these ports in the docker-compose.yml file.
ARG PORT=7159

EXPOSE $PORT
ENV ROCKET_ADDRESS 0.0.0.0
ENV OPRISH_PORT $PORT

ENV RUST_LOG debug

CMD ["/bin/oprish"]

