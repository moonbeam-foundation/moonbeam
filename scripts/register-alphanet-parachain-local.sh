#!/bin/bash

source scripts/_init_var.sh

RELAY_PORT=$((USER_PORT + 42))
RELAY_INDEX=0
BOOTNODES_ARGS=""


if [ -z "$SUDO_SEED" ]; then
    echo "Missing \$SUDO_SEED"
    exit 1
fi

if [ ! -f "$PARACHAIN_WASM" ]; then
    echo "Missing $PARACHAIN_WASM. Please run scripts/generate-parachain-specs.sh"
    exit 1
fi

if [ ! -f "$PARACHAIN_GENESIS" ]; then
    echo "Missing $PARACHAIN_GENESIS. Please run scripts/generate-parachain-specs.sh"
    exit 1
fi

PARACHAIN_CONFIG="$PARACHAIN_BUILD_FOLDER/moonbase-alphanet-runtime.config.json";
echo -n "1000 {\"genesis_head\":\"$(cat $PARACHAIN_GENESIS)\",\"validation_code\":\"" \
    > $PARACHAIN_CONFIG;
cat $PARACHAIN_WASM  >> $PARACHAIN_CONFIG;
echo -n "\",\"parachain\":true}" >> $PARACHAIN_CONFIG;

tools/node_modules/.bin/polkadot-js-api \
    --ws "ws://localhost:$((RELAY_PORT + 2))" \
    --sudo \
    --seed "$SUDO_SEED" \
    --params $(pwd)/$PARACHAIN_CONFIG \
    tx.parasSudoWrapper.sudoScheduleParaInitialize
