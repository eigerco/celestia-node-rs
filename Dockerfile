FROM rust:1.74 AS builder

WORKDIR /build

RUN apt-get update && \
  apt-get install -y protobuf-compiler && \
  apt-get clean

RUN cargo install wasm-pack@0.12.1 --locked

COPY . .

RUN wasm-pack build --release --target web node-wasm

RUN cargo build --release --bin lumina --features browser-node


FROM debian:bookworm-slim

COPY --from=builder /build/target/release/lumina /usr/local/bin/lumina

ENTRYPOINT ["lumina"]
CMD ["node"]
