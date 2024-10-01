#!/bin/bash

# First argument is the chain name: moonbeam or moonbase
CHAIN=$1

LATEST_RUNTIME_RELEASE=$(curl -s https://api.github.com/repos/moonbeam-foundation/moonbeam/releases | jq -r '.[] | select(.name | test("runtime";"i")) | .tag_name' | head -n 1 | tr -d '[:blank:]') && [[ ! -z "${LATEST_RUNTIME_RELEASE}" ]]
ENDPOINT="https://api.github.com/repos/moonbeam-foundation/moonbeam/git/refs/tags/$LATEST_RUNTIME_RELEASE"
RESPONSE=$(curl -s -H "Accept: application/vnd.github.v3+json" $ENDPOINT)
TYPE=$(echo $RESPONSE | jq -r '.object.type')
    if [[ $TYPE == "commit" ]]
    then
    LATEST_RT_SHA8=$(echo $RESPONSE | jq -r '.object.sha' | cut -c -8)
    elif [[ $TYPE == "tag" ]]
    then
    URL=$(echo $RESPONSE | jq -r '.object.url')
    TAG_RESPONSE=$(curl -s -H "Accept: application/vnd.github.v3+json" $URL)
    TAG_RESPONSE_CLEAN=$(echo $TAG_RESPONSE | tr -d '\000-\037')
    LATEST_RT_SHA8=$(echo $TAG_RESPONSE_CLEAN | jq -r '.object.sha' | cut -c -8)
    fi
DOCKER_TAG="moonbeamfoundation/moonbeam:sha-$LATEST_RT_SHA8"

echo $DOCKER_TAG

docker rm -f moonbeam_container 2> /dev/null | true
docker create --name moonbeam_container $DOCKER_TAG bash
docker cp moonbeam_container:moonbeam/moonbeam tmp/moonbeam_rt
docker rm -f moonbeam_container

chmod uog+x tmp/moonbeam_rt
chmod uog+x ../target/release/moonbeam
echo "Building plain Moonbase specs..."
tmp/moonbeam_rt build-spec --chain $CHAIN-local > tmp/$CHAIN\-plain-spec.json
pnpm tsx scripts/modify-plain-specs.ts process tmp/$CHAIN\-plain-spec.json tmp/$CHAIN\-modified-spec.json
tmp/moonbeam_rt build-spec --chain tmp/$CHAIN\-modified-spec.json --raw > tmp/$CHAIN\-raw-spec.json
pnpm tsx scripts/preapprove-rt-rawspec.ts process tmp/$CHAIN\-raw-spec.json tmp/$CHAIN\-modified-raw-spec.json ../target/release/wbuild/$CHAIN\-runtime/$CHAIN\_runtime.compact.compressed.wasm

echo "Done preparing chainspecs for Zombienet tests! ✅"
