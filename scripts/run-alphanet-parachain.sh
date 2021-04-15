#!/bin/bash

# User port XX000
# Relay port XX000
# 1xx for each node
# 42 for p2p
# 43 for http
# 44 for ws
#
# Parachain port (XX+1)000
# 52 for p2p
# 53 for http
# 54 for ws
#
# Ex: USER_PORT=20000 scripts/run-alphanet-parachain.sh
# will open port 21052, 21053, 21054

# The parachain will run on rococo-local relay

# Loading binary/specs variables
source scripts/_init_var.sh

if [ ! -f "$MOONBEAM_BINARY" ]; then
  echo "Moonbeam binary $MOONBEAM_BINARY is missing"
  echo "Please run: cargo build --release"
  exit 1
fi

# Will retrieve variable from the given network
NETWORK=${NETWORK:-"alphanet"}
PARACHAIN_ID=$(eval echo "\$${NETWORK^^}_PARACHAIN_ID")
STAKERS=($(eval echo "\${${NETWORK^^}_STAKERS[@]}"))

if [ -z "$CHAIN" ]; then
  CHAIN=$(eval echo "\$${NETWORK^^}_PARACHAIN_SPEC_RAW")
fi

# We retrieve the list of relay node for
RELAY_PORT=$((USER_PORT + 42))
RELAY_INDEX=0
RELAY_BOOTNODES_ARGS=""

while nc -z -v -w5 ${RELAY_IP} ${RELAY_PORT} 2> /dev/null
do
  echo "Found existing relay on ${RELAY_PORT}."
  RELAY_BOOTNODES_ARGS="$RELAY_BOOTNODES_ARGS \
    --bootnodes /ip4/$RELAY_IP/tcp/${RELAY_PORT}/p2p/${COMMON_LOCAL_IDS[$RELAY_INDEX]}"
  RELAY_INDEX=$((RELAY_INDEX + 1))
  RELAY_PORT=$((RELAY_PORT + 100))

  if [ $RELAY_PORT -ge $((USER_PORT + 1000)) ]
  then
    break
  fi
done


PARACHAIN_PORT=$((USER_PORT + 1000 + 42))
PARACHAIN_INDEX=0
PARACHAIN_BOOTNODES_ARGS=""
while nc -z -v -w5 ${PARACHAIN_IP} $((PARACHAIN_PORT + 10)) 2> /dev/null
do
  echo "Found existing parachain on $((PARACHAIN_PORT + 10))."
  PARACHAIN_BOOTNODES_ARGS="$PARACHAIN_BOOTNODES_ARGS --bootnodes \
    /ip4/$PARACHAIN_IP/tcp/$((PARACHAIN_PORT + 10))/p2p/${PARACHAIN_LOCAL_IDS[$PARACHAIN_INDEX]}"
  PARACHAIN_INDEX=$((PARACHAIN_INDEX + 1))
  PARACHAIN_PORT=$((PARACHAIN_PORT + 100))

  if [ $PARACHAIN_PORT -ge $((USER_PORT + 2000)) ]
  then
    echo "No more parachain port available! (limited to 9 parachains)"
    exit 1
  fi
done

if [ -z "$PARACHAIN_BASE_PREFIX" ]; then
  PARACHAIN_BASE_PATH="--tmp"
else
  PARACHAIN_BASE_PATH="$PARACHAIN_BASE_PREFIX-parachain-$PARACHAIN_INDEX"
fi

echo "parachain $PARACHAIN_INDEX ($PARACHAIN_ID) - p2p-port: $((PARACHAIN_PORT + 10)), \
http-port: $((PARACHAIN_PORT + 10 + 1)), ws-port: $((PARACHAIN_PORT + 10 + 2))"

sha256sum $CHAIN
$MOONBEAM_BINARY \
  --node-key ${PARACHAIN_NODE_KEYS[$PARACHAIN_INDEX]} \
  --port $((PARACHAIN_PORT + 10)) \
  --rpc-port $((PARACHAIN_PORT + 10 + 1)) \
  --ws-port $((PARACHAIN_PORT + 10 + 2)) \
  --collator \
  --rpc-cors all \
  --rpc-methods=unsafe \
  --execution wasm \
  --wasm-execution compiled \
  --name parachain_$PARACHAIN_INDEX \
  $PARACHAIN_BASE_PATH \
  '-linfo,evm=debug,ethereum=trace,rpc=trace,cumulus_collator=debug,txpool=debug' \
  --author-id ${STAKERS[$PARACHAIN_INDEX]} \
  --chain $CHAIN \
  $PARACHAIN_BOOTNODES_ARGS \
  -- \
    --node-key ${PARACHAIN_NODE_KEYS[$PARACHAIN_INDEX]} \
    $PARACHAIN_BASE_PATH \
    --port $((PARACHAIN_PORT)) \
    --rpc-port $((PARACHAIN_PORT + 1)) \
    --ws-port $((PARACHAIN_PORT + 2)) \
    --chain $ROCOCO_LOCAL_RAW_SPEC \
  $RELAY_BOOTNODES_ARGS;
