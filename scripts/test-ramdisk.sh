POLKADOT_COMMIT=$(egrep -o '/polkadot.*#([^\"]*)' Cargo.lock | head -1 | sed 's/.*#//' |  cut -c1-8)
POLKADOT_REPO=$(egrep -o 'https.*/polkadot' Cargo.lock | head -1)
TEST_HASH=$(md5sum tests/package-lock.json | cut -f1 -d' ')
TESTS_DOCKER_TAG="purestake/polkadot-para-tests:sha-$POLKADOT_COMMIT-${TEST_HASH:0:8}"
TESTS_DOCKER_EXISTS=$(docker manifest inspect $TESTS_DOCKER_TAG > /dev/null && \
    echo "true" || echo "false")

if [[ "$TESTS_DOCKER_EXISTS" == "false" ]]; then
    # cd moonbeam-types-bundle
    # npm install
    # npm run build
    # cd ../tests
    # npm install
    # cd ..

    # mkdir -p build
    # MOONBEAM_DOCKER_TAG="purestake/moonbase-relay-testnet:sha-$POLKADOT_COMMIT"
    # docker create --pull always -ti --name dummy $MOONBEAM_DOCKER_TAG bash
    # docker cp dummy:/usr/local/bin/polkadot build/polkadot
    # docker rm -f dummy

    docker build . --pull --no-cache -f docker/polkadot-para-tests.Dockerfile \
        --network=host \
        --build-arg HOST_UID="$UID" \
        -t $TESTS_DOCKER_TAG
fi

