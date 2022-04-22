FROM rust:1.60 as build

COPY ./ ./
RUN cargo build --release -p maccas-bot

FROM rust:1.60

COPY --from=build ./target/release/maccas-bot /bot/maccas-bot

CMD [ "/bot/maccas-bot" ]
