#!/bin/bash
# Loading binary/specs variables

if [ -z "$POLKADOT_COMMIT" ]; then
  POLKADOT_COMMIT=`egrep -o '/polkadot.*#([^\"]*)' Cargo.lock | \
    head -1 | sed 's/.*#//' |  cut -c1-8`
fi

if [ -z "$POLKADOT_REPO" ]; then
  POLKADOT_REPO=`egrep -o 'https://github.com/[^\/]*/polkadot\\?branch=' Cargo.lock | \
    head -1 | sed 's/?branch=//'`
fi

echo "Using Polkadot from $POLKADOT_REPO revision #${POLKADOT_COMMIT}"

docker build . -f docker/polkadot-relay.Dockerfile \
  --build-arg POLKADOT_COMMIT="$POLKADOT_COMMIT" \
  --build-arg POLKADOT_REPO="$POLKADOT_REPO" \
  -t purestake/moonbase-relay-testnet:sha-$POLKADOT_COMMIT
