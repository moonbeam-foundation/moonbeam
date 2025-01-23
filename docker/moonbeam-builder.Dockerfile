# Build Moonbeam Node
#
# Requires to run from repository root and to copy the binary in the build folder (part of the release workflow)

FROM docker.io/library/ubuntu:22.04 AS builder

ARG BUILD_PARAMS="--release --all"
ARG RUSTFLAGS="-C opt-level=3 -D warnings -C linker=clang -C link-arg=-fuse-ld=/build/mold/bin/mold"
ENV BUILD_PARAMS=$BUILD_PARAMS
ENV RUSTFLAGS=$RUSTFLAGS
ENV DEBIAN_FRONTEND=noninteractive
ENV CARGO_INCREMENTAL=0

WORKDIR /build
COPY . /build/

RUN echo "*** Installing Basic dependencies ***"
RUN apt-get update && apt-get install -y ca-certificates && update-ca-certificates
RUN apt install --assume-yes git clang curl libssl-dev llvm libudev-dev make protobuf-compiler pkg-config

RUN set -e

# Setup mold linker
RUN echo "*** Setup mold linker ***"
RUN mkdir -p mold
RUN curl -L --retry 10 --silent --show-error https://github.com/rui314/mold/releases/download/v2.30.0/mold-2.30.0-$(uname -m)-linux.tar.gz | tar -C $(realpath mold) --strip-components=1 -xzf -

RUN echo "*** Installing Rust environment ***"
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:$PATH"
RUN rustup default stable
# rustup version are pinned in the rust-toolchain file

# Print target cpu
RUN rustc --print target-cpus

RUN echo "*** Building Moonbeam ***"
RUN cargo build $BUILD_PARAMS
