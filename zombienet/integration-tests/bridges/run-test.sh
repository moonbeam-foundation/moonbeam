#!/bin/bash

set -e

trap 'kill -9 -$$ || echo "Environment already teared down"' SIGINT SIGTERM EXIT

test=$1

export LOCAL_BRIDGE_TESTING_PATH=~/local_bridge_testing

if [ -z "$FRAMEWORK_REPO_PATH" ]; then
  # Download the bridge testing "framework" from the `polkadot-sdk` repo
  # to `~/local_bridge_testing/downloads/polkadot-sdk`.
  export DOWNLOADS_PATH=$LOCAL_BRIDGE_TESTING_PATH/downloads
  echo "FRAMEWORK_REPO_PATH is NOT set, so downloading 'polkadot-sdk' repo to the: $DOWNLOADS_PATH"
  mkdir -p $DOWNLOADS_PATH
  framework_repo_path=$DOWNLOADS_PATH/polkadot-sdk
  rm -rf $framework_repo_path
  git clone --branch master -n --depth=1 --filter=tree:0 \
    https://github.com/paritytech/polkadot-sdk.git $framework_repo_path
  pushd $framework_repo_path
  git sparse-checkout set --no-cone bridges/testing/framework
  # Checkout specified tag-or-commit, if not specified then master
  if [ -n "$FRAMEWORK_REPO_TAG_OR_COMMIT" ]; then
    git fetch --tags
    git checkout $FRAMEWORK_REPO_TAG_OR_COMMIT
  else
    git checkout master
  fi
  popd
else
    framework_repo_path=$FRAMEWORK_REPO_PATH
fi

export FRAMEWORK_PATH=$framework_repo_path/bridges/testing/framework
echo "Using bridges testing framework from path: $FRAMEWORK_PATH"
echo

export ZOMBIENET_BINARY="${PWD}/zombienet/bin/zombienet";
export ZOMBIENET_CONFIGS="${PWD}/zombienet/configs";
export POLKADOT_BINARY="${PWD}/zombienet/bin/polkadot";
export MOONBEAM_BINARY="${PWD}/zombienet/bin/moonbeam";
export SUBSTRATE_RELAY_BINARY="${PWD}/zombienet/bin/substrate-relay";

export ALITH_PRIVATE_KEY="0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133";
export BALTATHAR_PRIVATE_KEY="0x8075991ce870b93a8870eca0c0f91913d12f47948ca0fd25b49c6fa7cdbeee8b";

BASE_TEST_DIR="/tmp/bridge-integration-tests"
mkdir -p "/tmp/bridge-integration-tests"
export TEST_DIR=`mktemp -d ${BASE_TEST_DIR}/run-XXXXX`
echo -e "Test folder: $TEST_DIR\n"

${BASH_SOURCE%/*}/tests/$test/run.sh