mkdir -p build/moonbeam-v0.8.0
docker create --name moonbeam-tmp purestake/moonbeam:v0.8.0
docker cp moonbeam-tmp:/moonbeam/moonbeam build/moonbeam-v0.8.0/
docker rm moonbeam-tmp

mkdir -p build/rococo-9001
docker create --name moonbeam-tmp purestake/moonbase-relay-testnet:sha-86a45114
docker cp moonbeam-tmp:/usr/local/bin/polkadot build/rococo-9001/
docker rm moonbeam-tmp