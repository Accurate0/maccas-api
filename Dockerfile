FROM rust:1.60-alpine as builder

RUN apk update && apk add musl-dev pkgconf openssl openssl-dev gcc --no-cache
COPY ./ ./
RUN cd maccas_bot && cargo build --release
RUN rm -rf ./maccas_bot/target/release/deps && rm -rf ./libmaccas/target/release/deps

CMD ["./maccas_bot/target/release/maccas_bot"]

FROM alpine

WORKDIR /bot

# Copy our build
COPY --from=builder /maccas_bot/target/release/maccas_bot ./

CMD ["/bot/maccas_bot"]
