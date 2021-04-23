#!/bin/bash

export WASMTIME_BACKTRACE_DETAILS=1

./../../target/release/moonbeam benchmark \
    --chain dev \
    --execution=wasm \
    --wasm-execution=compiled \
    --pallet "parachain_staking" \
    --extrinsic "*" \
    --steps 32 \
    --repeat 64 \
    --raw \
    --template=./frame-weight-template.hbs \
    --output /tmp/
