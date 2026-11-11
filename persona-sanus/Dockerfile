FROM rust:bookworm AS builder
RUN cargo install dioxus-cli --locked
WORKDIR /build
COPY . .
RUN dx build --release --fullstack

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/*
RUN mkdir -p /data /app
WORKDIR /app
COPY --from=builder /build/target/dx/persona-sanus/release/web/server /app/persona-sanus
COPY --from=builder /build/target/dx/persona-sanus/release/web/public /app/public
CMD ["/app/persona-sanus"]
