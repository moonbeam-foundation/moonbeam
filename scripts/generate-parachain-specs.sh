#!/bin/bash
source scripts/_init_var.sh

$PARACHAIN_BINARY build-spec \
  --disable-default-bootnode \
  --raw \
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
