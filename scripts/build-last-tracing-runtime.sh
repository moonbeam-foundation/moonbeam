#!/bin/bash -x

LOCAL_GIT_BRANCH="$(git symbolic-ref HEAD 2>/dev/null)"
LOCAL_GIT_BRANCH=${LOCAL_GIT_BRANCH##refs/heads/}

echo ${1:-"$LOCAL_GIT_BRANCH"}

rm -rf build/{moonbeam-runtime-overrides,wasm}
mkdir -p build/wasm
git clone --depth 1 -b master-without-wasm https://github.com/moonbeam-foundation/moonbeam-runtime-overrides build/moonbeam-runtime-overrides

cd build/moonbeam-runtime-overrides
bash -x ./scripts/import-tracing-runtime.sh local ${1:-"$LOCAL_GIT_BRANCH"}
bash -x ./scripts/build-tracing-runtime.sh local moonbeam
mv wasm/moonbeam-runtime-local-substitute-tracing.wasm ../wasm/moonbeam-runtime-local-substitute-tracing.wasm
