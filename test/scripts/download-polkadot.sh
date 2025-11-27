#!/bin/bash

# Exit on any error
set -e

# Grab Polkadot version
branch=$(egrep -o '/polkadot.*#([^\"]*)' $(dirname $0)/../../Cargo.lock | head -1 | sed 's/.*release-//#')
polkadot_release=$(echo $branch | sed 's/#.*//' | sed 's/\/polkadot-sdk?branch=moonbeam-polkadot-//')

# Always run the commands from the "test" dir
cd $(dirname $0)/..

if [[ -f tmp/polkadot ]]; then
  POLKADOT_VERSION=$(tmp/polkadot --version)
  if [[ $POLKADOT_VERSION == *$polkadot_release* ]]; then
    exit 0
  else
    echo "Updating polkadot binary..."

    wget https://github.com/paritytech/polkadot-sdk/releases/download/polkadot-$polkadot_release/polkadot -P tmp
    chmod +x tmp/polkadot

    wget https://github.com/paritytech/polkadot-sdk/releases/download/polkadot-$polkadot_release/polkadot-execute-worker -P tmp
    chmod +x tmp/polkadot-execute-worker

    wget https://github.com/paritytech/polkadot-sdk/releases/download/polkadot-$polkadot_release/polkadot-prepare-worker -P tmp
    chmod +x tmp/polkadot-prepare-worker
  fi
else
  echo "Polkadot binary not found, downloading..."
  wget https://github.com/paritytech/polkadot-sdk/releases/download/polkadot-$polkadot_release/polkadot -P tmp
  chmod +x tmp/polkadot

  wget https://github.com/paritytech/polkadot-sdk/releases/download/polkadot-$polkadot_release/polkadot-execute-worker -P tmp
  chmod +x tmp/polkadot-execute-worker

  wget https://github.com/paritytech/polkadot-sdk/releases/download/polkadot-$polkadot_release/polkadot-prepare-worker -P tmp
  chmod +x tmp/polkadot-prepare-worker
fi
