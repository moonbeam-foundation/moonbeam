#!/bin/bash

# User port X0000
# 1xxx for each type (relay vs parachain)
# 1xx for each node
# 42 for p2p
# 43 for http
# 44 for ws
#
# Ex: USER_PORT=20000 scripts/run-parachain.sh
# Will open port 21042, 21043, 21044

# Loading binary/specs variables
source scripts/_init_var.sh


RELAY_PORT=$((USER_PORT + 42))
RELAY_INDEX=0
BOOTNODES_ARGS=""
while nc -z -v -w5 ${RELAY_IP} ${RELAY_PORT} 2> /dev/null
do 
    echo "Found existing relay on ${RELAY_PORT}."
    BOOTNODES_ARGS="$BOOTNODES_ARGS --bootnodes /ip4/$RELAY_IP/tcp/${RELAY_PORT}/p2p/${RELAY_LOCAL_IDS[$RELAY_INDEX]}"
    RELAY_INDEX=$((RELAY_INDEX + 1))
    RELAY_PORT=$((RELAY_PORT + 100))
    
    if [ $RELAY_PORT -ge $((USER_PORT + 1000)) ]
    then
        echo "No more relay port available! (limited to 9 relays)"
        exit 1
    fi
done


echo "relay ${RELAY_INDEX} - p2p-port: $((RELAY_PORT)), http-port: $((RELAY_PORT + 1)) , ws-port: $((RELAY_PORT + 2))"

# The -v build:/build allows to pass the spec files from the build folder to the docker container
docker run \
    -v $(pwd)/build:/build \
    -p $RELAY_PORT:$RELAY_PORT \
    -p $((RELAY_PORT + 1)):$((RELAY_PORT + 1)) \
    -p $((RELAY_PORT + 2)):$((RELAY_PORT + 2)) \
    -it purestake/moonbase-relay-testnet:latest \
    /usr/local/bin/polkadot \
        --chain /$POLKADOT_SPEC_RAW \
        --node-key ${RELAY_KEYS[$RELAY_INDEX]} \
        --tmp \
        --rpc-external \
        --ws-external \
        --port $((RELAY_PORT)) \
        --rpc-port $((RELAY_PORT + 1)) \
        --ws-port $((RELAY_PORT + 2)) \
        $BOOTNODES_ARGS \
        '-linfo,evm=trace,ethereum=trace,rpc=trace'