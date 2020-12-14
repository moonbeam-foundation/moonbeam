#!/bin/bash
# Loading binary/specs variables

if [ -z "$POLKADOT_COMMIT" ]; then
  POLKADOT_COMMIT=`egrep -o 'paritytech/polkadot.*#([^\"]*)' Cargo.lock | \
    head -1 | sed 's/.*#//' |  cut -c1-8`
fi

echo "Using Polkadot revision #${POLKADOT_COMMIT}"

docker build . -f docker/polkadot-relay.Dockerfile \
  --build-arg POLKADOT_COMMIT="$POLKADOT_COMMIT" \
  -t purestake/moonbase-relay-testnet:sha-$POLKADOT_COMMIT
