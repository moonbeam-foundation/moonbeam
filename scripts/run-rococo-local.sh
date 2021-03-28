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
# Ex: USER_PORT=20000 scripts/run-rococo-local.sh
# will open port 20042, 20043, 20044

# Loading binary/specs variables
source scripts/_init_var.sh


RELAY_PORT=$((USER_PORT + 42))
RELAY_INDEX=0
BOOTNODES_ARGS=""
while nc -z -v -w5 ${RELAY_IP} ${RELAY_PORT} 2> /dev/null
do
    echo "Found existing relay on ${RELAY_PORT}."
    BOOTNODES_ARGS="$BOOTNODES_ARGS --bootnodes \
      /ip4/$RELAY_IP/tcp/${RELAY_PORT}/p2p/${COMMON_LOCAL_IDS[$RELAY_INDEX]}"
    RELAY_INDEX=$((RELAY_INDEX + 1))
    RELAY_PORT=$((RELAY_PORT + 100))

    if [ $RELAY_PORT -ge $((USER_PORT + 300)) ]
    then
        echo "No more relay port available! (limited to 3 relays)"
        exit 1
    fi
done

echo "relay ${RELAY_INDEX} - p2p-port: $((RELAY_PORT)), \
http-port: $((RELAY_PORT + 1)) , ws-port: $((RELAY_PORT + 2))"

if [ -z "$POLKADOT_VERSION" ]; then
  POLKADOT_VERSION="sha-`egrep -o '/polkadot.*#([^\"]*)' Cargo.lock | \
    head -1 | sed 's/.*#//' |  cut -c1-8`"
fi

echo "Using Polkadot revision #${POLKADOT_VERSION}"

# The -v build:/build allows to pass the spec files from the build folder to the docker container
docker run \
  -v $(pwd)/build:/build \
  -p $RELAY_PORT:$RELAY_PORT \
  -p $((RELAY_PORT + 1)):$((RELAY_PORT + 1)) \
  -p $((RELAY_PORT + 2)):$((RELAY_PORT + 2)) \
  -it purestake/moonbase-relay-testnet:$POLKADOT_VERSION \
    /usr/local/bin/polkadot \
      --chain rococo-local \
      --${WELL_KNOWN_USERS[$RELAY_INDEX]} \
      --node-key ${COMMON_NODE_KEYS[$RELAY_INDEX]} \
      --base-path /tmp \
      --validator \
      --force-authoring \
      --name relay_$RELAY_INDEX \
      --rpc-methods=Unsafe \
      --unsafe-rpc-external \
      --unsafe-ws-external \
      --rpc-cors all \
      --port $((RELAY_PORT)) \
      --rpc-port $((RELAY_PORT + 1)) \
      --ws-port $((RELAY_PORT + 2)) \
      $BOOTNODES_ARGS \
      '-linfo,evm=debug,ethereum=trace,rpc=trace,txpool=debug,validation-worker=debug'
