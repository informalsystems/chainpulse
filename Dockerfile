# syntax = docker/dockerfile:1.4

# Usage:
#   docker build . --build-arg -t informalsystems/chainpulse:0.1.0 -f ci/Dockerfile

FROM rust:1-slim-bullseye as builder

WORKDIR /usr/src

RUN     USER=root cargo new chainpulse
WORKDIR /usr/src/chainpulse
COPY    .cargo .cargo
COPY    Cargo.toml Cargo.lock ./
RUN     --mount=type=cache,target=/root/.rustup \
        --mount=type=cache,target=/root/.cargo/registry \
        --mount=type=cache,target=/root/.cargo/git \
        --mount=type=cache,target=/usr/src/target \
        cargo build --release
COPY    src src
RUN     touch src/main.rs
RUN     cargo build --release
RUN     objcopy --compress-debug-sections ./target/release/chainpulse ./chainpulse

FROM gcr.io/distroless/cc AS runtime 
LABEL maintainer="hello@informal.systems"

WORKDIR /app
COPY    --from=builder /usr/src/chainpulse/chainpulse ./

ENTRYPOINT ["/app/chainpulse"]
