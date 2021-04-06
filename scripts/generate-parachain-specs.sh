#!/bin/bash
set -e
source scripts/_init_var.sh

echo "=================== Alphanet ==================="
$MOONBEAM_BINARY build-spec \
  --disable-default-bootnode \
  --chain 'local' \
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
  --parachain-id $PARACHAIN_ID \
  --chain $ALPHANET_PARACHAIN_SPEC_RAW \
  > $ALPHANET_GENESIS;
echo $ALPHANET_GENESIS generated

cp $ALPHANET_PARACHAIN_EMBEDDED_SPEC $ALPHANET_BUILD_FOLDER/parachain-embedded-specs.json
cp $ALPHANET_ROCOCO_EMBEDDED_SPEC $ALPHANET_BUILD_FOLDER/rococo-embedded-specs.json
grep -v '/p2p/' $ALPHANET_PARACHAIN_EMBEDDED_SPEC > \
  $ALPHANET_BUILD_FOLDER/parachain-embedded-no-bootnodes-specs.json
grep -v '/p2p/' $ALPHANET_ROCOCO_EMBEDDED_SPEC > \
  $ALPHANET_BUILD_FOLDER/rococo-embedded-no-bootnodes-specs.json


echo "=================== Stagenet ==================="
$MOONBEAM_BINARY build-spec \
  --disable-default-bootnode \
  --chain 'local' \
  | grep '\"code\"' \
  | head -n1 > $STAGENET_PARACHAIN_SPEC_TMP
echo $STAGENET_PARACHAIN_SPEC_TMP generated	

echo "Using $STAGENET_PARACHAIN_SPEC_TEMPLATE..."	
sed -e "/\"<runtime_code>\"/{r $STAGENET_PARACHAIN_SPEC_TMP" -e 'd;}'  $STAGENET_PARACHAIN_SPEC_TEMPLATE \
  > $STAGENET_PARACHAIN_SPEC_PLAIN	
echo $STAGENET_PARACHAIN_SPEC_PLAIN generated

$MOONBEAM_BINARY build-spec \
  --disable-default-bootnode \
  --raw \
  --chain $STAGENET_PARACHAIN_SPEC_PLAIN \
  > $STAGENET_PARACHAIN_SPEC_RAW
echo $STAGENET_PARACHAIN_SPEC_RAW generated

$MOONBEAM_BINARY export-genesis-wasm \
  --chain $STAGENET_PARACHAIN_SPEC_RAW \
  > $STAGENET_WASM;
echo $STAGENET_WASM generated

$MOONBEAM_BINARY export-genesis-state \
  --parachain-id $PARACHAIN_ID \
  --chain $STAGENET_PARACHAIN_SPEC_RAW \
  > $STAGENET_GENESIS;
echo $STAGENET_GENESIS generated

cp $STAGENET_PARACHAIN_EMBEDDED_SPEC $STAGENET_BUILD_FOLDER/parachain-embedded-specs.json
cp $STAGENET_ROCOCO_EMBEDDED_SPEC $STAGENET_BUILD_FOLDER/rococo-embedded-specs.json
grep -v '/p2p/' $STAGENET_PARACHAIN_EMBEDDED_SPEC > \
  $STAGENET_BUILD_FOLDER/parachain-embedded-no-bootnodes-specs.json
grep -v '/p2p/' $STAGENET_ROCOCO_EMBEDDED_SPEC > \
  $STAGENET_BUILD_FOLDER/rococo-embedded-no-bootnodes-specs.json
