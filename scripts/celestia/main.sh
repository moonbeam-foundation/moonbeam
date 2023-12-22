export P2P_NETWORK=mocha
export NODE_TYPE=light
export RPC_URL=rpc-mocha.pops.one

NODE_STORE_PATH="$HOME/celestia-node-store"
PARENT_PATH="$(pwd)/scripts/celestia"

if [! -d "$NODE_STORE_PATH" ]; then
    mkdir $NODE_STORE_PATH
fi

docker run --rm -p 26658:26658 \
    -v $NODE_STORE_PATH:/home/celestia \
    -e NODE_TYPE=$NODE_TYPE -e P2P_NETWORK=$P2P_NETWORK -e RPC_URL=$RPC_URL\
    ghcr.io/celestiaorg/celestia-node:v0.12.0 \
    sh -c "$(<"$PARENT_PATH/start_node.sh")"
    

