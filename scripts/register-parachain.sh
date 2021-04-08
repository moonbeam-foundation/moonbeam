#!/bin/bash

source scripts/_init_var.sh

RELAY_PORT=$((USER_PORT + 42))
RELAY_INDEX=0
BOOTNODES_ARGS=""

# Will retrieve variable from the given network
NETWORK=${NETWORK:-"alphanet"}
WASM=$(eval echo "\$${NETWORK^^}_WASM")
GENESIS=$(eval echo "\$${NETWORK^^}_GENESIS")
TMP_FOLDER=$(eval echo "\$${NETWORK^^}_TMP_FOLDER")
PARACHAIN_ID=$(eval echo "\$${NETWORK^^}_PARACHAIN_ID")

if [ -z "$ROCOCO_SUDO_SEED" ]; then
    echo "Missing \$ROCOCO_SUDO_SEED"
    exit 1
fi

if [ ! -f "$WASM" ]; then
    echo "Missing $WASM. Please run scripts/generate-parachain-specs.sh"
    exit 1
fi

if [ ! -f "$GENESIS" ]; then
    echo "Missing $GENESIS. Please run scripts/generate-parachain-specs.sh"
    exit 1
fi

sha256sum $GENESIS $WASM

# Registering using regular register is taking longer.
# Keep the code for reference.

# CONFIG="$TMP_FOLDER/moonbase-alphanet-runtime.config.json";
# echo -n "$PARACHAIN_ID $(cat $GENESIS) $(cat $WASM)" > $CONFIG;

# TYPES="$TMP_FOLDER/relay-types.json"
# echo '{"Address": "MultiAddress", "LookupSource": "MultiAddress"}' > $TYPES;

# docker run --rm --network=host \
#   -v $(pwd)/$CONFIG:/config \
#   -v $(pwd)/$TYPES:/types \
#   jacogr/polkadot-js-tools:latest api \
#     --ws "ws://localhost:$((RELAY_PORT + 2))" \
#     --types /types \
#     --params /config \
#     --seed "$ROCOCO_SUDO_SEED" \
#     tx.registrar.register 


CONFIG="$TMP_FOLDER/moonbase-alphanet-runtime.config.json";
echo -n "$PARACHAIN_ID {\"genesis_head\":\"$(cat $GENESIS)\",\"validation_code\":\"" \
    > $CONFIG;
cat $WASM  >> $CONFIG;
echo -n "\",\"parachain\":true}" >> $CONFIG;

TYPES="$TMP_FOLDER/relay-types.json"
echo '{"Address": "MultiAddress", "LookupSource": "MultiAddress"}' > $TYPES;

docker run --rm --network=host \
  -v $(pwd)/$CONFIG:/config \
  -v $(pwd)/$TYPES:/types \
  jacogr/polkadot-js-tools:latest api \
    --ws "ws://localhost:$((RELAY_PORT + 2))" \
    --types /types \
    --sudo \
    --seed "$ROCOCO_SUDO_SEED" \
    --params /config \
    tx.parasSudoWrapper.sudoScheduleParaInitialize

paras_registrar::register
echo 'Registration done'

docker run --rm --network=host \
  -v $(pwd)/$CONFIG:/config \
  -v $(pwd)/$TYPES:/types \
  jacogr/polkadot-js-tools:latest api \
    --ws "ws://localhost:$((RELAY_PORT + 2))" \
    --types /types \
    --sudo \
    --seed "$ROCOCO_SUDO_SEED" \
    tx.slots.forceLease $PARACHAIN_ID $ROCOCO_SUDO_ADDR 0 0 10000

echo 'Force lease (10000 slots) done'