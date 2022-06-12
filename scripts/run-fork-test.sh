#!/bin/sh

# This script is expected to be included in a docker image (with node)
set -e 

export RUNTIME_NAME=${RUNTIME_NAME:-"moonbeam"}
export NETWORK=${NETWORK:-"moonbeam"} #moonbase-alpha for alphanet
export PARA_ID=${PARA_ID:-"2004"}
export PORT_PREFIX=${PORT_PREFIX:-"51"}
export ROOT_FOLDER=${ROOT_FOLDER:-"/data"}
export GIT_TAG=${GIT_TAG:-"master"}
export SKIP_INTERMEDIATE_RUNTIME=${SKIP_INTERMEDIATE_RUNTIME:-"true"}

export NODE_OPTIONS=--max-old-space-size=16000

echo "Preparation..."
trap "trap - TERM && kill -- -$$" INT TERM EXIT

mkdir -p $ROOT_FOLDER/states
cd $ROOT_FOLDER

# Clone moonbeam repo & building
echo "Cloning repository..."
git clone --depth 1 -b $GIT_TAG https://github.com/purestake/moonbeam
cd $ROOT_FOLDER/moonbeam
mkdir binaries

echo "Retrieving binaries..."
MOONBEAM_CLIENT_TAG=`curl -s https://api.github.com/repos/purestake/moonbeam/releases | jq -r '.[] | .tag_name' | grep '^v' | head -1`
POLKADOT_CLIENT_TAG=`curl -s https://api.github.com/repos/paritytech/polkadot/releases | jq -r '.[] | .tag_name' | grep '^v' | head -1`

wget -q https://github.com/PureStake/moonbeam/releases/download/${MOONBEAM_CLIENT_TAG}/moonbeam \
    -O $ROOT_FOLDER/moonbeam/binaries/moonbeam; 
export BINARY_PATH=$ROOT_FOLDER/moonbeam/binaries/moonbeam;

wget -q https://github.com/paritytech/polkadot/releases/download/${POLKADOT_CLIENT_TAG}/polkadot \
    -O $ROOT_FOLDER/moonbeam/binaries/polkadot; 
export RELAY_BINARY_PATH=$ROOT_FOLDER/moonbeam/binaries/polkadot;

chmod uog+x $BINARY_PATH $RELAY_BINARY_PATH

echo "Retrieving ${NETWORK} state... (few minutes)"
wget -q https://s3.us-east-2.amazonaws.com/snapshots.moonbeam.network/${NETWORK}/latest/${NETWORK}-state.json \
    -O $ROOT_FOLDER/states/${NETWORK}-state.json; 

## Build the runtime to test
echo "Building $GIT_TAG $RUNTIME_NAME runtime... (5 minutes)"
cd $ROOT_FOLDER/moonbeam
cargo build --release -p ${RUNTIME_NAME}-runtime

echo "Preparing tests... (3 minutes)"
cd $ROOT_FOLDER/moonbeam/moonbeam-types-bundle
npm install
cd $ROOT_FOLDER/moonbeam/tools
npm install

cd $ROOT_FOLDER/moonbeam/tests
git fetch origin crystalin-fork-test-preparation:crystalin-fork-test-preparation
git checkout crystalin-fork-test-preparation
npm install

# Modify state
echo "Customizing $NETWORK forked state..."
export SPEC_FILE=$ROOT_FOLDER/states/${NETWORK}-state.mod.json
node_modules/.bin/ts-node state-modifier.ts $ROOT_FOLDER/states/${NETWORK}-state.json

# Run the node
echo "Running nodes..."
./node_modules/.bin/ts-node spawn-fork-node.ts 2>&1 > spawn-node.log &
PID=$!

# Wait for the node to start
echo "Waiting nodes... (10 minutes)"
( tail -f -n0 spawn-node.log & ) | grep -q 'POLKADOT LAUNCH COMPLETE'

export RELAY_WSS_URL=ws://localhost:51002
export WSS_URL=ws://localhost:51102
# Run the fork test (without spawning the node using DEBUG_MODE)
echo "Running fork tests... (10 minutes)"
DEBUG_MODE=true npm run fork-test

echo "Running smoke tests... (10 minutes)"
SKIP_RELAY_TESTS=true DEBUG=smoke:* npm run smoke-test

echo "Retrieving runtime stats..."
cd $ROOT_FOLDER/moonbeam/tools
node_modules/.bin/ts-node extract-migration-logs.ts --log ../tests/51102.log

echo "Done !!"