#!/bin/bash
source scripts/_init_var.sh

echo "=================== Alphanet ==================="
$PARACHAIN_BINARY build-spec \
  --disable-default-bootnode \
  | grep '\"code\"' \
  | head -n1 > $ALPHANET_SPEC_TMP
echo $ALPHANET_SPEC_TMP generated	

echo "Using $ALPHANET_SPEC_TEMPLATE..."	
sed -e "/\"<runtime_code>\"/{r $ALPHANET_SPEC_TMP" -e 'd;}'  $ALPHANET_SPEC_TEMPLATE \
  > $ALPHANET_SPEC_PLAIN	
echo $ALPHANET_SPEC_PLAIN generated

$PARACHAIN_BINARY build-spec \
  --disable-default-bootnode \
  --raw \
  --chain $ALPHANET_SPEC_PLAIN \
  > $ALPHANET_SPEC_RAW
echo $ALPHANET_SPEC_RAW generated

$PARACHAIN_BINARY export-genesis-wasm \
  --chain $ALPHANET_SPEC_RAW \
  > $PARACHAIN_WASM;
echo $PARACHAIN_WASM generated

$PARACHAIN_BINARY export-genesis-state \
  --parachain-id $PARACHAIN_ID \
  --chain $ALPHANET_SPEC_RAW \
  > $PARACHAIN_GENESIS;
echo $PARACHAIN_GENESIS generated

echo "\n=================== Stagenet ==================="
$PARACHAIN_BINARY build-spec \
  --disable-default-bootnode \
  | grep '\"code\"' \
  | head -n1 > $STAGENET_SPEC_TMP
echo $STAGENET_SPEC_TMP generated	

echo "Using $STAGENET_SPEC_TEMPLATE..."	
sed -e "/\"<runtime_code>\"/{r $STAGENET_SPEC_TMP" -e 'd;}'  $STAGENET_SPEC_TEMPLATE \
  > $STAGENET_SPEC_PLAIN	
echo $STAGENET_SPEC_PLAIN generated

$PARACHAIN_BINARY build-spec \
  --disable-default-bootnode \
  --raw \
  --chain $STAGENET_SPEC_PLAIN \
  > $STAGENET_SPEC_RAW
echo $STAGENET_SPEC_RAW generated

$PARACHAIN_BINARY export-genesis-wasm \
  --chain $STAGENET_SPEC_RAW \
  > $PARACHAIN_WASM;
echo $PARACHAIN_WASM generated

$PARACHAIN_BINARY export-genesis-state \
  --parachain-id $PARACHAIN_ID \
  --chain $STAGENET_SPEC_RAW \
  > $PARACHAIN_GENESIS;
echo $PARACHAIN_GENESIS generated