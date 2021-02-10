#!/bin/bash

source scripts/_init_var.sh

RELAY_PORT=$((USER_PORT + 42))
RELAY_INDEX=0
BOOTNODES_ARGS=""


if [ -z "$SUDO_SEED" ]; then
    echo "Missing \$SUDO_SEED"
    exit 1
fi

if [ ! -f "$ALPHANET_WASM" ]; then
    echo "Missing $ALPHANET_WASM. Please run scripts/generate-parachain-specs.sh"
    exit 1
fi

if [ ! -f "$ALPHANET_GENESIS" ]; then
    echo "Missing $ALPHANET_GENESIS. Please run scripts/generate-parachain-specs.sh"
    exit 1
fi

ALPHANET_CONFIG="$PARACHAIN_BUILD_FOLDER/moonbase-alphanet-runtime.config.json";
TYPES="$PARACHAIN_BUILD_FOLDER/relay-types.json"
echo -n "1000 {\"genesis_head\":\"$(cat $ALPHANET_GENESIS)\",\"validation_code\":\"" \
    > $ALPHANET_CONFIG;
cat $ALPHANET_WASM  >> $ALPHANET_CONFIG;
echo -n "\",\"parachain\":true}" >> $ALPHANET_CONFIG;

echo '{"Address": "MultiAddress", "LookupSource": "MultiAddress"}' > $TYPES;

docker run --rm --network=host \
  -v $(pwd)/$ALPHANET_CONFIG:/config \
  -v $(pwd)/$TYPES:/types \
  jacogr/polkadot-js-tools:latest api \
    --ws "ws://localhost:$((RELAY_PORT + 2))" \
    --types /types \
    --sudo \
    --seed "$SUDO_SEED" \
    --params /config \
    tx.parasSudoWrapper.sudoScheduleParaInitialize
