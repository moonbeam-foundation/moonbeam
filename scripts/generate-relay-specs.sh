#!/bin/bash
source scripts/_init_var.sh

POLKADOT_VERSION=`egrep -o 'paritytech/polkadot.*#([^\"]*)' Cargo.lock | head -1 | sed 's/.*#//' |  cut -c1-8`

# TODO remove this once docker images are tagger with revision
POLKADOT_VERSION="latest"

echo "Using Polkadot revision #${POLKADOT_VERSION}"

docker run -it purestake/moonbase-relay-testnet:$POLKADOT_VERSION /usr/local/bin/polkadot \
    build-spec \
    -lerror \
    --disable-default-bootnode \
    --chain rococo-local \
    | grep '\"code\"' > $POLKADOT_SPEC_TMP
echo $POLKADOT_SPEC_TMP generated

echo "Using $POLKADOT_SPEC_TEMPLATE..."
sed -e "/\"<runtime_code>\"/{r $POLKADOT_SPEC_TMP" -e 'd}' $POLKADOT_SPEC_TEMPLATE > $POLKADOT_SPEC_PLAIN
echo $POLKADOT_SPEC_PLAIN generated

docker run -it -v $(pwd)/build:/build purestake/moonbase-relay-testnet:$POLKADOT_VERSION /usr/local/bin/polkadot \
  build-spec \
  -lerror \
  --disable-default-bootnode \
  --raw \
  --chain /$POLKADOT_SPEC_PLAIN \
  > $POLKADOT_SPEC_RAW
echo $POLKADOT_SPEC_RAW generated