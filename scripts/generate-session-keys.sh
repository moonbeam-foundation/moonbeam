
# POLKADOT_BINARY='../polkadot/target/release/polkadot'
POLKADOT_BINARY='docker run parity/polkadot:v0.9.1'

if [ -z "$MNEMONIC" ]; then
  MNEMONIC=`$POLKADOT_BINARY key generate -w 24 --output-type Json | jq -r '.secretPhrase'`
fi

KEY=`$POLKADOT_BINARY key inspect "$MNEMONIC" --output-type Json`
SEED=`jq -r '.secretSeed' <<< $KEY `
SR25519_SS58=`$POLKADOT_BINARY key inspect --scheme sr25519 "$MNEMONIC" --output-type Json 2>&1 | jq -r '.ss58PublicKey'`
ED25519_SS58=`$POLKADOT_BINARY key inspect --scheme ed25519 "$MNEMONIC" --output-type Json 2>&1 | jq -r '.ss58PublicKey'`
ECDSA_SS58=`$POLKADOT_BINARY key inspect --scheme ecdsa "$MNEMONIC" --output-type Json 2>&1 | jq -r '.ss58PublicKey'`
SR25519_PUB=`$POLKADOT_BINARY key inspect --scheme sr25519 "$MNEMONIC" --output-type Json 2>&1 | jq -r '.publicKey'`
ED25519_PUB=`$POLKADOT_BINARY key inspect --scheme ed25519 "$MNEMONIC" --output-type Json 2>&1 | jq -r '.publicKey'`
ECDSA_PUB=`$POLKADOT_BINARY key inspect --scheme ecdsa "$MNEMONIC" --output-type Json 2>&1 | jq -r '.publicKey'`
ECDSA_PUB_ENC=`cd tools && node_modules/.bin/ts-node beefy.ts $ECDSA_PUB`

echo "****************** $node account data ******************"
echo "secret_seed:      $SEED"
echo "mnemonic:         $MNEMONIC"
echo "sr25519 address:  $SR25519_PUB (SS58: $SR25519_SS58)"
echo "ed25519 address:  $ED25519_PUB (SS58: $ED25519_SS58)"
echo "ecdsa address:    $ECDSA_PUB (Encoded: $ECDSA_PUB_ENC)"
echo "    [
    \"$SR25519_SS58\",
    \"$ED25519_SS58\",
    {
        \"grandpa\": \"$ED25519_SS58\",
        \"babe\": \"$SR25519_SS58\",
        \"im_online\": \"$SR25519_SS58\",
        \"para_validator\": \"$SR25519_SS58\",
        \"para_assignment\": \"$SR25519_SS58\",
        \"authority_discovery\": \"$SR25519_SS58\",
        \"beefy\": \"$ECDSA_PUB_ENC\"
    }
]"

\