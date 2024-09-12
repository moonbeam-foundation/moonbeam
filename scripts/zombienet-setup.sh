#!/bin/bash
# Exit on any error
set -e

runtime="${1:-moonbase}"
commit="${2:85851603}"

echo "[+] Compiling runtime for $runtime... (this will take a while)"
cargo build --release

echo "[+] Creating test/tmp folder"
mkdir -p test/tmp

echo "[+] Copying latest runtime to test/tmp folder"
SHA8="$commit"
DOCKER_TAG="moonbeamfoundation/moonbeam:sha-$SHA8"
docker rm -f moonbeam_container 2> /dev/null | true
docker create --name moonbeam_container $DOCKER_TAG bash
docker cp moonbeam_container:moonbeam/moonbeam test/tmp/moonbeam_rt
docker rm -f moonbeam_container

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