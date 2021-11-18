#!/bin/bash

# This script is an example for running Moonbeam's benchmarks.
# It requires Moonbeam to be compiled with --features=runtime-benchmarks

export WASMTIME_BACKTRACE_DETAILS=1

./target/release/moonbeam benchmark \
    --chain dev \
    --execution=wasm \
    --wasm-execution=compiled \
    --pallet "parachain_staking" \
    --extrinsic "*" \
    --steps 32 \
    --repeat 64 \
    --raw \
    --template=./benchmarking/frame-weight-template.hbs \
    --output /tmp/ \
    --record-proof
