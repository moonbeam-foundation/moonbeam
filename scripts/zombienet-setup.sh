#!/bin/bash
# Exit on any error
set -e

runtime="${1:-moonbase}"
echo "[+] Compiling runtime for $runtime... (this will take a while)"
cargo build --release

echo "[+] Creating test/tmp folder"
mkdir -p test/tmp

echo "[+] Copying runtime to test/tmp folder"
cp target/release/moonbeam test/tmp/moonbeam_rt

echo "[+] Changing permissions"
chmod uog+x test/tmp/moonbeam_rt
chmod uog+x target/release/moonbeam

echo "[+] Building specs for $runtime"
test/tmp/moonbeam_rt build-spec --chain ${runtime}-local > test/tmp/${runtime}-plain-spec.json
echo "[+] Modifying specs for $runtime"
cd test
pnpm tsx scripts/modify-plain-specs.ts process tmp/${runtime}-plain-spec.json tmp/${runtime}-modified-spec.json
echo "[+] Building raw specs for $runtime"
tmp/moonbeam_rt build-spec --chain tmp/${runtime}-modified-spec.json --raw > tmp/${runtime}-raw-spec.json
echo "[+] Preapproving raw specs for $runtime"
pnpm tsx scripts/preapprove-rt-rawspec.ts process tmp/${runtime}-raw-spec.json tmp/${runtime}-modified-raw-spec.json ../target/release/wbuild/${runtime}-runtime/${runtime}_runtime.compact.compressed.wasm