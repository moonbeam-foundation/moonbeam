#!/bin/bash

LOCAL_GIT_BRANCH="$(git symbolic-ref HEAD 2>/dev/null)"
LOCAL_GIT_BRANCH=${LOCAL_GIT_BRANCH##refs/heads/}

echo ${1:-"$LOCAL_GIT_BRANCH"}

rm -rf build/{moonbeam-runtime-overrides,wasm}
mkdir -p build/wasm
git clone --depth 1 -b rq/tracing-improvements https://github.com/moonbeam-foundation/moonbeam-runtime-overrides build/moonbeam-runtime-overrides

cd build/moonbeam-runtime-overrides
./scripts/import-tracing-runtime.sh local ${1:-"$LOCAL_GIT_BRANCH"}
./scripts/build-tracing-runtime.sh local moonbase
mv wasm/moonbase-runtime-local-substitute-tracing.wasm ../wasm/moonbase-runtime-local-substitute-tracing.wasm
