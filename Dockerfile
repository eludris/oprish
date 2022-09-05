FROM ekidd/rust-musl-builder:stable as builder

RUN USER=root cargo new --bin oprish
WORKDIR ./oprish

COPY Cargo.lock Cargo.toml ./

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN rm ./target/x86_64-unknown-linux-musl/release/deps/oprish*
RUN cargo build --release


FROM alpine:latest

COPY --from=builder /home/rust/src/oprish/target/x86_64-unknown-linux-musl/release/oprish /bin/oprish

# Don't forget to also publish these ports in the docker-compose.yml file.
ARG PORT=8000

EXPOSE $PORT
ENV ROCKET_ADDRESS 0.0.0.0
ENV ROCKET_PORT $PORT

ENV RUST_LOG debug

CMD ["/bin/oprish"]

