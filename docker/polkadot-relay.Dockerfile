# Inspired by Polkadot Dockerfile

FROM phusion/baseimage:0.11 as builder
LABEL maintainer "alan@purestake.com"
LABEL description="This is the build stage for Polkadot. Here we create the binary."

ARG PROFILE=release
ARG POLKADOT_COMMIT=master
ARG POLKADOT_REPO=https://github.com/paritytech/polkadot
RUN echo "Using polkadot ${POLKADOT_COMMIT}"
WORKDIR /

# Install OS dependencies
RUN apt-get update && \
	apt-get upgrade -y && \
	apt-get install -y cmake pkg-config libssl-dev git clang

# Grab the Polkadot Code
# TODO how to grab the correct commit from the lock file?
RUN git clone ${POLKADOT_REPO}
WORKDIR /polkadot
RUN git checkout ${POLKADOT_COMMIT}

# Forces to use the compiled wasm engine for parachain validation
RUN sed -i '/sc_executor::WasmExecutionMethod::Interpreted/c\\t\tsc_executor::WasmExecutionMethod::Compiled,' parachain/src/wasm_executor/mod.rs

# Download rust dependencies and build the rust binary
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
	export PATH=$PATH:$HOME/.cargo/bin && \
	cargo build --$PROFILE --features=real-overseer

# ===== SECOND STAGE ======

FROM phusion/baseimage:0.11
LABEL maintainer "alan@purestake.com"
LABEL description="Polkadot for Moonbeam Relay Chains"
ARG PROFILE=release
COPY --from=builder /polkadot/target/$PROFILE/polkadot /usr/local/bin

RUN mv /usr/share/ca* /tmp && \
	rm -rf /usr/share/*  && \
	mv /tmp/ca-certificates /usr/share/ && \
	rm -rf /usr/lib/python* && \
	useradd -m -u 1000 -U -s /bin/sh -d /moonbase-alphanet moonbeam && \
	mkdir -p /moonbase-alphanet/.local/share/moonbase-alphanet && \
	chown -R moonbeam:moonbeam /moonbase-alphanet && \
	ln -s /moonbase-alphanet/.local/share/moonbase-alphanet /data && \
	rm -rf /usr/bin /usr/sbin

USER moonbeam

COPY --chown=moonbeam specs/stagenet/rococo-embedded-specs-v6.json /moonbase-alphanet/stagenet-relay-raw-specs.json
COPY --chown=moonbeam specs/alphanet/rococo-embedded-specs-v6.json /moonbase-alphanet/alphanet-relay-raw-specs.json
RUN grep -v '/p2p/' /moonbase-alphanet/stagenet-relay-raw-specs.json > \
    /moonbase-alphanet/stagenet-relay-raw-specs-no-bootnodes.json && \
	grep -v '/p2p/' /moonbase-alphanet/alphanet-relay-raw-specs.json > \
    /moonbase-alphanet/alphanet-relay-raw-specs-no-bootnodes.json

# 30333 for p2p traffic
# 9933 for RPC call
# 9944 for Websocket
# 9615 for Prometheus (metrics)
EXPOSE 30333 9933 9944 9615

VOLUME ["/data"]

CMD ["/usr/local/bin/polkadot"]
