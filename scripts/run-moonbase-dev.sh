#!/bin/bash

# User port XX000
# Standalone port XX500
# 42 for p2p
# 43 for http
# 44 for ws
#
# Ex: USER_PORT=20000 scripts/run-moonbase-standalone.sh
# will open port 21542, 21543, 21544

# Loading binary/specs variables
source scripts/_init_var.sh

if [ ! -f "$MOONBEAM_BINARY" ]; then
  echo "Moonbeam binary $MOONBEAM_BINARY is missing"
  echo "Please run: cargo build --release"
  exit 1
fi

STANDALONE_PORT=$((USER_PORT + 500 + 42))
STANDALONE_INDEX=0
STANDALONE_BOOTNODES_ARGS=""
while nc -z -v -w5 ${RELAY_IP} ${STANDALONE_PORT} 2> /dev/null
do
  echo "Found existing relay on ${STANDALONE_PORT}."
  BOOTNODES_ARGS="$BOOTNODES_ARGS --bootnodes \
    /ip4/$RELAY_IP/tcp/${STANDALONE_PORT}/p2p/${COMMON_LOCAL_IDS[$STANDALONE_INDEX]}"
  STANDALONE_INDEX=$((STANDALONE_INDEX + 1))
  STANDALONE_PORT=$((STANDALONE_PORT + 100))

  if [ $STANDALONE_PORT -ge $((USER_PORT + 800)) ]
  then
    echo "No more standalone port available! (limited to 3 standalone nodes)"
    exit 1
  fi
done

echo "Node $STANDALONE_INDEX - p2p-port: $((STANDALONE_PORT)), \
http-port: $((STANDALONE_PORT + 1)) , ws-port: $((STANDALONE_PORT + 2))"

if [ -z "$BASE_PREFIX" ]; then
  BASE_PATH="--tmp"
else
  BASE_PATH="$BASE_PREFIX-relay-$STANDALONE_INDEX"
fi

EXECUTABLE=$MOONBEAM_BINARY
if [ ! -z "$PERF" ]; then
  EXECUTABLE="$PERF $MOONBEAM_BINARY"
fi

$EXECUTABLE \
  --node-key ${COMMON_NODE_KEYS[$STANDALONE_INDEX]} \
  --dev \
  --tmp \
  --port $((STANDALONE_PORT)) \
  --rpc-port $((STANDALONE_PORT + 1)) \
  --ws-port $((STANDALONE_PORT + 2)) \
  --validator \
  --author-id 6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b \
  --rpc-cors all \
  --rpc-methods=unsafe \
  --execution native \
  --name STANDALONE_$STANDALONE_INDEX \
  $STANDALONE_BASE_PATH \
  '-linfo,evm=debug,ethereum=trace,rpc=trace,cumulus_collator=debug,txpool=debug' \
  $STANDALONE_BOOTNODES_ARGS
