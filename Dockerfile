# rust:1 (actual) — 1.84-slim no compila las deps que exigen edition2024.
FROM rust:1-slim-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
# Calienta el registro de deps: el crate declara [lib] + [[bin]], hay que
# crear ambos stubs o cargo falla al resolver el target 'media_gateway'.
RUN mkdir src && echo 'fn main() {}' > src/main.rs && echo '' > src/lib.rs && cargo build --release && rm -rf src

COPY sql/ sql/
COPY src/ src/
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/media-gateway /usr/local/bin/media-gateway
COPY --from=builder /app/sql /app/sql

WORKDIR /app

ENV HOST=0.0.0.0
ENV PORT=3004
ENV RUST_LOG=info

EXPOSE 3004

CMD ["media-gateway"]
