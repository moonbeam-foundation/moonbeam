# Node for Moonbeam
#
# Requires to run from repository root and to copy the binary in the build folder (part of the release workflow)

FROM docker.io/library/ubuntu:20.04 AS builder

# Branch or tag to build moonbeam from
ARG COMMIT="master"
ARG RUSTFLAGS=""
ENV RUSTFLAGS=$RUSTFLAGS
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

RUN echo "*** Cloning Moonbeam ***"
RUN git clone --depth=1 --branch $COMMIT https://github.com/purestake/moonbeam.git

WORKDIR /moonbeam/moonbeam

RUN echo "*** Building Moonbeam ***"
RUN cargo build --profile=production --all

FROM debian:bookworm-slim
LABEL maintainer "alan@purestake.com"
LABEL description="Binary for Moonbeam Nodes"

RUN useradd -m -u 1000 -U -s /bin/sh -d /moonbeam moonbeam && \
	mkdir -p /moonbeam/.local/share && \
	mkdir /data && \
	chown -R moonbeam:moonbeam /data && \
	ln -s /data /moonbeam/.local/share/moonbeam && \
	rm -rf /usr/sbin

USER moonbeam

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder --chown=moonbeam /moonbeam/target/production/moonbeam /moonbeam/moonbeam

RUN chmod uog+x /moonbeam/moonbeam

# 30333 for parachain p2p
# 30334 for relaychain p2p
# 9944 for Websocket & RPC call
# 9615 for Prometheus (metrics)
EXPOSE 30333 30334 9944 9615

VOLUME ["/data"]

ENTRYPOINT ["/moonbeam/moonbeam"]
