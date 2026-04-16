FROM lukemathwalker/cargo-chef:latest-rust-alpine3.23 AS chef
LABEL org.opencontainers.image.source="https://github.com/thatgurkangurk/skekbot-rs"

WORKDIR /skekbot-rs

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN apk add --no-cache mold clang

ENV RUSTFLAGS="-C linker=clang -C link-arg=-fuse-ld=mold"

COPY --from=planner /skekbot-rs/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

ARG APP_VERSION
ENV APP_VERSION=$APP_VERSION

ARG DATA_DIR="/app/data"

RUN cargo build --release --bin skekbot_rs

FROM alpine:3.23 AS runtime
WORKDIR /skekbot-rs

COPY --from=builder /skekbot-rs/target/release/skekbot_rs /usr/local/bin

RUN apk add --no-cache ca-certificates tzdata && \
    update-ca-certificates

ARG UID=1000
ARG GID=1000

RUN addgroup -g $GID -S skekbot && \
    adduser -u $UID -S skekbot -G skekbot

RUN mkdir -p /app/data && chown skekbot:skekbot /app/data

ENTRYPOINT [ "/usr/local/bin/skekbot_rs" ]