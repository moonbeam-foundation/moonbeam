#!/bin/bash
set -e
source scripts/_init_var.sh

if [ -z "$POLKADOT_VERSION" ]; then
  POLKADOT_VERSION="sha-`egrep -o '/polkadot.*#([^\"]*)' Cargo.lock | \
    head -1 | sed 's/.*#//' |  cut -c1-8`"
fi

echo "Using Polkadot revision #${POLKADOT_VERSION}"

echo "=================== Rococo-Local ==================="
docker run -it -v $(pwd)/build:/build moonbeamfoundation/moonbase-relay-testnet:$POLKADOT_VERSION \
  /usr/local/bin/polkadot \
    build-spec \
      --chain rococo-local \
      -lerror \
      --disable-default-bootnode \
      --raw \
    > $ROCOCO_LOCAL_RAW_SPEC
echo $ROCOCO_LOCAL_RAW_SPEC generated