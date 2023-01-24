#!/bin/bash

# This script is expected to be included in a docker image (with node)
set -e 

export RUNTIME_NAME=${RUNTIME_NAME:-"moonbeam"}
export NETWORK=${NETWORK:-"moonbeam"} #moonbase-alpha for alphanet
export PORT_PREFIX=${PORT_PREFIX:-"51"}
export ROOT_FOLDER=${ROOT_FOLDER:-"/data"}
export GIT_TAG=${GIT_TAG:-"master"}
export SKIP_INTERMEDIATE_RUNTIME=${SKIP_INTERMEDIATE_RUNTIME:-false}
export FORCE_COMPILED_WASM=${FORCE_COMPILED_WASM:-true}
export SINGLE_PARACHAIN_NODE=${SINGLE_PARACHAIN_NODE:-true}
export SKIP_DOWNLOAD=${SKIP_DOWNLOAD:-false}
export SKIP_COMPILATION=${SKIP_COMPILATION:-false}
export SKIP_STATE_MODIFICATION=${SKIP_STATE_MODIFICATION:-false}
export USE_LOCAL_CLIENT=${USE_LOCAL_CLIENT:-false}

export BINARY_PATH=${BINARY_PATH:-$ROOT_FOLDER/moonbeam/binaries/moonbeam};
export RELAY_BINARY_PATH=${RELAY_BINARY_PATH:-$ROOT_FOLDER/moonbeam/binaries/polkadot};
export SPEC_FILE=$ROOT_FOLDER/states/${NETWORK}-state.mod.json
export NODE_OPTIONS=--max-old-space-size=16000

echo "=========== Variables ==========="
echo "RUNTIME_NAME: ${RUNTIME_NAME}"
echo "NETWORK: ${NETWORK}"
echo "PORT_PREFIX: ${PORT_PREFIX}"
echo "ROOT_FOLDER: ${ROOT_FOLDER}"
echo "GIT_TAG: ${GIT_TAG}"
echo "SKIP_INTERMEDIATE_RUNTIME: ${SKIP_INTERMEDIATE_RUNTIME}"
echo "FORCE_COMPILED_WASM: ${FORCE_COMPILED_WASM}"
echo "SINGLE_PARACHAIN_NODE: ${SINGLE_PARACHAIN_NODE}"
echo "SKIP_DOWNLOAD: ${SKIP_DOWNLOAD}"
echo "SKIP_COMPILATION: ${SKIP_COMPILATION}"
echo "SKIP_STATE_MODIFICATION: ${SKIP_STATE_MODIFICATION}"
echo "USE_LOCAL_CLIENT: ${USE_LOCAL_CLIENT}"
echo "BINARY_PATH: ${BINARY_PATH}"
echo "RELAY_BINARY_PATH: ${RELAY_BINARY_PATH}"
echo "SPEC_FILE: ${SPEC_FILE}"
echo "NODE_OPTIONS: ${NODE_OPTIONS}"
echo "================================"

if [[ $PARA_ID == "" ]]; then
    if [[ $NETWORK == "moonbeam" ]]; then
        export PARA_ID=2004
    elif [[ $NETWORK == "moonriver" ]]; then
        export PARA_ID=2023
    elif [[ $NETWORK == "moonbase-alpha" ]]; then
        export PARA_ID=1000
    else
        export PARA_ID=1000
    fi
fi

echo "Preparation..."
echo " - moonbeam: ${GIT_TAG} [folder: ${ROOT_FOLDER} - port-prefix: ${PORT_PREFIX}]"
echo " -  network: ${NETWORK} [runtime: ${RUNTIME_NAME} - id: ${PARA_ID}]"
trap "trap - TERM && kill -- -$$" INT TERM EXIT

mkdir -p $ROOT_FOLDER/states
cd $ROOT_FOLDER

if [[ $SKIP_DOWNLOAD != true ]]
then
    # Clone moonbeam repo & building
    echo "Cloning repository..."
    git clone https://github.com/purestake/moonbeam
    cd $ROOT_FOLDER/moonbeam
    git checkout $GIT_TAG
    mkdir binaries

    echo "Retrieving binaries..."
    MOONBEAM_CLIENT_TAG=$(curl -s https://api.github.com/repos/purestake/moonbeam/releases | jq -r '.[] | select(.assets|.[]|.name|test("\\bmoonbeam\\b")) | .tag_name' | grep '^v' | head -1)
    POLKADOT_CLIENT_TAG=$(curl -s https://api.github.com/repos/paritytech/polkadot/releases | jq -r '.[] | select(.assets|.[]|.name|test("\\bpolkadot\\b")) | .tag_name' | grep '^v' | head -1)

    if [[ ! -f $BINARY_PATH && $USE_LOCAL_CLIENT != "true" ]]
    then
        echo "Downloading moonbeam ${MOONBEAM_CLIENT_TAG}"
        wget -q https://github.com/PureStake/moonbeam/releases/download/${MOONBEAM_CLIENT_TAG}/moonbeam \
            -O $BINARY_PATH
        chmod uog+x $BINARY_PATH
    fi

    if [[ ! -f $RELAY_BINARY_PATH ]]
    then
        echo "Downloading polkadot ${POLKADOT_CLIENT_TAG}"
        wget -q https://github.com/paritytech/polkadot/releases/download/${POLKADOT_CLIENT_TAG}/polkadot \
            -O $RELAY_BINARY_PATH
        chmod uog+x $RELAY_BINARY_PATH
    fi

    echo "Retrieving ${NETWORK} state... (few minutes)"
    wget -q https://s3.us-east-2.amazonaws.com/snapshots.moonbeam.network/${NETWORK}/latest/${NETWORK}-state.json \
        -O $ROOT_FOLDER/states/${NETWORK}-state.json; 
fi

if [[ $SKIP_COMPILATION != true ]]
then
    ## Build the runtime to test
    echo "Building $GIT_TAG $RUNTIME_NAME runtime... (5 minutes)"
    cd $ROOT_FOLDER/moonbeam
    git checkout $GIT_TAG
    cargo build --release -p ${RUNTIME_NAME}-runtime

    if [[ $USE_LOCAL_CLIENT == "true" ]]
    then
        cargo build --release -p moonbeam
        cp target/release/moonbeam $BINARY_PATH
    fi
fi

echo "Preparing tests... (3 minutes)"
cd $ROOT_FOLDER/moonbeam/moonbeam-types-bundle
npm install
cd $ROOT_FOLDER/moonbeam/tools
npm install

cd $ROOT_FOLDER/moonbeam/tests
npm ci


echo " - moonbeam binary: $BINARY_PATH"
echo "   - $($BINARY_PATH --version)"
echo " - polkadot binary: $RELAY_BINARY_PATH"
echo "   - $($RELAY_BINARY_PATH --version)"

if [[ $SKIP_STATE_MODIFICATION != true ]]
then
    # Modify state
    cd $ROOT_FOLDER/moonbeam/tests
    echo "Customizing $NETWORK forked state..."
    ./node_modules/.bin/ts-node state-modifier.ts $ROOT_FOLDER/states/${NETWORK}-state.json
fi

# Run the node
echo "Running nodes..."
cd $ROOT_FOLDER/moonbeam/tests
node_modules/.bin/ts-node spawn-fork-node.ts 2>&1 > spawn-node.log
