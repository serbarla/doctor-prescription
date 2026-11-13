FROM golang:1.24-bookworm AS build
ARG MINIO_VERSION=RELEASE.2025-10-15T17-29-55Z
RUN CGO_ENABLED=0 GOFLAGS=-trimpath go install github.com/minio/minio@${MINIO_VERSION}

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=build /go/bin/minio /usr/bin/minio
EXPOSE 9000 9001
ENTRYPOINT ["/usr/bin/minio"]
CMD ["server", "/data", "--console-address", ":9001"]
