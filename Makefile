ZOMBINET_VERSION := v1.3.121
POLKADOT_VERSION := stable2503-5
BRIDGE_RELAY_VERSION := v1.8.4

ZOMBINET_DOWNLOAD_URL := https://github.com/paritytech/zombienet/releases/download/${ZOMBINET_VERSION}
POLKADOT_DOWNLOAD_URL := https://github.com/paritytech/polkadot-sdk/releases/download/polkadot-${POLKADOT_VERSION}
BRIDGE_RELAY_DOWNLOAD_URL := https://github.com/Moonsong-Labs/parity-bridges-common/releases/download/${BRIDGE_RELAY_VERSION}

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
endif
ifneq ($(filter arm%,$(UNAME_P)),)
	ZOMBIENET_BIN_POSTFIX := ${ZOMBIENET_BIN_POSTFIX}-arm64
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

zombienet/bin/${POLKADOT_BIN} zombienet/bin/polkadot%:
	@echo "Downloading ${POLKADOT_DOWNLOAD_URL}/polkadot$*${POLKADOT_BIN_POSTFIX}"
	@curl -L -o zombienet/bin/polkadot$* ${POLKADOT_DOWNLOAD_URL}/polkadot$*${POLKADOT_BIN_POSTFIX}
	@chmod +x zombienet/bin/polkadot$*

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

export PATH = $(ZOMBINET_PATHS)
start-zombienet-moonbeam: all
	@zombienet spawn zombienet/configs/moonbeam-polkadot.toml

export PATH = $(ZOMBINET_PATHS)
start-zombienet-moonriver: all
	@zombienet spawn zombienet/configs/moonriver-kusama.toml