#!/bin/bash

trap "trap - TERM && kill -- -$$" INT TERM EXIT

if [[ ! -f "../target/release/moonbeam" ]]; 
then
  echo 'Missing moonbeam binary. Please run cargo build --release'
  exit 1;
fi

# Fail if any command fails

echo "Installing Packages"
pnpm i

echo "Starting moonbeam node"
../target/release/moonbeam --tmp --chain=moonbase-local --rpc-port=9933 &> /tmp/node-start.log &
PID=$!

echo "Waiting node...(5s)"
sleep 1
( tail -f -n0 /tmp/node-start.log & ) | grep -q 'Running JSON-RPC server:'

echo "Generating types...(10s)"
sleep 1
pnpm load:meta
pnpm load:meta:local
pnpm generate:defs
pnpm generate:meta

kill $PID
echo "Done :)"
exit 0