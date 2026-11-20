FROM rust:trixie AS builder
RUN cargo install dioxus-cli --version 0.7.10 --locked
WORKDIR /build
COPY . .
RUN dx build --release --fullstack

FROM debian:trixie-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl age && rm -rf /var/lib/apt/lists/*
ARG TARGETARCH
ARG MC_RELEASE=RELEASE.2025-08-13T08-35-41Z
RUN case "$TARGETARCH" in amd64|arm64) ;; *) echo "unsupported TARGETARCH: $TARGETARCH" >&2; exit 1 ;; esac && curl -fsSL -o /usr/local/bin/mc "https://github.com/minio/mc/releases/download/${MC_RELEASE}/mc.linux-${TARGETARCH}.${MC_RELEASE}" && curl -fsSL "https://github.com/minio/mc/releases/download/${MC_RELEASE}/mc.linux-${TARGETARCH}.${MC_RELEASE}.sha256sum" | awk -v bin=/usr/local/bin/mc '{print $1"  "bin}' | sha256sum -c - && chmod +x /usr/local/bin/mc
RUN mkdir -p /data /app
WORKDIR /app
COPY --from=builder /build/target/dx/persona-sanus/release/web/server /app/persona-sanus
COPY --from=builder /build/target/dx/persona-sanus/release/web/public /app/public
ENV IP=0.0.0.0
EXPOSE 8080
CMD ["/app/persona-sanus"]
