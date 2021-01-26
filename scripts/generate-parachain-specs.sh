#!/bin/bash
source scripts/_init_var.sh

$PARACHAIN_BINARY build-spec \
  --disable-default-bootnode \
  | grep '\"code\"' \
  | head -n1 > $PARACHAIN_SPEC_TMP
echo $PARACHAIN_SPEC_TMP generated	

echo "Using $PARACHAIN_SPEC_TEMPLATE..."	
sed -e "/\"<runtime_code>\"/{r $PARACHAIN_SPEC_TMP" -e 'd;}'  $PARACHAIN_SPEC_TEMPLATE \
  > $PARACHAIN_SPEC_PLAIN	
echo $PARACHAIN_SPEC_PLAIN generated

$PARACHAIN_BINARY build-spec \
  --disable-default-bootnode \
  --raw \
  --chain $PARACHAIN_SPEC_PLAIN \
  > $PARACHAIN_SPEC_RAW
echo $PARACHAIN_SPEC_RAW generated

$PARACHAIN_BINARY export-genesis-wasm \
  --chain $PARACHAIN_SPEC_RAW \
  > $PARACHAIN_WASM;
echo $PARACHAIN_WASM generated

$PARACHAIN_BINARY export-genesis-state \
  --parachain-id $PARACHAIN_ID \
  --chain $PARACHAIN_SPEC_RAW \
  > $PARACHAIN_GENESIS;
echo $PARACHAIN_GENESIS generated
