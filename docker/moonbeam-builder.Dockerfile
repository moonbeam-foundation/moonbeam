# Build Moonbeam Node
#
# Requires to run from repository root and to copy the binary in the build folder (part of the release workflow)

FROM docker.io/library/ubuntu:22.04 AS builder

ARG RUSTFLAGS=""
ENV RUSTFLAGS=$RUSTFLAGS
ENV BUILD_PARAMS="--release --all"
ENV DEBIAN_FRONTEND=noninteractive

WORKDIR /

RUN echo "*** Installing Basic dependencies ***"
RUN apt-get update && apt-get install -y ca-certificates && update-ca-certificates
RUN apt install --assume-yes git clang curl libssl-dev llvm libudev-dev make protobuf-compiler pkg-config

RUN set -e

RUN echo "*** Installing Rust environment ***"
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:$PATH"
RUN rustup default stable
# rustup version are pinned in the rust-toolchain file


WORKDIR /moonbeam/moonbeam

# Print target cpu
RUN rustc --print target-cpus

RUN echo "*** Building Moonbeam ***"
RUN cargo build "$BUILD_PARAMS"
