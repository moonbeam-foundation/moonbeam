#!/bin/bash
set -e
source scripts/_init_var.sh

echo "=================== Alphanet ==================="
$MOONBEAM_BINARY build-spec \
  --disable-default-bootnode \
  --chain 'moonbase-local' \
  | grep '\"code\"' \
  | head -n1 > $ALPHANET_PARACHAIN_SPEC_TMP
echo $ALPHANET_PARACHAIN_SPEC_TMP generated	

echo "Using $ALPHANET_PARACHAIN_SPEC_TEMPLATE..."	
sed -e "/\"<runtime_code>\"/{r $ALPHANET_PARACHAIN_SPEC_TMP" -e 'd;}'  $ALPHANET_PARACHAIN_SPEC_TEMPLATE \
  > $ALPHANET_PARACHAIN_SPEC_PLAIN	
echo $ALPHANET_PARACHAIN_SPEC_PLAIN generated

$MOONBEAM_BINARY build-spec \
  --disable-default-bootnode \
  --raw \
  --chain $ALPHANET_PARACHAIN_SPEC_PLAIN \
  > $ALPHANET_PARACHAIN_SPEC_RAW
echo $ALPHANET_PARACHAIN_SPEC_RAW generated

$MOONBEAM_BINARY export-genesis-wasm \
  --chain $ALPHANET_PARACHAIN_SPEC_RAW \
  > $ALPHANET_WASM;
echo $ALPHANET_WASM generated

$MOONBEAM_BINARY export-genesis-state \
  > $ALPHANET_GENESIS;
echo $ALPHANET_GENESIS generated

cp $ALPHANET_PARACHAIN_EMBEDDED_SPEC $ALPHANET_BUILD_FOLDER/parachain-embedded-specs.json
cp $ALPHANET_ROCOCO_EMBEDDED_SPEC $ALPHANET_BUILD_FOLDER/rococo-embedded-specs.json
grep -v '/p2p/' $ALPHANET_PARACHAIN_EMBEDDED_SPEC > \
  $ALPHANET_BUILD_FOLDER/parachain-embedded-no-bootnodes-specs.json
grep -v '/p2p/' $ALPHANET_ROCOCO_EMBEDDED_SPEC > \
  $ALPHANET_BUILD_FOLDER/rococo-embedded-no-bootnodes-specs.json
