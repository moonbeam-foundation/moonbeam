#!/bin/bash

LOCAL_GIT_BRANCH="$(git symbolic-ref HEAD 2>/dev/null)"

rm -rf build/{moonbeam-runtime-overrides,wasm}
mkdir -p build/wasm
git clone --depth 1 -b master-without-wasm https://github.com/PureStake/moonbeam-runtime-overrides build/moonbeam-runtime-overrides

cd build/moonbeam-runtime-overrides
./scripts/import-tracing-runtime.sh local ${1:-$LOCAL_GIT_BRANCH##refs/heads/}
cd tracing/local && cargo update -p evm && cd ../..
./scripts/build-tracing-runtime.sh local moonbase
mv wasm/moonbase-runtime-local-substitute-tracing.wasm ../wasm/moonbase-runtime-local-substitute-tracing.wasm
