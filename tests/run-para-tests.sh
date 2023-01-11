#!/bin/bash

POLKADOT_REPO=${POLKADOT_REPO:-$(egrep -o 'https.*/polkadot' ../Cargo.lock | head -1)}
POLKADOT_COMMIT=${POLKADOT_COMMIT:-$(egrep -o '/polkadot.*#([^\"]*)' ../Cargo.lock | head -1 | sed 's/.*#//' |  cut -c1-8)}
DOCKER_TAG="purestake/moonbase-relay-testnet:sha-$POLKADOT_COMMIT"

mkdir -p binaries
export RELAY_BINARY_PATH="binaries/polkadot-$POLKADOT_COMMIT"

if [[ ! -e $RELAY_BINARY_PATH ]]; then
    echo "missing $RELAY_BINARY_PATH - downloading..."
    docker rm -f dummy 2> /dev/null
    docker create -ti --name dummy $DOCKER_TAG bash
    docker cp dummy:/usr/local/bin/polkadot $RELAY_BINARY_PATH
    docker rm -f dummy
fi


export BINARY_PATH=${BINARY_PATH:-"../target/release/moonbeam"}
export FORCE_COMPILED_WASM=true

node node_modules/.bin/mocha -r ts-node/register 'para-tests/**/test-*.ts'