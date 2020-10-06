#!/bin/bash

source scripts/_init_var.sh

RELAY_PORT=$((USER_PORT + 42))
RELAY_INDEX=0
BOOTNODES_ARGS=""      


if [ -z "$SUDO_SEED" ]; then
    echo "Missing \$SUDO_SEED"
    exit 1
fi

polkadot-js-api \
    --ws "ws://localhost:$((RELAY_PORT + 2))" \
    --sudo \
    --seed "$SUDO_SEED" \
    tx.registrar.registerPara \
        1000 \
        "{\"scheduling\":\"Always\"}" \
        @"$PARACHAIN_WASM" \
        "$(cat $PARACHAIN_GENESIS)"
