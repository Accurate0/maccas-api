FROM rust:1.66.0 as builder

WORKDIR /usr/src/refresh-worker
COPY . .
RUN cargo build --locked --profile release-strip --bin refresh-worker

FROM alpine:3.17.1

RUN apk add --no-cache ca-certificates
RUN update-ca-certificates

WORKDIR /tmp
COPY --from=builder /usr/src/refresh-worker/target/x86_64-unknown-linux-gnu/release-strip/refresh-worker /usr/local/bin/refresh-worker
CMD ["refresh-worker"]
