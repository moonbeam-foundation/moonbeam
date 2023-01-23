#!/bin/bash

export BASE_PATH=/tmp/fork-upgrade-test/
export RELAY_WSS_URL=ws://localhost:51002
export WSS_URL=ws://localhost:51102

if [[ `which jq` == "" ]]; then
    echo "Missing jq"
    exit 1
fi

# maybe add in version numbers

# export RUNTIME_NAME=${RUNTIME_NAME:-"moonbeam"}
export NETWORK=moonbeam
# export PORT_PREFIX=${PORT_PREFIX:-"51"}
# export ROOT_FOLDER=${ROOT_FOLDER:-"/data"}
# export GIT_TAG=${GIT_TAG:-"master"}
# export GIT_TEST_TAG=${GIT_TEST_TAG:-$GIT_TAG}
# export SKIP_INTERMEDIATE_RUNTIME=${SKIP_INTERMEDIATE_RUNTIME:-false}
# export FORCE_COMPILED_WASM=${FORCE_COMPILED_WASM:-true}
# export SINGLE_PARACHAIN_NODE=${SINGLE_PARACHAIN_NODE:-true}
# export SKIP_DOWNLOAD=${SKIP_DOWNLOAD:-false}
# export SKIP_COMPILATION=${SKIP_COMPILATION:-false}
# export SKIP_STATE_MODIFICATION=${SKIP_STATE_MODIFICATION:-false}
# export KEEP_RUNNING=${KEEP_RUNNING:-false}
# export USE_LOCAL_CLIENT=${USE_LOCAL_CLIENT:-false}
# export ROUNDS_TO_WAIT=${ROUNDS_TO_WAIT:-"2"}

# export BINARY_PATH=${BINARY_PATH:-$ROOT_FOLDER/moonbeam/binaries/moonbeam};
export RELAY_BINARY_PATH=${RELAY_BINARY_PATH:-$ROOT_FOLDER/moonbeam/binaries/polkadot};
export SPEC_FILE=$BASE_PATH/states/${NETWORK}-state.mod.json
# export NODE_OPTIONS=--max-old-space-size=16000

# echo "=========== Variables ==========="
# echo "RUNTIME_NAME: ${RUNTIME_NAME}"
# echo "NETWORK: ${NETWORK}"
# echo "PORT_PREFIX: ${PORT_PREFIX}"
# echo "ROOT_FOLDER: ${ROOT_FOLDER}"
# echo "GIT_TAG: ${GIT_TAG}"
# echo "GIT_TEST_TAG: ${GIT_TEST_TAG}"
# echo "SKIP_INTERMEDIATE_RUNTIME: ${SKIP_INTERMEDIATE_RUNTIME}"
# echo "FORCE_COMPILED_WASM: ${FORCE_COMPILED_WASM}"
# echo "SINGLE_PARACHAIN_NODE: ${SINGLE_PARACHAIN_NODE}"
# echo "SKIP_DOWNLOAD: ${SKIP_DOWNLOAD}"
# echo "SKIP_COMPILATION: ${SKIP_COMPILATION}"
# echo "SKIP_STATE_MODIFICATION: ${SKIP_STATE_MODIFICATION}"
# echo "KEEP_RUNNING: ${KEEP_RUNNING}"
# echo "USE_LOCAL_CLIENT: ${USE_LOCAL_CLIENT}"
# echo "ROUNDS_TO_WAIT: ${ROUNDS_TO_WAIT}"
# echo "BINARY_PATH: ${BINARY_PATH}"
# echo "RELAY_BINARY_PATH: ${RELAY_BINARY_PATH}"
# echo "SPEC_FILE: ${SPEC_FILE}"
# echo "NODE_OPTIONS: ${NODE_OPTIONS}"
# echo "================================"

# if [[ $PARA_ID == "" ]]; then
#     if [[ $NETWORK == "moonbeam" ]]; then
#         export PARA_ID=2004
#     elif [[ $NETWORK == "moonriver" ]]; then
#         export PARA_ID=2023
#     elif [[ $NETWORK == "moonbase-alpha" ]]; then
#         export PARA_ID=1000
#     else
#         export PARA_ID=1000
#     fi
# fi

if [ -d "$BASE_PATH" ]; 
then 
    echo "Clearing existing files at $BASE_PATH"
    rm -Rf $BASE_PATH; 
fi

mkdir -p $BASE_PATH
cd $BASE_PATH

# Clone moonbeam-tools repo & building
git clone https://github.com/PureStake/moonbeam-tools.git
cd $BASE_PATH/moonbeam-tools
echo "Downloading libraries..."
npm i

npx ts-node ./src/tools/run-moonbeam-fork.ts \
                                        --network moonbeam  \
                                        --ephemeral \
                                        --base-path $BASE_PATH


./binaries/polkadot --base-path $BASE_PATH/$NETWORK/relay-alice --alice --chain ${relayRawSpecFile} --rpc-port 11001 --ws-port 12001 --port 10001 --node-key ${
        Object.keys(NODE_KEYS)[0]
      } --validator

echo "Done !!"                                    