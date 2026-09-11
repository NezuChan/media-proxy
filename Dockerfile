FROM rust:1-alpine AS build-stage

RUN apk add --no-cache musl-dev

WORKDIR /tmp/build

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --locked

FROM gcr.io/distroless/static-debian12:nonroot

LABEL name "NezukoChan Media Proxy"
LABEL maintainer "KagChi"

WORKDIR /app

COPY --from=build-stage /tmp/build/target/release/media-proxy /app/media-proxy

EXPOSE 3000

ENTRYPOINT ["/app/media-proxy"]
