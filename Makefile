ZOMBINET_VERSION := v1.3.121
POLKADOT_VERSION := stable2503-5

ZOMBINET_DOWNLOAD_URL := https://github.com/paritytech/zombienet/releases/download/${ZOMBINET_VERSION}
POLKADOT_DOWNLOAD_URL := https://github.com/paritytech/polkadot-sdk/releases/download/polkadot-${POLKADOT_VERSION}

UNAME_S := $(shell uname -s)

ZOMBINET_PATHS := ${PATH}:${PWD}/zombienet/bin

touch_done=@mkdir -p $(@D) && touch $@;

ZOMBIENET_BIN := zombienet
POLKADOT_BIN := polkadot
POLKADOT_EXECUTE_WORKER_BIN := polkadot-execute-worker
POLKADOT_PREPARE_WORKER_BIN := polkadot-prepare-worker
MOONBEAM_RELEASE_BIN := target/release/moonbeam

UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Linux)
	ZOMBIENET_BIN := ${ZOMBIENET_BIN}-linux
endif
ifeq ($(UNAME_S),Darwin)
	ZOMBIENET_BIN := ${ZOMBIENET_BIN}-macos
	POLKADOT_BIN_POSTFIX := -aarch64-apple-darwin
endif
UNAME_P := $(shell uname -p)
ifeq ($(UNAME_P),x86_64)
	ZOMBIENET_BIN := ${ZOMBIENET_BIN}-x64
endif
ifneq ($(filter arm%,$(UNAME_P)),)
	ZOMBIENET_BIN := ${ZOMBIENET_BIN}-arm64
endif

BINARIES := $(ZOMBIENET_BIN) $(POLKADOT_BIN) $(POLKADOT_EXECUTE_WORKER_BIN) $(POLKADOT_PREPARE_WORKER_BIN)

all: setup-moonbeam download-binaries

setup-moonbeam: zombienet/bin/moonbeam

download-binaries: $(BINARIES:%=zombienet/bin/%)

zombienet/bin/zombienet-%:
	@echo "Downloading ${ZOMBINET_DOWNLOAD_URL}/zombienet-$*"
	@curl -L -o zombienet/bin/zombienet-$* ${ZOMBINET_DOWNLOAD_URL}/zombienet-$*
	@chmod +x zombienet/bin/zombienet-$*

zombienet/bin/polkadot zombienet/bin/polkadot%:
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
	@zombienet/bin/${ZOMBIENET_BIN} spawn zombienet/moonbeam-polkadot.toml

export PATH = $(ZOMBINET_PATHS)
start-zombienet-moonriver: all
	@zombienet/bin/${ZOMBIENET_BIN} spawn zombienet/moonriver-kusama.toml