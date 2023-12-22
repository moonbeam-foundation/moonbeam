KEY_NAME="mocha_test_key"
export MNEMONIC="north hold song deliver occur indoor myself suspect scheme regret speed affair dutch cave tackle taste holiday use dumb bitter mosquito invite bright wasp"

output=$(cel-key list --node.type $NODE_TYPE --p2p.network $P2P_NETWORK)

if [ -z "$(echo "$output" | sed -n '2p')" ]; then
    # if there is no keys at $HOME/.celestia-light-mocha-4/keys, recover the key from mnemonic, 
    echo $MNEMONIC | cel-key add $KEY_NAME --recover --node.type $NODE_TYPE --p2p.network $P2P_NETWORK 
fi


export AUTH_TOKEN=$(celestia $NODE_TYPE auth admin --p2p.network $P2P_NETWORK)

echo ''
echo "=== Auth token ==="
echo $AUTH_TOKEN 
echo "=================="
echo ''


celestia $NODE_TYPE init --p2p.network $P2P_NETWORK --keyring.accname $KEY_NAME

celestia $NODE_TYPE start --core.ip $RPC_URL --p2p.network $P2P_NETWORK 
