FROM rust:1.84-slim-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --release && rm -rf src

COPY sql/ sql/
COPY src/ src/
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/media-gateway /usr/local/bin/media-gateway
COPY --from=builder /app/sql /app/sql

WORKDIR /app

ENV HOST=0.0.0.0
ENV PORT=3003
ENV RUST_LOG=info

EXPOSE 3003

CMD ["media-gateway"]
