FROM ubuntu:20.04

ARG RUST_VERSION=stable
ARG TARGET=x86_64-unknown-linux-gnu
ENV DEBIAN_FRONTEND=noninteractive \
    CARGO_HOME=/usr/local/cargo \
    RUSTUP_HOME=/usr/local/rustup

RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential  \
        clang            \
        curl ca-certificates   \
        git pkg-config   \
        libssl-dev       \
        && rm -rf /var/lib/apt/lists/*

RUN curl https://sh.rustup.rs -sSf \
      | sh -s -- -y --profile minimal --default-toolchain ${RUST_VERSION} \
 && /usr/local/cargo/bin/rustup target add ${TARGET}

ENV PATH="${CARGO_HOME}/bin:${PATH}"

