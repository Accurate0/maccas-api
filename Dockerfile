FROM rust:1.60 as build

COPY ./ ./
RUN cd maccas_bot && cargo build --release

FROM alpine:3.12

RUN apk update && apk add gcc
COPY --from=build ./maccas_bot/target/release/maccas_bot /bot/maccas_bot

CMD [ "/bot/maccas_bot" ]
