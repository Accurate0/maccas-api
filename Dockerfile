FROM rust:1.60-alpine as builder

WORKDIR /

RUN apk update && apk add musl-dev pkgconf openssl openssl-dev gcc --no-cache
COPY ./ ./
RUN cd maccas_bot && cargo build --release
RUN rm -rf ./maccas_bot/target/release/deps && rm -rf ./libmaccas/target/release/deps

FROM alpine:3.12

WORKDIR /bot

COPY --from=builder /maccas_bot/target/release/maccas_bot ./

CMD ["/bot/maccas_bot"]
