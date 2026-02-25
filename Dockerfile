FROM rust:alpine AS build
LABEL org.opencontainers.image.source="https://github.com/thatgurkangurk/skekbot-rs"

RUN apk update && \
    apk upgrade --no-cache && \
    apk add --no-cache lld mold musl musl-dev libc-dev cmake clang clang-dev openssl file \
        libressl-dev git make build-base bash curl wget zip gnupg coreutils gcc g++  zstd binutils ca-certificates upx

WORKDIR /skekbot-rs
COPY . ./
# or make build
RUN cargo build --release

####################################################################################################
## Final image
####################################################################################################
FROM alpine:3.20

# Install runtime deps (TLS + timezone)
RUN apk add --no-cache ca-certificates tzdata && \
    update-ca-certificates

# Create non-root user
RUN addgroup -S skekbot && adduser -S skekbot -G skekbot

WORKDIR /app

# Copy compiled binary
COPY --from=build /skekbot-rs/target/release/skekbot-rs /bin/skekbot-rs

# Use unprivileged user
USER skekbot

ENTRYPOINT ["/bin/skekbot-rs"]