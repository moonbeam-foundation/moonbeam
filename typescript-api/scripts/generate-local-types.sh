#!/bin/bash

trap "trap - TERM && kill -- -$$" INT TERM EXIT

if [[ ! -f "../target/release/moonbeam" ]]; 
then
  echo 'Missing moonbeam binary. Please run cargo build --release'
  exit 1;
fi

# Fail if any command fails

echo "Installing Packages"
npm install
npm ci

echo "Starting moonbeam node"
../target/release/moonbeam --tmp --chain=moonbase-local --rpc-port=9933 &> /tmp/node-start.log &
PID=$!

echo "Waiting node...(5s)"
sleep 1
( tail -f -n0 /tmp/node-start.log & ) | grep -q 'new connection'

echo "Generating types...(10s)"
sleep 1
npm run load:meta
npm run load:meta:local
npm run generate:defs
npm run generate:meta
npm run postgenerate

kill $PID
echo "Done :)"