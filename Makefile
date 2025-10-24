ZOMBINET_VERSION := v1.3.133
POLKADOT_VERSION := stable2506-1
BRIDGE_RELAY_VERSION := v1.8.7

ZOMBINET_DOWNLOAD_URL := https://github.com/paritytech/zombienet/releases/download/${ZOMBINET_VERSION}
POLKADOT_DOWNLOAD_URL := https://github.com/paritytech/polkadot-sdk/releases/download/polkadot-${POLKADOT_VERSION}
BRIDGE_RELAY_DOWNLOAD_URL := https://github.com/moonbeam-foundation/parity-bridges-common/releases/download/${BRIDGE_RELAY_VERSION}

ZOMBINET_PATHS := ${PATH}:${PWD}/zombienet/bin

touch_done=@mkdir -p $(@D) && touch $@;

BRIDGE_RELAY_BIN := substrate-relay
ZOMBIENET_BIN := zombienet
POLKADOT_BIN := polkadot
POLKADOT_EXECUTE_WORKER_BIN := polkadot-execute-worker
POLKADOT_PREPARE_WORKER_BIN := polkadot-prepare-worker
MOONBEAM_RELEASE_BIN := target/release/moonbeam

UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Linux)
	ZOMBIENET_BIN_POSTFIX := -linux
	BRIDGE_RELAY_BIN_POSTFIX := -linux
endif
ifeq ($(UNAME_S),Darwin)
	ZOMBIENET_BIN_POSTFIX := -macos
	BRIDGE_RELAY_BIN_POSTFIX := -macos
	POLKADOT_BIN_POSTFIX := -aarch64-apple-darwin
endif
UNAME_P := $(shell uname -p)
ifeq ($(UNAME_P),x86_64)
	ZOMBIENET_BIN_POSTFIX := ${ZOMBIENET_BIN_POSTFIX}-x64
	BRIDGE_RELAY_BIN_POSTFIX := ${BRIDGE_RELAY_BIN_POSTFIX}-x64
endif
ifneq ($(filter arm%,$(UNAME_P)),)
	ZOMBIENET_BIN_POSTFIX := ${ZOMBIENET_BIN_POSTFIX}-arm64
	BRIDGE_RELAY_BIN_POSTFIX := ${BRIDGE_RELAY_BIN_POSTFIX}-arm64
endif

BINARIES := $(ZOMBIENET_BIN) $(BRIDGE_RELAY_BIN) $(POLKADOT_BIN) $(POLKADOT_EXECUTE_WORKER_BIN) $(POLKADOT_PREPARE_WORKER_BIN)

all: setup-moonbeam download-binaries

setup-moonbeam: zombienet/bin/moonbeam

download-binaries: $(BINARIES:%=zombienet/bin/%)

zombienet/bin/${BRIDGE_RELAY_BIN}:
	@echo "Downloading ${BRIDGE_RELAY_DOWNLOAD_URL}/${BRIDGE_RELAY_BIN}${BRIDGE_RELAY_BIN_POSTFIX}"
	@curl -L -o "zombienet/bin/${BRIDGE_RELAY_BIN}" "${BRIDGE_RELAY_DOWNLOAD_URL}/${BRIDGE_RELAY_BIN}${BRIDGE_RELAY_BIN_POSTFIX}"
	@chmod +x "zombienet/bin/${BRIDGE_RELAY_BIN}"

zombienet/bin/${ZOMBIENET_BIN}:
	@echo "Downloading ${ZOMBINET_DOWNLOAD_URL}/${ZOMBIENET_BIN}${ZOMBIENET_BIN_POSTFIX}"
	@curl -L -o "zombienet/bin/${ZOMBIENET_BIN}" "${ZOMBINET_DOWNLOAD_URL}/${ZOMBIENET_BIN}${ZOMBIENET_BIN_POSTFIX}"
	@chmod +x "zombienet/bin/${ZOMBIENET_BIN}"

zombienet/bin/${POLKADOT_BIN}:
	@echo "Downloading ${POLKADOT_DOWNLOAD_URL}/${POLKADOT_BIN}${POLKADOT_BIN_POSTFIX}"
	@curl -L -o "zombienet/bin/${POLKADOT_BIN}$*" "${POLKADOT_DOWNLOAD_URL}/${POLKADOT_BIN}${POLKADOT_BIN_POSTFIX}"
	@chmod +x "zombienet/bin/${POLKADOT_BIN}"

zombienet/bin/${POLKADOT_BIN}%:
	@echo "Downloading ${POLKADOT_DOWNLOAD_URL}/${POLKADOT_BIN}$*${POLKADOT_BIN_POSTFIX}"
	@curl -L -o "zombienet/bin/${POLKADOT_BIN}$*" "${POLKADOT_DOWNLOAD_URL}/${POLKADOT_BIN}$*${POLKADOT_BIN_POSTFIX}"
	@chmod +x "zombienet/bin/${POLKADOT_BIN}$*"

zombienet/bin/moonbeam:
	@if [ ! -L "$@" ]; then \
  		echo "Creating symlink: $@ -> ${MOONBEAM_RELEASE_BIN}"; \
  		ln -s "../../${MOONBEAM_RELEASE_BIN}" "$@"; \
    fi
	@if [ ! -e "$@" ]; then \
		echo "Broken symlink detected, fixing..."; \
		$(MAKE) release-build; \
	fi

release-build:
	@cargo build --release

# Zombienet commands

export PATH = $(ZOMBINET_PATHS)
start-zombienet-moonbeam: all
	@zombienet/bin/${ZOMBIENET_BIN} spawn zombienet/configs/moonbeam-polkadot.toml

export PATH = $(ZOMBINET_PATHS)
start-zombienet-moonriver: all
	@zombienet/bin/${ZOMBIENET_BIN} spawn zombienet/configs/moonriver-kusama.toml

run-bridge-integration-tests: all
	@./zombienet/integration-tests/bridges/run-test.sh 0001-moonbeam-moonriver-asset-transfer

# Lazy loading commands

export RPC := https://trace.api.moonbeam.network
export RUNTIME := moonbeam
start-lazy-loading-node: all
	@if [ ! -e ".lazy-loading/wasm-runtime-overrides" ]; then \
  		echo "Setup Lazy loading runtime overrides..."; \
  		mkdir -p .lazy-loading/wasm-runtime-overrides; \
  		cp target/release/wbuild/${RUNTIME}-runtime/${RUNTIME}_runtime.wasm .lazy-loading/wasm-runtime-overrides/${RUNTIME}_runtime.wasm; \
	fi
	@if [ ! -e ".lazy-loading/state-overrides.json" ]; then \
  		echo "Generating default lazy loading config..."; \
		echo "[]" > .lazy-loading/state-overrides.json; \
	fi
	@echo "Starting Moonbeam node in lazy loading mode...";
	@${MOONBEAM_RELEASE_BIN} \
		--lazy-loading-remote-rpc ${RPC} \
		--lazy-loading-state-overrides .lazy-loading/state-overrides.json \
		--lazy-loading-runtime-override target/release/wbuild/${RUNTIME}-runtime/${RUNTIME}_runtime.wasm \
		-ltracing=trace,evm=trace,lazy-loading=trace,rpc=trace \
		--alice \
		--no-grandpa \
		--reserved-only \
		--sealing 6000 \
		--unsafe-rpc-external \
		--rpc-external \
		--rpc-cors all \
		--rpc-methods Unsafe \
		--ethapi=txpool,debug,trace \
		--force-authoring \
		--wasm-runtime-overrides .lazy-loading/wasm-runtime-overrides