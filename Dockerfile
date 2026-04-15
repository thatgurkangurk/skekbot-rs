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

RUN cargo build --release --bin skekbot-rs

FROM alpine:3.23 AS runtime
WORKDIR /skekbot-rs

COPY --from=builder /skekbot-rs/target/release/skekbot-rs /usr/local/bin

RUN apk add --no-cache ca-certificates tzdata && \
    update-ca-certificates

RUN addgroup -S skekbot && adduser -S skekbot -G skekbot

USER skekbot

ENTRYPOINT [ "/usr/local/bin/skekbot-rs" ]