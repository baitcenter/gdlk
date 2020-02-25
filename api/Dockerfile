FROM rust:slim

# This should be built with the repo root dir as the context

RUN apt-get update && \
    apt-get install -y \
    libpq-dev \
    pkg-config \
    && \
    rm -rf /var/lib/apt/lists/*

ENV DOCKERIZE_VERSION v0.6.1
ADD https://github.com/jwilder/dockerize/releases/download/$DOCKERIZE_VERSION/dockerize-alpine-linux-amd64-$DOCKERIZE_VERSION.tar.gz .
RUN tar -C /usr/local/bin -xzvf dockerize-alpine-linux-amd64-$DOCKERIZE_VERSION.tar.gz && \
    rm dockerize-alpine-linux-amd64-$DOCKERIZE_VERSION.tar.gz

# Need to pick up the rust-toolchain file first
WORKDIR /app
COPY rust-toolchain .
RUN cargo install diesel_cli --no-default-features --features=postgres

# Build the local project
COPY .clippy.toml .rustfmt.toml /app/
COPY ./api /app/api
COPY ./core /app/core
WORKDIR /app/api
RUN cargo build
