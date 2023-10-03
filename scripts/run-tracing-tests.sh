#!/bin/bash

echo 'Make sure you have built moonbeam-types-bundle and run "npm install" in the test/ folder.'

BUILD_LAST_TRACING_RUNTIME="no"

if [ -e test/moonbase-overrides/moonbase-runtime-local-substitute-tracing.wasm ]; then
  if [[ "$1" == "-f" ]]; then
    BUILD_LAST_TRACING_RUNTIME="yes"
  fi
else
  BUILD_LAST_TRACING_RUNTIME="yes"
fi

if [[ "$BUILD_LAST_TRACING_RUNTIME" == "yes" ]]; then
  ./scripts/build-last-tracing-runtime.sh
  mkdir -p test/moonbase-overrides/
  mv build/wasm/moonbase-runtime-local-substitute-tracing.wasm test/moonbase-overrides/
else
  echo "The tracing runtime is not rebuilt, if you want to rebuild it, use the option '-f'."
fi

echo "Preparing tests dependencies…"
cd moonbeam-types-bundle
npm ci
npm run build

cd ../typescript-api
npm ci

echo "Run tracing tests…"
cd ../test
pnpm install
pnpm compile-solidity
pnpm moonwall test dev_moonbase_tracing
cd ..
