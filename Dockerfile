FROM rust:1.60 as build

COPY ./ ./
RUN cd maccas_bot && cargo build --release

FROM rust:1.60

COPY --from=build ./maccas_bot/target/release/maccas_bot /bot/maccas_bot

CMD [ "/bot/maccas_bot" ]
