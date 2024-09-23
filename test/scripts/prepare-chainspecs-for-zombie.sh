#!/bin/bash

echo "Preparing chainspecs for Zombie"
echo "Building plain Moonbase specs..."
../target/release/moonbeam build-spec --chain moonbase-local > tmp/moonbase-plain-spec.json

echo "Modifying plain Moonbase specs..."
pnpm tsx scripts/modify-plain-specs.ts process tmp/moonbase-plain-spec.json tmp/moonbase-modified-spec.json

echo "Building raw Moonbase specs..."
../target/release/moonbeam build-spec --chain tmp/moonbase-modified-spec.json --raw > tmp/moonbase-raw-spec.json

echo "Preapproving runtime..."
pnpm tsx scripts/preapprove-rt-rawspec.ts process tmp/moonbase-raw-spec.json tmp/moonbase-modified-raw-spec.json ../target/release/wbuild/moonbase-runtime/moonbase_runtime.compact.compressed.wasm

echo "Done preparing chainspecs for Zombie ✅"