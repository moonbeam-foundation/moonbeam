# Node for Moonbase Parachains.
#
# Requires to run from repository root and to copy the binary in the build folder (part of the release workflow)

FROM node:16
LABEL maintainer "alan@purestake.com"
LABEL description="Moonbeam Fork Test image"

RUN apt update
RUN apt install -y jq

RUN mkdir /moonbeam-fork-tests && \
    chown -R node:node /moonbeam-fork-tests

RUN apt install -y git clang curl libssl-dev llvm libudev-dev pkg-config make

USER node

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/home/node/.cargo/bin:${PATH}"

RUN rustup default stable && \
	rustup update && \
	rustup update nightly && \
	rustup target add wasm32-unknown-unknown --toolchain nightly

COPY --chown=node ./run-fork-test.sh /moonbeam-fork-tests/run-fork-test.sh
RUN chmod uog+x /moonbeam-fork-tests/run-fork-test.sh

ENV ROOT_FOLDER /moonbeam-fork-tests
CMD ["/moonbeam-fork-tests/run-fork-test.sh"]
