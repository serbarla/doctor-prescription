FROM rust:bookworm AS builder
RUN cargo install dioxus-cli --version 0.7.10 --locked
WORKDIR /build
COPY . .
RUN dx build --release --fullstack || test -x /build/target/dx/persona-sanus/release/web/server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl age && rm -rf /var/lib/apt/lists/*
RUN curl -fsSL -o /usr/local/bin/mc https://github.com/minio/mc/releases/download/RELEASE.2025-08-13T08-35-41Z/mc.linux-amd64.RELEASE.2025-08-13T08-35-41Z && chmod +x /usr/local/bin/mc
RUN mkdir -p /data /app
WORKDIR /app
COPY --from=builder /build/target/dx/persona-sanus/release/web/server /app/persona-sanus
COPY --from=builder /build/target/dx/persona-sanus/release/web/public /app/public
CMD ["/app/persona-sanus"]
