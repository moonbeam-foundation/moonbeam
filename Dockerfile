# Inspired by Polkadot Dockerfile

FROM phusion/baseimage:0.11 as builder
LABEL maintainer "alan@purestake.com"
LABEL description="This is the build stage for Moonbeam. Here we create the binary."

ARG PROFILE=release
WORKDIR /moonbeam

COPY . /moonbeam

# Download Moonbeam repo
RUN apt-get update && \
	apt-get upgrade -y && \
	apt-get install -y cmake pkg-config libssl-dev git clang

# Download rust dependencies and build the rust binary
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
        export PATH=$PATH:$HOME/.cargo/bin && \
        scripts/init.sh && \
        cargo build --$PROFIL

# ===== SECOND STAGE ======

FROM phusion/baseimage:0.11
LABEL maintainer "alan@purestake.com"
LABEL description="This is the 2nd stage: a very small image where we copy the Moonbeam binary."
ARG PROFILE=release
COPY --from=builder /moonbeam/target/$PROFILE/node-moonbeam /usr/local/bin

RUN mv /usr/share/ca* /tmp && \
	rm -rf /usr/share/*  && \
	mv /tmp/ca-certificates /usr/share/ && \
	rm -rf /usr/lib/python* && \
	useradd -m -u 1000 -U -s /bin/sh -d /moonbeam moonbeam && \
	mkdir -p /moonbeam/.local/share/moonbeam && \
	chown -R moonbeam:moonbeam /moonbeam/.local && \
	ln -s /moonbeam/.local/share/moonbeam /data && \
	rm -rf /usr/bin /usr/sbin

USER moonbeam

# 30333 for p2p traffic
# 9933 for RPC call
# 9944 for Websocket
# 9615 for Prometheus (metrics)
EXPOSE 30333 9933 9944 9615

VOLUME ["/data"]

CMD ["/usr/local/bin/node-moonbeam"]