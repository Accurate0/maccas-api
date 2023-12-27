ARG RUST_VERSION=1.74.1
ARG BINARY_NAME

FROM rust:${RUST_VERSION}-slim-bookworm AS builder
ARG BINARY_NAME

RUN apt-get update -y && apt-get install -y pkg-config libssl-dev

WORKDIR /build
RUN mkdir /app

COPY . .
RUN \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release --bin ${BINARY_NAME} && \
    cp ./target/release/${BINARY_NAME} /app

FROM debian:bookworm-slim AS final
ARG BINARY_NAME

RUN apt-get update -y && apt-get install -y libssl-dev ca-certificates
RUN update-ca-certificates
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "10001" \
    appuser

COPY --from=builder /app/${BINARY_NAME} /usr/local/bin/${BINARY_NAME}
RUN chown appuser /usr/local/bin/${BINARY_NAME}

USER appuser

WORKDIR /opt/${BINARY_NAME}
COPY ${BINARY_NAME}.config.toml ${BINARY_NAME}.config.toml

RUN ln -s /usr/local/bin/${BINARY_NAME} executable
ENTRYPOINT ["./executable"]
EXPOSE 8000/tcp