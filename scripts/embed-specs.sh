#!/bin/bash
# This script copies a raw spec file from a moonbeam network repository into the correct place in this repository to be embedded in the moonbeam binary.
if [ -z "$2" ]; then
  echo "Usage: $0 [moonriver|alphanet] <docker_tag>"
  echo "Ex: $0 alphanet sha-081b1aab-4"
  exit 1
fi

NETWORK=$1
DOCKER_TAG=$2

PARACHAIN_DOCKER=moonbeamfoundation/moonbase-${NETWORK}:${DOCKER_TAG}
docker create --name moonbeam-tmp $PARACHAIN_DOCKER
docker cp moonbeam-tmp:/moonbase-parachain/parachain-raw-specs.json specs/${NETWORK}/parachain-embedded-specs-${DOCKER_TAG}.json
docker rm moonbeam-tmp
