#!/bin/bash

CHAINS=(
  moonbase
  moonriver
  moonbeam
)

# params
RUNTIME_CHAIN_SPEC=$1

# Bump package version
if [[ $# -gt 0 ]]; then
  echo "Bump package version to 0.$RUNTIME_CHAIN_SPEC.0"
  npm version --no-git-tag-version 0.$RUNTIME_CHAIN_SPEC.0
fi

if [[ ! -f ../target/release/moonbeam ]]; then
  echo "Missing ../target/release/moonbeam binary"
  exit 1
fi

# Install dependencies
pnpm install

# Get runtimes metadata
for CHAIN in ${CHAINS[@]}; do
  echo "Starting $CHAIN node"
  ../target/release/moonbeam \
    --no-hardware-benchmarks \
    --unsafe-force-node-key-generation \
    --no-telemetry --no-prometheus --alice \
    --tmp --chain=$CHAIN-dev \
    --wasm-execution=interpreted-i-know-what-i-do \
    --rpc-port=9933 &> /tmp/node-$CHAIN-start.log &
  PID=$!
  echo "Waiting node..."
  ( tail -f -n0 /tmp/node-$CHAIN-start.log & ) | grep -q 'Running JSON-RPC server'
  echo "Getting $CHAIN metadata"
  curl -s -H "Content-Type: application/json" -d '{"id":"1", "jsonrpc":"2.0", "method": "state_getMetadata", "params":[]}' http://localhost:9933 > metadata-$CHAIN.json
  kill $PID
  sleep 5
done

# Generate typescript api code
echo "Generating typescript api code..."
pnpm generate:defs && pnpm generate:meta

# We don't need anymore fix for BTreeSet
#
## Manually fix BTreeSet issue
#echo "Manually fix BTreeSet issue..."
#for CHAIN in ${CHAINS[@]}; do
#  sed -i -e 's/BTreeSet,/BTreeSet as BTreeSetType,/g' src/$CHAIN/interfaces/types-lookup.ts
#  sed -i -e 's/BTreeSet<Bytes>/BTreeSetType<Bytes>/g' src/$CHAIN/interfaces/types-lookup.ts
#done

# Build the package
pnpm run build
pnpm fmt:fix