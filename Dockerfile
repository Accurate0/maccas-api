FROM rust:1.60 as build

COPY ./ ./
RUN cd maccas-bot && cargo build --release

FROM rust:1.60

COPY --from=build ./maccas-bot/target/release/maccas-bot /bot/maccas-bot

CMD [ "/bot/maccas-bot" ]
