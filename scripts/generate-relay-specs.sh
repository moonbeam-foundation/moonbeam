#!/bin/bash
source scripts/_init_var.sh

if [ -z "$POLKADOT_VERSION" ]; then
  POLKADOT_VERSION="sha-`egrep -o 'paritytech/polkadot.*#([^\"]*)' Cargo.lock | \
    head -1 | sed 's/.*#//' |  cut -c1-8`"
fi


echo "Using Polkadot revision #${POLKADOT_VERSION}"

# "Chain does not have enough staking candidates to operate" is displayed when no
# staker is given at genesis

docker run -it -v $(pwd)/build:/build purestake/moonbase-relay-testnet:$POLKADOT_VERSION \
  /usr/local/bin/polkadot \
    build-spec \
    --chain rococo-local \
    -lerror \
    --disable-default-bootnode \
    --raw \
    | grep -v 'Chain does not have enough staking candidates to operate' \
    > $POLKADOT_SPEC_RAW
echo $POLKADOT_SPEC_RAW generated
