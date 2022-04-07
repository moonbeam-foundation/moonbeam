#/bin/sh

POLKADOT_COMMIT=$(egrep -o '/polkadot.*#([^\"]*)' Cargo.lock | head -1 | sed 's/.*#//' |  cut -c1-8)
DOCKER_TAG="purestake/moonbase-relay-testnet:sha-$POLKADOT_COMMIT"

# Build relay binary if needed
POLKADOT_EXISTS=docker manifest inspect $DOCKER_TAG > /dev/null && "true" || "false"
if [[ "$POLKADOT_EXISTS" == "false" ]]; then
  # $POLKADOT_COMMIT is used to build the relay image
  ./scripts/build-alphanet-relay-image.sh
fi

# Get relay binary
docker create -ti --name dummy $DOCKER_TAG bash
docker cp dummy:/usr/local/bin/polkadot target/release/polkadot
docker rm -f dummy
