#!/bin/bash

# Exit on any error
set -e

# Grab Polkadot version
branch=$(egrep -o '/polkadot.*#([^\"]*)' $(dirname $0)/../../Cargo.lock | head -1 | sed 's/.*release-//#')
polkadot_release=$(echo $branch | sed 's/#.*//' | sed 's/\/polkadot-sdk?branch=moonbeam-polkadot-v//')

# Always run the commands from the "test" dir
cd $(dirname $0)/..

if [[ -f tmp/polkadot ]]; then
  POLKADOT_VERSION=$(tmp/polkadot --version)
  if [[ $POLKADOT_VERSION == *$polkadot_release* ]]; then
    exit 0
  else
    echo "Updating polkadot binary..."

    wget https://github.com/paritytech/polkadot-sdk/releases/download/polkadot-v$polkadot_release/polkadot -P tmp
    chmod +x tmp/polkadot

    pnpm moonwall download polkadot-execute-worker $polkadot_release tmp
    chmod +x tmp/polkadot-execute-worker

    pnpm moonwall download polkadot-prepare-worker $polkadot_release tmp
    chmod +x tmp/polkadot-prepare-worker

  fi
else
  echo "Polkadot binary not found, downloading..."
  wget https://github.com/paritytech/polkadot-sdk/releases/download/polkadot-v$polkadot_release/polkadot -P tmp
  chmod +x tmp/polkadot

  pnpm moonwall download polkadot-execute-worker $polkadot_release tmp
  chmod +x tmp/polkadot-execute-worker

  pnpm moonwall download polkadot-prepare-worker $polkadot_release tmp
  chmod +x tmp/polkadot-prepare-worker
fi

# Create custom rococo chain spec that enable async backing
tmp/polkadot build-spec --chain rococo-local > tmp/rococo-plain-spec.json
pnpm tsx scripts/modify-rococo-plain-specs.ts process tmp/rococo-plain-spec.json tmp/rococo-modified-spec.json
tmp/polkadot build-spec --chain tmp/rococo-modified-spec.json --raw > tmp/rococo-raw-spec.json
