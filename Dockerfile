FROM lukemathwalker/cargo-chef:0.1.77-rust-1.93.1-alpine3.23 AS chef
LABEL org.opencontainers.image.source="https://github.com/thatgurkangurk/skekbot-rs"

WORKDIR /skekbot-rs

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN apk add --no-cache mold clang

ENV RUSTFLAGS="-C linker=clang -C link-arg=-fuse-ld=mold"

WORKDIR /skekbot-rs

COPY --from=planner /skekbot-rs/recipe.json recipe.json

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json

COPY . .

ARG APP_VERSION
ENV APP_VERSION=$APP_VERSION

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo build --release --bin skekbot_rs && \
    mkdir -p /out && \
    cp target/release/skekbot_rs /out/skekbot_rs

FROM alpine:3.23 AS runtime
WORKDIR /skekbot-rs

COPY --from=builder /out/skekbot_rs /usr/local/bin/skekbot_rs

RUN apk add --no-cache ca-certificates tzdata && \
    update-ca-certificates

ARG UID=1000
ARG GID=1000

RUN addgroup -g $GID -S skekbot && \
    adduser -u $UID -S skekbot -G skekbot

RUN mkdir -p /app/data && chown skekbot:skekbot /app/data

ENTRYPOINT ["/usr/local/bin/skekbot_rs"]