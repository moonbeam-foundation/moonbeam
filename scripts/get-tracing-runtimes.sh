#!/bin/bash

mkdir -p scripts/tmp
git clone --depth 1 https://github.com/PureStake/moonbeam-runtime-overrides scripts/tmp/moonbeam-runtime-overrides
mkdir -p build/moonbase-overrides
cp scripts/tmp/moonbeam-runtime-overrides/wasm/moonbase-*-tracing.wasm build/moonbase-overrides/
