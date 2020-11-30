#!/bin/bash
source scripts/_init_var.sh

if [ -z "$POLKADOT_VERSION" ]; then
  POLKADOT_VERSION="sha-`egrep -o 'paritytech/polkadot.*#([^\"]*)' Cargo.lock | \
    head -1 | sed 's/.*#//' |  cut -c1-8`"
fi


echo "Using Polkadot revision #${POLKADOT_VERSION}"

docker run -it purestake/moonbase-relay-testnet:$POLKADOT_VERSION \
    build-spec \
    -lerror \
    --disable-default-bootnode \
    --chain rococo-local \
    | grep '\"code\"' > $POLKADOT_SPEC_TMP
echo $POLKADOT_SPEC_TMP generated

echo "Using $POLKADOT_SPEC_TEMPLATE..."
sed -e "/\"<runtime_code>\"/{r $POLKADOT_SPEC_TMP" -e 'd}' $POLKADOT_SPEC_TEMPLATE \
  > $POLKADOT_SPEC_PLAIN
echo $POLKADOT_SPEC_PLAIN generated


# "Chain does not have enough staking candidates to operate" is displayed when no
# staker is given at genesis

docker run -it -v $(pwd)/build:/build purestake/moonbase-relay-testnet:$POLKADOT_VERSION \
  build-spec \
  -lerror \
  --disable-default-bootnode \
  --raw \
  --chain /$POLKADOT_SPEC_PLAIN \
  | grep -v 'Chain does not have enough staking candidates to operate' \
  > $POLKADOT_SPEC_RAW
echo $POLKADOT_SPEC_RAW generated
