#!/bin/bash
set -e
source scripts/_init_var.sh

if [ -z "$POLKADOT_VERSION" ]; then
  POLKADOT_VERSION="sha-`egrep -o '/polkadot.*#([^\"]*)' Cargo.lock | \
    head -1 | sed 's/.*#//' |  cut -c1-8`"
fi

echo "Using Polkadot revision #${POLKADOT_VERSION}"

echo "=================== Rococo-Local ==================="
docker run -it -v $(pwd)/build:/build purestake/moonbase-relay-testnet:$POLKADOT_VERSION \
  /usr/local/bin/polkadot \
    build-spec \
      --chain rococo-local \
      -lerror \
      --disable-default-bootnode \
      --raw \
    > $ROCOCO_LOCAL_RAW_SPEC
echo $ROCOCO_LOCAL_RAW_SPEC generated


echo "=================== Alphanet ==================="
docker run -it -v $(pwd)/build:/build purestake/moonbase-relay-testnet:$POLKADOT_VERSION \
  /usr/local/bin/polkadot \
    build-spec \
      --chain rococo-local \
      -lerror \
      --disable-default-bootnode \
    | grep '\"code\"' \
    | head -n1 > $ALPHANET_ROCOCO_SPEC_TMP \
    > $ALPHANET_ROCOCO_SPEC_TMP

echo "Using $ALPHANET_ROCOCO_SPEC_TEMPLATE..."	
sed -e "/\"<runtime_code>\"/{r $ALPHANET_ROCOCO_SPEC_TMP" -e 'd;}' $ALPHANET_ROCOCO_SPEC_TEMPLATE \
  > $ALPHANET_ROCOCO_SPEC_PLAIN	
echo $ALPHANET_ROCOCO_SPEC_PLAIN generated

docker run -it -v $(pwd)/build:/build purestake/moonbase-relay-testnet:$POLKADOT_VERSION \
  /usr/local/bin/polkadot \
    build-spec \
      --disable-default-bootnode \
      -lerror \
      --raw \
      --chain /$ALPHANET_ROCOCO_SPEC_PLAIN \
  > $ALPHANET_ROCOCO_SPEC_RAW
echo $ALPHANET_ROCOCO_SPEC_RAW generated


echo "=================== Stagenet ==================="
docker run -it -v $(pwd)/build:/build purestake/moonbase-relay-testnet:$POLKADOT_VERSION \
  /usr/local/bin/polkadot \
    build-spec \
      --chain rococo-local \
      -lerror \
      --disable-default-bootnode \
    | grep '\"code\"' \
    | head -n1 > $STAGENET_ROCOCO_SPEC_TMP \
    > $STAGENET_ROCOCO_SPEC_TMP

echo "Using $STAGENET_ROCOCO_SPEC_TEMPLATE..."	
sed -e "/\"<runtime_code>\"/{r $STAGENET_ROCOCO_SPEC_TMP" -e 'd;}' $STAGENET_ROCOCO_SPEC_TEMPLATE \
  > $STAGENET_ROCOCO_SPEC_PLAIN	
echo $STAGENET_ROCOCO_SPEC_PLAIN generated

docker run -it -v $(pwd)/build:/build purestake/moonbase-relay-testnet:$POLKADOT_VERSION \
  /usr/local/bin/polkadot \
    build-spec \
      --disable-default-bootnode \
      -lerror \
      --raw \
      --chain /$STAGENET_ROCOCO_SPEC_PLAIN \
  > $STAGENET_ROCOCO_SPEC_RAW
echo $STAGENET_ROCOCO_SPEC_RAW generated