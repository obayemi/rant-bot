FROM rust:1.58 as builder

RUN USER=root cargo new --bin rant-bot
WORKDIR ./rant-bot
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs
RUN rm ./target/release/deps/rant_bot*

ADD ./src ./src
ADD ./sqlx-data.json ./

RUN cargo build --release


FROM debian:buster-slim
ARG APP=/usr/src/app

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder /rant-bot/target/release/rant-bot ${APP}/rant-bot

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./rant-bot"]