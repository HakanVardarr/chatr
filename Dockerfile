FROM rust:1.89 AS builder
WORKDIR /app

RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get install -y --no-install-recommends musl-tools pkg-config

COPY Cargo.toml Cargo.lock ./
COPY crates/server/Cargo.toml crates/server/Cargo.toml

RUN mkdir -p crates/server/src && echo "fn main() {}" > crates/server/src/main.rs
RUN cargo build --release -p server || true

COPY . .
RUN cargo build --release -p server --target x86_64-unknown-linux-musl

FROM debian:bookworm-slim AS runtime

RUN useradd -m -u 10001 appuser
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/server /usr/local/bin/chatr-server
EXPOSE 3030

USER appuser

ENV RUST_LOG=info
ENTRYPOINT ["/usr/local/bin/chatr-server"]
