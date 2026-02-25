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
## This stage is used to get the correct files into the final image
####################################################################################################
FROM alpine:latest AS files

# mailcap is used for content type (MIME type) detection
# tzdata is used for timezone info
RUN apk update && \
    apk upgrade --no-cache && \
    apk add --no-cache ca-certificates mailcap tzdata

RUN update-ca-certificates

ENV USER=skekbot-rs
ENV UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"


####################################################################################################
## Final image
####################################################################################################
FROM scratch

# /etc/nsswitch.conf may be used by some DNS resolvers
# /etc/mime.types may be used to detect the MIME type of files
COPY --from=files --chmod=444 \
    /etc/passwd \
    /etc/group \
    /etc/nsswitch.conf \
    /etc/mime.types \
    /etc/

COPY --from=files --chmod=444 /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=files --chmod=444 /usr/share/zoneinfo /usr/share/zoneinfo

# Copy our build
COPY --from=build /skekbot-rs/target/release/skekbot-rs /bin/skekbot-rs

# Use an unprivileged user.
USER skekbot-rs:skekbot-rs

ENV DISCORD_TOKEN="CHANGE ME"

# The scratch image doesn't have a /tmp folder, you may need it
# WORKDIR /tmp

WORKDIR /app

ENTRYPOINT ["/bin/skekbot-rs"]