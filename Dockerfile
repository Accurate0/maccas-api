FROM rust:1.60-alpine

RUN apk update && apk add musl-dev pkgconf openssl openssl-dev gcc --no-cache
COPY ./ ./
RUN cd maccas_bot && cargo build --release
RUN rm -rf ./maccas_bot/target/release/deps && rm -rf ./libmaccas

CMD ["./maccas_bot/target/release/maccas_bot"]
