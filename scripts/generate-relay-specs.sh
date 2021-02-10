#!/bin/bash
source scripts/_init_var.sh

if [ -z "$POLKADOT_VERSION" ]; then
  POLKADOT_VERSION="sha-`egrep -o 'paritytech/polkadot.*#([^\"]*)' Cargo.lock | \
    head -1 | sed 's/.*#//' |  cut -c1-8`"
fi

echo "Using Polkadot revision #${POLKADOT_VERSION}"

docker run -it -v $(pwd)/build:/build purestake/moonbase-relay-testnet:$POLKADOT_VERSION \
  /usr/local/bin/polkadot \
    build-spec \
      --chain rococo-local \
      -lerror \
      --disable-default-bootnode \
      --raw \
    > $POLKADOT_SPEC_RAW
echo $POLKADOT_SPEC_RAW generated


echo "=================== Alphanet ==================="
docker run -it -v $(pwd)/build:/build purestake/moonbase-relay-testnet:$POLKADOT_VERSION \
  /usr/local/bin/polkadot \
    build-spec \
      --chain rococo-local \
      -lerror \
      --disable-default-bootnode \
    | grep '\"code\"' \
    | head -n1 > $ALPHANET_RELAY_SPEC_TMP \
    > $ALPHANET_RELAY_SPEC_TMP

echo "Using $ALPHANET_RELAY_SPEC_TEMPLATE..."	
sed -e "/\"<runtime_code>\"/{r $ALPHANET_RELAY_SPEC_TMP" -e 'd;}' $ALPHANET_RELAY_SPEC_TEMPLATE \
  > $ALPHANET_RELAY_SPEC_PLAIN	
echo $ALPHANET_RELAY_SPEC_PLAIN generated

docker run -it -v $(pwd)/build:/build purestake/moonbase-relay-testnet:$POLKADOT_VERSION \
  /usr/local/bin/polkadot \
    build-spec \
      --disable-default-bootnode \
      -lerror \
      --raw \
      --chain /$ALPHANET_RELAY_SPEC_PLAIN \
  > $ALPHANET_RELAY_SPEC_RAW
echo $ALPHANET_RELAY_SPEC_RAW generated


echo "=================== Stagenet ==================="
docker run -it -v $(pwd)/build:/build purestake/moonbase-relay-testnet:$POLKADOT_VERSION \
  /usr/local/bin/polkadot \
    build-spec \
      --chain rococo-local \
      -lerror \
      --disable-default-bootnode \
    | grep '\"code\"' \
    | head -n1 > $STAGENET_RELAY_SPEC_TMP \
    > $STAGENET_RELAY_SPEC_TMP

echo "Using $STAGENET_RELAY_SPEC_TEMPLATE..."	
sed -e "/\"<runtime_code>\"/{r $STAGENET_RELAY_SPEC_TMP" -e 'd;}' $STAGENET_RELAY_SPEC_TEMPLATE \
  > $STAGENET_RELAY_SPEC_PLAIN	
echo $STAGENET_RELAY_SPEC_PLAIN generated

docker run -it -v $(pwd)/build:/build purestake/moonbase-relay-testnet:$POLKADOT_VERSION \
  /usr/local/bin/polkadot \
    build-spec \
      --disable-default-bootnode \
      -lerror \
      --raw \
      --chain /$STAGENET_RELAY_SPEC_PLAIN \
  > $STAGENET_RELAY_SPEC_RAW
echo $STAGENET_RELAY_SPEC_RAW generated