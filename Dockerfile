# syntax=docker/dockerfile:1.3

FROM rust:latest as builder

ENV HOME=/home/root
WORKDIR $HOME/als-rs

RUN rustup component add rustfmt

RUN pwd && ls -la
COPY . ./



# make sure to use cache to speed up builds
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    # --mount=type=cache,target=/home/root/als-rs/target \
    cargo build --release && \
    # Copy executable out of the cache so it is available in the final image.
    cp $HOME/als-rs/target/release/als-rs ./als-rs

FROM debian:buster-slim
ARG APP=/usr/src/als-rs
ENV APP_USER=appuser

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder /home/root/als-rs/als-rs ${APP}/als-rs

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

EXPOSE 50051

CMD ["./als-rs"]