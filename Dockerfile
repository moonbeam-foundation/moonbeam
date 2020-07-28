# Note: This is currently designed to simplify development
# Future versions will include 2nd stage with smaller image

FROM rustlang/rust:nightly


ARG PROFILE=release
WORKDIR /moonbeam

# Upcd dates core parts
RUN apt-get update -y && \
	apt-get install -y cmake pkg-config libssl-dev git gcc build-essential clang libclang-dev

# Install rust wasm. Needed for substrate wasm engine
RUN rustup target add wasm32-unknown-unknown

# Download Moonbeam repo
RUN git clone -b moonbeam-tutorials https://github.com/PureStake/moonbeam /moonbeam
RUN cd /moonbeam && git submodule update --init --recursive

# Download rust dependencies and build the rust binary
RUN cargo build "--$PROFILE"

# 30333 for p2p traffic
# 9933 for RPC call
# 9944 for Websocket
# 9615 for Prometheus (metrics)
EXPOSE 30333 9933 9944 9615


ENV PROFILE ${PROFILE}

# The execution will re-compile the project to run it
# This allows to modify the code and not have to re-compile the
# dependencies.
CMD cargo run "--$PROFILE" -- --dev --ws-external
